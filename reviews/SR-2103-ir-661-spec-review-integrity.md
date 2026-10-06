---
id: SR-2103
title: "Integrity analysis \u2014 IR-661 FR-038/TC-048 relationship reader amendment"
type: SpecReview
analysis: integrity
scope: agent-ix/quire-contract-ir@705cef3f7a07370f111f19e8e7d86e1e07fd31d7; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; base 1540b3b6c0e4d167fe1ed9116c296e45e7dff258; Ticket IR-661
review_set: subset
---

## Summary

Consistency and atomicity of the amendment against FR-038's existing owner-recovery steps and merged QSpec FR-322 (60630b0). The relationship-node mapping for `relationship_end.declaration` is supported: QSL FR-094 and FR-322's published `model-reaches` vector name a `relation`/`relationship` node. Two defects: the object-node refusal rests on a recovery step FR-038 does not define, and `reaches` over the new mapping is left without a rule.

Ticket: IR-661. Method: `spec-review/spec-integrity-analysis`. Reviewer model `claude-opus-5-5`, run `236a8098-415e-43ce-aa4b-9a892aa7e251`.

## Verdict

**FAIL**: one `high` and one `medium` finding. Clean, as examined: the mapping to the relationship graph node (single mapper, no endpoint-object alternative); declaration-refusal order matching FR-154's table; the missing-name-before-receiver/direction/multiplicity precedence and its TC coverage (steps 5, 7, 8); a present inverse that does not override declared `source-to-target`; selection-row pointer and metadata for declaration refusals; operation paths under `body/operation/member`, `body/arguments/0` and `body/result_type`; no new limit.

## Scope Examined

- `FR-038#identity-slot` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:975-981
- `FR-038#relationship-ends` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:992-1000
- `FR-038#relationship-end-mapping` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1010-1016
- `FR-038#object-node-declaration` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1021-1027
- `FR-038#role-lookup` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1029-1037
- `FR-038#receiver-endpoint` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1039-1047
- `FR-038#navigation-derivation` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1046-1054
- `FR-038#other-operations` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1058-1063
- `FR-038#declaration-order-accounting` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1065-1072
- `FR-038#operation-order-paths` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1078-1086
- `FR-038-AC-165` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3387
- `FR-038-AC-166` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3388
- `FR-038-AC-167` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3389
- `FR-038-AC-168` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3390
- `FR-038-AC-169` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3392
- `FR-038-AC-170` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3393
- `FR-038-AC-171` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3394
- `TC-048#fcd-step-5` (examined) spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:992-1000
- `TC-048#fcd-step-5-object-node` (examined) spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1006-1014
- `TC-048#fcd-step-8` (examined) spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1043-1050
- `FR-038#step-2-owner-recovery` (context_only) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:963-971
- `FR-038-AC-155` (context_only) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3379
- `operations.rs relationship_end scope note` (context_only) crates/quire-contract-model/src/checked_package/v2/operations.rs:31-38
- `QSpec FR-322 Model-owned members step 2` (context_only) agent-ix/quire-specification@60630b0d3d5e9cca048d1c314675db3bf9c6e4f2:spec/objects/interfaces/FR-322-checked-package-artifact.md:634-642
- `QSpec FR-322 Model-owned members step 3` (context_only) agent-ix/quire-specification@60630b0d3d5e9cca048d1c314675db3bf9c6e4f2:spec/objects/interfaces/FR-322-checked-package-artifact.md:640-647
- `QSpec FR-322-AC-30` (context_only) agent-ix/quire-specification@60630b0d3d5e9cca048d1c314675db3bf9c6e4f2:spec/objects/interfaces/FR-322-checked-package-artifact.md:998
- `QSpec FR-322 Reaches over a field` (context_only) agent-ix/quire-specification@60630b0d3d5e9cca048d1c314675db3bf9c6e4f2:spec/objects/interfaces/FR-322-checked-package-artifact.md:748-750
- `QSL FR-094 Model declaration nodes` (context_only) agent-ix/quire-spec-language@bdb910ad5ac54e40d22c23480d52cce13c57edb3:spec/functional/FR-094-key-model-owned-reference-population-and-quantity-nodes.md:111-115

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Object-node declaration refusal relies on an undefined expected-kind recovery; contradicts step 2 | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1021-1027 |
| FND-002 | medium | reaches over the new relationship-node mapping has no stated receiver or end rule and no AC | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1058-1063 |

## Finding Detail

- **FND-001** (high, confidence medium, check `soundness`, unit `FR-038#object-node-declaration`): A valid `model`/`object_type` node used as `relationship_end.declaration` is given `missing_declaration`/`missing-selection` at `member.declaration` by an "existing expected-kind recovery". FR-038 defines no such recovery. Its own step 2 ("the declaring node's key is matched to one selected declaration's model declaration node key") recovers that object node's object type. Merged QSpec FR-322 step 2 "has no refusal of its own" and keys `T` by the node's own `node_tag`. FR-322-AC-30 refuses the mirrored case (a field or operation member naming a relationship declaring node) `ill_typed`/`operator-ineligible`. The operation-order paragraph also calls this recovery "existing", but `operations.rs` today checks `relationship_end` for presence and kind only. Readers implementing FR-038 as written can return either refusal pair.
- **FND-002** (medium, confidence medium, check `ambiguous`, unit `FR-038#other-operations`): Role lookup and the declaration-kind rule apply to every `relationship_end` member. For `quire.op.model.reaches`, the amendment says only that it "retain[s] [its] catalogued operands and result form". Merged FR-322 defines `reaches` as "the same predicate over a relationship end" as `reaches_field`. Its first condition is that operand 0 has type `Reference<D>` with D's model declaration node equal to the member's `declaration`. With `declaration` now a relationship node, that condition can never hold. Which end `reaches` follows, what it checks on its operands, and what it refuses are left open. No new AC or TC procedure exercises `reaches`.

## Dispositions

Round 1, reviewed at `agent-ix/quire-contract-ir@c50050da5f20d4af41ab2dd6ea573d5e4b9abacc` (prior review `705cef3f7a07370f111f19e8e7d86e1e07fd31d7`), reviewer model `claude-opus-5-5`, run `ae1a4470-06bc-44c7-90df-81c70be53842`. Every outcome was verified against the fix commit's own text, not the author's receipt. All new criteria and procedures remain PLANNED / UNRUN; no implementation, CI gate or procedure coverage is claimed.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed c50050da5f20d4af41ab2dd6ea573d5e4b9abacc | Step 2 now recovers a valid object owner normally. A new relationship-member kind check after recovery refuses `ill_typed`/`operator-ineligible` at `member.declaration`, before role lookup. That matches FR-322-AC-30's cross-kind code and cause, the AC-155 owner join is unchanged, and the order paragraph no longer claims today's reader performs it. |
| FND-002 | fixed c50050da5f20d4af41ab2dd6ea573d5e4b9abacc | `reaches` now uses the same relationship-node mapper. The resolved receiver endpoint is the static edge owner, checked by reference against FR-322 reference-edge admissibility and FR-043's static-edge conformance and homogeneous traversal (merged FR-043 Behavior: "a and b are Reference<S> values whose types conform to the owner T of edge e"). The result is Boolean. AC-172 and TC-048 step 10 add positive and adverse cases. |
