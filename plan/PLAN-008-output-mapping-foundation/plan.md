---
id: PLAN-008
title: "Implement the target-neutral output-mapping foundation"
type: Plan
status: done
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-032
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-033
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-034
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/95
    type: references
---
# PLAN-008: Implement the target-neutral output-mapping foundation

## Requirements Summary

### Functional Requirements

- [x] **FR-032**: Strictly admit one exact source/profile/limit-bound mapping request.
- [x] **FR-033**: Dispatch every selected obligation once and derive one valid loss record.
- [x] **FR-034**: Assemble and expose one deterministic package only after every invariant passes.

### Accepted upstream authority

- [x] QSpec FS06 AD-004 and FR-120/121/125/269/297/298/299 are accepted at `a343138`.
- [x] QSpec NFR-060/061 and TC-150/154/155 fix limits, dependency containment,
  determinism, atomicity, and observer separation.
- [x] Contract IR FR-023 supplies strict bound clauses and FR-028 supplies the
  cycle-free owner/model boundary.

## Dependency Graph

- `FR-023 + FR-028 + QSpec FR-120/297/NFR-060 -> TASK-025 / FR-032`
  Reason: target dispatch cannot begin until source clauses, target profile, and
  complete request limits are admitted without defaults.
- `TASK-025 / FR-032 -> TASK-026 / FR-033`
  Reason: mapper candidates and records require the request's exact obligation
  order, target profile, source selections, and budget.
- `TASK-025 + TASK-026 -> TASK-027 / FR-034`
  Reason: atomic assembly requires one validated record for every admitted
  obligation plus the exact request/profile/limit preimage.
- `TASK-027 -> qcir #55/#56/#57 -> #58`
  Reason: independent target mappers consume one stable common seam, and final
  formalization accounting consumes their actual record/package evidence.

### Shared dependencies

`BoundPackage`, `BoundClause`, `ClauseRef`, `SourceSpan`, `DependencyIdentity`,
`CanonicalDigest`, checked arithmetic, and deterministic serialization remain in
`quire-contract-model`. Target mappers add correspondence rules behind the common
trait; they do not own request, loss, package, or identity semantics.

### Cross-cutting constraints

All constructors fail closed, all identity inputs remain typed and
domain-separated, every resource total uses checked arithmetic, and allocation,
cancellation, mapper, record, or assembly failure exposes no package. The model
crate imports no QSL, QObs, Protocol, TL, target parser, filesystem, clock,
locale, network, or foreign runtime.

### The seams

Add one `output_mapping` module to `quire-contract-model` and re-export it through
the existing crate root and `quire-contract-ir` compatibility facade. The module
consumes the existing strict `BoundPackage`; later target modules implement the
bounded mapper trait without importing one another. TC-043 uses a deterministic
test mapper and public constructors only.

## Test Plan

### Unit tests

- [x] Admit all three exact family/profile pairs and reject every missing,
  unknown, cross-family, stale, duplicate, informational, foreign, zero-limit,
  over-limit, and overflowed request input.
- [x] Validate each disposition/source-state/condition/cause/output permutation
  and derive mutation-sensitive record identities.
- [x] Charge every request/node/depth/work/record/output boundary with zero,
  exact, just-over, and checked-overflow cases.
- [x] Validate half-open local and absolute regions, fragment ownership, and UTF-8 boundaries.

### Integration tests

- [x] Execute a multi-obligation deterministic mapper in admitted order and
  expose exactly one record per obligation.
- [x] Replay equal inputs to byte-identical fragments, records, raw digests, and
  package identities; mutate every semantic preimage axis independently.
- [x] Inject cancellation, mapper refusal, malformed candidates, and deterministic
  allocation failures at request, mapping, and assembly boundaries and observe no package.
- [x] Attach absent/accepted/refused observer references and verify package bytes,
  records, dispositions, and identities remain unchanged.
- [x] Compile the cycle-free model independently and prove no owner/target runtime
  or target-specific mapper dependency entered the model crate.

### Verification

- [x] TC-043 carries all FR-032–FR-034 trace tags in executable Rust tests.
- [x] The repository matrix census reports no implemented row without a resolved test symbol.
- [x] PR-time self `/rust-review` and `/gap-analysis` have no unresolved finding.
- [x] Rustfmt, warning-denied workspace/all-target Clippy, and locked workspace
  tests pass on one unchanged head with `target-codex-backends`.

## Remaining Work

### Track H: Common output foundation (serial critical path)

- **H1 = TASK-025** Request/profile admission — Done; only one exact,
  bounded, ordered request reaches a mapper.
- **H2 = TASK-026** Mapper/record accounting — Done; every obligation has
  one invariant-valid, identity-bearing disposition with no default preservation.
- **H3 = TASK-027** Atomic package and closure — Done; deterministic bytes,
  regions, records, identities, resource failures, and observer independence
  pass TC-043 with no partial package.

## Parallel Execution Summary

```text
TASK-025 -> TASK-026 -> TASK-027 -> PR for #95
                                  -> #55 / #56 / #57 (independent target mappers)
                                  -> #58 (cross-target accounting)
```

The common foundation is deliberately serial because each layer fixes the
constructor-private input consumed by the next. Target-specific parallelism
begins only after #95 merges.

## Task File Mapping

| Task | Track | Owns (references) | Verified by (verifies) | Status |
| --- | --- | --- | --- | --- |
| TASK-025 | H | FR-032 | TC-043 | done |
| TASK-026 | H | FR-033 | TC-043 | done |
| TASK-027 | H | FR-034, issue #95 | TC-043 | done |

## Coordination Rules

One owner edits the shared module, TC-043, TM-002, and PLAN-008 through the
single #95 PR. #55/#56/#57 must consume the merged public seam rather than an
unmerged branch or duplicate common types. Generated target text is derived data,
never native source or proof of preservation. Update #52/#55–#58 and QS #1/#9
immediately after #95 merges.
