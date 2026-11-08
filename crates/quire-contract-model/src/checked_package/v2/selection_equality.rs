// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Complete selected JSON comparison without recursive ownership or traversal.
//!
//! This is the package `PartialEq`, not the read: nothing here runs while a
//! package is read.

use super::selected_document::json_number;
use quire_canonical::{Document, Node};
use std::{collections::BTreeMap, sync::Arc};

/// Compare semantic selected inputs independently of admission-derived caches.
pub(super) fn equal_selections(left: &[Arc<Document>], right: &[Arc<Document>]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| Arc::ptr_eq(left, right) || equal_documents(left, right))
}

/// Compare the admitted JSON views while borrowing the original flat arenas.
/// Native stack use does not depend on document depth.
#[ix_trace_rs::string_edge]
pub(super) fn equal_documents(left: &Document, right: &Document) -> bool {
    let mut pending = vec![(left.root(), right.root())];
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
