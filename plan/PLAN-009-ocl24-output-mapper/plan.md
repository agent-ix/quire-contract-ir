---
id: PLAN-009
title: "Implement the bounded OCL 2.4 output mapper"
type: Plan
status: active
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-341
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-342
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-343
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/55
    type: references
---
# PLAN-009: Implement the bounded OCL 2.4 output mapper

## Requirements Summary

### Functional Requirements

- [ ] **FR-341**: Bind every target mapper to the exact canonical admitted request.
- [ ] **FR-342**: Admit one finite, typed, deterministic OCL correspondence catalog.
- [ ] **FR-343**: Emit only the accepted bounded OCL fragment and classify every
  other obligation without approximation.

### Accepted upstream authority

- [x] QSpec FS06 AD-004, FR-120–FR-122, FR-125, FR-269, FR-297–FR-299,
  NFR-060/061 and TC-150/151/154/155 are accepted at `a343138`.
- [x] Contract IR #95 / PR #98 provides the common request, mapper, record,
  package, limits and observer seam at `5ea7730`.
- [x] Local `/specify` revision `9baab67` and subset `/spec-review`
  revision `0a17e89` (base SR-548 plus EARS SR-549) pass.

## Dependency Graph

- `FR-032 + FR-033 -> TASK-028 / FR-341`
  Reason: an exact request identity must be retained and compared by the common
  coordinator before a target mapper can safely use request-scoped correspondence.
- `TASK-028 / FR-341 -> TASK-029 / FR-342`
  Reason: catalog completeness, source membership and model/profile identity are
  meaningful only after one immutable request context is fixed.
- `TASK-029 / FR-342 -> TASK-030 / FR-343`
  Reason: the renderer may emit a target name or preservation condition only
  after its typed correspondence and dependency record are admitted.
- `TASK-030 -> TASK-031 / TC-220 -> qcir #58`
  Reason: integrated record/package evidence and matrix promotion require the
  real mapper's supported and non-preserved outcomes; #58 consumes that producer.

### Shared dependencies

`MappingRequestId` and the request-bound mapper guard are target-neutral shared
infrastructure consumed by #55, #56 and #57. OCL identifier types, catalog
entries, condition/cause codes and renderer logic remain private to the OCL
module and shall not leak into the cycle-free common model or another target.

### Cross-cutting constraints

All construction and mapping totals use checked arithmetic. No invalid catalog,
unsupported expression, non-ready fact, cancellation, allocation failure or
work exhaustion exposes partial records or packages. Rust 1.98.1 is the only
first-party executable language, `publish = false` remains set, and no parser,
JVM, Maven, Eclipse, Electron, network or ambient lookup enters production or
qualification paths.

### The seams

TASK-028 extends `crates/quire-contract-model/src/output_mapping.rs` with a
typed request identity and mandatory mapper binding check. TASK-029 and TASK-030
add a target-specific `src/ocl24/` subsystem behind the existing root facade;
it consumes public `BoundClause`/`TypedExpression` and returns only public
`MappingCandidate` values. TASK-031 drives that mapper through the existing
common coordinator and atomic package assembler in `tests/ocl24_output_mapping.rs`.

## Test Plan

### Unit tests

- [ ] Derive equal request identities for equal admitted inputs and different
  identities for every source-package, obligation, native/model/semantic,
  profile and limit mutation; refuse a mismatched mapper before invocation.
- [ ] Admit catalog permutations to one canonical catalog and reject missing or
  extra obligation entries, duplicate/ambiguous bindings, invalid/reserved names,
  stale/foreign request identity, 10,001 members and 1,048,577 identifier bytes.
- [ ] Render every selected Boolean, comparison, bounded reject-on-overflow
  integer, exact field/state, zero-argument pre/post, Sequence literal/size/
  includes/count/forAll/exists and generated-local case with canonical bytes.
- [ ] Classify native-domain and exact definedness/relationship conditions,
  every unsupported node/clause/profile family, non-ready source state, missing
  correspondence and conflicting correspondence without substitute output.

### Integration tests

- [ ] Run Boolean, bounded ConfigVersion and exact zero-argument operation
  pre/post obligations through request admission, OCL mapping, record accounting
  and atomic package assembly; verify source/anchor/model/profile dependencies.
- [ ] Exercise ConfigVersion values 0, 1000, -1 and 1001 without widening the
  native domain, and exercise ParentPrecedes with complete and absent exact
  relationship/definedness correspondence.
- [ ] Preserve Sequence order, duplicates, maximum cardinality and quantifier
  local scope; refuse index and unsupported transformations.
- [ ] Repeat equal inputs under path/time/locale/observer/parser/tool variation
  and verify byte-identical candidates, records and packages with no authority
  promotion.
- [ ] Inject exact/just-over work budgets, cancellation and common assembly
  failures and verify no partial output population.

### Verification

- [ ] TC-220 backs all 14 FR-341–FR-343 acceptance criteria with real Rust test tags.
- [ ] The scoped Quire coverage census reports no unbacked row or status lie for
  FR-341–FR-343/TC-220 before matrix promotion.
- [ ] Locked workspace/all-target tests, all-feature warning-denied Clippy,
  rustfmt, cargo-deny and unsafe audit pass with literal
  `--target-dir target-codex-backends` where applicable.
- [ ] PR-time self `/rust-review` and `/gap-analysis` have no unresolved finding.

## Remaining Work

### Track O: OCL mapper critical path (serial)

- **O1 = TASK-028** Exact request-bound mapper seam — Medium; exit: a mapper
  cannot run against any semantically different admitted request.
- **O2 = TASK-029** Typed OCL correspondence catalog — Hard; exit: only one
  bounded, canonical, request-complete set of target names/dependencies is usable.
- **O3 = TASK-030** OCL renderer and loss classifier — Hard; exit: the admitted
  fragment emits deterministically and every other case is explicitly non-preserved.
- **Gate = TASK-031** Integrated TC-220 closure — Hard; measures end-to-end
  record/package agreement and traceability; pass: 14/14 criteria backed, all
  local gates green, no approximation or foreign-runtime dependency.

## Parallel Execution Summary

```text
TASK-028 -> TASK-029 -> TASK-030 -> TASK-031 -> PR for #55 -> #58
 request      catalog      renderer     integrated gate
```

The work is intentionally serial within this branch because every layer fixes
the constructor-private contract consumed by the next. Repository-level
parallelism continues in the independently owned Protocol and TL tracks.

## Task File Mapping

| Task | Track | Owns (references) | Verified by (verifies) | Status |
| --- | --- | --- | --- | --- |
| TASK-028 | O | FR-341 | TC-220 | in_progress |
| TASK-029 | O | FR-342 | TC-220 | not_started |
| TASK-030 | O | FR-343 | TC-220 | not_started |
| TASK-031 | O | FR-341–FR-343, issue #55 | TC-220 | not_started |

## Coordination Rules

One owner edits the common mapper seam, OCL module, TC-220, TM-002 and PLAN-009
through the single #55 PR. Do not modify QSpec's accepted FS06 architecture or
another target mapper. Do not edit `resources/native-v1/`, dispatch hosted CI,
or consume unmerged strategy work. Run focused tests while developing and the
full local gate set only at task/PR boundaries. After merge, immediately update
#52/#55/#58 and QSpec #1/#9 with the exact revision and remaining claims.
