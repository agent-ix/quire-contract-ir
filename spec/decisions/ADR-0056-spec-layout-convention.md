---
id: ADR-0056
title: "Subsystem spec layout, registry format and ID-block allocation for Contract IR, Codegen and Runtime"
type: ADR
status: accepted
owner: kreneskyp
relationships:
  - target: ix://agent-ix/quire-contract-ir/AD-001
    type: relates_to
  - target: ix://agent-ix/quire-rs/FR-050
    type: relates_to
  - target: ix://agent-ix/quire-contract-ir/FR-345
    type: relates_to
---
# ADR-0056: Subsystem spec layout, registry format and ID-block allocation

## Status

Accepted. Supersedes nothing.

This record is the one layout convention that the `quire-contract-ir`,
`quire-contract-codegen` and `quire-contract-runtime` specification trees
follow. It lives in this repository because Contract IR is the base of that
pipeline. Codegen and Runtime reference it as
`ix://agent-ix/quire-contract-ir/ADR-0056` and do not copy it.

## Context

The three repositories organise specifications by artifact kind: `spec/index.md`
as the master, then directories such as `functional/`, `nonfunctional/`,
`interface/`, `contract/` and `test/`, and one or two repository-wide matrices.
Requirements from unrelated parts of each crate sit side by side, and the directory says nothing about which
module owns a requirement or which matrix verifies it.

The public `ecaz` tree already shows the target shape in part:
`spec/functional/index.md` groups functional requirements by bounded context
while keeping each `FR-###` identifier, and `spec/matrix/` holds a module
TestMatrix beside the root matrix.

Quire supports this layout without a subsystem concept, as described under
[Tool support](#tool-support).

## Decision

### Directory layout

```text
spec/
├── spec.md                 # master-requirements: Subsystem Registry, ID Blocks
├── tests.md                # TestMatrixIndex: Requirements Traceability
├── core/                   # shared primitives and cross-subsystem invariants
│   ├── stakeholder/        # StR-###
│   ├── usecase/            # US-###
│   ├── functional/         # FR-###, STD-### owned by this subsystem
│   ├── non-functional/     # NFR-###
│   ├── integration/        # IT-###
│   └── matrix/             # tests.md (TestMatrix) and the TC-### it declares
├── <subsystem>/            # same six directories as core/
├── assurance/              # AD-### architecture descriptions and AA/AP/CAC/MP/SUR
├── decisions/              # ADR-#### decision records
├── program/                # repository-wide policy (PGM-##) and its standards
├── evidence/               # evidence suite registries
└── relocations/            # one relocation map per structural change
plan/                       # plans and tasks
reviews/                    # every SpecReview and review record
```

1. `spec/spec.md` is the only master-requirements document. No subsystem has a
   `spec.md`; a subsystem is a directory, not a sub-specification.
2. A subsystem directory holds only the six kind directories above. A kind
   directory exists only when it holds a file. Interface requirements are
   functional requirements and live in `functional/`.
3. Subsystems are derived from the repository's module architecture (its
   AD-001 and the crate or module boundaries it names). One subsystem maps to one
   or more crates or top-level modules; one crate or module maps to exactly one
   subsystem.
4. `core` holds the primitives and invariants that two or more subsystems
   build on: shared identity, error, digest and version types, and
   properties that hold across subsystems. A requirement that one subsystem
   alone consumes belongs to that subsystem. A `core` requirement depends on no
   other subsystem's requirement. A repository with a single subsystem has no
   `core`.
5. A standard (`STD-###`) that registers one subsystem's vocabulary lives in
   that subsystem's `functional/`; a registry every subsystem consumes lives in
   `core/functional/`.
6. Architecture descriptions (`AD-###`) and the assurance artifacts attached to
   them (`AA`, `AP`, `CAC`, `MP`, `SUR`) live in `spec/assurance/`. They span
   subsystems; the Subsystem Registry links each subsystem to the ADs that
   govern it.
7. Decision records (`ADR-####`) live in `spec/decisions/`, whichever
   subsystem they decide for.
8. SpecReviews and review records live in the repository-root `reviews/`.
   `spec/reviews/` does not exist. A review is a record of the revision it
   names: its body is not edited when the files it reviewed move.

### Subsystem Registry

`spec/spec.md` carries a `### Subsystem Registry` table with exactly these
columns, one row per subsystem directory, `core` first. Example:

| Subsystem | Path | Role | Owning crates/modules | ADs | Owner |
| --- | --- | --- | --- | --- | --- |
| Core | `spec/core/` | Shared identity, error and digest primitives; cross-subsystem invariants | `example-model::identity` | AD-001 | Example lane |

- **Subsystem** is the display name; **Path** is the directory, ending in `/`.
- **Role** is one sentence stating what the subsystem owns.
- **Owning crates/modules** names every crate or top-level module whose code
  implements the subsystem's requirements. The union over all rows is every
  crate and top-level module in the workspace, each named once.
- **ADs** lists the `AD-###` artifacts (and `ADR-####` records) that govern the
  subsystem.
- **Owner** names the one lane or person that writes the subsystem's
  specifications.

The registry and the directory tree agree: every `spec/<subsystem>/` has one
row, every row has a directory.

### Requirements Traceability

`spec/tests.md` is typed `TestMatrixIndex`. It carries a
`## Requirements Traceability` table with the columns
`Subsystem | Requirements | Local Matrix | Status`, one row per subsystem.
Example:

| Subsystem | Requirements | Local Matrix | Status |
| --- | --- | --- | --- |
| Core | StR-900, FR-900, FR-901, NFR-900 | `core/matrix/tests.md` | 🚧 two criteria unbacked |

**Requirements** lists every requirement ID in the subsystem's directory and no
other. **Status** starts with one of ✅ ❌ 🚧 ⛔. Cross-subsystem integration
scenarios are rows of the optional `## Integration Test Matrix` table in the
same document.

### Identifiers

1. IDs are flat and global per repository per family: one `FR` sequence, one
   `NFR` sequence, one `TC` sequence, and so on, across every subsystem. An ID
   never carries a subsystem, crate or module prefix (`FR-041`, never
   `FR-CORE-041`). The subsystem is the directory.
2. An ID is never renumbered by a move and never reissued after deletion. A
   file keeps its ID, its slug and its content when its directory changes.
3. A family keeps the digit width it already uses in the repository
   (`ADR-0056`, `FR-041`, `TC-058`); a move never re-pads an ID.
4. An unprefixed ID names an artifact in the same repository. A reference to
   another repository's artifact names the repository: frontmatter
   relationships use `ix://agent-ix/<repo>/<ID>`, and every other place —
   prose, matrix cells, trace tags and code comments — writes
   `<repo>:<ID>` with the full repository name (`quire-specification:TC-280`,
   `quire-specification:FR-340`). No short repository name stands in for the
   prefix.
5. The next free ID in a family is one above the highest of: every ID of that
   family declared in a spec artifact's frontmatter on the default branch, and
   the upper bound of every block recorded for that family.
6. A spec artifact is a document under `spec/` outside `spec/reviews/`: the
   StR, US, FR, NFR, IT, TC, AD, AP, ADR and STD artifacts, the other assurance
   and program artifacts, and the matrices. The identifier rules, the ID checks
   and the relocation-map ID columns apply to spec artifacts and to the `TC`
   rows the matrices declare. Plans under `plan/` and reviews under `reviews/`
   or `spec/reviews/` carry their own identifiers and are not checked.

### ID blocks for parallel authors

When two or more authors write specifications in one repository at the same
time, each writes only IDs from a block issued to it.

1. The lead that dispatches the parallel authors allocates the blocks before
   any author starts. A block is a contiguous range of one family, starts at
   the first multiple of ten at or above the next free ID, and spans a multiple
   of ten IDs.
2. The lead records every block in the `## ID Blocks` table of `spec/spec.md`
   in one commit on the default branch, before dispatch. Example:

   | Family | Range | Holder | Status |
   | --- | --- | --- | --- |
   | FR | FR-900..FR-919 | `example-branch` | open |
   | TC | TC-900..TC-949 | `example-branch` | open |

   **Holder** is the branch or work item the block is issued to. **Status** is
   `open` while the holder may allocate from it and `closed` once the holder's
   change merges.
3. An author allocates from its own block, lowest first, and never outside it.
   An author that needs more IDs asks the lead for a further block.
4. Unused IDs in a closed block are not reissued; the next-free rule already
   starts above them.
5. A single author with no concurrent spec writer in the repository uses the
   next free ID and records no block.

### Collision handling

A collision is two spec artifacts declaring the same frontmatter `id`, or two
matrix rows declaring the same `TC` in a Test Case Summary. `make spec` fails on
either, naming both files.

1. The claim that lies outside a block recorded for its author is the one that
   changes. When both or neither lie inside such a block, the claim that reached
   the default branch later changes.
2. The changing artifact takes a new ID from its holder's block, or the next
   free ID, and every reference to it in the repository is updated in the same
   commit.
3. The renumbering is recorded as a row in a relocation map (below) with
   distinct old and new IDs. The retained artifact keeps the ID; the abandoned
   number is never reissued.

### Matrices

1. Each subsystem has exactly one matrix, `spec/<subsystem>/matrix/tests.md`,
   typed `TestMatrix`, with its own `TM-###` ID. It carries the
   `## Functional Requirement Coverage` and `## Test Case Summary` tables the
   `TestMatrix` archetype requires.
2. The root `spec/tests.md` is typed `TestMatrixIndex` and indexes the
   subsystem matrices. It declares no test case.
3. Each requirement has one primary matrix: the matrix of the subsystem whose
   directory holds it. Another subsystem's matrix or the root index may cite the
   requirement in an integration row; that citation is never a second coverage
   authority.
4. Every live `TC` is declared by exactly one matrix, the matrix of its
   subsystem. A `TC-###` artifact file lives in the `matrix/` directory beside
   the matrix that declares it.
5. A withdrawn `TC` keeps its artifact file in the `matrix/` directory of the
   subsystem that owned the requirement it verified, and that matrix lists it
   in a `## Withdrawn Test Cases` section with the reason. It has no Test Case
   Summary row, so no matrix declares it, and its ID is never reissued.
6. An existing matrix keeps its `TM` ID wherever it moves. A new root index or
   a new subsystem matrix takes the next free `TM` ID.

### Restructure gate

A structural change moves existing artifacts into this layout. It is a pull
request of its own, and it passes every check below before it merges.

1. **Structural first.** It adds no requirement, acceptance criterion, test
   case, AD, ADR or standard, and no ID of those families. The only IDs it adds
   are the `TM` IDs of the new root index and of new subsystem matrices. New
   requirements in the repository are authored after it merges, in the new
   layout; a requirement the structural change itself needs, such as the
   repository-local check below, is authored and merged in a spec pull request
   before it.
2. **One writer.** While it is open, no other branch changes `spec/` in that
   repository. Every other spec branch rebases onto it after it merges.
3. **Relocation map.** It adds `spec/relocations/<YYYY-MM-DD>-<slug>.tsv`: a
   tab-separated file whose header row names the columns `old_path`,
   `new_path`, `old_id` and `new_id`, then one row per moved or added matrix
   file, sorted by `old_path` then `new_path`, paths relative to the repository
   root.
   - A moved file carries its ID in both ID columns; a file that is not a spec
     artifact, or has no frontmatter ID, carries `-` in both.
   - A new root index or subsystem matrix carries `-` as `old_path` and
     `old_id`, and its new `TM` ID as `new_id`.
   - Only a collision renumbering records two different, non-`-` IDs.
   - Every file whose path changed has exactly one row, and every row's
     `new_path` exists at the head.
4. **ID-set equality.** Take the spec-artifact frontmatter IDs at the merge
   base, replace each `old_id` by its `new_id` for every map row where both are
   IDs, and add every `new_id` whose `old_id` is `-`: the result is identical
   to the set at the head. The set of live `TC` IDs declared across all
   matrices is identical before and after, and each is declared exactly once.
5. **No meaning change.** Moves are Git renames. In a moved requirement,
   standard or test case file, every changed line differs from its original
   only inside Markdown link targets `](...)`. A file whose bytes code digests
   (`include_bytes!` into a digest) moves as an exact-byte rename with no edit
   at all, and the including path is updated in the same change. Matrix rows
   moved between matrices are carried verbatim, apart from link targets.
6. **Link check.** Every relative Markdown link in `spec/`, `plan/` and the
   repository-root documents resolves at the head. `ix://` targets name IDs,
   not paths, and rule 4 keeps them resolving. Bodies under `reviews/` are
   exempt: a review links the revision it names, and the relocation map
   resolves its old paths.
7. **Embedded-path scan.** Every occurrence of a `spec/` path outside `spec/`
   and `reviews/` — `include_str!`, `include_bytes!`, directory enumeration in
   tests, path lists in scripts, fixtures that build a spec tree, sealed
   assurance inputs, `Makefile` and CI targets, and links in repository-root
   documents — is found by a repository-wide search and either still names an
   existing path or is updated in the same pull request. The pull request lists
   each hit and its disposition.
8. **Same inputs, green gates.** Every check that enumerates spec files — a
   fixed directory list, a non-recursive glob, a `read_dir`, an
   `include_str!`/`include_bytes!` reader, a sealed configuration path — reads
   the same set of documents and IDs at the head as at the merge base, apart
   from the added root index and subsystem matrices listed in the relocation
   map with `old_id` `-`; the pull request records both counts per check. `make ci` passes at the head, and
   `quire coverage --scope . --strict` reports the same minted-ID set and the
   same backed-row count before and after.
9. **Prefixed foreign IDs.** Every foreign ID in a trace tag, code comment,
   matrix cell or prose line carries its `<repo>:` prefix at the head, so that
   no foreign ID can bind to a local ID the next-free rule later issues. A
   trace tag citing a withdrawn local ID is removed.

### Tool support

What Quire does and does not provide for this layout:

- `quire validate` checks each document against the archetype named by its
  `type:` frontmatter. It has no subsystem field, reads no directory as meaning,
  and accepts a document in any directory under `spec/`. It does not report two
  files declaring the same `id`.
- `quire coverage --scope <DIR>` takes the repository root. It reads documents
  from the whole `<DIR>/spec` tree, nested directories included, and trace tags
  from `<DIR>` excluding `spec/`. A test case is minted by any document typed
  `TestMatrix`, wherever it sits, so `spec/<subsystem>/matrix/tests.md` mints its
  rows (`quire-rs` FR-050, change CR-062). There is no per-subsystem scope: a
  subsystem directory passed as `--scope` has no `spec/` inside it and is
  refused. One report covers the repository; per-subsystem status is read from
  the subsystem matrices.
- The `TestMatrixIndex` archetype validates the root index's
  `Requirements Traceability` columns and mints no test case. The Subsystem
  Registry and ID Blocks tables in `spec/spec.md` are not validated by Quire;
  the registry-to-directory agreement is part of the restructure review.

### ID and relocation checks

The duplicate-ID, duplicate-TC, ID-block and relocation-map checks this record
requires belong in `quire validate`, which reads every repository the same way.
Until `quire validate` reports them, each repository runs its own small
repository-local check from `make spec`, specified in that repository's own
functional requirement ([FR-345](../functional/FR-345-check-artifact-ids-and-relocation-maps.md)
in this one). That requirement and its test case are authored and merged in a
spec pull request before the structural one; the check's code lands before or
with the structural pull request. No repository copies another repository's
check; when `quire validate` reports these defects, the local check is
deleted.

### Adoption by Codegen and Runtime

`quire-contract-codegen` and `quire-contract-runtime` adopt this layout the
same way as this repository. Each first merges a spec pull request specifying
its repository-local ID and relocation check (a new FR and TC in its own
sequences), then opens its structural pull request:

1. Its root `spec/spec.md` references
   `ix://agent-ix/quire-contract-ir/ADR-0056` and restates none of it.
2. `spec/index.md` becomes `spec/spec.md` and gains the Subsystem Registry and
   `## ID Blocks` tables. `spec/tests.md` is added as the `TestMatrixIndex`.
3. Its subsystems come from its own AD-001 module views. Its `spec/test/` case
   files move into the declaring subsystem's `matrix/`, and its
   `spec/test-matrix.md` rows move verbatim into the subsystem matrices; the
   existing matrix keeps its `TM` ID in the subsystem holding most of its rows.
4. `spec/nonfunctional/` becomes `non-functional/` and `spec/interface/`
   requirements move into `functional/`, per subsystem.
5. The structural pull request runs the already-specified check from
   `make spec` and adds no requirement or test case ID.
6. The structural pull request passes the restructure gate in its own
   repository, and IDs stay in that repository's own sequences.

### Adoption by this repository

The structural pull request for this repository moves `spec/index.md` to
`spec/spec.md` and the existing matrices TM-001 and TM-002 into subsystem
matrices, and meets rules 5, 7, 8 and 9. The sites below are a snapshot taken
when this record was accepted, not an exhaustive list: rules 7 and 9 govern,
and the structural pull request runs its own repository-wide search.

- **Readers of spec paths** (rules 7 and 8): `include_str!`, `read_dir` and path
  literals in `tests/it/{governance_reconciliation,governance,ecosystem_model,identity,output_mapping,conformance,foundation,toolchain_policy}.rs`;
  `include_bytes!` in `src/temporal/{request,admission}.rs`; the fixed,
  non-recursive `SPEC_DIRECTORIES` and matrix path list in
  `scripts/validate_matrix_status.py` and the fixture tree in
  `tests/test_matrix_status.py`; `tests/test_native_orchestration.py`;
  `assurance/change-assurance.json` sources and its `configuration`
  `spec/index.md`; spec links in `README.md` and `CONTRIBUTING.md`; spec paths
  in doc comments at
  `crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs:33`,
  `crates/quire-contract-model/src/output_mapping.rs:176`,
  `tests/it/checked_package_v2_frame_bodies.rs:107` and
  `tests/it/checked_package_v2_qsl_parameters.rs:42`.
  `scripts/assurance_chain.py` writes its own scratch spec tree and is
  unaffected.
- **Digest-bound files** (rule 5): FR-025 and FR-026, whose bytes
  `src/temporal/{request,admission}.rs` digest.
- **Unprefixed foreign IDs** (rule 9), each naming a quire-specification
  artifact without its prefix: the strings `FR-322` and `FR-340` in doc
  comments under `crates/quire-contract-model/src/checked_package/` and in
  `tests/it/checked_package_v2_*.rs`; `FR-322` at
  `tests/it/support/checked_package.rs:321,556,577,581,1168` and `FR-340` at
  `tests/it/support/checked_package.rs:461`;
  `FR-331` in `src/kani/outcome.rs`; and short-name prose such as
  `QSpec FR-340` in `spec/`.
- **Withdrawn trace tag** (rule 9): `#[trace("TC-221", "FR-031-AC-4")]` in
  `tests/it/kani_replay.rs`, citing the withdrawn TC-221.
- **Withdrawn test cases** (matrices rule 5): TC-046 and TC-054, today listed in
  prose in `spec/contract-test-matrix.md`.

## Consequences

- Every requirement's directory names the subsystem that owns it, the crates
  that implement it, and the one matrix that verifies it.
- Parallel authors cannot mint the same ID, and a collision that still occurs
  fails `make spec` naming both files instead of silently unbinding two tests.
- A restructure costs a spec pull request for the local check and then one
  structural pull request per repository, gated on ID-set equality and an
  embedded-path scan, and blocks other spec writers in that repository
  while it is open.
- Paths quoted in reviews and external notes go stale on the first move; the
  relocation maps under `spec/relocations/` resolve them.
- Coverage stays one repository-wide Quire report; a per-subsystem figure is
  read from the subsystem matrix, not from a Quire flag.

## Alternatives Considered

- **Keep the kind-first flat layout.** Hides subsystem ownership and forces one
  or two repository-wide matrices whose rows no single owner maintains.
- **Kind first, subsystem second (`spec/functional/<subsystem>/`).** Keeps a
  subsystem's requirements, cases and matrix in four separate trees, so the
  matrix cannot sit beside the requirements it verifies.
- **Per-subsystem ID prefixes (`FR-CORE-001`).** Makes a move a renumbering and
  breaks every trace tag and cross-repository reference to the old ID.
- **A `spec.md` per subsystem.** Creates several master documents and
  scattered scope boundaries for one crate workspace.
- **Next-free allocation for parallel authors.** Two branches read the same
  highest ID and allocate the same next one; blocks recorded on the default
  branch before dispatch remove the race.
