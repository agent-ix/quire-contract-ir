//! FR-322 `dependency_reference`: binding each `dependency_selections` entry
//! to the dependency package the caller supplies, and checking each
//! `{term: "dependency_reference", package, node}` term against it.
//!
//! Two stages, in the reader's first-refusal order:
//!
//! 1. [`admit_dependencies`], in the lock stage: every entry, in lock order,
//!    must have a package supplied under its identity (`missing_import` /
//!    `missing-selection` at the entry), whose own `package_id` equals the
//!    entry's (`stale_dependency` / `content-mismatch` at the entry's
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
//! `ModelOwner`. The signature takes the function node's own
//! `semantic_type` (the result type) and the `semantic_type` of each
//! `value`/`parameter` node referenced by its body's `parameters` binding.
//! The closure follows those type nodes' `dependencies`; a `model` or
//! `relation` node is a `ModelOwner` carrier.
//!
//! Documented limits, inherited from the reader's operation stage (see
//! `operations.rs`): a call's `result_type` and `semantic_type` are not
//! compared with the dependency function's result type (FR-322's `return:0`),
//! and the number and types of a call's arguments are not compared with the
//! function's parameters. What is checked is that the callee names a declared
//! `function` node of the supplied dependency whose signature is
//! package-independent.

use super::structural::{aggregate_members, binding, reference_target};
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
    pub(super) fn packages(&self) -> Vec<&'e CheckedPackageV2> {
        self.packages.iter().map(|entry| entry.package).collect()
    }
    /// Index already admitted, selection-matched packages for another closed
    /// reader that shares FR-322 dependency-reference semantics.
    pub(super) fn from_admitted(
        selections: &'e [CheckedDependencySelection],
        admitted: &BTreeMap<&str, &'e CheckedPackageV2>,
    ) -> Result<Self, usize> {
        let mut packages = Vec::with_capacity(selections.len());
        for (position, selection) in selections.iter().enumerate() {
            let Some(package) = admitted.get(selection.identity.as_ref()).copied() else {
                return Err(position);
            };
            let index = package
                .graph()
                .nodes
                .iter()
                .enumerate()
                .map(|(position, node)| (&node.node_id, position))
                .collect();
            packages.push(SuppliedDependency {
                package_id: &selection.package_id,
                package,
                index,
            });
        }
        Ok(Self { packages })
    }

    /// The package supplied for the entry whose `package_id` is `package`.
    fn resolve(&self, package: &CheckedSemanticId) -> Option<&SuppliedDependency<'e>> {
        self.packages
            .iter()
            .find(|supplied| supplied.package_id == package)
    }
}

struct SelectionPath {
    parent: Option<usize>,
    index: usize,
}

fn selection_pointer(root: &JsonPointer, paths: &[SelectionPath], path: usize) -> JsonPointer {
    let mut indices = Vec::new();
    let mut cursor = Some(path);
    while let Some(position) = cursor {
        let node = &paths[position];
        indices.push(node.index);
        cursor = node.parent;
    }
    indices.into_iter().rev().fold(root.clone(), |at, index| {
        at.key("lock").key("dependency_selections").index(index)
    })
}

/// Checks the recursive closure without copying it into every admitted package.
/// The input packages are aligned with the importing lock's direct selections.
pub(super) fn check_derived_closure<'a>(
    selections: &'a [CheckedDependencySelection],
    packages: &[&'a CheckedPackageV2],
    root: JsonPointer,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    if let ([selection], [package]) = (selections, packages) {
        if package
            .closure_index
            .get(&selection.identity)
            .is_none_or(|id| id == &selection.package_id)
        {
            // The admitted child already checked its closure. One new identity
            // cannot conflict when it is absent there or has the same ID.
            meter.charge(1, || {
                root.clone()
                    .key("lock")
                    .key("dependency_selections")
                    .index(0)
            })?;
            return Ok(());
        }
    }
    let mut paths = Vec::new();
    let mut pending = Vec::new();
    for (index, (selection, package)) in selections.iter().zip(packages).enumerate().rev() {
        paths.push(SelectionPath {
            parent: None,
            index,
        });
        pending.push((selection, *package, paths.len() - 1));
    }
    let mut seen: BTreeMap<&str, (&CheckedSemanticId, usize)> = BTreeMap::new();
    while let Some((selection, package, path)) = pending.pop() {
        meter.charge(1, || selection_pointer(&root, &paths, path))?;
        if let Some((prior_id, prior_path)) = seen.get(selection.identity.as_ref()) {
            if *prior_id != &selection.package_id {
                let mut refusal = ValidationFailure::refused_because(
                    CheckedPackageRefusalCode::InvalidPackage,
                    selection_pointer(&root, &paths, *prior_path),
                    CheckedPackageRefusalCause::ConflictingDefinition,
                );
                if let ValidationFailure::Refused(ref mut value) = refusal {
                    value.related_path = Some(selection_pointer(&root, &paths, path));
                }
                return Err(refusal);
            }
            continue;
        }
        seen.insert(selection.identity.as_ref(), (&selection.package_id, path));
        for (index, (child_selection, child_package)) in package
            .lock()
            .dependency_selections
            .iter()
            .zip(&package.direct_dependencies)
            .enumerate()
            .rev()
        {
            paths.push(SelectionPath {
                parent: Some(path),
                index,
            });
            pending.push((child_selection, child_package.as_ref(), paths.len() - 1));
        }
    }
    Ok(())
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
            return Err(ValidationFailure::Refused(Box::new(
                crate::checked_package::shared::CheckedPackageRefusal::mismatch(
                    at(index).key("package_id").key("digest"),
                    CheckedPackageRefusalCause::ContentMismatch,
                    entry.package_id.clone(),
                    package.package_id().clone(),
                ),
            )));
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
            // visited, including each parameter node.
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
    let members = aggregate_members(&function.body);
    let parameters = members.and_then(|members| {
        members
            .first()
            .and_then(|first| binding(first, "parameters"))
    });
    if let Some(parameters) = parameters {
        let Some(entries) = aggregate_members(parameters) else {
            return Closure {
                independent: false,
                work,
            };
        };
        for entry in entries {
            let Some((node, CheckedNodeKind::Value(ValueForm::Parameter))) =
                reference_target(entry).as_ref().and_then(&node_of)
            else {
                return Closure {
                    independent: false,
                    work,
                };
            };
            work = work.saturating_add(1);
            pending.push(node.semantic_type.clone());
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

/// Selected package/key pairs retained in an admitted body's dependency
/// references. The same term walk is used by admission above.
pub(super) fn selected_reference_targets(
    body: &Value,
) -> Result<Vec<(CheckedSemanticId, CheckedNodeId)>, ()> {
    let mut selected = Vec::new();
    let walked = visit_terms(body, |term| -> ControlFlow<()> {
        if body_term(term) == Some(BodyTerm::DependencyReference) {
            let (Some(package), Some(node)) = (
                dependency_reference_package(term),
                dependency_reference_node(term),
            ) else {
                return ControlFlow::Break(());
            };
            selected.push((package, node));
        }
        ControlFlow::Continue(())
    });
    match walked {
        ControlFlow::Continue(()) => Ok(selected),
        ControlFlow::Break(()) => Err(()),
    }
}
