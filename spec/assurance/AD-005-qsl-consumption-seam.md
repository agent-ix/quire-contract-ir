---
id: AD-005
title: "IR to QSL seam: what QSL takes from the model crate, and the one direction it points"
type: ArchitectureDescription
status: accepted
owner: kreneskyp
system: quire-contract-model public items that quire-spec-language consumes (checked-package reader and types, structural checked-package types, the authored-contract and expression model), the Cargo edge between the two repositories, and the checks that hold the direction
relationships:
  - target: ix://agent-ix/quire-contract-ir/AD-001
    type: references
  - target: ix://agent-ix/quire-contract-ir/AD-003
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-028
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-039
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-011
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-013
    type: references
---
# IR to QSL seam: what QSL takes from the model crate

AD-001 fixes who owns what and that IR never depends on QSL. This AD states the seam in full:
what QSL actually reads from IR (more than the checked-package reader), how identity is asserted
on each item, what holds the direction, and who reports each failure. It is one of the seam
descriptions of IR-323. The checked-package reader and its refusals are the QSpec to IR seam and
are described in AD-004; this AD does not restate them.

## System Boundary

IR owns the model crate's public items. QSL owns the compiler that builds values of those types
and emits `quire.checked-package/v2` bytes, and the `qsl-replay` crate. The arrow is one way: QSL
depends on IR, IR depends on no QSL crate.

Out of scope: the reader's own behaviour (AD-004), the Kani and replay crossing between codegen
and QSL (codegen's AD-002 and AD-003), and anything QSL decides inside its own crates (routed).

## Views

The seam is described as what crosses it, how identity is asserted, which way dependencies point
and who reports each failure.

### What crosses the seam

Measured at the commits of QSL and IR this AD was written against.

| Item | Direction | Owner | Where QSL uses it |
| --- | --- | --- | --- |
| The `read_checked_package` reader, `CheckedPackageEvidence`, `CheckedPackageReadLimits`, `CheckedPackageDispatchResult`, `CheckedPackageV2`, the refusal and limit types | IR to QSL | IR (the wire is QSpec's) | the I2 byte reader, the `use` of the model crate's reader in `qsl-package/src/checked_v2.rs`, and its tests |
| The typed lock, structural node-id and wire-envelope Rust types of the V2 package | IR to QSL | IR types, QSpec wire | QSL supplies the package name, lock, graph, source correspondence, capabilities and diagnostics to the IR encoder; IR performs strict admission of the resulting bytes |
| `quire.checked-package/v2` bytes | QSL to IR | QSpec | AD-004 |
| The authored-contract and expression model: `ValueType`, `Expression` and `ExpressionKind`, `StateObservation`, `RequirementRef`, `ExecutionPoint`, `TypeDeclaration`, `Diagnostic`, `SourceSpan`, `DeclarationEnvironment` and its `check_expression`, among others | IR to QSL | IR | QSL's root crate and `qsl-package` consume the model package directly; the exact symbol inventory remains QSL-owned as its layer extraction proceeds. |

What does not cross, as a target: no replay, witness, envelope, terminal-record or obligation-identity
type comes from IR (AD-001 Replay ownership; FR-039 "Items QSL owns"), and no QSL type reaches IR
(FR-028). IR-347 removed the root crate's former `KaniProviderResult` and
`KaniProviderRecord` exports (D-3); QSL did not use them.

### Identity and versions on this seam

- QSL depends on the model package by Cargo git source, `branch = "main"`. Its
  `Cargo.toml:28,88` and `qsl-package/Cargo.toml:47` use the local dependency
  name `quire-contract-model`, matching the selected package. The crate has no published semantic version
  (`publish = false`, version `0.1.0`). On the Rust types the assertion is the compiler plus
  one resolved copy in QSL's own lock; there is no version record to maintain and none is
  proposed.
- On the wire, `contract_version` distinguishes the closed document kind.
  The package's required `name` is its identity. Every node id is
  `{package, path, ordinal}` and is compared componentwise. A dependency
  selection is `{identity}`; a supplied dependency binds when its `name`
  matches. Diamond paths to unequal complete JSON packages under one identity
  refuse `invalid_package`/`conflicting-definition`. QSL emits canonical
  bytes; IR validates those bytes, the structural ids and the graph. Neither
  side computes a package or node content hash, and no staleness refusal
  exists. The emitted-package admission test remains QSL-owned.
- IR states no commit id, tool pin or tool version about QSL.

### Dependency direction and what enforces it

| Edge | Allowed | Held by | Gap |
| --- | --- | --- | --- |
| IR model to QSL | no | `tests/it/cycle_free_model.rs:206-249,251-323` checks the resolved model production graph for forbidden names, `qsl-` prefixes and named repository sources, including indirect normal and build edges; `deny.toml:31-44` rejects unlisted Git sources | the model test excludes dev edges, and no QSL crate-name ban covers a path or registry dev edge |
| IR root to QSL | no | `tests/it/cycle_free_model.rs:379-404` checks every direct root dependency kind by QSL name/prefix and repository source; `deny.toml:31-44` rejects unlisted Git sources | none for the declared root edge |
| QSL to the model crate | yes | QSL ADR-011 section 6.1 lists it as an external edge of layer 4 beside `quire-canonical` (`qsl-package/Cargo.toml` comment); QSL TC-390 pins that layer's other workspace edges | none |
| QSL to the IR root crate | no | QSL's root and `qsl-package` manifests use `quire-contract-model` as the local key and selected package | none; R3-Q1's naming cleanup landed |
| IR to codegen, IR to runtime | no | `deny.toml` `[bans]` denies both crate names, including path and registry edges; `[sources]` `unknown-git = "deny"` also rejects git edges to either repository. `tc_041` names both crates for model and root checks (D-2). | none |
| any cycle among QSL, IR, runtime, codegen | no | QSL's `arch-lint direction` (FB-05, FB-11) over local checkouts, run on request and not part of QSL's `make ci` (QSL `Makefile`, `arch-lint-direction`) | runs only when someone supplies the clones |
| two copies of one first-party crate | no | IR's `make deny` invokes the owned `quire check-lock Cargo.lock` command; its former copied AWK checker was removed under IR-641. QSL's own lock policy remains its responsibility (AD-007 R-1). | each repository guards its own lock; the IR check does not certify QSL's lock |

### Failure outcomes and who reports them

| Condition | Reported by | Outcome |
| --- | --- | --- |
| QSL's emission is refused by the reader | IR reports (AD-004 codes); QSL owns the producer defect | a failing QSL test, never a widened reader (AD-004 precedence) |
| A reader limit is reached | IR (`Incomplete` with limit, consumed, pointer) | QSL converts to its own `V2ReadIncomplete::Limit` (`V2ReadIncomplete` in `qsl-package/src/checked_v2.rs`) and its own limits struct is converted into IR's (`for_ir`, same file) |
| QSL's extent classification and IR's `requires_bound` disagree | QSL's agreement test (TC-440, test-only: `qsl-package/src/emit/extent_agreement.rs`, mounted under `cfg(test)`) | a failing QSL test; one fixture (a quantity) is an ignored test because the emitter omits the record |
| QSL emits duplicate or malformed structural ids, or a dependency whose name does not bind its selection | IR returns the applicable typed refusal | QSL's emission test fails |
| A QSL crate appears on an IR production path or as a direct root dependency | IR's TC-041 checks resolved model production edges and every direct root dependency kind; `make deny` rejects unlisted Git sources | IR test or `make deny` fails |

## Decisions

Four decisions govern the seam. None adds a layer between the repositories.

- A. QSL consumes IR through the model crate's items only. It never names the IR root crate. QSL
  reads only its layer-4 set (the reader, evidence, limits and structural checked-package types); the
  authored-contract items are not frozen by name here because they retire under QSL ADR-011 M-6c.
- B. A seam item changes in the order of AD-006 decision D: the consumer first gains what it needs, then IR removes the old item. IR does not widen a type to fit an emission
  (AD-004 precedence) and does not keep a second shape for QSL's convenience.
- C. One canonical byte encoder: IR and QSL use `quire-canonical` for
  the V2 wire. Neither implements a second canonical encoder, and neither
  derives product identities or known-answer seals from those bytes. IR's
  `serde_json::Value` encoding remains iterative through that crate.
- D. IR has no dependency on codegen, runtime or QSL, and that is a build failure, not
  prose. A git edge to those repositories fails `make deny` through `unknown-git`
  (`[sources]`, IR-346); `deny.toml` `[bans]` rejects codegen and runtime by crate name too.
  The home of this guard is the IR layout design, IR-343. IR guards its own edges (D-2).

No compatibility layer is proposed: a type moved or removed in IR follows the same order, with no shim
between the two steps.

### Invariants a test can check

Local labels; the repository assigns requirement ids when one is authored.

- D-1. The model and root packages declare no dependency on any QSL crate (existing:
  FR-028-AC-1, FR-028-AC-3, TC-041). TC-041 checks every direct root
  dependency kind by QSL name/prefix and source and checks the model's resolved production
  graph by forbidden name/prefix and selected source. `unknown-git` in `make deny`
  covers Git edges to unlisted repositories for both packages; TC-041 does not check
  the model's dev dependency graph.
- D-2. The model and root packages declare no dependency on `quire-contract-codegen` or
  `quire-contract-runtime`: by source, `unknown-git` in `make deny` rejects a git edge;
  by name, cargo-deny `bans` rejects either crate, and `tc_041` checks both model and root.
- D-3. No public item of either IR crate is a replay, witness, envelope, terminal-record or
  obligation-identity type. IR-347 removed `KaniProviderResult` and `KaniProviderRecord` from
  `src/kani/outcome.rs` and their re-exports from `src/kani/mod.rs` (FR-039-AC-3). The complete
  TC-055 public-inventory check and its negative Kani cases are implemented
  (FR-039-AC-4).
- D-4. Every public item QSL reads is exported by name from the model crate's root, with no
  glob (FR-019; implemented under IR-347).
- D-5. A package QSL emits for each node family and semantic form QSL can produce is admitted under
  the package `name` QSL wrote (runs in QSL; routed as R-Q6 of IR-324).
- D-6. For a record QSL classifies unbounded, IR lowers with `require_bounds` set and reports
  `requires_bound`, and conversely (QSL TC-440 today, one fixture ignored).

## Risks

What is measured today, what is open and with whom, and what is routed.

### Current state and gaps

- IR-347 replaced the model crate's seven root glob re-exports with the named FR-019 public
  inventory. QSL now depends on that model package under its own name `quire-contract-model`.
- IR-347 removed the IR root crate's model re-export and the obsolete `tc_041` bridge test.
  Codegen imports the model crate directly (AD-006). TC-055 now backs FR-039-AC-1's
  complete root public-inventory check.
- D-2's by-name check is in `deny.toml` `[bans]`, run by `make deny` and `make ci`, and
  `tc_041` checks both names in the model's production graph and every root dependency kind.
  AD-001 says runtime has no dependency in either direction with IR; codegen depends on IR
  and would close a cycle if IR depended on it. Git edges to QSL, codegen or runtime also fail
  `[sources]` `unknown-git = "deny"` (IR-346, AD-007).
- One encoder: QSL's ADR-013 section 2 and its `arch-lint canonical-encoder` forbid a second
  canonical encoder beside `quire-canonical`, and the one-encoder rule binds IR (decision C).
  This bullet once listed IR's own encoders; corrected in IR-346 (AD-007): on IR `main` the
  symbols `digest_json`, `CanonicalWriter` and `canonical_envelope_bytes` no longer occur in
  `src/` or `crates/`, because IR-274's code changes removed them. QSL states (not re-measured
  here) that the serde_json form diverges from RFC 8785 on integers above 2^53, floats, negative
  zero and key order under `preserve_order`; IR's own `serde_json` features do not enable
  `preserve_order` (`Cargo.toml`). QVC's public `jcs_canonicalize` already delegates to
  `quire_canonical::to_vec` (QVC `5070338d`, `src/lib.rs:616-623`), and its equality and
  digest functions wrap those bytes (`src/lib.rs:629-644`). VER-52, if still needed,
  concerns the wrapper API's ownership, not moving a second encoder implementation.
- The QSL root crate still reads the authored-contract model widely (table above), while QSL
  ADR-011 states that `check` never imports the model crate and that its checker does not ask a
  backend to decide a semantic question (FB-06). Which of these uses survive QSL's layer
  extraction is QSL's to say.

### Open questions

| Question | Owner | Recommendation | Cost of the alternative |
| --- | --- | --- | --- |
| Which authored-contract model items stay on the seam? | QSL | QSL reads only the layer-4 set; the root-crate uses of the authored-contract model retire with its layer extraction (ADR-011 M-6c, as relayed), so IR does not freeze those items by name | IR would assert items that are about to retire |
| Does the model's dev-edge policy need a QSL name ban? | IR | Current `deny.toml:17-23` bans codegen and runtime by name; `unknown-git` rejects a QSL Git edge, and TC-041 checks model production and all direct root edges. Decide whether a model dev path or registry edge to a QSL crate must also be a build failure (IR-343 follow-up if so). | Without that ban, the uncovered model dev edge can enter through an allowed non-Git source while production remains guarded. |

### Routed gaps

Needs stated to owners, not decisions. Ids are routing ids of IR-323; they are not requirement
ids. Rows R-Q6 and R-S1 to R-S8 of IR-324 (codegen PR 214, IR AD-004) are not repeated.

To QSL (QSL reviews these rows):

| Id | Stated need |
| --- | --- |
| R3-Q1 | Resolved on QSL `adde683e`: both the root and `qsl-package` use `quire-contract-model` as the local dependency key (`Cargo.toml:28,88`; `qsl-package/Cargo.toml:47`). |
| R3-Q4 | Track the ignored TC-440 quantity case (the emitter omits the record) until an emitted quantity can be compared: QSL-247 and QSL-238, and IR-450 on the IR side. |

Answered, no longer routed (QSL's answer as relayed by the IR planner, untrusted until QSL signs off): R3-Q2, QSL reads only the layer-4 set (decision A); R3-Q3, the one-encoder rule binds IR (decision C); R3-Q5, no CI option, IR guards its own edges (decision D).

To QSpec: none added by this AD.

IR-347 removed the root glob, bridge, exported `KaniProvider*` items and Kani family
lowerings. IR-1229 enforces D-2 dependency direction through IR's existing gates.
