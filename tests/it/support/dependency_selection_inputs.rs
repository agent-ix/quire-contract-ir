// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Original dependency-selection fixture, shared without copying its recipe.

use super::checked_package::{refresh_temporal_bindings, v2_all_families};
use serde_json::{json, Value};

pub fn dependency_selection_package(name: &str) -> Value {
    fn rename_package(value: &mut Value, name: &str) {
        match value {
            Value::Array(items) => items.iter_mut().for_each(|item| rename_package(item, name)),
            Value::Object(members) => {
                if let Some(package) = members.get_mut("package") {
                    if package == "test/families" {
                        *package = json!(name);
                    }
                }
                members
                    .values_mut()
                    .for_each(|item| rename_package(item, name));
            }
            _ => {}
        }
    }
    let mut package = v2_all_families();
    let scalar = package["semantic_graph"]["nodes"][0].clone();
    let scalar_id = scalar["node_id"].clone();
    package["semantic_graph"]["nodes"] = json!([scalar]);
    package["source_map"] = json!(package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .filter(|entry| entry["node_id"] == scalar_id)
        .cloned()
        .collect::<Vec<_>>());
    rename_package(&mut package, name);
    package["name"] = json!(name);
    refresh_temporal_bindings(&mut package);
    package
}
