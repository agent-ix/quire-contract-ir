// SPDX-License-Identifier: AGPL-3.0-or-later
//! The semantic-term validator over a [`CheckedJson`] node, charged.
//!
//! Implements: FR-038 "Budget-charged strict read" (FR-038-AC-506, AC-513),
//! FR-322.
//!
//! The same closed shapes as `common::validate_term`, read through the
//! reader's node API instead of a `serde_json::Value`, so a term is validated
//! in place with no copy of it. The walk is `quire-walk`'s `walk_reserved`:
//! every growth of its two lists is charged before it is made, and the list of
//! positions this validator keeps for the entered node, the node id built for
//! each reference it reports, the position it remembers for the caller and the
//! pointer of each refusal are charged the same way, all at the calling check's
//! stage and index with the form `"walk"`. An integer is read from its text
//! with `str::parse`, which requests no heap.

use super::stage_storage::{self, Position, Refusal, Site};
use super::{ApplicationOperator, BodyTerm, LiteralKind, WorkMeter};
use crate::checked_package::common::{
    ReferenceMember, ReferenceSite, Step, Trail, ValidationFailure,
};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedNodePathSegment, CheckedPackageRefusalCode,
};
use quire_canonical::{Items, Node, NodeRef};
use quire_walk::{walk_reserved, Children, Denied, Reservation, Walk};
use std::cell::RefCell;
use std::mem::size_of;
use std::ops::ControlFlow;

/// The locus form of every request the term walk makes.
const FORM: &str = "walk";

/// The largest integer an I-JSON wire carries exactly.
const I_JSON_BOUND: u64 = 9_007_199_254_740_991;

/// What a term walk found.
pub(super) struct Checked<'s> {
    /// The work the validated terms are charged.
    pub(super) work: u64,
    /// The position of the first reference `visit` asked to remember.
    pub(super) remembered: Option<Vec<Step<'s>>>,
}

/// Validates the semantic term `term` at `at`, charging the heap it requests
/// to `meter` at `site`'s stage and index, and reports each reference target to
/// `visit` with the site that carried it. `visit` answers whether the position
/// of the target is to be remembered; the first position it asks for is
/// returned in [`Checked::remembered`]. `is_body_root` is as for
/// `common::validate_term`: only the term handed to this call can be a node
/// body's own top-level term.
///
/// # Errors
///
/// The first refusal, with its pointer charged before it is built, or the
/// budget's denial of a request.
pub(super) fn validate_term_json<'s>(
    term: NodeRef<'s>,
    is_body_root: bool,
    at: &Trail<'s>,
    meter: &mut WorkMeter<'_>,
    site: &Site<'_>,
    visit: &mut dyn FnMut(&CheckedNodeId, ReferenceSite) -> bool,
) -> Result<Checked<'s>, ValidationFailure> {
    let shared = Shared {
        meter: RefCell::new(meter),
        failure: RefCell::new(None),
    };
    let depth = depth(at);
    let mut walker = TermWalk {
        shared: &shared,
        stage: site.stage,
        index: site.index,
        steps: Vec::new(),
        visit,
        work: 0,
        remembered: None,
    };
    walker.charge_at(
        Position::Trail(*at),
        depth.saturating_mul(size_of::<Step<'_>>()),
    )?;
    walker.steps = Vec::with_capacity(depth);
    collect(at, &mut walker.steps);
    let mut charges = Charges {
        shared: &shared,
        stage: site.stage,
        index: site.index,
        at: *at,
    };
    let root = Item {
        node: term,
        keep: depth,
        key: None,
        index: None,
        is_body_root,
    };
    match walk_reserved(&mut walker, root, &mut charges) {
        Ok(ControlFlow::Continue(())) => Ok(Checked {
            work: walker.work,
            remembered: walker.remembered,
        }),
        Ok(ControlFlow::Break(failure)) => Err(failure),
        Err(refusal) => {
            let requested = match refusal {
                quire_walk::Refusal::Denied { requested }
                | quire_walk::Refusal::Allocation { requested } => requested,
            };
            Err(shared
                .failure
                .take()
                .unwrap_or_else(|| stage_storage::unavailable(requested)))
        }
    }
}

/// The steps from the root to `trail`.
fn depth(trail: &Trail<'_>) -> usize {
    let mut total = 0_usize;
    let mut current = trail;
    loop {
        match current {
            Trail::Base(steps) => return total.saturating_add(steps.len()),
            Trail::Child(parent, _) => {
                total = total.saturating_add(1);
                current = parent;
            }
        }
    }
}

/// Appends the steps from the root to `trail`, in order, to `into`, which
/// has room for [`depth`] of them.
fn collect<'s>(trail: &Trail<'s>, into: &mut Vec<Step<'s>>) {
    let mut current = trail;
    let base = loop {
        match current {
            Trail::Base(steps) => break *steps,
            Trail::Child(parent, _) => current = parent,
        }
    };
    into.extend_from_slice(base);
    let below = into.len();
    current = trail;
    while let Trail::Child(parent, step) = current {
        into.push(*step);
        current = parent;
    }
    if let Some(tail) = into.get_mut(below..) {
        tail.reverse();
    }
}

/// What the walk and its reservation both charge through.
struct Shared<'m, 'b> {
    meter: RefCell<&'m mut WorkMeter<'b>>,
    /// The failure of the request the reservation refused.
    failure: RefCell<Option<ValidationFailure>>,
}

/// `quire-walk`'s reservation: the walk's own list growths, charged at the
/// term's position.
struct Charges<'a, 'm, 'b, 's> {
    shared: &'a Shared<'m, 'b>,
    stage: super::PackageReadStage,
    index: usize,
    at: Trail<'s>,
}

impl Reservation for Charges<'_, '_, '_, '_> {
    fn reserve(&mut self, bytes: usize) -> Result<(), Denied> {
        let site = Site::new(self.stage, self.index, FORM, Position::Trail(self.at));
        let mut meter = self.shared.meter.borrow_mut();
        match stage_storage::reserve(&mut meter, &site, bytes) {
            Ok(()) => Ok(()),
            Err(failure) => {
                *self.shared.failure.borrow_mut() = Some(failure);
                Err(Denied)
            }
        }
    }
}

/// A term the walk will enter and where it sits below its parent.
struct Item<'s> {
    node: NodeRef<'s>,
    /// How many steps the walk holds at the parent.
    keep: usize,
    key: Option<&'static str>,
    index: Option<usize>,
    is_body_root: bool,
}

struct TermWalk<'a, 'm, 'b, 's, 'v> {
    shared: &'a Shared<'m, 'b>,
    stage: super::PackageReadStage,
    index: usize,
    /// The position of the entered term, as the steps from the document root.
    steps: Vec<Step<'s>>,
    visit: &'v mut dyn FnMut(&CheckedNodeId, ReferenceSite) -> bool,
    work: u64,
    remembered: Option<Vec<Step<'s>>>,
}

impl<'s> TermWalk<'_, '_, '_, 's, '_> {
    /// Admits `bytes` of heap at `position` before it is requested.
    fn charge_at(&self, position: Position<'_>, bytes: usize) -> Result<(), ValidationFailure> {
        let site = Site::new(self.stage, self.index, FORM, position);
        let mut meter = self.shared.meter.borrow_mut();
        stage_storage::reserve(&mut meter, &site, bytes)
    }

    /// Admits `bytes` of heap at the entered term.
    fn charge(&self, bytes: usize) -> Result<(), ValidationFailure> {
        self.charge_at(Position::Trail(Trail::Base(&self.steps)), bytes)
    }

    /// The refusal `code` at the entered term, or at its member `member`, its
    /// owned diagnostic charged before it is built.
    fn reject(&self, code: CheckedPackageRefusalCode, member: Option<&str>) -> ValidationFailure {
        let base = Trail::Base(&self.steps);
        let child;
        let position = match member {
            Some(key) => {
                child = base.key(key);
                child
            }
            None => base,
        };
        let site = Site::new(self.stage, self.index, FORM, Position::Trail(position));
        let mut meter = self.shared.meter.borrow_mut();
        stage_storage::refuse(&mut meter, &site, Refusal::new(code, position))
    }

    fn invalid(&self, member: Option<&str>) -> ValidationFailure {
        self.reject(CheckedPackageRefusalCode::InvalidSemanticGraph, member)
    }

    /// Moves to `item`, cutting the steps back to its parent's and adding its
    /// own, the list growing in charged exact blocks.
    fn move_to(&mut self, item: &Item<'s>) -> Result<(), ValidationFailure> {
        self.steps.truncate(item.keep);
        let needed = item
            .keep
            .saturating_add(usize::from(item.key.is_some()))
            .saturating_add(usize::from(item.index.is_some()));
        let capacity = self.steps.capacity();
        if needed > capacity {
            let target = needed.max(capacity.saturating_mul(2)).max(4);
            let bytes = target.saturating_mul(size_of::<Step<'_>>());
            self.charge(bytes)?;
            if self
                .steps
                .try_reserve_exact(target.saturating_sub(self.steps.len()))
                .is_err()
            {
                return Err(stage_storage::unavailable(bytes));
            }
        }
        self.steps.extend(item.key.map(Step::Key));
        self.steps.extend(item.index.map(Step::Index));
        Ok(())
    }

    /// Keeps the entered term's position, and `member` below it, for the
    /// caller.
    fn remember(&mut self, member: &'static str) -> Result<(), ValidationFailure> {
        let length = self.steps.len().saturating_add(1);
        self.charge(length.saturating_mul(size_of::<Step<'_>>()))?;
        let mut copy = Vec::with_capacity(length);
        copy.extend_from_slice(&self.steps);
        copy.push(Step::Key(member));
        self.remembered = Some(copy);
        Ok(())
    }

    /// Reports the node id held in `object[key]`.
    fn visit_member(
        &mut self,
        object: NodeRef<'s>,
        key: &'static str,
        member: ReferenceMember,
        is_body_root: bool,
    ) -> Result<u64, ValidationFailure> {
        let Some(value) = object.get(key) else {
            return Err(self.invalid(None));
        };
        let spec = match IdSpec::read(value) {
            Ok(spec) => spec,
            Err(IdFault::Shape) => return Err(self.invalid(Some(key))),
            Err(IdFault::Bounds) => {
                return Err(self.reject(CheckedPackageRefusalCode::MalformedWire, Some(key)))
            }
        };
        self.charge(spec.bytes())?;
        let id = spec.build();
        let site = ReferenceSite {
            member,
            is_body_root,
        };
        if (self.visit)(&id, site) && self.remembered.is_none() {
            self.remember(key)?;
        }
        Ok(1)
    }

    /// Names the elements of `elements`, below the entered term's `key`.
    fn push_children(
        &self,
        children: &mut Children<'_, Item<'s>>,
        elements: Items<'s>,
        key: &'static str,
    ) {
        for (index, node) in elements.enumerate() {
            let item = Item {
                node,
                keep: self.steps.len(),
                key: Some(key),
                index: Some(index),
                is_body_root: false,
            };
            if children.try_push(item).is_err() {
                break;
            }
        }
    }

    /// Validates the one term `term` without its subterms, as
    /// `common::validate_one` does, and returns the work it is charged.
    #[ix_trace_rs::string_edge]
    fn check(
        &mut self,
        term: NodeRef<'s>,
        is_body_root: bool,
        children: &mut Children<'_, Item<'s>>,
    ) -> Result<u64, ValidationFailure> {
        let Node::Object(members) = term.node() else {
            return Err(self.invalid(None));
        };
        let count = members.len();
        let Some(tag) = term.get("term") else {
            return Err(self.invalid(None));
        };
        let Node::String(tag) = tag.node() else {
            return Err(self.invalid(Some("term")));
        };
        let kind = if tag == "extension_form" {
            None
        } else {
            BodyTerm::from_wire(tag)
        };
        let Some(kind) = kind else {
            return Err(self.invalid(None));
        };
        let exact = |expected: &[&str]| {
            count == expected.len() && expected.iter().all(|member| term.get(member).is_some())
        };
        match kind {
            BodyTerm::Literal => {
                if exact(&["term", "type", "value_kind", "value"])
                    && string_of(term, "value_kind")
                        .is_some_and(|kind| LiteralKind::from_wire(kind).is_some())
                    && term.get("value").is_some_and(is_literal_value)
                {
                    self.visit_member(term, "type", ReferenceMember::Type, is_body_root)
                } else {
                    Err(self.invalid(None))
                }
            }
            BodyTerm::Reference => {
                if exact(&["term", "target"]) {
                    self.visit_member(term, "target", ReferenceMember::Target, is_body_root)
                } else {
                    Err(self.invalid(None))
                }
            }
            BodyTerm::Application => {
                if !(exact(&["term", "operator", "operation", "result_type", "arguments"])
                    && string_of(term, "operator")
                        .is_some_and(|operator| ApplicationOperator::from_wire(operator).is_some()))
                {
                    return Err(self.invalid(None));
                }
                let result_type = self.visit_member(
                    term,
                    "result_type",
                    ReferenceMember::ResultType,
                    is_body_root,
                )?;
                let Some(Node::Array(arguments)) = term.get("arguments").map(|node| node.node())
                else {
                    return Err(self.invalid(Some("arguments")));
                };
                self.push_children(children, arguments, "arguments");
                Ok(result_type.saturating_add(1))
            }
            BodyTerm::Aggregate => {
                if !exact(&["term", "members"]) {
                    return Err(self.invalid(None));
                }
                let Some(Node::Array(elements)) = term.get("members").map(|node| node.node())
                else {
                    return Err(self.invalid(Some("members")));
                };
                self.push_children(children, elements, "members");
                Ok(1)
            }
            BodyTerm::Binding => {
                if !(exact(&["term", "name", "value"])
                    && string_of(term, "name").is_some_and(|name| !name.is_empty()))
                {
                    return Err(self.invalid(None));
                }
                if let Some(value) = term.get("value") {
                    let item = Item {
                        node: value,
                        keep: self.steps.len(),
                        key: Some("value"),
                        index: None,
                        is_body_root: false,
                    };
                    // A refusal is recorded by `children` and ends the walk.
                    let _ = children.try_push(item);
                }
                Ok(1)
            }
            // `{term, package, node}`: the target names a node of a dependency
            // package, so it is never a reference of this graph.
            BodyTerm::DependencyReference => {
                if !exact(&["term", "package", "node"]) {
                    return Err(self.invalid(None));
                }
                if !string_of(term, "package").is_some_and(|package| !package.is_empty()) {
                    return Err(self.invalid(Some("package")));
                }
                if !term
                    .get("node")
                    .is_some_and(|node| IdSpec::read(node).is_ok())
                {
                    return Err(self.invalid(Some("node")));
                }
                Ok(1)
            }
            BodyTerm::Frame | BodyTerm::AbstractionRelation => Err(self.invalid(None)),
        }
    }
}

impl<'s> Walk for TermWalk<'_, '_, '_, 's, '_> {
    type Node = Item<'s>;
    type Frame = ();
    type Stop = ValidationFailure;

    fn enter(
        &mut self,
        item: Item<'s>,
        children: &mut Children<'_, Item<'s>>,
    ) -> ControlFlow<ValidationFailure> {
        if let Err(failure) = self.move_to(&item) {
            return ControlFlow::Break(failure);
        }
        match self.check(item.node, item.is_body_root, children) {
            Ok(work) => {
                self.work = self.work.saturating_add(work);
                ControlFlow::Continue(())
            }
            Err(failure) => ControlFlow::Break(failure),
        }
    }

    fn exit(&mut self, (): ()) -> ControlFlow<ValidationFailure> {
        ControlFlow::Continue(())
    }
}

/// The string member `key` of `object`.
fn string_of<'d>(object: NodeRef<'d>, key: &str) -> Option<&'d str> {
    match object.get(key)?.node() {
        Node::String(text) => Some(text),
        _ => None,
    }
}

/// Whether `value` can be a literal's value: a boolean, string or null, or a
/// number whose text is an integer within the signed or unsigned 64-bit range.
fn is_literal_value(value: NodeRef<'_>) -> bool {
    match value.node() {
        Node::Bool(_) | Node::String(_) | Node::Null => true,
        Node::Number(number) => {
            let text = number.text();
            text.parse::<i64>().is_ok() || text.parse::<u64>().is_ok()
        }
        Node::Array(_) | Node::Object(_) => false,
    }
}

/// The unsigned integer a number's text spells.
fn unsigned(value: NodeRef<'_>) -> Option<u64> {
    match value.node() {
        Node::Number(number) => number.text().parse().ok(),
        _ => None,
    }
}

/// Why a value is not a node id.
enum IdFault {
    /// It is no flat node id object: the deserializer refuses it.
    Shape,
    /// It is one, outside the V2 wire's bounds.
    Bounds,
}

/// One segment of a wire node id's path.
enum Segment<'d> {
    Name(&'d str),
    Index(u64),
    Operand(u64),
}

#[ix_trace_rs::string_edge]
fn segment(node: NodeRef<'_>) -> Option<Segment<'_>> {
    match node.node() {
        Node::String(name) => Some(Segment::Name(name)),
        Node::Object(mut members) => {
            let (key, value) = members.next()?;
            if members.next().is_some() {
                return None;
            }
            let slot = unsigned(value).filter(|slot| *slot <= I_JSON_BOUND)?;
            match key {
                "node" => Some(Segment::Index(slot)),
                "operand" => Some(Segment::Operand(slot)),
                _ => None,
            }
        }
        _ => None,
    }
}

/// A wire node id read in place, before any of it is copied.
struct IdSpec<'d> {
    package: &'d str,
    path: Items<'d>,
    ordinal: u64,
    /// The bytes of the names in the path.
    names: usize,
}

impl<'d> IdSpec<'d> {
    fn read(value: NodeRef<'d>) -> Result<Self, IdFault> {
        let Node::Object(members) = value.node() else {
            return Err(IdFault::Shape);
        };
        if members.len() != 3 {
            return Err(IdFault::Shape);
        }
        let (Some(package), Some(path), Some(ordinal)) = (
            value.get("package"),
            value.get("path"),
            value.get("ordinal"),
        ) else {
            return Err(IdFault::Shape);
        };
        let Node::String(package) = package.node() else {
            return Err(IdFault::Shape);
        };
        let Node::Array(path) = path.node() else {
            return Err(IdFault::Shape);
        };
        let ordinal = unsigned(ordinal).ok_or(IdFault::Shape)?;
        let mut names = 0_usize;
        for item in path.clone() {
            if let Segment::Name(name) = segment(item).ok_or(IdFault::Shape)? {
                names = names.saturating_add(name.len());
            }
        }
        if package.is_empty() || path.len() == 0 || ordinal > I_JSON_BOUND {
            return Err(IdFault::Bounds);
        }
        Ok(Self {
            package,
            path,
            ordinal,
            names,
        })
    }

    /// The heap the id requests: its package name, its path and each name in
    /// it, which is what `stage_storage::node_id_clone_bytes` counts.
    fn bytes(&self) -> usize {
        self.package
            .len()
            .saturating_add(
                self.path
                    .len()
                    .saturating_mul(size_of::<CheckedNodePathSegment>()),
            )
            .saturating_add(self.names)
    }

    /// The id, in blocks of exactly the sizes [`Self::bytes`] counts.
    fn build(self) -> CheckedNodeId {
        let mut path = Vec::with_capacity(self.path.len());
        for item in self.path {
            if let Some(segment) = segment(item) {
                path.push(match segment {
                    Segment::Name(name) => CheckedNodePathSegment::Name(name.into()),
                    Segment::Index(slot) => CheckedNodePathSegment::Index(slot),
                    Segment::Operand(slot) => CheckedNodePathSegment::Operand(slot),
                });
            }
        }
        CheckedNodeId {
            package: self.package.into(),
            path,
            ordinal: self.ordinal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::common::{validate_term, TermGrammar};
    use crate::checked_package::v2::checked_json::CheckedJson;
    use crate::checked_package::v2::native_join_budget::NativeJoinBudget;
    use crate::checked_package::v2::PackageReadStage;
    use ix_trace_rs::trace;
    use serde_json::{json, Value};

    fn id(name: &str) -> Value {
        json!({"package": "test", "path": [name, {"operand": 1}], "ordinal": 0})
    }

    fn literal(value: Value) -> Value {
        json!({"term": "literal", "type": id("integer"), "value_kind": "integer", "value": value})
    }

    fn application(arguments: Vec<Value>) -> Value {
        json!({"term": "application", "operator": "call", "operation": {"x": 1},
            "result_type": id("result"), "arguments": arguments})
    }

    fn terms() -> Vec<Value> {
        let reference = json!({"term": "reference", "target": id("target")});
        let mut bad_bound = id("a");
        bad_bound["path"] = json!([{"node": 9_007_199_254_740_992_u64}]);
        let mut empty_package = id("a");
        empty_package["package"] = json!("");
        let mut empty_path = id("a");
        empty_path["path"] = json!([]);
        let mut extra = id("a");
        extra["more"] = json!(1);
        vec![
            literal(json!(7)),
            literal(json!(-7)),
            literal(json!(1.5)),
            literal(json!([])),
            literal(Value::Null),
            reference.clone(),
            json!({"term": "aggregate", "members": [reference.clone(), literal(json!(true))]}),
            json!({"term": "binding", "name": "n", "value": reference.clone()}),
            json!({"term": "binding", "name": "", "value": reference.clone()}),
            application(vec![
                reference.clone(),
                json!({"term": "aggregate", "members": [literal(json!("s")), application(vec![])]}),
            ]),
            application(vec![
                reference.clone(),
                json!({"term": "aggregate", "members": 3}),
            ]),
            json!({"term": "application", "operator": "nope", "operation": 1,
                "result_type": id("r"), "arguments": []}),
            json!({"term": "application", "operator": "call", "operation": 1,
                "result_type": id("r"), "arguments": "x"}),
            json!({"term": "dependency_reference", "package": "dep", "node": id("n")}),
            json!({"term": "dependency_reference", "package": "", "node": id("n")}),
            json!({"term": "dependency_reference", "package": "dep", "node": empty_path.clone()}),
            json!({"term": "dependency_reference", "package": "dep"}),
            json!({"term": "reference", "target": "text"}),
            json!({"term": "reference", "target": bad_bound}),
            json!({"term": "reference", "target": empty_package}),
            json!({"term": "reference", "target": empty_path}),
            json!({"term": "reference", "target": extra}),
            json!({"term": "reference"}),
            json!({"term": "reference", "target": id("t"), "more": 1}),
            json!({"term": "literal", "type": id("t"), "value_kind": "other", "value": 1}),
            json!({"term": "frame"}),
            json!({"term": "extension_form"}),
            json!({"term": "unknown"}),
            json!({"term": 4}),
            json!({"no_term": 4}),
            json!("text"),
            json!([]),
        ]
    }

    fn describe(outcome: &Result<u64, ValidationFailure>) -> String {
        format!("{outcome:?}")
    }

    /// The charged validator over a document node answers every term as the
    /// `Value` validator does: the same work or refusal, the same references
    /// reported in the same order and the same first reference position.
    #[trace("TC-472", "FR-038-AC-506")]
    #[test]
    fn the_node_validator_agrees_with_the_value_validator() {
        for (case, term) in terms().into_iter().enumerate() {
            for is_body_root in [true, false] {
                let mut expected_targets = Vec::new();
                let mut expected_first = None;
                let expected = validate_term(
                    &term,
                    TermGrammar::V2,
                    is_body_root,
                    &Trail::Base(&[Step::Key("body")]),
                    &mut |target, site, at| {
                        expected_targets.push((target.clone(), site));
                        if expected_first.is_none() {
                            expected_first = Some(at.pointer());
                        }
                    },
                );
                let json = CheckedJson::from_value(&term).expect("canonical term");
                let mut meter = WorkMeter::new(u64::MAX);
                let site = Site::new(
                    PackageReadStage::Diagnostics,
                    0,
                    "diagnostics.entries",
                    stage_storage::Path::root(),
                );
                let mut targets = Vec::new();
                let base = [Step::Key("body")];
                let outcome = validate_term_json(
                    json.node(),
                    is_body_root,
                    &Trail::Base(&base),
                    &mut meter,
                    &site,
                    &mut |target, site| {
                        targets.push((target.clone(), site));
                        true
                    },
                );
                let first = outcome
                    .as_ref()
                    .ok()
                    .and_then(|checked| checked.remembered.as_ref())
                    .map(|steps| {
                        crate::checked_package::common::pointer_from_steps(steps.iter().copied())
                    });
                let actual = outcome.map(|checked| checked.work);
                assert_eq!(
                    describe(&actual),
                    describe(&expected),
                    "case {case}: {term}"
                );
                assert_eq!(targets, expected_targets, "case {case}: {term}");
                if actual.is_ok() {
                    assert_eq!(first, expected_first, "case {case}: {term}");
                }
            }
        }
    }

    /// A term of 20000 nested aggregates is validated on a 256 KiB stack, and
    /// the heap it requests is the heap it charged.
    #[trace("TC-472", "FR-038-AC-506")]
    #[test]
    fn a_deep_term_is_validated_on_a_small_stack_and_charged_exactly() {
        const LEVELS: usize = 20_000;
        let (required, work) = std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                let mut text = String::new();
                for _ in 0..LEVELS {
                    text.push_str("{\"members\":[");
                }
                text.push_str(&literal(json!(1)).to_string());
                for _ in 0..LEVELS {
                    text.push_str("],\"term\":\"aggregate\"}");
                }
                let document = quire_canonical::read(text.as_bytes(), u64::MAX).expect("read");
                let node = document.root().id();
                let json = CheckedJson::new(std::sync::Arc::new(document), node);
                let mut budget = NativeJoinBudget::unlimited();
                let mut work = None;
                let measured = allocation_counter::measure(|| {
                    let mut meter = WorkMeter::over(&mut budget);
                    let site = Site::new(
                        PackageReadStage::Diagnostics,
                        0,
                        "diagnostics.entries",
                        stage_storage::Path::root(),
                    );
                    work = Some(
                        validate_term_json(
                            json.node(),
                            true,
                            &Trail::Base(&[]),
                            &mut meter,
                            &site,
                            &mut |_, _| false,
                        )
                        .map(|checked| checked.work),
                    );
                });
                assert_eq!(
                    measured.bytes_total,
                    budget.read_storage_consumed() as u64,
                    "bytes allocated equal bytes charged"
                );
                (budget.read_storage_consumed(), work.expect("ran"))
            })
            .expect("spawns")
            .join()
            .expect("the walk ran to completion");
        assert!(required > 0);
        assert_eq!(work, Ok(LEVELS as u64 + 1));
    }

    /// Every limit below the measured requirement denies, allocating no more
    /// than the limit admits; the requirement itself admits.
    #[trace("TC-472", "FR-038-AC-506")]
    #[test]
    fn every_limit_below_the_requirement_denies_within_the_limit() {
        let term = application(vec![
            json!({"term": "reference", "target": id("a")}),
            json!({"term": "reference", "target": id("b")}),
            json!({"term": "aggregate", "members": [literal(json!(1))]}),
        ]);
        let json = CheckedJson::from_value(&term).expect("canonical term");
        let run = |limit: u64| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut outcome = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                let site = Site::new(
                    PackageReadStage::Diagnostics,
                    0,
                    "diagnostics.entries",
                    stage_storage::Path::root(),
                );
                outcome = Some(
                    validate_term_json(
                        json.node(),
                        true,
                        &Trail::Base(&[Step::Key("body")]),
                        &mut meter,
                        &site,
                        &mut |_, _| true,
                    )
                    .map(|checked| checked.work),
                );
            });
            (
                outcome.expect("ran"),
                measured.bytes_total,
                budget.read_storage_consumed(),
            )
        };
        let (outcome, allocated, required) = run(u64::MAX);
        assert!(outcome.is_ok());
        assert_eq!(
            allocated, required as u64,
            "bytes allocated equal bytes charged"
        );
        let (outcome, _, consumed) = run(required as u64);
        assert!(outcome.is_ok());
        assert_eq!(consumed, required);
        for limit in 0..required as u64 {
            let (outcome, allocated, _) = run(limit);
            assert!(
                matches!(outcome, Err(ValidationFailure::Budget(_))),
                "limit {limit}: {outcome:?}"
            );
            assert!(allocated <= limit, "limit {limit}: allocated {allocated}");
        }
    }
}
