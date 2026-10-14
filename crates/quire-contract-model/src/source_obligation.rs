// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-052 source-derived correspondence census for proof-coverage consumers.
//!
//! This module records owner inventories and explicit source-contract assertions.
//! It does not infer relationships from evidence, generated artifacts, or text.

use std::{collections::BTreeMap, fmt};

use thiserror::Error;

use crate::{
    CheckedNodeId, CheckedOccurrence, CheckedSemanticId, ClauseRef, RequirementRef, SourceIdentity,
};

/// The owner whose complete inventory an envelope carries.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourceInventoryOwner {
    /// Contract IR V2 semantic graph and source occurrences.
    IrV2,
    /// QSL checked requirements and native pending obligations.
    Qsl,
    /// Selected `quire-code-rs` modules and functions.
    QuireCodeRs,
}

/// The complete named scope an owner must enumerate.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourceInventoryScope {
    /// Every semantic node and source occurrence in the selected V2 graph.
    IrV2SemanticGraphAndOccurrences,
    /// Every checked requirement and native pending obligation for the request.
    QslCheckedRequirementsAndPendingObligations,
    /// Every selected production module and function fact.
    QuireCodeRsProductionModulesAndFunctions,
}

impl SourceInventoryOwner {
    fn required_scope(self) -> SourceInventoryScope {
        match self {
            Self::IrV2 => SourceInventoryScope::IrV2SemanticGraphAndOccurrences,
            Self::Qsl => SourceInventoryScope::QslCheckedRequirementsAndPendingObligations,
            Self::QuireCodeRs => SourceInventoryScope::QuireCodeRsProductionModulesAndFunctions,
        }
    }
}

/// Typed cause explaining why owner enumeration did not exhaust its scope.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SourceInventoryIncompleteReason {
    /// Enumeration stopped before normal exhaustion.
    EarlyStop {
        /// Owner whose enumeration stopped.
        owner: SourceInventoryOwner,
    },
    /// The owner's selected input could not be parsed.
    ParseFailed {
        /// Owner whose selected input failed to parse.
        owner: SourceInventoryOwner,
        /// Owner-native parse cause retained by the adapter.
        cause: Box<str>,
    },
    /// The owner request failed.
    RequestFailed {
        /// Owner whose request failed.
        owner: SourceInventoryOwner,
        /// Owner-native request cause retained by the adapter.
        cause: Box<str>,
    },
    /// A selected production module was unavailable or could not be parsed.
    ModuleUnavailable {
        /// Selected module that was unavailable.
        module: SourceProductionModuleId,
        /// Owner-native module cause retained by the adapter.
        cause: Box<str>,
    },
}

/// Owner-emitted completeness state for one inventory envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceInventoryCompleteness {
    /// The owner normally exhausted the entire named scope.
    Complete,
    /// Enumeration did not establish a complete scope.
    Incomplete(SourceInventoryIncompleteReason),
}

/// A production module identity supplied by the native code owner adapter.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceProductionModuleId(Box<str>);

impl SourceProductionModuleId {
    /// Retain the owner-native module key without interpreting it.
    pub fn new(value: impl Into<Box<str>>) -> Self {
        Self(value.into())
    }

    /// Borrow the owner-native module key.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SourceProductionModuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A production function identity scoped by its owning module.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceProductionFunctionId {
    module: SourceProductionModuleId,
    function: Box<str>,
}

impl SourceProductionFunctionId {
    /// Retain owner-native module and function keys.
    pub fn new(module: SourceProductionModuleId, function: impl Into<Box<str>>) -> Self {
        Self {
            module,
            function: function.into(),
        }
    }

    /// Borrow this function's module identity.
    pub fn module(&self) -> &SourceProductionModuleId {
        &self.module
    }

    /// Borrow the owner-native function key.
    pub fn function(&self) -> &str {
        &self.function
    }
}

/// One IR V2 source occurrence, retaining its native node and occurrence keys.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SourceSemanticOccurrenceId {
    node: CheckedNodeId,
    occurrence: CheckedOccurrence,
}

impl SourceSemanticOccurrenceId {
    /// Retain native IR V2 node and occurrence identities.
    pub fn new(node: CheckedNodeId, occurrence: CheckedOccurrence) -> Self {
        Self { node, occurrence }
    }

    /// Borrow the native semantic node identity.
    pub fn node(&self) -> &CheckedNodeId {
        &self.node
    }

    /// Borrow the native source occurrence identity.
    pub fn occurrence(&self) -> &CheckedOccurrence {
        &self.occurrence
    }
}

/// Typed identity of an item in an owner inventory.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SourceInventoryIdentity {
    /// A checked source criterion, identified by its native clause reference.
    Criterion(ClauseRef),
    /// A complete checked requirement record reported by QSL.
    RequirementRecord(RequirementRef),
    /// A semantic node from the selected IR V2 graph.
    SemanticNode {
        /// Native checked-node identity.
        identity: CheckedNodeId,
        /// Native semantic identity carried by the node.
        semantic: CheckedSemanticId,
    },
    /// A source occurrence from the selected IR V2 graph.
    SemanticOccurrence(SourceSemanticOccurrenceId),
    /// A native QSL pending-obligation identity.
    PendingObligation(CheckedSemanticId),
    /// A selected production module.
    ProductionModule(SourceProductionModuleId),
    /// A selected production function.
    ProductionFunction(SourceProductionFunctionId),
}

impl SourceInventoryIdentity {
    fn owner(&self) -> SourceInventoryOwner {
        match self {
            Self::SemanticNode { .. } | Self::SemanticOccurrence(_) => SourceInventoryOwner::IrV2,
            Self::Criterion(_) | Self::RequirementRecord(_) | Self::PendingObligation(_) => {
                SourceInventoryOwner::Qsl
            }
            Self::ProductionModule(_) | Self::ProductionFunction(_) => {
                SourceInventoryOwner::QuireCodeRs
            }
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum InventoryKey {
    Criterion(ClauseRef),
    RequirementRecord(RequirementRef),
    SemanticNode(CheckedNodeId),
    SemanticOccurrence(SourceSemanticOccurrenceId),
    PendingObligation(CheckedSemanticId),
    ProductionModule(SourceProductionModuleId),
    ProductionFunction(SourceProductionFunctionId),
}

impl From<&SourceInventoryIdentity> for InventoryKey {
    fn from(identity: &SourceInventoryIdentity) -> Self {
        match identity {
            SourceInventoryIdentity::Criterion(id) => Self::Criterion(id.clone()),
            SourceInventoryIdentity::RequirementRecord(id) => Self::RequirementRecord(id.clone()),
            SourceInventoryIdentity::SemanticNode { identity, .. } => {
                Self::SemanticNode(identity.clone())
            }
            SourceInventoryIdentity::SemanticOccurrence(id) => Self::SemanticOccurrence(id.clone()),
            SourceInventoryIdentity::PendingObligation(id) => Self::PendingObligation(id.clone()),
            SourceInventoryIdentity::ProductionModule(id) => Self::ProductionModule(id.clone()),
            SourceInventoryIdentity::ProductionFunction(id) => Self::ProductionFunction(id.clone()),
        }
    }
}

/// Disposition of a preserved owner inventory row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceInventoryDisposition {
    /// At least one accepted assertion uses this entity.
    Mapped,
    /// The selected entity has no authored mapping.
    ZeroMappings(SourceMissingMappingReason),
    /// The row belongs to a different selected source.
    ForeignInventory,
    /// The same typed identity was supplied with conflicting facts.
    ConflictingIdentity,
}

/// Typed explanation for a source entity that has no production mapping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceMissingMappingReason {
    /// No authored assertion maps the entity.
    NoAuthoredMapping,
    /// The source contract explicitly records that no production function exists.
    ExplicitNoProductionFunction,
}

/// One retained item and its selected-source provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceInventoryRow {
    owner: SourceInventoryOwner,
    source: SourceIdentity,
    identity: SourceInventoryIdentity,
    disposition: SourceInventoryDisposition,
}

impl SourceInventoryRow {
    /// Build an owner-adapter row; the census constructor assigns its disposition.
    pub fn new(source: SourceIdentity, identity: SourceInventoryIdentity) -> Self {
        let owner = identity.owner();
        Self {
            owner,
            source,
            identity,
            disposition: SourceInventoryDisposition::ZeroMappings(
                SourceMissingMappingReason::NoAuthoredMapping,
            ),
        }
    }

    /// Return the inventory owner.
    pub const fn owner(&self) -> SourceInventoryOwner {
        self.owner
    }

    /// Borrow the selected source represented by this row.
    pub fn source(&self) -> &SourceIdentity {
        &self.source
    }

    /// Borrow this row's typed owner identity.
    pub fn identity(&self) -> &SourceInventoryIdentity {
        &self.identity
    }

    /// Borrow this row's computed mapping disposition.
    pub fn disposition(&self) -> &SourceInventoryDisposition {
        &self.disposition
    }
}

/// One complete or incomplete inventory envelope from an owner adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceInventoryEnvelope {
    owner: SourceInventoryOwner,
    source: SourceIdentity,
    scope: SourceInventoryScope,
    item_count: usize,
    completeness: SourceInventoryCompleteness,
    items: Vec<SourceInventoryRow>,
}

impl SourceInventoryEnvelope {
    /// Retain an owner-emitted inventory result and its declared count.
    pub fn new(
        owner: SourceInventoryOwner,
        source: SourceIdentity,
        scope: SourceInventoryScope,
        item_count: usize,
        completeness: SourceInventoryCompleteness,
        items: Vec<SourceInventoryRow>,
    ) -> Self {
        Self {
            owner,
            source,
            scope,
            item_count,
            completeness,
            items,
        }
    }

    /// Return the owner named by this envelope.
    pub const fn owner(&self) -> SourceInventoryOwner {
        self.owner
    }
    /// Borrow the envelope's source selection.
    pub fn source(&self) -> &SourceIdentity {
        &self.source
    }
    /// Return the enumerated owner scope.
    pub const fn scope(&self) -> SourceInventoryScope {
        self.scope
    }
    /// Return the count reported by the owner adapter.
    pub const fn item_count(&self) -> usize {
        self.item_count
    }
    /// Borrow the owner-emitted completeness state.
    pub fn completeness(&self) -> &SourceInventoryCompleteness {
        &self.completeness
    }
    /// Borrow all retained inventory rows.
    pub fn items(&self) -> &[SourceInventoryRow] {
        &self.items
    }
}

/// Stable identity assigned by the source-contract author to one assertion.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceAssertionId(Box<str>);

impl SourceAssertionId {
    /// Retain the author's assertion key without parsing it.
    pub fn new(value: impl Into<Box<str>>) -> Self {
        Self(value.into())
    }

    /// Borrow the owner's assertion key.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Identity of a source-contract author in the selected source provenance.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceContractAuthorId(Box<str>);

impl SourceContractAuthorId {
    /// Retain the author's native identity without parsing it.
    pub fn new(value: impl Into<Box<str>>) -> Self {
        Self(value.into())
    }

    /// Borrow the author's native key.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Provenance attached to a source-contract assertion.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceAssertionProvenance {
    /// Source selection to which the author attached this assertion.
    pub source: SourceIdentity,
    /// Source-contract author identity.
    pub author: SourceContractAuthorId,
}

/// An explicit many-to-many source-contract assertion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceCorrespondenceAssertion {
    id: SourceAssertionId,
    source: SourceIdentity,
    criterion: ClauseRef,
    occurrence: SourceSemanticOccurrenceId,
    production_function: Option<SourceProductionFunctionId>,
    provenance: SourceAssertionProvenance,
}

impl SourceCorrespondenceAssertion {
    /// Construct an explicit criterion-to-occurrence assertion.
    ///
    /// `production_function: None` records the author's explicit absence of a
    /// production function for this occurrence.
    pub fn new(
        id: SourceAssertionId,
        source: SourceIdentity,
        criterion: ClauseRef,
        occurrence: SourceSemanticOccurrenceId,
        production_function: Option<SourceProductionFunctionId>,
        provenance: SourceAssertionProvenance,
    ) -> Self {
        Self {
            id,
            source,
            criterion,
            occurrence,
            production_function,
            provenance,
        }
    }

    /// Borrow the author's stable assertion identity.
    pub fn id(&self) -> &SourceAssertionId {
        &self.id
    }
    /// Borrow the source named by this assertion.
    pub fn source(&self) -> &SourceIdentity {
        &self.source
    }
    /// Borrow the criterion endpoint.
    pub fn criterion(&self) -> &ClauseRef {
        &self.criterion
    }
    /// Borrow the semantic occurrence endpoint.
    pub fn occurrence(&self) -> &SourceSemanticOccurrenceId {
        &self.occurrence
    }
    /// Borrow the explicit production function, if present.
    pub fn production_function(&self) -> Option<&SourceProductionFunctionId> {
        self.production_function.as_ref()
    }
    /// Borrow the original source-contract provenance.
    pub fn provenance(&self) -> &SourceAssertionProvenance {
        &self.provenance
    }
}

/// Rejection reason for an assertion that cannot enter the accepted relation set.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SourceCorrespondenceRejectionReason {
    /// The assertion names another selected source.
    ForeignSource,
    /// The criterion endpoint is not in the selected inventory.
    DanglingCriterion,
    /// The occurrence endpoint is not in the selected inventory.
    DanglingOccurrence,
    /// The production function endpoint is not in the selected inventory.
    DanglingProductionSymbol,
    /// One assertion identity carries different relation facts.
    ConflictingRelation,
    /// One assertion identity carries different provenance facts.
    ConflictingProvenance,
}

/// An inspectable assertion rejected from the accepted relation set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceCorrespondenceRejection {
    assertion: SourceCorrespondenceAssertion,
    reason: SourceCorrespondenceRejectionReason,
}

impl SourceCorrespondenceRejection {
    /// Borrow the rejected assertion, including its original provenance.
    pub fn assertion(&self) -> &SourceCorrespondenceAssertion {
        &self.assertion
    }
    /// Return the deterministic typed rejection reason.
    pub const fn reason(&self) -> SourceCorrespondenceRejectionReason {
        self.reason
    }
}

/// Accepted explicit criterion-to-occurrence-to-function relationship.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceObligationRow {
    criterion: ClauseRef,
    occurrence: SourceSemanticOccurrenceId,
    production_function: Option<SourceProductionFunctionId>,
    provenance: SourceAssertionProvenance,
}

impl SourceObligationRow {
    /// Borrow the owning criterion identity.
    pub fn criterion(&self) -> &ClauseRef {
        &self.criterion
    }
    /// Borrow the semantic occurrence identity.
    pub fn occurrence(&self) -> &SourceSemanticOccurrenceId {
        &self.occurrence
    }
    /// Borrow the explicit production-function mapping, if one was authored.
    pub fn production_function(&self) -> Option<&SourceProductionFunctionId> {
        self.production_function.as_ref()
    }
    /// Borrow the author-supplied assertion provenance.
    pub fn provenance(&self) -> &SourceAssertionProvenance {
        &self.provenance
    }
}

/// Complete, immutable census over one selected source and three owner scopes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceObligationCensus {
    source: SourceIdentity,
    inventory_rows: Vec<SourceInventoryRow>,
    correspondence_rows: Vec<SourceObligationRow>,
    rejected_assertions: Vec<SourceCorrespondenceRejection>,
}

impl SourceObligationCensus {
    /// Borrow the source selection represented by this census.
    pub fn source(&self) -> &SourceIdentity {
        &self.source
    }
    /// Borrow every deduplicated inventory row, including zero-mapping rows.
    pub fn inventory_rows(&self) -> &[SourceInventoryRow] {
        &self.inventory_rows
    }
    /// Borrow all accepted explicit correspondence rows.
    pub fn correspondence_rows(&self) -> &[SourceObligationRow] {
        &self.correspondence_rows
    }
    /// Borrow all rejected assertions and their typed dispositions.
    pub fn rejected_assertions(&self) -> &[SourceCorrespondenceRejection] {
        &self.rejected_assertions
    }
}

/// Typed failure to establish all three complete owner inventories.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SourceCorrespondenceError {
    /// The owner envelope was not supplied.
    #[error("missing {owner:?} inventory")]
    MissingOwnerInventory {
        /// Owner whose required envelope was absent.
        owner: SourceInventoryOwner,
    },
    /// The owner did not exhaust its selected scope.
    #[error("incomplete {owner:?} inventory")]
    IncompleteOwnerInventory {
        /// Owner whose scope was incomplete.
        owner: SourceInventoryOwner,
        /// Original typed incomplete cause from the adapter.
        cause: SourceInventoryIncompleteReason,
    },
    /// The envelope or one of its rows names another source.
    #[error("{owner:?} inventory source mismatch")]
    InventorySourceMismatch {
        /// Owner whose source identity differed.
        owner: SourceInventoryOwner,
        /// Top-level selected source expected by this census.
        expected: SourceIdentity,
        /// Source identity supplied by the owner.
        observed: SourceIdentity,
    },
    /// The owner envelope names a scope not required for its input position.
    #[error("{owner:?} inventory scope mismatch")]
    InventoryScopeMismatch {
        /// Owner position being validated.
        owner: SourceInventoryOwner,
        /// Required owner scope.
        expected: SourceInventoryScope,
        /// Supplied scope or scope implied by the supplied owner.
        observed: SourceInventoryScope,
    },
    /// Declared item count differs from retained rows.
    #[error("{owner:?} inventory item count mismatch")]
    InventoryItemCountMismatch {
        /// Owner whose retained item count differed.
        owner: SourceInventoryOwner,
        /// Count declared by the owner envelope.
        expected: usize,
        /// Number of rows retained in the envelope.
        observed: usize,
    },
}

/// Typed input to the complete source-obligation census constructor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceObligationInput {
    source: SourceIdentity,
    ir_v2: Option<SourceInventoryEnvelope>,
    qsl: Option<SourceInventoryEnvelope>,
    quire_code_rs: Option<SourceInventoryEnvelope>,
    assertions: Vec<SourceCorrespondenceAssertion>,
}

impl SourceObligationInput {
    /// Construct a census request from the selection, three optional envelopes,
    /// and source-author assertions.
    pub fn new(
        source: SourceIdentity,
        ir_v2: Option<SourceInventoryEnvelope>,
        qsl: Option<SourceInventoryEnvelope>,
        quire_code_rs: Option<SourceInventoryEnvelope>,
        assertions: Vec<SourceCorrespondenceAssertion>,
    ) -> Self {
        Self {
            source,
            ir_v2,
            qsl,
            quire_code_rs,
            assertions,
        }
    }
}

/// Validate owner completeness and construct the deterministic census.
pub fn build_source_obligation_census(
    input: SourceObligationInput,
) -> Result<SourceObligationCensus, SourceCorrespondenceError> {
    let mut inventory = Vec::new();
    for (owner, envelope) in [
        (SourceInventoryOwner::IrV2, input.ir_v2),
        (SourceInventoryOwner::Qsl, input.qsl),
        (SourceInventoryOwner::QuireCodeRs, input.quire_code_rs),
    ] {
        let envelope =
            envelope.ok_or(SourceCorrespondenceError::MissingOwnerInventory { owner })?;
        if envelope.owner != owner {
            return Err(SourceCorrespondenceError::InventoryScopeMismatch {
                owner,
                expected: owner.required_scope(),
                observed: envelope.owner.required_scope(),
            });
        }
        if envelope.source != input.source {
            return Err(SourceCorrespondenceError::InventorySourceMismatch {
                owner,
                expected: input.source.clone(),
                observed: envelope.source,
            });
        }
        if envelope.scope != owner.required_scope() {
            return Err(SourceCorrespondenceError::InventoryScopeMismatch {
                owner,
                expected: owner.required_scope(),
                observed: envelope.scope,
            });
        }
        if let SourceInventoryCompleteness::Incomplete(cause) = envelope.completeness {
            return Err(SourceCorrespondenceError::IncompleteOwnerInventory { owner, cause });
        }
        if envelope.item_count != envelope.items.len() {
            return Err(SourceCorrespondenceError::InventoryItemCountMismatch {
                owner,
                expected: envelope.item_count,
                observed: envelope.items.len(),
            });
        }
        inventory.extend(envelope.items);
    }

    // Every owner-position row is checked before joining; a wrong typed item
    // cannot masquerade as a complete entry for another owner.
    inventory.sort_by(|a, b| {
        (&a.owner, &a.identity, &a.source).cmp(&(&b.owner, &b.identity, &b.source))
    });
    let mut normalized = Vec::with_capacity(inventory.len());
    for row in inventory {
        if row.owner != row.identity.owner() {
            return Err(SourceCorrespondenceError::InventoryScopeMismatch {
                owner: row.owner,
                expected: row.owner.required_scope(),
                observed: row.identity.owner().required_scope(),
            });
        }
        normalized.push(row);
    }
    let mut inventory = deduplicate_inventory(normalized, &input.source);
    let selected = inventory
        .iter()
        .filter(|row| {
            row.disposition != SourceInventoryDisposition::ForeignInventory
                && row.disposition != SourceInventoryDisposition::ConflictingIdentity
        })
        .cloned()
        .collect::<Vec<_>>();
    let criteria = selected
        .iter()
        .filter_map(|row| match &row.identity {
            SourceInventoryIdentity::Criterion(id) => Some(id.clone()),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    let semantic_nodes = selected
        .iter()
        .filter_map(|row| match &row.identity {
            SourceInventoryIdentity::SemanticNode { identity, .. } => Some(identity.clone()),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    let occurrences = selected
        .iter()
        .filter_map(|row| match &row.identity {
            SourceInventoryIdentity::SemanticOccurrence(id)
                if semantic_nodes.contains(&id.node) =>
            {
                Some(id.clone())
            }
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    let production_modules = selected
        .iter()
        .filter_map(|row| match &row.identity {
            SourceInventoryIdentity::ProductionModule(id) => Some(id.clone()),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    let functions = selected
        .iter()
        .filter_map(|row| match &row.identity {
            SourceInventoryIdentity::ProductionFunction(id)
                if production_modules.contains(&id.module) =>
            {
                Some(id.clone())
            }
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();

    let mut assertion_groups: BTreeMap<SourceAssertionId, Vec<SourceCorrespondenceAssertion>> =
        BTreeMap::new();
    for assertion in input.assertions {
        assertion_groups
            .entry(assertion.id.clone())
            .or_default()
            .push(assertion);
    }
    let mut accepted_assertions = Vec::new();
    let mut rejected = Vec::new();
    for (_, mut group) in assertion_groups {
        group.sort_by(|a, b| assertion_sort_key(a).cmp(&assertion_sort_key(b)));
        group.dedup();
        if group
            .iter()
            .any(|a| a.source != input.source || a.provenance.source != input.source)
        {
            rejected.extend(
                group
                    .into_iter()
                    .map(|assertion| SourceCorrespondenceRejection {
                        assertion,
                        reason: SourceCorrespondenceRejectionReason::ForeignSource,
                    }),
            );
            continue;
        }
        let relation_facts = group
            .iter()
            .map(|a| (&a.criterion, &a.occurrence, &a.production_function))
            .collect::<std::collections::BTreeSet<_>>();
        if relation_facts.len() > 1 {
            rejected.extend(
                group
                    .into_iter()
                    .map(|assertion| SourceCorrespondenceRejection {
                        assertion,
                        reason: SourceCorrespondenceRejectionReason::ConflictingRelation,
                    }),
            );
            continue;
        }
        let provenance_facts = group
            .iter()
            .map(|a| &a.provenance)
            .collect::<std::collections::BTreeSet<_>>();
        if provenance_facts.len() > 1 {
            rejected.extend(
                group
                    .into_iter()
                    .map(|assertion| SourceCorrespondenceRejection {
                        assertion,
                        reason: SourceCorrespondenceRejectionReason::ConflictingProvenance,
                    }),
            );
            continue;
        }
        let Some(assertion) = group.into_iter().next() else {
            continue;
        };
        let reason = if !criteria.contains(&assertion.criterion) {
            Some(SourceCorrespondenceRejectionReason::DanglingCriterion)
        } else if !occurrences.contains(&assertion.occurrence) {
            Some(SourceCorrespondenceRejectionReason::DanglingOccurrence)
        } else if assertion
            .production_function
            .as_ref()
            .is_some_and(|function| !functions.contains(function))
        {
            Some(SourceCorrespondenceRejectionReason::DanglingProductionSymbol)
        } else {
            None
        };
        if let Some(reason) = reason {
            rejected.push(SourceCorrespondenceRejection { assertion, reason });
        } else {
            accepted_assertions.push(assertion);
        }
    }
    let mut relation_groups: BTreeMap<
        (
            ClauseRef,
            SourceSemanticOccurrenceId,
            Option<SourceProductionFunctionId>,
        ),
        Vec<SourceCorrespondenceAssertion>,
    > = BTreeMap::new();
    for assertion in accepted_assertions {
        relation_groups
            .entry((
                assertion.criterion.clone(),
                assertion.occurrence.clone(),
                assertion.production_function.clone(),
            ))
            .or_default()
            .push(assertion);
    }
    let mut accepted = Vec::new();
    for (_, mut group) in relation_groups {
        group.sort_by(|a, b| assertion_sort_key(a).cmp(&assertion_sort_key(b)));
        group.dedup();
        let provenance_facts = group
            .iter()
            .map(|assertion| &assertion.provenance)
            .collect::<std::collections::BTreeSet<_>>();
        if provenance_facts.len() > 1 {
            rejected.extend(
                group
                    .into_iter()
                    .map(|assertion| SourceCorrespondenceRejection {
                        assertion,
                        reason: SourceCorrespondenceRejectionReason::ConflictingProvenance,
                    }),
            );
        } else if let Some(assertion) = group.into_iter().next() {
            accepted.push(SourceObligationRow {
                criterion: assertion.criterion,
                occurrence: assertion.occurrence,
                production_function: assertion.production_function,
                provenance: assertion.provenance,
            });
        }
    }
    accepted.sort();
    rejected.sort_by(|a, b| {
        (assertion_sort_key(&a.assertion), a.reason)
            .cmp(&(assertion_sort_key(&b.assertion), b.reason))
    });
    rejected.dedup();

    let mapped_criteria = accepted
        .iter()
        .map(|r| r.criterion.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let mapped_requirements = mapped_criteria
        .iter()
        .map(|criterion| criterion.requirement().clone())
        .collect::<std::collections::BTreeSet<_>>();
    let mapped_occurrences = accepted
        .iter()
        .filter(|r| r.production_function.is_some())
        .map(|r| r.occurrence.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let mapped_functions = accepted
        .iter()
        .filter_map(|r| r.production_function.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let mapped_modules = mapped_functions
        .iter()
        .map(|f| f.module.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let explicit_no_function = accepted
        .iter()
        .filter(|r| r.production_function.is_none())
        .map(|r| r.occurrence.clone())
        .collect::<std::collections::BTreeSet<_>>();
    for row in &mut inventory {
        if row.disposition == SourceInventoryDisposition::ForeignInventory
            || row.disposition == SourceInventoryDisposition::ConflictingIdentity
        {
            continue;
        }
        let mapped = match &row.identity {
            SourceInventoryIdentity::Criterion(id) => mapped_criteria.contains(id),
            SourceInventoryIdentity::RequirementRecord(id) => mapped_requirements.contains(id),
            SourceInventoryIdentity::SemanticOccurrence(id) => mapped_occurrences.contains(id),
            SourceInventoryIdentity::ProductionFunction(id) => mapped_functions.contains(id),
            SourceInventoryIdentity::ProductionModule(id) => mapped_modules.contains(id),
            SourceInventoryIdentity::SemanticNode { identity, .. } => accepted
                .iter()
                .any(|r| r.production_function.is_some() && &r.occurrence.node == identity),
            SourceInventoryIdentity::PendingObligation(id) => selected.iter().any(|candidate| matches!(&candidate.identity, SourceInventoryIdentity::SemanticNode { semantic, .. } if semantic == id)),
        };
        row.disposition = if mapped {
            SourceInventoryDisposition::Mapped
        } else if let SourceInventoryIdentity::SemanticOccurrence(id) = &row.identity {
            SourceInventoryDisposition::ZeroMappings(if explicit_no_function.contains(id) {
                SourceMissingMappingReason::ExplicitNoProductionFunction
            } else {
                SourceMissingMappingReason::NoAuthoredMapping
            })
        } else {
            SourceInventoryDisposition::ZeroMappings(SourceMissingMappingReason::NoAuthoredMapping)
        };
    }
    inventory.sort_by(|a, b| {
        (&a.owner, &a.identity, &a.source).cmp(&(&b.owner, &b.identity, &b.source))
    });
    Ok(SourceObligationCensus {
        source: input.source,
        inventory_rows: inventory,
        correspondence_rows: accepted,
        rejected_assertions: rejected,
    })
}

fn assertion_sort_key(
    a: &SourceCorrespondenceAssertion,
) -> (
    &SourceAssertionId,
    &SourceIdentity,
    &ClauseRef,
    &SourceSemanticOccurrenceId,
    &Option<SourceProductionFunctionId>,
    &SourceAssertionProvenance,
) {
    (
        &a.id,
        &a.source,
        &a.criterion,
        &a.occurrence,
        &a.production_function,
        &a.provenance,
    )
}

fn deduplicate_inventory(
    mut rows: Vec<SourceInventoryRow>,
    selected_source: &SourceIdentity,
) -> Vec<SourceInventoryRow> {
    rows.sort_by(|a, b| {
        (
            &a.owner,
            InventoryKey::from(&a.identity),
            &a.source,
            &a.identity,
        )
            .cmp(&(
                &b.owner,
                InventoryKey::from(&b.identity),
                &b.source,
                &b.identity,
            ))
    });
    let mut result = Vec::new();
    let mut index = 0;
    while index < rows.len() {
        let owner = rows[index].owner;
        let identity = InventoryKey::from(&rows[index].identity);
        let mut end = index + 1;
        while end < rows.len()
            && rows[end].owner == owner
            && InventoryKey::from(&rows[end].identity) == identity
        {
            end += 1;
        }
        let group = &rows[index..end];
        let mut selected = group
            .iter()
            .filter(|row| &row.source == selected_source)
            .cloned()
            .collect::<Vec<_>>();
        selected.sort_by(|a, b| a.source.cmp(&b.source));
        selected.dedup();
        let distinct_facts = selected
            .iter()
            .map(|row| &row.identity)
            .collect::<std::collections::BTreeSet<_>>();
        if distinct_facts.len() > 1 {
            result.extend(selected.into_iter().map(|mut row| {
                row.disposition = SourceInventoryDisposition::ConflictingIdentity;
                row
            }));
        } else if let Some(mut row) = selected.pop() {
            row.disposition = SourceInventoryDisposition::ZeroMappings(
                SourceMissingMappingReason::NoAuthoredMapping,
            );
            result.push(row);
        }
        let mut foreign = group
            .iter()
            .filter(|row| &row.source != selected_source)
            .cloned()
            .collect::<Vec<_>>();
        foreign.sort_by(|a, b| {
            (&a.source, &a.owner, &a.identity).cmp(&(&b.source, &b.owner, &b.identity))
        });
        foreign.dedup();
        result.extend(foreign.into_iter().map(|mut row| {
            row.disposition = SourceInventoryDisposition::ForeignInventory;
            row
        }));
        index = end;
    }
    result.sort_by(|a, b| {
        (
            &a.owner,
            InventoryKey::from(&a.identity),
            &a.source,
            &a.identity,
        )
            .cmp(&(
                &b.owner,
                InventoryKey::from(&b.identity),
                &b.source,
                &b.identity,
            ))
    });
    result
}
