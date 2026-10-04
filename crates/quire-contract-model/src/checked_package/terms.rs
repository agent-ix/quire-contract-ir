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

use super::common::{body_term, Step, Trail};
use super::v2::BodyTerm;
use quire_walk::{walk, Children, Walk};
use serde_json::Value;
use std::marker::PhantomData;
use std::ops::ControlFlow;

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
    let (key, listed, valued) = match body_term(term) {
        Some(BodyTerm::Application) => ("arguments", array(term, "arguments"), None),
        Some(BodyTerm::Aggregate) => ("members", array(term, "members"), None),
        Some(BodyTerm::Binding) => ("value", &[][..], term.get("value")),
        Some(
            BodyTerm::Literal
            | BodyTerm::Reference
            | BodyTerm::DependencyReference
            | BodyTerm::Frame
            | BodyTerm::AbstractionRelation,
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

/// The array held in `term[key]`; empty when absent or not an array.
fn array<'a>(term: &'a Value, key: &str) -> &'a [Value] {
    term.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

/// A node a walk will enter: the value, the steps that lead to it from its
/// parent, and the walk's own `extra` for it. The steps are resolved by
/// [`Cursor::enter`] once the walk reaches the node, so no node owns a path.
#[derive(Debug)]
pub(super) struct At<'a, X> {
    pub(super) value: &'a Value,
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
    pub(super) fn at(base: &Trail<'a>) -> Self {
        Self {
            steps: base.steps(),
        }
    }

    /// The walk's root node, `value` at the cursor's base, with `extra`.
    pub(super) fn root<X>(&self, value: &'a Value, extra: X) -> At<'a, X> {
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
    pub(super) fn child<X>(
        &self,
        value: &'a Value,
        key: Option<&'a str>,
        index: Option<usize>,
        extra: X,
    ) -> At<'a, X> {
        At {
            value,
            keep: self.steps.len(),
            key,
            index,
            extra,
        }
    }

    /// The node `subterm` of the entered node, with `extra`.
    pub(super) fn subterm<X>(&self, subterm: Subterm<'a>, extra: X) -> At<'a, X> {
        self.child(subterm.value, Some(subterm.key), subterm.index, extra)
    }

    /// Moves the cursor to `node`, the next node the walk enters.
    pub(super) fn enter<X>(&mut self, node: &At<'a, X>) {
        self.steps.truncate(node.keep);
        self.steps.extend(node.key.map(Step::Key));
        self.steps.extend(node.index.map(Step::Index));
    }

    /// The entered node's position.
    pub(super) fn trail(&self) -> Trail<'_> {
        Trail::Base(&self.steps)
    }
}

/// Visits every term of `root` in document pre-order, outermost first, until
/// `visit` returns [`ControlFlow::Break`].
pub(super) fn visit_terms<'a, B>(
    root: &'a Value,
    visit: impl FnMut(&'a Value) -> ControlFlow<B>,
) -> ControlFlow<B> {
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
    ///
    /// Tracing: TC-048, FR-038-AC-117
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
    ///
    /// Tracing: TC-048, FR-038-AC-115
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
    ///
    /// Tracing: TC-048, FR-038-AC-115
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
    ///
    /// Tracing: TC-048, FR-038-AC-117
    #[trace("TC-048", "FR-038-AC-117")]
    #[test]
    fn tc_048_a_deep_term_is_visited_on_a_small_stack() {
        const DEPTH: usize = 200_000;
        let visited = std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                // Built by moving each level into the next: `json!` would copy
                // the term below it, recursively.
                let mut term = leaf("bottom");
                for _ in 0..DEPTH {
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
        assert_eq!(visited, DEPTH + 1);
    }
}
