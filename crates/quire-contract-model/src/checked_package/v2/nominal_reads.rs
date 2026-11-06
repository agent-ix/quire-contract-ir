// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded borrowed nominal reads for FR-354. Identity payloads originate in
//! the QSL stage; a caller-created row alone grants no producer authority.

use super::super::{
    CheckedDeclaration, CheckedLiteralValue, CheckedNodeId, CheckedNodeOwner,
    CheckedNodePathSegment, CheckedPackageV2, CheckedSemanticNodeV2, CheckedTerm, NativeJoinBudget,
    NativeJoinCharge, NativeJoinChargeKind, NativeJoinLocus, NativeJoinLocusSource,
};
use super::{CheckedRational, DimensionTerm, NominalBodyKind};
use quire_exact::{NodeKey, UnitId};
use std::collections::BTreeSet;

/// A producer-issued identity with its closed nominal form.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NominalIdentity<'a> {
    /// A declared unit, never a derived compound-unit identity.
    Unit(&'a UnitId),
    /// An enum declaration.
    Enum(&'a NodeKey),
}

/// A QSL-stage declaration row carrying canonical IR selection facts.
/// The owner and original declaration are compared with the admitted node;
/// no identity is derived from their display spelling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NominalDeclarationRow<'a> {
    /// Selected structural node.
    pub node: &'a CheckedNodeId,
    /// Selected source/document owner.
    pub owner: &'a CheckedNodeOwner,
    /// Original declaration selected by the producer.
    pub declaration: &'a CheckedDeclaration,
    /// Opaque Exact identity issued by that producer.
    pub identity: NominalIdentity<'a>,
}

/// A QSL-stage member row tied to one selected enum and case.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumMemberRow<'a> {
    /// Admitted enum-value node.
    pub node: &'a CheckedNodeId,
    /// Selected declaring enum node.
    pub declaration: &'a CheckedNodeId,
    /// Producer-selected case, not an identity inferred from a name.
    pub case: &'a str,
    /// Producer-issued enum member key.
    pub identity: &'a NodeKey,
}

/// Stable reason a nominal read cannot return a complete view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NominalReadCode {
    /// The selected structural node is absent.
    MissingNode,
    /// Required QSL-stage identity evidence is absent.
    MissingWitness,
    /// A row names a different owner, declaration or enum case.
    ForeignWitness,
    /// Repeated nodes or competing Exact identities are not unique.
    ConflictingWitness,
    /// The selected admitted form has no faithful nominal read.
    UnsupportedForm,
    /// A dimension, target or selected conversion closure is absent.
    UnresolvedConversion,
    /// Finite caller work allowance was exhausted.
    LimitReached,
}

/// A typed refusal with work accounting and the first failing node locus.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("nominal read {code:?} at {node:?} after {work} work units (limit {limit})")]
pub struct NominalReadError<'a> {
    /// Stable closed refusal code.
    pub code: NominalReadCode,
    /// First selected or referenced node that could not be read.
    pub node: &'a CheckedNodeId,
    /// Successfully charged native work; numeric enum reads include the refused charge.
    pub work: u64,
    /// Caller-supplied work allowance.
    pub limit: u64,
}

struct Budget<'b> {
    work: u64,
    limit: u64,
    native: Option<&'b mut NativeJoinBudget>,
    before: NativeJoinCharge,
}
impl Budget<'_> {
    fn numeric(limit: u64) -> Self {
        Self {
            work: 0,
            limit,
            native: None,
            before: NativeJoinCharge::default(),
        }
    }

    fn native(budget: &mut NativeJoinBudget) -> Budget<'_> {
        let before = budget.begin();
        Budget {
            work: 0,
            limit: budget.remaining_work(),
            native: Some(budget),
            before,
        }
    }

    fn graph_locus(&mut self, position: usize) {
        if let Some(budget) = &mut self.native {
            if budget.active_locus().is_some() {
                budget.set_active_locus(Some(NativeJoinLocus {
                    source: NativeJoinLocusSource::GraphNode(position),
                    form: "selected_unit_definition",
                }));
            }
        }
    }

    fn storage<'a>(
        &mut self,
        amount: u128,
        node: &'a CheckedNodeId,
    ) -> Result<(), NominalReadError<'a>> {
        if self
            .native
            .as_mut()
            .is_some_and(|budget| !budget.charge(NativeJoinChargeKind::Storage, amount))
        {
            return Err(self.error(NominalReadCode::LimitReached, node));
        }
        Ok(())
    }

    fn finish(&mut self) {
        if let Some(budget) = &mut self.native {
            budget.finish(self.before);
        }
    }

    fn error<'a>(&self, code: NominalReadCode, node: &'a CheckedNodeId) -> NominalReadError<'a> {
        NominalReadError {
            code,
            node,
            work: self.work,
            limit: self.limit,
        }
    }
    fn charge<'a>(
        &mut self,
        amount: usize,
        node: &'a CheckedNodeId,
    ) -> Result<(), NominalReadError<'a>> {
        let amount = u64::try_from(amount).unwrap_or(u64::MAX);
        if let Some(budget) = &mut self.native {
            if !budget.charge(NativeJoinChargeKind::Work, u128::from(amount)) {
                self.work = budget.consumed().work - self.before.work;
                return Err(self.error(NominalReadCode::LimitReached, node));
            }
            self.work = budget.consumed().work - self.before.work;
            return Ok(());
        }
        let Some(work) = self.work.checked_add(amount) else {
            self.work = u64::MAX;
            return Err(self.error(NominalReadCode::LimitReached, node));
        };
        self.work = work;
        if self.work > self.limit {
            return Err(self.error(NominalReadCode::LimitReached, node));
        }
        Ok(())
    }
    fn identity<'a>(&mut self, root: &'a CheckedNodeId) -> Result<(), NominalReadError<'a>> {
        self.storage(2, root)?;
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            self.charge(node.package.len().saturating_add(1), root)?;
            for segment in &node.path {
                self.charge(1, root)?;
                match segment {
                    CheckedNodePathSegment::Name(name) => self.charge(name.len(), root)?,
                    CheckedNodePathSegment::Node(child) => {
                        self.storage(1, root)?;
                        pending.push(child);
                    }
                    CheckedNodePathSegment::Index(_) => {}
                }
            }
        }
        Ok(())
    }
    fn node<'a>(
        &mut self,
        package: &'a CheckedPackageV2,
        id: &'a CheckedNodeId,
    ) -> Result<&'a CheckedSemanticNodeV2, NominalReadError<'a>> {
        self.node_with_position(package, id).map(|(_, node)| node)
    }

    fn node_with_position<'a>(
        &mut self,
        package: &'a CheckedPackageV2,
        id: &'a CheckedNodeId,
    ) -> Result<(usize, &'a CheckedSemanticNodeV2), NominalReadError<'a>> {
        // Identity work is paid before comparing the key in the retained index.
        self.identity(id)?;
        let (position, node) = package
            .node_with_position(id)
            .ok_or_else(|| self.error(NominalReadCode::MissingNode, id))?;
        self.graph_locus(position);
        Ok((position, node))
    }
}

/// One unit's admitted conversion edge. All semantic values are borrowed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnitRead<'a> {
    /// Canonical admitted unit node, including owner and original declaration.
    pub node: &'a CheckedSemanticNodeV2,
    /// Producer-issued declared identity.
    pub identity: &'a UnitId,
    /// Admitted dimension node.
    pub dimension: &'a CheckedSemanticNodeV2,
    /// Whether the admitted dimension is declared as a base.
    pub dimension_base: bool,
    /// Canonical dimension powers, also empty for a dimensionless derived dimension.
    pub dimension_terms: &'a [DimensionTerm],
    /// Admitted target node, absent only for a root unit.
    pub target: Option<&'a CheckedSemanticNodeV2>,
    /// Exact conversion scale borrowed from the canonical body.
    pub scale: &'a CheckedRational,
    /// Exact conversion offset borrowed from the canonical body.
    pub offset: &'a CheckedRational,
}

/// One canonical dimension in the selected units' transitive dimension closure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DimensionRead<'a> {
    /// Admitted dimension node with original owner and declaration.
    pub node: &'a CheckedSemanticNodeV2,
    /// Whether this dimension was declared as a base.
    pub base: bool,
    /// Canonical powers, borrowed without rebuilding the dimension expression.
    pub terms: &'a [DimensionTerm],
}

/// A complete selected unit closure, not a second unit graph implementation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnitDefinitionRead<'a> {
    /// Units in the caller's selected order; every target has a witnessed row.
    pub units: Vec<UnitRead<'a>>,
    /// Complete dimension dependency closure, in first-encounter order.
    pub dimensions: Vec<DimensionRead<'a>>,
    /// Charged bounded work, for reproducible exact-limit controls.
    pub work: u64,
}

/// A selected enum member in its admitted authored position.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumMemberRead<'a> {
    /// Canonical admitted member node.
    pub node: &'a CheckedSemanticNodeV2,
    /// Producer-issued member identity.
    pub identity: &'a NodeKey,
    /// Case borrowed from the admitted declaration, in authored order.
    pub case: &'a str,
    /// Zero-based authored member position.
    pub position: usize,
}

/// One selected enum's canonical declaration and complete witnessed members.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumRead<'a> {
    /// Canonical admitted declaration node.
    pub node: &'a CheckedSemanticNodeV2,
    /// Producer-issued declaration identity.
    pub identity: &'a NodeKey,
    /// Admitted ordering flag.
    pub ordered: bool,
    /// Every declared member, in its admitted authored order.
    pub members: Vec<EnumMemberRead<'a>>,
    /// Charged bounded work.
    pub work: u64,
}

fn declaration<'a>(
    package: &'a CheckedPackageV2,
    row: &NominalDeclarationRow<'a>,
    budget: &mut Budget<'_>,
) -> Result<&'a CheckedSemanticNodeV2, NominalReadError<'a>> {
    let node = budget.node(package, row.node)?;
    budget.charge(1, row.node)?;
    match row.owner {
        CheckedNodeOwner::Source {
            authority,
            identity,
        } => {
            budget.charge(authority.len(), row.node)?;
            budget.charge(identity.len(), row.node)?;
        }
        CheckedNodeOwner::Model { identity, node } => {
            budget.charge(identity.len(), row.node)?;
            budget.charge(node.len(), row.node)?;
        }
        CheckedNodeOwner::ModelRoot { identity } => budget.charge(identity.len(), row.node)?,
    }
    for segment in &row.declaration.qualified_name {
        budget.charge(segment.len().saturating_add(1), row.node)?;
    }
    // Nominal source bodies are retained as typed terms. Model-owned nominal
    // forms that need a different retained producer join refuse below.
    if !matches!(node.owner, Some(CheckedNodeOwner::Source { .. })) {
        return Err(budget.error(NominalReadCode::UnsupportedForm, row.node));
    }
    if node.owner.as_ref() != Some(row.owner) || node.declaration.as_ref() != Some(row.declaration)
    {
        return Err(budget.error(NominalReadCode::ForeignWitness, row.node));
    }
    Ok(node)
}

impl CheckedPackageV2 {
    /// Read a complete selected unit conversion closure from QSL-stage rows.
    ///
    /// Every requested unit needs a unique row, and every target must
    /// occur exactly once; roots retain `None`, not an invented self edge.
    /// Opaque Exact identities remain producer facts, as FR-352 specifies.
    /// The caller budget charges traversal work and temporary collection storage.
    pub fn read_unit_definition<'a>(
        &'a self,
        requested: &'a [CheckedNodeId],
        rows: &'a [NominalDeclarationRow<'a>],
        native_budget: &mut NativeJoinBudget,
    ) -> Result<UnitDefinitionRead<'a>, NominalReadError<'a>> {
        self.read_unit_definition_inner(requested, rows, None, None, native_budget)
    }

    pub(crate) fn read_unit_definition_runtime<'a>(
        &'a self,
        requested: &'a [CheckedNodeId],
        rows: &'a [NominalDeclarationRow<'a>],
        positions: &[usize],
        dimension_positions: &mut Vec<usize>,
        native_budget: &mut NativeJoinBudget,
    ) -> Result<UnitDefinitionRead<'a>, NominalReadError<'a>> {
        self.read_unit_definition_inner(
            requested,
            rows,
            Some(positions),
            Some(dimension_positions),
            native_budget,
        )
    }

    fn read_unit_definition_inner<'a>(
        &'a self,
        requested: &'a [CheckedNodeId],
        rows: &'a [NominalDeclarationRow<'a>],
        positions: Option<&[usize]>,
        mut dimension_positions: Option<&mut Vec<usize>>,
        native_budget: &mut NativeJoinBudget,
    ) -> Result<UnitDefinitionRead<'a>, NominalReadError<'a>> {
        let mut budget = Budget::native(native_budget);
        if let Some(position) = positions.and_then(|positions| positions.first()) {
            budget.graph_locus(*position);
        }
        let locus = requested
            .first()
            .or_else(|| rows.first().map(|row| row.node));
        if let Some(locus) = locus {
            budget.storage(4, locus)?;
        }
        let mut pending = Vec::new();
        let mut units = Vec::new();
        let mut nodes = BTreeSet::new();
        let mut identities = BTreeSet::new();
        for (index, row) in rows.iter().enumerate() {
            if let Some(position) = positions.and_then(|positions| positions.get(index)) {
                budget.graph_locus(*position);
            }
            budget.charge(1, row.node)?;
            let node = declaration(self, row, &mut budget)?;
            let NominalIdentity::Unit(identity @ UnitId::Declared(key)) = row.identity else {
                return Err(budget.error(NominalReadCode::UnsupportedForm, row.node));
            };
            budget.storage(2, row.node)?;
            if !nodes.insert(row.node) {
                return Err(budget.error(NominalReadCode::ConflictingWitness, row.node));
            }
            budget.storage(2, row.node)?;
            if !identities.insert(key) {
                return Err(budget.error(NominalReadCode::ConflictingWitness, row.node));
            }
            let CheckedTerm::NominalBody(body) = &node.body else {
                return Err(budget.error(NominalReadCode::UnsupportedForm, row.node));
            };
            let NominalBodyKind::Unit {
                dimension_node_id,
                target_unit_node_id,
                scale,
                offset,
            } = &body.kind
            else {
                return Err(budget.error(NominalReadCode::UnsupportedForm, row.node));
            };
            let (dimension_position, dimension) =
                budget.node_with_position(self, dimension_node_id)?;
            let CheckedTerm::NominalBody(dimension_body) = &dimension.body else {
                return Err(budget.error(NominalReadCode::UnresolvedConversion, dimension_node_id));
            };
            let NominalBodyKind::Dimension {
                base: dimension_base,
                terms: dimension_terms,
            } = &dimension_body.kind
            else {
                return Err(budget.error(NominalReadCode::UnresolvedConversion, dimension_node_id));
            };
            budget.charge(dimension_terms.len(), dimension_node_id)?;
            let target = target_unit_node_id
                .as_ref()
                .map(|id| budget.node(self, id))
                .transpose()?;
            budget.graph_locus(dimension_position);
            budget.storage(2, row.node)?;
            pending.push((dimension_position, dimension));
            if let Some(position) = positions.and_then(|positions| positions.get(index)) {
                budget.graph_locus(*position);
            }
            budget.storage(1, row.node)?;
            units.push(UnitRead {
                node,
                identity,
                dimension,
                dimension_base: *dimension_base,
                dimension_terms,
                target,
                scale,
                offset,
            });
        }
        if let Some(locus) = locus {
            budget.storage(1, locus)?;
        }
        let mut requests = BTreeSet::new();
        for (index, id) in requested.iter().enumerate() {
            if let Some(position) = positions.and_then(|positions| positions.get(index)) {
                budget.graph_locus(*position);
            }
            budget.identity(id)?;
            budget.storage(2, id)?;
            if !requests.insert(id) {
                return Err(budget.error(NominalReadCode::ConflictingWitness, id));
            }
            if !nodes.contains(id) {
                return Err(budget.error(NominalReadCode::MissingWitness, id));
            }
        }
        for (index, row) in rows.iter().enumerate() {
            if let Some(position) = positions.and_then(|positions| positions.get(index)) {
                budget.graph_locus(*position);
            }
            budget.charge(1, row.node)?;
            if !requests.contains(row.node) {
                return Err(budget.error(NominalReadCode::ForeignWitness, row.node));
            }
        }
        // Admission checks conversion consistency/cycles. This read additionally
        // requires a witnessed selected row for every retained target.
        for (index, unit) in units.iter().enumerate() {
            if let Some(position) = positions.and_then(|positions| positions.get(index)) {
                budget.graph_locus(*position);
            }
            budget.charge(1, &unit.node.node_id)?;
            if let Some(target) = unit.target {
                if !nodes.contains(&target.node_id) {
                    return Err(budget.error(NominalReadCode::MissingWitness, &target.node_id));
                }
            }
        }
        if let Some(locus) = locus {
            budget.storage(2, locus)?;
        }
        let mut dimensions = Vec::new();
        let mut seen = BTreeSet::new();
        let mut next = 0;
        while let Some((position, dimension)) = pending.get(next).copied() {
            budget.graph_locus(position);
            next += 1;
            budget.identity(&dimension.node_id)?;
            budget.storage(2, &dimension.node_id)?;
            if !seen.insert(&dimension.node_id) {
                continue;
            }
            let CheckedTerm::NominalBody(body) = &dimension.body else {
                return Err(budget.error(NominalReadCode::UnresolvedConversion, &dimension.node_id));
            };
            let NominalBodyKind::Dimension { base, terms } = &body.kind else {
                return Err(budget.error(NominalReadCode::UnresolvedConversion, &dimension.node_id));
            };
            for term in terms {
                budget.charge(term.exponent.len().saturating_add(1), &dimension.node_id)?;
                let (child_position, child) =
                    budget.node_with_position(self, &term.dimension_node_id)?;
                budget.storage(2, &dimension.node_id)?;
                pending.push((child_position, child));
            }
            budget.graph_locus(position);
            if let Some(metadata) = dimension_positions.as_deref_mut() {
                budget.storage(1, &dimension.node_id)?;
                metadata.push(position);
            }
            budget.storage(1, &dimension.node_id)?;
            dimensions.push(DimensionRead {
                node: dimension,
                base: *base,
                terms,
            });
        }
        let work = budget.work;
        budget.finish();
        Ok(UnitDefinitionRead {
            units,
            dimensions,
            work,
        })
    }

    /// Read one enum and all its witnessed members in canonical authored order.
    /// Missing, competing and foreign-case rows refuse before returning a view.
    pub fn read_enum_index<'a>(
        &'a self,
        selected: &NominalDeclarationRow<'a>,
        rows: &'a [EnumMemberRow<'a>],
        limit: u64,
    ) -> Result<EnumRead<'a>, NominalReadError<'a>> {
        let mut budget = Budget::numeric(limit);
        let node = declaration(self, selected, &mut budget)?;
        let NominalIdentity::Enum(identity) = selected.identity else {
            return Err(budget.error(NominalReadCode::UnsupportedForm, selected.node));
        };
        let CheckedTerm::NominalBody(body) = &node.body else {
            return Err(budget.error(NominalReadCode::UnsupportedForm, selected.node));
        };
        let Some((ordered, cases)) = body.enum_details() else {
            return Err(budget.error(NominalReadCode::UnsupportedForm, selected.node));
        };
        let mut members = Vec::new();
        let mut nodes = BTreeSet::new();
        let mut keys = BTreeSet::new();
        for row in rows {
            budget.identity(row.declaration)?;
            budget.charge(row.case.len().saturating_add(1), row.node)?;
            let mut declared = false;
            for case in cases {
                budget.charge(case.len().saturating_add(1), row.node)?;
                declared |= case.as_ref() == row.case;
            }
            if row.declaration != selected.node || !declared {
                return Err(budget.error(NominalReadCode::ForeignWitness, row.node));
            }
            if !nodes.insert(row.node) || !keys.insert(row.identity) || row.identity == identity {
                return Err(budget.error(NominalReadCode::ConflictingWitness, row.node));
            }
            let member = budget.node(self, row.node)?;
            let Some(literal) = member.body.literal_read() else {
                return Err(budget.error(NominalReadCode::UnsupportedForm, row.node));
            };
            if literal.type_id != selected.node
                || literal.value != CheckedLiteralValue::Enum(row.case)
            {
                return Err(budget.error(NominalReadCode::ForeignWitness, row.node));
            }
        }
        for (position, case) in cases.iter().enumerate() {
            let mut matched = None;
            for row in rows {
                budget.charge(case.len().saturating_add(1), &node.node_id)?;
                if row.case == case.as_ref() {
                    if matched.is_some() {
                        return Err(budget.error(NominalReadCode::ConflictingWitness, row.node));
                    }
                    matched = Some(row);
                }
            }
            let row = matched
                .ok_or_else(|| budget.error(NominalReadCode::MissingWitness, &node.node_id))?;
            let member = budget.node(self, row.node)?;
            members.push(EnumMemberRead {
                node: member,
                identity: row.identity,
                case,
                position,
            });
        }
        Ok(EnumRead {
            node,
            identity,
            ordered,
            members,
            work: budget.work,
        })
    }
}
