---
id: SR-11761
title: "Gap analysis — IR-795 four-owner source-inventory port"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/source_obligation.rs, tests/it/source_obligation.rs, spec/model/functional/FR-019-rust-library-interface.md, spec/model/functional/FR-052-source-obligation-correspondence.md, spec/model/matrix/TC-452-source-obligation-correspondence.md, spec/model/matrix/tests.md (PR #413); FR-052-AC-14, FR-052-AC-15, TC-452"
review_set: subset
---

## Summary

Ticket: IR-795 (IR-759 Slice B). This is a planless gap analysis of PR #413. Plan completion: not assessed.

Commands run: `quire validate --scope . 'spec/**/*.md' 'plan/**/*.md' 'reviews/**/*.md' --summary` (893/893 grammar-clean) and `quire coverage --scope . --strict`. Coverage reports 47 unbacked rows and 0 contradicted. The same command on origin/main reports 49 unbacked and 0 contradicted, which confirms the author's numbers. The two rows that moved are FR-052-AC-14 and FR-052-AC-15. `quire matrix` shows both tagged by tests/it/source_obligation.rs:136 and :200.

The FR-019 public-items row for `source_obligation` lists 29 types and 1 trait, matching the 30 `pub` items in the module exactly, and `public_interface` TC-058 passes. AC-16 has no claim and no Quire producer, and AC-9 through AC-11 stay Planned. No census, builder or adapter code was added.

## Examined scope

- FR-052-AC-14 (examined): A default-feature neutral consumer distinguishes a typed Quire criterion identity containing obligation declaration, scoped document and authored criterion ID from `ClauseRef` and QSL `RequirementRecord`. Two neutral fixture rows with distinct authored criterion identities, one without a correspondence assertion, remain distinct Quire-owned rows before any generation or result. A formal clause, QSL requirement record or assertion-only key cannot substitute for either fixture criterion. This neutral fixture does not claim Quire owner-issued completeness.
- FR-052-AC-15 (examined): An explicit source-author binding connects the selected Quire corpus/obligation scope to the selected IR `SourceIdentity`. A missing, foreign or conflicting binding refuses a complete census with distinct typed causes; a matching label or revision alone is not treated as proof of equal source bytes across Quire, IR V2 and QSL. Foreign and dangling AC-to-occurrence/function assertions retain their separate typed dispositions.
- FR-052 statement, corrected-port paragraph (spec/model/functional/FR-052-source-obligation-correspondence.md:89-101) (examined): The current port's `Criterion(ClauseRef)` and its QSL owner assignment are provisional defects to correct under IR-795 ... FR-019 shall add the two new public items when implemented, while retaining its inventory of the current compiled surface until then.
- FR-052 status paragraph (spec/model/functional/FR-052-source-obligation-correspondence.md:363-370) (examined): FR-052-AC-14 is Implemented for the neutral typed identity and synthetic Quire-owned rows, with traced integration and compile-fail tests. The neutral binding retention portion of FR-052-AC-15 is Implemented with traced tests; its missing, foreign, and conflicting binding census refusals and assertion dispositions remain Planned/UNRUN until the census builder exists.
- FR-019 source_obligation public-items row (examined): it matches the compiled surface.
- TC-452 Execution Status and tests.md TC-452 row (examined): AC-14 and the AC-15 port are recorded as Implemented, and the AC-15 census refusals as Planned/UNRUN.
- FR-052-AC-16 (context_only): out of scope (IR-812).

## Verdict

CHANGES REQUESTED (medium). The AC-14 status is truthful: two distinct Quire-owned rows, one without an assertion, are tested, and the mutations confirm the owner assignment. The weakness in its negative oracle is recorded in SR-11760 FND-001.

The AC-15 prose is truthful, since census refusals stay Planned. However, the trace tags give the computed matrix full AC-15 credit for an accessor round-trip (FND-001). FR-052 still describes the defect this PR fixes as current (FND-002).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Coverage inflation: two tests carry an FR-052-AC-15 trace tag but only round-trip `SourceCriterionSourceBinding` accessors, and `quire coverage --strict` now counts AC-15 as backed (49 to 47). AC-15's falsifiable content has no test and no code. That content is the distinct typed refusal of a missing, foreign or conflicting binding, and the separate assertion dispositions. Drop the AC-15 tag until Slice C, or split AC-15 into a retention criterion and a census-refusal criterion. | tests/it/source_obligation.rs:133-134, tests/it/source_obligation.rs:197-198, spec/model/functional/FR-052-source-obligation-correspondence.md:353 |
| FND-002 | medium | FR-052 contradicts the code after this PR. It still says "The current port's `Criterion(ClauseRef)` and its QSL owner assignment are provisional defects to correct under IR-795". It also says FR-019 "shall add the two new public items when implemented, while retaining its inventory of the current compiled surface until then". This PR removes `Criterion(ClauseRef)` and adds both items to FR-019. | spec/model/functional/FR-052-source-obligation-correspondence.md:95-101 |
| FND-003 | low | The FR-052 status says AC-14 has "traced integration and compile-fail tests", but the `compile_fail` doctests carry no trace (TC-452 correctly names only tests/it/source_obligation.rs). Reword this to "traced integration tests plus untraced compile_fail doctests". | spec/model/functional/FR-052-source-obligation-correspondence.md:363-364 |

## Coverage

Before (origin/main): 49 unbacked, 0 contradicted. After : 47 unbacked, 0 contradicted. FR-052 is backed for 6 of 16 criteria. AC-14 is genuinely backed. AC-15 is tagged, but see FND-001.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed in the reviewed PR. The FR-052-AC-15 trace tags are removed from both tests, and AC-15 is now unbacked: `quire coverage --strict` reports 48 unbacked and 0 contradicted, with AC-15 listed. AC-15 kept as one criterion that is Planned and unbacked is consistent in the prose and in the computed matrix, which is acceptable. |
| FND-002 | fixed | Fixed in the reviewed PR. FR-052:89-99 now gives the current state: the neutral port uses `SourceAuthoredCriterionId`, a Criterion row has owner Quire, and FR-019 lists both public items. |
| FND-003 | fixed | Fixed in the reviewed PR. FR-052:364-365 now reads "traced integration tests and untraced compile-fail doctests". |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | copied SR-11760 over the existing `reviews/ir-795-code-review.md`. On main that file is SR-11135, the PR #401 code review, so the merge would delete that review record from HEAD. The slug collision came from this reviewer's file name. Restore the main version and commit the PR #413 artifacts under distinct names (`reviews/ir-795-pr413-code-review.md`, `reviews/ir-795-pr413-gap-analysis.md`). | reviews/ir-795-code-review.md, reviews/ir-795-gap-analysis.md |
