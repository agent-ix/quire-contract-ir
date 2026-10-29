// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Model-root structural keys against QSpec's authoritative published vector.

use super::{code_word, derived_with, fixtures_dir, proposal_dir_of, read_json, read_package};
use ix_trace_rs::trace;
use quire_contract_model::{
    derive_structural_candidates, CheckedPackageV2ReadResult, CheckedSemanticNodeV2,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn apply_vector_patch(package: &mut Value, patch: &Value) {
    let path = patch["path"].as_str().expect("published patch path");
    let (parent, member) = path.rsplit_once('/').expect("patch member");
    let target = package.pointer_mut(parent).expect("published patch parent");
    match (patch["op"].as_str().expect("published operation"), target) {
        ("add", Value::Array(items)) if member == "-" => items.push(patch["value"].clone()),
        ("add", Value::Object(map)) => {
            map.insert(member.to_owned(), patch["value"].clone());
        }
        ("replace", Value::Object(map)) => {
            *map.get_mut(member).expect("replaced member") = patch["value"].clone();
        }
        ("remove", Value::Array(items)) => {
            items.remove(member.parse::<usize>().expect("array index"));
        }
        ("remove", Value::Object(map)) => {
            map.remove(member).expect("removed member");
        }
        (operation, _) => panic!("unsupported published patch {operation}: {path}"),
    }
}
