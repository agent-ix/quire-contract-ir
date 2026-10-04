---
id: AD-007
title: "Cross-repo dependency graph: one owner per concept, no cycle, and the shared-kernel extraction question"
type: ArchitectureDescription
status: proposed
owner: kreneskyp
system: the Cargo dependency edges among quire-spec-language, quire-contract-ir, quire-contract-codegen, quire-contract-runtime, quire-specification, quire-canonical, quire-verification-contracts and quire-walk, the crate that owns each shared concept, and where the shared kernel crates live
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

AD-005 and AD-006 describe IR's two consumer seams; AD-004 describes the QSpec seam. This AD
describes the whole graph in one place, measured from each repository's manifests and lock on
`main` on 2026-10-04 (read-only), and states who owns each shared concept. It is IR-346. It
records measurements and the IR repository's position; it does not decide anything that
belongs to another repository, and it does not decide the extraction question (O-1).

## System Boundary

In scope: Cargo edges (normal, dev, build) between the eight repositories below and between the
crates inside the QSL workspace; the owner of each shared concept; repo-level and crate-level
cycles. Out of scope: third-party crates; what each crate does inside (their own ADs); the RT
layout (IR-345) and the CG and RT refactors (IR-349 and CG's own).

| Repo | Visibility | Crates (workspace) |
| --- | --- | --- |
| quire-spec-language (QSL) | public | root `quire-spec-language`, `qsl-attrs`, `qsl-foundation`, `qsl-cst`, `qsl-forms`, `qsl-semantics`, `qsl-package`, `qsl-eval`, `qsl-replay`, `qsl-route`, `qsl-source`, `qsl-analyze`, `qsl-bench`, `qsl-walk-grow`, `quire-exact`, `quire-semantic-value`, `xtask`, `tools/arch-lint`; `fuzz` is its own workspace |
| quire-contract-ir (IR) | public | `quire-contract-model`, root `quire-contract-ir` |
| quire-contract-codegen (CG) | public | `quire-contract-codegen` |
| quire-contract-runtime (RT) | public | `quire-contract-runtime`, `measurement/footprint` |
| quire-specification (QSpec) | private | `quire-specification-qualification` (dev edges only) |
| quire-canonical | public | `quire-canonical`, `quire-canonical-derive` |
| quire-verification-contracts (QVC) | public | `quire-verification-contracts` |
| quire-walk | public | `quire-walk` (no dependency beyond `core` and `alloc`) |

`quire-walk` is its own repository (not in the QSL workspace). Other first-party repositories
that appear as edges: `ix-trace-rs` (public, dev), `filament-core-data` (public) and `quire-rs`
(public).

## Views

The graph is described as repo-level edges, crate-level edges inside QSL, concept owners and cycles.

### Repo-level edges

"Rule" means the first-party `branch = "main"` rule (no `rev`, one lock entry per repo): yes,
or no with how it is declared. Kind: N normal, D dev, B build.

| From | To (crate) | Kind | Declared | Rule | Disposition |
| --- | --- | --- | --- | --- | --- |
| QSL root, `qsl-package` | IR (`quire-contract-model`) | N | git, `branch = "main"` | yes | keep (AD-005); the dependency key is now the crate's real name |
| QSL (7 crates) | quire-canonical | N | git, `branch = "main"`, workspace dependency | yes | keep |
| QSL `qsl-forms`, `qsl-semantics` | quire-walk | N | git, `branch = "main"`, workspace dependency | yes | keep |
| QSL (all crates) | `ix-trace-rs` | D | git, `branch = "main"` | yes | keep |
| QSL `qsl-semantics` | `filament-core-data` (`agent-ix-extraction-frontend`, `agent-ix-semantic-ir`) | N | git, `rev` | no | route-to-QSL: justify the `rev` or move to the rule (R-1) |
| QSL `qsl-source` | `quire-rs` | N, optional (`quire-extraction`) | git, `rev` | no | route-to-QSL: as above (R-1) |
| IR model | quire-walk | N | git, `branch = "main"` | yes | keep |
| IR model | quire-canonical (`serde_json` feature) | N | git, `branch = "main"`, `=0.3.0` | yes | keep |
| IR model | QVC | N | git, `branch = "main"`, `=0.1.0` | yes | keep |
| IR root | IR model | N | path and `=0.1.0` | n/a (same repo) | keep |
| IR root | IR model (`fault-injection`), quire-canonical, QVC, `ix-trace-rs` | D | path; git `branch = "main"` | yes | keep |
| IR model | `ix-trace-rs` | D | git, `branch = "main"` | yes | keep |
| CG | IR root (`quire-contract-ir`) and IR model (`quire-contract-model`) | N | git, `branch = "main"` | yes | keep the model edge; the root-crate edge is AD-006's bridge question (CG's lane) |
| CG | RT (`exact`) | N | git, `branch = "main"` | yes | keep |
| CG | QSL `qsl-replay` | N | git, `branch = "main"` | yes | keep (ADR-011 T-14 repoint is in place); heavy, see O-1 |
| CG | quire-canonical | N | git, `branch = "main"` | yes | keep |
| CG | RT (`proptest`, `exact`), QVC | D | git, `branch = "main"` | yes | keep |
| RT | QSL repo (`quire-exact`) | N, optional (`exact`) | git, `branch = "main"`, no default features | yes | justify only until O-1 is decided; unused by RT `src/` today |
| QSpec qualification | QVC, `ix-trace-rs` | D | git, `branch = "main"` | yes | keep |
| quire-canonical, QVC | no first-party edge | | | | none |
| quire-walk | `ix-trace-rs` | D | git, `branch = "main"` | yes | keep |

Absent by measurement: no IR manifest names QSL, CG or RT; no QSL manifest names CG, RT, QVC or
QSpec; no RT manifest names IR, CG or QSL beyond `quire-exact`; QSpec is no one's dependency. The
two QSL dev edges the ticket text lists (to CG and to a historical IR) are not in any QSL manifest
or its lock; the QSL table of edges (ADR-011, "Differences from today") still lists them as
removed with M-6c, which is QSL's to restate (R-2).

### Crate-level edges inside the QSL workspace

N unless marked. Every edge is a `path` dependency inside one workspace; none uses the
`branch = "main"` rule.

| Crate | Depends on (workspace crates) |
| --- | --- |
| `quire-exact`, `qsl-attrs`, `qsl-walk-grow`, `tools/arch-lint` | none |
| `quire-semantic-value` | `quire-exact` |
| `qsl-foundation` | `qsl-attrs`, `quire-exact` |
| `qsl-cst` | `qsl-attrs`, `qsl-foundation`, `quire-exact` |
| `qsl-forms` | `qsl-cst`, `qsl-foundation`, `quire-exact` |
| `qsl-semantics` | `qsl-attrs`, `qsl-forms`, `qsl-foundation`, `quire-exact`, `quire-semantic-value`; D: `qsl-cst`, own `test-support` |
| `qsl-package` | `qsl-attrs`, `qsl-foundation`, `qsl-semantics`, `quire-exact`, `quire-semantic-value`; D: `qsl-forms`, `qsl-cst` |
| `qsl-eval` | `qsl-attrs`, `qsl-foundation`, `qsl-package`, `qsl-semantics`, `quire-exact`, `quire-semantic-value`; D: `qsl-forms`, `qsl-cst` |
| `qsl-replay` | `qsl-attrs`, `qsl-cst`, `qsl-eval`, `qsl-forms`, `qsl-foundation`, `qsl-package`, `qsl-semantics`, `qsl-source` (optional), `quire-exact`, `quire-semantic-value` |
| `qsl-route` | `qsl-foundation`, `qsl-semantics`; D: `quire-exact` |
| `qsl-source` | optional `qsl-attrs`, `qsl-foundation` |
| `qsl-analyze` | `quire-exact` |
| root `quire-spec-language` | `quire-exact`, `quire-semantic-value`, `qsl-foundation`, `qsl-cst`, `qsl-forms`, `qsl-semantics`, `qsl-package`, `qsl-replay`, `qsl-source` (optional), `qsl-attrs`; D: `qsl-route`, `xtask`, `qsl-semantics` and `quire-exact` with `test-support` |
| `xtask` | `tools/arch-lint`, `qsl-attrs` |
| `qsl-bench` | `qsl-cst`, `qsl-foundation`, `qsl-forms`, `qsl-semantics`, `qsl-package`, `qsl-eval`, `qsl-replay`, `quire-exact`, `quire-semantic-value`, `qsl-attrs` |

No crate depends on `qsl-walk-grow`, and no crate depends on the root crate.

### Concept owners

One owning crate per concept. "Today" is what the manifests and sources show; a row with two
owners or a copy is a finding.

| Concept | Owner (target) | Today | Finding |
| --- | --- | --- | --- |
| Canonical bytes and their SHA-256 | quire-canonical | IR, CG, QSL use it; IR has no encoder of its own (`digest_json` is gone from IR) | two owners: QVC has its own RFC 8785 function (`jcs_canonicalize`, `jcs_equal`, on `serde_json_canonicalizer`); IR and CG call neither, QSpec's tests call it (R-3) |
| Raw-byte digest (no canonicalization) | QSL `qsl-foundation` (`ByteDigest`) | one owner | none: byte integrity, not canonical bytes |
| Walk (explicit-stack walker, arena) | quire-walk | IR model, `qsl-forms`, `qsl-semantics` use it | none |
| Stack growth | no owner named | `qsl-walk-grow` (QSL, depended on by no one) and a direct `stacker` and `serde_stacker` in IR model | two sites, no shared type; recorded, not a copy (O-3) |
| Exact scalar, text, composite kernel, `Meter`, `Outcome` | `quire-exact` | RT `src/exact` (24 files, 12,080 lines) is a second implementation | two owners; 124 `pub struct` or `enum` names are shared between RT `src/exact` and `quire-exact` plus `quire-semantic-value` (same-named, not proven identical), among them `Meter`, `Integer`, `Decimal`, `Text`, `Outcome`, `ScalarLimits`; RT's own edge to `quire-exact` is declared and unused (IR-349) |
| Semantic value (declaration, containment, unit, quantity, enumeration, origin and location, checking limits) | `quire-semantic-value` | RT holds same-named copies; no RT edge to it | two owners, as above (IR-349; the residue exception is RT's) |
| Replay, witness, envelope, terminal record, obligation identity | `qsl-replay` | IR root still exports `KaniProviderResult` and `KaniProviderRecord` | two owners until IR-347 removes them (AD-005 D-3) |
| Kani outcome type (`KaniOutcome`) | IR root crate | one owner | none |
| Checked-package wire types, strict reader, lowering | IR model (wire contract is QSpec's) | one owner | none |
| Compile-side checked package and emitter | `qsl-package` | one owner; its I2 reader delegates to IR's | none: `CheckedPackage` and `CheckedPackageV2` are different types |
| Authored-contract and expression model | IR model | consumed by QSL until its retirement | retiring with QSL ADR-011 M-6c (AD-005); no new owner |
| Verification contract schemas and types | QVC | IR model, IR root dev, CG dev, QSpec dev | none |
| Source evaluation | `qsl-eval` | one owner in QSL | RT `src/exact/expression.rs` is a second evaluator for generated code; boundary is IR-345's layout (not decided here) |
| Generated-code runtime (verdict, observation, accounting) | RT | one owner | none |

### Cycles

| Level | Cycle found | Evidence |
| --- | --- | --- |
| Repo, normal edges | none | the repo-level table: the longest chain is CG, RT, QSL, IR, QVC; quire-canonical, quire-walk and QVC have no first-party normal edge |
| Repo, dev and build edges | none | no repo has a dev edge to a repo that depends on it |
| Crate, QSL workspace | none | the crate-level table is a DAG over normal and dev edges; `quire-exact` is the only source |
| IR root to QSL (ADR-011 T-5 row) | not present | neither IR manifest names QSL; `tc_041` checks it by name and by git source |
| QSL dev edges to CG and a historical IR | not present | no QSL manifest or lock entry; routed as R-2 for QSL to restate |
| Diamond (not a cycle) | CG reaches IR directly and through QSL `qsl-package`; CG reaches `quire-exact` through RT and through `qsl-replay` | one copy per repo is required: the `branch = "main"` rule and IR's `scripts/check_one_copy.awk` make the lock hold one entry per repo |

A cycle is one `dev` edge away in two places, which the extraction decision (O-1) bears on:
QSL depending on RT or CG for fixtures, and any RT or CG crate depending on a QSL crate other
than the shared kernel. RT bans 14 QSL crate names in `deny.toml` as a snapshot; a crate QSL adds is
not banned until listed.

## Decisions

- A. Each concept in the owner table has one owning crate. A second implementation is removed in
  its owning repository, never copied or wrapped in a shim. No compatibility layer is proposed.
- B. IR depends on no QSL, CG or RT crate, in any kind. IR's own guard is `cargo deny` (IR-343,
  AD-005 decision D); it does not rely on QSL's lint.
- C. Edges between first-party repositories use `git` with `branch = "main"` and no `rev`.
  The two QSL edges declared by `rev` are routed (R-1).
- D. The extraction question is an owner decision (O-1), recorded below with a recommendation.
  This AD does not move any crate.

### Invariants a test can check

- G-1. No IR manifest names a QSL, CG or RT crate (existing: `tc_041` for QSL; target for CG
  and RT: IR-343).
- G-2. The workspace crate-level graph of every repository is acyclic over normal, dev and build
  edges (`cargo metadata`, one check per repository; not run in IR for other repositories).
- G-3. Every first-party git edge is `branch = "main"` with no `rev` (target; two exceptions,
  R-1).
- G-4. Each lock holds one entry per first-party crate (existing: IR `check_one_copy.awk`; QSL
  `arch-lint duplicate-revisions`).
- G-5. No public type is defined in two first-party crates (target; RT `src/exact` fails it, IR-349).

## Risks

What is measured today, what is open and with whom, and what is routed.

### Current state and gaps

| Item | Owner | Where |
| --- | --- | --- |
| RT `src/exact` duplicates `quire-exact` and `quire-semantic-value` (124 shared public names) | RT | IR-349 |
| IR root exports `KaniProviderResult` and `KaniProviderRecord` | IR | IR-347 |
| IR `deny.toml` has no `bans` for CG, RT or QSL crates | IR | IR-343 |
| QVC has a second RFC 8785 function | QVC, quire-canonical | R-3 |
| CG's `Cargo.lock` still resolves `quire-walk` from the QSL repository, which `main` of QSL no longer contains | CG | lock refresh; informational |
| AD-005's text on IR's own digest encoder (`digest_json`) is out of date: `digest_json` is gone | IR | AD-005 edit, not made here |

### Open questions

O-1 is an owner decision. This AD recommends; it does not decide.

**O-1. Where do `quire-exact` and `quire-semantic-value` live?** Measured:

| Fact | Value |
| --- | --- |
| Crates that depend on `quire-exact` | 12 inside QSL (root, analyze, bench, cst, eval, forms, foundation, package, replay, route as dev, semantics, semantic-value) and 1 outside (RT, optional) |
| Crates that depend on `quire-semantic-value` | 6 inside QSL; none outside declared; RT and CG are to take it (ADR-011 shared-leaf class) |
| First-party dependencies of the two crates | `quire-exact`: none; `quire-semantic-value`: `quire-exact` and quire-canonical |
| Size of what RT, CG and IR fetch with the QSL repository | 18 workspace members, 872 commits on `main`, 2,886 tracked files (about 33 MiB), against 23 and 15 files for the two crates |
| What RT takes from it | one package (`quire-exact`) and the cost of banning 14 other crates by name |
| What CG takes from it | 8 QSL crates, 2 `filament-core-data` crates and `quire-rs` through `qsl-replay`, plus `quire-exact` and `quire-semantic-value` |
| What IR takes from it | nothing (no edge) |
| Repo-level cycle risk of extracting | none: both crates would be leaves below QSL; the new repository would depend only on quire-canonical |
| Repo-level cycle risk of staying | RT and CG take normal edges into the repository that also holds the producer crates; a later QSL dev edge to RT or CG for fixtures would close a cycle |

| Option | What it costs | What it gives |
| --- | --- | --- |
| 1. Both stay in QSL | nothing now; RT's ban list and heavy fetch stay; each new QSL crate widens RT's ban list; RT and CG stay one QSL dev edge away from a cycle | no QSL change |
| 2. Extract `quire-exact` only | one QSL change (path to git edge in 12 crates, `test-support` dev edges, `arch-lint` and TC-390 amendments) and a second one later for `quire-semantic-value`, which RT needs next | RT's one edge becomes light |
| 3. Extract both, into one repository (two crates) or two | the QSL change above, once, with ADR-011's shared-leaf row amended; QSL's `quire-exact-no-std` and `quire-semantic-value-no-std` gates move with them; `test-support` becomes a git-sourced feature | RT and CG depend on leaf repositories like they do on quire-walk; RT's 14 bans and the QSL `allow-git` entry go; cycles through QSL become structurally impossible |

Recommendation: option 3. The graph is already shaped for it (both are `no_std` leaves with no
first-party dependency beyond each other and quire-canonical), `quire-walk` and quire-canonical
show the pattern, and option 2 repeats the same question when RT adopts `quire-semantic-value`.
The cost is QSL's and is one change; this repository holds nothing in it. The risk is that
extraction before RT's `src/exact` is deleted (IR-349) leaves two moves in flight; the order is
QSL's and RT's to set, not this AD's. Owner: kreneskyp decides; QSL carries the change
(R-4). Until decided, nothing here changes.

O-2. Does the IR root crate remain a dependency of CG? CG's lane (AD-006); this AD only records
the edge.

O-3. Is stack growth one concept? `qsl-walk-grow` is depended on by nothing and IR calls
`stacker` directly. Owner: quire-walk (a shared home for the growth call) or each repository
(accept two call sites). Low priority; no recommendation beyond not adding a dependency from IR
to QSL.

### Routed gaps

Needs stated to owners, not decisions. Ids are routing ids of IR-346.

To QSL (QSL reviews these rows):

| Id | Stated need |
| --- | --- |
| R-1 | The two `rev` edges (`filament-core-data`, `quire-rs`) are outside the `branch = "main"` rule: justify each or move it to the rule. |
| R-2 | The QSL dev edges to CG and a historical IR are not in any QSL manifest or lock; restate ADR-011's "Differences from today" rows and AD-016 WP9 as done or still open. |
| R-4 | Carry the extraction of `quire-exact` and `quire-semantic-value` if the owner accepts O-1 option 3, including the ADR-011 shared-leaf row and the arch-lint and TC-390 edits. |

To QVC and quire-canonical: R-3, remove the second RFC 8785 function from QVC in favour of
quire-canonical (no cycle: quire-canonical has no first-party dependency), or state why QVC must
stay standalone.

To RT: delete `src/exact` and the same-named semantic-value copies in favour of the shared crates
(IR-349); keep the `deny.toml` bans until O-1 is decided.

To CG: the root-crate edge (O-2, AD-006); refresh the lock.

IR-owned (no routing): removal of `KaniProvider*` (IR-347), the `bans` entries (IR-343), and the
AD-005 text correction above.
