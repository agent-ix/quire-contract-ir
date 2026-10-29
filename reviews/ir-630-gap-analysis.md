---
id: SR-7121
title: "IR-630 PR 346 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-226-checked-package-v2-anonymous-structural-node-bodies.md"
review_set: subset
---

## Summary

Ticket: IR-630. The computed matrix carries trace tags, but direct test-to-criterion inspection found six partial acceptance controls. Plan completion: not assessed. Optional semantic review was not run as a separate opt-in pass.

## Verdict

**CONDITIONAL** — The named controls need evidence.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The List and Tree admission fixtures lack the SourceOwner and declaration required for source-owned grouped positives; the separate interface test is ungrouped. | tests/it/checked_package_v2_structural_keys.rs:1425 |
| FND-002 | medium | AC-146 has no retained-key lone-label controls for Boolean, Integer or reference, and its range control changes the body as well as the label. | tests/it/checked_package_v2_structural_keys.rs:1587 |
| FND-003 | medium | AC-147 lacks an off-cycle Set node carrying the genuine Tree label with its ordinary retained key, plus a fully rekeyed off-cycle positive. | tests/it/checked_package_v2_structural_keys.rs:1609 |
| FND-004 | medium | AC-148 does not mutate grouped collection_bounds semantic_type while retaining its key; its current Tree mutations cover bounds and sequence target only. | tests/it/checked_package_v2_structural_keys.rs:1520 |
| FND-005 | medium | AC-149 has no independently accounted Tree M/M-1 work boundary, in-group denial with pending work, or continuation mutant control; G1 checks only a final stage budget. | crates/quire-contract-model/src/checked_package/v2/derived_keys.rs:683 |
| FND-006 | medium | AC-150 has no label-only second-pass refusal, first-pass-versus-label precedence, retained-key wire permutation, or fully recomputed permutation control. | tests/it/checked_package_v2_structural_keys.rs:1497 |

## Coverage

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-145 | examined | PLANNED (IR-630); UNRUN. Correctly keyed and labelled record List { next?: List; }, record Tree { kids: Sequence<Tree>[0, 3]; } and the owner-free QSL G1 self-option group admit through both structural-key and group-digest passes. Every structural member is authenticated, including the declared record, option, sequence and collection_bounds nodes. The source-owned positives use actual joined wire owners. An application member retains its separate application key check and contributes its local preimage to the group digest. A reader that still skips any member fails the tamper control |
| FR-038-AC-146 | examined | PLANNED (IR-630); UNRUN; retires ordinary-ungrouped-key-plus-label admission. Boolean, Integer, integer_range and reference retain their existing closed-shape checks and are never skipped because of a label or cycle. Adding a lone label to a correctly keyed ungrouped Int[0, 1000] node while retaining its key, mirroring projection and recomputing package identity now refuses invalid_package/stale-node-key at its node_id, with or without changing max to 10. The same stale-key mutations over Boolean, Integer and reference refuse. Correctly rekeyed and labelled nodes are judged by both p |
| FR-038-AC-147 | examined | PLANNED (IR-630); UNRUN; retires off-cycle-same-label-ordinary-key admission. All nodes sharing a retained label participate in its wire-order group preimages, even off the names cycle. Adding an off-cycle Set<Int[0, 1000]> node carrying the Tree group label and its unchanged ordinary ungrouped key, then mirroring projection and refreshing package identity, refuses invalid_package/stale-node-key at the least first-pass stale node_id; it cannot admit by verifying the extra node as ungrouped. A lone labelled acyclic collection_bounds node with its old key also refuses. Dependencies do  |
| FR-038-AC-148 | examined | PLANNED (IR-630); UNRUN; replaces former residual admissions with refusal regressions. On correctly keyed Tree, keep node keys while separately changing collection_bounds max to 5, min to 1, its semantic_type to another group member, or Sequence<Tree> body reference to another member; mirror projection and refresh package identity. Each refuses invalid_package/stale-node-key under the two-pass order. Forge a self-option by starting from a correctly keyed ungrouped Option<Integer>, changing its body reference to its own retained node_id and adding a label, retaining the old key, mirro |
| FR-038-AC-149 | examined | PLANNED (IR-630); UNRUN. Group membership, every structural-key visit, local-preimage term visits and member/group digest computations are charged before work as specified above, including owner-bearing and off-cycle labelled members. For a fully valid Tree read, independently account for these visits and find the exact smallest total work M that admits; work M-1 returns work-incomplete at the first unpaid value with limit, consumed count and pointer, and exposes no package. Separately, select a charge inside group-local preimage or group-digest work on the valid Tree, with at least  |
| FR-038-AC-150 | examined | PLANNED (IR-630); UNRUN; the prior owner gate and SCC skip are retired. First authenticate each structural node in ascending retained node-id digest order using actual joined owner/declaration, null recursion outside a labelled group, or its wire-order group-local preimage with recursion.group equal to the retained label. Then recompute every label from all member-local preimage digests in wire ordinal order. A stale node key reports invalid_package/stale-node-key at that node_id; a stale label reports that refusal at the least retained member, with failing groups ordered by least me |
| TC-226 | examined | IR-630 owner-bearing, declared and labelled-group controls are PLANNED and UNRUN. |

Plan completion: not assessed. The computed matrix was inspected; static trace presence does not discharge the specific behavior controls above.

## Dispositions

Round 1 reviewed.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed in the reviewed PR. |
| FND-002 | fixed | Fixed in the reviewed PR. |
| FND-003 | fixed | Fixed in the reviewed PR. |
| FND-004 | fixed | Fixed in the reviewed PR. |
| FND-005 | fixed | Fixed in the reviewed PR. |
| FND-006 | fixed | Fixed in the reviewed PR. |
