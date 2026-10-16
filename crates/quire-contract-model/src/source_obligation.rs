// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-052 typed source-obligation input and owner enumeration boundary.
//!
//! Owner outcomes retain enumerator state, while the census deterministically
//! evaluates four synthetic or owner-issued inventories and authored assertions.
//! External owner traversal remains the responsibility of each owner adapter.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use crate::{
    CheckedNodeId, CheckedOccurrence, CheckedPackageIncomplete, CheckedSemanticId, SourceIdentity,
};

/// The owner whose inventory rows an outcome carries.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourceInventoryOwner {
    /// Quire's selected authored acceptance criteria.
    Quire,
    /// Contract IR V2 semantic graph and source occurrences.
    IrV2,
    /// QSL checked requirements and native pending obligations.
    Qsl,
    /// Selected `quire-code-rs` modules and functions.
    QuireCodeRs,
}

/// The named scope an owner reports rows for.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourceInventoryScope {
    /// Every authored criterion in the selected Quire corpus and obligation scope.
    QuireSelectedAuthoredCriteria,
    /// Every semantic node and source occurrence in the selected V2 graph.
    IrV2SemanticGraphAndOccurrences,
    /// Every checked requirement and native pending obligation for the request.
    QslCheckedRequirementsAndPendingObligations,
    /// Every selected production module and function fact.
    QuireCodeRsProductionModulesAndFunctions,
}

impl SourceInventoryScope {
    /// The only owner that can enumerate this named scope.
    pub const fn owner(self) -> SourceInventoryOwner {
        match self {
            Self::QuireSelectedAuthoredCriteria => SourceInventoryOwner::Quire,
            Self::IrV2SemanticGraphAndOccurrences => SourceInventoryOwner::IrV2,
            Self::QslCheckedRequirementsAndPendingObligations => SourceInventoryOwner::Qsl,
            Self::QuireCodeRsProductionModulesAndFunctions => SourceInventoryOwner::QuireCodeRs,
        }
    }
}

/// A cancellation reason reported by an owner that actually accepts cancellation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceInventoryCancelCause {
    /// Explicit cancellation was requested.
    Requested,
    /// The owner's deadline expired.
    Deadline,
}

/// Typed cause explaining why owner enumeration did not exhaust its scope.
#[derive(Clone, Debug, Eq, PartialEq)]
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
    /// QSL's type work stopped at its first unaffordable charge.
    TypeWorkExhausted(SourceTypeExhaustion),
    /// QSL reported unfinished type admission for a native declaration handle.
    TypeAdmissionUnfinished {
        /// Native declaration handle whose type admission did not complete.
        declaration: SourceDeclarationHandle,
    },
    /// A selected production module was unavailable or could not be parsed.
    ModuleUnavailable {
        /// Selected module that was unavailable.
        module: SourceProductionModuleId,
        /// Owner-native module cause retained by the adapter.
        cause: Box<str>,
    },
    /// The admitted IR V2 reader stopped at its first exhausted limit.
    IrV2ReadLimit(CheckedPackageIncomplete),
    /// An owner with a native cancellation path stopped.
    Cancelled {
        /// Owner whose enumeration was cancelled.
        owner: SourceInventoryOwner,
        /// Native cancellation distinction retained without a dependency on the owner.
        cause: SourceInventoryCancelCause,
    },
}

impl SourceInventoryIncompleteReason {
    /// The owner to which this stop cause belongs.
    pub const fn owner(&self) -> SourceInventoryOwner {
        match self {
            Self::EarlyStop { owner }
            | Self::ParseFailed { owner, .. }
            | Self::RequestFailed { owner, .. }
            | Self::Cancelled { owner, .. } => *owner,
            Self::TypeWorkExhausted(_) | Self::TypeAdmissionUnfinished { .. } => {
                SourceInventoryOwner::Qsl
            }
            Self::ModuleUnavailable { .. } => SourceInventoryOwner::QuireCodeRs,
            Self::IrV2ReadLimit(_) => SourceInventoryOwner::IrV2,
        }
    }
}

/// QSL's native type-work capacity, without a dependency on QSL.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SourceTypeWorkDimension {
    /// Inspected or copied bytes.
    Bytes,
    /// Entered declarations.
    Declarations,
    /// Syntax visits and boundary probes.
    Expressions,
    /// Type constraints and lookups.
    Constraints,
    /// Dependency visits.
    Edges,
    /// Reserved normalization work.
    Normalization,
    /// Retained records.
    Records,
    /// Maximum inspected depth.
    Depth,
}

/// QSL declaration handle projected into the selected binding namespace.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceDeclarationHandle(pub usize);

/// QSL unit handle projected into the selected binding namespace.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceUnitHandle(pub usize);

/// QSL expression handle projected from the original unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceExpressionHandle(pub usize);

/// Exact native locus of a pending QSL obligation or exhausted charge.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceObligationSite {
    /// Handle in the selected binding namespace.
    pub declaration: SourceDeclarationHandle,
    /// Handle in that namespace's units.
    pub unit: SourceUnitHandle,
    /// Unit-local expression handle, when present.
    pub expression: Option<SourceExpressionHandle>,
    /// Inclusive start byte of the original source span.
    pub start: usize,
    /// Exclusive end byte of the original source span.
    pub end: usize,
}

/// QSL's first unaffordable type-work charge, retaining its native fields.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceTypeExhaustion {
    /// Exhausted capacity.
    pub dimension: SourceTypeWorkDimension,
    /// Effective capacity limit.
    pub limit: usize,
    /// Successfully charged usage or prior depth.
    pub prior: usize,
    /// Unpaid increment or requested depth.
    pub requested: usize,
    /// Original location where work stopped.
    pub site: SourceObligationSite,
}

/// QSL checked requirement map key: wire node plus native role and ordinal.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceRequirementOccurrenceId {
    /// Bytes of QSL's existing wire node id; the model computes no digest.
    pub node: [u8; 32],
    /// QSL origin role, without restricting its native vocabulary.
    pub role: Box<str>,
    /// QSL origin ordinal.
    pub ordinal: u64,
}

/// QSL's native pending-obligation kind and any target it carries.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SourcePendingObligationKind {
    /// Presence of an optional value.
    Presence,
    /// Arithmetic result range.
    ArithmeticRange,
    /// Nonzero divisor.
    Nonzero,
    /// Sum projection range.
    SumProjection,
    /// Every intermediate sum prefix.
    SumPrefixes,
    /// Totality of the exact callee declaration.
    PredicateTotal { target: SourceDeclarationHandle },
    /// Runtime graph closure.
    GraphClosure,
    /// Runtime invocation context.
    InvocationContext,
    /// Runtime capture input.
    CaptureInput,
}

/// Identity of a QSL pending obligation from its exact native site and kind.
/// Identical site and kind values denote one inventory identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourcePendingObligationId {
    /// The native declaration, unit, expression, and span.
    pub site: SourceObligationSite,
    /// The native kind, including a predicate target when present.
    pub kind: SourcePendingObligationKind,
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

/// Quire's authored criterion within an obligation declaration and scoped document.
/// This identity is independent of a formal Contract IR clause.
///
/// ```compile_fail,E0308
/// use quire_contract_model::source_obligation::SourceAuthoredCriterionId;
/// use quire_contract_model::{ClauseId, ClauseRef, PackageId, RequirementId, RequirementRef, RequirementRevision};
/// let clause = ClauseRef::new(
///     RequirementRef::new(PackageId::new("pkg").unwrap(), RequirementId::new("REQ_a").unwrap(), RequirementRevision::new(1).unwrap()),
///     ClauseId::new("AC_1").unwrap(),
/// );
/// let _: SourceAuthoredCriterionId = clause;
/// ```
///
/// ```compile_fail,E0277
/// use quire_contract_model::source_obligation::SourceAuthoredCriterionId;
/// use quire_contract_model::{ClauseId, ClauseRef, PackageId, RequirementId, RequirementRef, RequirementRevision};
/// let clause = ClauseRef::new(
///     RequirementRef::new(PackageId::new("pkg").unwrap(), RequirementId::new("REQ_a").unwrap(), RequirementRevision::new(1).unwrap()),
///     ClauseId::new("AC_1").unwrap(),
/// );
/// let _: SourceAuthoredCriterionId = SourceAuthoredCriterionId::from(clause);
/// ```
///
/// ```compile_fail,E0277
/// use quire_contract_model::source_obligation::SourceAuthoredCriterionId;
/// use quire_contract_model::{ClauseId, ClauseRef, PackageId, RequirementId, RequirementRef, RequirementRevision};
/// let clause = ClauseRef::new(
///     RequirementRef::new(PackageId::new("pkg").unwrap(), RequirementId::new("REQ_a").unwrap(), RequirementRevision::new(1).unwrap()),
///     ClauseId::new("AC_1").unwrap(),
/// );
/// let _: SourceAuthoredCriterionId = clause.into();
/// ```
///
/// ```compile_fail,E0308
/// use quire_contract_model::source_obligation::{SourceAuthoredCriterionId, SourceRequirementOccurrenceId};
/// let requirement = SourceRequirementOccurrenceId { node: [1; 32], role: "claim".into(), ordinal: 0 };
/// let _: SourceAuthoredCriterionId = requirement;
/// ```
///
/// ```compile_fail,E0277
/// use quire_contract_model::source_obligation::{SourceAuthoredCriterionId, SourceRequirementOccurrenceId};
/// let requirement = SourceRequirementOccurrenceId { node: [1; 32], role: "claim".into(), ordinal: 0 };
/// let _: SourceAuthoredCriterionId = SourceAuthoredCriterionId::from(requirement);
/// ```
///
/// ```compile_fail,E0277
/// use quire_contract_model::source_obligation::{SourceAuthoredCriterionId, SourceRequirementOccurrenceId};
/// let requirement = SourceRequirementOccurrenceId { node: [1; 32], role: "claim".into(), ordinal: 0 };
/// let _: SourceAuthoredCriterionId = requirement.into();
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceAuthoredCriterionId {
    obligation_declaration: Box<str>,
    scoped_document: Box<str>,
    criterion_id: Box<str>,
}

impl SourceAuthoredCriterionId {
    /// Retain the three Quire-authored identity components without interpreting them.
    pub fn new(
        obligation_declaration: impl Into<Box<str>>,
        scoped_document: impl Into<Box<str>>,
        criterion_id: impl Into<Box<str>>,
    ) -> Self {
        Self {
            obligation_declaration: obligation_declaration.into(),
            scoped_document: scoped_document.into(),
            criterion_id: criterion_id.into(),
        }
    }

    /// Borrow the declared obligation source.
    pub fn obligation_declaration(&self) -> &str {
        &self.obligation_declaration
    }
    /// Borrow the document path relative to the selected scope.
    pub fn scoped_document(&self) -> &str {
        &self.scoped_document
    }
    /// Borrow the authored criterion identifier.
    pub fn criterion_id(&self) -> &str {
        &self.criterion_id
    }
}

/// Typed identity of an item in an owner inventory.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SourceInventoryIdentity {
    /// A Quire-authored acceptance criterion.
    Criterion(SourceAuthoredCriterionId),
    /// A complete checked requirement record reported by QSL.
    RequirementRecord(SourceRequirementOccurrenceId),
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
    PendingObligation(SourcePendingObligationId),
    /// A selected production module.
    ProductionModule(SourceProductionModuleId),
    /// A selected production function.
    ProductionFunction(SourceProductionFunctionId),
}

impl SourceInventoryIdentity {
    fn owner(&self) -> SourceInventoryOwner {
        match self {
            Self::Criterion(_) => SourceInventoryOwner::Quire,
            Self::SemanticNode { .. } | Self::SemanticOccurrence(_) => SourceInventoryOwner::IrV2,
            Self::RequirementRecord(_) | Self::PendingObligation(_) => SourceInventoryOwner::Qsl,
            Self::ProductionModule(_) | Self::ProductionFunction(_) => {
                SourceInventoryOwner::QuireCodeRs
            }
        }
    }
}

/// One owner-reported item with its selected-source provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceInventoryRow {
    owner: SourceInventoryOwner,
    source: SourceIdentity,
    identity: SourceInventoryIdentity,
}

impl SourceInventoryRow {
    /// Retain a native owner identity and the selected source it reports.
    pub fn new(source: SourceIdentity, identity: SourceInventoryIdentity) -> Self {
        Self {
            owner: identity.owner(),
            source,
            identity,
        }
    }

    /// Return the owner implied by the typed identity.
    pub const fn owner(&self) -> SourceInventoryOwner {
        self.owner
    }

    /// Borrow the source reported for this row.
    pub fn source(&self) -> &SourceIdentity {
        &self.source
    }

    /// Borrow this row's typed owner identity.
    pub fn identity(&self) -> &SourceInventoryIdentity {
        &self.identity
    }
}

/// The terminal state issued by an owner enumerator after normal end or stop.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceInventoryCompleteness {
    /// The owner enumerator reached the normal end of its selected scope.
    Complete,
    /// The owner enumerator stopped with its original typed cause.
    Incomplete(SourceInventoryIncompleteReason),
}

/// A selected owner enumerator or typed adapter. Implementations must draw
/// rows from the selected native owner object and report its own observed count.
/// A model type cannot authenticate an external owner's traversal; that
/// obligation is checked against the owner's full output in the owner crate.
pub trait SourceInventoryEnumerator {
    /// Selected source whose native output is being traversed.
    fn source(&self) -> &SourceIdentity;
    /// Native owner of the selected output.
    fn owner(&self) -> SourceInventoryOwner;
    /// Whole named scope being traversed.
    fn scope(&self) -> SourceInventoryScope;
    /// The next native row, normal end, or original typed stop cause.
    fn next_row(&mut self) -> Result<Option<SourceInventoryRow>, SourceInventoryIncompleteReason>;
    /// Number of rows the owner observed, including rows emitted before a stop.
    fn observed_count(&self) -> usize;
}

/// A structurally inconsistent owner enumeration cannot become an outcome.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum SourceInventoryOutcomeError {
    /// The owner and named scope cannot describe the same inventory.
    #[error("inventory owner does not own its scope")]
    OwnerScopeMismatch,
    /// A retained row belongs to another owner.
    #[error("inventory row belongs to another owner")]
    RowOwnerMismatch,
    /// A retained row names a different selected source.
    #[error("inventory row names another source")]
    RowSourceMismatch,
    /// The stopped owner differs from the selected owner.
    #[error("inventory stop cause belongs to another owner")]
    StopOwnerMismatch,
    /// The owner's observed count differs from the rows retained by the model.
    #[error("inventory observed count differs from retained row count")]
    ObservedCountMismatch { observed: usize, retained: usize },
    /// The model could not retain the next owner-emitted row.
    #[error("inventory row allocation failed")]
    AllocationFailed,
}

/// One owner-issued terminal outcome. Its fields are private so a caller
/// cannot turn a caller-supplied row vector or absent stop cause into `Complete`.
/// A default-feature consumer can build an outcome through the enumerator and
/// inspect it through its public accessors:
///
/// ```
/// use quire_contract_model::source_obligation::{
///     SourceInventoryCompleteness, SourceInventoryEnumerator, SourceInventoryIncompleteReason,
///     SourceInventoryOutcome, SourceInventoryOwner, SourceInventoryRow, SourceInventoryScope,
/// };
/// use quire_contract_model::{SourceDocumentId, SourceIdentity, SourceRevision};
///
/// struct Empty(SourceIdentity);
/// impl SourceInventoryEnumerator for Empty {
///     fn source(&self) -> &SourceIdentity { &self.0 }
///     fn owner(&self) -> SourceInventoryOwner { SourceInventoryOwner::Qsl }
///     fn scope(&self) -> SourceInventoryScope {
///         SourceInventoryScope::QslCheckedRequirementsAndPendingObligations
///     }
///     fn next_row(&mut self) -> Result<Option<SourceInventoryRow>, SourceInventoryIncompleteReason> {
///         Ok(None)
///     }
///     fn observed_count(&self) -> usize { 0 }
/// }
/// let source = SourceIdentity::new(
///     SourceDocumentId::new("selected").expect("document"),
///     SourceRevision::new(1).expect("revision"),
/// );
/// let outcome = SourceInventoryOutcome::from_enumerator(Empty(source.clone())).expect("end");
/// assert_eq!(outcome.source(), &source);
/// assert!(outcome.items().is_empty());
/// assert_eq!(outcome.observed_count(), 0);
/// assert_eq!(outcome.completeness(), &SourceInventoryCompleteness::Complete);
/// ```
///
/// Direct reads of rows and count, and a caller-authored struct literal,
/// respectively fail at Rust's privacy boundary:
///
/// ```compile_fail,E0616
/// use quire_contract_model::source_obligation::SourceInventoryOutcome;
/// fn forbidden(outcome: &SourceInventoryOutcome) { let _ = &outcome.items; }
/// ```
///
/// ```compile_fail,E0616
/// use quire_contract_model::source_obligation::SourceInventoryOutcome;
/// fn forbidden(outcome: &SourceInventoryOutcome) { let _ = outcome.observed_count; }
/// ```
///
/// ```compile_fail,E0451
/// use quire_contract_model::source_obligation::{
///     SourceInventoryCompleteness, SourceInventoryOutcome, SourceInventoryOwner,
///     SourceInventoryScope,
/// };
/// use quire_contract_model::{SourceDocumentId, SourceIdentity, SourceRevision};
/// let source = SourceIdentity::new(
///     SourceDocumentId::new("selected").expect("document"),
///     SourceRevision::new(1).expect("revision"),
/// );
/// let _ = SourceInventoryOutcome {
///     owner: SourceInventoryOwner::Qsl,
///     source,
///     scope: SourceInventoryScope::QslCheckedRequirementsAndPendingObligations,
///     items: vec![],
///     observed_count: 0,
///     completeness: SourceInventoryCompleteness::Complete,
/// };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceInventoryOutcome {
    owner: SourceInventoryOwner,
    source: SourceIdentity,
    scope: SourceInventoryScope,
    items: Vec<SourceInventoryRow>,
    observed_count: usize,
    completeness: SourceInventoryCompleteness,
}

impl SourceInventoryOutcome {
    /// Drive the selected native enumerator to its terminal branch and retain
    /// exactly the rows it emitted. `Complete` follows only from `Ok(None)`.
    pub fn from_enumerator(
        mut enumerator: impl SourceInventoryEnumerator,
    ) -> Result<Self, SourceInventoryOutcomeError> {
        let owner = enumerator.owner();
        let scope = enumerator.scope();
        if scope.owner() != owner {
            return Err(SourceInventoryOutcomeError::OwnerScopeMismatch);
        }
        let source = enumerator.source().clone();
        let mut items = Vec::new();
        let completeness = loop {
            match enumerator.next_row() {
                Ok(Some(row)) => {
                    if row.owner() != owner {
                        return Err(SourceInventoryOutcomeError::RowOwnerMismatch);
                    }
                    if row.source() != &source {
                        return Err(SourceInventoryOutcomeError::RowSourceMismatch);
                    }
                    items
                        .try_reserve(1)
                        .map_err(|_| SourceInventoryOutcomeError::AllocationFailed)?;
                    items.push(row);
                }
                Ok(None) => break SourceInventoryCompleteness::Complete,
                Err(cause) => {
                    if cause.owner() != owner {
                        return Err(SourceInventoryOutcomeError::StopOwnerMismatch);
                    }
                    break SourceInventoryCompleteness::Incomplete(cause);
                }
            }
        };
        let observed_count = enumerator.observed_count();
        if observed_count != items.len() {
            return Err(SourceInventoryOutcomeError::ObservedCountMismatch {
                observed: observed_count,
                retained: items.len(),
            });
        }
        Ok(Self {
            owner,
            source,
            scope,
            items,
            observed_count,
            completeness,
        })
    }

    /// Return the owner of this terminal outcome.
    pub const fn owner(&self) -> SourceInventoryOwner {
        self.owner
    }

    /// Borrow the selected source.
    pub fn source(&self) -> &SourceIdentity {
        &self.source
    }

    /// Return the complete named scope.
    pub const fn scope(&self) -> SourceInventoryScope {
        self.scope
    }

    /// Borrow every retained owner-emitted row, including rows before a stop.
    pub fn items(&self) -> &[SourceInventoryRow] {
        &self.items
    }

    /// Return the owner's observed row count, validated against retained rows.
    pub const fn observed_count(&self) -> usize {
        self.observed_count
    }

    /// Borrow the owner-issued terminal state.
    pub fn completeness(&self) -> &SourceInventoryCompleteness {
        &self.completeness
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

/// The source-contract author's explicit link from a selected Quire scope to an IR source.
/// This records provenance; matching labels do not authenticate equal source bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceCriterionSourceBinding {
    quire_corpus: Box<str>,
    traceability_model: Box<str>,
    obligation_scope: Box<str>,
    source: SourceIdentity,
    author: SourceContractAuthorId,
}

impl SourceCriterionSourceBinding {
    /// Retain the selected corpus, validated model, declared scope, source, and author.
    pub fn new(
        quire_corpus: impl Into<Box<str>>,
        traceability_model: impl Into<Box<str>>,
        obligation_scope: impl Into<Box<str>>,
        source: SourceIdentity,
        author: SourceContractAuthorId,
    ) -> Self {
        Self {
            quire_corpus: quire_corpus.into(),
            traceability_model: traceability_model.into(),
            obligation_scope: obligation_scope.into(),
            source,
            author,
        }
    }

    /// Borrow the selected Quire corpus identity.
    pub fn quire_corpus(&self) -> &str {
        &self.quire_corpus
    }
    /// Borrow the selected validated traceability model identity.
    pub fn traceability_model(&self) -> &str {
        &self.traceability_model
    }
    /// Borrow the selected obligation declaration scope.
    pub fn obligation_scope(&self) -> &str {
        &self.obligation_scope
    }
    /// Borrow the source selected by the source-contract author.
    pub fn source(&self) -> &SourceIdentity {
        &self.source
    }
    /// Borrow the source-contract author identity.
    pub fn author(&self) -> &SourceContractAuthorId {
        &self.author
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
    criterion: SourceAuthoredCriterionId,
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
        criterion: SourceAuthoredCriterionId,
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
    pub fn criterion(&self) -> &SourceAuthoredCriterionId {
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

/// Four optional owner outcomes and authored assertions for one source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceObligationInput {
    source: SourceIdentity,
    criterion_source_binding: Option<SourceCriterionSourceBinding>,
    quire: Option<SourceInventoryOutcome>,
    ir_v2: Option<SourceInventoryOutcome>,
    qsl: Option<SourceInventoryOutcome>,
    quire_code_rs: Option<SourceInventoryOutcome>,
    assertions: Vec<SourceCorrespondenceAssertion>,
}

impl SourceObligationInput {
    /// Retain the selected source, optional owner outcomes, and authored assertions.
    pub fn new(
        source: SourceIdentity,
        criterion_source_binding: Option<SourceCriterionSourceBinding>,
        quire: Option<SourceInventoryOutcome>,
        ir_v2: Option<SourceInventoryOutcome>,
        qsl: Option<SourceInventoryOutcome>,
        quire_code_rs: Option<SourceInventoryOutcome>,
        assertions: Vec<SourceCorrespondenceAssertion>,
    ) -> Self {
        Self {
            source,
            criterion_source_binding,
            quire,
            ir_v2,
            qsl,
            quire_code_rs,
            assertions,
        }
    }

    /// Borrow the selected source.
    pub fn source(&self) -> &SourceIdentity {
        &self.source
    }

    /// Borrow the explicit Quire-selection-to-source binding, if supplied.
    pub fn criterion_source_binding(&self) -> Option<&SourceCriterionSourceBinding> {
        self.criterion_source_binding.as_ref()
    }

    /// Borrow the Quire outcome, if supplied.
    pub fn quire(&self) -> Option<&SourceInventoryOutcome> {
        self.quire.as_ref()
    }

    /// Borrow the IR V2 outcome, if supplied.
    pub fn ir_v2(&self) -> Option<&SourceInventoryOutcome> {
        self.ir_v2.as_ref()
    }

    /// Borrow the QSL outcome, if supplied.
    pub fn qsl(&self) -> Option<&SourceInventoryOutcome> {
        self.qsl.as_ref()
    }

    /// Borrow the `quire-code-rs` outcome, if supplied.
    pub fn quire_code_rs(&self) -> Option<&SourceInventoryOutcome> {
        self.quire_code_rs.as_ref()
    }

    /// Borrow the authored assertions without accepting or rejecting them.
    pub fn assertions(&self) -> &[SourceCorrespondenceAssertion] {
        &self.assertions
    }
}

/// Why an inventoried entity has no accepted production mapping.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SourceMissingMappingReason {
    /// No accepted authored mapping names the entity.
    NoAuthoredMapping,
    /// An accepted assertion explicitly has no production function.
    ExplicitNoProductionFunction,
}

/// Disposition of one distinct owner inventory row.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SourceInventoryDisposition {
    /// An accepted relation maps this entity to a production function.
    Mapped,
    /// The entity remains inventoried without a production mapping.
    ZeroMappings(SourceMissingMappingReason),
    /// One node key has differing semantic facts.
    ConflictingIdentity,
}

/// One owner row retained in the census with its disposition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceCensusInventoryRow {
    row: SourceInventoryRow,
    disposition: SourceInventoryDisposition,
}

impl SourceCensusInventoryRow {
    /// Borrow the owner row.
    pub fn row(&self) -> &SourceInventoryRow {
        &self.row
    }
    /// Return its disposition.
    pub const fn disposition(&self) -> SourceInventoryDisposition {
        self.disposition
    }
}

/// Reason an authored assertion was rejected.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SourceCorrespondenceRejectionReason {
    /// Assertion or provenance names another source.
    ForeignSource,
    /// Criterion is absent from Quire's inventory.
    DanglingCriterion,
    /// Occurrence or its unconflicted node is absent.
    DanglingOccurrence,
    /// Function or its module is absent.
    DanglingProductionSymbol,
    /// One assertion ID names differing relation facts.
    ConflictingRelation,
    /// One assertion ID or relation names differing provenance.
    ConflictingProvenance,
}

/// Rejected authored assertion with its original fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceCorrespondenceRejection {
    assertion: SourceCorrespondenceAssertion,
    reason: SourceCorrespondenceRejectionReason,
}

impl SourceCorrespondenceRejection {
    /// Borrow the original assertion.
    pub fn assertion(&self) -> &SourceCorrespondenceAssertion {
        &self.assertion
    }
    /// Return its rejection reason.
    pub const fn reason(&self) -> SourceCorrespondenceRejectionReason {
        self.reason
    }
}

/// One accepted authored relation.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SourceObligationRow {
    criterion: SourceAuthoredCriterionId,
    occurrence: SourceSemanticOccurrenceId,
    production_function: Option<SourceProductionFunctionId>,
    provenance: SourceAssertionProvenance,
}

impl SourceObligationRow {
    /// Borrow the authored criterion.
    pub fn criterion(&self) -> &SourceAuthoredCriterionId {
        &self.criterion
    }
    /// Borrow the semantic occurrence.
    pub fn occurrence(&self) -> &SourceSemanticOccurrenceId {
        &self.occurrence
    }
    /// Borrow the production function when present.
    pub fn production_function(&self) -> Option<&SourceProductionFunctionId> {
        self.production_function.as_ref()
    }
    /// Borrow the authored provenance.
    pub fn provenance(&self) -> &SourceAssertionProvenance {
        &self.provenance
    }
}

/// Immutable census over four complete owner outcomes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceObligationCensus {
    source: SourceIdentity,
    criterion_source_binding: SourceCriterionSourceBinding,
    inventory_rows: Vec<SourceCensusInventoryRow>,
    correspondence_rows: Vec<SourceObligationRow>,
    rejected_assertions: Vec<SourceCorrespondenceRejection>,
}

impl SourceObligationCensus {
    /// Borrow the selected source.
    pub fn source(&self) -> &SourceIdentity {
        &self.source
    }
    /// Borrow the source author's Quire selection binding.
    pub fn criterion_source_binding(&self) -> &SourceCriterionSourceBinding {
        &self.criterion_source_binding
    }
    /// Borrow every distinct owner row.
    pub fn inventory_rows(&self) -> &[SourceCensusInventoryRow] {
        &self.inventory_rows
    }
    /// Borrow accepted relations.
    pub fn correspondence_rows(&self) -> &[SourceObligationRow] {
        &self.correspondence_rows
    }
    /// Borrow rejected assertions.
    pub fn rejected_assertions(&self) -> &[SourceCorrespondenceRejection] {
        &self.rejected_assertions
    }
}

/// A required owner outcome or Quire binding cannot establish the census.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum SourceCorrespondenceError {
    /// No Quire selection binding was supplied.
    #[error("missing criterion source binding")]
    MissingCriterionSourceBinding,
    /// Binding names another source.
    #[error("foreign criterion source binding")]
    ForeignCriterionSourceBinding {
        expected: SourceIdentity,
        observed: SourceIdentity,
    },
    /// Binding scope disagrees with an authored criterion.
    #[error("conflicting criterion source binding")]
    ConflictingCriterionSourceBinding {
        binding_scope: Box<str>,
        criterion: SourceAuthoredCriterionId,
    },
    /// No outcome was supplied at this owner position.
    #[error("missing {owner:?} inventory")]
    MissingOwnerInventory { owner: SourceInventoryOwner },
    /// Outcome ended before normal exhaustion.
    #[error("incomplete {owner:?} inventory")]
    IncompleteOwnerInventory {
        owner: SourceInventoryOwner,
        cause: SourceInventoryIncompleteReason,
    },
    /// Outcome names another selected source.
    #[error("{owner:?} inventory source mismatch")]
    InventorySourceMismatch {
        owner: SourceInventoryOwner,
        expected: SourceIdentity,
        observed: SourceIdentity,
    },
    /// Outcome belongs to a different owner position.
    #[error("{owner:?} inventory scope mismatch")]
    InventoryScopeMismatch {
        owner: SourceInventoryOwner,
        expected: SourceInventoryScope,
        observed: SourceInventoryScope,
    },
}

fn census_owner_rows(
    owner: SourceInventoryOwner,
    source: &SourceIdentity,
    outcome: Option<SourceInventoryOutcome>,
) -> Result<Vec<SourceInventoryRow>, SourceCorrespondenceError> {
    let outcome = outcome.ok_or(SourceCorrespondenceError::MissingOwnerInventory { owner })?;
    let expected = match owner {
        SourceInventoryOwner::Quire => SourceInventoryScope::QuireSelectedAuthoredCriteria,
        SourceInventoryOwner::IrV2 => SourceInventoryScope::IrV2SemanticGraphAndOccurrences,
        SourceInventoryOwner::Qsl => {
            SourceInventoryScope::QslCheckedRequirementsAndPendingObligations
        }
        SourceInventoryOwner::QuireCodeRs => {
            SourceInventoryScope::QuireCodeRsProductionModulesAndFunctions
        }
    };
    if outcome.owner != owner || outcome.scope != expected {
        return Err(SourceCorrespondenceError::InventoryScopeMismatch {
            owner,
            expected,
            observed: outcome.scope,
        });
    }
    if &outcome.source != source {
        return Err(SourceCorrespondenceError::InventorySourceMismatch {
            owner,
            expected: source.clone(),
            observed: outcome.source,
        });
    }
    if let SourceInventoryCompleteness::Incomplete(cause) = outcome.completeness {
        return Err(SourceCorrespondenceError::IncompleteOwnerInventory { owner, cause });
    }
    Ok(outcome.items)
}

fn assertion_key(
    a: &SourceCorrespondenceAssertion,
) -> (
    &SourceAssertionId,
    &SourceIdentity,
    &SourceAuthoredCriterionId,
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

/// Build the ordered source census from four complete outcomes and authored assertions.
pub fn build_source_obligation_census(
    input: SourceObligationInput,
) -> Result<SourceObligationCensus, SourceCorrespondenceError> {
    let binding = input
        .criterion_source_binding
        .ok_or(SourceCorrespondenceError::MissingCriterionSourceBinding)?;
    if binding.source != input.source {
        return Err(SourceCorrespondenceError::ForeignCriterionSourceBinding {
            expected: input.source,
            observed: binding.source,
        });
    }
    let mut rows = census_owner_rows(SourceInventoryOwner::Quire, &input.source, input.quire)?;
    let first_conflict = rows
        .iter()
        .filter_map(|r| match &r.identity {
            SourceInventoryIdentity::Criterion(c)
                if c.obligation_declaration() != binding.obligation_scope() =>
            {
                Some(c)
            }
            SourceInventoryIdentity::Criterion(_) => None,
            SourceInventoryIdentity::RequirementRecord(_)
            | SourceInventoryIdentity::SemanticNode { .. }
            | SourceInventoryIdentity::SemanticOccurrence(_)
            | SourceInventoryIdentity::PendingObligation(_)
            | SourceInventoryIdentity::ProductionModule(_)
            | SourceInventoryIdentity::ProductionFunction(_) => None,
        })
        .min();
    if let Some(criterion) = first_conflict {
        return Err(
            SourceCorrespondenceError::ConflictingCriterionSourceBinding {
                binding_scope: binding.obligation_scope.clone(),
                criterion: criterion.clone(),
            },
        );
    }
    rows.extend(census_owner_rows(
        SourceInventoryOwner::IrV2,
        &input.source,
        input.ir_v2,
    )?);
    rows.extend(census_owner_rows(
        SourceInventoryOwner::Qsl,
        &input.source,
        input.qsl,
    )?);
    rows.extend(census_owner_rows(
        SourceInventoryOwner::QuireCodeRs,
        &input.source,
        input.quire_code_rs,
    )?);
    rows.sort_by(|a, b| (&a.owner, &a.identity).cmp(&(&b.owner, &b.identity)));
    rows.dedup();

    let mut node_facts: BTreeMap<&CheckedNodeId, BTreeSet<&CheckedSemanticId>> = BTreeMap::new();
    for row in &rows {
        if let SourceInventoryIdentity::SemanticNode { identity, semantic } = &row.identity {
            node_facts.entry(identity).or_default().insert(semantic);
        }
    }
    let conflicting_nodes: BTreeSet<_> = node_facts
        .iter()
        .filter_map(|(n, facts)| (facts.len() > 1).then_some((*n).clone()))
        .collect();
    let nodes: BTreeSet<_> = node_facts
        .keys()
        .filter(|n| !conflicting_nodes.contains(*n))
        .map(|n| (*n).clone())
        .collect();
    let criteria: BTreeSet<_> = rows
        .iter()
        .filter_map(|r| match &r.identity {
            SourceInventoryIdentity::Criterion(c) => Some(c.clone()),
            SourceInventoryIdentity::RequirementRecord(_)
            | SourceInventoryIdentity::SemanticNode { .. }
            | SourceInventoryIdentity::SemanticOccurrence(_)
            | SourceInventoryIdentity::PendingObligation(_)
            | SourceInventoryIdentity::ProductionModule(_)
            | SourceInventoryIdentity::ProductionFunction(_) => None,
        })
        .collect();
    let occurrences: BTreeSet<_> = rows
        .iter()
        .filter_map(|r| match &r.identity {
            SourceInventoryIdentity::SemanticOccurrence(o) if nodes.contains(o.node()) => {
                Some(o.clone())
            }
            SourceInventoryIdentity::SemanticOccurrence(_)
            | SourceInventoryIdentity::Criterion(_)
            | SourceInventoryIdentity::RequirementRecord(_)
            | SourceInventoryIdentity::SemanticNode { .. }
            | SourceInventoryIdentity::PendingObligation(_)
            | SourceInventoryIdentity::ProductionModule(_)
            | SourceInventoryIdentity::ProductionFunction(_) => None,
        })
        .collect();
    let modules: BTreeSet<_> = rows
        .iter()
        .filter_map(|r| match &r.identity {
            SourceInventoryIdentity::ProductionModule(m) => Some(m.clone()),
            SourceInventoryIdentity::Criterion(_)
            | SourceInventoryIdentity::RequirementRecord(_)
            | SourceInventoryIdentity::SemanticNode { .. }
            | SourceInventoryIdentity::SemanticOccurrence(_)
            | SourceInventoryIdentity::PendingObligation(_)
            | SourceInventoryIdentity::ProductionFunction(_) => None,
        })
        .collect();
    let functions: BTreeSet<_> = rows
        .iter()
        .filter_map(|r| match &r.identity {
            SourceInventoryIdentity::ProductionFunction(f) if modules.contains(f.module()) => {
                Some(f.clone())
            }
            SourceInventoryIdentity::ProductionFunction(_)
            | SourceInventoryIdentity::Criterion(_)
            | SourceInventoryIdentity::RequirementRecord(_)
            | SourceInventoryIdentity::SemanticNode { .. }
            | SourceInventoryIdentity::SemanticOccurrence(_)
            | SourceInventoryIdentity::PendingObligation(_)
            | SourceInventoryIdentity::ProductionModule(_) => None,
        })
        .collect();

    let mut by_id: BTreeMap<SourceAssertionId, Vec<SourceCorrespondenceAssertion>> =
        BTreeMap::new();
    for a in input.assertions {
        by_id.entry(a.id.clone()).or_default().push(a);
    }
    let mut candidates = Vec::new();
    let mut rejected = Vec::new();
    for (_, mut group) in by_id {
        group.sort_by(|a, b| assertion_key(a).cmp(&assertion_key(b)));
        group.dedup();
        let relations: BTreeSet<_> = group
            .iter()
            .map(|a| {
                (
                    &a.source,
                    &a.criterion,
                    &a.occurrence,
                    &a.production_function,
                )
            })
            .collect();
        let provenances: BTreeSet<_> = group.iter().map(|a| &a.provenance).collect();
        let group_reason = if relations.len() > 1 {
            Some(SourceCorrespondenceRejectionReason::ConflictingRelation)
        } else if provenances.len() > 1 {
            Some(SourceCorrespondenceRejectionReason::ConflictingProvenance)
        } else {
            None
        };
        for assertion in group {
            let reason = group_reason.or_else(|| {
                if assertion.source != input.source || assertion.provenance.source != input.source {
                    Some(SourceCorrespondenceRejectionReason::ForeignSource)
                } else if !criteria.contains(&assertion.criterion) {
                    Some(SourceCorrespondenceRejectionReason::DanglingCriterion)
                } else if !occurrences.contains(&assertion.occurrence) {
                    Some(SourceCorrespondenceRejectionReason::DanglingOccurrence)
                } else if assertion
                    .production_function
                    .as_ref()
                    .is_some_and(|f| !functions.contains(f))
                {
                    Some(SourceCorrespondenceRejectionReason::DanglingProductionSymbol)
                } else {
                    None
                }
            });
            if let Some(reason) = reason {
                rejected.push(SourceCorrespondenceRejection { assertion, reason });
            } else {
                candidates.push(assertion);
            }
        }
    }
    let mut by_relation: BTreeMap<
        (
            SourceAuthoredCriterionId,
            SourceSemanticOccurrenceId,
            Option<SourceProductionFunctionId>,
        ),
        Vec<SourceCorrespondenceAssertion>,
    > = BTreeMap::new();
    for a in candidates {
        by_relation
            .entry((
                a.criterion.clone(),
                a.occurrence.clone(),
                a.production_function.clone(),
            ))
            .or_default()
            .push(a);
    }
    let mut accepted = Vec::new();
    for (_, group) in by_relation {
        if group
            .iter()
            .map(|a| &a.provenance)
            .collect::<BTreeSet<_>>()
            .len()
            > 1
        {
            rejected.extend(
                group
                    .into_iter()
                    .map(|assertion| SourceCorrespondenceRejection {
                        assertion,
                        reason: SourceCorrespondenceRejectionReason::ConflictingProvenance,
                    }),
            );
        } else if let Some(a) = group.into_iter().next() {
            accepted.push(SourceObligationRow {
                criterion: a.criterion,
                occurrence: a.occurrence,
                production_function: a.production_function,
                provenance: a.provenance,
            });
        }
    }
    accepted.sort();
    rejected.sort_by(|a, b| {
        (assertion_key(&a.assertion), a.reason).cmp(&(assertion_key(&b.assertion), b.reason))
    });

    let mapped_criteria: BTreeSet<_> = accepted.iter().map(|r| &r.criterion).collect();
    let mapped_occurrences: BTreeSet<_> = accepted
        .iter()
        .filter(|r| r.production_function.is_some())
        .map(|r| &r.occurrence)
        .collect();
    let absent_occurrences: BTreeSet<_> = accepted
        .iter()
        .filter(|r| r.production_function.is_none())
        .map(|r| &r.occurrence)
        .collect();
    let mapped_functions: BTreeSet<_> = accepted
        .iter()
        .filter_map(|r| r.production_function.as_ref())
        .collect();
    let mapped_modules: BTreeSet<_> = mapped_functions.iter().map(|f| f.module()).collect();
    let mapped_nodes: BTreeSet<_> = mapped_occurrences.iter().map(|o| o.node()).collect();
    let absent_nodes: BTreeSet<_> = absent_occurrences.iter().map(|o| o.node()).collect();
    let inventory_rows = rows
        .into_iter()
        .map(|row| {
            let disposition = match &row.identity {
                SourceInventoryIdentity::Criterion(c) => {
                    if mapped_criteria.contains(c) {
                        SourceInventoryDisposition::Mapped
                    } else {
                        SourceInventoryDisposition::ZeroMappings(
                            SourceMissingMappingReason::NoAuthoredMapping,
                        )
                    }
                }
                SourceInventoryIdentity::SemanticNode { identity, .. } => {
                    if conflicting_nodes.contains(identity) {
                        SourceInventoryDisposition::ConflictingIdentity
                    } else if mapped_nodes.contains(identity) {
                        SourceInventoryDisposition::Mapped
                    } else if absent_nodes.contains(identity) {
                        SourceInventoryDisposition::ZeroMappings(
                            SourceMissingMappingReason::ExplicitNoProductionFunction,
                        )
                    } else {
                        SourceInventoryDisposition::ZeroMappings(
                            SourceMissingMappingReason::NoAuthoredMapping,
                        )
                    }
                }
                SourceInventoryIdentity::SemanticOccurrence(o) => {
                    if mapped_occurrences.contains(o) {
                        SourceInventoryDisposition::Mapped
                    } else if absent_occurrences.contains(o) {
                        SourceInventoryDisposition::ZeroMappings(
                            SourceMissingMappingReason::ExplicitNoProductionFunction,
                        )
                    } else {
                        SourceInventoryDisposition::ZeroMappings(
                            SourceMissingMappingReason::NoAuthoredMapping,
                        )
                    }
                }
                SourceInventoryIdentity::ProductionModule(m) => {
                    if mapped_modules.contains(m) {
                        SourceInventoryDisposition::Mapped
                    } else {
                        SourceInventoryDisposition::ZeroMappings(
                            SourceMissingMappingReason::NoAuthoredMapping,
                        )
                    }
                }
                SourceInventoryIdentity::ProductionFunction(f) => {
                    if mapped_functions.contains(f) {
                        SourceInventoryDisposition::Mapped
                    } else {
                        SourceInventoryDisposition::ZeroMappings(
                            SourceMissingMappingReason::NoAuthoredMapping,
                        )
                    }
                }
                SourceInventoryIdentity::RequirementRecord(_)
                | SourceInventoryIdentity::PendingObligation(_) => {
                    SourceInventoryDisposition::ZeroMappings(
                        SourceMissingMappingReason::NoAuthoredMapping,
                    )
                }
            };
            SourceCensusInventoryRow { row, disposition }
        })
        .collect();
    Ok(SourceObligationCensus {
        source: input.source,
        criterion_source_binding: binding,
        inventory_rows,
        correspondence_rows: accepted,
        rejected_assertions: rejected,
    })
}
