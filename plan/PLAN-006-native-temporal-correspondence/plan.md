---
id: PLAN-006
title: "Implement native temporal-to-TL correspondence"
type: Plan
status: active
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-026
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/71
    type: references
---
# PLAN-006: Implement native temporal-to-TL correspondence

## Requirements Summary

### Functional Requirements

- [x] **FR-026**: Construct exact owner-read sibling native/TL temporal requests and join their formula-wide results without parsing or evaluation in Contract IR.

## Dependency Graph

- `FR-025 + QSL FR-051/FR-052 + Quire Observation FR-004 + tl-syntax FR-011/FR-012 + tl-mltl FR-007 -> FR-026`
  Reason: the temporal bridge consumes constructor-private subject, predicate, observation, formula, request, evaluator-result, and mapping contracts from their owners.
- `TASK-017 -> TASK-018`
  Reason: formula and complete position valuations are the common semantic input to both sibling requests.
- `TASK-018 -> TASK-019`
  Reason: result joining is meaningful only after both independently readable requests bind one correspondence.
- `TASK-019 -> TASK-020`
  Reason: closing review audits the complete public projection/read/join/readback surface.

### Shared dependencies

The FR-025 predicate projection and valuation decision are the single shared Boolean-leaf boundary. Both temporal request lanes consume the same admitted rows; neither request is derived from the other's bytes.

### Cross-cutting constraints

FR-026 requires canonical identities, closed typed decisions, bounded work, exact immutable owner selections, no Boolean coercion, and no parser/evaluator invocation within Contract IR across every task.

### The seams

`src/temporal/admission.rs` owns orchestration and selection, `formula.rs` and `valuation.rs` own semantic construction, `request.rs` owns sibling owner inputs, `correspondence.rs` owns the immutable projection, and `join.rs`/`reader.rs` own result comparison and strict re-derivation. `src/predicate/` remains the only Quire Protocol leaf-result boundary.

## Test Plan

### Integration Tests

- [x] **TC-039 future projection/read/join** (FR-026-AC-1/2/3/5/6/8): exercise real QSL, Observation, Protocol, tl-syntax, and tl-mltl views through public construction and strict readers.
- [x] **TC-039 open/fixed-sample/past/activation profiles** (FR-026-AC-1/3/5/7): distinguish online, fixed-sample, origin-complete past, and per-activation guard semantics through both independent evaluators.
- [x] **TC-039 interval and profile boundaries** (FR-026-AC-3/4/8): preserve zero/max bounds and reject unrepresented timestamp profiles without substitute artifacts.
- [x] **TC-039 contract-axis and hostile-input refusals** (FR-026-AC-2/6/8): mutate every selected owner axis and reject missing, replayed, cross-wired, noncanonical, and resource-incomplete inputs.

### Verification

- [x] **Rust gates**: workspace tests, warning-denied all-feature Clippy, rustfmt, release build, unsafe audit, and cargo-deny.
- [ ] **Closing review gate**: `/code-review`, `/rust-review`, and `/gap-analysis` yield no unresolved finding before merge.

## Remaining Work

### Track A: Critical Path (serial)

- **A1 = TASK-017** Semantic projection core — Hard; exit: every supported native occurrence and complete valuation row maps deterministically or fails closed.
- **A2 = TASK-018** Sibling request construction — Hard; exit: real native and TL owner readers admit independently constructed requests sharing one correspondence.
- **A3 = TASK-019** Formula-result join and readers — Hard; exit: exact results agree or preserve a typed non-value/conflict with direct-predecessor validation.
- **Gate = TASK-020** Closing review — measures implementation, Rust, and trace completeness; pass: all required gates are green and no review finding remains unresolved.

## Parallel Execution Summary

```text
TASK-017 -> TASK-018 -> TASK-019 -> TASK-020 -> PR #71 merge
```

The shared owner boundaries made this a deliberately serial critical path; no independent implementation track could change the same correspondence contracts safely.

## Task File Mapping

| Task | Track | Owns (references) | Verified by (verifies) | Status |
| --- | --- | --- | --- | --- |
| TASK-017 | A | FR-026 | TC-039 | done |
| TASK-018 | A | FR-026 | TC-039 | done |
| TASK-019 | A | FR-026 | TC-039 | done |
| TASK-020 | A | FR-026 | TC-039 | in_progress |

## Coordination Rules

Owner APIs and schema bytes remain pinned by merge revision. Contract IR never mirrors owner wire types or evaluates either language. Changes to shared correspondence identity, request axes, or result normalization stay single-writer until TASK-020 closes, and issue #71 is merged only after every review finding is fixed.
