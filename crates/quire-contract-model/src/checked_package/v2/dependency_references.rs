//! FR-322 `dependency_reference`: binding each `dependency_selections` entry
//! to the dependency package the caller supplies, and checking each
//! `{term: "dependency_reference", package, node}` term against it.
//!
//! Two stages, in the reader's first-refusal order:
//!
//! 1. [`admit_dependencies`], in the lock stage: every entry, in lock order,
//!    must have a package supplied under its identity (`missing_import` /
//!    `missing-selection` at the entry), whose own `package_id` equals the
//!    entry's (`stale_dependency` / `byte-digest-mismatch` at the entry's
//!    `package_id.digest`). This runs before any node is read, so before any
//!    `dependency_reference` check.
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
//!
//! Documented limits, inherited from the reader's operation stage (see
//! `operations.rs`): a call's `result_type` and `semantic_type` are not
//! compared with the dependency function's result type (FR-322's `return:0`),
//! and the number and types of a call's arguments are not compared with the
//! function's parameters. What is checked is that the callee names a declared
//! `function` node of the supplied dependency whose signature is
//! package-independent.

use super::{
    BodyTerm, CheckedDependencySelection, CheckedNodeKind, CheckedNodeTag, CheckedPackageV2,
    CheckedSemanticNodeV2, NominalIdentityPreimage, NominalOwner, ValueForm, WorkMeter,
};
use crate::checked_package::common::{
    body_term, count, dependency_reference_node, dependency_reference_package, Step, Trail,
    ValidationFailure,
};
use crate::checked_package::evidence::CheckedPackageEvidence;
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedSemanticId,
    JsonPointer,
};
use crate::checked_package::terms::{subterms, visit_terms, At, Cursor};
use quire_walk::{walk, Children, Walk};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::Infallible;
use std::ops::ControlFlow;

/// The operation whose callee (argument 0) a `dependency_reference` may be.
const FUNCTION_CALL_OPERATION: &str = "quire.op.function.call";

/// One supplied dependency, with an index of its graph built once when the
/// lock stage binds it.
#[derive(Debug)]
pub(super) struct SuppliedDependency<'e> {
    package_id: &'e CheckedSemanticId,
    package: &'e CheckedPackageV2,
    index: BTreeMap<&'e CheckedNodeId, usize>,
}

/// The dependency packages the caller supplied, one per
/// `dependency_selections` entry and in the same order.
#[derive(Debug, Default)]
pub(super) struct SuppliedDependencies<'e> {
    packages: Vec<SuppliedDependency<'e>>,
}

impl<'e> SuppliedDependencies<'e> {
    /// The package supplied for the entry whose `package_id` is `package`.
    fn resolve(&self, package: &CheckedSemanticId) -> Option<&SuppliedDependency<'e>> {
        self.packages
            .iter()
            .find(|supplied| supplied.package_id == package)
    }
}

/// Binds every selection entry to the package `evidence` supplies for its
/// identity. The first entry, in lock order, that is unsupplied or does not
/// bind refuses. Indexing each bound package's graph is charged to `meter`
/// once, at the entry, one unit per node.
pub(super) fn admit_dependencies<'e>(
    selections: &'e [CheckedDependencySelection],
    evidence: &'e CheckedPackageEvidence,
    meter: &mut WorkMeter,
) -> Result<SuppliedDependencies<'e>, ValidationFailure> {
    let at = |index: usize| {
        JsonPointer::root()
            .key("lock")
            .key("dependency_selections")
            .index(index)
    };
    let mut packages = Vec::with_capacity(selections.len());
    for (index, entry) in selections.iter().enumerate() {
        let Some(package) = evidence.dependency_package(&entry.identity) else {
            return Err(ValidationFailure::refused_because(
                CheckedPackageRefusalCode::MissingImport,
                at(index),
                CheckedPackageRefusalCause::MissingSelection,
            ));
        };
        // A `CheckedPackageV2` exists only as the reader's admitted result,
        // so its `package_id` is the digest the reader recomputed from its
        // identity preimage; the binding rests on that content digest.
        if package.package_id() != &entry.package_id {
            return Err(ValidationFailure::refused_because(
                CheckedPackageRefusalCode::StaleDependency,
                at(index).key("package_id").key("digest"),
                CheckedPackageRefusalCause::ByteDigestMismatch,
            ));
        }
        let nodes = &package.graph().nodes;
        meter.charge(count(nodes.len()), || at(index))?;
        let index_of = nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect();
        packages.push(SuppliedDependency {
            package_id: &entry.package_id,
            package,
            index: index_of,
        });
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
        self.walk_terms(referrer, meter)
    }

    /// Checks every `dependency_reference` under the arguments of the
    /// application that is `referrer`'s body root, in pre-order. The
    /// application's own checks are the caller's. An application's own term is
    /// no `dependency_reference`, so this is the whole-body walk of an
    /// application body.
    pub(super) fn walk_arguments(
        self,
        referrer: Referrer<'_>,
        meter: &mut WorkMeter,
    ) -> Result<(), ValidationFailure> {
        self.walk_terms(referrer, meter)
    }

    /// Enters the terms of `referrer`'s body in document pre-order on
    /// `quire-walk`'s heap stack, checking each `dependency_reference`. The
    /// first argument of a `quire.op.function.call` application is its callee.
    fn walk_terms(
        self,
        referrer: Referrer<'_>,
        meter: &mut WorkMeter,
    ) -> Result<(), ValidationFailure> {
        let steps = [
            Step::Key("semantic_graph"),
            Step::Key("nodes"),
            Step::Index(referrer.position),
            Step::Key("body"),
        ];
        let cursor = Cursor::at(&Trail::Base(&steps));
        let root = cursor.root(&referrer.node.body, false);
        let mut references = Dependencies {
            references: self,
            referrer,
            meter,
            cursor,
        };
        match walk(&mut references, root) {
            ControlFlow::Continue(()) => Ok(()),
            ControlFlow::Break(failure) => Err(failure),
        }
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
        let Some(supplied) = self.supplied.resolve(&package) else {
            return Err(referrer.refuse(
                CheckedPackageRefusalCode::MissingDeclaration,
                CheckedPackageRefusalCause::MissingSelection,
                at.clone().key("package"),
            ));
        };
        let dependency = supplied.package;
        // The lookup is one unit, charged at the term.
        meter.charge(1, || at.clone())?;
        let target = supplied
            .index
            .get(&node)
            .and_then(|position| Some((*position, dependency.graph().nodes.get(*position)?)))
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
        if is_callee && is_function {
            let closure = signature_closure(supplied, position);
            // The closure walk is charged at the term: one unit per node
            // visited and per body term walked.
            meter.charge(closure.work, || at.clone())?;
            if closure.independent {
                return Ok(());
            }
        }
        Err(referrer.refuse(
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::OperatorIneligible,
            at.clone(),
        ))
    }
}

/// The walk of [`DependencyReferences::walk_terms`]: each term with whether it
/// is the callee of a function call.
struct Dependencies<'r, 'a, 'm> {
    references: DependencyReferences<'r>,
    referrer: Referrer<'a>,
    meter: &'m mut WorkMeter,
    cursor: Cursor<'a>,
}

impl<'a> Walk for Dependencies<'_, 'a, '_> {
    type Node = At<'a, bool>;
    type Frame = ();
    type Stop = ValidationFailure;

    fn enter(
        &mut self,
        node: At<'a, bool>,
        children: &mut Children<'_, At<'a, bool>>,
    ) -> ControlFlow<ValidationFailure> {
        self.cursor.enter(&node);
        match body_term(node.value) {
            Some(BodyTerm::DependencyReference) => {
                let at = self.cursor.trail().pointer();
                if let Err(failure) =
                    self.references
                        .check(self.referrer, node.value, &at, node.extra, self.meter)
                {
                    return ControlFlow::Break(failure);
                }
            }
            Some(BodyTerm::Application) => {
                let calls_function = is_function_call(node.value);
                children.extend(subterms(node.value).map(|subterm| {
                    let is_callee = subterm.index == Some(0) && calls_function;
                    self.cursor.subterm(subterm, is_callee)
                }));
            }
            Some(BodyTerm::Aggregate | BodyTerm::Binding) => {
                children.extend(
                    subterms(node.value).map(|subterm| self.cursor.subterm(subterm, false)),
                );
            }
            Some(
                BodyTerm::Literal
                | BodyTerm::Reference
                | BodyTerm::Frame
                | BodyTerm::AbstractionRelation,
            )
            | None => {}
        }
        ControlFlow::Continue(())
    }

    fn exit(&mut self, (): ()) -> ControlFlow<ValidationFailure> {
        ControlFlow::Continue(())
    }
}

/// Whether an application term is a `quire.op.function.call`.
// Decodes the wire operation identity to tell a function call from any other operation.
fn is_function_call(application: &Value) -> bool {
    application
        .get("operation")
        .and_then(|operation| operation.get("identity"))
        .and_then(Value::as_str)
        .is_some_and(|identity| identity == FUNCTION_CALL_OPERATION)
}

/// The outcome of walking a function's signature closure and the work it
/// took.
struct Closure {
    independent: bool,
    work: u64,
}

/// Implements: FR-322. No node in the transitive closure of the function's signature
/// type nodes carries a `declaration` or a `ModelOwner`.
fn signature_closure(supplied: &SuppliedDependency<'_>, function: usize) -> Closure {
    let nodes = &supplied.package.graph().nodes;
    let kinds = supplied.package.node_kinds();
    let node_of = |id: &CheckedNodeId| {
        let position = *supplied.index.get(id)?;
        Some((nodes.get(position)?, *kinds.get(position)?))
    };
    let Some(function) = nodes.get(function) else {
        return Closure {
            independent: false,
            work: 1,
        };
    };
    let mut work = 1_u64;
    let mut pending: Vec<CheckedNodeId> = vec![function.semantic_type.clone()];
    // The function's signature is what its `dependencies` list and what its
    // body references: a non-application function body need not list them.
    let mut listed: Vec<CheckedNodeId> = function.dependencies.clone();
    work = work.saturating_add(body_reference_targets(&function.body, &mut listed));
    for id in &listed {
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
                | ValueForm::UnionValue
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
        work = work.saturating_add(1);
        if carries_declaration_or_owner(node, kind) {
            return Closure {
                independent: false,
                work,
            };
        }
        pending.extend(node.dependencies.iter().cloned());
        pending.push(node.semantic_type.clone());
        work = work.saturating_add(body_reference_targets(&node.body, &mut pending));
    }
    Closure {
        independent: true,
        work,
    }
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

/// Every `reference` term target in `body`, at any depth, and the number of
/// terms walked.
fn body_reference_targets(body: &Value, out: &mut Vec<CheckedNodeId>) -> u64 {
    let mut walked = 0_u64;
    let walked_all = visit_terms(body, |term| -> ControlFlow<Infallible> {
        walked = walked.saturating_add(1);
        if body_term(term) == Some(BodyTerm::Reference) {
            out.extend(
                term.get("target")
                    .and_then(|target| serde_json::from_value(target.clone()).ok()),
            );
        }
        ControlFlow::Continue(())
    });
    let ControlFlow::Continue(()) = walked_all;
    walked
}
