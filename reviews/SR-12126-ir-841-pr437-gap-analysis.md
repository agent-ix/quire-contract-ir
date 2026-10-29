---
id: SR-12126
title: "IR-841 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, crates/quire-contract-model/src/checked_package/common.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

## Summary

Ticket: IR-841, PR quire-contract-ir#437. The scope is the TC-048 test `tc_048_a_deep_syntax_error_is_refused_in_linear_time`, the FR-038 text on deep documents and the strict first pass, and the first-pass code it measures. Plan completion: not assessed.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038 (strict parse prose, line 220) | examined | A syntax or duplicate-member defect anywhere in the document refuses first. |
| FR-038-AC-117 | context_only | ... an otherwise canonical document whose node body nests a term 300 levels deep, past the strict parse's recursion limit of 128, read on a thread whose stack is 256 KiB, refuses `malformed_wire` with no pointer at the strict parse ... with no stack overflow. |
| TC-048 | examined | CheckedPackage V2 strict reader re-derives package and nominal identities (the test's only trace tag) |
| common.rs:1285-1297 `strict_shape` doc | examined | read iteratively with an explicit stack of open containers, so cost is linear in the input and independent of nesting |
| common.rs:1964-1999 the test | examined | asserts work at 524,000 levels is 3.5x to 4.5x the work at 131,000 |

## Method

I searched FR-038 and TC-048 for any stated time or work bound on the first pass, then ran `quire trace --id TC-048 --json` and `make spec`. I compared what the old wall-clock oracle and the new work-count oracle can each detect, using the mutants in SR-12125.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No acceptance criterion states the linear-cost property the test checks. FR-038 and TC-048 require that a deep document refuses `malformed_wire` without a stack overflow (FR-038-AC-117) and that a syntax defect anywhere refuses first. Neither states that the first pass is linear in the input. That claim lives only in the `strict_shape` doc comment and in this test's name. The test carries `#[trace("TC-048")]` alone, so the matrix cannot show which criterion the linear-time check backs. A weakened oracle (SR-12125 FND-002) therefore changes no coverage row. This gap predates the PR; the PR neither opens nor closes it. | crates/quire-contract-model/src/checked_package/common.rs:1285-1290, 1977-1978; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:220 |

## Verdict

One low finding, which predates the PR. Nothing in this PR's scope blocks merge.

- Whether the oracle is faithful: a work count is a sound oracle for "linear time" only over the steps that charge. On this test's input, that is one step: the outer loop turn (SR-12125 FND-001). A real quadratic that takes the charged `pointer` path is caught. One that does uncharged work (a per-level stack rescan) is not, and the old wall-clock test would have caught it. The new test is deterministic: it gives the same counts on every run and does not depend on load. The trade is that it detects fewer regressions.
- The test has no stub or tautology. It asserts a count the real parser produces on a real input, plus the `malformed` outcome.
- Tag and trace are unchanged. `quire trace --id TC-048 --json` still resolves the test by the same name. `make spec` gives strict 43 unbacked / 0 contradicted and `Coverage: 508/565 rows backed`, both unchanged. No spec or plan file is in the diff.

## Dispositions

Round 1, against.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | This predates the PR and is out of its scope: closing it means a spec change to FR-038/TC-048, adding a criterion for the first pass's cost, and IR-841 is a test-determinism ticket. No follow-up ticket exists yet (tracker search found none); the lead should file one if the property is wanted as a criterion. The PR body acknowledges the gap, and the test's doc comment no longer claims more than the count shows. |
