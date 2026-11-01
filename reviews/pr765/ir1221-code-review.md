---
id: SR-202610310124
title: "IR-1221 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1221; typed fairness target union and related reader/test/spec changes"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Independent review of the typed fairness target change for IR-1221 / PR #765. Reviewed the canonical union, closed decoder, dependency and identity walks, temporal/protocol/hybrid joins, authored controls, owner vectors and changed requirements.

## Verdict

FAIL. The changed refusal ordering violates the reader contract despite passing focused controls. No source edits or Cargo commands were run by this reviewer.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Protocol fairness target joins run after profile fit instead of in the temporal fairness-resolution slot. A missing or foreign protocol target combined with a bounded-profile fairness or interval-fit defect returns operation-member-mismatch first, violating the required fairness-before-fit order. | crates/quire-contract-model/src/checked_package/v2/temporal.rs:908 |

## Examined scope

- FR-038-AC-98 (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: The `fairness` member of `quire.op.temporal.fair` admits as `{kind: fairness, fairness_kind: weak, granularity: whole, target: {kind: operation, declaration, name}}` and as `strong` with `each`, with a `declaration` and `name` that resolve (the resolution refusals are FR-038-AC-103's); a `fairness_kind` of `medium`, a `granularity` of `part`, an operation target lacking `name`, a member holding an extra member, a `null` member and a member of kind `temporal_interval` each refuse `invalid_package`/`operation-member-mismatch` at `operation.member`; the closed protocol-root, attempt, parallel-bra
- FR-038-AC-102 (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: The temporal step runs after the frame and state-clause step and before the operation step, placement first: a package holding one node with a placement defect (a `temporal_formula` application as the body root of a `function` node) and a second with a earlier structural `node_id` whose operation identity is unknown refuses for the placement defect, and so does the same placement defect beside a earlier-structural-id node whose `operator` differs from its entry's class (`quire.op.control.case` under `unary`), in both structural id orders each, the placement defect taken in the shape of a `temp
- FR-038-AC-103 (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: A clause whose `over` argument is a `reference` to a `value`/`parameter` node among its `dependencies` admits; one whose `over` references a `value`/`parameter` node that is not among its `dependencies` refuses `missing_declaration`/`missing-name`, and one that references a declared dependency that is a `scalar_type` node refuses `invalid_model_binding`/`malformed-declaration`, each with path `/semantic_graph/nodes/{clause}/body/arguments/0` and the locus of the "Path and locus" table; a fairness member whose `declaration` and `name` resolve to an operation of a `model`/`object_type` declarati
- FR-349-AC-9 (examined), spec/checked_package/functional/FR-349-retain-typed-checked-v2-terms.md: The FR-019 exact public-item inventory and external compile probes show that IR exports the admitted read views but no QSL checked-expression constructor, `TermOperation` mirror, checker, or evaluator. Dependency-graph checks show that `quire-contract-model` does not depend on `quire-checked` or QSL and `quire-checked` does not depend on IR.
- FR-019 (examined), spec/model/functional/FR-019-rust-library-interface.md: | `checked_package` (V2 typed read views) | types `CheckedBindingName`, `CheckedBindingNameKind`, `CheckedLiteralRead`, `CheckedLiteralValue`, `CheckedOperationRead`, `CheckedOperationMember`, `CheckedFieldReadForm`, `CheckedOperationConstraint`, `CheckedOperationLeaf`, `CheckedOperationLaw`, `CheckedOperationMode`, `CheckedOperationInterval`, `CheckedOperationRational`, `CheckedIntervalEnd`, `CheckedFairnessKind`, `CheckedFairnessGranularity`, `CheckedFairnessTarget`, `LawRole`, `OperationMemberKind`, `OperationConstraintKind` | FR-349 |

## Evidence and limits

Parent-owned focused receipts show 61 passing temporal/typed/protocol reader tests, workspace all-target Clippy with warnings denied completed successfully, and one external canonical-target vector test passed. This reviewer inspected the outputs and source; no Cargo or Kani command was run. CI workflows and manifests are unchanged. Source review found no shim, mirror target type, QSL dependency, unsafe block or evaluator. The manually streamed target decoder preserves direct use of the canonical node-id decoder and rejects duplicate, missing and unknown fields. No active repository AssuranceProfile was found; no assurance-profile claim is made.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Protocol target joins now run at the temporal fairness slot using the factored canonical protocol membership collector and the same target validator; combined profile/interval defect controls pass. |

## Same-review fix audit

All original findings are repaired in the same PR. No additional independent review or Cargo execution was performed. Pre-rebase parent-owned final receipts show 237 focused reader/inventory tests, four hybrid conformance tests and one catalog unit test passed, with workspace all-target Clippy and formatting passing. The new public declaring-node read follows the canonical nested target only; no flat fairness fallback is retained. Hybrid fixture composition relocates published graph-local indexes against the complete base/restored source table and sorts dependencies by the existing canonical node resolver; it does not copy a schema or add a production shim.

Current merge status is CONDITIONAL on post-rebase integration checks. The first post-rebase attempt reports E0026 in the newly introduced iterative identity walker, which still matches the retired flat fairness declaration field. Parent has been notified to update that integration site and rerun the focused gates. Original findings are fixed, but these red post-rebase gates are not accepted as completion evidence.

## Rebase integration audit

The new iterative identity-walker integration now maps the nested target identity once and synchronizes both wire payload and borrowed typed member. Only operation/hybrid targets populate the declaring-node view. The new six-variant unit control asserts exactly two callback visits (target and result), exact wire/view equality, and declaring-node presence or absence. The retired flat fairness field match is removed. Source fix audited; final post-rebase receipt inspection remains pending.

## Final validation

PASS. All original findings remain fixed, and the post-rebase integration gates now pass. Inspected final focused output: 237 reader/inventory tests passed, including all touched protocol/temporal/typed tests, TC-058/TC-456 and the explicitly enabled eleven owner refusal vectors. Four hybrid/canonical-vector tests passed, including borrowed HybridTransition data and wrong declaration/name controls. Both catalog and six-variant identity-mapping unit tests passed. Workspace all-target Clippy completed with warnings denied; formatting output is empty with parent-reported exit zero. The final delta imports typed vocabulary for the unit controls only. No further source finding arose in this same-review fix audit. No Cargo or Kani command was run by this reviewer.
