//! FR-322 `dependency_reference`: binding each `dependency_selections` entry
//! to the dependency package the caller supplies, and checking each
//! `{term: "dependency_reference", package, node}` term against it.
//!
//! Two stages, in the reader's first-refusal order:
//!
//! 1. [`admit_dependencies`], in the lock stage: every entry, in lock order,
//!    must have a package supplied under its identity (`missing_import` /
//!    `missing-selection` at the entry), whose version equals the entry's
//!    (`stale_dependency` / `revision-mismatch` at the entry's `version`) and
//!    whose own `package_id` equals the entry's (`stale_dependency` /
//!    `byte-digest-mismatch` at the entry's `package_id.digest`). This runs
//!    before any node is read, so before any `dependency_reference` check.
//! 2. [`DependencyReferences::walk_arguments`] and
//!    [`DependencyReferences::walk_body`], in step 7 of the operation stage: a
//!    term is checked at its place in its node's pre-order walk, wherever it
//!    stands, by `missing-selection` (`missing_declaration`), `missing-name`
//!    (`missing_declaration`) and `operator-ineligible` (`ill_typed`), in
//!    that order.
//!
//! The term stands only as argument 0 of a `quire.op.function.call`
//! application and names a `function` declaration of the dependency whose
//! signature types are package-independent: no node in the transitive
//! closure of the signature type nodes carries a `declaration` or a
//! `ModelOwner`. FR-322 does not say which nodes of a dependency's `function`
//! node are its signature, so this reader takes the function node's own
//! `semantic_type` (the result type) and, from its `dependencies`, each
//! `parameter` node's `semantic_type` and each type node listed directly. The
//! closure follows `dependencies`, `semantic_type` and the `reference` terms
//! of each node's body, and a `model` or `relation` node is a `ModelOwner`
//! carrier.

use super::{
    BodyTerm, CheckedDependencySelection, CheckedNodeKind, CheckedNodeTag, CheckedPackageV2,
    CheckedSemanticNodeV2, NominalIdentityPreimage, NominalOwner, ValueForm, WorkMeter,
};
use crate::checked_package::common::{
    body_term, count, dependency_reference_node, dependency_reference_package, node_pointer,
    ValidationFailure,
};
use crate::checked_package::evidence::CheckedPackageEvidence;
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedSemanticId,
    JsonPointer,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// The operation whose callee (argument 0) a `dependency_reference` may be.
const FUNCTION_CALL_OPERATION: &str = "quire.op.function.call";

/// The dependency packages the caller supplied, one per
/// `dependency_selections` entry and in the same order.
#[derive(Debug, Default)]
pub(super) struct SuppliedDependencies<'e> {
    packages: Vec<(&'e CheckedSemanticId, &'e CheckedPackageV2)>,
}

impl<'e> SuppliedDependencies<'e> {
    /// The package supplied for the entry whose `package_id` is `package`.
    fn resolve(&self, package: &CheckedSemanticId) -> Option<&'e CheckedPackageV2> {
        self.packages
            .iter()
            .find(|(id, _)| *id == package)
            .map(|(_, supplied)| *supplied)
    }
}

/// Binds every selection entry to the package `evidence` supplies for its
/// identity. The first entry, in lock order, that is unsupplied or does not
/// bind refuses.
pub(super) fn admit_dependencies<'e>(
    selections: &'e [CheckedDependencySelection],
    evidence: &'e CheckedPackageEvidence,
) -> Result<SuppliedDependencies<'e>, ValidationFailure> {
    let at = |index: usize| {
        JsonPointer::root()
            .key("lock")
            .key("dependency_selections")
            .index(index)
    };
    let mut packages = Vec::with_capacity(selections.len());
    for (index, entry) in selections.iter().enumerate() {
        let Some((version, package)) = evidence.dependency_package(&entry.identity) else {
            return Err(ValidationFailure::refused_because(
                CheckedPackageRefusalCode::MissingImport,
                at(index),
                CheckedPackageRefusalCause::MissingSelection,
            ));
        };
        if version != entry.version.as_ref() {
            return Err(ValidationFailure::refused_because(
                CheckedPackageRefusalCode::StaleDependency,
                at(index).key("version"),
                CheckedPackageRefusalCause::RevisionMismatch,
            ));
        }
        // The package was admitted by the reader, which recomputed its
        // `package_id` from its identity preimage.
        if package.package_id() != &entry.package_id {
            return Err(ValidationFailure::refused_because(
                CheckedPackageRefusalCode::StaleDependency,
                at(index).key("package_id").key("digest"),
                CheckedPackageRefusalCause::ByteDigestMismatch,
            ));
        }
        packages.push((&entry.package_id, package));
    }
    Ok(SuppliedDependencies { packages })
}

/// The node whose terms are being checked.
#[derive(Clone, Copy)]
pub(super) struct Referrer<'a> {
    pub(super) node: &'a CheckedSemanticNodeV2,
    pub(super) position: usize,
}

impl Referrer<'_> {
    fn body(self) -> JsonPointer {
        node_pointer(self.position).key("body")
    }

    fn refuse(
        self,
        code: CheckedPackageRefusalCode,
        cause: CheckedPackageRefusalCause,
        path: JsonPointer,
    ) -> ValidationFailure {
        ValidationFailure::refused_at(code, path, Some(cause), self.node.node_id.clone())
    }
}

/// Everything a `dependency_reference` check reads besides the referencing
/// node.
#[derive(Clone, Copy)]
pub(super) struct DependencyReferences<'a> {
    supplied: &'a SuppliedDependencies<'a>,
}

impl<'a> DependencyReferences<'a> {
    pub(super) fn new(supplied: &'a SuppliedDependencies<'a>) -> Self {
        Self { supplied }
    }

    /// Checks every `dependency_reference` in `referrer`'s whole body, in
    /// pre-order, the body root itself being no callee.
    pub(super) fn walk_body(
        self,
        referrer: Referrer<'_>,
        meter: &mut WorkMeter,
    ) -> Result<(), ValidationFailure> {
        self.walk(referrer, &referrer.node.body, referrer.body(), false, meter)
    }

    /// Checks every `dependency_reference` under the arguments of the
    /// application that is `referrer`'s body root, in pre-order. The
    /// application's own checks are the caller's.
    pub(super) fn walk_arguments(
        self,
        referrer: Referrer<'_>,
        meter: &mut WorkMeter,
    ) -> Result<(), ValidationFailure> {
        self.walk_application(referrer, &referrer.node.body, referrer.body(), meter)
    }

    fn walk(
        self,
        referrer: Referrer<'_>,
        term: &Value,
        at: JsonPointer,
        is_callee: bool,
        meter: &mut WorkMeter,
    ) -> Result<(), ValidationFailure> {
        match body_term(term) {
            Some(BodyTerm::DependencyReference) => {
                self.check(referrer, term, &at, is_callee, meter)
            }
            Some(BodyTerm::Application) => self.walk_application(referrer, term, at, meter),
            Some(BodyTerm::Aggregate) => {
                let members = term.get("members").and_then(Value::as_array);
                for (index, member) in members.into_iter().flatten().enumerate() {
                    let member_at = at.clone().key("members").index(index);
                    self.walk(referrer, member, member_at, false, meter)?;
                }
                Ok(())
            }
            Some(BodyTerm::Binding) => match term.get("value") {
                Some(value) => self.walk(referrer, value, at.key("value"), false, meter),
                None => Ok(()),
            },
            Some(BodyTerm::Literal | BodyTerm::Reference | BodyTerm::Frame) | None => Ok(()),
        }
    }

    fn walk_application(
        self,
        referrer: Referrer<'_>,
        application: &Value,
        at: JsonPointer,
        meter: &mut WorkMeter,
    ) -> Result<(), ValidationFailure> {
        let calls_function = is_function_call(application);
        let arguments = application.get("arguments").and_then(Value::as_array);
        for (index, argument) in arguments.into_iter().flatten().enumerate() {
            let argument_at = at.clone().key("arguments").index(index);
            let is_callee = index == 0 && calls_function;
            self.walk(referrer, argument, argument_at, is_callee, meter)?;
        }
        Ok(())
    }

    /// One `dependency_reference` term at `at`: `missing-selection`, then
    /// `missing-name`, then `operator-ineligible`.
    fn check(
        self,
        referrer: Referrer<'_>,
        term: &Value,
        at: &JsonPointer,
        is_callee: bool,
        meter: &mut WorkMeter,
    ) -> Result<(), ValidationFailure> {
        let malformed = || {
            ValidationFailure::refused(CheckedPackageRefusalCode::InvalidSemanticGraph, at.clone())
        };
        // The closed shape was admitted by the term walk; a term that is not
        // is the value at fault here too.
        let (Some(package), Some(node)) = (
            dependency_reference_package(term),
            dependency_reference_node(term),
        ) else {
            return Err(malformed());
        };
        let Some(dependency) = self.supplied.resolve(&package) else {
            return Err(referrer.refuse(
                CheckedPackageRefusalCode::MissingDeclaration,
                CheckedPackageRefusalCause::MissingSelection,
                at.clone().key("package"),
            ));
        };
        let graph = dependency.graph();
        // The lookup and the signature closure read the dependency's graph:
        // one unit per node, charged at the term.
        meter.charge(count(graph.nodes.len()).max(1), || at.clone())?;
        let index: BTreeMap<&CheckedNodeId, usize> = graph
            .nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect();
        let target = index
            .get(&node)
            .and_then(|position| Some((*position, graph.nodes.get(*position)?)))
            .filter(|(_, target)| target.declaration.is_some());
        let Some((position, _)) = target else {
            return Err(referrer.refuse(
                CheckedPackageRefusalCode::MissingDeclaration,
                CheckedPackageRefusalCause::MissingName,
                at.clone().key("node"),
            ));
        };
        let is_function = dependency
            .node_kinds()
            .get(position)
            .is_some_and(|kind| kind.tag() == CheckedNodeTag::Function);
        if is_callee
            && is_function
            && signature_is_package_independent(dependency, position, &index)
        {
            return Ok(());
        }
        Err(referrer.refuse(
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::OperatorIneligible,
            at.clone(),
        ))
    }
}

/// Whether an application term is a `quire.op.function.call`.
// string-edge: decodes the wire operation identity to tell a function call from any other operation.
fn is_function_call(application: &Value) -> bool {
    application
        .get("operation")
        .and_then(|operation| operation.get("identity"))
        .and_then(Value::as_str)
        .is_some_and(|identity| identity == FUNCTION_CALL_OPERATION)
}

/// FR-322: no node in the transitive closure of the function's signature
/// type nodes carries a `declaration` or a `ModelOwner`.
fn signature_is_package_independent(
    dependency: &CheckedPackageV2,
    function: usize,
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> bool {
    let nodes = &dependency.graph().nodes;
    let kinds = dependency.node_kinds();
    let node_of = |id: &CheckedNodeId| {
        let position = *index.get(id)?;
        Some((nodes.get(position)?, *kinds.get(position)?))
    };
    let Some(function) = nodes.get(function) else {
        return false;
    };
    let mut pending: Vec<CheckedNodeId> = vec![function.semantic_type.clone()];
    for id in &function.dependencies {
        let Some((node, kind)) = node_of(id) else {
            continue;
        };
        match kind {
            CheckedNodeKind::Value(ValueForm::Parameter) => {
                pending.push(node.semantic_type.clone());
            }
            CheckedNodeKind::ScalarType(_)
            | CheckedNodeKind::CompositeType(_)
            | CheckedNodeKind::BoundedDomain(_) => pending.push(id.clone()),
            CheckedNodeKind::Value(
                ValueForm::Literal
                | ValueForm::EnumValue
                | ValueForm::CollectionValue
                | ValueForm::RecordValue
                | ValueForm::TupleValue
                | ValueForm::OptionValue,
            )
            | CheckedNodeKind::Expression(_)
            | CheckedNodeKind::Function(_)
            | CheckedNodeKind::Model(_)
            | CheckedNodeKind::Relation(_)
            | CheckedNodeKind::State(_)
            | CheckedNodeKind::Temporal(_)
            | CheckedNodeKind::Protocol(_)
            | CheckedNodeKind::Claim(_)
            | CheckedNodeKind::Correspondence(_) => {}
        }
    }
    let mut seen = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        let Some((node, kind)) = node_of(&id) else {
            continue;
        };
        if carries_declaration_or_owner(node, kind) {
            return false;
        }
        pending.extend(node.dependencies.iter().cloned());
        pending.push(node.semantic_type.clone());
        body_reference_targets(&node.body, &mut pending);
    }
    true
}

/// Whether a node is package-dependent: it carries a `declaration`, or it is
/// keyed by a `ModelOwner` (a `model` or `relation` node, or a nominal
/// preimage owned by a domain package).
fn carries_declaration_or_owner(node: &CheckedSemanticNodeV2, kind: CheckedNodeKind) -> bool {
    node.declaration.is_some()
        || matches!(kind.tag(), CheckedNodeTag::Model | CheckedNodeTag::Relation)
        || matches!(
            node.nominal_identity_preimage
                .as_ref()
                .and_then(NominalIdentityPreimage::owner),
            Some(NominalOwner::Model { .. })
        )
}

/// Every `reference` term target in `body`, at any depth.
fn body_reference_targets(body: &Value, out: &mut Vec<CheckedNodeId>) {
    let mut pending = vec![body];
    while let Some(term) = pending.pop() {
        match body_term(term) {
            Some(BodyTerm::Reference) => {
                let target = term
                    .get("target")
                    .and_then(|target| serde_json::from_value(target.clone()).ok());
                out.extend(target);
            }
            Some(BodyTerm::Application) => {
                pending.extend(
                    term.get("arguments")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten(),
                );
            }
            Some(BodyTerm::Aggregate) => {
                pending.extend(
                    term.get("members")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten(),
                );
            }
            Some(BodyTerm::Binding) => pending.extend(term.get("value")),
            Some(BodyTerm::Literal | BodyTerm::DependencyReference | BodyTerm::Frame) | None => {}
        }
    }
}
