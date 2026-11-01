---
id: SR-202610310120
title: "IR-1208 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1208; shared.rs node-ID decoding and typed_terms_deserialize.rs"
review_set: subset
---

## Summary

Ticket: IR-1208. Independent frozen-candidate review of the two changed files. Rust lane is included in code-review. No implementation edits or Cargo commands. QSL public spelling is an explicitly recorded request only, outside this PR.

## Verdict

**CONDITIONAL** — one medium strict-wire regression requires correction.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Derived WireNodeId accepts positional arrays, widening the object-only node-ID grammar | crates/quire-contract-model/src/checked_package/shared.rs:765 |

The derived struct visitor implements visit_seq. serde_json text deserialize_struct accepts `[` and its borrowed Value implementation accepts Value::Array. Thus `["p",["x"],0]` now decodes as a CheckedNodeId, including through from_value because is_well_formed succeeds. A reference term with that array as target consequently decodes too. Both prior paths required a JSON object. Require a root map-only visitor and add array-root refusal controls for public term text/value and supplied-value decoding. This was traced through installed serde_json sources; no runtime reproduction was run under the no-Cargo constraint.

## Coverage

Plan completion: not assessed

Diff-scoped review only; no whole-repository assurance claim. Computed matrix saved at /tmp/ir1208-review/matrix.json. New tests carry TC-048 and IR-1208; they do not claim full FR-038-AC-377 reader admission. Seven node-bearing variants, strict segment forms, duplicate/unknown members, unchanged authority boundary and existing supplied-value shape controls were examined. No applicable AssuranceProfile was found. No spec/plan files changed, so spec review and its sub-analyses do not apply. Optional whole-repository semantic audit skipped; ordinary diff semantics reviewed as requested.

Parent-reported validation: Rust 1.98.1 formatting and workspace lib/tests Clippy with warnings denied; two new external tests; existing strict supplied-value and three checked-term conversion controls passed. Baseline valid reference failed only after compilation, demonstrating the original buffered-content failure. These results were supplied by the lead, not rerun here. Whole-suite and Kani intentionally not run under the user's speed rule.

## Reviewed units

- CheckedNodeId::deserialize (examined), crates/quire-contract-model/src/checked_package/shared.rs: `struct WireNodeId {
            package: Box<str>,
            path: Vec<Segment>,
            ordinal: u64,
        }`
- CheckedNodeId::from_value (examined), crates/quire-contract-model/src/checked_package/shared.rs: `let node = Self::deserialize(value).ok()?;
        node.is_well_formed().then_some(node)`
- public_term_deserialization_retains_node_ids_from_text_and_buffered_values (examined), crates/quire-contract-model/tests/typed_terms_deserialize.rs: `assert_eq!(text, buffered);
        assert_eq!(serde_json::to_value(&buffered).unwrap(), wire);`
- public_term_deserialization_rejects_malformed_node_ids_and_unknown_members (examined), crates/quire-contract-model/tests/typed_terms_deserialize.rs: `assert!(serde_json::from_str::<CheckedTerm>(text).is_err());`
