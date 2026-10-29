---
id: SR-16604
title: "IR-9 PR 541 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir PR #541 @; crates/quire-contract-model/src/checked_package/v2/flat_wire.rs"
review_set: subset
---
# SR-16604: IR-9 PR 541 code and Rust review

## Summary

Independent diff-scoped code and Rust review against. The changed flat-wire header check uses `decode_node_kind` for the successful typed kind and rechecks only failed extension, hybrid and timed forms to retain their earlier `malformed_wire` refusal priority.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope examined

- The failed-header branch obtains `CheckedNodeTag` and `CheckedNodeKind` from the existing closed vocabulary; it preserves `malformed_wire` at `semantic_form` when extension, hybrid or timed forms are invalid even if `schema_version` is also invalid. Valid special forms and ordinary invalid headers retain `decode_node_kind`'s code and path.
- Each valid header is decoded once and its `CheckedNodeKind` is retained for body dispatch, timed interval checks and hybrid formula classification. The body is checked before a later node's header; diagnostics remain after nodes. The changed branch adds no panic, unsafe path, public API, wire member, lock, rule or marker.
- Four new `#[trace("TC-048")]` private tests invoke the actual `flat_wire::check`, covering the three special families with bad form and schema, known forms with bad schema, ordinary unknown tag/form, and body versus later-header document order. Their assertions compare typed refusal code, pointer and absent cause. Published QSpec extension, hybrid and timed fixtures remain the public admission controls.
- The test helper constructs private wire input only; it does not claim authenticated identity. `git diff --check` passed. No aggregate gate was run by this reviewer.

## Verdict

PASS for the exact PR diff. This is incremental typed-consumption progress; it does not claim FR-056-AC-8 source cleanliness.
