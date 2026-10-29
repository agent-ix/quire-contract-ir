---
id: SR-8423
title: spec-review/integrity review of admitted traversal view
type: SpecReview
analysis: integrity
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md,
  spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md; FR-035-AC-6..8
review_set: subset
---

## Summary

Ticket: IR-766. Reviewed spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md and spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md. AC-7 permits omission of bound and claim subsets even when lowering derived them. The other changed criteria have discriminating adverse cases and the planned TC identifies their expected outcomes.

## Verdict

**CONDITIONAL** — AC-7 needs a required positive bound-and-claim case.

## Examined units

- FR-035-AC-6: For an admitted lowered root, the typed view returns that exact root and each local node reached through `semantic_type`, declared `dependencies` or validated body references once in ascending typed-key order; independently walking those three edge sources yields the same key set, including a shared or cyclic reference, and every returned node retains its exact source correspondence.
- FR-035-AC-7: A selected dependency reference in the view retains its selected dependency identity and target key and cannot be satisfied by an identically keyed local node or an absent, stale or wrong selected package; any bound and claim key subsets exposed for the lowered root equal the subsets already derived by Contract IR lowering and are in typed-key order.
- FR-035-AC-8: Querying a non-`lowered` requested key returns its exact typed disposition, while an unrequested or absent key returns typed absence even if its node occurs only as another root's reached meaning; neither returns a fabricated closure, and querying the view changes none of the six FR-330 wire properties or identity.
- TC-047: From a genuinely admitted mixed graph, query the typed view for a lowered root whose local closure uses `semantic_type`, declared `dependencies`, and body references, including a shared node and an explicit recursion group. Independently enumerate those validated edges and compare the reached keys, node values and source-map entries with the view in typed-key order. Include an adversarial body reference whose target is omitted from a declared dependency list, so a walk limited to that list fails the comparison. Check that any bound and claim key subsets agree with the existing IR lowering resu

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Bound and claim subsets may always be omitted despite derived values | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:117 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | — FR-035-AC-7 now requires both subsets for every lowered root; TC-047 checks exact nonempty and empty cases against independently derived results. |
