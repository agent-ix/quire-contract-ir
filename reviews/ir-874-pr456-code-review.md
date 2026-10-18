---
id: SR-12470
title: "IR-874 PR 456 code and Rust review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/mod.rs; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs"
review_set: subset
---

## Summary

Ticket: IR-874. Diff-scoped code and Rust review of frozen head d1ec73c2802101d7694046fc79f56e3fa391a14e. The staged outer-body validator has no reader call yet, as documented in the module. Its twenty operator/form entries and the named outer bindings were checked against QSpec FR-464, FR-465 and FR-466. Focused tests passed (5/5). The five local TC-048 trace attributes resolve; no triple-slash Trace or Tracing lines were added.

## Examined units

- **TC-048** (`examined`): "Use QSpec FR-464, FR-465 and FR-466 and the authored tables in TC-946, TC-947 and TC-948 as the current independent expected outcomes."
- **FR-038-AC-281** (`examined`): "A reader-admitted protocol member has a `quire.op.protocol.control` body root, exactly one `protocol_profile` law matching the selected lock row, `null` mode, no leaves, Boolean result and semantic types, and a `profile_operator` member."
- **FR-038-AC-282** (`examined`): "Each of the seven protocol member forms admits only the FR-464, FR-465 or FR-466 fixed argument names, order and term shapes for its selected operator."

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Positive layout tests build expected bindings from the production catalog itself, so a catalog name or shape regression can leave those tests green. Add an independent expected layout check grounded in QSpec for the staged outer fields. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:377; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1504 |

## Verdict

CONDITIONAL. The validator's current table agrees with the authored QSpec tables, the focused tests pass, and Rust structure and error handling present no other diff-scoped issue. FND-001 leaves the fixed binding table without independent test evidence. This is a partial reader slice; FR-038-AC-281 through AC-292 are not claimed as complete.
