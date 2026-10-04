---
id: AD-005
title: "IR to QSL seam: what QSL takes from the model crate, and the one direction it points"
type: ArchitectureDescription
status: proposed
owner: kreneskyp
system: quire-contract-model public items that quire-spec-language consumes (checked-package reader and types, identity-preimage types, the authored-contract and expression model), the Cargo edge between the two repositories, and the checks that hold the direction
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
| The identity-preimage and wire-envelope Rust types of the v2 package | IR to QSL | IR types, QSpec wire | the emitter builds the wire as a `Serialize` struct over them (`qsl-package/Cargo.toml` comment on the model dependency; the wire structs `CheckedPackageLockV2` and `CheckedPackageIdentityPreimageV2` built in `qsl-package/src/emit.rs`; on the reader side `canonical_preimage` in `qsl-package/src/checked_v2.rs` encodes the preimage for the `package_id` check) |
| `quire.checked-package/v2` bytes | QSL to IR | QSpec | AD-004 |
| The authored-contract and expression model: `ValueType`, `Expression` and `ExpressionKind`, `StateObservation`, `RequirementRef`, `ExecutionPoint`, `TypeDeclaration`, `Diagnostic`, `SourceSpan`, `DeclarationEnvironment` and its `check_expression`, among others | IR to QSL | IR | 57 files across QSL's root crate and `qsl-package` name the model crate; `ValueType` alone is named 212 times through the `ir::` alias (checking, linking, lowering, `model_source`, `native_model`, `command`) |

What does not cross, as a target: no replay, witness, envelope, terminal-record or obligation-identity
type comes from IR (AD-001 Replay ownership; FR-039 "Items QSL owns"), and no QSL type reaches IR
(FR-028). Current: the root crate still exports two such items, `KaniProviderResult` and
`KaniProviderRecord` (D-3), to be removed under IR-347; QSL does not use them.

### Identity and versions on this seam

- QSL depends on the model package by Cargo git source, `branch = "main"`, declared under the
  dependency name `quire-contract-ir` with `package = "quire-contract-model"` when this AD was
  written; QSL's `Cargo.toml` and `qsl-package/Cargo.toml` now key it as `quire-contract-model`
  (R3-Q1 is done; measured in IR-346, AD-007). The crate has no published semantic version
  (`publish = false`, version `0.1.0`). On the Rust types the assertion is the compiler plus
  one resolved copy in QSL's own lock; there is no version record to maintain and none is
  proposed.
- On the wire the assertion is `contract_version`, read once by IR (AD-004). The one digest on
  this seam is `package_id`, the content identity of a checked package. QSL computes it through
  `quire-canonical` (a dependency of QSL's root `Cargo.toml`, at the time of writing); IR recomputes it and
  refuses a mismatch (`stale_dependency`). One encoder produces that identity on both sides: IR
  recomputes it from the typed `identity_preimage` struct with `quire_canonical::sha256`
  (`checked_package/v2/mod.rs`, after IR-533), after the reader has refused bytes that are not
  `quire_canonical::to_vec`'s. The other identity digests IR computes (a nominal, application
  and structural node key, a lowered node's `ir_id` and the lowered package's id, the selected
  model document's digest) were `digest_json` when this AD was written, which was
  `serde_json::to_vec` followed by SHA-256 (since removed; see Current state and AD-007); FR-038 ("Every
  identity digest is computed through quire-canonical") states that all of them go through
  `quire-canonical` and that no encoder of IR's own remains, and the check of agreement by
  emission tests (QSL `tc_469_step_6_the_emitted_package_admits_via_i04`, one fixture; D-5)
  stays as a test, not as the guard. The v1 `quire.contract.canonical-json/v1` objects and the
  output-mapping identities take the same encoder (FR-016, FR-034), which spells the v1
  64-bit integers as decimal strings (a public v1 wire change that QSL's lowering wire emitted
  as numbers when this AD was written; whether QSL has followed is QSL's to say).
- IR states no pin, commit id or tool version about QSL, and QSL's references to "the pinned IR
  revision" in comments (for example the comment in `qsl-package/src/emit.rs` that says "pinned IR revision") name its Cargo lock, which
  is not an identity this seam asserts.

### Dependency direction and what enforces it

| Edge | Allowed | Held by | Gap |
| --- | --- | --- | --- |
| IR model to QSL | no | `tests/it/cycle_free_model.rs`: the model's production dependency names are checked against the `forbidden` list in that test and every one must be non-optional | by name only; no source check for the model package in `tc_041` (a git edge to the QSL repository is caught by `unknown-git`, not by this test) |
| IR root to QSL | no | the same file, root package checked by name prefix `qsl-` and by git source of the QSL repository (the loop over the root package's dependencies) | none for QSL |
| QSL to the model crate | yes | QSL ADR-011 section 6.1 lists it as an external edge of layer 4 beside `quire-canonical` (`qsl-package/Cargo.toml` comment); QSL TC-390 pins that layer's other workspace edges | none |
| QSL to the IR root crate | no | nothing in IR; QSL's manifests name the model package | none: QSL's manifests now key the dependency as `quire-contract-model` (R3-Q1 done); it was once aliased `quire-contract-ir`, which read as the root crate |
| IR to codegen, IR to runtime | no | a git edge to either repository fails `make deny` today: `[sources]` `unknown-git = "deny"` with an `allow-git` list that names neither (IR-346, AD-007); decision D adds name-level `bans` (IR-343) | no check by crate name, and the forbidden list in `tc_041` names neither `quire-contract-codegen` nor `quire-contract-runtime` (D-2) |
| any cycle among QSL, IR, runtime, codegen | no | QSL's `arch-lint direction` (FB-05, FB-11) over local checkouts, run on request and not part of QSL's `make ci` (QSL `Makefile`, `arch-lint-direction`) | runs only when someone supplies the clones |
| two copies of one first-party crate | no | IR `scripts/check_one_copy.awk` over IR's lock via `make deny`, to be replaced by one owned tool with no copy in IR (owner's ruling, IR-581, AD-007 O-4, FR-043). QSL has no general check: its `arch-lint duplicate-revisions` no longer exists on QSL `main`, and `deny.toml` `deny-multiple-versions` covers `quire-canonical` alone (IR-346, AD-007) | each guards its own lock only; QSL's lock has no one-entry-per-first-party-crate check (routed to QSL, AD-007 R-1) |

### Failure outcomes and who reports them

| Condition | Reported by | Outcome |
| --- | --- | --- |
| QSL's emission is refused by the reader | IR reports (AD-004 codes); QSL owns the producer defect | a failing QSL test, never a widened reader (AD-004 precedence) |
| A reader limit is reached | IR (`Incomplete` with limit, consumed, pointer) | QSL converts to its own `V2ReadIncomplete::Limit` (`V2ReadIncomplete` in `qsl-package/src/checked_v2.rs`) and its own limits struct is converted into IR's (`for_ir`, same file) |
| QSL's extent classification and IR's `requires_bound` disagree | QSL's agreement test (TC-440, test-only: `qsl-package/src/emit/extent_agreement.rs`, mounted under `cfg(test)`) | a failing QSL test; one fixture (a quantity) is an ignored test because the emitter omits the record |
| QSL and IR compute different `package_id` for one package | IR refuses `stale_dependency` | QSL's emission test fails |
| A QSL crate appears in IR's graph | IR's `tc_041` (root package by name and source, model package by a name list) and `make deny` `unknown-git` | IR test or `make deny` fails |

## Decisions

Four decisions govern the seam. None adds a layer between the repositories.

- A. QSL consumes IR through the model crate's items only. It never names the IR root crate. QSL
  reads only its layer-4 set (the reader, evidence, limits and identity-preimage types); the
  authored-contract items are not frozen by name here because they retire under QSL ADR-011 M-6c.
- B. A seam item changes in the order of AD-006 decision D: the consumer first gains what it needs, then IR removes the old item. IR does not widen a type to fit an emission
  (AD-004 precedence) and does not keep a second shape for QSL's convenience.
- C. One encoder for one identity, and the one-encoder rule binds IR: IR computes digests with
  `quire_canonical::to_vec(v, Limits)` and `quire_canonical::sha256`, not with `serde_json`. IR-274
  (adopt `quire-canonical` for every identity digest and delete IR's own encoders) is the
  one-owner fix; its spec is FR-016, FR-034 and FR-038, and its code has landed (IR-274 is
  done). The encode of a `serde_json::Value` is `quire-canonical`'s (one owner for IR, codegen
  and QSL; the upstream `Encode` is quire-canonical #7), so IR holds no walker of its own
  (FR-038-AC-91).
- D. Target: IR has no dependency on codegen, runtime or QSL, and that is a build failure, not
  prose. A git edge to those repositories already fails `make deny` through `unknown-git`
  (`[sources]`, IR-346); `deny.toml` `[bans]` is to list the crates under `deny` so an edge fails by name too. No such entry exists today. The home of this guard is the IR
  layout design, IR-343. IR does not rely on QSL's lint for its half (D-2); there is no CI option
  for running that lint (owner ruling, as relayed), so IR guards its own edges.

No compatibility layer is proposed: a type moved or removed in IR follows the same order, with no shim
between the two steps.

### Invariants a test can check

Local labels; the repository assigns requirement ids when one is authored.

- D-1. The model and root packages declare no dependency on any QSL crate (existing:
  FR-028-AC-1, FR-028-AC-3, TC-041). `tc_041` checks the root package by name prefix and by git
  source, and the model package by a fixed name list only; by source, `unknown-git` in
  `make deny` covers both packages.
- D-2. The model and root packages declare no dependency on `quire-contract-codegen` or
  `quire-contract-runtime`: by source, `unknown-git` in `make deny` holds it today for a git
  edge; by name, a cargo-deny `bans` failure is the target (IR-343), and `tc_041`'s lists are
  to name both crates.
- D-3. No public item of either IR crate is a replay, witness, envelope, terminal-record or
  obligation-identity type. Current: not true. The root crate exports `KaniProviderResult` and
  `KaniProviderRecord` (defined in `src/kani/outcome.rs`, re-exported by `src/kani/mod.rs`; the
  record's doc calls itself an FR-331 terminal record), which FR-039 assigns to QSL. Target:
  removed from IR under IR-347 (FR-039-AC-3, planned in TC-055).
- D-4. Every public item QSL reads is exported by name from the model crate's root, with no
  glob (statement FR-019; not true today, see Current state).
- D-5. A package QSL emits for each node family and semantic form QSL can produce is admitted at
  the `package_id` QSL wrote (runs in QSL; routed as R-Q6 of IR-324).
- D-6. For a record QSL classifies unbounded, IR lowers with `require_bounds` set and reports
  `requires_bound`, and conversely (QSL TC-440 today, one fixture ignored).

## Risks

What is measured today, what is open and with whom, and what is routed.

### Current state and gaps

- The model crate's own root uses glob re-exports for seven modules
  (the `pub use` lines of `crates/quire-contract-model/src/lib.rs`), against AD-001 and FR-019 ("by name, no
  glob"). The set of items a consumer such as QSL may rely on is therefore whatever the globs
  expose, not FR-019's table. IR-347 (reopened) carries glob removal; not restated here.
- The IR root crate is a re-export bridge: `src/lib.rs` holds `pub use quire_contract_model::*;`
  and its crate doc calls it a "compatibility bridge", although AD-001 and FR-039 say it re-exports
  nothing. `tc_041_bridge_reexports_the_exact_model_api_and_keeps_model_sources_single`
  (in `tests/it/cycle_free_model.rs`) asserts the bridge, the opposite of FR-039-AC-1, whose
  test (TC-055) is planned and does not exist. The decision is that the test goes with the
  IR-347 work (retag or remove `tc_041`'s bridge test); FR-039-AC-1 is not changed. QSL does not
  use the bridge; the consumer that does is codegen (AD-006).
- D-2 has no by-name check. AD-001 says runtime has no dependency in either direction with IR; codegen
  depends on IR and would close a cycle if IR depended on it. Today nothing in IR fails if an edge
  is added by name (`deny.toml` `[bans]` has no `deny` list); a git edge to the QSL, codegen or
  runtime repository already fails `make deny` through `[sources]` `unknown-git = "deny"` and its
  `allow-git` list (corrected in IR-346, AD-007). Decision D adds name-level `bans` entries, homed in IR-343. QSL's `arch-lint direction` would also
  report a cycle, but only when someone runs it with all three clones.
- One encoder: QSL's ADR-013 section 2 and its `arch-lint canonical-encoder` forbid a second
  canonical encoder beside `quire-canonical`, and the one-encoder rule binds IR (decision C).
  This bullet once listed IR's own encoders; corrected in IR-346 (AD-007): on IR `main` the
  symbols `digest_json`, `CanonicalWriter` and `canonical_envelope_bytes` no longer occur in
  `src/` or `crates/`, because IR-274's code changes removed them. QSL states (not re-measured
  here) that the serde_json form diverges from RFC 8785 on integers above 2^53, floats, negative
  zero and key order under `preserve_order`; IR's own `serde_json` features do not enable
  `preserve_order` (`Cargo.toml`), but a canonicalizer crate (`serde_json_canonicalizer`) is in
  IR's lock through `quire-verification-contracts`, which VER-52 moves onto `quire-canonical`;
  no IR source file names it.
- The QSL root crate still reads the authored-contract model widely (table above), while QSL
  ADR-011 states that `check` never imports the model crate and that its checker does not ask a
  backend to decide a semantic question (FB-06). Which of these uses survive QSL's layer
  extraction is QSL's to say.

### Open questions

| Question | Owner | Recommendation | Cost of the alternative |
| --- | --- | --- | --- |
| Which authored-contract model items stay on the seam? | QSL | QSL reads only the layer-4 set; the root-crate uses of the authored-contract model retire with its layer extraction (ADR-011 M-6c, as relayed), so IR does not freeze those items by name | IR would assert items that are about to retire |
| Which cargo-deny `bans` entries does IR carry? | IR (IR-343) | `quire-contract-codegen`, `quire-contract-runtime`, `quire-spec-language` and each QSL crate by name; a ban matches a crate name, so the list is revisited when QSL adds a crate, and `tc_041` (D-1) stays as a test; a git edge to those repositories already fails `unknown-git` | a by-name edge from an allowed repository or a path edge goes unnoticed until QSL's lint is run with clones |

### Routed gaps

Needs stated to owners, not decisions. Ids are routing ids of IR-323; they are not requirement
ids. Rows R-Q6 and R-S1 to R-S8 of IR-324 (codegen PR 214, IR AD-004) are not repeated.

To QSL (QSL reviews these rows):

| Id | Stated need |
| --- | --- |
| R3-Q1 | Done (QSL's manifests now key it `quire-contract-model`, measured in IR-346). It asked QSL to key the dependency as `quire-contract-model`, not `quire-contract-ir`, so the manifest does not read as a dependency on IR's root crate (which QSL must never depend on) (relayed by the IR planner; QSL-owned change). |
| R3-Q4 | Track the ignored TC-440 quantity case (the emitter omits the record) until an emitted quantity can be compared: QSL-247 and QSL-238, and IR-450 on the IR side. |

Answered, no longer routed (QSL's answer as relayed by the IR planner, untrusted until QSL signs off): R3-Q2, QSL reads only the layer-4 set (decision A); R3-Q3, the one-encoder rule binds IR (decision C); R3-Q5, no CI option, IR guards its own edges (decision D).

To QSpec: none added by this AD.

IR-owned items (D-2 coverage; removal of the glob, the bridge, the exported `KaniProvider*` items and the
Kani family lowerings) stay in this repository and are IR-347 scope (the planner's relayed decision);
no new ticket is filed.
