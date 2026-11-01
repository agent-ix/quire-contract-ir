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

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Root decoding now calls deserialize_map; NodeVisitor implements only visit_map and delegates closed fields through MapAccessDeserializer. Positional arrays cannot reach the derived struct sequence visitor. Added standalone CheckedNodeId and CheckedTerm text/value array-root refusal controls, plus the private supplied-value control. |

### Disposition round 1 verdict

**PASS** — FND-001 is fixed; no new defect found in the fix. Original findings above remain unchanged. Inspected the map-only root gate, reused strict-field decode and adverse array tests. The string_edge annotation marks the existing segment string-key boundary without changing its runtime wire grammar. Parent reports focused tests, workspace lib/tests Clippy and formatting passed. Local qsl-string-edge was unavailable; CI rerun is queued, and no local gate pass is claimed. No Cargo commands, implementation edits or whole-suite/Kani runs were performed by this reviewer.

Fixed excerpt:

```rust
fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<WireNodeId, A::Error> {
                WireNodeId::deserialize(serde::de::value::MapAccessDeserializer::new(map))
            }
        }
        let wire = deserializer.deserialize_map(NodeVisitor)?;
```

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | Generic node-ID decoding exposed Serde member paths in native package refusals, breaking required diagnostic loci | crates/quire-contract-model/src/checked_package/v2/intake.rs:119 |

The original review missed the native package diagnostic contract. Completed CI Test after the original merge reported 1102 passed and two failures: malformed external owner_model exposed /owner_model/node instead of /owner_model; an inline nested operand exposed /semantic_type/path/0/ordinal instead of the path segment. These were real regressions; existing assertions must remain unchanged. The original focused tests did not establish full reader integration correctness.

### Disposition round 2

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | Native intake now uses private WireNodeId for node_id, semantic_type, dependencies and owner_model. It borrows RawValue and invokes the sole public CheckedNodeId decoder once. Converting the inner JSON error to the outer Serde error restores the ID boundary before invalid_node_id_field refines ordinal or segment loci. |

### Corrective review scope and result

**PASS for corrective static review; integration verification pending.** No new defect found in the corrective diff. Only intake.rs changed; protected files and public generic Deserialize remain unchanged. The wrapper introduces no alternate grammar, fallback, allocation cap, unsafe code or admission authority. Borrowed raw decoding is confined to the native JSON intake which already consumes source slices. The resulting ID owns its data and cannot escape with a borrowed lifetime. Both existing diagnostic test assertions remain unchanged.

Examined: WireNodeId::deserialize (intake.rs), excerpt: `let raw = <&RawValue>::deserialize(deserializer)?;`; BorrowedNode ID slots, excerpt: `dependencies: Vec<WireNodeId>`; decode_borrowed_wire unwrapping, excerpt: `owner_model: node.owner_model.map(|id| id.0)`; invalid_node_id_field existing refinement, excerpt: `return Some(at.clone().key("path").index(index));`; inline_path_operand_refuses_at_the_segment, role examined, excerpt: `/semantic_type/path/0`; original external owner_model failure, role context_only, source: lead-provided completed CI result.

Parent reports initial Clippy passed and a focused baseline reproduced the inline failure. Final formatting/Clippy and full workspace integration target with conformance-cli, ignored tests and one test thread are queued. No completed full integration or CI Test success is asserted by this review; merge readiness depends on that result. No Cargo or implementation edits were performed by this reviewer.

Fixed excerpt:

```rust
let raw = <&RawValue>::deserialize(deserializer)?;
        serde_json::from_str(raw.get())
            .map(Self)
            .map_err(serde::de::Error::custom)
```
