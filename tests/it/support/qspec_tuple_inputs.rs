// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Evidence and wire patch adapters shared with the original tuple driver.

use crate::support::checked_package as fixture_support;
use quire_contract_model::CheckedPackageEvidence;
use serde_json::Value;

pub(crate) fn selected_evidence(document: &Value) -> CheckedPackageEvidence {
    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document(
            document["package"]["identity"]
                .as_str()
                .expect("document identity"),
            fixture_support::canonical(document),
        )
        .expect("selected document evidence");
    evidence.support_feature("quire.value.complete/v1");
    evidence
}

pub(crate) fn apply_wire_patch(package: &mut Value, patch: &Value) {
    // Published patches address the wire: operand rows and operand slots.
    fixture_support::wire_operands(package);
    let path = patch["path"].as_str().expect("published patch path");
    let (parent, member) = path.rsplit_once('/').expect("patch member");
    let target = package.pointer_mut(parent).expect("published patch parent");
    let operation = patch["op"].as_str().expect("published patch operation");
    match (operation, target) {
        ("add", Value::Array(items)) if member == "-" => items.push(patch["value"].clone()),
        ("replace", Value::Object(map)) => {
            *map.get_mut(member).expect("published replacement target") = patch["value"].clone();
        }
        ("replace", Value::Array(items)) => {
            let index = member.parse::<usize>().expect("published array index");
            *items.get_mut(index).expect("published replacement target") = patch["value"].clone();
        }
        ("add", Value::Object(map)) => {
            assert!(map
                .insert(member.to_owned(), patch["value"].clone())
                .is_none());
        }
        ("remove", Value::Object(map)) => {
            assert!(map.remove(member).is_some(), "published removal target");
        }
        ("remove", Value::Array(items)) => {
            let index = member.parse::<usize>().expect("published array index");
            assert!(index < items.len(), "published removal target");
            items.remove(index);
        }
        _ => panic!("unsupported published patch {operation} at {path}"),
    }
}
