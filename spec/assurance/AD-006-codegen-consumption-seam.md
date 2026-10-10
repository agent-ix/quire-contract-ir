---
id: AD-006
title: "IR to codegen seam: the checked-package and Kani types codegen consumes"
type: ArchitectureDescription
status: proposed
owner: kreneskyp
system: quire-contract-ir root crate and quire-contract-model public items that quire-contract-codegen imports, the import path it uses, and the Cargo edge between the two repositories
relationships:
  - target: ix://agent-ix/quire-contract-ir/AD-001
    type: references
  - target: ix://agent-ix/quire-contract-ir/AD-002
    type: references
  - target: ix://agent-ix/quire-contract-ir/AD-003
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-030
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-039
    type: references
---
# IR to codegen seam: what codegen consumes

Codegen is IR's one consumer that imports through the root crate. This AD states the seam in
full: what crosses, which path it crosses by, who owns each item, how identity is asserted, what
holds the direction, and what fails where. It is one of the seam descriptions of IR-323. The
reader and its refusals are AD-004; the Kani profile is AD-002; the replay and evidence chain on
codegen's side are codegen's AD-002 and AD-003. None is restated here.

## System Boundary

IR owns the reader, the lowering types, the authored-contract model and the shared Kani contract
(profile, finite input, dispatch, outcome). Codegen owns generation: oracles, harnesses, Kani
obligations and the Kani family lowerings. Codegen depends on IR; IR depends on nothing of
codegen's.

Out of scope: how codegen uses what it reads (codegen's own specification) and the runtime seam
(`quire-contract-runtime` AD-003).

## Views

The seam is described as what crosses it, how identity is asserted, which way dependencies point
and who reports each failure.

### What crosses the seam

Measured at the commits of codegen and IR this AD was written against.

| Group | Items | Owner | Codegen files that name them |
| --- | --- | --- | --- |
| Checked-package v2 and lowering | `CheckedPackageV2`, `CheckedNodeId`, `CheckedNodeTag`, `CheckedSemanticId`, `CheckedSemanticNodeV2`, `CheckedSourceMapEntry`, `CompleteContractNodeV2`, `CompleteLoweringProfileV2`, `CompleteLoweringRecordV2` | IR (FR-035, FR-038; wire is QSpec's) | `composite_equality.rs`, `exact_function.rs`, `exact_scalar.rs`, `state_frame.rs`, `routed_generation.rs`, `generation.rs`, `kani_obligations.rs` |
| Authored-contract projection | `BoundPackage`, `BoundClause`, `ClauseRef`, `ClauseKind`, `Expression`, `ExpressionKind`, `TypedExpression`, `StateObservation`, `DependencyIdentity`, `SourceSpan` and related | IR (FR-011 to FR-023) | `bound.rs`, `bound_coverage.rs`, `bound_strategy/generation.rs`, `oracle.rs`, `harness.rs`, `kani.rs`, `strategy.rs`, `kani_obligations.rs` |
| Shared Kani contract | `PROFILE` `kani-bounded/1`, `KaniProfile`, `ProfileSelection`, `FiniteInput` family, `DispatchIndex`, `KaniOutcome`, `KaniOutcomeKind` | IR (FR-029 to FR-031) | `bounded_kani_profile.rs`, `bounded_kani_corpus.rs`, `bounded_collections.rs`, `finite_reference_graphs.rs`, `definedness_arithmetic.rs`, `kani_execution.rs` |
| Kani family lowerings | `lower_checked_arithmetic`, `lower_query`, `lower_reaches` and their request and result types | declared codegen's (AD-001, FR-039 "Items codegen owns"); in IR's source today | `definedness_arithmetic.rs:19`, `bounded_collections.rs:18`, `finite_reference_graphs.rs:18` are one-line calls into them |
| Lowering request from codegen | `CompleteLoweringProfileV2` (`supported_tags`, `require_bounds`, `work_limit`), built by each generator (for example `composite_equality.rs:670`) | codegen supplies, IR defines the type | one profile per generator |

What does not cross: no runtime type (the two repositories have no edge), and, as a target, no
replay, witness or terminal-record type (AD-005, AD-001). IR-347 removed the former
`KaniProviderResult` and `KaniProviderRecord` exports and `provider_result` map; codegen's source
used none of them.

### The import path

Codegen's manifest now declares both IR packages: `quire-contract-model` for model items and
`quire-contract-ir` for the root `kani` interface. Codegen imports model items through
`quire_contract_model`, and IR-347 removed the root crate's former
`pub use quire_contract_model::*` glob (AD-001, FR-039).

### Identity and versions on this seam

- Cargo asserts the dependency: `=0.1.0` and a branch, with the commit in codegen's lock. Both IR
  packages resolve in codegen's `Cargo.lock` at one commit. No version record is kept and none is
  proposed.
- The Rust contract is the compiler. IR has no `#[non_exhaustive]` on any public enum (zero
  occurrences in `src/` and `crates/`), so an added IR variant stops codegen's exhaustive matches
  from compiling. That is the intended failure: loud, at build time.
- The Kani profile carries its own identity, `kani-bounded/1` (`src/kani/mod.rs:29`), selected
  together with its capability matrix, input ABI and module revisions (AD-001, Versioned
  contracts). Codegen selects by `ProfileSelection` and IR refuses a selection it does not know.
- The checked-package seam asserts `contract_version` and `package_id` (AD-004). Codegen does not
  recompute either; it reads an admitted `CheckedPackageV2`.
- IR-605 changed `KaniOutcome.code` to the model crate's `Std001Code`; codegen compares the
  registered `Std001Code::KANI_VACUOUS_PROOF` constant. The family lowerings still in IR build
  unregistered codes with `std001_code!` until their move to codegen under IR-347.

### Dependency direction and what enforces it

| Edge | Allowed | Held by | Gap |
| --- | --- | --- | --- |
| codegen to IR | yes, root crate and model crate | codegen `Cargo.toml`; codegen `make deny` checks one revision of each IR package | none for the model import path |
| IR to codegen | no | nothing in IR yet; AD-005 decision D makes it a cargo-deny `bans` failure (IR-343) | the forbidden list in `tests/it/cycle_free_model.rs` does not name `quire-contract-codegen` |
| IR to QSL | no | `tc_041` today; target: cargo-deny `bans` entries as well (AD-005 decision D) | no deny entry exists yet |
| codegen to QSL | only `qsl-replay` | codegen `deny.toml` exceptions and QSL's `arch-lint api-surface` T12-A | not an IR concern |
| IR and codegen both to runtime | IR no, codegen yes | AD-001 for IR (runtime: no edge in either direction with IR) | no IR test (AD-005 D-2) |

### Failure outcomes and who reports them

| Condition | Reported by | Outcome |
| --- | --- | --- |
| Checked package refused, or a limit reached | IR (AD-004) | typed refusal or `Incomplete`; codegen reports no claim for the package |
| One requested node cannot lower | IR | one of seven record kinds (AD-004); codegen turns each into its claim map entry or a refusal, and never skips a request |
| Kani profile selection or finite input refused | IR | `ProfileError`, `DispatchError`, `KaniOutcome` of kind `Refused`, `InvalidInput`, `IncompleteInput` |
| A run ends without a proof or a counterexample | IR's `KaniOutcomeKind` names it; codegen's parser decides which kind | codegen maps to a terminal value (codegen AD-003; IR does not map) |
| IR adds or changes a public item | the compiler, in codegen's build | build failure in codegen, not a runtime defect |

## Decisions

- A. Target: codegen reads IR through the model crate for model items and the root crate for the
  `kani` module only. Order: codegen first adds the direct `quire-contract-model` dependency, then
  IR removes the root glob (the order the planner relays for IR-347); nothing sits between.
- B. Every Kani family lowering lives in codegen. IR keeps the profile, finite input, dispatch
  index and outcome, which any backend can read. Order: codegen takes the lowerings over first,
  then IR deletes them, with no copy left in IR and no re-export. The move is IR-347 scope (the
  IR planner's relayed decision). The final form is this one, and CG's ADs agree.
- C. IR's `KaniOutcome` is built only through its constructors (FR-030-AC-5), and the cause codes
  codegen matches are `Std001Code` constants exported by `quire-contract-model`, not literals on
  both sides (FR-044, FR-030-AC-6; IR-605). Codegen's own refusal codes (`kani_corpus_*` and
  `kani_profile_input_mismatch`) are built with `std001_code!`; codegen has no registry document
  for them today, and writing one is codegen's to decide. Neither is built today.
- D. Order across the two repositories: a consumer first gains what it needs, then IR removes the
  old item. No deprecated item or shim sits between the two steps.

### Invariants a test can check

Local labels; the repository assigns requirement ids when one is authored.

- G-1. The root crate's public items are exactly FR-039's table and re-export no model item
  (FR-039-AC-1, planned as TC-055; the no-model-item part is implemented, while the complete
  inventory is not).
- G-2. No item named in FR-039 "Items codegen owns" is exported by the root crate (FR-039-AC-3;
  not true today: `src/kani/mod.rs:19` exports `lower_checked_arithmetic` and the other two).
- G-3. A struct-literal `KaniOutcome` outside the `kani` module does not compile (FR-030-AC-5;
  not true today: all four fields are `pub`, `src/kani/outcome.rs`).
- G-4. `Proved` carries a check count and a request with count zero returns `KaniOutcomeError`
  (FR-030-AC-4; not built: `proved` takes no count and no `proved` request can be refused for
  a zero count; `non_success` already returns `KaniOutcomeError` for a `Proved` or
  `Counterexample` request).
- G-5. Target: neither IR package depends on the codegen or runtime packages, as a cargo-deny `bans` build failure in `make deny` (AD-005 D-2; IR-343); no such entry exists today.
- G-6. Codegen's lock holds one `quire-contract-model` and one `quire-contract-ir` (existing
  gate).
- G-7. Every lowering request codegen sends yields exactly one record (AD-004; FR-035).

## Risks

What is measured today, what is open and with whom, and what is routed.

### Current state and gaps

- Original audit divergences and subsequent IR-347 corrections:
  1. IR-347 removed the root glob and model bridge (AD-005); codegen now imports the model crate
     directly.
  2. IR-347 removed `KaniProviderResult`, `KaniProviderRecord` and
     `KaniOutcomeKind::provider_result` from IR. FR-039 "Items QSL owns" assigns the first two to
     QSL; codegen owns the outcome-to-QSL terminal-value map and used none of these IR items.
  3. The three family lowerings are in IR (`src/kani/arithmetic.rs`, `collections.rs`,
     `objects.rs`, 392 lines) and codegen calls them. Moving them out of IR is IR-347 scope (the
     IR planner's relayed decision); the root glob has been removed.
  4. FR-030-AC-4 and AC-5, FR-039 and FR-037-AC-6 remain planned or partial in the matrix;
     `KaniOutcomeError` now exists for the invalid non-success constructor request.
  5. `proved_from_checks`' doc cites codegen's `classify_run` by file name
     (`src/kani/outcome.rs`): IR's documentation names its consumer, which is a reverse reference
     in prose and goes stale when codegen moves.
- Two generation paths are open on one seam: the authored-contract projection group and the
  checked-package group are both live in codegen (the table above lists eight and seven files).
  The IR-311 ruling, recorded in CG AD-004 (PR 215, "One input model"), retires the first:
  `CheckedPackageV2` is the one input model and `BoundPackage` is deleted path by path in
  codegen's work order. Until then IR still exports the V1 items. Nothing is routed for it.
- Codegen's own `deny.toml` still comments that `quire-spec-language` is "reached through
  quire-contract-ir". IR declares no such dependency (FR-028; TC-041) and codegen's lock holds no `quire-spec-language` package, so the comment and its exception are stale.
  Owner: codegen.

### Open questions

None open: the first draft's questions (the lowering move, the authored-contract path, the cause
codes) are decided above.

### Routed gaps

Needs stated to owners, not decisions. Ids are routing ids of IR-323; they are not requirement
ids. IR-owned items are the numbered divergences above and need no routing.

To codegen:

| Id | Stated need |
| --- | --- |
| R3-C1 | Take over the three Kani family lowerings (their request and result types and the three one-line wrappers) so IR can delete them. The final form is this AD's (lowerings in codegen, model items through the model crate, per IR-347 as relayed), and CG's ADs agree. |
| R3-C2 | Declare `quire-contract-model` directly and import model items from it, before IR removes the root glob. |
| R3-C4 | Remove the stale comment in `deny.toml` that places `quire-spec-language` behind `quire-contract-ir`. |

To QSL: none. QSL does not consume this seam.
