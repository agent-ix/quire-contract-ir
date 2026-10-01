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

Measured at QSL `origin/main` d81193f9 and IR `origin/main` 0a889f9.

| Item | Direction | Owner | Where QSL uses it |
| --- | --- | --- | --- |
| The `read_checked_package` reader, `CheckedPackageEvidence`, `CheckedPackageReadLimits`, `CheckedPackageDispatchResult`, `CheckedPackageV2`, the refusal and limit types | IR to QSL | IR (the wire is QSpec's) | the I2 byte reader, `qsl-package/src/checked_v2.rs:94-98`, and its tests |
| The identity-preimage and wire-envelope Rust types of the v2 package | IR to QSL | IR types, QSpec wire | the emitter builds the wire as a `Serialize` struct over them (`qsl-package/Cargo.toml` comment on the model dependency; `checked_v2.rs:595`) |
| `quire.checked-package/v2` bytes | QSL to IR | QSpec | AD-004 |
| The authored-contract and expression model: `ValueType`, `Expression` and `ExpressionKind`, `StateObservation`, `RequirementRef`, `ExecutionPoint`, `TypeDeclaration`, `Diagnostic`, `SourceSpan`, `DeclarationEnvironment` and its `check_expression`, among others | IR to QSL | IR | 57 files across QSL's root crate and `qsl-package` name the model crate; `ValueType` alone is named 212 times through the `ir::` alias (checking, linking, lowering, `model_source`, `native_model`, `command`) |

What does not cross: no replay, witness, envelope, terminal-record or obligation-identity type
comes from IR (AD-001 Replay ownership; FR-039 "Items QSL owns"), and no QSL type reaches IR
(FR-028).

### Identity and versions on this seam

- QSL depends on the model package by Cargo git source, `branch = "main"`, declared under the
  dependency name `quire-contract-ir` with `package = "quire-contract-model"` (QSL
  `Cargo.toml:75`, `qsl-package/Cargo.toml:36`). The crate has no published semantic version
  (`publish = false`, version `0.1.0`). On the Rust types the assertion is the compiler plus
  one resolved copy in QSL's own lock; there is no version record to maintain and none is
  proposed.
- On the wire the assertion is `contract_version`, read once by IR (AD-004). The one digest on
  this seam is `package_id`, the content identity of a checked package. QSL computes it through
  `quire-canonical` (QSL `Cargo.toml:41`); IR recomputes it from the bytes it was given and
  refuses a mismatch (`stale_dependency`). Two encoders therefore produce one identity: IR's
  digest helper is `serde_json::to_vec` of a parsed value followed by SHA-256
  (`crates/quire-contract-model/src/checked_package/common.rs:419-421`), after the reader has
  refused non-canonical bytes. That the two agree for every admissible value is checked only by
  emission tests (QSL `tc_469_step_6_the_emitted_package_admits_via_i04` for one fixture; D-5
  below).
- IR states no pin, commit id or tool version about QSL, and QSL's references to "the pinned IR
  revision" in comments (for example `qsl-package/src/emit.rs:961`) name its Cargo lock, which
  is not an identity this seam asserts.

### Dependency direction and what enforces it

| Edge | Allowed | Held by | Gap |
| --- | --- | --- | --- |
| IR model to QSL | no | `tests/it/cycle_free_model.rs`: the model's production dependency names are checked against a forbidden list (`:45-53`) and every one must be non-optional | by name only; no source check for the model package |
| IR root to QSL | no | the same file, root package checked by name prefix `qsl-` and by git source of the QSL repository (`:78-85`) | none for QSL |
| QSL to the model crate | yes | QSL ADR-011 section 6.1 lists it as layer 4's one external edge (`qsl-package/Cargo.toml` comment); QSL TC-390 pins that layer's other workspace edges | none |
| QSL to the IR root crate | no | nothing in IR; QSL's manifests name the model package | the dependency name `quire-contract-ir` reads as the root crate (see R3-Q1) |
| IR to codegen, IR to runtime | no | nothing in IR yet; decision D makes it a cargo-deny `bans` failure (IR-343) | the forbidden list in `tc_041` names neither `quire-contract-codegen` nor `quire-contract-runtime` (D-2) |
| any cycle among QSL, IR, runtime, codegen | no | QSL's `arch-lint direction` (FB-05, FB-11) over local checkouts, run on request and not part of QSL's `make ci` (QSL `Makefile`, `arch-lint-direction`) | runs only when someone supplies the clones |
| two copies of one first-party crate | no | IR `scripts/check_one_copy.awk` over IR's lock via `make deny`; QSL `arch-lint duplicate-revisions` over QSL's lock | each guards its own lock only |

### Failure outcomes and who reports them

| Condition | Reported by | Outcome |
| --- | --- | --- |
| QSL's emission is refused by the reader | IR reports (AD-004 codes); QSL owns the producer defect | a failing QSL test, never a widened reader (AD-004 precedence) |
| A reader limit is reached | IR (`Incomplete` with limit, consumed, pointer) | QSL converts to its own `V2ReadIncomplete::Limit` (`checked_v2.rs:53`) and its own limits struct is converted into IR's (`for_ir`, `:223`) |
| QSL's extent classification and IR's `requires_bound` disagree | QSL's agreement test (TC-440, test-only: `qsl-package/src/emit/extent_agreement.rs`, mounted under `cfg(test)`) | a failing QSL test; one fixture (a quantity) is an ignored test because the emitter omits the record |
| QSL and IR compute different `package_id` for one package | IR refuses `stale_dependency` | QSL's emission test fails |
| A QSL crate appears in IR's graph | IR's `tc_041` | IR test fails |

## Decisions

Four decisions govern the seam. None adds a layer between the repositories.

- A. QSL consumes IR through the model crate's items only. It never names the IR root crate, and
  what QSL consumes beyond the reader is listed (not inferred from grep) once QSL states it
  (R3-Q2).
- B. A seam item is changed in IR first; QSL follows. IR does not widen a type to fit an emission
  (AD-004 precedence) and does not keep a second shape for QSL's convenience.
- C. One encoder for one identity: a single RFC 8785 implementation, `quire-canonical`, that
  both sides call, not two implementations kept equal by tests. IR-274 (adopt `quire-canonical`,
  drop the serde_json canonicalizer; QSL Architecture Remediation R1) is the one-owner fix and
  this AD recommends it be brought forward.
- D. IR has no dependency on codegen, runtime or QSL, and that is a build failure, not prose:
  `deny.toml` `[bans]` lists those crates under `deny`, so `make deny` (part of `make ci`) fails
  on any such edge, by crate name. The home of this guard is the IR layout design, IR-343. IR
  does not rely on QSL's lint for its half (D-2).

No compatibility layer is proposed: a type moved or removed in IR is moved in QSL in the same
step.

### Invariants a test can check

Local labels; the repository assigns requirement ids when one is authored.

- D-1. The model and root packages declare no dependency on any QSL crate, by name and by
  source (existing: FR-028-AC-1, FR-028-AC-3, TC-041).
- D-2. The model and root packages declare no dependency on `quire-contract-codegen` or
  `quire-contract-runtime`, by name, enforced as a cargo-deny `bans` failure in `make deny`
  (not enforced today; IR-343).
- D-3. No public item of either IR crate is a replay, witness, envelope, terminal-record or
  obligation-identity type (existing statement: FR-039-AC-3, planned in TC-055).
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
  (`crates/quire-contract-model/src/lib.rs:31-47`), against AD-001 and FR-019 ("by name, no
  glob"). The set of items a consumer such as QSL may rely on is therefore whatever the globs
  expose, not FR-019's table. IR-347 (reopened) carries glob removal; not restated here.
- The IR root crate is a re-export bridge: `src/lib.rs:12` is `pub use quire_contract_model::*;`
  and its crate doc calls it a "compatibility bridge", although AD-001 and FR-039 say it re-exports
  nothing. `tc_041_bridge_reexports_the_exact_model_api_and_keeps_model_sources_single`
  (`tests/it/cycle_free_model.rs:98-107`) asserts the bridge, the opposite of FR-039-AC-1, whose
  test (TC-055) is planned and does not exist. The decision is that the test goes with the
  IR-347 work (retag or remove `tc_041`'s bridge test); FR-039-AC-1 is not changed. QSL does not
  use the bridge; the consumer that does is codegen (AD-006).
- D-2 has no check. AD-001 says runtime has no dependency in either direction with IR; codegen
  depends on IR and would close a cycle if IR depended on it. Today nothing in IR fails if an edge
  is added (`deny.toml` `[bans]` has no `deny` list). The fix is decision D: cargo-deny `bans`
  entries, a build failure in `make deny`, homed in IR-343. QSL's `arch-lint direction` would also
  report a cycle, but only when someone runs it with all three clones.
- Two encoders: QSL's ADR-013 section 2 and its `arch-lint canonical-encoder` forbid a second
  canonical encoder beside `quire-canonical`, by finding a `serde_json` serializer paired with a
  hash in a shipped file of QSL. IR's `digest_json` is that pattern, in a repository the lint
  does not scan. Agreement is by test only. Not measured: whether the two differ on any value IR
  admits; the reader refuses non-canonical bytes first.
- The QSL root crate still reads the authored-contract model widely (table above), while QSL
  ADR-011 states that `check` never imports the model crate and that its checker does not ask a
  backend to decide a semantic question (FB-06). Which of these uses survive QSL's layer
  extraction is QSL's to say.

### Open questions

| Question | Owner | Recommendation | Cost of the alternative |
| --- | --- | --- | --- |
| Which authored-contract model items will QSL keep reading? | QSL | QSL lists them; IR asserts them by name in a test and drops the rest from its public surface | IR cannot remove or rename any model item without grepping QSL |
| When does IR-274 land? | IR, QSL (R1, due 09-30 per the IR-274 text, untrusted) | bring it forward; it removes the second encoder and the lint gap | two encoders held equal by fixtures only |
| Which cargo-deny `bans` entries (crate names) does IR carry? | IR (IR-343) | `quire-contract-codegen`, `quire-contract-runtime`, `quire-spec-language` and each QSL crate by name | a reverse edge goes unnoticed until QSL's lint is run with clones |
| Run QSL's `arch-lint direction` in a CI that has the clones | QSL | yes, or state that IR's own checks are the guard | cycle detection depends on a manual run |

### Routed gaps

Needs stated to owners, not decisions. Ids are routing ids of IR-323; they are not requirement
ids. Rows R-Q6 and R-S1 to R-S8 of IR-324 (codegen PR 214, IR PR 239) are not repeated.

To QSL (QSL reviews these rows):

| Id | Stated need |
| --- | --- |
| R3-Q1 | Declare the model dependency under its own name `quire-contract-model`. The name `quire-contract-ir` with `package = "quire-contract-model"` (QSL `Cargo.toml:75`, `qsl-package/Cargo.toml:36`) names the root crate, which QSL must not depend on. |
| R3-Q2 | List the model items QSL intends to keep reading beyond the checked-package reader, so IR can assert them by name. |
| R3-Q3 | Name the `quire-canonical` entry point IR should call under IR-274, and confirm IR-274's timing. |
| R3-Q4 | Track the ignored TC-440 quantity case (the emitter omits the record) until an emitted quantity can be compared. |
| R3-Q5 | Either run `arch-lint direction` in a CI that holds the IR, runtime and codegen clones, or say it is not the guard for IR's side. |

To QSpec: none added by this AD.

IR-owned items (D-2 coverage, glob and bridge removal) stay in this repository; the glob and
bridge are in IR-347's reopened scope and no new ticket is filed.
