---
id: SR-597002
title: "Gap analysis — IR-874 catalog graph PR diff"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #597 source and tests; FR-038-AC-325; FR-064-AC-5"
review_set: subset
---

## Summary

Ticket: IR-874. This planless, PR-diff audit compared the changed catalog code and tests with the computed Quoin matrix and the two governing acceptance criteria. The full criteria remain untagged and expressly unclaimed; the implemented subset has behavioral tests, but the closed await class case and model-only operation class boundary have no test.

## Verdict

**CONDITIONAL** — two missing cases allow the code-review defects in SR-597001 to escape the changed tests. This is a subset verdict, not a repository-wide PASS.

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-325 | examined | The protocol step compares every present refinement `class_catalog` against the complete concrete and abstract classes reconstructed from selected model declarations and the admitted local or directly selected dependency protocol graph, including distinct `fork` and `join` classes on one parallel node, checked binder types and source-default memory classes. |
| FR-064-AC-5 | examined | A protocol refinement whose complete concrete catalog includes classes from an admitted clause owned by its selected concrete model is admitted even when no step row names those classes. A clause whose edge names another model contributes no class to that catalog, regardless of similar declaration names or binder types. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No catalog test includes an admitted `await` holder, so the invented await class goes unchallenged. | tests/it/checked_package_v2_refinement.rs:968 |
| FND-002 | medium | No abstract-protocol refinement test supplies a concrete model operation class and checks admission; the current positives have only clause-derived concrete classes. | tests/it/checked_package_v2_refinement.rs:1110 |

## Coverage

Plan completion: not assessed.

`quoin matrix --repo . --json` at the reviewed worktree reports both FR-038-AC-325 and FR-064-AC-5 as `untagged` with no run evidence. That matches the draft PR's explicit unclaimed state and is not presented as completed acceptance coverage. New tests exercise owned finish, foreign clause, fork/join, send carries, local abstract join and direct-dependency abstract join through the public reader; the model unit tests exercise inherited operation receiver/result and direct record field order. Semantic intent was checked for the changed subset in this requested review; optional repository-wide semantic review was not requested. No code without an owning FR was found in this diff.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At `21f525d9`, `refinement_catalog_joins_timeout_on_await_node` constructs an admitted await holder with two timeout classes and asserts strict-reader admission; QSpec FR-426 confirms timeout belongs to await. |
| FND-002 | still-open | At `21f525d9`, no positive test covers a concrete model operation class in an abstract-protocol refinement. The model-root carrier required to reconstruct it remains in IR-1043; PR #597 is draft and does not claim FR-038-AC-325. |
