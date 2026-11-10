// SPDX-License-Identifier: AGPL-3.0-or-later
//! Charged heap requests of the stages that follow the graph: source map,
//! equivalence, extension, capabilities, diagnostics, operation effects and
//! checked-term data, and the owned diagnostics those stages build.
//!
//! Implements: FR-038 "Budget-charged strict read" (stages `SourceMap` to
//! `CheckedTermData`), FR-358.
//!
//! Every request is charged to the caller budget's read-storage counter at
//! `PackageRead(stage, index)` before it is made, with the size of the whole
//! block requested. A meter that holds its own limit (lowering, unit tests)
//! makes the same requests uncharged.
//!
//! A pointer is never built before it is paid for: a site names its position
//! as borrowed [`Step`]s, the exact byte length of the pointer is counted from
//! them, charged, and only then is a block of exactly that length requested.

use super::budgeted_read::{BudgetStop, CheckedPackagePendingOutcome, PackageReadResource};
use super::native_join_budget::{
    NativeJoinBudget, NativeJoinChargeKind, NativeJoinLocus, NativeJoinLocusSource,
    PackageReadStage,
};
use super::{WorkMeter, WorkSource};
use crate::checked_package::common::{Step, Trail, ValidationFailure};
use crate::checked_package::shared::{
    push_index, push_key, CheckedNodeId, CheckedNodePathSegment, CheckedPackageIncomplete,
    CheckedPackageLimit, CheckedPackageRefusal, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageRefusalLocation, JsonPointer,
};
use std::mem::size_of;

/// The locus form of an owned refusal or `incomplete` diagnostic.
const DIAGNOSTIC_FORM: &str = "diagnostic";

/// A position in the package as borrowed RFC 6901 steps, held on the stack.
#[derive(Clone, Copy, Debug)]
pub(super) struct Path<'a> {
    steps: [Step<'a>; Path::CAPACITY],
    len: usize,
}

impl<'a> Path<'a> {
    /// The deepest position a stage names.
    // A binder or control-reference target reaches thirteen closed-shape
    // steps, as `argument_path(2)/members/{entry}/members/0/value/target`
    // does. Arbitrary-depth expressions use Position::Trail instead.
    const CAPACITY: usize = 16;

    /// The document root.
    pub(super) const fn root() -> Self {
        Self {
            steps: [Step::Index(0); Self::CAPACITY],
            len: 0,
        }
    }

    /// The holder of a graph path, inspected as typed steps without building
    /// or parsing an owned diagnostic pointer.
    pub(super) fn graph_node_index(&self) -> Option<usize> {
        match self.steps() {
            [Step::Key("semantic_graph"), Step::Key("nodes"), Step::Index(index), ..] => {
                Some(*index)
            }
            _ => None,
        }
    }

    /// This position one object member below.
    pub(super) fn key(mut self, key: &'a str) -> Self {
        self.steps[self.len] = Step::Key(key);
        self.len += 1;
        self
    }

    /// This position one array element below.
    pub(super) fn index(mut self, index: usize) -> Self {
        self.steps[self.len] = Step::Index(index);
        self.len += 1;
        self
    }

    pub(super) fn steps(&self) -> &[Step<'a>] {
        &self.steps[..self.len]
    }

    /// The exact byte length of the pointer to this position.
    pub(super) fn pointer_len(&self) -> usize {
        self.steps()
            .iter()
            .fold(0_usize, |total, step| total.saturating_add(step_len(step)))
    }

    /// The pointer, in a block of exactly its length. The caller has charged
    /// [`Self::pointer_len`] first.
    pub(super) fn build(&self) -> JsonPointer {
        let length = self.pointer_len();
        let mut text = String::with_capacity(length);
        for step in self.steps() {
            match step {
                Step::Key(key) => push_key(&mut text, key),
                Step::Index(index) => push_index(&mut text, *index),
            }
        }
        debug_assert_eq!(text.capacity(), length);
        JsonPointer::from_escaped(text)
    }
}

fn decimal_digits(mut value: usize) -> usize {
    let mut digits = 1;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits
}

/// The byte length of one step's reference token, its `/` included.
fn step_len(step: &Step<'_>) -> usize {
    match step {
        Step::Key(key) => key.chars().fold(1_usize, |length, character| {
            length.saturating_add(match character {
                '~' | '/' => 2,
                other => other.len_utf8(),
            })
        }),
        Step::Index(index) => 1 + decimal_digits(*index),
    }
}

/// Writes `step`'s token into `buffer`, which is exactly [`step_len`] bytes.
fn write_step(step: &Step<'_>, buffer: &mut [u8]) {
    let Some((slash, rest)) = buffer.split_first_mut() else {
        return;
    };
    *slash = b'/';
    match step {
        Step::Key(key) => {
            let mut at = 0;
            for character in key.chars() {
                let mut encoded = [0_u8; 4];
                let bytes: &[u8] = match character {
                    '~' => b"~0",
                    '/' => b"~1",
                    other => other.encode_utf8(&mut encoded).as_bytes(),
                };
                if let Some(slot) = rest.get_mut(at..at + bytes.len()) {
                    slot.copy_from_slice(bytes);
                }
                at += bytes.len();
            }
        }
        Step::Index(index) => {
            let mut value = *index;
            for slot in rest.iter_mut().rev() {
                *slot = b'0' + (value % 10) as u8;
                value /= 10;
            }
        }
    }
}

/// The exact byte length of the pointer to `trail`, counted along its chain
/// without allocating.
fn trail_pointer_len(trail: &Trail<'_>) -> usize {
    let mut total = 0_usize;
    let mut current = trail;
    loop {
        match current {
            Trail::Base(steps) => {
                return steps
                    .iter()
                    .fold(total, |sum, step| sum.saturating_add(step_len(step)));
            }
            Trail::Child(parent, step) => {
                total = total.saturating_add(step_len(step));
                current = parent;
            }
        }
    }
}

/// The pointer to `trail` in one block of exactly [`trail_pointer_len`] bytes.
/// The block is filled from its end, the chain being read leaf first.
fn build_trail_pointer(trail: &Trail<'_>) -> JsonPointer {
    let length = trail_pointer_len(trail);
    let mut bytes = vec![0_u8; length];
    let mut end = length;
    let mut current = trail;
    loop {
        match current {
            Trail::Base(steps) => {
                for step in steps.iter().rev() {
                    let token = step_len(step);
                    end = end.saturating_sub(token);
                    if let Some(block) = bytes.get_mut(end..end + token) {
                        write_step(step, block);
                    }
                }
                break;
            }
            Trail::Child(parent, step) => {
                let token = step_len(step);
                end = end.saturating_sub(token);
                if let Some(block) = bytes.get_mut(end..end + token) {
                    write_step(step, block);
                }
                current = parent;
            }
        }
    }
    JsonPointer::from_escaped(String::from_utf8(bytes).unwrap_or_default())
}

/// A position a charge or a refusal names: a fixed path, or a chain of borrowed
/// steps of any depth.
// A position is built on the stack at the moment a diagnostic is paid for;
// boxing the stack path would be a heap request of its own.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Copy, Debug)]
pub(super) enum Position<'a> {
    Path(Path<'a>),
    Trail(Trail<'a>),
    /// A path already recorded during the charged reference walk.
    Recorded(&'a JsonPointer),
}

impl<'a> From<Path<'a>> for Position<'a> {
    fn from(path: Path<'a>) -> Self {
        Self::Path(path)
    }
}

impl<'a> From<Trail<'a>> for Position<'a> {
    fn from(trail: Trail<'a>) -> Self {
        Self::Trail(trail)
    }
}

impl Position<'_> {
    /// The exact byte length of the pointer to this position.
    pub(super) fn pointer_len(&self) -> usize {
        match self {
            Self::Path(path) => path.pointer_len(),
            Self::Trail(trail) => trail_pointer_len(trail),
            Self::Recorded(pointer) => pointer.as_str().len(),
        }
    }

    /// The pointer, in a block of exactly its length. The caller has charged
    /// [`Self::pointer_len`] first.
    pub(super) fn build(&self) -> JsonPointer {
        match self {
            Self::Path(path) => path.build(),
            Self::Trail(trail) => build_trail_pointer(trail),
            Self::Recorded(pointer) => (*pointer).clone(),
        }
    }
}

/// Where a stage's next request is charged: its locus and the position of the
/// value being read, which a denied request reports.
#[derive(Clone, Copy, Debug)]
pub(super) struct Site<'a> {
    pub(super) stage: PackageReadStage,
    pub(super) index: usize,
    pub(super) form: &'static str,
    pub(super) at: Position<'a>,
}

impl<'a> Site<'a> {
    pub(super) fn new(
        stage: PackageReadStage,
        index: usize,
        form: &'static str,
        at: impl Into<Position<'a>>,
    ) -> Self {
        Self {
            stage,
            index,
            form,
            at: at.into(),
        }
    }

    fn locus(&self, form: &'static str) -> NativeJoinLocus {
        NativeJoinLocus {
            source: NativeJoinLocusSource::PackageRead(self.stage, self.index),
            form,
        }
    }
}

/// The caller's budget behind a meter, when the meter has one.
fn budget<'m>(meter: &'m mut WorkMeter<'_>) -> Option<&'m mut NativeJoinBudget> {
    match &mut meter.source {
        WorkSource::Budget(budget) => Some(&mut **budget),
        WorkSource::Own { .. } => None,
    }
}

/// The failure a denied charge of `amount` of `resource` at `site` becomes. Its
/// pointer is the diagnostic the budgeted outcome owns: paid for here, before
/// the block is requested, so the outcome's own charge is already admitted.
fn denied(
    budget: &mut NativeJoinBudget,
    site: &Site<'_>,
    resource: PackageReadResource,
    amount: u128,
) -> ValidationFailure {
    let length = site.at.pointer_len();
    let paid = budget.prepay_diagnostic(length as u128);
    let path = (paid && length > 0).then(|| site.at.build());
    ValidationFailure::Budget(BudgetStop {
        resource,
        amount,
        path,
    })
}

/// The failure of a request of `bytes` that the allocator could not provide.
pub(super) fn unavailable(bytes: usize) -> ValidationFailure {
    ValidationFailure::Budget(BudgetStop {
        resource: PackageReadResource::ReadStorage,
        amount: bytes as u128,
        path: None,
    })
}

/// Admits `bytes` of heap at `site` before it is requested.
pub(super) fn reserve(
    meter: &mut WorkMeter<'_>,
    site: &Site<'_>,
    bytes: usize,
) -> Result<(), ValidationFailure> {
    if bytes == 0 {
        return Ok(());
    }
    let Some(budget) = budget(meter) else {
        return Ok(());
    };
    let previous = budget.active_locus();
    budget.set_active_locus(Some(site.locus(site.form)));
    let admitted = budget.charge_read_storage(bytes as u128);
    budget.set_active_locus(previous);
    if admitted {
        Ok(())
    } else {
        Err(denied(
            budget,
            site,
            PackageReadResource::ReadStorage,
            bytes as u128,
        ))
    }
}

/// Admits `count` items of `T` before a block for them is requested.
pub(super) fn reserve_items<T>(
    meter: &mut WorkMeter<'_>,
    site: &Site<'_>,
    count: usize,
) -> Result<(), ValidationFailure> {
    reserve(meter, site, count.saturating_mul(size_of::<T>()))
}

/// An empty vector with room for exactly `capacity` items, charged first.
pub(super) fn vec<T>(
    meter: &mut WorkMeter<'_>,
    site: &Site<'_>,
    capacity: usize,
) -> Result<Vec<T>, ValidationFailure> {
    reserve_items::<T>(meter, site, capacity)?;
    Ok(Vec::with_capacity(capacity))
}

/// The position of the first of `keys` that repeats an earlier one, found on a
/// sorted table of exactly `keys.len()` entries charged at `site` first.
pub(super) fn first_repeat<K: Ord>(
    meter: &mut WorkMeter<'_>,
    site: &Site<'_>,
    keys: impl ExactSizeIterator<Item = K>,
) -> Result<Option<usize>, ValidationFailure> {
    let mut table: Vec<(K, usize)> = vec(meter, site, keys.len())?;
    table.extend(keys.enumerate().map(|(position, key)| (key, position)));
    table.sort_unstable();
    Ok(table
        .windows(2)
        .filter(|pair| pair[0].0 == pair[1].0)
        .map(|pair| pair[1].1)
        .min())
}

/// Charges `work` units to the budget's work counter for the value at `site`.
/// A denial reports the pointer, built after it is paid for.
pub(super) fn work(
    meter: &mut WorkMeter<'_>,
    site: &Site<'_>,
    work: u64,
) -> Result<(), ValidationFailure> {
    let Some(budget) = budget(meter) else {
        return meter.charge(work, || site.at.build());
    };
    let previous = budget.active_locus();
    budget.set_active_locus(Some(site.locus(site.form)));
    let admitted = budget.charge(NativeJoinChargeKind::Work, u128::from(work));
    budget.set_active_locus(previous);
    if admitted {
        Ok(())
    } else {
        Err(denied(
            budget,
            site,
            PackageReadResource::Work,
            u128::from(work),
        ))
    }
}

/// Admits the owned diagnostic of an outcome: `bytes` at `site`, form
/// `"diagnostic"`. A denial is the read's `exhausted` outcome, with `pending`
/// the outcome being built.
fn diagnostic(
    meter: &mut WorkMeter<'_>,
    site: &Site<'_>,
    bytes: usize,
    pending: CheckedPackagePendingOutcome,
) -> Result<(), ValidationFailure> {
    let Some(budget) = budget(meter) else {
        return Ok(());
    };
    let previous = budget.active_locus();
    budget.set_active_locus(Some(site.locus(DIAGNOSTIC_FORM)));
    let admitted = budget.charge_read_storage(bytes as u128);
    budget.set_active_locus(previous);
    if admitted {
        budget.set_paid(pending);
        Ok(())
    } else {
        budget.set_pending(Some(pending));
        Err(ValidationFailure::Budget(BudgetStop {
            resource: PackageReadResource::ReadStorage,
            amount: bytes as u128,
            path: None,
        }))
    }
}

/// The heap a clone of `id` requests: its package name, its path and each
/// name in it. Shared children are counted by reference and request nothing.
pub(super) fn node_id_clone_bytes(id: &CheckedNodeId) -> usize {
    id.path.iter().fold(
        id.package.len().saturating_add(
            id.path
                .len()
                .saturating_mul(size_of::<CheckedNodePathSegment>()),
        ),
        |total, segment| match segment {
            CheckedNodePathSegment::Name(name) => total.saturating_add(name.len()),
            CheckedNodePathSegment::Dependency { package, .. } => {
                total.saturating_add(package.len())
            }
            _ => total,
        },
    )
}

/// The canonical writer's reservation, charged to a meter at one site. A
/// denial leaves its failure here for the caller.
pub(super) struct WriterCharges<'a, 'm, 'b> {
    meter: &'a mut WorkMeter<'b>,
    site: Site<'m>,
    pub(super) failure: Option<ValidationFailure>,
}

impl<'a, 'm, 'b> WriterCharges<'a, 'm, 'b> {
    pub(super) fn new(meter: &'a mut WorkMeter<'b>, site: Site<'m>) -> Self {
        Self {
            meter,
            site,
            failure: None,
        }
    }
}

impl quire_canonical::Reservation for WriterCharges<'_, '_, '_> {
    fn reserve(&mut self, bytes: usize) -> Result<(), quire_canonical::ReservationDenied> {
        match reserve(self.meter, &self.site, bytes) {
            Ok(()) => Ok(()),
            Err(failure) => {
                self.failure = Some(failure);
                Err(quire_canonical::ReservationDenied)
            }
        }
    }
}

/// A canonical sink that keeps nothing and counts what it is given.
#[derive(Default)]
pub(super) struct CountingSink(pub(super) usize);

impl quire_canonical::Sink for CountingSink {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), quire_canonical::Error> {
        self.0 = self.0.saturating_add(bytes.len());
        Ok(())
    }
}

/// What a refusal is about.
#[derive(Clone, Copy)]
pub(super) struct Refusal<'a> {
    pub(super) code: CheckedPackageRefusalCode,
    pub(super) cause: Option<CheckedPackageRefusalCause>,
    pub(super) path: Position<'a>,
    /// The node a refusal is located at; its clone is part of the diagnostic.
    pub(super) locus: Option<&'a CheckedNodeId>,
    /// Heap the caller adds to the outcome after it is admitted (a detail).
    pub(super) extra: usize,
    /// The refusal of an extension node whose selected definition is not held,
    /// which carries its pointer twice.
    pub(super) unheld_definition: bool,
    /// The package-document position the refusal also names.
    pub(super) document_pointer: Option<Position<'a>>,
}

impl<'a> Refusal<'a> {
    pub(super) fn new(code: CheckedPackageRefusalCode, path: impl Into<Position<'a>>) -> Self {
        Self {
            code,
            cause: None,
            path: path.into(),
            locus: None,
            extra: 0,
            unheld_definition: false,
            document_pointer: None,
        }
    }

    /// Also names the package-document position `pointer`.
    pub(super) fn in_document(mut self, pointer: impl Into<Position<'a>>) -> Self {
        self.document_pointer = Some(pointer.into());
        self
    }

    /// The `unknown_required_capability` refusal of the extension node `locus`
    /// whose selected definition is not held, at `path`.
    pub(super) fn unheld_definition(
        path: impl Into<Position<'a>>,
        locus: &'a CheckedNodeId,
    ) -> Self {
        let mut refusal = Self::new(CheckedPackageRefusalCode::UnknownRequiredCapability, path);
        refusal.locus = Some(locus);
        refusal.unheld_definition = true;
        refusal
    }

    /// Adds `bytes` of heap the caller builds into the outcome.
    pub(super) fn with_extra(mut self, bytes: usize) -> Self {
        self.extra = self.extra.saturating_add(bytes);
        self
    }

    pub(super) fn because(mut self, cause: CheckedPackageRefusalCause) -> Self {
        self.cause = Some(cause);
        self
    }

    pub(super) fn at_node(mut self, locus: &'a CheckedNodeId) -> Self {
        self.locus = Some(locus);
        self
    }

    /// The heap the refusal owns: the boxed record, its one location, the
    /// pointer and the located node's clone.
    fn bytes(&self) -> usize {
        size_of::<CheckedPackageRefusal>()
            .saturating_add(size_of::<CheckedPackageRefusalLocation>())
            .saturating_add(self.path.pointer_len())
            .saturating_add(if self.unheld_definition {
                self.path.pointer_len()
            } else {
                0
            })
            .saturating_add(self.locus.map_or(0, node_id_clone_bytes))
            .saturating_add(
                self.document_pointer
                    .map_or(0, |pointer| pointer.pointer_len()),
            )
            .saturating_add(self.extra)
    }

    /// Builds only after the caller has admitted the complete diagnostic.
    fn build(self) -> ValidationFailure {
        let path = self.path.build();
        if let (true, Some(locus)) = (self.unheld_definition, self.locus) {
            return ValidationFailure::unknown_extension_definition(path, locus.clone());
        }
        let mut failure = match (self.cause, self.locus) {
            (cause, Some(locus)) => {
                ValidationFailure::refused_at(self.code, path, cause, locus.clone())
            }
            (Some(cause), None) => ValidationFailure::refused_because(self.code, path, cause),
            (None, None) => ValidationFailure::refused(self.code, path),
        };
        if let (ValidationFailure::Refused(refusal), Some(pointer)) =
            (&mut failure, self.document_pointer)
        {
            refusal.document_pointer = Some(pointer.build());
        }
        failure
    }
}

/// The refusal, its owned diagnostic charged to the budget before any of it is
/// built.
pub(super) fn refuse(
    meter: &mut WorkMeter<'_>,
    site: &Site<'_>,
    refusal: Refusal<'_>,
) -> ValidationFailure {
    if let Err(stop) = diagnostic(
        meter,
        site,
        refusal.bytes(),
        CheckedPackagePendingOutcome::Refused(refusal.code),
    ) {
        return stop;
    }
    refusal.build()
}

/// A walk's refusal, paid through its existing caller-owned diagnostic sink
/// before the pointer, record, location or node clone is built.
pub(super) fn refuse_charged(
    meter: &dyn super::read_charge::ChargeAt,
    site: super::read_charge::Site,
    refusal: Refusal<'_>,
) -> ValidationFailure {
    if meter
        .reserve_diagnostic(
            site,
            refusal.bytes() as u128,
            CheckedPackagePendingOutcome::Refused(refusal.code),
        )
        .is_err()
    {
        return meter.denial();
    }
    refusal.build()
}

/// The position of `/semantic_graph/nodes/{position}` on the stack.
pub(super) fn node_path(position: usize) -> Path<'static> {
    Path::root()
        .key("semantic_graph")
        .key("nodes")
        .index(position)
}

/// A refusal located at `node`, its whole owned diagnostic charged before any
/// of it is built.
pub(super) fn refuse_at<'a>(
    meter: &mut WorkMeter<'_>,
    code: CheckedPackageRefusalCode,
    cause: Option<CheckedPackageRefusalCause>,
    at: impl Into<Position<'a>>,
    node: &'a CheckedNodeId,
) -> ValidationFailure {
    let at = at.into();
    let site = super::natural::site(meter, at);
    let mut refusal = Refusal::new(code, at).at_node(node);
    refusal.cause = cause;
    refuse(meter, &site, refusal)
}

/// A refusal about the value at `at`, not located at any node.
pub(super) fn refuse_unlocated<'a>(
    meter: &mut WorkMeter<'_>,
    code: CheckedPackageRefusalCode,
    at: impl Into<Position<'a>>,
) -> ValidationFailure {
    let at = at.into();
    let site = super::natural::site(meter, at);
    refuse(meter, &site, Refusal::new(code, at))
}

/// Charges one unit of work at `at`; a denial pays for its pointer first.
pub(super) fn charge_at(meter: &mut WorkMeter<'_>, at: Path<'_>) -> Result<(), ValidationFailure> {
    charge_amount_at(meter, at, 1)
}

/// Charges `amount` units of work at `at`; a denial pays for its pointer first.
pub(super) fn charge_amount_at(
    meter: &mut WorkMeter<'_>,
    at: Path<'_>,
    amount: u64,
) -> Result<(), ValidationFailure> {
    let site = super::natural::site(meter, at);
    super::natural::work_at(meter, site, amount)
}

/// Moves a pointer whose String buffer was already paid by the canonical
/// reader. Only new record/location storage and an actual String-to-Box shrink
/// request are charged here; the original buffer is never charged twice.
pub(super) fn refuse_held_pointer(
    meter: &mut WorkMeter<'_>,
    site: &Site<'_>,
    code: CheckedPackageRefusalCode,
    pointer: String,
) -> ValidationFailure {
    let shrink = if pointer.capacity() == pointer.len() {
        0
    } else {
        pointer.len()
    };
    let bytes = size_of::<CheckedPackageRefusal>()
        .saturating_add(size_of::<CheckedPackageRefusalLocation>())
        .saturating_add(shrink);
    if let Err(stop) = diagnostic(
        meter,
        site,
        bytes,
        CheckedPackagePendingOutcome::Refused(code),
    ) {
        return stop;
    }
    ValidationFailure::refused(code, JsonPointer::from_escaped(pointer))
}

/// Refuses at a scanner-built pointer, charging its bytes and the owned
/// refusal diagnostic to the caller's budget.
pub(super) fn refuse_pointer(
    meter: &mut WorkMeter<'_>,
    site: &Site<'_>,
    code: CheckedPackageRefusalCode,
    pointer: JsonPointer,
) -> ValidationFailure {
    let bytes = size_of::<CheckedPackageRefusal>()
        .saturating_add(size_of::<CheckedPackageRefusalLocation>())
        .saturating_add(pointer.as_str().len());
    if let Err(stop) = diagnostic(
        meter,
        site,
        bytes,
        CheckedPackagePendingOutcome::Refused(code),
    ) {
        return stop;
    }
    ValidationFailure::refused(code, pointer)
}

/// A structural `incomplete` at `path`, its pointer charged before it is built.
pub(super) fn incomplete(
    meter: &mut WorkMeter<'_>,
    site: &Site<'_>,
    limit_kind: CheckedPackageLimit,
    limit: u64,
    consumed: u64,
    path: Path<'_>,
) -> ValidationFailure {
    if let Err(stop) = diagnostic(
        meter,
        site,
        path.pointer_len(),
        CheckedPackagePendingOutcome::Incomplete(limit_kind),
    ) {
        return stop;
    }
    ValidationFailure::Incomplete(CheckedPackageIncomplete {
        limit_kind,
        limit,
        consumed,
        path: Some(path.build()),
    })
}

/// Pays the boxed byte-stream refusal before constructing it, without a path.
pub(super) fn refuse_bytes(
    meter: &mut WorkMeter<'_>,
    site: &Site<'_>,
    code: CheckedPackageRefusalCode,
) -> ValidationFailure {
    if let Err(stop) = diagnostic(
        meter,
        site,
        size_of::<CheckedPackageRefusal>() + size_of::<CheckedPackageRefusalLocation>(),
        CheckedPackagePendingOutcome::Refused(code),
    ) {
        return stop;
    }
    ValidationFailure::refused_bytes(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::evidence::CheckedPackageEvidence;
    use crate::checked_package::v2::{
        test_support::node_id, CheckedNodeKind, ExtensionForm, RawCheckedPackageWireV2, ValueForm,
    };
    use ix_trace_rs::trace;
    use serde_json::{json, Value};

    fn wire(report: &[&str], required: &[&str]) -> RawCheckedPackageWireV2 {
        let definition = json!({"authority": "test", "identity": "stage"});
        let report: Vec<Value> = report
            .iter()
            .map(|feature| json!({"feature": feature, "disposition": "available"}))
            .collect();
        serde_json::from_value(json!({
            "contract_version": super::super::CHECKED_PACKAGE_V2,
            "name": "test/stage",
            "lock": {"sources": [], "edition": {"role": "edition", "definition": definition},
                "profile_selections": [], "definition_selections": [definition],
                "required_features": required, "dependency_selections": []},
            "semantic_graph": {"graph_version": "quire.checked-semantic-graph/v2",
                "nodes": [], "operand_rows": []},
            "source_map": [], "capability_report": report,
            "diagnostics": {"catalog": definition, "entries": []}
        }))
        .expect("private stage wire")
    }

    fn budget(limit: u64) -> NativeJoinBudget {
        NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit)
    }

    #[trace("TC-472", "FR-038-AC-510")]
    #[test]
    fn graph_holder_prefix_uses_typed_steps_without_pointer_escaping() {
        let graph = Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(17)
            .key("body")
            .key("/~é");
        let measured =
            allocation_counter::measure(|| assert_eq!(graph.graph_node_index(), Some(17)));
        assert_eq!(measured.bytes_total, 0);
        assert_eq!(
            Path::root()
                .key("semantic_graph")
                .key("nodes")
                .graph_node_index(),
            None
        );
        assert_eq!(
            Path::root()
                .key("semantic_graph/nodes")
                .index(17)
                .graph_node_index(),
            None
        );
        assert_eq!(
            Path::root()
                .key("semantic_graph")
                .key("nodes/~")
                .index(17)
                .graph_node_index(),
            None
        );
        assert_eq!(
            Path::root().key("diagnostics").index(17).graph_node_index(),
            None
        );
    }

    #[trace("TC-472", "FR-038-AC-510")]
    #[test]
    fn closed_task_value_path_preserves_all_twelve_steps() {
        let at = Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(17)
            .key("body")
            .key("members")
            .index(4)
            .key("value")
            .key("members")
            .index(2)
            .key("members")
            .index(5)
            .key("value");
        let expected = "/semantic_graph/nodes/17/body/members/4/value/members/2/members/5/value";
        assert_eq!(at.pointer_len(), expected.len());
        assert_eq!(at.build().as_str(), expected);
    }

    #[trace("TC-472", "FR-038-AC-505")]
    #[test]
    fn pointer_length_counts_escapes_and_digits_and_the_block_has_exactly_that_length() {
        let at = Path::root()
            .key("a~b")
            .key("c/d")
            .index(0)
            .index(10)
            .key("é");
        // "/a~0b" + "/c~1d" + "/0" + "/10" + "/é"
        let expected = "/a~0b/c~1d/0/10/é";
        assert_eq!(at.pointer_len(), expected.len());
        let built = at.build();
        assert_eq!(built.as_str(), expected);
        assert_eq!(Path::root().pointer_len(), 0);
        assert_eq!(Path::root().build().as_str(), "");
    }

    #[trace("TC-472", "FR-038-AC-505")]
    #[test]
    fn built_pointer_is_one_block_of_exactly_its_charged_length() {
        let at = Path::root().key("source_map").index(12345).key("regions");
        let length = at.pointer_len();
        let measured = allocation_counter::measure(|| drop(at.build()));
        assert_eq!(measured.bytes_total, length as u64);
        assert_eq!(measured.count_total, 1, "no reallocation while it is built");
    }

    #[trace("TC-472", "FR-038-AC-505")]
    #[test]
    fn capability_table_request_equals_the_bytes_allocated() {
        let wire = wire(&["b", "a", "c"], &[]);
        let mut budget = budget(u64::MAX);
        let mut result = None;
        let measured = allocation_counter::measure(|| {
            let mut meter = WorkMeter::over(&mut budget);
            result = Some(super::super::validate_capabilities(
                &wire,
                &CheckedPackageEvidence::new(),
                &mut meter,
            ));
        });
        assert!(result.expect("ran").is_ok());
        let table = 3 * size_of::<(&str, usize, super::super::CheckedCapabilityDisposition)>();
        assert_eq!(budget.read_storage_consumed(), table as u64);
        assert_eq!(measured.bytes_total, table as u64);
    }

    #[trace("TC-472", "FR-038-AC-505")]
    #[test]
    fn capability_table_one_byte_short_is_denied_at_its_locus_and_the_exact_allowance_admits() {
        let wire = wire(&["b", "a", "c"], &[]);
        let table =
            (3 * size_of::<(&str, usize, super::super::CheckedCapabilityDisposition)>()) as u64;
        let mut exact = budget(table);
        let mut meter = WorkMeter::over(&mut exact);
        assert!(super::super::validate_capabilities(
            &wire,
            &CheckedPackageEvidence::new(),
            &mut meter
        )
        .is_ok());
        let mut short = budget(table - 1);
        let mut meter = WorkMeter::over(&mut short);
        let stop =
            super::super::validate_capabilities(&wire, &CheckedPackageEvidence::new(), &mut meter)
                .expect_err("one byte short is denied");
        let ValidationFailure::Budget(stop) = stop else {
            panic!("a budget denial: {stop:?}");
        };
        assert_eq!(stop.resource, PackageReadResource::ReadStorage);
        assert_eq!(stop.amount, u128::from(table));
        let locus = short.first_unpaid_locus().expect("locus");
        assert_eq!(
            (locus.source, locus.form),
            (
                NativeJoinLocusSource::PackageRead(PackageReadStage::Capabilities, 0),
                "capability_report"
            )
        );
        // The denied table consumes nothing; the pointer the outcome owns is
        // paid for before it is built.
        assert_eq!(
            stop.path.as_ref().map(JsonPointer::as_str),
            Some("/capability_report")
        );
        assert_eq!(
            short.read_storage_consumed(),
            "/capability_report".len() as u64
        );
    }

    #[trace("TC-472", "FR-038-AC-505")]
    #[test]
    fn repeated_capability_refusal_charges_its_diagnostic_before_building_it() {
        let wire = wire(&["a", "b", "a"], &[]);
        let table =
            (3 * size_of::<(&str, usize, super::super::CheckedCapabilityDisposition)>()) as u64;
        let pointer = "/capability_report/2/feature";
        let diagnostic = (size_of::<CheckedPackageRefusal>()
            + size_of::<CheckedPackageRefusalLocation>()
            + pointer.len()) as u64;
        // The exact allowance builds the refusal; the diagnostic's own request
        // is the one a byte short denies.
        let mut exact = budget(table + diagnostic);
        let mut meter = WorkMeter::over(&mut exact);
        let measured = allocation_counter::measure(|| {
            let ValidationFailure::Refused(refusal) = super::super::validate_capabilities(
                &wire,
                &CheckedPackageEvidence::new(),
                &mut meter,
            )
            .expect_err("a repeated feature refuses") else {
                panic!("a refusal");
            };
            assert_eq!(
                refusal.loci[0].path.as_ref().map(JsonPointer::as_str),
                Some(pointer)
            );
            std::mem::forget(refusal);
        });
        assert_eq!(measured.bytes_total, table + diagnostic);
        assert_eq!(exact.read_storage_consumed(), table + diagnostic);

        let mut short = budget(table + diagnostic - 1);
        let mut meter = WorkMeter::over(&mut short);
        let stop =
            super::super::validate_capabilities(&wire, &CheckedPackageEvidence::new(), &mut meter)
                .expect_err("the diagnostic is denied");
        assert!(matches!(stop, ValidationFailure::Budget(_)), "{stop:?}");
        let locus = short.first_unpaid_locus().expect("locus");
        assert_eq!(
            (locus.source, locus.form),
            (
                NativeJoinLocusSource::PackageRead(PackageReadStage::Capabilities, 2),
                "diagnostic"
            )
        );
        assert_eq!(
            short.take_pending(),
            Some(CheckedPackagePendingOutcome::Refused(
                CheckedPackageRefusalCode::UnknownRequiredCapability
            ))
        );
    }

    #[trace("TC-472", "FR-038-AC-510")]
    #[test]
    fn work_denial_builds_its_pointer_only_after_the_pointer_is_paid_for() {
        let at = Path::root().key("operation_effects").index(0);
        let site = Site::new(
            PackageReadStage::OperationEffects,
            0,
            "operation_effects",
            at,
        );
        let pointer = "/operation_effects/0".len() as u64;
        let deny = |storage: u64| {
            let mut budget = NativeJoinBudget::new(0, 0).with_read_storage_limit(storage);
            let mut stop = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                stop = Some(work(&mut meter, &site, 1).expect_err("no work is allowed"));
            });
            (stop.expect("ran"), measured.bytes_total, budget)
        };
        // One byte short of the pointer: nothing is allocated, and the denial
        // is reported without a pointer.
        let (stop, allocated, mut budget) = deny(pointer - 1);
        let ValidationFailure::Budget(stop) = stop else {
            panic!("a budget denial");
        };
        assert_eq!((allocated, stop.path), (0, None));
        assert_eq!(budget.take_prepaid_diagnostic(), Some(false));
        // The exact allowance pays, then builds, exactly the pointer.
        let (stop, allocated, mut budget) = deny(pointer);
        let ValidationFailure::Budget(stop) = stop else {
            panic!("a budget denial");
        };
        assert_eq!(allocated, pointer);
        assert_eq!(
            stop.path.as_ref().map(JsonPointer::as_str),
            Some("/operation_effects/0")
        );
        assert_eq!(budget.read_storage_consumed(), pointer);
        assert_eq!(budget.take_prepaid_diagnostic(), Some(true));
    }

    /// A graph of three nodes, the third an extension, with a source-map entry
    /// of two regions for the extension.
    fn extension_with_two_regions() -> (RawCheckedPackageWireV2, Vec<CheckedNodeKind>) {
        let mut wire = wire(&[], &[]);
        let node = |id: char, kind: CheckedNodeKind| {
            serde_json::from_value(json!({
                "node_id": node_id(id), "schema_version": "quire.checked-semantic-graph/v2",
                "node_tag": kind.tag().as_wire(), "semantic_form": kind.form_wire(),
                "semantic_type": node_id(id), "dependencies": [], "occurrences": [],
                "body": {"term": "aggregate", "members": []}
            }))
            .expect("private graph node")
        };
        let kinds = vec![
            CheckedNodeKind::Value(ValueForm::Parameter),
            CheckedNodeKind::Extension(ExtensionForm::ExtensionForm),
        ];
        wire.semantic_graph.nodes = vec![node('0', kinds[0]), node('1', kinds[1])];
        let source = json!({"authority": "a", "identity": "i"});
        wire.source_map = serde_json::from_value(json!([{
            "node_id": node_id('1'), "role": "declaration", "ordinal": 0,
            "regions": [{"source": source, "start": 0, "end": 1},
                {"source": source, "start": 2, "end": 3}]
        }]))
        .expect("source map");
        (wire, kinds)
    }

    #[trace("TC-472", "FR-038-AC-505")]
    #[test]
    fn extension_region_table_is_denied_one_byte_short_and_a_refusal_follows_the_exact_one() {
        let (wire, kinds) = extension_with_two_regions();
        let table = size_of::<(&CheckedNodeId, usize)>() as u64;
        let mut short = budget(table - 1);
        let mut meter = WorkMeter::over(&mut short);
        let stop = super::super::extension::validate_source_regions(&wire, &kinds, &mut meter)
            .expect_err("one byte short is denied");
        assert!(matches!(stop, ValidationFailure::Budget(_)), "{stop:?}");
        let locus = short.first_unpaid_locus().expect("locus");
        assert_eq!(
            (locus.source, locus.form),
            (
                NativeJoinLocusSource::PackageRead(PackageReadStage::Extension, 0),
                "extension"
            )
        );
        let mut unlimited = budget(u64::MAX);
        let mut meter = WorkMeter::over(&mut unlimited);
        let ValidationFailure::Refused(refusal) =
            super::super::extension::validate_source_regions(&wire, &kinds, &mut meter)
                .expect_err("two regions on an extension refuse")
        else {
            panic!("a refusal");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidSourceMap);
        assert_eq!(refusal.locus.as_ref(), Some(&node_id('1')));
        let pointer = "/source_map/0/regions";
        let located = node_id_clone_bytes(&node_id('1'));
        assert_eq!(
            unlimited.read_storage_consumed(),
            table
                + (size_of::<CheckedPackageRefusal>()
                    + size_of::<CheckedPackageRefusalLocation>()
                    + pointer.len()
                    + located) as u64
        );
    }

    /// The oracle for the stages that order node ids: the bytes allocated equal
    /// the bytes requested once identity comparison allocates nothing.
    #[trace("TC-472", "FR-038-AC-505")]
    #[test]
    fn extension_stage_allocates_exactly_the_bytes_it_requests() {
        let (mut wire, kinds) = extension_with_two_regions();
        wire.source_map[0].regions.truncate(1);
        let mut budget = budget(u64::MAX);
        let mut meter = WorkMeter::over(&mut budget);
        let measured = allocation_counter::measure(|| {
            super::super::extension::validate_source_regions(&wire, &kinds, &mut meter)
                .expect("one region admits");
        });
        assert_eq!(measured.bytes_total, budget.read_storage_consumed());
    }
}
