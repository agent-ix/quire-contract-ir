// SPDX-License-Identifier: AGPL-3.0-or-later
//! The budget-charged strict V2 read: its entries, outcomes and the package
//! envelope's read and canonical comparison under the caller's budget.
//!
//! Implements: FR-038 "Budget-charged strict read", FR-358.

use super::native_join_budget::{NativeJoinBudget, NativeJoinLocus, NativeJoinLocusSource};
use super::{
    CatalogReadError, CheckedPackageEvidence, CheckedPackageV2, CheckedPackageV2ReadResult,
    PackageReadStage, ValidationFailure,
};
use crate::checked_package::common::{
    exceeds, refuse_bytes_exceeded, Step, Trail, JSON_NESTING_LIMIT,
};
use crate::checked_package::shared::{
    CheckedPackageIncomplete, CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCode, JsonPointer,
};
use ix_trace_rs::trace;
use quire_canonical::{Document, ReadError, Reservation, ReservationDenied};

/// The budget counter a strict read ran out of.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackageReadResource {
    /// Validation work units.
    Work,
    /// Bytes of heap requested by the read.
    ReadStorage,
}

/// A charge the budget could not admit: its resource and full attempted amount.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackageReadDeniedCharge {
    /// The resource that ran out.
    pub resource: PackageReadResource,
    /// The full amount that was attempted.
    pub amount: u128,
}

/// A budgeted read that stopped at its first unpaid charge, with the owned
/// diagnostic affordable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedPackageBudgetIncomplete {
    /// The first unpaid charge.
    pub denied: PackageReadDeniedCharge,
    /// The budget's configured limit for the denied resource.
    pub limit: u64,
    /// Cumulative admitted consumption of the resource plus the denied amount.
    pub consumed: u64,
    /// The RFC 6901 pointer of the value whose charge failed, when a package
    /// value was being decoded or validated.
    pub path: Option<JsonPointer>,
}

/// The fixed-size class of the outcome a read was building when its budget ran
/// out.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedPackagePendingOutcome {
    /// A refusal with this code.
    Refused(CheckedPackageRefusalCode),
    /// A structural `incomplete` of this kind.
    Incomplete(CheckedPackageLimit),
}

/// A budgeted read whose budget could not pay for the diagnostic of its own
/// outcome. It owns no pointer, locus list or string.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedPackageReadExhausted {
    /// The original first unpaid charge.
    pub denied: PackageReadDeniedCharge,
    /// The configured limit of the denied resource, including zero.
    pub configured_bound: u64,
    /// Where the denied charge was made.
    pub locus: NativeJoinLocus,
    /// The outcome being built, if any.
    pub pending: Option<CheckedPackagePendingOutcome>,
}

/// The closed result of a budgeted strict V2 read.
// The inline typed refusal makes this larger than its other cases; boxing it
// would change the unbudgeted result's shape this one mirrors.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedPackageV2BudgetedReadResult {
    /// A complete V2 package is safe for lowering.
    Admitted(Box<CheckedPackageV2>),
    /// The input is invalid for V2.
    Refused(CheckedPackageRefusal),
    /// The input exceeded a structural cap or the byte limit.
    Incomplete(CheckedPackageIncomplete),
    /// The budget ran out; the diagnostic was affordable.
    BudgetIncomplete(CheckedPackageBudgetIncomplete),
    /// The budget ran out and could not pay for the diagnostic.
    Exhausted(CheckedPackageReadExhausted),
    /// The reader's operation catalog is unreadable; no input was judged.
    CatalogUnavailable(CatalogReadError),
}

/// The closed result of dispatching bytes through the budgeted read.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedPackageBudgetedDispatchResult {
    /// A `quire.checked-package/v2` package admitted by the V2 reader.
    AdmittedV2(Box<CheckedPackageV2>),
    /// The input is invalid for its selected version, or selects none.
    Refused(CheckedPackageRefusal),
    /// The input exceeded a structural cap or the byte limit.
    Incomplete(CheckedPackageIncomplete),
    /// The budget ran out; the diagnostic was affordable.
    BudgetIncomplete(CheckedPackageBudgetIncomplete),
    /// The budget ran out and could not pay for the diagnostic.
    Exhausted(CheckedPackageReadExhausted),
    /// The reader's operation catalog is unreadable; no input was judged.
    CatalogUnavailable(CatalogReadError),
}

/// A budget denial inside the read, before its outcome is built.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::checked_package) struct BudgetStop {
    pub(in crate::checked_package) resource: PackageReadResource,
    pub(in crate::checked_package) amount: u128,
    /// The pointer of the value whose charge failed, when there is one.
    pub(in crate::checked_package) path: Option<JsonPointer>,
}

impl BudgetStop {
    /// The structural `incomplete` a reader that holds no caller budget (the
    /// FR-035 contract-package reader, lowering) reports for this stop. Those
    /// readers meter against their own `work` limit and never build one, so
    /// this only keeps their matches over the read outcomes closed.
    pub(in crate::checked_package) fn without_budget(self) -> CheckedPackageIncomplete {
        CheckedPackageIncomplete {
            limit_kind: CheckedPackageLimit::Work,
            limit: 0,
            consumed: u64::try_from(self.amount).unwrap_or(u64::MAX),
            path: self.path,
        }
    }
}

/// The locus form of an owned refusal or `incomplete` diagnostic.
const DIAGNOSTIC_FORM: &str = "diagnostic";

impl CheckedPackageV2 {
    /// Reads one canonical V2 value under the caller's budget.
    ///
    /// This is the one read implementation: every work step and every heap
    /// request is charged to `budget` before it is made, and `limits.work` is
    /// not consulted. A refused, incomplete or exhausted read keeps its
    /// admitted charges in the budget's cumulative consumption.
    #[trace("FR-038-AC-495", "FR-038-AC-496")]
    pub fn read_with_budget(
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
        budget: &mut NativeJoinBudget,
    ) -> CheckedPackageV2BudgetedReadResult {
        let work_before = budget.consumed();
        let storage_before = budget.read_storage_mark();
        budget.resume();
        if exceeds(bytes.len(), limits.bytes) {
            budget.abort();
            return budgeted_outcome(refuse_bytes_exceeded(limits.bytes, bytes.len()), budget);
        }
        let catalog = match super::operation_catalog::read_catalog_budgeted(budget) {
            Ok(catalog) => catalog,
            Err(super::operation_catalog_index::CatalogStop::Unavailable(error)) => {
                budget.abort();
                return CheckedPackageV2BudgetedReadResult::CatalogUnavailable(error);
            }
            Err(super::operation_catalog_index::CatalogStop::Budget(stop)) => {
                budget.abort();
                return budgeted_outcome(stop, budget);
            }
        };
        Self::admit_with_catalog(
            catalog,
            bytes,
            limits,
            evidence,
            budget,
            work_before,
            storage_before,
        )
    }

    /// [`Self::read_with_budget`] against the given result of reading the
    /// operation catalog: an unreadable catalog returns before any charge.
    #[cfg(test)]
    pub(super) fn read_with_budget_against(
        catalog: &'static Result<super::operation_catalog::OperationCatalog, CatalogReadError>,
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
        budget: &mut NativeJoinBudget,
    ) -> CheckedPackageV2BudgetedReadResult {
        let catalog = match super::operation_catalog::available(catalog) {
            Ok(catalog) => catalog,
            Err(error) => return CheckedPackageV2BudgetedReadResult::CatalogUnavailable(error),
        };
        let work_before = budget.consumed();
        let storage_before = budget.read_storage_mark();
        budget.resume();
        Self::admit_with_catalog(
            catalog,
            bytes,
            limits,
            evidence,
            budget,
            work_before,
            storage_before,
        )
    }

    fn admit_with_catalog(
        catalog: super::operation_catalog::CatalogRef,
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
        budget: &mut NativeJoinBudget,
        work_before: super::NativeJoinCharge,
        storage_before: u64,
    ) -> CheckedPackageV2BudgetedReadResult {
        match Self::admit_bytes(catalog, bytes, limits, evidence, budget) {
            Ok(package) => {
                let boxed_package = {
                    let site =
                        super::read_charge::Site::new(PackageReadStage::Retained, 0, "result");
                    let bytes = std::alloc::Layout::new::<CheckedPackageV2>()
                        .pad_to_align()
                        .size();
                    if let Err(stop) = super::read_charge::charge(budget, site, bytes as u128) {
                        budget.abort();
                        return budgeted_outcome(stop, budget);
                    }
                    Box::new(package)
                };
                budget.finish_read(work_before, storage_before);
                CheckedPackageV2BudgetedReadResult::Admitted(boxed_package)
            }
            Err(stop) => {
                budget.abort();
                budgeted_outcome(stop, budget)
            }
        }
    }

    /// Reads one canonical V2 value without exposing a partial package.
    ///
    /// Equivalent to [`Self::read_with_budget`] under a budget whose work
    /// limit is `limits.work` and whose read storage is unbounded.
    pub fn read(
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
    ) -> CheckedPackageV2ReadResult {
        let mut budget =
            NativeJoinBudget::new(limits.work, u64::MAX).with_read_storage_limit(u64::MAX);
        let outcome = Self::read_with_budget(bytes, limits, evidence, &mut budget);
        Self::unbudgeted_result(outcome, limits, &budget)
    }

    /// [`Self::read`] against the given result of reading the operation
    /// catalog.
    #[cfg(test)]
    pub(super) fn read_against(
        catalog: &'static Result<super::operation_catalog::OperationCatalog, CatalogReadError>,
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
    ) -> CheckedPackageV2ReadResult {
        let mut budget =
            NativeJoinBudget::new(limits.work, u64::MAX).with_read_storage_limit(u64::MAX);
        let outcome = Self::read_with_budget_against(catalog, bytes, limits, evidence, &mut budget);
        Self::unbudgeted_result(outcome, limits, &budget)
    }

    fn unbudgeted_result(
        outcome: CheckedPackageV2BudgetedReadResult,
        limits: CheckedPackageReadLimits,
        budget: &NativeJoinBudget,
    ) -> CheckedPackageV2ReadResult {
        match outcome {
            CheckedPackageV2BudgetedReadResult::Admitted(package) => {
                CheckedPackageV2ReadResult::Admitted(package)
            }
            CheckedPackageV2BudgetedReadResult::Refused(refusal) => {
                CheckedPackageV2ReadResult::Refused(refusal)
            }
            CheckedPackageV2BudgetedReadResult::Incomplete(incomplete) => {
                CheckedPackageV2ReadResult::Incomplete(incomplete)
            }
            CheckedPackageV2BudgetedReadResult::CatalogUnavailable(error) => {
                CheckedPackageV2ReadResult::CatalogUnavailable(error)
            }
            CheckedPackageV2BudgetedReadResult::BudgetIncomplete(incomplete) => {
                CheckedPackageV2ReadResult::Incomplete(CheckedPackageIncomplete {
                    limit_kind: CheckedPackageLimit::Work,
                    limit: limits.work,
                    consumed: incomplete.consumed,
                    path: incomplete.path,
                })
            }
            // The read-storage limit of `u64::MAX` exceeds every possible total
            // of requested bytes, so the unbudgeted read is never exhausted;
            // the work counter's own account stands in if it ever were.
            CheckedPackageV2BudgetedReadResult::Exhausted(_) => {
                CheckedPackageV2ReadResult::Incomplete(CheckedPackageIncomplete {
                    limit_kind: CheckedPackageLimit::Work,
                    limit: limits.work,
                    consumed: budget.consumed().work,
                    path: None,
                })
            }
        }
    }
}

/// Builds the budgeted outcome of a stopped read, charging its owned
/// diagnostic to the budget first.
fn budgeted_outcome(
    stop: ValidationFailure,
    budget: &mut NativeJoinBudget,
) -> CheckedPackageV2BudgetedReadResult {
    match stop {
        ValidationFailure::Refused(refusal) => {
            // A stage that built the refusal paid its diagnostic first.
            let pending = CheckedPackagePendingOutcome::Refused(refusal.code);
            if budget.take_paid() == Some(pending) {
                return CheckedPackageV2BudgetedReadResult::Refused(*refusal);
            }
            let bytes = refusal
                .loci
                .iter()
                .filter_map(|location| location.path.as_ref())
                .chain(refusal.document_pointer.as_ref())
                .map(|pointer| pointer.as_str().len() as u128)
                .sum();
            match charge_outcome_diagnostic(
                bytes,
                CheckedPackagePendingOutcome::Refused(refusal.code),
                budget,
            ) {
                None => CheckedPackageV2BudgetedReadResult::Refused(*refusal),
                Some(exhausted) => exhausted,
            }
        }
        ValidationFailure::Incomplete(incomplete) => {
            let pending = CheckedPackagePendingOutcome::Incomplete(incomplete.limit_kind);
            if budget.take_paid() == Some(pending) {
                return CheckedPackageV2BudgetedReadResult::Incomplete(incomplete);
            }
            let bytes = incomplete
                .path
                .as_ref()
                .map_or(0, |pointer| pointer.as_str().len() as u128);
            match charge_outcome_diagnostic(
                bytes,
                CheckedPackagePendingOutcome::Incomplete(incomplete.limit_kind),
                budget,
            ) {
                None => CheckedPackageV2BudgetedReadResult::Incomplete(incomplete),
                Some(exhausted) => exhausted,
            }
        }
        ValidationFailure::Budget(stop) => budget_denial(stop, budget),
    }
}

/// Charges the owned diagnostic of a refusal or `incomplete` outcome before it
/// is returned; `Some` is the `Exhausted` outcome when the budget cannot pay.
fn charge_outcome_diagnostic(
    bytes: u128,
    pending: CheckedPackagePendingOutcome,
    budget: &mut NativeJoinBudget,
) -> Option<CheckedPackageV2BudgetedReadResult> {
    // The diagnostic is charged at the stage the read stopped in.
    let source = match budget.active_locus() {
        Some(NativeJoinLocus {
            source: source @ NativeJoinLocusSource::PackageRead(..),
            ..
        }) => source,
        _ => NativeJoinLocusSource::PackageRead(PackageReadStage::WireDecode, 0),
    };
    let locus = NativeJoinLocus {
        source,
        form: DIAGNOSTIC_FORM,
    };
    budget.set_active_locus(Some(locus));
    let paid = budget.charge_read_storage(bytes);
    budget.set_active_locus(None);
    if paid {
        return None;
    }
    Some(CheckedPackageV2BudgetedReadResult::Exhausted(
        CheckedPackageReadExhausted {
            denied: PackageReadDeniedCharge {
                resource: PackageReadResource::ReadStorage,
                amount: bytes,
            },
            configured_bound: budget.read_storage_limit(),
            locus: budget.first_unpaid_locus().unwrap_or(locus),
            pending: Some(pending),
        },
    ))
}

fn budget_denial(
    stop: BudgetStop,
    budget: &mut NativeJoinBudget,
) -> CheckedPackageV2BudgetedReadResult {
    let denied = PackageReadDeniedCharge {
        resource: stop.resource,
        amount: stop.amount,
    };
    let (limit, consumed) = match stop.resource {
        PackageReadResource::Work => (budget.work_limit(), budget.consumed().work),
        PackageReadResource::ReadStorage => {
            (budget.read_storage_limit(), budget.read_storage_consumed())
        }
    };
    let original_locus = budget.first_unpaid_locus();
    // A pointer a stage paid for before building it is already charged; a
    // diagnostic its stage could not pay for makes this outcome `exhausted`.
    let pending = budget.take_pending();
    let diagnostic_bytes = match (pending, budget.take_prepaid_diagnostic()) {
        (Some(_), _) | (None, Some(false)) => u128::MAX,
        (None, Some(true)) => 0,
        (None, None) => stop
            .path
            .as_ref()
            .map_or(0, |path| path.as_str().len() as u128),
    };
    if let Some(locus) = original_locus {
        budget.set_active_locus(Some(NativeJoinLocus {
            source: locus.source,
            form: DIAGNOSTIC_FORM,
        }));
    }
    let affordable = budget.charge_read_storage(diagnostic_bytes);
    budget.set_active_locus(None);
    match (affordable, original_locus) {
        (true, _) => {
            let consumed =
                u64::try_from(u128::from(consumed).saturating_add(stop.amount)).unwrap_or(u64::MAX);
            CheckedPackageV2BudgetedReadResult::BudgetIncomplete(CheckedPackageBudgetIncomplete {
                denied,
                limit,
                consumed,
                path: stop.path,
            })
        }
        (false, Some(locus)) => {
            CheckedPackageV2BudgetedReadResult::Exhausted(CheckedPackageReadExhausted {
                denied,
                configured_bound: limit,
                locus,
                pending,
            })
        }
        // A budget denial always records its locus; with none recorded the
        // denial still reports its fixed-size facts at the package level.
        (false, None) => {
            CheckedPackageV2BudgetedReadResult::Exhausted(CheckedPackageReadExhausted {
                denied,
                configured_bound: limit,
                locus: NativeJoinLocus {
                    source: NativeJoinLocusSource::Package,
                    form: DIAGNOSTIC_FORM,
                },
                pending,
            })
        }
    }
}

/// The shared reader's and writer's reservation: charges read storage on the
/// caller's budget at its active locus.
pub(super) struct ReadStorageReservation<'b>(pub(super) &'b mut NativeJoinBudget);

impl Reservation for ReadStorageReservation<'_> {
    fn reserve(&mut self, bytes: usize) -> Result<(), ReservationDenied> {
        if self.0.charge_read_storage(bytes as u128) {
            Ok(())
        } else {
            Err(ReservationDenied)
        }
    }
}

/// The denial the budget just recorded, as the stop that ends the read.
pub(in crate::checked_package) fn denial(budget: &NativeJoinBudget) -> ValidationFailure {
    let (resource, amount) = budget
        .first_unpaid_resource()
        .unwrap_or((PackageReadResource::ReadStorage, 0));
    ValidationFailure::Budget(BudgetStop {
        resource,
        amount,
        path: None,
    })
}

pub(super) fn at_stage(budget: &mut NativeJoinBudget, stage: PackageReadStage, form: &'static str) {
    budget.set_active_locus(Some(NativeJoinLocus {
        source: NativeJoinLocusSource::PackageRead(stage, 0),
        form,
    }));
}

/// A sink comparing the canonical encoding with the package bytes as it is
/// written; it asks the writer for no memory of its own.
struct ComparingSink<'a> {
    original: &'a [u8],
    at: usize,
    agrees: bool,
}

impl quire_canonical::Sink for ComparingSink<'_> {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), quire_canonical::Error> {
        let end = self.at.checked_add(bytes.len());
        if end.and_then(|end| self.original.get(self.at..end)) != Some(bytes) {
            self.agrees = false;
        }
        self.at = end.unwrap_or(usize::MAX);
        Ok(())
    }
}

/// The package document as the canonical writer encodes it, with the
/// encoder's refusal of an integer past 2^53 (see [`super::checked_json`]).
struct PackageDocument<'d> {
    document: &'d Document,
}

impl quire_canonical::Encode for PackageDocument<'_> {
    fn encode_into<S: quire_canonical::Sink + ?Sized>(
        &self,
        writer: &mut quire_canonical::Writer<'_, S>,
    ) -> Result<(), quire_canonical::Error> {
        super::checked_json::encode_node(self.document.root(), writer)
    }
}

/// Reads the package bytes with the shared reader and requires them to be
/// their own canonical encoding, charging every reservation of both to the
/// budget's read storage. The byte limit is checked before any request.
///
/// A byte-stream fault refuses before `noncanonical_wire`, as FR-038-AC-509
/// orders them.
/// The size of the allocation `Arc::new` makes for a `T`: its two counts,
/// then the value, padded to the value's alignment.
pub(super) fn arc_bytes<T>() -> usize {
    std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .map_or(usize::MAX, |(layout, _)| layout.pad_to_align().size())
}

/// Shares the package document with the handles the decode hands out,
/// charging the shared allocation at the decode locus first.
pub(super) fn share_document(
    document: Document,
    budget: &mut NativeJoinBudget,
) -> Result<std::sync::Arc<Document>, ValidationFailure> {
    at_stage(budget, PackageReadStage::WireDecode, "package");
    if !budget.charge_read_storage(arc_bytes::<Document>() as u128) {
        return Err(denial(budget));
    }
    budget.set_active_locus(None);
    Ok(std::sync::Arc::new(document))
}

pub(in crate::checked_package) fn read_envelope(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    budget: &mut NativeJoinBudget,
) -> Result<Document, ValidationFailure> {
    if exceeds(bytes.len(), limits.bytes) {
        return Err(refuse_bytes_exceeded(limits.bytes, bytes.len()));
    }
    at_stage(budget, PackageReadStage::Envelope, "package_bytes");
    let document = match quire_canonical::read_with_reservation(
        bytes,
        limits.bytes,
        &mut ReadStorageReservation(budget),
    ) {
        Ok(document) => document,
        Err(ReadError::Limit(_)) => return Err(refuse_bytes_exceeded(limits.bytes, bytes.len())),
        Err(ReadError::DuplicateName { pointer, .. }) => {
            let mut meter = super::WorkMeter::over(budget);
            return Err(super::stage_storage::refuse_held_pointer(
                &mut meter,
                &super::stage_storage::Site::new(
                    PackageReadStage::Envelope,
                    0,
                    "package_bytes",
                    super::stage_storage::Path::root(),
                ),
                CheckedPackageRefusalCode::DuplicateMember,
                pointer,
            ));
        }
        Err(ReadError::ReservationDenied { .. }) => return Err(denial(budget)),
        Err(_) => {
            return Err(prepaid_bytes_refusal(
                budget,
                PackageReadStage::Envelope,
                CheckedPackageRefusalCode::MalformedWire,
            ));
        }
    };
    // The reservation-backed reader has already checked syntax and duplicate
    // members. Walk its arena with fixed stack storage to locate the cap value.
    if let Some(failure) = nesting_cap_refusal(&document, budget) {
        return Err(failure);
    }
    at_stage(budget, PackageReadStage::Canonical, "canonical_bytes");
    let mut sink = ComparingSink {
        original: bytes,
        at: 0,
        agrees: true,
    };
    let encoded = quire_canonical::encode_with_reservation(
        &mut sink,
        &PackageDocument {
            document: &document,
        },
        quire_canonical::Limits::new(limits.bytes),
        &mut ReadStorageReservation(budget),
    );
    match encoded {
        Err(quire_canonical::Error::ReservationDenied { .. }) => Err(denial(budget)),
        Ok(_) if sink.agrees && sink.at == bytes.len() => {
            budget.set_active_locus(None);
            Ok(document)
        }
        _ => Err(prepaid_bytes_refusal(
            budget,
            PackageReadStage::Canonical,
            CheckedPackageRefusalCode::NoncanonicalWire,
        )),
    }
}

/// One fixed-stack frame over the charged document's existing child iterators.
enum ShapeChildren<'d> {
    Array {
        items: quire_canonical::Items<'d>,
        next_index: usize,
    },
    Object(quire_canonical::Members<'d>),
}

impl<'d> ShapeChildren<'d> {
    fn for_node(node: quire_canonical::NodeRef<'d>) -> Option<Self> {
        match node.node() {
            quire_canonical::Node::Array(items) => Some(Self::Array {
                items,
                next_index: 0,
            }),
            quire_canonical::Node::Object(members) => Some(Self::Object(members)),
            _ => None,
        }
    }

    fn next(&mut self) -> Option<(quire_canonical::NodeRef<'d>, Step<'d>)> {
        match self {
            Self::Array { items, next_index } => {
                let child = items.next()?;
                let index = *next_index;
                *next_index = next_index.saturating_add(1);
                Some((child, Step::Index(index)))
            }
            Self::Object(members) => members.next().map(|(name, child)| (child, Step::Key(name))),
        }
    }
}

/// Finds and refuses the first container reaching the fixed nesting cap.
/// Traversal and path scratch use fixed stack arrays; the only heap request is
/// the located refusal, charged from borrowed path steps before it is built.
fn nesting_cap_refusal(
    document: &Document,
    budget: &mut NativeJoinBudget,
) -> Option<ValidationFailure> {
    let mut frames: [Option<ShapeChildren<'_>>; JSON_NESTING_LIMIT] = std::array::from_fn(|_| None);
    let mut path = [Step::Index(0); JSON_NESTING_LIMIT];
    let Some(root) = ShapeChildren::for_node(document.root()) else {
        return None;
    };
    frames[0] = Some(root);
    let mut frame_len = 1;

    while frame_len > 0 {
        let parent_index = frame_len - 1;
        let Some(children) = frames.get_mut(parent_index).and_then(Option::as_mut) else {
            return None;
        };
        let Some((child, edge)) = children.next() else {
            frames[parent_index] = None;
            frame_len -= 1;
            continue;
        };
        let Some(child) = ShapeChildren::for_node(child) else {
            continue;
        };
        let Some(slot) = path.get_mut(parent_index) else {
            return None;
        };
        *slot = edge;
        if frame_len + 1 == JSON_NESTING_LIMIT {
            let steps = path.get(..frame_len)?;
            let at = Trail::Base(steps);
            let refusal =
                super::stage_storage::Refusal::new(CheckedPackageRefusalCode::MalformedWire, at);
            let mut meter = super::WorkMeter::over(budget);
            let site = super::stage_storage::Site::new(
                PackageReadStage::Envelope,
                0,
                "package_bytes",
                super::stage_storage::Path::root(),
            );
            return Some(super::stage_storage::refuse(&mut meter, &site, refusal));
        }
        let Some(frame) = frames.get_mut(frame_len) else {
            return None;
        };
        *frame = Some(child);
        frame_len += 1;
    }
    None
}

fn prepaid_bytes_refusal(
    budget: &mut NativeJoinBudget,
    stage: PackageReadStage,
    code: CheckedPackageRefusalCode,
) -> ValidationFailure {
    let mut meter = super::WorkMeter::over(budget);
    super::stage_storage::refuse_bytes(
        &mut meter,
        &super::stage_storage::Site::new(
            stage,
            0,
            "package_bytes",
            super::stage_storage::Path::root(),
        ),
        code,
    )
}

#[cfg(test)]
mod early_refusal_allocation_tests {
    use super::*;
    use ix_trace_rs::trace;

    #[trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn minimal_early_refusals_are_paid_for_unlimited_exact_and_short_reads() {
        let evidence = CheckedPackageEvidence::new();
        for (bytes, code, path, version) in [
            (
                b"null".as_slice(),
                CheckedPackageRefusalCode::MalformedWire,
                "",
                None,
            ),
            (
                br#""text""#.as_slice(),
                CheckedPackageRefusalCode::MalformedWire,
                "",
                None,
            ),
            (
                b"[]".as_slice(),
                CheckedPackageRefusalCode::MalformedWire,
                "",
                None,
            ),
            (
                b"[null]".as_slice(),
                CheckedPackageRefusalCode::MalformedWire,
                "",
                None,
            ),
            (
                br#"{"contract_version":"future"}"#.as_slice(),
                CheckedPackageRefusalCode::UnknownContractVersion,
                "/contract_version",
                Some("future"),
            ),
            (
                br#"{"contract_version":"quire.checked-package/v2"}"#.as_slice(),
                CheckedPackageRefusalCode::MalformedWire,
                "",
                None,
            ),
            (
                br#"{"":null,"":null}"#.as_slice(),
                CheckedPackageRefusalCode::DuplicateMember,
                "/",
                None,
            ),
        ] {
            let run = |mut budget| {
                let mut result = None;
                let measured = allocation_counter::measure(|| {
                    result = Some(CheckedPackageV2::read_with_budget(
                        bytes,
                        CheckedPackageReadLimits::bounded(),
                        &evidence,
                        &mut budget,
                    ));
                });
                (result.expect("ran"), measured.bytes_total, budget)
            };
            // The first measured public read precedes the plain reference.
            let (result, requested, budget) = run(NativeJoinBudget::unlimited());
            let total = budget.read_storage_consumed();
            assert_eq!(requested, total, "{bytes:?}");
            let CheckedPackageV2BudgetedReadResult::Refused(refusal) = &result else {
                panic!("typed refusal: {result:?}");
            };
            assert_eq!(refusal.code, code);
            assert_eq!(
                refusal.loci[0].path.as_ref().map(JsonPointer::as_str),
                Some(path)
            );
            assert_eq!(refusal.contract_version.as_deref(), version);
            let CheckedPackageV2ReadResult::Refused(plain) =
                CheckedPackageV2::read(bytes, CheckedPackageReadLimits::bounded(), &evidence)
            else {
                panic!("plain read refuses");
            };
            assert_eq!(&plain, refusal);
            let (exact, requested, budget) =
                run(NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(total));
            assert_eq!(exact, result);
            assert_eq!(requested, budget.read_storage_consumed());
            let (short, requested, budget) =
                run(NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(total - 1));
            assert!(
                matches!(short, CheckedPackageV2BudgetedReadResult::Exhausted(exhausted)
                if exhausted.pending == Some(CheckedPackagePendingOutcome::Refused(code)))
            );
            assert_eq!(requested, budget.read_storage_consumed());
            assert_eq!(budget.consumed().storage, 0);
        }
    }

    #[trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn held_pointer_empty_exact_and_shrinking_buffers_charge_only_new_requests() {
        for (capacity, text) in [(0, ""), (64, ""), (5, "/same"), (64, "/")] {
            let run = |limit| {
                // This models the already-paid canonical-reader buffer.
                let mut pointer = String::with_capacity(capacity);
                pointer.push_str(text);
                let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
                let mut result = None;
                let measured = allocation_counter::measure(|| {
                    let mut meter = super::super::WorkMeter::over(&mut budget);
                    result = Some(super::super::stage_storage::refuse_held_pointer(
                        &mut meter,
                        &super::super::stage_storage::Site::new(
                            PackageReadStage::Envelope,
                            0,
                            "package_bytes",
                            super::super::stage_storage::Path::root(),
                        ),
                        CheckedPackageRefusalCode::DuplicateMember,
                        pointer,
                    ));
                });
                (
                    result.expect("ran"),
                    measured.bytes_total,
                    budget.read_storage_consumed(),
                )
            };
            let (result, requested, charged) = run(u64::MAX);
            assert_eq!(requested, charged, "capacity {capacity}");
            assert!(matches!(&result, ValidationFailure::Refused(refusal)
                if refusal.code == CheckedPackageRefusalCode::DuplicateMember
                    && refusal.primary_path().is_some_and(|path| path.as_str() == text)));
            let (exact, requested, consumed) = run(charged);
            assert_eq!(exact, result);
            assert_eq!(requested, consumed);
            let (short, requested, consumed) = run(charged - 1);
            assert!(matches!(short, ValidationFailure::Budget(_)));
            assert_eq!((requested, consumed), (0, 0));
        }
    }
}
