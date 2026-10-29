---
id: SR-610033
title: "IR-1053 PR #603 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir PR #603; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, refinement.rs; tests/it/checked_package_v2_protocol_body.rs, checked_package_v2_refinement.rs"
review_set: subset
---

## Summary

Reviewed the frozen non-memory owner and catalog slice against merged FR-038, FR-061, FR-064 and QSpec FR-375/FR-464. The owner-kind gate is corrected, but the unselected-object refusal still uses the wrong typed result.

## Verdict

**CONDITIONAL** — one medium code finding requires a fix in this PR. The source-default memory/drain catalog remains an open IR-1034 dependency, not a claim of completion here.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | An unselected `object_type` owner returns `missing_declaration`/`missing-selection` at `owner_model`; FR-064-AC-4 requires `invalid_model_binding`/`malformed-declaration`. The new kind check passes the target, then forwards `ModelOwners::recover`'s unselected result unchanged at protocol_body.rs:4804-4810. | FR-064-AC-4; `protocol_body.rs:4804-4810` | implementation-bug-despite-evidence |

## Examined units

- FR-064-AC-4 (examined): With wire, graph and identities valid, an `owner_model` pointing at a non-model node, a source-local declaration, a model declaration owned by no selected declaration, or a selected declaration's model node of any form other than `object_type` (a `state_machine`, `process`, `record_value_type`, a relation node, or the selected package's `model_root`) refuses at the edge under QSpec FR-464's typed model-binding check before clause interpretation. A separate earlier wire, graph or stale-key fault wins when paired with such a meaning defect. A source producer given a clause de
- FR-064-AC-5 (examined): Selected concrete model-root and exact selected object-type clause-owner join; no-step class counts, foreign class exclusion.
- FR-038-AC-325 (examined): Complete selected class catalog, including deferred source-default memory/drain entries.
- FR-061-AC-1/3/7 (context): Refinement semantic type is the canonical selected model root.

## Rust review

No copied QSpec content, duplicate evaluator, public API addition, unsafe code, new panic, or gate weakening appears in the four-file diff. Existing model-owner recovery is reused. The new wrong-kind refusal and no-step catalog test use the public reader, and writer readback asserts node and package identity. The error mapping above remains incorrect for an object-type key outside selected model documents. No aggregate gate was run in this review pass, per dispatch cadence.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | QSpec FR-464-AC-20 now fixes the owner-join phase: genuinely unselected object-type nodes refuse `missing_declaration`/`missing-selection` at their own `node_id` before any clause step. FR-064-AC-4 and TC-048 now state that exact oracle, and the new public-reader test asserts code, cause, path and locus. The protocol-step mapping remains correct for targets that reach it. |

## Rebase and spec-delta check

Compared PR #603 with and QSpec FR-464-AC-20. The changed FR-064 Behavior and AC-4 assign unselected `model`/`object_type` to the earlier target owner join; wrong-kind or missing local targets to the clause-edge check; and body defects to their respective earlier or later phases. TC-048 step 10 names the same phases. The source unchanged from the earlier reviewed owner recovery returns the now-specified unselected result. No new code defect found in this delta.
