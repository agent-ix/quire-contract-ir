// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

/// Returns the root and every normal/build dependency by resolved Cargo package ID.
///
/// Rejects cycles and packages refused by the caller's ownership policy. Dev-only
/// edges are excluded; renamed and target-specific production edges are retained.
pub fn production_graph(
    metadata: &Value,
    root: &str,
    forbidden: impl Fn(&Value) -> bool,
) -> Result<BTreeSet<String>, String> {
    let packages: BTreeMap<_, _> = metadata["packages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|package| (package["id"].as_str().unwrap(), package))
        .collect();
    let nodes: BTreeMap<_, _> = metadata["resolve"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| (node["id"].as_str().unwrap(), node))
        .collect();
    let mut active = BTreeSet::new();
    let mut complete = BTreeSet::new();
    let mut pending = vec![(root, false)];
    while let Some((id, leaving)) = pending.pop() {
        if leaving {
            active.remove(id);
            complete.insert(id.to_owned());
            continue;
        }
        if complete.contains(id) {
            continue;
        }
        if !active.insert(id) {
            return Err(format!("production dependency cycle at {id}"));
        }
        if forbidden(packages[id]) {
            return Err(format!("forbidden production dependency {id}"));
        }
        pending.push((id, true));
        for edge in nodes[id]["deps"].as_array().unwrap() {
            let production = edge["dep_kinds"].as_array().unwrap().iter().any(|kind| {
                match kind["kind"].as_str() {
                    None | Some("build") => true,
                    Some("dev") => false,
                    other => panic!("unknown Cargo dependency kind {other:?}"),
                }
            });
            if production {
                pending.push((edge["pkg"].as_str().unwrap(), false));
            }
        }
    }
    Ok(complete)
}

