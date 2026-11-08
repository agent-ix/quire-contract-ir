//! FR-322 `dependency_reference`: binding each `dependency_selections` entry
//! to the dependency package the caller supplies, and checking each
//! `{term: "dependency_reference", package, node}` term against it.
//!
//! Two stages, in the reader's first-refusal order:
//!
//! 1. [`admit_dependencies`], in the lock stage: every entry, in lock order,
//!    must have a package supplied under its name with the same package name;
//!    absence or a nonmatching name refuses `missing_import` /
//!    `missing-selection` at the entry.
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

use super::model_members::{Budget, SortedMap};
use super::native_join_budget::{NativeJoinChargeKind, PackageReadStage};
use super::read_charge::ReadStorage;
use super::stage_storage::{self, Path, Refusal, Site};
use super::{
    CheckedDependencySelection, CheckedNodeKind, CheckedNodeTag, CheckedPackageV2, CheckedTerm,
    RawSemanticNodeV2 as CheckedSemanticNodeV2, ValueForm, WorkMeter,
};
use crate::checked_package::common::{
    count, node_pointer, Step as PointerStep, Trail, ValidationFailure,
};
use crate::checked_package::evidence::CheckedPackageEvidence;
use crate::checked_package::shared::{
    CheckedNodeId, CheckedNodePathSegment, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    ForeignNodeKey, JsonPointer, NodeKeyProbe,
};

/// The form of every request the dependency checks make.
const F: &str = "dependencies";

/// The operation whose callee (argument 0) a `dependency_reference` may be.
const FUNCTION_CALL_OPERATION: &str = "quire.op.function.call";

/// One supplied dependency, with an index of its graph built once when the
/// lock stage binds it.
#[derive(Debug)]
pub(super) struct SuppliedDependency<'e> {
    identity: &'e str,
    package: &'e CheckedPackageV2,
    /// The package's node ids with their positions, sorted by id.
    index: Vec<(&'e CheckedNodeId, usize)>,
}

impl SuppliedDependency<'_> {
    /// The position of the graph node with this id.
    fn position(&self, id: &CheckedNodeId) -> Option<usize> {
        let after = self.index.partition_point(|row| row.0 <= id);
        let row = self.index.get(after.checked_sub(1)?)?;
        (row.0 == id).then_some(row.1)
    }
}

/// `table` filled with the node ids of `package`'s graph and their
/// positions, sorted by id. It holds the graph's node count already, so
/// filling it allocates nothing.
fn fill_table<'e>(
    package: &'e CheckedPackageV2,
    mut table: Vec<(&'e CheckedNodeId, usize)>,
) -> Vec<(&'e CheckedNodeId, usize)> {
    table.extend(
        package
            .resolved_graph()
            .nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position)),
    );
    table.sort_unstable_by(|a, b| a.0.cmp(b.0).then(a.1.cmp(&b.1)));
    table
}

impl SuppliedDependency<'_> {
    /// The position of the node `id` names in this dependency's graph. The
    /// dependency's own ids hold index segments of its graph; an importing
    /// document names those as `{package, node}` dependency positions of this
    /// dependency, and a local index segment names no node here.
    fn position_of(&self, id: &CheckedNodeId) -> Option<usize> {
        // A local index segment, or a position of another dependency, names
        // no node here. The lookup reads the rest in place.
        if id.path.iter().any(|segment| match segment {
            CheckedNodePathSegment::Dependency { package, .. } => package.as_ref() != self.identity,
            CheckedNodePathSegment::Index(_) => true,
            _ => false,
        }) {
            return None;
        }
        let key = ForeignNodeKey::new(id, self.identity);
        let probe = &key as &dyn NodeKeyProbe;
        let first = self
            .index
            .partition_point(|row| (row.0 as &dyn NodeKeyProbe).cmp(probe).is_lt());
        self.index
            .get(first)
            .filter(|row| (row.0 as &dyn NodeKeyProbe).cmp(probe).is_eq())
            .map(|row| row.1)
    }
}

/// The dependency packages the caller supplied, one per
/// `dependency_selections` entry and in the same order.
#[derive(Debug, Default)]
pub(super) struct SuppliedDependencies<'e> {
    packages: Vec<SuppliedDependency<'e>>,
}

/// One selected graph's structural tokens and corresponding node positions.
type SelectedStructuralGraph<'e> = (&'e str, &'e CheckedPackageV2, Vec<(usize, usize)>);

/// Per-read cross-document equality lookup. Tokens are interned across all
/// selected graphs and the importing graph from shallow structural descriptors.
pub(super) struct StructuralImportIndex<'e> {
    importer: Vec<usize>,
    /// Each selected package, with its `(token, position)` pairs in token order.
    selected: Vec<SelectedStructuralGraph<'e>>,
}

impl<'e> StructuralImportIndex<'e> {
    pub(super) fn lookup(
        &self,
        package: &str,
        importing_position: usize,
    ) -> Option<(usize, &'e CheckedPackageV2)> {
        let token = self.importer.get(importing_position)?;
        self.selected
            .iter()
            .find(|(identity, _, _)| *identity == package)
            .and_then(|(_, selected, positions)| {
                let at = positions
                    .binary_search_by_key(token, |(held, _)| *held)
                    .ok()?;
                Some((positions[at].1, *selected))
            })
    }
}

impl<'e> SuppliedDependencies<'e> {
    pub(super) fn packages(
        &self,
        meter: &mut WorkMeter,
    ) -> Result<Vec<&'e CheckedPackageV2>, ValidationFailure> {
        let mut packages = meter.vec(self.packages.len())?;
        packages.extend(self.packages.iter().map(|entry| entry.package));
        Ok(packages)
    }
    /// Whether a directly selected package holds the node `id`: a
    /// component composition's subject or clause in a component package.
    pub(super) fn holds_node(&self, id: &CheckedNodeId) -> bool {
        self.packages
            .iter()
            .any(|supplied| supplied.position_of(id).is_some())
    }

    /// The node `id` of the directly selected package `package`: the package
    /// and the node's position in it.
    pub(super) fn selected_node(
        &self,
        package: &str,
        id: &CheckedNodeId,
    ) -> Option<(&'e CheckedPackageV2, usize)> {
        let supplied = self.resolve(package)?;
        Some((supplied.package, supplied.position_of(id)?))
    }

    /// The admitted directly selected package with this exact identity.
    pub(super) fn selected_package(&self, package: &str) -> Option<&'e CheckedPackageV2> {
        self.resolve(package).map(|entry| entry.package)
    }

    /// Build one bounded lookup for imported nodes, sharing compact tokens
    /// across all selected packages and the importing graph.
    pub(super) fn structural_import_index(
        &self,
        importer_keys: &[std::sync::Arc<CheckedNodeId>],
        meter: &mut WorkMeter,
    ) -> Result<StructuralImportIndex<'e>, ValidationFailure> {
        // One order over the importer and every selected graph, so equal keys
        // of different documents share a token. The tables below are plain
        // integers, so the order itself ends with this call.
        let mut lists: Vec<&[std::sync::Arc<CheckedNodeId>]> =
            meter.vec(self.packages.len() + 1)?;
        lists.push(importer_keys);
        lists.extend(
            self.packages
                .iter()
                .map(|package| package.package.structural_keys.as_slice()),
        );
        let order = meter
            .structural_order(&lists)?
            .expect("reader rebuilt each key of every selected graph");
        let mut importer = meter.vec(importer_keys.len())?;
        for key in importer_keys {
            importer.push(order.token(key).expect("reader rebuilt each importer key"));
        }
        meter.charge(count(importer.len()), || {
            JsonPointer::root().key("semantic_graph").key("nodes")
        })?;
        let mut selected = meter.vec(self.packages.len())?;
        for package in &self.packages {
            let keys = &package.package.structural_keys;
            let mut positions = meter.vec(keys.len())?;
            for (position, key) in keys.iter().enumerate() {
                let token = order
                    .token(key)
                    .expect("admitted dependency has rebuilt keys");
                positions.push((token, position));
            }
            positions.sort_unstable();
            meter.charge(count(positions.len()), || {
                JsonPointer::root().key("lock").key("dependency_selections")
            })?;
            selected.push((package.identity, package.package, positions));
        }
        Ok(StructuralImportIndex { importer, selected })
    }
    /// Index already admitted, selection-matched packages for another closed
    /// reader that shares FR-322 dependency-reference semantics.
    pub(super) fn from_admitted(
        selections: &'e [CheckedDependencySelection],
        admitted: impl Fn(&str) -> Option<&'e CheckedPackageV2>,
    ) -> Result<Self, usize> {
        let mut packages = Vec::with_capacity(selections.len());
        for (position, selection) in selections.iter().enumerate() {
            let Some(package) = admitted(selection.identity.as_ref()) else {
                return Err(position);
            };
            let index = fill_table(
                package,
                Vec::with_capacity(package.resolved_graph().nodes.len()),
            );
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

#[derive(Clone, Copy)]
struct SelectionPath {
    parent: Option<usize>,
    index: usize,
    /// The entry of the importing lock this path descends from.
    root: usize,
}

fn selection_pointer(root: &Path<'_>, paths: &[SelectionPath], path: usize) -> JsonPointer {
    let mut indices = Vec::new();
    let mut cursor = Some(path);
    while let Some(position) = cursor {
        let node = &paths[position];
        indices.push(node.index);
        cursor = node.parent;
    }
    indices.into_iter().rev().fold(root.build(), |at, index| {
        at.key("lock").key("dependency_selections").index(index)
    })
}

/// Checks the recursive closure without copying it into every admitted package.
/// The input packages are aligned with the importing lock's direct selections.
pub(super) fn check_derived_closure<'a>(
    selections: &'a [CheckedDependencySelection],
    packages: &[&'a CheckedPackageV2],
    root: Path<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut budget = Budget::lock(meter, "dependency_selections");
    if let ([selection], [package]) = (selections, packages) {
        if package
            .closure_index
            .get(&selection.identity)
            .is_none_or(|candidate| candidate.wire == package.wire)
        {
            // The admitted child already checked its closure. One new identity
            // cannot conflict when it is absent there or has the same ID.
            budget.charge_at_position(
                1,
                F,
                root.key("lock").key("dependency_selections").index(0),
            )?;
            return Ok(());
        }
    }
    let mut paths: Vec<SelectionPath> = budget.vec(selections.len(), F)?;
    let mut pending: Vec<(&CheckedDependencySelection, &CheckedPackageV2, usize)> =
        budget.vec(selections.len(), F)?;
    for (index, (selection, package)) in selections.iter().zip(packages).enumerate().rev() {
        budget.set_index(index);
        budget.push(
            &mut paths,
            SelectionPath {
                parent: None,
                index,
                root: index,
            },
            F,
        )?;
        pending.push((selection, *package, paths.len() - 1));
    }
    let mut seen: SortedMap<&str, (&CheckedPackageV2, usize)> = SortedMap::new();
    while let Some((selection, package, path)) = pending.pop() {
        let entry = paths[path].root;
        budget.set_index(entry);
        budget.charge_at(1, F, || selection_pointer(&root, &paths, path))?;
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
        seen.insert(&mut budget, selection.identity.as_ref(), (package, path), F)?;
        for (index, (child_selection, child_package)) in package
            .lock()
            .dependency_selections
            .iter()
            .zip(&package.direct_dependencies)
            .enumerate()
            .rev()
        {
            budget.push(
                &mut paths,
                SelectionPath {
                    parent: Some(path),
                    index,
                    root: entry,
                },
                F,
            )?;
            budget.push(
                &mut pending,
                (child_selection, child_package.as_ref(), paths.len() - 1),
                F,
            )?;
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
        Path::root()
            .key("lock")
            .key("dependency_selections")
            .index(index)
    };
    let mut packages = Budget::lock(meter, "dependency_selections").vec(selections.len(), F)?;
    for (index, entry) in selections.iter().enumerate() {
        let package = match evidence.dependency_package(&entry.identity) {
            Some(package) if package.name() == entry.identity.as_ref() => package,
            _ => {
                let path = at(index);
                return Err(stage_storage::refuse(
                    meter,
                    &Site::new(PackageReadStage::Lock, index, F, path),
                    Refusal::new(CheckedPackageRefusalCode::MissingImport, path)
                        .because(CheckedPackageRefusalCause::MissingSelection),
                ));
            }
        };
        let mut budget = Budget::lock(meter, "dependency_selections");
        let nodes = &package.resolved_graph().nodes;
        budget.set_index(index);
        budget.charge_at_position(count(nodes.len()), F, at(index))?;
        let table = budget.vec(nodes.len(), F)?;
        packages.push(SuppliedDependency {
            identity: &entry.identity,
            package,
            index: fill_table(package, table),
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
        at: &Place<'_>,
        member: Option<&str>,
        budget: &mut Budget<'_, '_>,
        form: &'static str,
    ) -> ValidationFailure {
        let (count, _) = at.layout(member);
        if let Some(reader) = budget.reader_budget(form) {
            let bytes = count.saturating_mul(std::mem::size_of::<PointerStep<'_>>());
            if reader
                .reserve_diagnostic(
                    bytes as u128,
                    super::read_charge::Site::new(PackageReadStage::Graph, self.position, form),
                    super::CheckedPackagePendingOutcome::Refused(code),
                )
                .is_err()
            {
                return reader.denial();
            }
        }
        let steps = at.flat_steps(member, count);
        let path = super::stage_storage::Position::from(Trail::Base(&steps));
        // `over` only borrows the existing budget; it does not reset caller
        // work, first-denial records, or either success marker. This meter is
        // used solely by the diagnostic factory, which makes no work charge.
        let mut meter = match budget.reader_budget(form) {
            Some(reader) => WorkMeter::over(reader),
            None => WorkMeter::new(u64::MAX),
        };
        stage_storage::refuse(
            &mut meter,
            &Site::new(PackageReadStage::Graph, self.position, form, path),
            Refusal::new(code, path)
                .because(cause)
                .at_node(&self.node.node_id),
        )
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
        let form =
            CheckedNodeTag::from_wire(&referrer.node.node_tag).map_or("", CheckedNodeTag::as_wire);
        let mut budget = Budget::node(meter, referrer.position);
        // A term stands at a step of a chain that leads back to the node's
        // body; the pointer is built from it only when a refusal needs it.
        let mut steps: Vec<Step> = budget.vec(1, form)?;
        steps.push(Step {
            parent: None,
            segment: Segment::Body,
        });
        let mut pending: Vec<(&CheckedTerm, usize, bool)> = budget.vec(1, form)?;
        pending.push((root, 0, false));
        while let Some((term, step, is_callee)) = pending.pop() {
            let at = Place {
                steps: &steps,
                step,
                position: referrer.position,
            };
            match term {
                CheckedTerm::DependencyReference { package, node } => {
                    self.check_parts(referrer, package, node, &at, is_callee, &mut budget, form)?;
                }
                CheckedTerm::Application {
                    operation,
                    arguments,
                    ..
                } => {
                    let calls_function = operation.identity() == Some(FUNCTION_CALL_OPERATION);
                    for (index, child) in arguments.iter().enumerate().rev() {
                        let child_step = steps.len();
                        budget.push(
                            &mut steps,
                            Step {
                                parent: Some(step),
                                segment: Segment::Arguments(index),
                            },
                            form,
                        )?;
                        budget.push(
                            &mut pending,
                            (child, child_step, index == 0 && calls_function),
                            form,
                        )?;
                    }
                }
                CheckedTerm::Aggregate { members } => {
                    push_members(&mut budget, &mut steps, &mut pending, step, members, form)?;
                }
                CheckedTerm::OperationAnchorBody(body) => {
                    push_members(
                        &mut budget,
                        &mut steps,
                        &mut pending,
                        step,
                        &body.members,
                        form,
                    )?;
                }
                CheckedTerm::Binding { value, .. } => {
                    let child_step = steps.len();
                    budget.push(
                        &mut steps,
                        Step {
                            parent: Some(step),
                            segment: Segment::Value,
                        },
                        form,
                    )?;
                    budget.push(&mut pending, (value, child_step, false), form)?;
                }
                CheckedTerm::Literal { .. }
                | CheckedTerm::Reference { .. }
                | CheckedTerm::Frame { .. }
                | CheckedTerm::AbstractionRelation { .. }
                | CheckedTerm::ImplementationGuarantee { .. }
                | CheckedTerm::ComponentComposition { .. }
                | CheckedTerm::Refinement { .. }
                | CheckedTerm::ExtensionForm { .. }
                | CheckedTerm::NominalBody(_) => {}
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn check_parts(
        self,
        referrer: Referrer<'_>,
        package: &str,
        node: &CheckedNodeId,
        at: &Place<'_>,
        is_callee: bool,
        budget: &mut Budget<'_, '_>,
        form: &'static str,
    ) -> Result<(), ValidationFailure> {
        let Some(supplied) = self.supplied.resolve(package) else {
            return Err(referrer.refuse(
                CheckedPackageRefusalCode::MissingDeclaration,
                CheckedPackageRefusalCause::MissingSelection,
                at,
                Some("package"),
                budget,
                form,
            ));
        };
        let dependency = supplied.package;
        // The lookup is one unit, charged at the term.
        at.charge(1, budget, form)?;
        let target = supplied
            .position(node)
            .and_then(|position| Some((position, dependency.resolved_graph().nodes.get(position)?)))
            .filter(|(_, target)| target.declaration.is_some());
        let Some((position, _)) = target else {
            return Err(referrer.refuse(
                CheckedPackageRefusalCode::MissingDeclaration,
                CheckedPackageRefusalCause::MissingName,
                at,
                Some("node"),
                budget,
                form,
            ));
        };
        let is_function = dependency
            .node_kinds()
            .get(position)
            .is_some_and(|kind| kind.tag() == CheckedNodeTag::Function);
        if is_callee && is_function {
            let closure = signature_closure(supplied, position, budget, form)?;
            // The closure walk is charged at the term: one unit per node
            // visited, including each parameter node.
            at.charge(closure.work, budget, form)?;
            if closure.independent {
                return Ok(());
            }
        }
        Err(referrer.refuse(
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::OperatorIneligible,
            at,
            None,
            budget,
            form,
        ))
    }
}

/// One step from a term to the term it stands in.
#[derive(Clone, Copy)]
struct Step {
    parent: Option<usize>,
    segment: Segment,
}

#[derive(Clone, Copy)]
enum Segment {
    Body,
    Arguments(usize),
    Members(usize),
    Value,
}

/// Where a term stands in its node.
struct Place<'p> {
    steps: &'p [Step],
    step: usize,
    position: usize,
}

impl Place<'_> {
    /// Count the flat borrowed steps and pointer bytes without constructing
    /// either diagnostic allocation.
    fn layout(&self, member: Option<&str>) -> (usize, usize) {
        let mut count = 3_usize;
        let mut bytes = Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(self.position)
            .pointer_len();
        let mut cursor = Some(self.step);
        while let Some(at) = cursor {
            let Some(step) = self.steps.get(at) else {
                break;
            };
            let (parts, path) = match step.segment {
                Segment::Body => (1, Path::root().key("body")),
                Segment::Arguments(index) => (2, Path::root().key("arguments").index(index)),
                Segment::Members(index) => (2, Path::root().key("members").index(index)),
                Segment::Value => (1, Path::root().key("value")),
            };
            count = count.saturating_add(parts);
            bytes = bytes.saturating_add(path.pointer_len());
            cursor = step.parent;
        }
        if let Some(member) = member {
            count = count.saturating_add(1);
            bytes = bytes.saturating_add(Path::root().key(member).pointer_len());
        }
        (count, bytes)
    }

    /// One exact-capacity scratch block, already paid as part of the pending
    /// diagnostic. Reversing the leaf-first chain needs no second worklist.
    fn flat_steps<'a>(&self, member: Option<&'a str>, count: usize) -> Vec<PointerStep<'a>> {
        let mut steps = Vec::with_capacity(count);
        let mut cursor = Some(self.step);
        while let Some(at) = cursor {
            let Some(step) = self.steps.get(at) else {
                break;
            };
            match step.segment {
                Segment::Body => steps.push(PointerStep::Key("body")),
                Segment::Arguments(index) => {
                    steps.push(PointerStep::Index(index));
                    steps.push(PointerStep::Key("arguments"));
                }
                Segment::Members(index) => {
                    steps.push(PointerStep::Index(index));
                    steps.push(PointerStep::Key("members"));
                }
                Segment::Value => steps.push(PointerStep::Key("value")),
            }
            cursor = step.parent;
        }
        steps.extend([
            PointerStep::Index(self.position),
            PointerStep::Key("nodes"),
            PointerStep::Key("semantic_graph"),
        ]);
        steps.reverse();
        if let Some(member) = member {
            steps.push(PointerStep::Key(member));
        }
        debug_assert_eq!(steps.len(), count);
        steps
    }

    /// Keep the original native work charge and its first unpaid locus. A
    /// failed charge pays both actual diagnostic blocks before constructing
    /// its scratch steps and final pointer.
    fn charge(
        &self,
        work: u64,
        budget: &mut Budget<'_, '_>,
        form: &'static str,
    ) -> Result<(), ValidationFailure> {
        let Some(reader) = budget.reader_budget(form) else {
            return budget.charge_at(work, form, || self.pointer());
        };
        if reader.charge(NativeJoinChargeKind::Work, u128::from(work)) {
            return Ok(());
        }
        #[cfg(test)]
        super::work_probe::charge_denied();
        let (count, pointer) = self.layout(None);
        let bytes = count
            .saturating_mul(std::mem::size_of::<PointerStep<'_>>())
            .saturating_add(pointer);
        let path = if reader.prepay_diagnostic(bytes as u128) {
            let steps = self.flat_steps(None, count);
            Some(super::stage_storage::Position::from(Trail::Base(&steps)).build())
        } else {
            None
        };
        Err(ValidationFailure::Budget(super::BudgetStop {
            resource: super::PackageReadResource::Work,
            amount: u128::from(work),
            path,
        }))
    }

    /// The pointer of the term, built on demand.
    fn pointer(&self) -> JsonPointer {
        let mut chain = Vec::new();
        let mut cursor = Some(self.step);
        while let Some(at) = cursor {
            let Some(step) = self.steps.get(at) else {
                break;
            };
            chain.push(step.segment);
            cursor = step.parent;
        }
        chain
            .into_iter()
            .rev()
            .fold(node_pointer(self.position), |at, segment| match segment {
                Segment::Body => at.key("body"),
                Segment::Arguments(index) => at.key("arguments").index(index),
                Segment::Members(index) => at.key("members").index(index),
                Segment::Value => at.key("value"),
            })
    }
}

/// Queues `members` for the walk, last first, so they pop in order.
fn push_members<'t>(
    budget: &mut Budget<'_, '_>,
    steps: &mut Vec<Step>,
    pending: &mut Vec<(&'t CheckedTerm, usize, bool)>,
    parent: usize,
    members: &'t [CheckedTerm],
    form: &'static str,
) -> Result<(), ValidationFailure> {
    for (index, child) in members.iter().enumerate().rev() {
        let child_step = steps.len();
        budget.push(
            steps,
            Step {
                parent: Some(parent),
                segment: Segment::Members(index),
            },
            form,
        )?;
        budget.push(pending, (child, child_step, false), form)?;
    }
    Ok(())
}

/// The outcome of walking a function's signature closure and the work it
/// took.
struct Closure {
    independent: bool,
    work: u64,
}

/// Implements: FR-322. No node in the transitive closure of the function's signature
/// type nodes carries a `declaration` or a `ModelOwner`.
fn signature_closure(
    supplied: &SuppliedDependency<'_>,
    function: usize,
    budget: &mut Budget<'_, '_>,
    form: &'static str,
) -> Result<Closure, ValidationFailure> {
    let nodes = &supplied.package.resolved_graph().nodes;
    let kinds = supplied.package.node_kinds();
    let node_of = |id: &CheckedNodeId| {
        let position = supplied.position(id)?;
        Some((position, nodes.get(position)?, *kinds.get(position)?))
    };
    let Some(function) = nodes.get(function) else {
        return Ok(Closure {
            independent: false,
            work: 1,
        });
    };
    let mut work = 1_u64;
    let mut pending: Vec<&CheckedNodeId> = Vec::new();
    budget.push(&mut pending, &function.semantic_type, form)?;
    let members = match &function.body {
        CheckedTerm::Aggregate { members } => Some(members.as_slice()),
        _ => None,
    };
    let parameters = members.and_then(|members| match members.first()? {
        CheckedTerm::Binding { name, value }
            if name.internal_kind() == super::TermBindingName::Parameters =>
        {
            Some(value.as_ref())
        }
        _ => None,
    });
    if let Some(parameters) = parameters {
        let CheckedTerm::Aggregate { members: entries } = parameters else {
            return Ok(Closure {
                independent: false,
                work,
            });
        };
        for entry in entries {
            let Some((_, node, CheckedNodeKind::Value(ValueForm::Parameter))) = (match entry {
                CheckedTerm::Reference { target } => Some(target),
                _ => None,
            })
            .and_then(node_of) else {
                return Ok(Closure {
                    independent: false,
                    work,
                });
            };
            work = work.saturating_add(1);
            budget.push(&mut pending, &node.semantic_type, form)?;
        }
    }
    // Each node is visited once; the table holds a flag per graph position.
    let mut seen: Vec<bool> = budget.vec(nodes.len(), form)?;
    seen.resize(nodes.len(), false);
    while let Some(id) = pending.pop() {
        let Some((position, node, kind)) = node_of(id) else {
            continue;
        };
        if std::mem::replace(&mut seen[position], true) {
            continue;
        }
        work = work.saturating_add(1);
        if carries_declaration_or_owner(node, kind) {
            return Ok(Closure {
                independent: false,
                work,
            });
        }
        for dependency in &node.dependencies {
            budget.push(&mut pending, dependency, form)?;
        }
    }
    Ok(Closure {
        independent: true,
        work,
    })
}

/// Whether a node is package-dependent: it carries a `declaration`, or it is
/// keyed by a `ModelOwner` (a `model` or `relation` node, or a nominal
/// nominal declaration owned by a domain package).
fn carries_declaration_or_owner<Body>(
    node: &super::CheckedSemanticNodeV2<Body>,
    kind: CheckedNodeKind,
) -> bool {
    node.declaration.is_some()
        || matches!(kind.tag(), CheckedNodeTag::Model | CheckedNodeTag::Relation)
        || matches!(
            node.owner.as_ref(),
            Some(super::CheckedNodeOwner::Model { .. })
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
            | CheckedTerm::ImplementationGuarantee { .. }
            | CheckedTerm::ComponentComposition { .. }
            | CheckedTerm::Refinement { .. }
            | CheckedTerm::ExtensionForm { .. }
            | CheckedTerm::NominalBody(_) => {}
        }
    }
    selected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::v2::composite_operands::tests::package;
    use crate::checked_package::v2::{PackageReadStage, TermBindingName};
    use crate::{NativeJoinBudget, NativeJoinLocus, NativeJoinLocusSource};
    use ix_trace_rs::trace;

    /// What one run did under `allowance` bytes of read storage.
    struct Run {
        ok: bool,
        consumed: u64,
        allocated: u64,
        locus: Option<NativeJoinLocus>,
    }

    fn run(
        allowance: u64,
        body: impl FnOnce(&mut WorkMeter) -> Result<(), ValidationFailure>,
    ) -> Run {
        let mut budget =
            NativeJoinBudget::new(u64::MAX, u64::MAX).with_read_storage_limit(allowance);
        let mut ok = false;
        let measured = allocation_counter::measure(|| {
            let mut meter = WorkMeter::over(&mut budget);
            ok = body(&mut meter).is_ok();
        });
        Run {
            ok,
            consumed: budget.read_storage_consumed(),
            allocated: measured.bytes_total,
            locus: budget.first_unpaid_locus(),
        }
    }

    /// The bytes a refusal pointer takes to build.
    fn pointer_bytes(pointer: impl FnOnce() -> JsonPointer) -> u64 {
        allocation_counter::measure(|| drop(pointer())).bytes_total
    }

    /// Every byte the run allocates is a byte it requested; the exact
    /// allowance admits it; one byte short refuses at `locus`; and no
    /// shorter allowance allocates past what it was granted.
    fn assert_exact(
        locus: impl Fn(&NativeJoinLocus) -> bool,
        denial: u64,
        body: impl Fn(&mut WorkMeter) -> Result<(), ValidationFailure>,
    ) {
        let unlimited = run(u64::MAX, &body);
        assert!(unlimited.ok, "the run succeeds without a limit");
        assert!(unlimited.consumed > 0, "the run requests storage");
        assert_eq!(
            unlimited.allocated, unlimited.consumed,
            "every byte allocated is a byte requested"
        );
        let exact = run(unlimited.consumed, &body);
        assert!(exact.ok, "the exact allowance admits the run");
        assert_eq!(exact.consumed, unlimited.consumed);
        let short = run(unlimited.consumed - 1, &body);
        assert!(!short.ok, "one byte short refuses");
        assert!(
            short.locus.as_ref().is_some_and(locus),
            "refused at {:?}",
            short.locus
        );
        for allowance in (0..unlimited.consumed).step_by(7) {
            let refused = run(allowance, &body);
            assert!(!refused.ok, "allowance {allowance} is short");
            assert!(
                refused.allocated <= allowance + denial,
                "allowance {allowance}: allocated {}",
                refused.allocated
            );
        }
    }

    fn lock_locus(locus: &NativeJoinLocus) -> bool {
        locus.form == F
            && matches!(
                locus.source,
                NativeJoinLocusSource::PackageRead(PackageReadStage::Lock, _)
            )
    }

    fn selection(identity: &str) -> CheckedDependencySelection {
        CheckedDependencySelection {
            identity: identity.into(),
        }
    }

    /// The dependency-identity table and the admission list are requested
    /// before they are made.
    #[trace("TC-472", "FR-038-AC-502")]
    #[test]
    fn tc_472_dependency_admission_allocates_exactly_what_it_requests() {
        let dependency = package();
        let name = dependency.name().to_owned();
        let mut evidence = CheckedPackageEvidence::new();
        evidence
            .insert_dependency_package(name.as_str(), dependency)
            .expect("supplied");
        let selections = [selection(&name)];
        let denial = pointer_bytes(|| {
            JsonPointer::root()
                .key("lock")
                .key("dependency_selections")
                .index(0)
        });
        assert_exact(lock_locus, denial, |meter| {
            admit_dependencies(&selections, &evidence, meter).map(drop)
        });
    }

    /// The derived-closure worklists and the identity table they search are
    /// requested before they grow.
    #[trace("TC-472", "FR-038-AC-502")]
    #[test]
    fn tc_472_derived_closure_allocates_exactly_what_it_requests() {
        let dependency = package();
        let names = ["a", "b", "c", "d", "e", "f", "g", "h", "i"];
        let selections: Vec<_> = names.iter().map(|name| selection(name)).collect();
        let packages: Vec<&CheckedPackageV2> = names.iter().map(|_| &dependency).collect();
        let denial = pointer_bytes(|| {
            JsonPointer::root()
                .key("lock")
                .key("dependency_selections")
                .index(8)
        });
        assert_exact(lock_locus, denial, |meter| {
            check_derived_closure(&selections, &packages, Path::root(), meter)
        });
    }

    /// The same fast path retains the contract-package caller's nonempty root.
    #[ix_trace_rs::trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn derived_closure_work_denial_preserves_nonempty_root_and_first_charge() {
        use crate::PackageReadResource;
        let dependency = package();
        let selections = [selection(dependency.name())];
        let packages = [&dependency];
        let root = Path::root().key("package").key("source");
        let observe = |storage| {
            let mut budget = NativeJoinBudget::new(0, 0).with_read_storage_limit(storage);
            let mut result = None;
            let requested = allocation_counter::measure(|| {
                result = Some(check_derived_closure(
                    &selections,
                    &packages,
                    root,
                    &mut WorkMeter::over(&mut budget),
                ));
            })
            .bytes_total;
            assert_eq!(requested, budget.read_storage_consumed());
            (
                result
                    .expect("closure probe ran")
                    .expect_err("first Work is denied"),
                budget,
                requested,
            )
        };
        let (ample, budget, requested) = observe(u64::MAX);
        let ValidationFailure::Budget(stop) = &ample else {
            panic!("Work stop: {ample:?}");
        };
        assert_eq!((stop.resource, stop.amount), (PackageReadResource::Work, 1));
        assert_eq!(
            stop.path.as_ref().map(JsonPointer::as_str),
            Some("/package/source/lock/dependency_selections/0")
        );
        let first = budget.first_unpaid().expect("first Work");
        assert_eq!((first.kind, first.amount), (NativeJoinChargeKind::Work, 1));
        let locus = budget.first_unpaid_locus().expect("original lock locus");
        assert_eq!(
            locus.source,
            NativeJoinLocusSource::PackageRead(PackageReadStage::Lock, 0)
        );
        assert_eq!(locus.form, F);
        let (exact, exact_budget, exact_requested) = observe(requested);
        assert_eq!(exact, ample);
        assert_eq!(exact_requested, requested);
        assert_eq!(exact_budget.first_unpaid(), Some(first));
        assert_eq!(exact_budget.first_unpaid_locus(), Some(locus));
        assert_eq!(exact_budget.consumed(), budget.consumed());
        assert_eq!(
            exact_budget.read_storage_consumed(),
            budget.read_storage_consumed()
        );
        for storage in [0, requested - 1] {
            let (short, short_budget, short_requested) = observe(storage);
            let (repeated, repeated_budget, repeated_requested) = observe(storage);
            assert_eq!(short, repeated);
            assert_eq!(short_budget, repeated_budget);
            assert_eq!(short_requested, repeated_requested);
            assert_eq!(short_requested, 0);
            assert_eq!(short_budget.first_unpaid(), Some(first));
            assert_eq!(short_budget.first_unpaid_locus(), Some(locus));
            assert!(short_budget.first_unpaid_read_storage().is_none());
            let ValidationFailure::Budget(stop) = short else {
                panic!("original Work remains first");
            };
            assert_eq!((stop.resource, stop.amount), (PackageReadResource::Work, 1));
            assert!(stop.path.is_none());
        }
    }

    /// The signature closure's worklist and flags are requested at the
    /// referring node, whichever node's closure is walked.
    #[trace("TC-472", "FR-038-AC-502")]
    #[test]
    fn tc_472_signature_closure_allocates_exactly_what_it_requests() {
        let dependency = package();
        let mut evidence = CheckedPackageEvidence::new();
        evidence
            .insert_dependency_package(dependency.name(), package())
            .expect("supplied");
        let selections = [selection(dependency.name())];
        let nodes = dependency.resolved_graph().nodes.len();
        let mut walked = 0;
        for function in 0..nodes {
            let supplied = {
                let mut meter = WorkMeter::new(u64::MAX);
                admit_dependencies(&selections, &evidence, &mut meter).expect("admits")
            };
            let Some(entry) = supplied.packages.first() else {
                panic!("one package is supplied");
            };
            let denial = pointer_bytes(|| node_pointer(function).key("body"));
            let probe = run(u64::MAX, |meter| {
                let mut budget = Budget::node(meter, 3);
                signature_closure(entry, function, &mut budget, "dependency").map(drop)
            });
            assert_eq!(probe.allocated, probe.consumed, "node {function}");
            if probe.consumed == 0 {
                continue;
            }
            walked += 1;
            assert_exact(
                |locus| {
                    locus.form == "dependency"
                        && matches!(
                            locus.source,
                            NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, 3)
                        )
                },
                denial,
                |meter| {
                    let mut budget = Budget::node(meter, 3);
                    signature_closure(entry, function, &mut budget, "dependency").map(drop)
                },
            );
        }
        assert!(walked > 0, "some closure requests storage");
        let _ = TermBindingName::Parameters;
    }

    /// The term walk's step chain and worklist are requested before they grow.
    #[trace("TC-472", "FR-038-AC-502")]
    #[test]
    fn tc_472_the_term_walk_allocates_exactly_what_it_requests() {
        let dependency = package();
        let node = dependency.resolved_graph().nodes[0].to_raw();
        let target = dependency.resolved_graph().nodes[0].node_id.clone();
        let leaf = || CheckedTerm::Reference {
            target: target.clone(),
        };
        let term = CheckedTerm::Aggregate {
            members: (0..9)
                .map(|at| CheckedTerm::Aggregate {
                    members: (0..=at).map(|_| leaf()).collect(),
                })
                .collect(),
        };
        let supplied = SuppliedDependencies::default();
        let denial = pointer_bytes(|| node_pointer(5).key("body"));
        assert_exact(
            |locus| {
                matches!(
                    locus.source,
                    NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, 5)
                )
            },
            denial,
            |meter| {
                DependencyReferences::new(&supplied).walk_admitted(
                    Referrer {
                        node: &node,
                        position: 5,
                    },
                    &term,
                    meter,
                )
            },
        );
    }
}

#[cfg(test)]
mod allocation_tests {
    use super::{
        admit_dependencies, CheckedDependencySelection, CheckedPackageEvidence, WorkMeter,
    };
    use crate::checked_package::common::ValidationFailure;
    use crate::checked_package::shared::{
        CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
    };
    use crate::checked_package::v2::budgeted_read::{
        CheckedPackagePendingOutcome, PackageReadResource,
    };
    use crate::checked_package::v2::native_join_budget::{
        NativeJoinBudget, NativeJoinLocusSource, PackageReadStage,
    };

    #[ix_trace_rs::trace("TC-472", "FR-038-AC-511", "FR-038-AC-512")]
    #[test]
    fn missing_dependency_refusal_is_prepaid_at_exact_and_short_storage_bounds() {
        let selections = [CheckedDependencySelection {
            identity: "test/absent".into(),
        }];
        let evidence = CheckedPackageEvidence::new();
        let observe = |limit| {
            let mut budget =
                NativeJoinBudget::new(u64::MAX, u64::MAX).with_read_storage_limit(limit);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                result = Some(admit_dependencies(&selections, &evidence, &mut meter));
            });
            (
                result
                    .expect("the lock-stage read ran")
                    .expect_err("the first dependency is absent"),
                budget,
                measured.bytes_total,
            )
        };
        let (ample, budget, requested) = observe(u64::MAX);
        assert_eq!(
            requested,
            budget.read_storage_consumed(),
            "every refusal allocation is admitted before construction"
        );
        let ValidationFailure::Refused(refusal) = &ample else {
            panic!("the ample read retains its semantic refusal: {ample:?}");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingImport);
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::MissingSelection)
        );
        assert_eq!(
            refusal.loci[0].path.as_ref().map(JsonPointer::as_str),
            Some("/lock/dependency_selections/0")
        );
        let total = budget.read_storage_consumed();
        assert!(total > 0);
        let (exact, exact_budget, exact_requested) = observe(total);
        assert_eq!(
            exact, ample,
            "an exact allowance preserves the complete refusal"
        );
        assert_eq!(exact_requested, total);
        assert_eq!(exact_budget.read_storage_consumed(), total);
        let (short, mut short_budget, short_requested) = observe(total - 1);
        let (repeated, repeated_budget, repeated_requested) = observe(total - 1);
        assert_eq!(short, repeated, "the same refused reservation repeats");
        assert_eq!(short_budget, repeated_budget);
        assert_eq!(short_requested, repeated_requested);
        assert_eq!(
            short_requested,
            short_budget.read_storage_consumed(),
            "a denied diagnostic makes no unpaid request"
        );
        let ValidationFailure::Budget(stop) = short else {
            panic!("one byte short must deny the diagnostic: {short:?}");
        };
        assert_eq!(stop.resource, PackageReadResource::ReadStorage);
        assert!(
            stop.path.is_none(),
            "the denied diagnostic builds no pointer"
        );
        let locus = short_budget
            .first_unpaid_locus()
            .expect("the denied diagnostic has a locus");
        assert_eq!(
            (locus.source, locus.form),
            (
                NativeJoinLocusSource::PackageRead(PackageReadStage::Lock, 0),
                "diagnostic"
            )
        );
        assert_eq!(
            short_budget.take_pending(),
            Some(CheckedPackagePendingOutcome::Refused(
                CheckedPackageRefusalCode::MissingImport
            ))
        );
    }

    #[ix_trace_rs::trace("TC-472", "FR-038-AC-511", "FR-038-AC-512")]
    #[test]
    fn reference_refusal_prepays_both_diagnostic_blocks_and_preserves_caller_state() {
        use super::{DependencyReferences, Referrer, SuppliedDependencies};
        use crate::checked_package::v2::composite_operands::tests::package;
        use crate::checked_package::v2::native_join_budget::{
            NativeJoinChargeKind, NativeJoinLocus,
        };
        let package = package();
        let node = package.resolved_graph().nodes[0].to_raw();
        let term = super::CheckedTerm::DependencyReference {
            package: "test/unselected".into(),
            node: node.node_id.clone(),
        };
        let supplied = SuppliedDependencies::default();
        let observe = |limit| {
            let mut budget = NativeJoinBudget::new(100, 100).with_read_storage_limit(limit);
            let caller = NativeJoinLocus {
                source: NativeJoinLocusSource::Package,
                form: "caller",
            };
            budget
                .accounted_charge(NativeJoinChargeKind::Storage, 2, caller)
                .expect("prior logical storage");
            let before = budget.begin();
            budget
                .accounted_charge(NativeJoinChargeKind::Work, 3, caller)
                .expect("prior work");
            budget.finish_read(before, 0);
            let consumed = budget.consumed();
            let success = budget.last_success_delta();
            let read_success = budget.last_success_read_storage_delta();
            let mut result = None;
            let requested = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                result = Some(DependencyReferences::new(&supplied).walk_admitted(
                    Referrer {
                        node: &node,
                        position: 5,
                    },
                    &term,
                    &mut meter,
                ));
            })
            .bytes_total;
            assert_eq!(
                budget.consumed(),
                consumed,
                "the semantic refusal adds no work or logical storage"
            );
            assert_eq!(
                budget.last_success_delta(),
                success,
                "diagnostic reborrowing preserves the caller success marker"
            );
            assert_eq!(budget.last_success_read_storage_delta(), read_success);
            (
                result
                    .expect("reference walk ran")
                    .expect_err("the package was not selected"),
                budget,
                requested,
            )
        };
        let (ample, budget, requested) = observe(u64::MAX);
        assert_eq!(
            requested,
            budget.read_storage_consumed(),
            "scratch, pointer, node identity and refusal payload are all prepaid"
        );
        let ValidationFailure::Refused(refusal) = &ample else {
            panic!("semantic refusal: {ample:?}");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::MissingSelection)
        );
        assert_eq!(
            refusal.loci[0].path.as_ref().map(JsonPointer::as_str),
            Some("/semantic_graph/nodes/5/body/package")
        );
        assert_eq!(refusal.locus.as_ref(), Some(&node.node_id));
        let total = budget.read_storage_consumed();
        let (exact, exact_budget, exact_requested) = observe(total);
        assert_eq!(exact, ample);
        assert_eq!(exact_requested, total);
        assert_eq!(exact_budget.read_storage_consumed(), total);
        let (short, mut short_budget, short_requested) = observe(total - 1);
        let (repeat, repeat_budget, repeat_requested) = observe(total - 1);
        assert_eq!(short, repeat);
        assert_eq!(short_budget, repeat_budget);
        assert_eq!(short_requested, repeat_requested);
        assert_eq!(short_requested, short_budget.read_storage_consumed());
        assert!(
            matches!(short, ValidationFailure::Budget(_)),
            "a short payload cannot produce an unpaid refusal"
        );
        assert_eq!(
            short_budget.take_pending(),
            Some(CheckedPackagePendingOutcome::Refused(
                CheckedPackageRefusalCode::MissingDeclaration
            ))
        );
        let walk_blocks = std::mem::size_of::<super::Step>()
            + std::mem::size_of::<(&super::CheckedTerm, usize, bool)>();
        let (_, mut scratch_denied, scratch_requested) =
            observe(u64::try_from(walk_blocks).expect("bounded walk blocks"));
        assert_eq!(scratch_requested, scratch_denied.read_storage_consumed());
        assert_eq!(
            scratch_denied.take_pending(),
            Some(CheckedPackagePendingOutcome::Refused(
                CheckedPackageRefusalCode::MissingDeclaration
            )),
            "scratch denial retains the same pending semantic outcome"
        );
    }

    #[ix_trace_rs::trace("TC-472", "FR-038-AC-511", "FR-038-AC-512")]
    #[test]
    fn reference_work_denial_retains_the_first_charge_and_prepays_its_pointer() {
        use super::{Budget, Place, Segment, Step};
        use crate::checked_package::v2::native_join_budget::NativeJoinChargeKind;
        let steps = [
            Step {
                parent: None,
                segment: Segment::Body,
            },
            Step {
                parent: Some(0),
                segment: Segment::Arguments(2),
            },
            Step {
                parent: Some(1),
                segment: Segment::Value,
            },
        ];
        let at = Place {
            steps: &steps,
            step: 2,
            position: 5,
        };
        let observe = |limit| {
            let mut budget = NativeJoinBudget::new(0, u64::MAX).with_read_storage_limit(limit);
            let mut result = None;
            let requested = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                let mut source = Budget::node(&mut meter, 5);
                result = Some(at.charge(7, &mut source, "expression"));
            })
            .bytes_total;
            (
                result
                    .expect("work probe ran")
                    .expect_err("work is exhausted"),
                budget,
                requested,
            )
        };
        let (ample, budget, requested) = observe(u64::MAX);
        assert_eq!(requested, budget.read_storage_consumed());
        let first = budget.first_unpaid().expect("first work charge");
        assert_eq!((first.kind, first.amount), (NativeJoinChargeKind::Work, 7));
        let locus = budget.first_unpaid_locus().expect("work locus");
        assert_eq!(
            (locus.source, locus.form),
            (
                NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, 5),
                "expression"
            )
        );
        let ValidationFailure::Budget(stop) = &ample else {
            panic!("work stop: {ample:?}");
        };
        assert_eq!(stop.resource, PackageReadResource::Work);
        assert_eq!(stop.amount, 7);
        assert_eq!(
            stop.path.as_ref().map(JsonPointer::as_str),
            Some("/semantic_graph/nodes/5/body/arguments/2/value")
        );
        let (exact, exact_budget, exact_requested) = observe(requested);
        assert_eq!(exact, ample);
        assert_eq!(exact_requested, requested);
        assert_eq!(exact_budget.first_unpaid(), Some(first));
        let (short, short_budget, short_requested) = observe(requested - 1);
        let (repeat, repeat_budget, repeat_requested) = observe(requested - 1);
        assert_eq!(short, repeat);
        assert_eq!(short_budget, repeat_budget);
        assert_eq!(short_requested, repeat_requested);
        assert_eq!(short_requested, short_budget.read_storage_consumed());
        assert_eq!(short_budget.first_unpaid(), Some(first));
        assert_eq!(short_budget.first_unpaid_locus(), Some(locus));
        let ValidationFailure::Budget(stop) = short else {
            panic!("original work stop: {short:?}");
        };
        assert_eq!((stop.resource, stop.amount), (PackageReadResource::Work, 7));
        assert!(
            stop.path.is_none(),
            "a short diagnostic builds neither block"
        );
    }
}
