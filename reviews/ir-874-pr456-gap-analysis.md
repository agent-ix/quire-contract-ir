---
id: SR-12495
title: "IR-874 PR 456 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/mod.rs; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs"
review_set: subset
---

## Summary

Ticket: IR-874. Diff-scoped traceability and reverse-gap review of frozen. The new module is a staged implementation owned by FR-038-AC-281 and AC-282 and remains unwired to the reader. Its five unit tests bind local TC-048; the QSpec TC-946, TC-947 and TC-948 sources are retained as plain prose. No new criterion coverage is claimed for this partial stage.

## Examined units

- **FR-038-AC-281** (`examined`): "A reader-admitted protocol member has a `quire.op.protocol.control` body root, exactly one `protocol_profile` law matching the selected lock row, `null` mode, no leaves, Boolean result and semantic types, and a `profile_operator` member."
- **FR-038-AC-282** (`examined`): "Each of the seven protocol member forms admits only the FR-464, FR-465 or FR-466 fixed argument names, order and term shapes for its selected operator."
- **TC-048** (`examined`): "Use QSpec FR-464, FR-465 and FR-466 and the authored tables in TC-946, TC-947 and TC-948 as the current independent expected outcomes."

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Base and head each have 527/596 backed rows, 40 unbacked, zero contradicted, and 562 equal normalized criterion rows. Final head has zero unmatched trace-tag warnings. FR-038-AC-281 through AC-292 remain untagged until reader integration. The added source has an owning FR; no new stub or coverage inflation was found. Independent test-oracle quality is recorded in SR-12470. Plan completion: not assessed. Optional semantic review: not run.

## Verdict

PASS for this staged diff's traceability and reverse-gap delta. Repository-wide strict coverage remains at its existing 40-unbacked baseline; the full protocol reader remains outstanding.
