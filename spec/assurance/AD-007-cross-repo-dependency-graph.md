---
id: AD-007
title: "Cross-repo dependency graph and shared concept owners"
type: ArchitectureDescription
status: proposed
owner: kreneskyp
system: the Cargo dependency edges among quire-spec-language, quire-contract-ir, quire-contract-codegen, quire-contract-runtime, quire-specification, quire-canonical, quire-verification-contracts and quire-walk, the crate that owns each shared concept, and their remaining duplicate-owner gaps
relationships:
  - target: ix://agent-ix/quire-contract-ir/AD-001
    type: references
  - target: ix://agent-ix/quire-contract-ir/AD-004
    type: references
  - target: ix://agent-ix/quire-contract-ir/AD-005
    type: references
  - target: ix://agent-ix/quire-contract-ir/AD-006
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-028
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-039
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-011
    type: references
---
# Cross-repo dependency graph

AD-004 describes the QSpec-to-IR reader seam; AD-005 and AD-006 describe IR's QSL
and codegen consumers. This AD records the current repository edges and concept
owners. Its measurement is from IR `36bcd0a2`, CG `bec3f380`, RT `06d99d1c`,
QSL `adde683e` and QSpec `b1feb484` on 2026-11-02. Manifest and source
locations below are evidence for this snapshot, not revisions to pin.

## System Boundary

In scope are first-party Cargo dependencies among IR, CG, RT, QSL, QSpec,
quire-canonical, quire-walk, quire-verification-contracts, quire-exact,
quire-semantic-value and quire-eval, plus the ownership of concepts they share.
This AD does not prescribe internals of another repository or a compatibility
path. QSpec owns the published wire contract; it is not a runtime dependency
of IR.

## Views

The repository edges give the dependency direction; the concept table assigns
the owning crate and records the one remaining duplicate implementation.

### Repository edges and direction

| Consumer | Direct first-party production edges relevant to this seam | Evidence and disposition |
| --- | --- | --- |
| IR model | quire-canonical, quire-walk, quire-verification-contracts | `crates/quire-contract-model/Cargo.toml`; IR has no production edge to QSL, CG or RT. |
| QSL | IR's `quire-contract-model`, quire-exact, quire-semantic-value, quire-eval, quire-walk, quire-canonical | QSL `Cargo.toml:7-28`; the shared leaves are separate Git repositories. QSL's local package name is now `quire-contract-model`. |
| CG | IR root and model, RT, QSL `qsl-replay`, quire-exact, quire-semantic-value, quire-eval and quire-canonical | CG `Cargo.toml:28-46`; the root crate supplies Kani outcome types, while checked-package types come from the model crate (AD-006). |
| RT | optional quire-exact and quire-semantic-value under its `exact` feature | RT `Cargo.toml:30-38`; neither is fetched from QSL's repository. |
| QSpec qualification | verification contracts and trace tooling as dev edges | QSpec's specification remains the authority for wire semantics, not a linked library in the reader. |

The normal dependency direction through the main seam is QSpec's contract to
IR's reader, then QSL's producer verification and CG's consumer. CG also uses
RT's exact runtime and QSL's replay facade. No IR manifest points back to
QSL, CG or RT, and no QSL manifest points to CG or RT. These edges therefore
do not form a repository dependency cycle. A code reference to another owner
is not itself a Cargo edge.

### Concept owners and remaining duplicate

| Concept | Owner | Current source and finding |
| --- | --- | --- |
| Exact scalar, text, composite kernel, `Meter` and `Outcome` | standalone quire-exact | QSL `Cargo.toml:12-15`, CG `Cargo.toml:31-32`, RT `Cargo.toml:35-38` take the shared crate. RT `src/exact/mod.rs:6-9` says its scalar and metering copies have moved. The former count of 23 duplicate RT files and 124 shared public names no longer describes current main. |
| Semantic values, declarations and shared checking vocabulary | standalone quire-semantic-value | The same three consumers declare direct dependencies. RT `src/exact/mod.rs:24-30` imports shared definition, node and value types. |
| Expression checking and evaluation | quire-eval for the shared evaluator; RT for generated-code execution | RT `src/exact/mod.rs:8-9,33-39` retains `expression.rs` and exports `CheckedExpression`, `CheckedPackage`, plans and refusals. It calls this ported expression residue an open FR-275 defect. Shared scalar extraction did not resolve this duplicate-owner concern; RT must retire or delimit that module against quire-eval under its own ticket. |
| Checked-package wire types, strict reader and lowering | IR model, with QSpec owning the published wire | QSL and CG consume the model crate; source-integrity convergence is the open IR reader gap in AD-004. |
| Compile-side package and emitter | QSL `qsl-package` | QSL produces and verifies through IR's model reader (AD-005). |
| Kani outcome type | IR root crate | CG imports the root for this surface (AD-006); QSL `qsl-replay` owns replay envelopes and terminal records. |
| Generated-code runtime | RT | Native exact execution is distinct from QSL's source evaluation and CG's generation. |
| Canonical JSON encoding | quire-canonical | IR, CG and QSL consume the owned encoder. QVC's `jcs_canonicalize` delegates to `quire_canonical::to_vec`; `jcs_equal` and `jcs_sha256` are verification-facing wrappers over those bytes, not a second encoder (`quire-verification-contracts` `5070338d`, `src/lib.rs:616-644`). VER-52 can decide the wrapper API's ownership. |
| First-party lock coherence command | quire-cli `quire check-lock` | IR `Makefile:188-192` invokes the owned command. The copied IR `scripts/check_one_copy.awk` was removed by IR-641. Each repository qualifies its own lock. |

The QSL workspace still contains `qsl-walk-grow`; it is not the extracted
quire-walk crate. Any question about consolidating stack-growth calls is
separate from the exact-value extraction and cannot justify another owner in
IR. The shared leaves are already separate repositories, so the former
question of whether to extract them is closed by current manifests.

## Decisions

- One crate owns each shared exact and semantic-value concept. Consumers use
  the standalone quire-exact and quire-semantic-value packages directly.
  RT's remaining `expression.rs` is a recorded open duplication, not an
  exception or a settled second owner.
- IR owns its reader and model API, not QSL's emitter or CG's generation.
  CG's root IR edge is deliberate for Kani outcomes; its model edge serves
  checked-package types. QSL's model edge points only from QSL to IR.
- The published QSpec wire is read by reference. The IR reader's current
  two-member source references do not yet satisfy QSpec's source-integrity
  revision; AD-004 records the gap without changing dependency direction.
- IR uses the released quire-cli lock checker. Its removal of the copied AWK
  checker does not establish lock qualification for QSL, CG or RT.

## Risks

| Risk or open work | Owner and control |
| --- | --- |
| RT's ported expression checker remains under `src/exact/expression.rs` beside the extracted shared evaluator | RT FR-275: remove or explicitly delimit this residue; do not count the scalar extraction as closure. |
| QSpec's source-integrity wire outruns IR's strict reader | IR-1273; keep published and implemented shapes separate in AD-004 and reader conformance claims. |
| CG's direct IR-root edge outlives its Kani outcome use | CG owns the import audit (AD-006); no bridge is added to IR. |
| A first-party lock resolves two revisions of the same crate | Each consuming repository owns its lock qualification. IR invokes `quire check-lock` (`Makefile:188-192`); CG `make deny` currently invokes only `cargo deny check` (`bec3f380`, `Makefile:145-147`), and IR-1292 owns that missing CG gate. |
| QVC's verification-facing JCS wrapper API remains alongside direct quire-canonical consumers | QVC and quire-canonical, VER-52: decide whether the wrapper API is still needed; current QVC source delegates canonical encoding and is not a duplicate implementation. |
