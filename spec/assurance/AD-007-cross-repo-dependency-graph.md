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
| quire-contract-runtime (RT) | public | `quire-contract-runtime`, `quire-contract-runtime-footprint` (`measurement/footprint`) |
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
| QSL (every crate except `qsl-attrs` and `qsl-bench`) | `ix-trace-rs` | D | git, `branch = "main"` | yes | keep |
| QSL `qsl-semantics` | `filament-core-data` (`agent-ix-extraction-frontend`, `agent-ix-semantic-ir`) | N | git, `rev` | no | route-to-QSL, linked with the next row (R-1) |
| QSL `qsl-source` | `quire-rs` | N, optional (`quire-extraction`) | git, `rev` | no | route-to-QSL, linked (R-1) |
| `filament-core-data` (`agent-ix-extraction-frontend`) | `quire-rs` | N, in QSL's and CG's lock at the same `rev` as `qsl-source`'s edge | git, `rev` | no | so `quire-rs` is mandatory in QSL's and CG's graph, not only behind `quire-extraction`; the two `rev` edges move together (R-1) |
| IR model | quire-walk | N | git, `branch = "main"` | yes | keep |
| IR model | quire-canonical (`serde_json` feature) | N | git, `branch = "main"` | yes | keep |
| IR model | QVC | N | git, `branch = "main"` | yes | keep |
| IR root | IR model | N | path | n/a (same repo) | keep |
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
removed with M-6c, which is QSL's to restate (R-2); the same table's row "QSL tests to RT" is
equally absent from QSL's manifests.

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
| `qsl-semantics` | `qsl-attrs`, `qsl-forms`, `qsl-foundation`, `quire-exact`, `quire-semantic-value`; D: `qsl-cst`, `quire-exact` with `test-support` |
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
| Canonical bytes and their SHA-256 | quire-canonical | IR, CG, QSL use it; IR has no encoder of its own (`digest_json` and `CanonicalWriter` are gone from IR, as FR-038 requires) | two owners: QVC has its own RFC 8785 path (`jcs_canonicalize`, `jcs_equal`, `jcs_sha256`, on `serde_json_canonicalizer`); IR and CG call none of them, QSpec's tests do (R-3) |
| Raw-byte digest (no canonicalization) | QSL `qsl-foundation` (`ByteDigest`) | one owner | none: byte integrity, not canonical bytes |
| Walk (explicit-stack walker, arena) | quire-walk | IR model, `qsl-forms`, `qsl-semantics` use it | none |
| Stack growth | no owner named | `qsl-walk-grow` (QSL, depended on by no one) and a direct `stacker` and `serde_stacker` in IR model | two sites, no shared type; recorded, not a copy (O-3) |
| Exact scalar, text, composite kernel, `Meter`, `Outcome` | `quire-exact` | RT `src/exact` (23 files, 12,080 lines) is a second implementation | two owners; 124 `pub struct` or `enum` names are shared between RT `src/exact` and `quire-exact` plus `quire-semantic-value` (same-named, not proven identical), among them `Meter`, `Integer`, `Decimal`, `Text`, `Outcome`, `ScalarLimits`; RT's own edge to `quire-exact` is declared and unused (IR-349) |
| Semantic value (declaration, containment, unit, quantity, enumeration, origin and location, checking limits) | `quire-semantic-value` | RT holds same-named copies; no RT edge to it | two owners, as above: RT's copies are defects to delete under IR-349 (with QSL-358); no exception exists (decision A) |
| Replay, witness, envelope, terminal record, obligation identity | `qsl-replay` | IR root still exports `KaniProviderResult` and `KaniProviderRecord` | two owners until IR-347 removes them (FR-039, AD-005 D-3) |
| Kani outcome type (`KaniOutcome`) | IR root crate | one owner | none |
| Checked-package wire types, strict reader, lowering | IR model (wire contract is QSpec's) | one owner | none |
| Compile-side checked package and emitter | `qsl-package` | `qsl-package` owns `CheckedPackage` and `EmittedPackage`; its I2 reader delegates to IR's; the QSL root crate also defines a public `CheckedPackage<'a>` (`src/checking.rs`, over IR's authored-contract model) | a second same-named type until QSL's M-6c retires the root crate's lane (QSL's; not a copy of `qsl-package`'s); `CheckedPackageV2` is IR's wire type, a different one |
| Authored-contract and expression model | IR model | consumed by QSL until its retirement | retiring with QSL ADR-011 M-6c (AD-005); no new owner |
| Verification contract schemas and types | QVC | IR model, IR root dev, CG dev, QSpec dev | none |
| Source evaluation | `qsl-eval` | one owner in QSL | RT `src/exact/expression.rs` is a second evaluator for generated code; boundary is IR-345's layout (not decided here) |
| Generated-code runtime (verdict, observation, accounting) | RT | one owner | none |

### Cycles

| Level | Cycle found | Evidence |
| --- | --- | --- |
| Repo, normal edges | none | the repo-level table: the longest chain is CG, RT, QSL, IR, QVC; quire-canonical, quire-walk and QVC have no first-party normal edge |
| Repo, dev and build edges | none | no repo has a dev edge to a repo that depends on it |
| Crate, QSL workspace | none | the crate-level table is a DAG over normal and dev edges; `quire-exact`, `qsl-attrs`, `qsl-walk-grow` and `tools/arch-lint` have no workspace dependency |
| IR root to QSL (ADR-011 T-5 row) | not present | neither IR manifest names QSL; `tc_041` checks it by name and by git source |
| QSL dev edges to CG and a historical IR | not present | no QSL manifest or lock entry; routed as R-2 for QSL to restate |
| Diamond (not a cycle) | CG reaches IR directly and through QSL `qsl-package`; CG reaches `quire-exact` through RT and through `qsl-replay` | one copy per repo is required: the `branch = "main"` rule and each repository's own copy of `scripts/check_one_copy.awk` (IR, CG and RT each run one over their own lock) make each lock hold one entry per repo |

A cycle is one `dev` edge away in two places: QSL depending on RT or CG for fixtures, and any
RT or CG crate depending on a QSL crate other than the shared kernel. The second is guarded in
RT by 14 QSL crate names banned in `deny.toml`, a snapshot of QSL's members: `qsl-analyze` and
`qsl-walk-grow` are already absent from it, and a crate QSL adds later is not banned until listed.
The extraction decision (O-1) bears on RT's side only; CG keeps its normal `qsl-replay` edge
under every option.

## Decisions

- A. Each concept in the owner table has one owning crate. A second implementation is removed in
  its owning repository, never copied or wrapped in a shim. No compatibility layer is proposed.
  The RT copy of QSL code is such a defect: RT FR-275 records it as vendored code with no
  exception, no expiry and no approval, to be deleted under IR-349 (with QSL-358).
- B. IR depends on no QSL, CG or RT crate, in any kind (AD-001, FR-028). Two guards hold it
  today: `tc_041` and `cargo deny`. `[sources]` `unknown-git = "deny"` with an `allow-git` list
  of four repositories (ix-trace-rs, quire-canonical, QVC, quire-walk) runs in `make deny`, so a
  git edge to the QSL, CG or RT repository fails for both IR crates in every dependency kind. IR-343's
  scope is therefore what that does not catch: name-level `bans` (a crate of an allowed repo or
  a path edge) and a by-name check of the model crate. IR does not rely on QSL's lint.
- C. Edges from IR, CG and RT to other first-party repositories use `git` with
  `branch = "main"` and no `rev`: this is the ecosystem's first-party convention (the QSL
  Cargo comment cites it as org-wide, IR #225 and RT #88), recorded here as IR's position. IR's
  own edges conform. For QSL and the other repositories it is a recommendation routed to their
  owners (R-1), not a rule this AD sets.
- D. The extraction question is an owner decision (O-1), recorded below with a recommendation.
  This AD does not move any crate.

### Invariants a test can check

Targets proposed to each owning repository; only G-1 and G-4's IR halves are IR's to hold.

- G-1. No IR manifest names a QSL, CG or RT crate. Existing: `tc_041` checks the root package
  by `qsl-` name prefix and by QSL git source (and requires the model edge), and checks the
  model package against a fixed name list (`quire-spec-language`, `quire-contract-ir`,
  observation, protocol and TL crates) with no `qsl-*`, `quire-exact` or source check, so a model edge to
  `qsl-foundation` or `quire-exact` passes it; `cargo deny` `unknown-git` catches any git edge to
  those repositories (decision B). Target: the by-name check for the model crate and the CG and RT
  names (IR-343).
- G-2. The workspace crate-level graph of each repository is acyclic over normal, dev and build
  edges (`cargo metadata`, one check per repository; proposed to each owner, not run in IR for
  other repositories).
- G-3. Every first-party git edge is `branch = "main"` with no `rev` (holds for IR, CG and RT;
  proposed to QSL, R-1).
- G-4. Each lock holds one entry per first-party crate (existing: each of IR, CG and RT runs its own copy of
  `check_one_copy.awk` over its own lock; QSL `arch-lint duplicate-revisions`).
- G-5. No public type is defined in two first-party crates (proposed target; RT `src/exact`
  fails it, IR-349).

## Risks

What is measured today, what is open and with whom, and what is routed.

### Current state and gaps

| Item | Owner | Where |
| --- | --- | --- |
| RT `src/exact` duplicates `quire-exact` and `quire-semantic-value` (124 shared public names) | RT | IR-349 |
| IR root exports `KaniProviderResult` and `KaniProviderRecord` | IR | IR-347 |
| IR `deny.toml` has no name-level `bans` for CG, RT or QSL crates, and `tc_041` does not check the model crate by `qsl-*` name or source; a git edge to those repositories already fails `unknown-git` (decision B) | IR | IR-343 |
| QVC has a second RFC 8785 path (`jcs_canonicalize`, `jcs_equal`, `jcs_sha256`) | QVC, quire-canonical | R-3 |
| CG's `Cargo.lock` still resolves `quire-walk` from the QSL repository, which `main` of QSL no longer contains | CG | resolves on CG's next full lock refresh, which also updates the QSL source; IR-565 is CG's adaptation to IR-495 on its next bump and does not itself cover this |
| AD-005's text on IR's own digest encoder (`digest_json`, `CanonicalWriter`) and on the absence of any IR guard was out of date | IR | corrected in this change (AD-005 Current state and Identity bullets) |

### Open questions

O-1 is an owner decision. This AD recommends; it does not decide.

**O-1. Where do `quire-exact` and `quire-semantic-value` live?** Measured:

| Fact | Value |
| --- | --- |
| Crates that depend on `quire-exact` | 12 inside QSL (root, analyze, bench, cst, eval, forms, foundation, package, replay, route as dev, semantics, semantic-value) and 1 outside (RT, optional) |
| Crates that depend on `quire-semantic-value` | 6 inside QSL; none outside declared; RT and CG are to take it (ADR-011 shared-leaf class) |
| First-party dependencies of the two crates | `quire-exact`: none; `quire-semantic-value`: `quire-exact` and quire-canonical |
| What a Cargo git dependency on the QSL repository fetches | the whole repository, history included: 18 workspace members, 872 commits on `main`, 2,886 tracked files; the 33 MiB of tracked files at HEAD is a lower bound; the two crates are 23 and 15 files |
| What RT takes from it | one package (`quire-exact`) and the cost of banning 14 other crates by name; taking `quire-semantic-value` also gives RT an edge to quire-canonical (that crate depends on it), under every option |
| What CG takes from it | 8 QSL crates, 2 `filament-core-data` crates and `quire-rs` through `qsl-replay`, plus `quire-exact` and `quire-semantic-value`; under every option the `qsl-replay` edge stays |
| What IR takes from it | nothing (no edge) |
| Spec and trace footprint in QSL | under QSL `spec/`, 74 files name `quire-exact` and 22 name `quire-semantic-value`; 11 files in `quire-exact` carry trace tags against QSL requirements |
| Repo-level cycle risk of extracting | none: both crates would be leaves below QSL; the new repository would depend only on quire-canonical (and, for `quire-semantic-value`, on `quire-exact`) |
| Repo-level cycle risk of staying | RT takes a normal edge into the repository that also holds the producer crates; a later QSL dev edge to RT for fixtures would close a cycle; CG's is the same under every option (`qsl-replay`) |

| Option | What it costs | What it gives |
| --- | --- | --- |
| 1. Both stay in QSL | nothing now; RT's heavy fetch and 14-name ban snapshot stay and each new QSL crate widens it; RT is one QSL dev edge from a cycle; RT still gains the quire-canonical edge later | no QSL change, no new repository to gate |
| 2. Extract `quire-exact` only | a QSL change (path to git edge in 12 crates, `test-support` dev edges, arch-lint and TC-390 amendments); the spec and trace move or cross-repo trace for the requirements that name it; a new repository with its own CI, `deny` and spec gates; a second extraction later for `quire-semantic-value`, which RT needs next, either into the same new repository (its gates are then not paid twice, the QSL change and trace move are) or into another (gates paid twice) | RT's one edge becomes light |
| 3. Extract both, one repository (two crates) or two | the QSL change of option 2 once, with ADR-011's shared-leaf row amended; the same spec and trace move for both crates; the new repository's CI, `deny` and spec gates (for one repository, once); QSL's two `no_std` make gates move; `test-support` becomes a git-sourced feature | RT depends on leaf repositories as it would on quire-walk; RT's 14 bans and the QSL `allow-git` entry go; a QSL dev edge to RT no longer closes a cycle; CG's `qsl-replay` edge is unchanged |

Recommendation, re-derived with the costs above: option 3 still holds, but its margin is
smaller than the first draft claimed. Options 2 and 3 pay the same fixed costs (the QSL change,
the spec and trace footprint, a new repository's gates); option 3 pays them once, option 2 pays
the QSL change and trace move again when RT takes `quire-semantic-value` (and the gates again if
it goes to a second repository), and only option 1 avoids them. The deciding
question is therefore whether RT will take `quire-semantic-value` (QSL-358 plans that): if so,
option 3; if the owner expects RT to stay on `quire-exact` alone, option 1 or 2 is cheaper. The
benefit is RT's alone; CG's edge to QSL stays. The risk is that extraction before RT's
`src/exact` is deleted (IR-349) leaves two moves in flight; the order is QSL's and RT's to set.
Owner: kreneskyp decides; QSL carries the change (R-4). Until decided, nothing here changes.

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
| R-1 | The two `rev` edges (`filament-core-data` from `qsl-semantics`, `quire-rs` from `qsl-source`) are outside the `branch = "main"` convention IR, CG and RT follow. They are linked: `filament-core-data` itself depends on `quire-rs` at the same `rev`, so moving only `qsl-source`'s edge leaves two `quire-rs` lock entries (G-4). Justify the `rev` for both, or move both together (`filament-core-data` first). QSL's call. |
| R-2 | The QSL dev edges to CG and a historical IR, and ADR-011's "QSL tests to RT" row, are not in any QSL manifest or lock; restate those three "Differences from today" rows and AD-016 WP9 as done or still open. |
| R-4 | Carry the extraction of `quire-exact` and `quire-semantic-value` if the owner accepts O-1 option 3, including the ADR-011 shared-leaf row and the arch-lint and TC-390 edits. |

To QVC and quire-canonical: R-3, remove the second RFC 8785 path (`jcs_canonicalize`,
`jcs_equal`, `jcs_sha256`) from QVC in favour of quire-canonical (no cycle: quire-canonical has
no first-party dependency), or state why QVC must stay standalone.

To RT: delete `src/exact` and the same-named semantic-value copies in favour of the shared crates
(IR-349; QSL-358); keep the `deny.toml` bans until O-1 is decided, and add `qsl-analyze` and
`qsl-walk-grow`, which the 14 do not name.

To CG: the root-crate edge (O-2, AD-006); the lock refresh that moves `quire-walk` off the QSL
repository comes with CG's next full lock refresh (not covered by IR-565, which is CG's adaptation to IR-495).

IR-owned (no routing): removal of `KaniProvider*` (IR-347), the name-level `bans` entries and the
model-crate check (IR-343). The AD-005 correction (`digest_json` and `CanonicalWriter` gone, and
the source guard of decision B) is made in this change, in AD-005, which is IR's own document.
