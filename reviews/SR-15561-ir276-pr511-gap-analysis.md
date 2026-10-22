---
id: SR-15561
title: "IR-276 PR 511 criterion coverage and evidence review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir PR #511; FR-038-AC-308, FR-023-AC-7, changed Rust tests and production paths"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-023
    type: references
---

## Summary

The changed tests carry trace tags for both criteria, and the implementation has owning requirements. The computed repository matrix reports FR-023-AC-7 as tagged but does not list FR-038-AC-308, so that criterion's matrix status remains unconfirmed by this command. Two tagged test groups exercise only part of their criterion: the lowering parity comparison shares the new implementation on both sides, and the projection cache test covers only schema reuse on invalid input.

## Verdict

**CONDITIONAL** — criterion coverage needs independent output parity and a valid/invalid decode plus lookup-visit test before FR-038-AC-308 and FR-023-AC-7 can be claimed fully backed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new batch-versus-single lowering assertions compare the new algorithm with itself; they do not assert the criterion's unchanged records, work-limit consumed values, or canonical package bytes against an independent pre-change baseline. The shared-closure counter also uses a hand-built package and test-only fact scanner, so it does not directly measure reader admission. | crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs:331; tests/it/checked_package_v2_parameters.rs:334; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5024 |
| FND-002 | medium | FR-023-AC-7 covers valid and invalid projection decodes, bounded declaration search visits, and unchanged identities or diagnostics. The new test calls schema validation twice on the same invalid object and compares cached pointer identity; it does not exercise valid decoding, count declaration visits, or compare decode results. | crates/quire-contract-model/src/binding.rs:473; spec/model/functional/FR-023-executable-projection-binding.md:139 |

## Coverage

The computed Quire matrix reports FR-023-AC-7 as tagged to binding.rs:473. It did not list FR-038-AC-308 among its 44 coverage rows; the source tests do carry that trace attribute at ceiling_tests.rs:329 and 362. Their cited tests were inspected against the criterion text and production code. No unowned new production behavior or stub was found. Plan completion: not assessed. Optional semantic review was not requested; the narrow test-to-criterion checks above are direct evidence checks for this PR.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At a86717e, the counter increments in `validate_body` itself. Independent old/new compiled-source snapshots from be5fcb1 and the new source compare equal with `cmp` for full and stopped records, including consumed values, and both canonical package byte streams. The eight recorded old/new artifacts under `qualification-logs/ir276-ir-read-lower/{base,new}` were compared by this reviewer. |
| FND-002 | still-open | At a86717e, valid and invalid decodes and output parity are covered. The new `DECLARATION_VISITS` counter at binding.rs:278 and :317 counts outer clause and binding loops, but never counts the linear `resolve_clause` fallback at :321-323. For a missing reference, that fallback scans requirements and clauses in identity.rs:1272-1275 and :1312-1315. Thus the test cannot establish the D+B visit bound for invalid decodes, and a large D with one missing binding can exceed it. |
| FND-002 | fixed | At b4bf151, binding.rs:322-330 resolves both valid and invalid bindings through prebuilt requirement and clause indexes. `resolve_clause_candidates` in identity.rs:1334-1345 shares the package/orphan/stale/clause diagnostic order with ordinary resolution, without scanning the package on an index miss. The AC-7 test counts D+B for valid decodes and D+1 for a first missing clause, twice; prior valid and invalid result comparisons remain. |
