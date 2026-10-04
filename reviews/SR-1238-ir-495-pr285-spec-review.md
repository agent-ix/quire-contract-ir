---
id: SR-1238
title: "spec review of PR 285 (IR-495 status flips and FR-038 wording)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@f3d41bca80fe145bdabd345e1d13749f6be1fcaf; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, spec/assurance/AD-004-checked-package-seam.md, spec/tests.md"
review_set: subset
---
# SR-1238: spec review of PR 285

## Summary

Ticket: IR-495. This reviews the spec edits in the PR. They are the flips from
planned to implemented in FR-038 (Inputs, Reading, order of checks, The flat wire,
AC-114 through AC-118), FR-040-AC-10, AD-004, TC-048, the checked-package matrix and
`spec/tests.md`, plus the FR-038-AC-9 change from "seven" to "six".

What holds:

- The AC texts of AC-114 through AC-118 are unchanged apart from the removed 🚧 marker.
  R-S8 stays as merged: AC-115 and AC-116 keep the nested-`case` reading, and no
  "malformed_wire wherever they sit" wording was added.
- AC-9 "six" is correct: the Inputs paragraph lists six limits (bytes, nodes, edges,
  occurrences, diagnostics, work).
- Each flip points at a tagged test (see SR-1237). TC-048 now names
  `tests/it/checked_package_v2_flat_wire.rs` and `tests/conformance_qspec/main.rs`.
- The FR-346 text on an application nested in an abstraction relation body was already
  merged (`malformed_wire` at strict wire validation), so the code's change there is no
  silent spec change.
- `quire validate` passes. Grammar findings: 1 (FR-014, pre-existing).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-117 tests nesting at only two points: 20 levels (refused by the grammar) and 300 levels (refused by the parse). Nothing tests the window between them, a body nested beyond the grammar but within the parse limit of 128. FR-038 "Reading" promises that window two things: refusal at the first value outside the grammar, and no recursion on the call stack. It is also the window where the reader overflows a 256 KiB debug stack today (SR-1236 FND-002). Add a clause: a body nested to the parse limit (127 JSON levels) read on a 256 KiB thread refuses `malformed_wire` at the first value outside the grammar, with no stack overflow | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2317 |

## Verdict

The edits are accurate except for FND-001 here and the "Reading" claim in SR-1237
FND-001. Adding the parse-limit clause to AC-117 would have caught the overflow, and it
would protect the fix.

## Dispositions

Round 1, reviewed at b7dd914b9c995dc9979d31cebfea5feafb4822ad.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | b7dd914. AC-117 adds "one nested within the strict parse's limit but far past the grammar (a body of 61 aggregates, 126 JSON levels) ... on a 256 KiB thread in a debug build with no stack overflow", and TC-048 lists 41 to 61 aggregates and a 50-deep `details` term. `tc_048_a_document_nested_past_the_parse_limit_refuses_at_the_parse` and `tc_048_a_deep_details_term_within_the_parse_limit_refuses_at_the_grammar` back the clause |
