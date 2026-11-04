// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Complete selected JSON values, retained without recursive ownership.

use quire_canonical::{Document, Node, Number};
use std::{collections::BTreeMap, sync::Arc};

/// One already-read selected document. Flat ownership and explicit comparison
/// keep native stack use independent of document depth during clone,
/// comparison and final-owner drop.
#[derive(Clone, Debug)]
pub(super) struct SelectedDocument(Arc<Document>);

impl SelectedDocument {
    pub(super) fn new(document: Document) -> Self {
        Self(Arc::new(document))
    }
}

impl PartialEq for SelectedDocument {
    #[ix_trace_rs::string_edge]
    fn eq(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.0, &other.0) {
            return true;
        }
        let mut pending = vec![(self.0.root(), other.0.root())];
        while let Some((left, right)) = pending.pop() {
            match (left.node(), right.node()) {
                (Node::Null, Node::Null) => {}
                (Node::Bool(left), Node::Bool(right)) if left == right => {}
                (Node::String(left), Node::String(right)) if left == right => {}
                (Node::Number(left), Node::Number(right))
                    if json_number(left) == json_number(right) => {}
                (Node::Array(left), Node::Array(right)) => {
                    if left.len() != right.len() {
                        return false;
                    }
                    pending.extend(left.zip(right));
                }
                (Node::Object(left), Node::Object(right)) => {
                    if left.len() != right.len() {
                        return false;
                    }
                    let right = right.collect::<BTreeMap<_, _>>();
                    for (name, value) in left {
                        let Some(other) = right.get(name) else {
                            return false;
                        };
                        pending.push((value, *other));
                    }
                }
                _ => return false,
            }
        }
        true
    }
}

impl Eq for SelectedDocument {}

/// The numeric value used by the selected document's existing JSON view.
pub(super) fn json_number(number: Number<'_>) -> Option<serde_json::Number> {
    number
        .text()
        .parse::<i64>()
        .map(serde_json::Number::from)
        .or_else(|_| number.text().parse::<u64>().map(serde_json::Number::from))
        .ok()
        .or_else(|| serde_json::Number::from_f64(number.value()))
}
