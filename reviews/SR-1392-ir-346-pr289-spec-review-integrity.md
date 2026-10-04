---
id: SR-1392
title: "integrity review of PR 289 (AD-007 measured claims)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@cb94d57e427dd1154a442b8ba7ba0461e99443e1; spec/assurance/AD-007-cross-repo-dependency-graph.md (edge tables, concept owners, invariants G-1..G-5, current state and gaps); re-measured read-only from origin/main manifests, Cargo.lock files, deny.toml files and sources of quire-spec-language, quire-contract-ir, quire-contract-codegen, quire-contract-runtime, quire-specification, quire-canonical, quire-verification-contracts and quire-walk (gh api), fetched 2026-10-04"
review_set: subset
---
# SR-1392: integrity review of PR 289

## Summary

Ticket: IR-346. Every measured claim in AD-007 was re-measured independently from
the repositories' own manifests, locks and sources. Most hold. Two claims about
IR's own guards are wrong in a way that matters for IR-343's scope, and several
counts and table cells are slightly off.

Confirmed by measurement:

- All 19 QSL manifests (18 workspace members plus the separate `fuzz` workspace,
  which depends only on `qsl-bench`): every crate-level row matches except the
  cells in FND-003. No QSL manifest, normal or dev, names CG, RT, QVC or QSpec,
  and QSL's Cargo.lock holds no `quire-contract-codegen`, `quire-contract-runtime`
  or `quire-contract-ir` (root) entry. QVC appears in QSL's lock only
  transitively through `quire-contract-model`. QSL source references to CG and RT
  are doc comments only (no test-time crate generation). The AD's contradiction
  of the ticket premise (QSL dev edges to CG and a historical IR) is correct.
- IR: neither manifest names a QSL, CG or RT crate in any kind; no build edges.
  Edges to quire-walk, quire-canonical, QVC and `ix-trace-rs` are as tabled.
- CG: normal edges to IR root, IR model, RT (`exact`), `qsl-replay`,
  quire-canonical; dev edges to RT (`proptest`, `exact`) and QVC. CG's lock holds
  exactly 8 QSL crates (`qsl-attrs`, `-cst`, `-eval`, `-forms`, `-foundation`,
  `-package`, `-replay`, `-semantics`) plus `quire-exact` and
  `quire-semantic-value`, both `filament-core-data` crates and `quire-rs`. CG's
  lock still resolves `quire-walk` from the QSL repository (confirmed).
- RT: one first-party edge, optional `quire-exact` from the QSL repo behind
  `exact`; `git grep quire_exact` finds no use in `src/`. `deny.toml` bans exactly
  14 QSL crate names. `src/exact/` is 12,080 lines.
- The 124 shared names reproduce exactly: unique `pub struct`/`pub enum` names in
  RT `src/exact` (151) intersected with those in `quire-exact/src` plus
  `quire-semantic-value/src` (167) gives 124, including `Meter`, `Integer`,
  `Decimal`, `Text`, `Outcome`, `ScalarLimits`. The AD's qualifier "same-named,
  not proven identical" is the correct strength for a name intersection.
- `KaniProviderResult` and `KaniProviderRecord` are `pub` in IR
  `src/kani/outcome.rs`; `qsl-replay` holds the proof-result envelope.
  `KaniOutcome` is IR's alone.
- QVC has its own RFC 8785 path on `serde_json_canonicalizer`; IR, CG, QSL and RT
  call none of its `jcs_*` functions; QSpec's tests do.
- `qsl-walk-grow` is depended on by no crate; IR model calls `stacker` and
  `serde_stacker` directly.
- O-1 figures: QSL main has 872 commits and 2,886 tracked files totalling 33.4 MiB
  at HEAD; `quire-exact` has 23 files and `quire-semantic-value` 15. Dependents:
  12 crates on `quire-exact` (root, analyze, bench, cst, eval, forms, foundation,
  package, replay, route as dev, semantics, semantic-value) plus RT; 6 on
  `quire-semantic-value` (root, bench, eval, package, replay, semantics). Both
  crates are `#![no_std]`; `quire-exact` has no first-party dependency;
  `quire-semantic-value` depends on `quire-exact` and quire-canonical.
- quire-canonical, QVC and quire-walk have no first-party normal edge.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | G-1 claims an existing check ("existing: `tc_041` for QSL") for "No IR manifest names a QSL ... crate", but `tc_041` covers only the root crate by `qsl-` prefix and git source. For `quire-contract-model` it checks a fixed name list (`quire-spec-language`, `quire-contract-ir`, observation, protocol, TL crates): no `qsl-*` prefix, no `quire-exact` or `quire-semantic-value`, no source check. A model edge to `qsl-foundation` or `quire-exact` passes `tc_041`. Restate G-1 as root-only for `tc_041`, and name what actually guards the model crate (see FND-002). | spec/assurance/AD-007-cross-repo-dependency-graph.md:171-172, tests/it/cycle_free_model.rs:38-90 |
| FND-002 | medium | The AD says IR has no guard against CG, RT or QSL edges beyond `tc_041` ("IR `deny.toml` has no `bans` ... IR-343"; Decision B "IR's own guard is `cargo deny` (IR-343)"). IR's `deny.toml` already has `[sources] unknown-git = "deny"`, `unknown-registry = "deny"` and an `allow-git` list of only `ix-trace-rs`, quire-canonical, QVC and quire-walk; `make deny` runs `cargo deny check`, so any git edge to the QSL, CG or RT repository fails today in every dependency kind, for both IR crates. The gap row overstates the gap and sends IR-343 to build a guard that partly exists. Record the existing source guard and state what name-level `bans` would add beyond it. | spec/assurance/AD-007-cross-repo-dependency-graph.md:162-163, spec/assurance/AD-007-cross-repo-dependency-graph.md:191 |
| FND-003 | low | Crate-level and repo-level cells that do not match the manifests: (a) `qsl-semantics` "D: ... own `test-support`" is wrong; its dev table enables `quire-exact` with `test-support`, not its own feature; (b) "`quire-exact` is the only source" is wrong; `qsl-attrs`, `qsl-walk-grow` and `tools/arch-lint` also have no workspace dependency; (c) "QSL (all crates) -> `ix-trace-rs` D" is wrong; `qsl-attrs` and `qsl-bench` do not declare it. | spec/assurance/AD-007-cross-repo-dependency-graph.md:70, spec/assurance/AD-007-cross-repo-dependency-graph.md:107, spec/assurance/AD-007-cross-repo-dependency-graph.md:148 |
| FND-004 | low | Counts and names in the RT and O-1 rows: (a) RT `src/exact` is 23 files, not 24 (12,080 lines is correct); (b) RT's second workspace member is the package `quire-contract-runtime-footprint` at `measurement/footprint`; (c) the O-1 row "Size of what RT, CG and IR fetch with the QSL repository" includes IR, which the next rows say fetches nothing; (d) "about 33 MiB" is the tracked tree at HEAD, while a Cargo git fetch clones history, so it is a lower bound; say so. | spec/assurance/AD-007-cross-repo-dependency-graph.md:46, spec/assurance/AD-007-cross-repo-dependency-graph.md:131, spec/assurance/AD-007-cross-repo-dependency-graph.md:207 |
| FND-005 | low | The canonical-bytes row names QVC's `jcs_canonicalize` and `jcs_equal` but omits `jcs_sha256`, which is exactly this row's concept (canonical bytes and their SHA-256). QSpec's `tests/shared_reference.rs` imports `jcs_canonicalize` and `jcs_sha256`. Add it so R-3's removal scope is complete. | spec/assurance/AD-007-cross-repo-dependency-graph.md:127 |
| FND-006 | low | "Compile-side checked package and emitter: `qsl-package`, one owner" misses that the QSL root crate also defines a public `CheckedPackage<'a>` (`src/checking.rs`, SEAM-era, over the IR authored-contract model). It is not a copy of `qsl-package`'s type, but it is a second same-named owner until M-6c; note it and its retirement. | spec/assurance/AD-007-cross-repo-dependency-graph.md:136 |
| FND-007 | low | "a crate QSL adds is not banned until listed" understates the RT ban gap: two existing QSL library crates, `qsl-analyze` and `qsl-walk-grow`, are already absent from RT's 14 bans today. (`quire-semantic-value` is rightly absent, since RT is to take it.) | spec/assurance/AD-007-cross-repo-dependency-graph.md:155-156 |

## Verdict

The edge graph, the cycle-free claims, the ticket-premise contradiction, the
124-name count and the O-1 figures all reproduce. FND-001 and FND-002 must be fixed:
both misstate what guards IR today, and IR-343's scope depends on them. The low
findings are table corrections.

## New findings (disposition pass 1)

Re-reviewed at agent-ix/quire-contract-ir@568adea9219264dec13494a3b3fdb10174784845. I re-read `tc_041` (`tests/it/cycle_free_model.rs:36-90`). The new G-1 description is accurate. The test checks the root package by `qsl-` prefix and by QSL git source and requires the model edge. It checks the model package against a fixed name list. Each check covers every dependency kind that `cargo metadata` lists. IR `deny.toml` has `unknown-git = "deny"`, a four-repository `allow-git` list and no `exclude-dev`, as decision B now says. Decision B's description of what IR-343 has left to catch is right: path edges, and crates of an allowed repository, pass the source check.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-008 | low | The diamond row says "IR's `scripts/check_one_copy.awk` make[s] the lock hold one entry per repo", but the diamond is in CG's lock, and IR's script reads only IR's lock. CG runs its own `scripts/check_one_copy.awk` over its own `Cargo.lock` (CG `Makefile:113`). Attribute the guard to CG's check. The text was there in the first draft and I missed it in the review pass. Wording only. | spec/assurance/AD-007-cross-repo-dependency-graph.md:153 |

## New findings (disposition pass 2)

Re-reviewed at agent-ix/quire-contract-ir@495aba4c183aa5ba371bccbb7038e34970e5dbbf. IR (`Makefile` `deny`), CG (`Makefile:113`) and RT (`Makefile:180`, over every tracked `Cargo.lock`) each run `scripts/check_one_copy.awk` over their own lock, as the diamond row and G-4 now say.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-009 | low | The three `scripts/check_one_copy.awk` files in IR, CG and RT are the same 9-line script (byte-identical on each main). The diamond row and G-4 now describe "each repository's own copy" as the guard. Under this AD's own rule ("a row with two owners or a copy is a finding"), and the no-copy-between-repos rule, it is a copy, and the concept-owner table does not record it. Record it as a finding row with no owner named, and route the owner question (a shared home, or a check each repository owns differently); propose no exception. Not a correctness issue; tracking and consistency. | spec/assurance/AD-007-cross-repo-dependency-graph.md:153, spec/assurance/AD-007-cross-repo-dependency-graph.md:198-199 |

## New findings (disposition pass 3)

Re-reviewed at agent-ix/quire-contract-ir@88288896ba424288948e40886f49426f92f7720b. `scripts/check_one_copy.awk` hashes the same on IR, CG and RT `main` (9 lines); QSL has no such file. The new owner-table row, the O-4 open question (addressed to the owners of IR, CG, RT and the ecosystem tooling; no exception, no shared copy proposed, no decision), the gap row, the diamond row and G-4 are consistent with one another and with this AD's copy rule. `quire validate` exits 0 and strict coverage is 23.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-010 | medium | The QSL half of the one-copy lock guard does not exist. G-4 ("QSL `arch-lint duplicate-revisions`"), the owner-table row ("QSL uses a different check, `arch-lint duplicate-revisions`"), the diamond row ("QSL has its own") and AD-005's direction table all cite a QSL check that QSL deleted on its `main` in QSL-477 (#614, merged 2026-10-02, before this AD's 2026-10-04 measurement). `tools/arch-lint` has no `duplicate_revisions.rs`, and the Makefile has no target for it. QSL's only remaining duplicate guard is cargo-deny `deny-multiple-versions` for `quire-canonical` alone (`deny.toml` line 28). The claim was in the first draft, and I missed it in the review pass. Correct each place, state that QSL has no general one-entry-per-first-party-crate check today, and route that gap to QSL (R-1 already leans on G-4 for QSL's lock). | spec/assurance/AD-007-cross-repo-dependency-graph.md:135, spec/assurance/AD-007-cross-repo-dependency-graph.md:154, spec/assurance/AD-007-cross-repo-dependency-graph.md:199-201, spec/assurance/AD-005-qsl-consumption-seam.md:104 |

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 568adea9219264dec13494a3b3fdb10174784845: G-1 now says `tc_041` checks the root by prefix and source and the model by a fixed name list, which a `qsl-foundation` or `quire-exact` model edge passes |
| FND-002 | fixed | 568adea9219264dec13494a3b3fdb10174784845: decision B and the gap row record the `unknown-git` / `allow-git` guard and narrow IR-343 to name-level bans and the model by-name check |
| FND-003 | fixed | 568adea9219264dec13494a3b3fdb10174784845: the `qsl-semantics` dev cell, the four crates with no workspace dependency, and the `ix-trace-rs` exceptions now match the manifests |
| FND-004 | fixed | 568adea9219264dec13494a3b3fdb10174784845: 23 files, `quire-contract-runtime-footprint`, the fetch row no longer names IR, and 33 MiB is stated as a lower bound |
| FND-005 | fixed | 568adea9219264dec13494a3b3fdb10174784845: `jcs_sha256` is added to the canonical-bytes row, the gap row and R-3 |
| FND-006 | fixed | 568adea9219264dec13494a3b3fdb10174784845: the compile-side row records the root crate's `CheckedPackage<'a>` until M-6c |
| FND-007 | fixed | 568adea9219264dec13494a3b3fdb10174784845: the text names `qsl-analyze` and `qsl-walk-grow` as already unbanned, and the RT routing asks for them to be added |
| FND-008 | fixed | 495aba4c183aa5ba371bccbb7038e34970e5dbbf: the diamond row and G-4 attribute the one-copy check to each of IR, CG and RT over its own lock |
| FND-009 | fixed | 88288896ba424288948e40886f49426f92f7720b: the owner table, gap row and O-4 record the byte-identical script as a copy, with no owner named and no exception or shared copy proposed; the diamond row and G-4 call it a finding, not the design |
| FND-010 | fixed | 54602f9eb24fdb753d29d5145493b44600f66beb: G-4, the owner-table row, the diamond row and AD-005's "two copies" row now say QSL has no general one-copy check and only `deny-multiple-versions` on `quire-canonical` (QSL `deny.toml` line 28, re-measured), and R-1 routes the gap to QSL. Re-measured on QSL main: `tools/arch-lint` has no `duplicate_revisions.rs`; the Makefile arch-lint targets are direction, api-surface, api-surface-qsl, canonical-encoder and qualified-core; `reviews/qsl-477-code-review.md` (SR-1258) records the deletion |
