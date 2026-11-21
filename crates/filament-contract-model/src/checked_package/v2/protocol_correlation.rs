// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The authored correlation clauses of protocol controls: the optional
//! top-level `protocol_correlation_occurrences` member and its typed read.
//!
//! Implements: FSpec FR-831 (the member, its reader checks and its typed read),
//! FR-322 "Check order" (the last carrier step) and filament-contract-ir FR-358
//! (the charges, on the one caller-owned budget).
//!
//! The member records, for each related-bearing control occurrence, which
//! canonical `related` record each authored clause became and under which
//! local alias it was written. It lives outside every identity preimage: it
//! adds no node, changes no node id and no other member.
//!
//! The reader checks consistency only. It cannot prove that a producer
//! recorded every authored clause (an omitted trailing duplicate), or the exact
//! authored spelling of an alias; that is producer conformance.

use super::native_join_budget::{NativeJoinBudget, PackageReadStage};
use super::protocol_body::binding_value;
use super::stage_storage::{self, Path, Refusal, Site};
use super::{
    CheckedNodeKind, CheckedPackageV2, CheckedSemanticNodeV2, CheckedTerm, ProtocolForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedNodeId, CheckedNodePathSegment, CheckedOccurrence, CheckedOccurrenceRole,
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    CheckedProtocolRefusalDetail, CheckedSourceMapEntry,
};
use quire_canonical::{Encode, Error, Sink, Writer};
use serde::de::{Deserializer, Error as _};
use serde::{Deserialize, Serialize};

const MAX_SAFE: u64 = 9_007_199_254_740_991;

/// The six control operators that carry `related` records (FR-466; `recommend`
/// is the FR-822 control).
const ELIGIBLE: [&str; 6] = ["send", "receive", "attempt", "effect", "event", "recommend"];

/// One authored correlation clause of a control occurrence.
///
/// A row is one clause as written, before deduplication. The clauses of one
/// control occurrence are its rows by `source_ordinal`; two rows of a group may
/// share a `canonical_index` (identical clauses, or two aliases of one
/// relationship with equal ends) and one `alias` may repeat across
/// `canonical_index` values.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolCorrelationRow {
    /// The control node of the package the clause belongs to.
    pub control: CheckedNodeId,
    /// The `declaration` occurrence of that control the clause was written under.
    pub occurrence: CheckedOccurrence,
    /// The zero-based position, in the control's emitted `related` argument, of
    /// the canonical record the clause became.
    #[serde(deserialize_with = "safe_integer")]
    pub canonical_index: u64,
    /// The local relationship alias exactly as the source spelled it: an
    /// `Identifier`. It names no relationship and is for diagnostics only.
    #[serde(deserialize_with = "identifier")]
    pub alias: Box<str>,
    /// The zero-based source position of the clause among the correlation
    /// clauses of that control occurrence.
    #[serde(deserialize_with = "safe_integer")]
    pub source_ordinal: u64,
}

fn safe_integer<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    let value = u64::deserialize(deserializer)?;
    if value > MAX_SAFE {
        return Err(D::Error::custom("integer above 9007199254740991"));
    }
    Ok(value)
}

fn identifier<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Box<str>, D::Error> {
    let value = Box::<str>::deserialize(deserializer)?;
    if !super::identity::is_identifier(&value) {
        return Err(D::Error::custom("alias is not an Identifier"));
    }
    Ok(value)
}

impl Encode for ProtocolCorrelationRow {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::encoded(writer, "control", &self.control)?;
        super::encode::member(writer, "occurrence", &self.occurrence)?;
        super::encode::member(writer, "canonical_index", &self.canonical_index)?;
        super::encode::member(writer, "alias", self.alias.as_ref())?;
        super::encode::member(writer, "source_ordinal", &self.source_ordinal)?;
        writer.end_object()
    }
}

/// The stable name a consumer reports when it needs the clause order of a
/// control and the package carries no `protocol_correlation_occurrences`.
///
/// The absence is a capability the producer did not supply, not an empty
/// provenance: it is distinct from `incomplete` (a work limit) and from every
/// refusal of the member.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnavailableCorrelationProvenance;

impl UnavailableCorrelationProvenance {
    /// The stable name of the refusal a consumer returns.
    pub const NAME: &'static str = "unavailable_correlation_provenance";

    /// [`Self::NAME`].
    pub const fn as_str(self) -> &'static str {
        Self::NAME
    }
}

impl std::fmt::Display for UnavailableCorrelationProvenance {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(Self::NAME)
    }
}

impl std::error::Error for UnavailableCorrelationProvenance {}

/// What a package says about the authored correlation clauses of its controls:
/// exactly one of two outcomes.
#[derive(Clone, Copy, Debug)]
pub enum ProtocolCorrelationProvenance<'a> {
    /// The package carries no member. Every other read is unchanged; the
    /// absence is never an empty provenance.
    Unavailable(UnavailableCorrelationProvenance),
    /// The package carries the member and the reader validated it. A present
    /// `[]` is a complete empty provenance.
    Complete(ProtocolCorrelationTable<'a>),
}

/// The complete table, borrowed from the admitted package: its groups in table
/// order. The reader holds it all or not at all; no prefix is ever exposed.
#[derive(Clone, Copy, Debug)]
pub struct ProtocolCorrelationTable<'a> {
    package: &'a CheckedPackageV2,
    groups: &'a [Group],
}

/// The native record of one `(control, occurrence)` group: the span of its rows
/// in the table and the position of its whole `source_map` entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Group {
    row_start: usize,
    row_len: usize,
    source_entry: usize,
}

/// What an admitted package keeps of the member: the validated groups, and the
/// control positions the coverage pass walks.
#[derive(Clone, Debug, Default)]
pub(super) struct Retained {
    /// `None` when the package carries no member; `Some`, possibly empty, when
    /// it does.
    pub(super) groups: Option<Box<[Group]>>,
    /// The `protocol`/`control` node positions in ascending node-id order, held
    /// only when the member is present.
    controls: Vec<usize>,
}

impl Retained {
    pub(super) fn new(controls: Vec<usize>) -> Self {
        Self {
            groups: None,
            controls,
        }
    }
}

/// One control occurrence of the table: its clauses in source order and the
/// whole `source_map` entry of its `(control, declaration, ordinal)` key.
#[derive(Clone, Copy, Debug)]
pub struct ProtocolCorrelationGroup<'a> {
    rows: &'a [ProtocolCorrelationRow],
    source_entry: &'a CheckedSourceMapEntry,
}

impl<'a> ProtocolCorrelationTable<'a> {
    /// The number of control occurrences with clauses.
    pub fn len(&self) -> usize {
        self.groups.len()
    }

    /// Whether the table is the complete empty provenance.
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    /// The groups in table order: by control in structural node-id order, then
    /// occurrence ordinal.
    pub fn groups(&self) -> impl ExactSizeIterator<Item = ProtocolCorrelationGroup<'a>> + 'a {
        let package = self.package;
        self.groups.iter().map(move |group| package.group(group))
    }

    /// The group of the control occurrence `(control, declaration, ordinal)`.
    pub fn group(
        &self,
        control: &CheckedNodeId,
        ordinal: u64,
    ) -> Option<ProtocolCorrelationGroup<'a>> {
        let rows = self.package.correlation_rows();
        let at = self.groups.partition_point(|group| {
            let row = &rows[group.row_start];
            (&row.control, row.occurrence.ordinal) < (control, ordinal)
        });
        let group = self.groups.get(at)?;
        let row = &rows[group.row_start];
        (&row.control == control && row.occurrence.ordinal == ordinal)
            .then(|| self.package.group(group))
    }
}

impl<'a> ProtocolCorrelationGroup<'a> {
    /// The control node.
    pub fn control(&self) -> &'a CheckedNodeId {
        &self.rows[0].control
    }

    /// The `declaration` occurrence of the control this group belongs to.
    pub fn occurrence(&self) -> &'a CheckedOccurrence {
        &self.rows[0].occurrence
    }

    /// The authored clauses, in source order: `source_ordinal` is each row's
    /// position. Nothing is sorted or deduplicated; repeated aliases and
    /// repeated canonical indices are as written.
    pub fn clauses(&self) -> &'a [ProtocolCorrelationRow] {
        self.rows
    }

    /// The whole admitted `source_map` entry of `(control, declaration,
    /// occurrence.ordinal)`, every region of it, borrowed from the package.
    pub fn source_entry(&self) -> &'a CheckedSourceMapEntry {
        self.source_entry
    }
}

/// Why a consumer cannot take the clauses of a control.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolCorrelationReadError {
    /// The control's canonical `related` is nonempty and the package carries
    /// no member: refuse; never proceed on empty or guessed premises.
    Unavailable(UnavailableCorrelationProvenance),
    /// The node is no eligible control occurrence of this package.
    NotAnEligibleControl,
}

impl std::fmt::Display for ProtocolCorrelationReadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable(unavailable) => unavailable.fmt(formatter),
            Self::NotAnEligibleControl => formatter.write_str("not_an_eligible_control"),
        }
    }
}

impl std::error::Error for ProtocolCorrelationReadError {}

impl CheckedPackageV2 {
    /// What this package says about the authored correlation clauses of its
    /// controls: the validated complete table, or the typed unavailable
    /// provenance when the package carries no
    /// `protocol_correlation_occurrences`.
    ///
    /// Each package answers for its own controls only. A dependency's table is
    /// read from the dependency (see [`Self::direct_dependencies`]), never from
    /// this package's, and a dependency without a table is unavailable whatever
    /// this package carries.
    ///
    /// The member is consistent, not authenticated: a reader cannot tell
    /// whether a producer omitted a trailing duplicate clause or respelled an
    /// alias.
    pub fn protocol_correlation_provenance(&self) -> ProtocolCorrelationProvenance<'_> {
        match &self.correlation.groups {
            Some(groups) => ProtocolCorrelationProvenance::Complete(ProtocolCorrelationTable {
                package: self,
                groups,
            }),
            None => ProtocolCorrelationProvenance::Unavailable(UnavailableCorrelationProvenance),
        }
    }

    /// The authored clauses of the declaration occurrence of `control`, in
    /// source order, for a consumer that needs the clause order.
    ///
    /// A control whose canonical `related` is empty needs no clause order: it
    /// yields no clauses whether or not the package carries the member. A
    /// control whose `related` is nonempty and a package without the member is
    /// refused as [`UnavailableCorrelationProvenance`].
    pub fn protocol_correlation_clauses(
        &self,
        control: &CheckedNodeId,
    ) -> Result<&[ProtocolCorrelationRow], ProtocolCorrelationReadError> {
        let position = self
            .node_position(control)
            .ok_or(ProtocolCorrelationReadError::NotAnEligibleControl)?;
        let facts = self
            .control_facts(position)
            .ok_or(ProtocolCorrelationReadError::NotAnEligibleControl)?;
        match self.protocol_correlation_provenance() {
            ProtocolCorrelationProvenance::Unavailable(unavailable) if facts.related > 0 => {
                Err(ProtocolCorrelationReadError::Unavailable(unavailable))
            }
            ProtocolCorrelationProvenance::Unavailable(_) => Ok(&[]),
            ProtocolCorrelationProvenance::Complete(table) => Ok(table
                .group(control, 0)
                .map_or(&[][..], |group| group.clauses())),
        }
    }

    fn correlation_rows(&self) -> &[ProtocolCorrelationRow] {
        self.wire
            .protocol_correlation_occurrences
            .as_deref()
            .unwrap_or_default()
    }

    fn group(&self, group: &Group) -> ProtocolCorrelationGroup<'_> {
        ProtocolCorrelationGroup {
            rows: &self.correlation_rows()[group.row_start..group.row_start + group.row_len],
            source_entry: &self.wire.source_map[group.source_entry],
        }
    }

    /// The operator and canonical `related` length of the node at `position`
    /// when it is an eligible control.
    fn control_facts(&self, position: usize) -> Option<ControlFacts> {
        control_facts(
            self.kinds.get(position)?,
            &self.wire.semantic_graph.nodes.get(position)?.body,
        )
    }
}

#[derive(Clone, Copy, Debug)]
struct ControlFacts {
    /// `m`: the canonical `related` records.
    related: usize,
}

/// `Some` for a `protocol`/`control` node of an eligible operator.
fn control_facts(kind: &CheckedNodeKind, body: &CheckedTerm) -> Option<ControlFacts> {
    if *kind != CheckedNodeKind::Protocol(ProtocolForm::Control) {
        return None;
    }
    let CheckedTerm::Application {
        operation,
        arguments,
        ..
    } = body
    else {
        return None;
    };
    let operator = operation.member_text("operator")?;
    if !ELIGIBLE.contains(&operator) {
        return None;
    }
    let related = match binding_value(arguments, "related")? {
        CheckedTerm::Aggregate { members } => members.len(),
        _ => return None,
    };
    Some(ControlFacts { related })
}

/// The byte length of a node reference as FR-831 counts it: the package name
/// plus every path segment (an index segment counts its decimal numeral).
fn reference_bytes(id: &CheckedNodeId) -> u64 {
    let digits = |value: u64| u64::from(value.checked_ilog10().unwrap_or(0)) + 1;
    id.path
        .iter()
        .fold(id.package.len() as u64, |total, segment| {
            total.saturating_add(match segment {
                CheckedNodePathSegment::Name(name) => name.len() as u64,
                CheckedNodePathSegment::Index(value) | CheckedNodePathSegment::Operand(value) => {
                    digits(*value)
                }
                CheckedNodePathSegment::Dependency { package, node } => {
                    (package.len() as u64).saturating_add(digits(*node))
                }
                // An operand node is never a segment of a control's declared path.
                CheckedNodePathSegment::Node(_) => 0,
            })
        })
}

fn member_path() -> Path<'static> {
    Path::root().key("protocol_correlation_occurrences")
}

fn site(index: usize, form: &'static str, at: Path<'static>) -> Site<'static> {
    Site::new(PackageReadStage::Retained, index, form, at)
}

/// The positions of the package's `protocol`/`control` nodes in ascending
/// structural node-id order.
///
/// Built during retention, and only for a package that carries the member, so
/// that the coverage pass walks controls without an unmetered scan of every
/// node. The block is charged to the read's budget before it is requested, with
/// the other retained position tables, and is part of the baseline the step's
/// own charges are measured over.
pub(super) fn control_positions(
    kinds: &[CheckedNodeKind],
    nodes: &[CheckedSemanticNodeV2],
    budget: &mut NativeJoinBudget,
) -> Result<Vec<usize>, ValidationFailure> {
    let is_control =
        |kind: &CheckedNodeKind| *kind == CheckedNodeKind::Protocol(ProtocolForm::Control);
    let count = kinds.iter().filter(|kind| is_control(kind)).count();
    let site =
        super::read_charge::Site::new(PackageReadStage::Retained, 0, "protocol_control_positions");
    let mut positions: Vec<usize> = super::read_charge::with_capacity(budget, site, count)?;
    positions.extend(
        kinds
            .iter()
            .enumerate()
            .filter_map(|(position, kind)| is_control(kind).then_some(position)),
    );
    positions.sort_unstable_by(|left, right| nodes[*left].node_id.cmp(&nodes[*right].node_id));
    Ok(positions)
}

fn invalid(
    meter: &mut WorkMeter<'_>,
    site: &Site<'_>,
    code: Code,
    cause: Cause,
    at: Path<'_>,
) -> ValidationFailure {
    stage_storage::refuse(meter, site, Refusal::new(code, at).because(cause))
}

/// The last carrier step of the strict read: the row pass, then the coverage
/// pass, on the admitted package, charged to the caller's budget.
///
/// An absent member charges nothing and is the typed unavailable provenance.
pub(super) fn admit(
    package: &CheckedPackageV2,
    budget: &mut NativeJoinBudget,
) -> Result<Option<Box<[Group]>>, ValidationFailure> {
    let Some(rows) = package.wire.protocol_correlation_occurrences.as_deref() else {
        return Ok(None);
    };
    let mut meter = WorkMeter::over(budget);
    let nodes = &package.wire.semantic_graph.nodes;
    let mut groups = 0_usize;
    let mut previous: Option<&ProtocolCorrelationRow> = None;
    for (index, row) in rows.iter().enumerate() {
        let at = member_path().index(index);
        let row_site = site(index, "protocol_correlation_row", at);
        // One charge, whole, before the row is inspected. The join and the
        // source-map lookup read this row's control text and alias; the key
        // comparison reads the previous row's control up to the first
        // differing byte, which `Bp` bounds. The previous alias is no part of
        // the key and is not charged.
        let own = reference_bytes(&row.control);
        let before = previous.map_or(0, |earlier| reference_bytes(&earlier.control));
        let charge = 1_u64
            .saturating_add(own)
            .saturating_add(row.alias.len() as u64)
            .saturating_add(before);
        stage_storage::work(&mut meter, &row_site, charge)?;

        // Joins, in order.
        let Some((position, node)) = package
            .node_position(&row.control)
            .and_then(|position| nodes.get(position).map(|node| (position, node)))
        else {
            return Err(invalid(
                &mut meter,
                &row_site,
                Code::InvalidPackage,
                Cause::InvalidValue,
                at.key("control"),
            ));
        };
        let Some(facts) = package.control_facts(position) else {
            return Err(invalid(
                &mut meter,
                &row_site,
                Code::IllTyped,
                Cause::OperatorIneligible,
                at.key("control"),
            ));
        };
        if !carries_declaration(node, &row.occurrence)
            || package
                .source_position(
                    &row.control,
                    &CheckedOccurrenceRole::Declaration,
                    row.occurrence.ordinal,
                )
                .is_none()
        {
            return Err(invalid(
                &mut meter,
                &row_site,
                Code::InvalidPackage,
                Cause::InvalidValue,
                at.key("occurrence"),
            ));
        }
        if usize::try_from(row.canonical_index).map_or(true, |wanted| wanted >= facts.related) {
            return Err(invalid(
                &mut meter,
                &row_site,
                Code::InvalidPackage,
                Cause::InvalidValue,
                at.key("canonical_index"),
            ));
        }

        // Key, against the previous row, by (control, occurrence ordinal,
        // source ordinal): an equal key before an order defect.
        let mut starts_group = true;
        if let Some(earlier) = previous {
            let key = (&row.control, row.occurrence.ordinal, row.source_ordinal);
            let earlier_key = (
                &earlier.control,
                earlier.occurrence.ordinal,
                earlier.source_ordinal,
            );
            match key.cmp(&earlier_key) {
                std::cmp::Ordering::Equal => {
                    return Err(invalid(
                        &mut meter,
                        &row_site,
                        Code::InvalidPackage,
                        Cause::ConflictingDefinition,
                        at,
                    ));
                }
                std::cmp::Ordering::Less => {
                    return Err(invalid(
                        &mut meter,
                        &row_site,
                        Code::InvalidPackage,
                        Cause::InvalidValue,
                        at,
                    ));
                }
                std::cmp::Ordering::Greater => {}
            }
            starts_group = (&row.control, row.occurrence.ordinal)
                != (&earlier.control, earlier.occurrence.ordinal);
        }

        // Ordinal: 0 for the first row of a group, then one more each row.
        let expected = match (starts_group, previous) {
            (false, Some(earlier)) => earlier.source_ordinal.saturating_add(1),
            _ => 0,
        };
        if row.source_ordinal != expected {
            return Err(invalid(
                &mut meter,
                &row_site,
                Code::InvalidPackage,
                Cause::InvalidValue,
                at.key("source_ordinal"),
            ));
        }
        groups += usize::from(starts_group);
        previous = Some(row);
    }

    // The typed view: one block of `G` bytes, charged in full before it is
    // requested. A member with no row requests none.
    let view_site = site(0, "protocol_correlation_view", member_path());
    stage_storage::reserve_items::<Group>(&mut meter, &view_site, groups)?;
    let mut view: Vec<Group> = Vec::with_capacity(groups);
    for (index, row) in rows.iter().enumerate() {
        if let Some(last) = view.last_mut() {
            let earlier = &rows[last.row_start];
            if (&earlier.control, earlier.occurrence.ordinal)
                == (&row.control, row.occurrence.ordinal)
            {
                last.row_len += 1;
                continue;
            }
        }
        // The lookup of the group's first row: its control text was paid by
        // that row's own charge.
        let source_entry = package
            .source_position(
                &row.control,
                &CheckedOccurrenceRole::Declaration,
                row.occurrence.ordinal,
            )
            .ok_or_else(|| {
                invalid(
                    &mut meter,
                    &view_site,
                    Code::InvalidPackage,
                    Cause::InvalidValue,
                    member_path().index(index).key("occurrence"),
                )
            })?;
        view.push(Group {
            row_start: index,
            row_len: 1,
            source_entry,
        });
    }
    coverage(package, rows, &view, &mut meter)?;
    Ok(Some(view.into_boxed_slice()))
}

/// Whether `node` carries the `declaration` occurrence `occurrence`.
fn carries_declaration(node: &CheckedSemanticNodeV2, occurrence: &CheckedOccurrence) -> bool {
    occurrence.role == CheckedOccurrenceRole::Declaration
        && node
            .occurrences
            .iter()
            .any(|held| held.role == occurrence.role && held.ordinal == occurrence.ordinal)
}

/// The coverage pass: every control node in ascending structural node-id order,
/// charged before it is inspected, against the groups in table order.
fn coverage(
    package: &CheckedPackageV2,
    rows: &[ProtocolCorrelationRow],
    groups: &[Group],
    meter: &mut WorkMeter<'_>,
) -> Result<(), ValidationFailure> {
    let nodes = &package.wire.semantic_graph.nodes;
    let mut next = 0_usize;
    for &position in package.correlation.controls.iter() {
        let node = &nodes[position];
        let site = site(position, "protocol_correlation_scan", member_path());
        let bn = reference_bytes(&node.node_id);
        let facts = package.control_facts(position);
        let occurrences = node
            .occurrences
            .iter()
            .filter(|held| held.role == CheckedOccurrenceRole::Declaration)
            .count();
        let charge = 1_u64
            .saturating_add(bn.saturating_mul(2))
            .saturating_add(facts.map_or(0, |facts| (occurrences + facts.related) as u64));
        stage_storage::work(meter, &site, charge)?;
        let Some(facts) = facts else {
            continue;
        };
        // The group window of this control: groups are in the node order.
        let window_start = next;
        while groups
            .get(next)
            .is_some_and(|group| rows[group.row_start].control == node.node_id)
        {
            next += 1;
        }
        let window = &groups[window_start..next];
        if facts.related == 0 {
            continue;
        }
        let flag_site = self::site(position, "protocol_correlation_flags", member_path());
        stage_storage::reserve(meter, &flag_site, facts.related)?;
        let mut covered = vec![false; facts.related];
        for held in node
            .occurrences
            .iter()
            .filter(|held| held.role == CheckedOccurrenceRole::Declaration)
        {
            covered.fill(false);
            if let Some(group) = window
                .iter()
                .find(|group| rows[group.row_start].occurrence.ordinal == held.ordinal)
            {
                for row in &rows[group.row_start..group.row_start + group.row_len] {
                    if let Some(flag) = usize::try_from(row.canonical_index)
                        .ok()
                        .and_then(|at| covered.get_mut(at))
                    {
                        *flag = true;
                    }
                }
            }
            if let Some(first) = covered.iter().position(|flag| !flag) {
                return Err(uncovered(
                    meter,
                    &site,
                    &node.node_id,
                    held.ordinal,
                    first as u64,
                ));
            }
        }
    }
    Ok(())
}

/// The coverage refusal: `invalid_package`/`invalid-value` at the member,
/// retaining the control, the occurrence and the first uncovered
/// `canonical_index`. Its owned detail is charged with the diagnostic.
fn uncovered(
    meter: &mut WorkMeter<'_>,
    site: &Site<'_>,
    control: &CheckedNodeId,
    ordinal: u64,
    canonical_index: u64,
) -> ValidationFailure {
    let detail_bytes = std::mem::size_of::<CheckedProtocolRefusalDetail>()
        .saturating_add(stage_storage::node_id_clone_bytes(control));
    let refusal = Refusal::new(Code::InvalidPackage, member_path())
        .at_node(control)
        .because(Cause::InvalidValue)
        .with_extra(detail_bytes);
    let mut failure = stage_storage::refuse(meter, site, refusal);
    if let ValidationFailure::Refused(refused) = &mut failure {
        refused.protocol_detail = Some(Box::new(
            CheckedProtocolRefusalDetail::CorrelationCoverage {
                control: control.clone(),
                occurrence: CheckedOccurrence {
                    role: CheckedOccurrenceRole::Declaration,
                    ordinal,
                },
                canonical_index,
            },
        ));
    }
    failure
}
