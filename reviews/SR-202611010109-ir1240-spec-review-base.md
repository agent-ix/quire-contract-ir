---
id: SR-202611010109
title: "IR-1240 PR 792 FR-048 base spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; IR-1240 PR #792; spec/model/functional/FR-048-derive-structural-and-group-candidates.md"
review_set: subset
---

## Summary

Reviewed the amended FR-048 behavior and AC-6/7 against the closed operation-member model types and computed test matrix. The named structural ID sites and callback order are concrete, but the existing AC-6/7 tags do not verify the newly required nested operation-member case.

## Verdict

**CONDITIONAL** — mark the new nested-member obligation as planned until the IR-1240 code tests land.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-6/7 gain nested operation-member obligations while existing tagged tests exercise no operation member; record the new coverage as planned/unrun until its external-consumer test lands. | FR-048-AC-6, FR-048-AC-7; tests/it/checked_package_v2_node_map.rs:84 |

## Coverage

- FR-048 Behavior, examined: operation-member slot names, nested whole-ID handling, callback order, and raw/typed mirror rule match the model's closed `CheckedOperationMember` and `CheckedFairnessTarget` variants.
- FR-048-AC-6, examined: None/Some, non-cascade, distinct equal slots, and nested whole-ID member sites are verifiable.
- FR-048-AC-7, examined: the named variant census, mirror, visit order, and unchanged non-ID data are verifiable.
- FR-048-AC-8, context_only: existing malformed replacement and 100000-node bounds still own the adverse and resource cases.
- `quire matrix --scope . --format tsv` calls AC-6 and AC-7 tagged. `tests/it/checked_package_v2_node_map.rs` has the corresponding trace tags but no Application operation or nested operation-member ID case; the new behavior awaits IR-1240 code.

## Evidence

Scoped `quire validate --scope . spec/model/functional/FR-048-derive-structural-and-group-candidates.md --summary` passed with 0 grammar findings. The spec introduced no hash, digest, version pin, new dependency edge, or compatibility layer. No applicable AssuranceProfile was found under `spec/`.
