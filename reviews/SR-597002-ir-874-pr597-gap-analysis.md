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

`quoin matrix --repo. --json` at the reviewed worktree reports both FR-038-AC-325 and FR-064-AC-5 as `untagged` with no run evidence. That matches the draft PR's explicit unclaimed state and is not presented as completed acceptance coverage. New tests exercise owned finish, foreign clause, fork/join, send carries, local abstract join and direct-dependency abstract join through the public reader; the model unit tests exercise inherited operation receiver/result and direct record field order. Semantic intent was checked for the changed subset in this requested review; optional repository-wide semantic review was not requested. No code without an owning FR was found in this diff.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed |, `refinement_catalog_joins_timeout_on_await_node` constructs an admitted await holder with two timeout classes and asserts strict-reader admission; QSpec FR-426 confirms timeout belongs to await. |
| FND-002 | still-open |, no positive test covers a concrete model operation class in an abstract-protocol refinement. The model-root carrier required to reconstruct it remains in IR-1043; PR #597 is draft and does not claim FR-038-AC-325. |

## Disposition Round 2

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | still-open |, `refinement_catalog_joins_concrete_model_operation_without_a_step_row` exercises the selected concrete operation inventory through the public reader, including admission, omission, node and binder refusals. Its fixture calls `row_empty_refinement`, whose body has no `protocol` member, so it does not exercise the abstract-protocol refinement configuration named by this finding. Add an admitted `body.protocol` case with the operation class on the concrete side and the complete abstract catalog; keep FR-038-AC-325 and FR-064-AC-5 unclaimed until their full obligations are met. |

**CONDITIONAL; NOT MERGEABLE ON THIS REVIEW** — the new model-operation test improves slice evidence but leaves the specified abstract-protocol combination unverified. No aggregate gate was run in this reviewer pass.

## Disposition Round 3

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed |, the public-reader `refinement_catalog_joins_concrete_operation_and_abstract_protocol_without_step_rows` test retains the concrete `Worker::run` operation class, adds the selected local clause as `body.protocol`, supplies its complete abstract `OrderFlow::Closed` finish class, and asserts strict-reader admission. It then removes that abstract class, rekeys, and asserts `invalid_semantic_graph` at `/body/class_catalog/abstract`. Both positive and adverse paths fail if the abstract-protocol join is skipped or misapplied. The test carries Provenance only, so FR-038-AC-325 and FR-064-AC-5 remain unclaimed. |

**PASS for the reviewed partial slice** — FND-001 and FND-002 are fixed by the latest dispositions. The PR is review-mergeable for this partial slice, subject to the code lead's exact-head gates and normal branch preparation. Full FR-038-AC-325 and FR-064-AC-5 acceptance remains unclaimed; no aggregate gate was run in this reviewer pass.
