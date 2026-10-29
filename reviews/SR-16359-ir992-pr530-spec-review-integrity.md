---
id: SR-16359
title: "spec-review-integrity of PR 530 (IR-992 timed-model reader order)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---
# SR-16359: spec-review-integrity of PR 530

## Summary

Ticket: IR-992. At QSpec #425, the owner schema fixes the outer semantic type, eight closed bodies and the hybrid → timed-model → protocol order. IR AC-318..322 are consistent with those owner rules; no integrity defect was found.

Reviewed exact IR against QSpec #425. Spec-only review; no Cargo or Kani gate run.

## Scope examined

- FR-038-AC-318: The public reader admits exactly QSpec FR-322-AC-120's eight `timed_model` forms. Each admitted form retains its published semantic type, occurrence roles, owner, declaration and complete body; an unknown form or malformed outer/body member refuses at strict wire validation before any timed-model body check. The writer emits one complete body per node and no wire identity projection.
- FR-038-AC-319: For a timed model with source, place, reset, guard, invariant, urgency and constraint nodes, graph-shape and structural-key checks compare each node's exact dependencies and QSpec preimage before the timed-model step. Changing one meaning-bearing body or owner member and rebuilding dependent keys changes the affected node and package identities; keeping a stale key or dependency list refuses at the earlier stage. Reordering only source occurrences changes neither identity.
- FR-038-AC-320: The timed-model step reconstructs the selected model from exactly one root and its referenced nodes. A missing root, dangling or wrong-form target, repeated place, noncanonical path, and invalid or non-time quantity each refuse with FR-322-AC-120's paired code and cause at the failing member, visiting timed nodes in ascending node-id digest order. The valid QSpec TC-367 CK-25 model admits and retains its normalized constraints.
- FR-038-AC-321: A package with both a timed-model defect and a protocol or operation defect reports the timed defect; one with a hybrid defect and a timed defect reports the hybrid defect. Strict-wire, graph-shape and stale-key defects continue to precede either body step.
- FR-038-AC-322: The public V2 writer round-trips an admitted timed model through the reader with the same complete node order, semantic identities, selected model and exact normalized quantities, and emits no `identity_preimage.identity_projection` member.
- FR-038 timed-model reader prose and fixture-path instruction (examined).
- FR-019 V2 vocabulary public inventory: `TimedModelForm` added (examined).
- QSpec FR-322-AC-120 and Body steps at PR #425 (context only).
- QSpec checked-package and node-identity-preimage schemas at PR #425 (context only).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
