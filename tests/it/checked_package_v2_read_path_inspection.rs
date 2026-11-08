// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Source inspection of the V2 read path (TC-472).
//!
//! FR-038-AC-513 bans, on the path of `CheckedPackageV2::read`, any
//! `serde_json` parser, deserializer or `RawValue`, any `serde_json::Value`,
//! `num-bigint` parse or arithmetic, any stable sort, and any `BTreeMap`,
//! `BTreeSet`, `HashMap` or `serde_json::Map`. The criterion allows no
//! exception, so there is no allowlist: every non-comment line of the read
//! path's non-test code is scanned.

use ix_trace_rs::trace;
use std::path::Path;

/// The read path's source files under `checked_package/v2`: the staged readers
/// of the stage table (wire decode, wire checks, node index, lock, model
/// documents, graph validators, source map, extension, diagnostics, retained
/// structures) and the budgeted entry and its charging helpers.
const READ_PATH: &[&str] = &[
    "budgeted_read.rs",
    "read_charge.rs",
    "mod.rs",
    "intake.rs",
    "lossless.rs",
    "flat_wire.rs",
    "typed_terms.rs",
    "node_index.rs",
    "identity.rs",
    "closure_index.rs",
    "operation_catalog.rs",
    "model_members.rs",
    "model_fields.rs",
    "model_root.rs",
    "selected_document.rs",
    "dependency_references.rs",
    "model_population_catalog.rs",
    "structural.rs",
    "operations.rs",
    "temporal.rs",
    "timed_model.rs",
    "protocol_body.rs",
    "protocol_locations.rs",
    "natural.rs",
    "refinement.rs",
    "state.rs",
    "step_relation.rs",
    "extension.rs",
    "operation_effects.rs",
    "checked_term_data.rs",
];

/// The banned spellings of FR-038-AC-513.
const BANNED: &[&str] = &[
    "serde_json::from_",
    "serde_json::Value",
    "serde_json::Map",
    "serde_json::value",
    "serde_json::json!",
    "serde_json::Deserializer",
    "serde_json::StreamDeserializer",
    "json!(",
    "RawValue",
    "num_bigint",
    "BigInt",
    "BigUint",
    ".sort(",
    ".sort_by(",
    ".sort_by_key(",
    ".sort_by_cached_key(",
    "BTreeMap",
    "BTreeSet",
    "HashMap",
];

fn banned_lines(file: &str, source: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed == "#[cfg(test)]" {
            break;
        }
        if trimmed.starts_with("//") {
            continue;
        }
        if let Some(word) = BANNED.iter().find(|word| line.contains(**word)) {
            found.push(format!("{file}:{}: {word}: {}", index + 1, trimmed));
        }
    }
    found
}

/// The V2 read path contains no banned parser, tree, big-integer, stable sort
/// or ordered/hashed container.
#[trace("TC-472", "FR-038-AC-513")]
#[test]
fn the_read_path_uses_no_banned_parser_container_or_sort() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("crates/quire-contract-model/src/checked_package/v2");
    let mut found = Vec::new();
    for file in READ_PATH {
        let source = std::fs::read_to_string(root.join(file))
            .unwrap_or_else(|error| panic!("read-path file {file}: {error}"));
        found.extend(banned_lines(file, &source));
    }
    assert!(
        found.is_empty(),
        "{} banned occurrences on the read path:\n{}",
        found.len(),
        found.join("\n")
    );
}
