---
id: SR-11190
title: IR-474 typed closed-member provenance gap analysis
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; FR-038-AC-2, FR-038-AC-24, TC-048 and changed checked-package Rust/test paths"
review_set: subset
---

## Summary

Ticket: IR-474. Diff-scoped trace and evidence review of `...`. The changed `unknown field` role mutation reaches the public V2 reader, asserts `MalformedWire` at the exact role pointer, and sits beside genuine unknown-member and wrong-kind controls in the same traced test.

## Verdict

**PASS** — no new criterion, trace, source-owner, stub, or hollow-test gap was found in the changed slice. The repository-wide baseline is outside this diff-scoped review.

## Examined units

- `FR-038-AC-2` (examined; `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): A role value spelled `unknown field` refuses as `malformed_wire`, while an extra member is `unknown_member` and a wrong-kind value remains `malformed_wire`.
- `FR-038-AC-24` (examined; same path): Refusal pointers identify the offending role value, unknown member, or wrong-kind value.
- `TC-048` (examined; `spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md`): Public reader mutation controls require the exact code and pointer, with the role mutation based on an otherwise admitted package.
- `FR-038-AC-5` (context_only; `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): Nominal preimage errors keep the established graph-refusal classification except genuine unknown members.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed.

`quoin matrix --repo . --json` was inspected for the changed criteria. The new mutation is inside `tc_048_v2_reader_refuses_injected_wire_evidence_and_graph_faults`, tagged `FR-038-AC-2` and `FR-038-AC-24`; the test has exact code and pointer assertions through its refusal table. Existing nominal and structural controls retain their assertions. Optional repository-wide semantic review was not run; this review is scoped to the PR diff.
