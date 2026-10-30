// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Direct production reads of QSpec's corrected recursion-group fixtures.

use super::{fixtures_dir, proposal_dir_of, read_positive, DOMAIN_DOCUMENT};
use ix_trace_rs::trace;
use serde_json::Value;

/// QSpec FR-322-AC-274: a group member's structural path names its declaration;
/// its producer-assigned node ordinal is distinct from the recursion position.
#[trace("TC-048", "FR-038-AC-107", "FR-038-AC-176")]
#[test]
fn corrected_qspec_grouped_fixtures_admit_with_declared_paths() {
    let fixtures = fixtures_dir();
    let document = proposal_dir_of(&fixtures).join(DOMAIN_DOCUMENT);
    for name in [
        "positive-integer-literal-references.json",
        "positive-two-owners-a.json",
        "positive-two-owners-b.json",
        "positive-recursive-records.json",
    ] {
        let (wire, admitted) =
            read_positive(&fixtures, name, &document).unwrap_or_else(|why| panic!("{why}"));
        let nodes = wire["semantic_graph"]["nodes"]
            .as_array()
            .expect("published graph nodes");
        assert_eq!(admitted.graph().nodes.len(), nodes.len(), "{name}");
        let mut grouped = 0;
        for (index, published) in nodes.iter().enumerate() {
            if published.get("recursion_group").is_none_or(Value::is_null) {
                continue;
            }
            grouped += 1;
            let group = published["recursion_group"]
                .as_str()
                .expect("published group name");
            assert_eq!(
                published["node_id"]["path"],
                serde_json::json!([group]),
                "{name}: node {index}"
            );
            assert_eq!(
                serde_json::to_value(&admitted.graph().nodes[index].node_id)
                    .expect("typed node id"),
                published["node_id"],
                "{name}: node {index} retains its authored ordinal"
            );
        }
        assert!(grouped >= 2, "{name}: expected a real recursion group");
    }
}
