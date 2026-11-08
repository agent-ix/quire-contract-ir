//! The shared iterative walk over the terms of a body, on `quire-walk`.
//!
//! Every walk of the V2 reader over a term (the flat-wire check, the term
//! validator, the abstraction scan, the reference and dependency collectors)
//! enters the term in document pre-order on `quire-walk`'s explicit heap stack
//! and never recurses on the call stack, so no refusal and no admission depends
//! on how deep a term nests (FR-038-AC-117). This module holds what those
//! walks share: the subterms of a term ([`subterms`]), the position of the
//! entered node ([`Cursor`], [`At`]), and the plain pre-order visit
//! ([`visit_terms`]).

use super::common::{Step, TermField, TermTag, Trail, ValidationFailure, WireTerm};
use super::shared::CheckedNodeId;
use super::v2::read_charge::{Denied, ReadStorage, Site};
use super::v2::{ApplicationOperator, BodyTerm, NodeIdRead, TermNode};
#[cfg(test)]
use super::v2::{CheckedTerm, CheckedTermView};
use crate::checked_package::shared::CheckedOperandEntry;
use serde::Deserialize;
use serde_json::Value;

/// One term below another and where it sits: the member `key` of the parent
/// (`arguments`, `members` or `value`) and, for an array member, the `index`
/// within it.
#[derive(Clone, Copy, Debug)]
pub(super) struct Subterm<'a> {
    pub(super) value: &'a Value,
    pub(super) key: &'static str,
    pub(super) index: Option<usize>,
}

/// The terms directly below `term`, in document order: an application's
/// `arguments`, an aggregate's `members` and a binding's `value`. Any other
/// value, and a term whose member is absent or of the wrong kind, has none.
pub(super) fn subterms(term: &Value) -> impl Iterator<Item = Subterm<'_>> {
    subterms_of(WireTerm::decode(term))
}

/// Children of a term whose discriminator has already been decoded at this
/// walk entry. A malformed member still leaves the other staged refusals in
/// their original order.
pub(super) fn subterms_of(wire: WireTerm<'_>) -> impl Iterator<Item = Subterm<'_>> {
    let (key, listed, valued) = match wire.kind() {
        Some(BodyTerm::Application) => {
            let arguments = wire.array(TermField::Arguments).unwrap_or(&[]);
            ("arguments", arguments, None)
        }
        Some(BodyTerm::Aggregate) => {
            let members = wire.array(TermField::Members).unwrap_or(&[]);
            ("members", members, None)
        }
        Some(BodyTerm::Binding) => {
            let value = wire.field(TermField::Value);
            ("value", &[][..], value)
        }
        Some(
            BodyTerm::Literal
            | BodyTerm::Reference
            | BodyTerm::DependencyReference
            | BodyTerm::Frame
            | BodyTerm::AbstractionRelation
            | BodyTerm::ImplementationGuarantee
            | BodyTerm::ComponentComposition,
        )
        | None => ("", &[][..], None),
    };
    listed
        .iter()
        .enumerate()
        .map(move |(index, value)| Subterm {
            value,
            key,
            index: Some(index),
        })
        .chain(valued.into_iter().map(|value| Subterm {
            value,
            key,
            index: None,
        }))
}

/// What the flat-grammar walks read of a term: implemented by a
/// `serde_json::Value` and by a [`TermNode`] of the retained document, so one
/// walk serves a validator that still holds a `Value` and one that reads the
/// node views.
pub(super) trait TermSource<'a>: Copy {
    /// The same kind of source over a shorter borrow of the same value, for a
    /// check that stands at a position built on the caller's stack.
    type Shorter<'b>: TermSource<'b>
    where
        'a: 'b;

    /// This source over a shorter borrow.
    fn shorter<'b>(self) -> Self::Shorter<'b>
    where
        'a: 'b;

    /// The `term` tag when it names a body term.
    fn kind(self) -> Option<BodyTerm>;
    /// An application's decoded `operator` class.
    fn operator(self) -> Option<ApplicationOperator>;
    /// Whether the `members` array is present and empty.
    fn has_empty_members(self) -> bool;
    /// The terms directly below this one (see [`subterms`]).
    fn subterms(self) -> impl Iterator<Item = (Self, &'static str, Option<usize>)>;
    /// Every value directly below this one: an object's members by name and an
    /// array's elements by index.
    fn children(self) -> impl Iterator<Item = (Self, Option<&'a str>, Option<usize>)>;
    /// The member `name` of an object.
    fn member(self, name: &str) -> Option<Self>;
    /// The elements of an array.
    fn items(self) -> Option<impl Iterator<Item = Self>>;
    /// The text of a string.
    fn text(self) -> Option<&'a str>;
    /// Whether this is an object.
    fn is_object(self) -> bool;
    /// The `term` discriminator, decoded.
    fn tag(self) -> TermTag;
    /// Whether this is an object with exactly the members `expected`.
    fn exact_members(self, expected: &[&str]) -> bool;
    /// Whether this is a literal payload the V2 grammar admits: a boolean, a
    /// string, `null`, or an integer token within the 64-bit range.
    fn literal_value_ok(self) -> bool;
    /// Reads this value as a node ID. A `Value` reads through the uncharged
    /// bridge, ignoring `sink`; a [`TermNode`] charges `sink` and reads its
    /// operand slots through `row`.
    fn node_id(
        self,
        sink: &mut dyn ReadStorage,
        site: Site,
        row: &[CheckedOperandEntry],
    ) -> Result<NodeIdRead, Denied>;
    /// Borrow a legacy materialized value when this source is one.
    fn value(self) -> Option<&'a Value>;
}

impl<'a> TermSource<'a> for &'a Value {
    type Shorter<'b>
        = &'b Value
    where
        'a: 'b;

    fn shorter<'b>(self) -> &'b Value
    where
        'a: 'b,
    {
        self
    }

    fn kind(self) -> Option<BodyTerm> {
        WireTerm::decode(self).kind()
    }

    fn operator(self) -> Option<ApplicationOperator> {
        WireTerm::decode(self).operator()
    }

    fn has_empty_members(self) -> bool {
        WireTerm::decode(self)
            .array(TermField::Members)
            .is_some_and(<[Value]>::is_empty)
    }

    fn subterms(self) -> impl Iterator<Item = (Self, &'static str, Option<usize>)> {
        subterms(self).map(|subterm| (subterm.value, subterm.key, subterm.index))
    }

    fn children(self) -> impl Iterator<Item = (Self, Option<&'a str>, Option<usize>)> {
        let (members, items) = match self {
            Value::Object(members) => (Some(members.iter()), None),
            Value::Array(items) => (None, Some(items.iter())),
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => (None, None),
        };
        members
            .into_iter()
            .flatten()
            .map(|(key, member)| (member, Some(key.as_str()), None))
            .chain(
                items
                    .into_iter()
                    .flatten()
                    .enumerate()
                    .map(|(index, item)| (item, None, Some(index))),
            )
    }

    fn member(self, name: &str) -> Option<Self> {
        self.get(name)
    }

    fn items(self) -> Option<impl Iterator<Item = Self>> {
        self.as_array().map(|items| items.iter())
    }

    fn text(self) -> Option<&'a str> {
        self.as_str()
    }

    fn is_object(self) -> bool {
        self.is_object()
    }

    fn tag(self) -> TermTag {
        WireTerm::decode(self).tag()
    }

    fn exact_members(self, expected: &[&str]) -> bool {
        self.as_object()
            .is_some_and(|object| super::common::exact_members(object, expected))
    }

    fn literal_value_ok(self) -> bool {
        match self {
            Value::Bool(_) | Value::String(_) | Value::Null => true,
            Value::Number(number) => number.is_i64() || number.is_u64(),
            Value::Array(_) | Value::Object(_) => false,
        }
    }

    fn node_id(
        self,
        _sink: &mut dyn ReadStorage,
        _site: Site,
        _row: &[CheckedOperandEntry],
    ) -> Result<NodeIdRead, Denied> {
        Ok(match CheckedNodeId::deserialize(self) {
            Ok(id) if id.is_well_formed() => NodeIdRead::Id(id),
            Ok(id) => NodeIdRead::Invalid(id),
            Err(_) => NodeIdRead::Malformed,
        })
    }

    fn value(self) -> Option<&'a Value> {
        Some(self)
    }
}

impl<'a> TermSource<'a> for TermNode<'a> {
    type Shorter<'b>
        = TermNode<'b>
    where
        'a: 'b;

    fn shorter<'b>(self) -> TermNode<'b>
    where
        'a: 'b,
    {
        self
    }

    fn kind(self) -> Option<BodyTerm> {
        TermNode::kind(self)
    }

    fn operator(self) -> Option<ApplicationOperator> {
        TermNode::operator(self)
    }

    fn has_empty_members(self) -> bool {
        self.array(TermField::Members)
            .is_some_and(|items| items.len() == 0)
    }

    fn subterms(self) -> impl Iterator<Item = (Self, &'static str, Option<usize>)> {
        TermNode::subterms(self).map(|subterm| (subterm.value, subterm.key, subterm.index))
    }

    fn children(self) -> impl Iterator<Item = (Self, Option<&'a str>, Option<usize>)> {
        let (members, items) = (self.members(), self.items());
        members
            .into_iter()
            .flatten()
            .map(|(key, member)| (member, Some(key), None))
            .chain(
                items
                    .into_iter()
                    .flatten()
                    .enumerate()
                    .map(|(index, item)| (item, None, Some(index))),
            )
    }

    fn member(self, name: &str) -> Option<Self> {
        self.get(name)
    }

    fn items(self) -> Option<impl Iterator<Item = Self>> {
        TermNode::items(self)
    }

    fn text(self) -> Option<&'a str> {
        self.as_str()
    }

    fn is_object(self) -> bool {
        TermNode::is_object(self)
    }

    fn tag(self) -> TermTag {
        TermNode::tag(self)
    }

    fn exact_members(self, expected: &[&str]) -> bool {
        TermNode::exact_members(self, expected)
    }

    fn literal_value_ok(self) -> bool {
        TermNode::literal_value_ok(self)
    }

    fn node_id(
        self,
        sink: &mut dyn ReadStorage,
        site: Site,
        row: &[CheckedOperandEntry],
    ) -> Result<NodeIdRead, Denied> {
        self.node_id_in(sink, site, row)
    }

    fn value(self) -> Option<&'a Value> {
        None
    }
}

/// A node a walk will enter: the value, the steps that lead to it from its
/// parent, and the walk's own `extra` for it. The steps are resolved by
/// [`Cursor::enter`] once the walk reaches the node, so no node owns a path.
#[derive(Debug)]
pub(super) struct At<'a, X, V = &'a Value> {
    pub(super) value: V,
    /// How many steps the cursor holds at the node's parent.
    keep: usize,
    key: Option<&'a str>,
    index: Option<usize>,
    pub(super) extra: X,
}

/// The position of the node a walk has entered, as the steps from the
/// document root. A walk enters its nodes depth first, so one vector of steps
/// serves every node: entering a node cuts the vector back to its parent's
/// length and adds the node's own steps. A pointer or a [`Trail`] is built
/// only when an outcome needs one.
pub(super) struct Cursor<'a> {
    steps: Vec<Step<'a>>,
}

impl<'a> Cursor<'a> {
    /// A cursor at `base`, the position of the walk's root.
    #[cfg(test)]
    pub(super) fn at(base: &Trail<'a>) -> Self {
        Self {
            steps: base.steps(),
        }
    }

    /// A cursor at `base` whose every allocation, now and when a deeper node
    /// is entered, is first charged through `charge` as a count of bytes.
    pub(super) fn charged(
        base: &Trail<'a>,
        charge: &dyn Fn(usize) -> Result<(), ValidationFailure>,
    ) -> Result<Self, ValidationFailure> {
        charge(
            base.step_count()
                .saturating_mul(std::mem::size_of::<Step<'a>>()),
        )?;
        Ok(Self {
            steps: base.steps(),
        })
    }

    /// The walk's root node, `value` at the cursor's base, with `extra`.
    pub(super) fn root<X, V>(&self, value: V, extra: X) -> At<'a, X, V> {
        At {
            value,
            keep: self.steps.len(),
            key: None,
            index: None,
            extra,
        }
    }

    /// The node `value` one level below the entered node, at member `key`
    /// and/or array `index`, with `extra`.
    pub(super) fn child<X, V>(
        &self,
        value: V,
        key: Option<&'a str>,
        index: Option<usize>,
        extra: X,
    ) -> At<'a, X, V> {
        At {
            value,
            keep: self.steps.len(),
            key,
            index,
            extra,
        }
    }

    /// The node `subterm` of the entered node, with `extra`.
    #[cfg(test)]
    pub(super) fn subterm<X>(&self, subterm: Subterm<'a>, extra: X) -> At<'a, X> {
        self.child(subterm.value, Some(subterm.key), subterm.index, extra)
    }

    /// The node `subterm` of a document-backed term, with `extra`.
    pub(super) fn term_subterm<X>(
        &self,
        value: crate::checked_package::v2::TermNode<'a>,
        key: &'static str,
        index: Option<usize>,
        extra: X,
    ) -> At<'a, X, crate::checked_package::v2::TermNode<'a>> {
        self.child(value, Some(key), index, extra)
    }

    /// Moves the cursor to `node`, the next node the walk enters.
    pub(super) fn enter<X, V>(&mut self, node: &At<'a, X, V>) {
        self.steps.truncate(node.keep);
        self.steps.extend(node.key.map(Step::Key));
        self.steps.extend(node.index.map(Step::Index));
    }

    /// [`Self::enter`], first charging and reserving the steps the node adds
    /// beyond the cursor's capacity (doubling from four, reserved exactly).
    pub(super) fn enter_charged<X, V>(
        &mut self,
        node: &At<'a, X, V>,
        charge: &dyn Fn(usize) -> Result<(), ValidationFailure>,
    ) -> Result<(), ValidationFailure> {
        self.steps.truncate(node.keep);
        let needed =
            node.keep + usize::from(node.key.is_some()) + usize::from(node.index.is_some());
        let capacity = self.steps.capacity();
        if needed > capacity {
            let target = needed.max(capacity.saturating_mul(2)).max(4);
            charge(target.saturating_mul(std::mem::size_of::<Step<'a>>()))?;
            self.steps.reserve_exact(target - self.steps.len());
        }
        self.enter(node);
        Ok(())
    }

    /// The entered node's position.
    pub(super) fn trail(&self) -> Trail<'_> {
        Trail::Base(&self.steps)
    }
}

#[cfg(test)] // the strict read walks `TermNode` subterms; kept as the Value oracle
/// Visits every term of `root` in document pre-order, outermost first, until
/// `visit` returns [`std::ops::ControlFlow::Break`].
pub(super) fn visit_terms<'a, B>(
    root: &'a Value,
    visit: impl FnMut(&'a Value) -> std::ops::ControlFlow<B>,
) -> std::ops::ControlFlow<B> {
    use quire_walk::{walk, Children, Walk};
    use std::marker::PhantomData;
    use std::ops::ControlFlow;

    struct Visit<'a, B, F>(F, PhantomData<fn(&'a Value) -> B>);

    impl<'a, B, F: FnMut(&'a Value) -> ControlFlow<B>> Walk for Visit<'a, B, F> {
        type Node = &'a Value;
        type Frame = ();
        type Stop = B;

        fn enter(
            &mut self,
            node: &'a Value,
            children: &mut Children<'_, &'a Value>,
        ) -> ControlFlow<B, ()> {
            (self.0)(node)?;
            children.extend(subterms(node).map(|subterm| subterm.value));
            ControlFlow::Continue(())
        }

        fn exit(&mut self, (): ()) -> ControlFlow<B> {
            ControlFlow::Continue(())
        }
    }

    walk(&mut Visit(visit, PhantomData), root)
}

#[cfg(test)]
mod tests {
    use super::{subterms, visit_terms, Cursor};
    use crate::checked_package::common::Trail;
    use ix_trace_rs::trace;
    use serde_json::{json, Value};
    use std::ops::ControlFlow;

    fn leaf(name: &str) -> Value {
        json!({"term": "literal", "label": name})
    }

    /// An application, an aggregate and a binding hold their subterms under
    /// `arguments`, `members` and `value`, in document order, with the array
    /// index of an element and none for a binding's one value; a leaf and a
    /// term whose member is absent or of the wrong kind hold none.
    #[trace("TC-048", "FR-038-AC-117")]
    #[test]
    fn tc_048_a_terms_subterms_are_its_arguments_members_or_value_in_order() {
        let application = json!({"term": "application", "arguments": [leaf("a"), leaf("b")]});
        let places = |term: &Value| {
            subterms(term)
                .map(|subterm| (subterm.key, subterm.index, subterm.value["label"].clone()))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            places(&application),
            [
                ("arguments", Some(0), json!("a")),
                ("arguments", Some(1), json!("b"))
            ]
        );
        let aggregate = json!({"term": "aggregate", "members": [leaf("m")]});
        assert_eq!(places(&aggregate), [("members", Some(0), json!("m"))]);
        let binding = json!({"term": "binding", "name": "n", "value": leaf("v")});
        assert_eq!(places(&binding), [("value", None, json!("v"))]);
        assert_eq!(places(&leaf("x")), []);
        assert_eq!(places(&json!({"term": "aggregate", "members": 3})), []);
        assert_eq!(places(&json!(7)), []);
    }

    /// The pre-order visit enters a term before its subterms and each subterm's
    /// whole subtree before the next sibling, and stops at the first `Break`,
    /// entering nothing after it.
    #[trace("TC-048", "FR-038-AC-115")]
    #[test]
    fn tc_048_the_visit_is_pre_order_and_stops_at_the_first_break() {
        let tree = json!({"term": "application", "arguments": [
            {"term": "aggregate", "members": [leaf("a"), leaf("b")]},
            {"term": "binding", "name": "n", "value": leaf("c")},
            leaf("d"),
        ]});
        let mut seen = Vec::new();
        let stopped = visit_terms(&tree, |term| {
            seen.push(
                term["label"]
                    .as_str()
                    .unwrap_or(term["term"].as_str().unwrap_or("?")),
            );
            ControlFlow::<()>::Continue(())
        });
        assert_eq!(stopped, ControlFlow::Continue(()));
        assert_eq!(
            seen,
            ["application", "aggregate", "a", "b", "binding", "c", "d"]
        );
        let mut entered = 0;
        let stopped = visit_terms(&tree, |term| {
            entered += 1;
            if term["label"] == "b" {
                ControlFlow::Break("b")
            } else {
                ControlFlow::Continue(())
            }
        });
        assert_eq!(stopped, ControlFlow::Break("b"));
        assert_eq!(entered, 4, "application, aggregate, a and b, and no more");
    }

    /// A cursor holds the steps from the walk's root to the entered node: a
    /// child adds its member and index to its parent's, and a sibling entered
    /// after a subtree cuts that subtree's steps back.
    #[trace("TC-048", "FR-038-AC-115")]
    #[test]
    fn tc_048_the_cursor_follows_the_walk_in_and_out_of_a_subtree() {
        let tree = json!({"term": "application", "arguments": [
            {"term": "aggregate", "members": [leaf("a")]},
            leaf("b"),
        ]});
        let mut cursor = Cursor::at(&Trail::Base(&[]));
        let root = cursor.root(&tree, ());
        cursor.enter(&root);
        assert_eq!(cursor.trail().pointer().as_str(), "");
        let mut children = subterms(&tree).map(|subterm| cursor.subterm(subterm, ()));
        let (first, second) = (
            children.next().expect("first"),
            children.next().expect("second"),
        );
        drop(children);
        cursor.enter(&first);
        assert_eq!(cursor.trail().pointer().as_str(), "/arguments/0");
        let inner = subterms(first.value)
            .map(|subterm| cursor.subterm(subterm, ()))
            .next()
            .expect("inner");
        cursor.enter(&inner);
        assert_eq!(cursor.trail().pointer().as_str(), "/arguments/0/members/0");
        cursor.enter(&second);
        assert_eq!(cursor.trail().pointer().as_str(), "/arguments/1");
    }

    /// The walk's native stack use does not follow the depth of the term: a
    /// term of 200000 nested aggregates is visited whole on a thread whose
    /// stack is 256 KiB.
    #[trace("TC-048", "FR-038-AC-117")]
    #[test]
    fn tc_048_a_deep_term_is_visited_on_a_small_stack() {
        const LEVELS: usize = 200_000;
        let visited = std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                // Built by moving each level into the next: `json!` would copy
                // the term below it, recursively.
                let mut term = leaf("bottom");
                for _ in 0..LEVELS {
                    let mut aggregate = serde_json::Map::new();
                    aggregate.insert("term".to_owned(), Value::String("aggregate".to_owned()));
                    aggregate.insert("members".to_owned(), Value::Array(vec![term]));
                    term = Value::Object(aggregate);
                }
                let mut visited = 0_usize;
                let stopped = visit_terms(&term, |_| {
                    visited += 1;
                    ControlFlow::<()>::Continue(())
                });
                assert_eq!(stopped, ControlFlow::Continue(()));
                // `Value`'s own drop recurses, so the term is taken apart
                // by `quire-canonical`'s iterative one.
                quire_canonical::drop_value(term);
                visited
            })
            .expect("spawns")
            .join()
            .expect("the visit ran to completion");
        assert_eq!(visited, LEVELS + 1);
    }
}
