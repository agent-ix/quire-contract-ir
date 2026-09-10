---
id: SR-041
title: "Scope-boundary review of issue 64 native temporal TL correspondence"
type: SpecReview
analysis: scope-boundary
scope: "FR-025, issues 52/57/63/64, and TL semantic-profile boundary"
review_set: all
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-025
    type: reviews
---
# SR-041: Scope-boundary review of issue 64

## Summary

The scope review assigns the correspondence record to Contract IR while leaving
authored language, predicates, observations, temporal evaluation, exports,
evidence, and release decisions with their existing owners. Two ambiguity
paths were closed.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-411 | high | **Closed:** the earlier ADR could still be read as permitting a FRETish authored source. FR-025 explicitly makes native Quire the sole authored clause authority and issue #57 output-only. | ADR-0053; FR-025 Description/Dependencies | wrong-requirement |
| FND-412 | medium | **Closed:** the draft named an evaluator input without saying whether Contract IR invokes it or reads observations. FR-025 now operates purely on validated immutable values and emits no execution verdict. | FR-025 Inputs/Outputs | missing-requirement |

## In-Scope Responsibilities

- Define the v1 supported/unsupported correspondence decision and its exact
  structural join identity.
- Compare every declared semantic/provenance dimension in a deterministic order.
- Admit only exact current TL correspondence and return explicit unsupported or
  existing validation diagnostics otherwise.

## External Dependencies

| Dependency | Assumed or Guaranteed | Contract |
|---|---|---|
| Native Quire frontend/profile | assumed as source authority; correspondence checked | reviewed native profile identity plus TC-038 vectors |
| Issue #63 predicate projection | guaranteed at the consumed boundary | issue #63 reviewed contract and future TC-038 integration |
| Observation/clock authority | assumed valid, identity consistency checked | native profile and supplied immutable observation identities |
| TL syntax/profile/evaluator | guaranteed for admitted correspondence | exact revisions plus independent differential TC-038 corpus |
| FRETish exporter | downstream only | issue #57; no authored-input authority |

## Responsibility Allocation

| Requirement | Owning component | Class |
|---|---|---|
| StR-001 | Contract IR specification | core |
| FR-012 | Contract IR identity model | core |
| FR-023 | Contract IR executable binder | core |
| FR-025 | Contract IR temporal correspondence boundary | core |
| issue #63 | Contract IR predicate bridge | core |
| native temporal meaning | native Quire frontend/specification | external semantic authority |
| formula syntax and evaluation | TL repositories | external semantic authority |
| issue #57 mapping | FRETish output exporter | downstream feature |

Quoin evidence retention, ix-flow human decisions, runtime state acquisition,
and source-release authority remain out of scope.

## Result

**PASS after remediation.** No responsibility is left shared or unallocated in
the reviewed bridge boundary.
