---
id: SR-202610310136
title: "Gap analysis of IR-1209 admitted equality PR 778"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 778 frozen head 5178bd8f; FR-353-AC-2 through AC-7, FR-352-AC-11; IR-1209"
review_set: subset
---

## Summary

Ticket: IR-1209. Scoped the computed Test Matrix, changed tests, and production source to the delivered admission and primitive Plan read. The new code has ownership in FR-353. The positive Text, enum, quantity and declaration-bearing Plan cases await specified QSL stage rows and are outside this PR's claimed delivery. No applicable AssuranceProfile was found under `spec/`.

## Verdict

CONDITIONAL. The FR-353-AC-4 binding is present but its one-less assertion accepts the wrong error code, leaving the exhaustion oracle hollow. No plan completion was assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The AC-4 one-less work test accepts generic Materialization and does not assert attempted_work or the first denied locus. | crates/quire-contract-model/src/checked_package/v2/composite_operands.rs:1461 |

## Coverage

Plan completion: not assessed. `quire matrix --scope . --format json` completed successfully on the frozen tree. Scoped trace tags were inspected for FR-353-AC-2, AC-3, AC-4, AC-5, AC-7 and FR-352-AC-11. FR-353-AC-1, AC-6 and AC-8 remain broader future obligations; this scoped review makes no completeness claim for them. The AC-7 source-unit test establishes missing SourceUnit then DimensionDeclaration refusals, but does not claim the other witness kinds are delivered in this PR. Optional semantic review was not invoked.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: the one-less test asserts WorkLimit, attempted_work=1 and the first denied `short.at` node, closing the missing oracle. |
