---
id: SR-1393
title: "dependency review of PR 289 (AD-007 edges, cycles and extraction options)"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-contract-ir@cb94d57e427dd1154a442b8ba7ba0461e99443e1; spec/assurance/AD-007-cross-repo-dependency-graph.md (repo-level edges, cycles, O-1 options and recommendation, routed gaps R-1..R-4); measured read-only from origin/main Cargo.toml and Cargo.lock of quire-spec-language and quire-contract-codegen, QSL ADR-011 'Differences from today' table, and QSL spec and test sources, fetched 2026-10-04"
review_set: subset
---
# SR-1393: dependency review of PR 289

## Summary

Ticket: IR-346. The cycle claims hold: there is no repo-level or crate-level cycle
over normal, dev or build edges; IR's root has no QSL edge (the ADR-011 T-5 row is
not present); the QSL dev edges to CG and to a historical IR are absent from every
QSL manifest (all dev tables checked) and from QSL's Cargo.lock; the CG diamond
(IR directly and through `qsl-package`; `quire-exact` through RT and through
`qsl-replay`) is real and IR's `check_one_copy.awk` and QSL's arch-lint keep one
lock entry per repo. The AD records O-1 as an owner decision, recommends without
deciding, and leaves order to QSL and RT. The defects are a missed `quire-rs` edge
that couples R-1's two items, an overclaim in option 3, and costs missing from the
option table.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The `quire-rs` edge is described as QSL `qsl-source` -> `quire-rs`, "N, optional (`quire-extraction`)", but `quire-rs` is always in QSL's and CG's graphs: `filament-core-data`'s `agent-ix-extraction-frontend` (a non-optional dependency of `qsl-semantics`) depends on `quire-rs`, at the same `rev` (both Cargo.lock files show it). The table has no `filament-core-data` -> `quire-rs` edge. R-1 asks QSL to justify or move the two `rev` edges independently, but moving only `qsl-source`'s `quire-rs` to `branch = "main"` would put two `quire-rs` entries in the lock (one `rev`, one branch), breaking G-4. Add the edge, say `quire-rs` is mandatory transitively, and state in R-1 that the two edges move together (or `filament-core-data` moves first). | spec/assurance/AD-007-cross-repo-dependency-graph.md:71-72, spec/assurance/AD-007-cross-repo-dependency-graph.md:244 |
| FND-002 | medium | Option 3 claims "cycles through QSL become structurally impossible", and the "risk of staying" row puts RT and CG in the same position. CG keeps a normal edge to `qsl-replay` (and so to 8 QSL crates) under every option, so a later QSL dev edge to CG for fixtures would still close a cycle after extraction. Extraction removes the risk for RT only. As written the option table credits option 3 with a benefit it does not give; correct both rows. | spec/assurance/AD-007-cross-repo-dependency-graph.md:212, spec/assurance/AD-007-cross-repo-dependency-graph.md:218 |
| FND-003 | medium | The option costs omit real parts of the extraction, so "The cost is QSL's and is one change" understates option 3: (a) the requirements and trace move: 85 QSL spec files name `quire-exact`, and its tests carry trace tags against QSL requirements, which either move with the crate or become cross-repo traces; (b) the new repository's own gates (CI, `deny`, spec validation and coverage), not only the two `no_std` gates; (c) RT gains a quire-canonical edge when it takes `quire-semantic-value` (that crate depends on it), under any option, which the "What RT takes" rows do not show. Add them so the owner weighs the real cost. | spec/assurance/AD-007-cross-repo-dependency-graph.md:205-226 |
| FND-004 | low | R-2 asks QSL to restate ADR-011's "Differences from today" rows for the CG and historical-IR dev edges, but the same table's row "QSL tests -> RT (fixture crate, IT-010 generated crates)", also "Removed with SEAM-4 ... M-6c", is equally absent today (QSL's RT and CG mentions are doc comments; no generated crate names RT). Include that row in R-2 so QSL restates all three at once. | spec/assurance/AD-007-cross-repo-dependency-graph.md:245 |

## Verdict

The graph, the absence claims and the diamond are correct, and O-1 is framed as an
owner decision with a recommendation, not a decision. FND-001 to FND-003 must be
fixed before merge: R-1 as written would lead QSL to a two-copy lock, and the option
table overstates option 3's benefit and understates its cost. The recommendation
itself (option 3) may still be right once corrected; that is the owner's call.

Owner decisions and the downstream tickets: O-1 blocks neither IR-345 nor IR-349.
IR-345 designs RT's layout against `quire-exact` and `quire-semantic-value` as the
owners, which ADR-011 already fixes; where those crates live changes only RT's git
URL and its `deny.toml` bans, not its layout. IR-349 can delete `src/exact` against
the crates where they are now; if the owner takes option 3 later, RT's edge changes
URL in one line. The AD's own risk (two moves in flight) is a sequencing preference,
not a blocker. O-2 (CG's lane, AD-006) and O-3 (stack growth) block neither.
RT's FR-275 on main lists the residue as vendored code with no exception, expiry or
approval, with deletion order tracked by QSL-358 and IR-349; IR-349's ordering
against QSL-358 is the real sequencing constraint, not O-1.

## New findings (disposition pass 1)

Re-reviewed at agent-ix/quire-contract-ir@568adea9219264dec13494a3b3fdb10174784845. On the spec-footprint count, both figures are true and differ only in method, measured on QSL main:
- Files under `spec/` naming `quire-exact`: 74. With `quire_exact` included as well: 85, which was my review-pass figure.
- Markdown files repository-wide: 129 naming `quire-exact` (147 with either spelling), and 41 naming `quire-semantic-value` (54 with either).
- Files in `quire-exact` carrying `#[trace]`: 11 (21 with any `tc_` reference). `quire-semantic-value` has 1.

The AD states its method ("name `quire-exact`"), and its figures match that method. QSL-358 (Done) is the RT-residue ticket whose stated goal is code RT needs living in QSL `no_std` leaf crates, so "QSL-358 plans that RT takes `quire-semantic-value`" is a fair reading. The option table is now fair: option 3's benefit is limited to RT, CG's `qsl-replay` edge is stated as unchanged under every option, the fixed costs appear in options 2 and 3, and the quire-canonical edge appears under every option.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | Two wording points in the re-derived O-1 text. (a) The recommendation says option 2 pays the fixed costs "again" when RT takes `quire-semantic-value`. If `quire-semantic-value` later joins the same new repository, the new repository's gates are not paid twice; only the QSL edge change and the trace move repeat. "Partly again" is accurate. (b) The footprint row mixes scopes (files under `spec/` for `quire-exact`, Markdown repository-wide for `quire-semantic-value`) and omits `quire-semantic-value`'s one traced file. State one scope for both. Neither changes the recommendation. | spec/assurance/AD-007-cross-repo-dependency-graph.md:233, spec/assurance/AD-007-cross-repo-dependency-graph.md:243-248 |

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 568adea9219264dec13494a3b3fdb10174784845: the `filament-core-data` -> `quire-rs` edge is added, and R-1 links the two `rev` edges (move both together, `filament-core-data` first) |
| FND-002 | fixed | 568adea9219264dec13494a3b3fdb10174784845: option 3's benefit and the stay-risk row now cover RT only; CG's `qsl-replay` edge stays under every option |
| FND-003 | fixed | 568adea9219264dec13494a3b3fdb10174784845: the spec and trace footprint, the new repository's gates and RT's quire-canonical edge are in the option table; the 74 versus 85 difference is method (see above) |
| FND-004 | fixed | 568adea9219264dec13494a3b3fdb10174784845: R-2 and the absence paragraph include ADR-011's "QSL tests to RT" row |
| FND-005 | fixed | 495aba4c183aa5ba371bccbb7038e34970e5dbbf: option 2's row and the recommendation say the gates repeat only for a second repository; the footprint row uses one scope (QSL `spec/`: 74 `quire-exact`, 22 `quire-semantic-value`, both re-counted). It still omits `quire-semantic-value`'s one traced file, which is immaterial to the comparison |
