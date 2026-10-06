---
id: SR-1790
title: "Author Rust review — IR-627 owner-free structural keys"
type: SpecReview
analysis: code-review
scope: "quire-contract-ir@5b73113; IR-627 diff against 1540b3b (production, FR-038, conformance and integration fixtures)"
review_set: subset
---

## Summary

Author-authored prehandoff Rust review of the full IR-627 PR1 delta. The owner-free key stage, its group classifier, reader order, FR-038 scope, QSpec checkout harness and migrated fixture builders were checked against the repository conventions and the rust-review checklist. This is not the independent PR review reserved for SR-1780 through SR-1789.

## Verdict

**PASS** — no unresolved Rust finding on the reviewed source tree. The full pre-PR gate remains pending the serialized gate release; QSpec positive owner conformance is a planned row pending QSpec #191 and does not block this owner-free scope. This verdict does not substitute for either check.

## Reviewed Files

Production: `crates/quire-contract-model/src/checked_package/v2/{derived_keys,mod,model_members,structural}.rs`.

Specification and conformance: `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`, `tests/conformance_qspec/main.rs`.

Integration: `tests/it/{checked_package_v2_abstraction_relation,checked_package_v2_canonical_encoding,checked_package_v2_dependency_reference,checked_package_v2_flat_wire,checked_package_v2_frame_bodies,checked_package_v2_frame_entries,checked_package_v2_identity_digests,checked_package_v2_lowering,checked_package_v2_owners,checked_package_v2_parameters,checked_package_v2_reader,checked_package_v2_recursive_leaves,checked_package_v2_require_bounds,checked_package_v2_structural_keys,checked_package_v2_temporal,checked_package_v2_union,complete_v1_checked_package,complete_v1_contract_package,main}.rs` and `tests/it/support/checked_package.rs`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Focused Evidence

On the rebased source tree, 17 TC-226 tests, 10 TC-228 tests, and the selected TC-048 owner-free reader test passed. The group-performance, formatting and scoped Clippy checks also passed. The ignored 12,000-node group fixture admitted at exactly 12,000 work units and refused at 11,999; an earlier controlled comparison restored the old per-candidate component scan and made its three-second ceiling fail after 8.03 seconds. The linear code was restored and rerun green. The wall-clock assertion is confined to the named ignored performance lane.

The review verified that the generic preimage excludes owner-bearing, declared, grouped, nominal, application, model, relation and FR-451 abstraction-relation nodes; IR-630 owns the owner-bearing and grouped follow-up. The new TC-228 witness explicitly records that a changed model-owned function body with its old key still admits under this scope. QSpec positive owner fixtures are not copied into this repository.
