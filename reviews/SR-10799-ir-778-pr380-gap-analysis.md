---
id: SR-10799
title: "IR-778 PR 380 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; src/kani/mod.rs, src/kani/outcome.rs, tests/it/kani_shared.rs"
review_set: subset
---

## Summary

Ticket: IR-778. Diff-scoped requirement ↔ test ↔ source check of PR #380. Quire tags FR-030-AC-7, but the local tests leave one malformed shape untested and do not prove the future CG producer clause.

## Verdict

**CONDITIONAL** for this PR's criterion evidence. The IR carrier behavior works in the focused tests; FR-030-AC-7 remains only partly evidenced. The pre-PR full gate remains red at strict coverage (45 baseline to 43 on this PR).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Non-cancelled kind with nested cancellation origin is not tested | tests/it/kani_shared.rs:253 |
| FND-002 | medium | TC-449 tags credit the full AC before CG event-origin control exists | tests/it/kani_shared.rs:200 |

## Coverage

Plan completion: not assessed. Examined FR-030-AC-7, FR-039's public-item table, TC-449, and the three changed files. `quire matrix --scope . --format tsv` reports FR-030-AC-7 tagged by the three new tests; FR-039-AC-1 through AC-4 remain untagged from pre-existing TC-055 work and are not new findings for this diff. The CG producer control is assigned to IR-701 by TC-449. Three focused integration tests passed.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | PR adds `{"timed_out": {"source": "deadline"}}` to the malformed-kind refusal table; the focused test passes. |
| FND-002 | fixed | PR removes the FR-030-AC-7 tag from all three IR-only tests, leaving TC-449 identity; Quire now reports AC-7 untagged, and the spec states that CG's live-event reversal control remains for IR-701. |
