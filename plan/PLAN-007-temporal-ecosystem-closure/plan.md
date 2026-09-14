---
id: PLAN-007
title: "Close the temporal ecosystem and export its bounded model"
type: Plan
status: complete
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-027
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/74
    type: references
  - target: ix://agent-ix/tl-syntax/Task-011
    type: references
---
# PLAN-007: Close the temporal ecosystem and export its bounded model

## Requirements Summary

### Functional Requirements

- [x] **FR-027**: Strict-read the exact nine-repository campaign manifest and export/re-read one deterministic bounded non-authoritative ecosystem model.

### Campaign closure

- [x] **FR-025/Task-006**: Native predicate projection is merged.
- [x] **FR-026/Task-007**: Native temporal correspondence and result joining are merged.
- [x] **PLAN-010 Task-011 QCI allocation**: Execute the public owner path end to end, publish the model, reconcile all status, and close every review finding.

## Dependency Graph

- `merged Task-009 TL owners + Task-010 Quire owners + FR-025 + FR-026 -> TASK-021`
  Reason: the manifest must select delivered immutable repository and contract revisions rather than anticipated interfaces.
- `TASK-021 -> TASK-022`
  Reason: relation validation operates only on an admitted complete node population.
- `TASK-022 -> TASK-023`
  Reason: export may derive adjacency and topological order only from the validated typed graph.
- `TASK-023 -> TASK-024`
  Reason: end-to-end and closing reviews exercise the complete public manifest/export/read surface on unchanged code.

### Shared dependencies

Canonical JSON, domain-separated identities, contract selections, limits and diagnostics remain in `bridge`. The ecosystem subsystem describes owner artifacts and exact revisions but imports no parser, evaluator, network client, repository scanner or owner-private wire type.

### Cross-cutting constraints

All work is Rust, deterministic and caller-lowerable beneath owner maxima. Untrusted documents fail closed before exposing partial views. Model and improvement-proposal output has no acceptance, authority or execution role. Exact revisions and schemas are data selected by the manifest, never discovered from ambient repositories.

### The seams

`ecosystem_model::manifest` owns schema and admission; `graph` owns relation typing, uniqueness and topology; `document` owns deterministic derived output and identity; `reader` owns re-export byte equality. Public constructors exist only for authored manifest bytes and deterministic model export, while checked/validated views remain constructor-private.

## Test Plan

### TC-040 model contract

- [x] Exact nine-repository manifest and permutation-invariant export with mutation-sensitive identity.
- [x] Duplicate, missing, dangling, multi-owner, ill-typed, self-forbidden, cyclic and moving-revision refusals.
- [x] Strict manifest/model unknown, duplicate, trailing, noncanonical, cross-campaign, identity, count, adjacency and topology refusal.
- [x] Exact/one-over byte, depth, string, every node population, edge, work and deterministic allocation boundaries.
- [x] Compile-time constructor/privacy and authority-boundary assertions; proposal identity has no acceptance state.

### End-to-end integration

- [x] Replay the delivered future and origin-complete past owner-reader path through predicate projection, temporal request construction, independent owner evaluation and structural joining under the manifest's exact selections.
- [x] Preserve every exercised typed non-success state and correction relation without Boolean fallback.

### Verification

- [x] Workspace tests, warning-denied all-feature Clippy/rustdoc, rustfmt, release build, unsafe/stub audit, cargo-deny and cargo-audit.
- [x] `/code-review`, `/rust-review`, `/gap-analysis`, and `/spec-architecture-evaluation` have no unresolved finding before merge.

## Remaining Work

### Track G: Ecosystem closure (serial critical path)

- **G1 = TASK-021** Manifest contract/admission — Hard; exit: exact complete selections strict-read or return one typed decision without partial state.
- **G2 = TASK-022** Typed graph validation — Hard; exit: all node/edge/ownership/topology invariants are enforced deterministically.
- **G3 = TASK-023** Model export/read — Hard; exit: canonical model bytes, identity, adjacency and topology rederive exactly and remain non-authoritative.
- **G4 = TASK-024** End-to-end closure — Hard; exit: TC-040 and owner-path integration pass; reviews, matrices and tickets match merged evidence.

## Parallel Execution Summary

```text
TASK-021 -> TASK-022 -> TASK-023 -> TASK-024 -> PR #74 -> PLAN-010 Task-011 closure
```

The graph and document layers have a strict semantic dependency. Tests may be authored alongside their owning layer, but no layer is accepted or deferred independently from the complete feature.

## Task File Mapping

| Task | Track | Owns (references) | Verified by (verifies) | Status |
| --- | --- | --- | --- | --- |
| TASK-021 | G | FR-027 | TC-040 | done |
| TASK-022 | G | FR-027 | TC-040 | done |
| TASK-023 | G | FR-027 | TC-040 | done |
| TASK-024 | G | FR-027, Task-011 | TC-040 and integration | done |

## Coordination Rules

Implement and review the full plan before merge. Preserve exact merged owner identities, use public owner readers only, and do not create qualification artifacts. Update issue #74 and the PLAN-010 umbrella only from committed/merged evidence.
