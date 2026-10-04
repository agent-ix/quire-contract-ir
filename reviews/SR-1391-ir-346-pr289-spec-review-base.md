---
id: SR-1391
title: "spec review (base) of PR 289 (AD-007 cross-repo dependency graph)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@cb94d57e427dd1154a442b8ba7ba0461e99443e1; git diff origin/main...HEAD: spec/assurance/AD-007-cross-repo-dependency-graph.md, spec/spec.md (one index line); measured read-only against origin/main of quire-spec-language, quire-contract-ir, quire-contract-codegen, quire-contract-runtime, quire-specification, quire-canonical, quire-verification-contracts and quire-walk (gh api), fetched 2026-10-04"
review_set: subset
---
# SR-1391: base spec review of PR 289

## Summary

Ticket: IR-346. Spec-only PR. It adds AD-007, an ArchitectureDescription of the
Cargo graph across eight repositories, the owner of each shared concept, the cycle
analysis and the `quire-exact` / `quire-semantic-value` extraction question (O-1),
plus one index line in `spec/spec.md`. Review set: base plus integrity (SR-1392),
dependency (SR-1393) and scope-boundary (SR-1394). EARS, evidence and criterion
analyses do not apply: the AD adds no FR, AC or TC.

Measured at the reviewed sha:

- `quire validate --scope . 'spec/**/*.md' 'plan/**/*.md' 'reviews/**/*.md'`
  exits 0. The only grammar warning is the pre-existing FR-014 line 137.
- `quire coverage --scope . --strict` reports 23 unbacked rows at the PR head
  and 23 at origin/main (measured on both). Unchanged.
- Frontmatter and section layout follow AD-004/AD-005/AD-006: same keys
  (`id`, `type: ArchitectureDescription`, `status: proposed`, `owner`, `system`,
  `relationships`), same headings (System Boundary, Views, Decisions with
  Invariants, Risks with Current state and gaps, Open questions, Routed gaps).
- Every relationship target resolves: AD-001, AD-004, AD-005, AD-006, FR-028,
  FR-038, FR-039 exist in this repository; QSL ADR-011 exists on QSL main.
- No commit SHA or digest is recorded; measurements are dated and say
  "read-only". No vendoring or compatibility layer is proposed (Decision A
  rules them out). No quire-research internals, branch names or delivery order
  appear; sequencing is explicitly left to QSL and RT.
- The AD-005 staleness claim is confirmed: `digest_json` has no occurrence in
  `src/` or `crates/` on main, while AD-005 line 188 still says it "still has
  five production call sites". Leaving AD-005 unedited in this PR is acceptable
  scope discipline, provided the correction is tracked (FND-004).
- The visibility column is correct for all ten repositories named (QSpec is the
  only private one).

## Scope examined

- AD-007 frontmatter (examined).
- AD-007 Repo-level edges table and "Absent by measurement" paragraph (examined; SR-1392, SR-1393).
- AD-007 Crate-level edges table (examined; SR-1392).
- AD-007 Concept owners table (examined; SR-1392).
- AD-007 Cycles table (examined; SR-1393).
- AD-007 Decisions A-D and Invariants G-1..G-5 (examined; SR-1392, SR-1394).
- AD-007 Current state and gaps, O-1, O-2, O-3, Routed gaps R-1..R-4 (examined; SR-1393, SR-1394).
- spec/spec.md index line (examined, clean).
- AD-005 lines 73-90 and 180-195 (context only, for the digest_json claim).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The repo-level edge table records Cargo version requirements in prose (`=0.3.0` for quire-canonical, `=0.1.0` for QVC and for the IR model path edge). This repository's CLAUDE.md says package versions live in Cargo.toml and lockfiles only; the strings add nothing to the edge's kind or source and will drift on the next bump. Drop them; "git, `branch = \"main\"`" is the whole claim. | spec/assurance/AD-007-cross-repo-dependency-graph.md:74-76 |
| FND-002 | low | `relationships` lists AD-001, FR-028, FR-038 and FR-039, but the body never names any of them, so a reader cannot tell what each edge is for. The links are real (FR-028 is the cycle-free model `tc_041` backs; FR-038 is the one-encoder rule behind the `digest_json` row; FR-039 is the root-crate interface that still exports `KaniProvider*`; AD-001 states the RT/IR no-edge rule); cite each where it applies. | spec/assurance/AD-007-cross-repo-dependency-graph.md:8-22 |
| FND-003 | medium | "(IR-349; the residue exception is RT's)" asserts that an exception exists for RT's copied semantic-value code. RT's FR-275 on main says the opposite: every residue item "is vendored QSL code that violates the no-vendoring rule. There is no exception, no expiry and no approval" (FR-275 lines 46, 81, 170-171), with deletion tracked by QSL-358 and IR-349. The phrase also contradicts this AD's Decision A. Replace it with the fact: the copies are defects RT removes under IR-349 (and QSL-358); no exception exists. | spec/assurance/AD-007-cross-repo-dependency-graph.md:132 |
| FND-004 | low | Two follow-ups are recorded with no ticket: the AD-005 `digest_json` text correction ("AD-005 edit, not made here"; "IR-owned (no routing)") and CG's lock refresh ("lock refresh; informational"). Every other gap row names a ticket or a routing id. Untracked, the AD-005 correction is the kind that is never made. Name a ticket (or fold the AD-005 edit into IR-347) for each. | spec/assurance/AD-007-cross-repo-dependency-graph.md:193-194, spec/assurance/AD-007-cross-repo-dependency-graph.md:257-258 |

## Verdict

Structure, frontmatter, validation, coverage baseline and the public-repo rules are
clean. FND-003 (medium) must be fixed: the AD states an exception that RT's own spec
says does not exist. The other substantive defects are in the measured claims and
the extraction framing, recorded in SR-1392 (integrity), SR-1393 (dependency) and
SR-1394 (scope-boundary); six of those are medium. Not mergeable until the fix round
addresses them; all are text edits to AD-007.

## New findings (disposition pass 1)

Re-reviewed at agent-ix/quire-contract-ir@568adea9219264dec13494a3b3fdb10174784845 (fix commit over cb94d57). The fix also edits AD-005, which brings AD-005's corrected passages into this PR's scope. `quire validate` exits 0 (baseline FR-014 warning only); `quire coverage --strict` reports 23 unbacked rows (unchanged). Measured on IR main: `digest_json`, `CanonicalWriter` and `canonical_envelope_bytes` do not occur in `src/` or `crates/`, so the AD-005 encoder correction is true. The corrected D-2 bullet is true for a git edge, and `deny.toml` has no `exclude-dev`, so every dependency kind is covered.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | The CG-lock row now names "the lock move of IR-565 (CG's next lock bump)" as the owner of moving `quire-walk` off the QSL repository. IR-565's own text covers CG adapting to IR-495 on its next lock bump and does not mention `quire-walk`. CG's lock resolves `quire-walk` through the QSL crates at an old QSL revision, so only a refresh that also updates the QSL source re-resolves it; a targeted `cargo update -p quire-contract-ir` would not. Citing IR-565 as the owner overstates what that ticket commits to. Say it resolves on CG's next full lock refresh (IR-565's bump is the next planned one, but its text does not name this), or add the item to IR-565's text in the tracker first. Wording and tracking only. | spec/assurance/AD-007-cross-repo-dependency-graph.md:215, spec/assurance/AD-007-cross-repo-dependency-graph.md:281-282 |
| FND-006 | medium | AD-005 still contradicts its own corrected guard text in two places this PR left alone. (a) The dependency-direction table row "IR to codegen, IR to runtime / nothing in IR yet" is false now that the D-2 bullet says a git edge to those repositories fails `make deny` through `unknown-git`. (b) D-1 says the model and root packages are checked "by name and by source (existing: ... TC-041)", but the same AD's table (line 98) and `tc_041` itself check the model package by name only. A reader of AD-005 alone still gets the wrong guard state that SR-1392 FND-001/FND-002 corrected in AD-007. | spec/assurance/AD-005-qsl-consumption-seam.md:102, spec/assurance/AD-005-qsl-consumption-seam.md:146-147 |
| FND-007 | low | AD-005 encoder leftovers after the correction. (a) The bullet still opens "Two encoders:" but now lists none, and its first sentence reads as if the lint forbids encoders "in a repository the lint does not scan". (b) Decision C's "its code lands in three changes" and the open question "When does IR-274 land?" are stale: IR-274 is Done and the encoders are gone. (c) Decision C records a commit SHA for quire-canonical #7, which this repository's CLAUDE.md says to remove when found; this PR now edits AD-005. Wording only. | spec/assurance/AD-005-qsl-consumption-seam.md:125-131, spec/assurance/AD-005-qsl-consumption-seam.md:186-191, spec/assurance/AD-005-qsl-consumption-seam.md:207 |

## New findings (disposition pass 2)

Re-reviewed at agent-ix/quire-contract-ir@495aba4c183aa5ba371bccbb7038e34970e5dbbf. `quire validate` exits 0 and strict coverage is 23. Measured on QSL main: R3-Q1 is done (the root `Cargo.toml` and `qsl-package/Cargo.toml` key the dependency `quire-contract-model`, with no alias). The "emitted as numbers when this AD was written; whether QSL has followed is QSL's to say" sentence is honest: it is dated and defers to QSL, with no claim about today. IR-274 is Done in Linear, and its symbols are absent from IR `src/` and `crates/`.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-008 | low | AD-005's Current-state bullet still opens "D-2 has no check." The next sentence and the restated D-2 invariant (lines 149-152) say `unknown-git` in `make deny` holds D-2 today for a git edge. The opener is now false; say "D-2 has no by-name check." Wording only. | spec/assurance/AD-005-qsl-consumption-seam.md:182 |
| FND-009 | low | R-3 asks QVC to remove its second RFC 8785 path "or state why QVC must stay standalone" without citing VER-52 ("quire-verification-contracts: replace serde_json_canonicalizer with quire-canonical", Backlog), which already tracks exactly this. AD-005 cites VER-52. Cite it in R-3 and the gap row so the routed need points at its ticket. Tracking only. | spec/assurance/AD-007-cross-repo-dependency-graph.md:214, spec/assurance/AD-007-cross-repo-dependency-graph.md:274-276 |

## New findings (disposition pass 4)

Re-reviewed at agent-ix/quire-contract-ir@54602f9eb24fdb753d29d5145493b44600f66beb. Final sweep of AD-005 and AD-007, covering every claim about QSL, CG and RT tooling and every count. These hold on each `main`:

- QSL has `arch-lint` direction (outside `make ci`), canonical-encoder, api-surface-qsl and qualified-core (in `make ci`). QSL has the two `no_std` gates and TC-390.
- RT has 14 bans and runs `check_one_copy.awk` (`Makefile:180`). CG runs `check_one_copy.awk` (`Makefile:113`).
- All edge counts, the 124 shared names, the 23 files / 12,080 lines, the 74 and 22 footprint counts, R3-Q1, IR-274 and VER-52 match.

`quire validate` exits 0 and strict coverage is 23.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-010 | low | Several AD-005 source line references have drifted and now point at the wrong lines. They predate this PR but sit in a file the PR now edits throughout. `crates/quire-contract-model/src/lib.rs:31-47` (the seven globs are at lines 93-109); QSL `qsl-package/src/emit.rs:961` ("the pinned IR revision" is at 1082); QSL `checked_v2.rs` `for_ir` cited as `:223` (now 227); QSL `checked_v2.rs:595` (now a closing brace). Cite symbols (`for_ir`, the `pub use ...::*` block, the comment text) instead of line numbers, which drift on every edit. Wording only; no claim is false, only where to look. | spec/assurance/AD-005-qsl-consumption-seam.md:55, spec/assurance/AD-005-qsl-consumption-seam.md:91, spec/assurance/AD-005-qsl-consumption-seam.md:111, spec/assurance/AD-005-qsl-consumption-seam.md:172 |

## New findings (disposition pass 5)

Re-reviewed at agent-ix/quire-contract-ir@f7a71125a5e6a2bc98ca31ef86fa603d3522d451. AD-005 and AD-007 contain no remaining line-number citations of code (grepped both for `.rs:N`, `` `:N` ``, `.toml:N` and `Makefile:N`). Each replaced citation names something real on its repository's `main`, apart from FND-011:

- QSL `qsl-package/src/checked_v2.rs` defines `V2ReadIncomplete` and `for_ir`, and has the `use quire_contract_model::{read_checked_package, ...}` block.
- QSL `qsl-package/src/emit.rs` carries the "at the pinned IR revision" comment.
- In IR `tests/it/cycle_free_model.rs`: `let forbidden = [...]`, the loop over the root package's dependencies, and `tc_041_bridge_reexports_...`.
- IR `src/kani/outcome.rs` defines `KaniProvider*`, and `src/kani/mod.rs` re-exports them.
- The model `lib.rs` has seven `pub use ...::*` lines, and IR `src/lib.rs` has `pub use quire_contract_model::*;`.

`quire validate` exits 0 and strict coverage is 23.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-011 | low | This round's replacement for `checked_v2.rs:595` names the wrong file. The row says "the emitter builds the wire as a `Serialize` struct over them (... the wire-building code in `qsl-package/src/checked_v2.rs`)". On QSL main the emitter builds the wire structs in `qsl-package/src/emit.rs` (it constructs `CheckedPackageLockV2` and `CheckedPackageIdentityPreimageV2`). `checked_v2.rs` is the I2 reader; its only use of the identity-preimage type is `canonical_preimage`, which encodes the preimage for the `package_id` check. Introduced by this PR's own text (round-5 commit), so fix it here: cite `emit.rs` for the wire build, and `canonical_preimage` in `checked_v2.rs` if the reader-side use is meant. A wrong citation, not a wrong design claim. | spec/assurance/AD-005-qsl-consumption-seam.md:55 |

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 568adea9219264dec13494a3b3fdb10174784845: the `=0.3.0` and `=0.1.0` strings are gone from the edge table |
| FND-002 | fixed | 568adea9219264dec13494a3b3fdb10174784845: the body now cites FR-038 (canonical-bytes row), FR-039 (replay row), and AD-001 and FR-028 (decision B) |
| FND-003 | fixed | 568adea9219264dec13494a3b3fdb10174784845: the semantic-value row and decision A now state that no exception exists, matching RT FR-275 |
| FND-004 | fixed | 568adea9219264dec13494a3b3fdb10174784845: the AD-005 correction is made in this PR; the CG lock item now names an owner, but the citation is new finding FND-005 |
| FND-005 | fixed | 495aba4c183aa5ba371bccbb7038e34970e5dbbf: the CG-lock row and the CG routing now say the lock refresh comes with CG's next full refresh, and that IR-565 does not cover it |
| FND-006 | fixed | 495aba4c183aa5ba371bccbb7038e34970e5dbbf: the AD-005 direction row now records `unknown-git`; D-1 says `tc_041` checks the model by name list only and `unknown-git` covers source |
| FND-007 | fixed | 495aba4c183aa5ba371bccbb7038e34970e5dbbf: the bullet is retitled "One encoder" and its sentence repaired; decision C says IR-274 has landed; the IR-274 open question and the commit SHA are removed |
| FND-008 | fixed | 88288896ba424288948e40886f49426f92f7720b: the AD-005 bullet now opens "D-2 has no by-name check." |
| FND-009 | fixed | 88288896ba424288948e40886f49426f92f7720b: R-3 and the QVC gap row cite VER-52 (Backlog) |
| FND-010 | fixed | f7a71125a5e6a2bc98ca31ef86fa603d3522d451: every code line-number citation in AD-005 is replaced by a file and symbol name; none remain in AD-005 or AD-007 (one replacement is new finding FND-011) |
| FND-011 | fixed | dd322819354405b68133c7d0a22d0eb3b9ab0c96: AD-005 now cites `CheckedPackageLockV2` and `CheckedPackageIdentityPreimageV2` as built in QSL `qsl-package/src/emit.rs`, and `canonical_preimage` in `checked_v2.rs` for the reader side (both re-measured on QSL main ccea57c). The final grep of AD-005 and AD-007 finds no line-number citation and no SHA |
