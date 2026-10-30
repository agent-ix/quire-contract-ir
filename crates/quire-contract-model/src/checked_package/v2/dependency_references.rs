//! FR-322 `dependency_reference`: binding each `dependency_selections` entry
//! to the dependency package the caller supplies, and checking each
//! `{term: "dependency_reference", package, node}` term against it.
//!
//! Two stages, in the reader's first-refusal order:
//!
//! 1. [`admit_dependencies`], in the lock stage: every entry, in lock order,
//!    must have a package supplied under its name (`missing_import` /
//!    `missing-selection` at the entry), whose own name equals the entry's
//!    (`stale_dependency` / `content-mismatch` at the entry's `identity`).
//!    This runs before any node is read, so before any `dependency_reference`
//!    check.
//! 2. [`DependencyReferences::walk_admitted`] in step 7 of the operation stage
//!    and strict contract-package intake: a
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

use super::{
    CheckedDependencySelection, CheckedNodeKind, CheckedNodeTag, CheckedPackageV2, CheckedTerm,
    NominalIdentityPreimage, NominalOwner, RawSemanticNodeV2 as CheckedSemanticNodeV2, ValueForm,
    WorkMeter,
};
use crate::checked_package::common::{count, node_pointer, ValidationFailure};
use crate::checked_package::evidence::CheckedPackageEvidence;
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// The operation whose callee (argument 0) a `dependency_reference` may be.
const FUNCTION_CALL_OPERATION: &str = "quire.op.function.call";

/// One supplied dependency, with an index of its graph built once when the
/// lock stage binds it.
#[derive(Debug)]
pub(super) struct SuppliedDependency<'e> {
    identity: &'e str,
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
    /// The admitted directly selected package with this exact identity.
    pub(super) fn selected_package(&self, package: &str) -> Option<&'e CheckedPackageV2> {
        self.resolve(package).map(|entry| entry.package)
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
                identity: &selection.identity,
                package,
                index,
            });
        }
        Ok(Self { packages })
    }

    /// The package supplied for this exact dependency identity.
    fn resolve(&self, package: &str) -> Option<&SuppliedDependency<'e>> {
        self.packages
            .iter()
            .find(|supplied| supplied.identity == package)
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
            .is_none_or(|candidate| candidate.wire == package.wire)
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
    let mut seen: BTreeMap<&str, (&CheckedPackageV2, usize)> = BTreeMap::new();
    while let Some((selection, package, path)) = pending.pop() {
        meter.charge(1, || selection_pointer(&root, &paths, path))?;
        if let Some((prior_package, prior_path)) = seen.get(selection.identity.as_ref()) {
            if prior_package.wire != package.wire {
                let mut refusal = ValidationFailure::refused_because(
                    CheckedPackageRefusalCode::InvalidPackage,
                    selection_pointer(&root, &paths, *prior_path),
                    CheckedPackageRefusalCause::ConflictingDefinition,
                );
                if let ValidationFailure::Refused(ref mut value) = refusal {
                    value.push_path(selection_pointer(&root, &paths, path));
                }
                return Err(refusal);
            }
            continue;
        }
        seen.insert(selection.identity.as_ref(), (package, path));
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
        if package.name() != entry.identity.as_ref() {
            return Err(ValidationFailure::refused_because(
                CheckedPackageRefusalCode::StaleDependency,
                at(index).key("identity"),
                CheckedPackageRefusalCause::ContentMismatch,
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
            identity: &entry.identity,
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

    /// Checks an already admitted body without selecting terms by wire keys.
    pub(super) fn walk_admitted(
        self,
        referrer: Referrer<'_>,
        root: &CheckedTerm,
        meter: &mut WorkMeter,
    ) -> Result<(), ValidationFailure> {
        let mut pending = vec![(root, node_pointer(referrer.position).key("body"), false)];
        while let Some((term, at, is_callee)) = pending.pop() {
            match term {
                CheckedTerm::DependencyReference { package, node } => {
                    self.check_parts(referrer, package, node, &at, is_callee, meter)?;
                }
                CheckedTerm::Application {
                    operation,
                    arguments,
                    ..
                } => {
                    let calls_function = operation.get("identity").and_then(Value::as_str)
                        == Some(FUNCTION_CALL_OPERATION);
                    for (index, child) in arguments.iter().enumerate().rev() {
                        pending.push((
                            child,
                            at.clone().key("arguments").index(index),
                            index == 0 && calls_function,
                        ));
                    }
                }
                CheckedTerm::Aggregate { members } => {
                    for (index, child) in members.iter().enumerate().rev() {
                        pending.push((child, at.clone().key("members").index(index), false));
                    }
                }
                CheckedTerm::OperationAnchorBody(body) => {
                    for (index, child) in body.members.iter().enumerate().rev() {
                        pending.push((child, at.clone().key("members").index(index), false));
                    }
                }
                CheckedTerm::Binding { value, .. } => {
                    pending.push((value, at.key("value"), false));
                }
                CheckedTerm::Literal { .. }
                | CheckedTerm::Reference { .. }
                | CheckedTerm::Frame { .. }
                | CheckedTerm::AbstractionRelation { .. }
                | CheckedTerm::Refinement { .. }
                | CheckedTerm::ExtensionForm { .. } => {}
            }
        }
        Ok(())
    }

    fn check_parts(
        self,
        referrer: Referrer<'_>,
        package: &str,
        node: &CheckedNodeId,
        at: &JsonPointer,
        is_callee: bool,
        meter: &mut WorkMeter,
    ) -> Result<(), ValidationFailure> {
        let Some(supplied) = self.supplied.resolve(package) else {
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
            .get(node)
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
    let members = match &function.body {
        CheckedTerm::Aggregate { members } => Some(members.as_slice()),
        _ => None,
    };
    let parameters = members.and_then(|members| match members.first()? {
        CheckedTerm::Binding { name, value }
            if name.kind() == super::TermBindingName::Parameters =>
        {
            Some(value.as_ref())
        }
        _ => None,
    });
    if let Some(parameters) = parameters {
        let CheckedTerm::Aggregate { members: entries } = parameters else {
            return Closure {
                independent: false,
                work,
            };
        };
        for entry in entries {
            let Some((node, CheckedNodeKind::Value(ValueForm::Parameter))) = (match entry {
                CheckedTerm::Reference { target } => Some(target),
                _ => None,
            })
            .and_then(node_of) else {
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
fn carries_declaration_or_owner<Body>(
    node: &super::CheckedSemanticNodeV2<Body>,
    kind: CheckedNodeKind,
) -> bool {
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
/// references, in document pre-order.
pub(super) fn selected_reference_targets(body: &CheckedTerm) -> Vec<(Box<str>, CheckedNodeId)> {
    let mut selected = Vec::new();
    let mut pending = vec![body];
    while let Some(term) = pending.pop() {
        match term {
            CheckedTerm::DependencyReference { package, node } => {
                selected.push((package.clone(), node.clone()));
            }
            CheckedTerm::Application { arguments, .. } => pending.extend(arguments.iter().rev()),
            CheckedTerm::Aggregate { members } => pending.extend(members.iter().rev()),
            CheckedTerm::OperationAnchorBody(body) => pending.extend(body.members.iter().rev()),
            CheckedTerm::Binding { value, .. } => pending.push(value),
            CheckedTerm::Literal { .. }
            | CheckedTerm::Reference { .. }
            | CheckedTerm::Frame { .. }
            | CheckedTerm::AbstractionRelation { .. }
            | CheckedTerm::Refinement { .. }
            | CheckedTerm::ExtensionForm { .. } => {}
        }
    }
    selected
}
