---
id: SR-16453
title: "code-review of PR 531 (IR-992 structural timed model)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@b6e500b0481af5cc4ddd00106a3b87f089600a0d; crates/quire-contract-model/src/checked_package/v2; tests/conformance_qspec/timed_model.rs"
review_set: subset
---
# SR-16453: code-review of PR 531

## Summary

Ticket: IR-992. Exact reviewed head b6e500b0481af5cc4ddd00106a3b87f089600a0d on base 6815d7a. QSpec #425 head 1bb2fb47. This review covers only the PR diff. Full timed semantics remain IR-997. No Cargo aggregate or Kani gate was run by the reviewer.

## Scope examined

- FR-038-AC-318: The public reader admits exactly QSpec FR-322-AC-120's eight `timed_model` forms. Each admitted form retains its published semantic type, occurrence roles, owner, declaration and complete body; an unknown form or malformed outer/body member refuses at strict wire validation before any timed-model body check. The writer emits one complete body per node and no wire identity projection.
- FR-038-AC-319: For a timed model with source, place, reset, guard, invariant, urgency and constraint nodes, graph-shape and structural-key checks compare each node's exact dependencies and QSpec preimage before the timed-model step. Changing one meaning-bearing body or owner member and rebuilding dependent keys changes the affected node and package identities; keeping a stale key or dependency list refuses at the earlier stage. Reordering only source occurrences changes neither identity.
- FR-038-AC-320: The timed-model step reconstructs the selected model from exactly one root and its referenced nodes. A missing root, dangling or wrong-form target, repeated place, noncanonical path, and invalid or non-time quantity each refuse with FR-322-AC-120's paired code and cause at the failing member, visiting timed nodes in ascending node-id digest order. The valid QSpec TC-367 CK-25 model admits and retains its normalized constraints.
- FR-038-AC-321: A package with both a timed-model defect and a protocol or operation defect reports the timed defect; one with a hybrid defect and a timed defect reports the hybrid defect. Strict-wire, graph-shape and stale-key defects continue to precede either body step.
- FR-038-AC-322: The public V2 writer round-trips an admitted timed model through the reader with the same complete node order, semantic identities, selected model and exact normalized quantities, and emits no `identity_preimage.identity_projection` member.

## Verdict

CONDITIONAL. The finding requires an author fix or a documented scope decision before merge. The remaining diff was inspected without another finding from this method.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The duplicate-place set is keyed by target node position. Two clock_place nodes can have the same model and canonical path but different guards, so their node ids and positions differ and both are admitted. Repeated places must be compared by the authored place path, and a distinct-node same-path control is needed. | crates/quire-contract-model/src/checked_package/v2/timed_model.rs:570-580 |
