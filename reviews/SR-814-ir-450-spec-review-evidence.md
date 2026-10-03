---
id: SR-814
title: "evidence review of PR 257 quantity bound class (IR-450)"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir@10a17ec6b705ec88b5442a3367b826043d76ed2b; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/tests.md; spec/checked_package/matrix/TC-050-checked-package-v2-lowering.md"
review_set: subset
---
# SR-814: evidence review of PR 257 quantity bound class

## Summary

Ticket: IR-450. `Test (TC-050)` is the right method for FR-038-AC-73. The TC-050 row and the
FR-038 row mark AC-73 planned with no test, which matches the code: `requires_bound` returns
false for both forms. The TC-050 description and coverage design name AC-73. `quire coverage
--strict` stays at 23 unbacked rows. The amended AC-8 keeps its TC-052 trace and stays in the
FR-038 row's implemented list.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-8's new clause covers "a requested `unit` or `compound_unit`", but TC-052, its traced test, never lowers a `compound_unit`. `tc_052_unbounded_forms_are_exactly_the_eight_declared_forms` lowers the nominal fixture, which holds only enum, dimension and unit nodes; the test's comment says "enum, dimension and unit are the three remaining declared scalar forms". The TC-052 coverage-design row ("the three nominal forms on the fixture that carries them") was not updated. AC-8 is still listed as implemented and TC-052 as ✅. So an explicit clause of an implemented AC has no evidence. Either drop `compound_unit` from AC-8, since AC-73 already covers a requested `compound_unit` through TC-050, or add a requested `compound_unit` to TC-052's required cases and mark that clause planned. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1117; spec/checked_package/matrix/tests.md:24, 36; tests/it/checked_package_v2_lowering.rs:522-538 |

## Verdict

Request changes (medium). AC-73's evidence plan is sound. AC-8's new `compound_unit` clause
needs either a test or removal.

## Dispositions

Round 1 at 4b19dbe6da4c34665076d325b4d7d385d265fa50. FR-038-AC-8 is now byte-identical to origin/main, so TC-052's evidence covers it as before. AC-73's requested-compound_unit case stays under TC-050, where it is planned. `make spec`: grammar 312/313 (baseline 1), strict 23 unbacked, 0 contradicted. No new findings.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | fixed 4b19dbe6da4c34665076d325b4d7d385d265fa50 |
