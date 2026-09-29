---
id: ADR-0056
title: "Subsystem spec layout and registry format for Contract IR, Codegen and Runtime"
type: ADR
status: accepted
owner: kreneskyp
relationships:
  - target: ix://agent-ix/quire-contract-ir/AD-001
    type: relates_to
  - target: ix://agent-ix/quire-rs/FR-050
    type: relates_to
---
# ADR-0056: Subsystem spec layout and registry format

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
├── spec.md                 # master-requirements: Subsystem Registry
├── tests.md                # TestMatrixIndex: Requirements Traceability
├── core/                   # shared primitives and cross-subsystem invariants
│   ├── stakeholder/        # StR-###
│   ├── usecase/            # US-###
│   ├── functional/         # FR-###, STD-### owned by this subsystem
│   ├── non-functional/     # NFR-###
│   ├── integration/        # IT-###
│   └── matrix/             # tests.md (TestMatrix) and the TC-### it declares
├── <subsystem>/            # same six directories as core/
├── assurance/              # AD-### architecture descriptions
└── decisions/              # ADR-#### decision records
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
6. Architecture descriptions (`AD-###`) live in `spec/assurance/`. They span
   subsystems; the Subsystem Registry links each subsystem to the ADs that
   govern it.
7. Decision records (`ADR-####`) live in `spec/decisions/`, whichever
   subsystem they decide for.
8. SpecReviews and review records live in the repository-root `reviews/`.
   `spec/reviews/` does not exist.

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
   file keeps its ID and its slug when its directory changes.
3. A family keeps the digit width it already uses in the repository
   (`ADR-0056`, `FR-041`, `TC-058`); a move never re-pads an ID.
4. An unprefixed ID names an artifact in the same repository. A reference to
   another repository's artifact names the repository: frontmatter
   relationships use `ix://agent-ix/<repo>/<ID>`, and every other place —
   prose, matrix cells, trace tags and code comments — writes
   `<repo>:<ID>` with the full repository name (`quire-specification:TC-280`,
   `quire-specification:FR-340`). No short repository name stands in for the
   prefix.
5. An author takes the next free ID; a collision found in review is
   renumbered in the later pull request.
6. A spec artifact is a document under `spec/` outside `spec/reviews/`: the
   StR, US, FR, NFR, IT, TC, AD, ADR and STD artifacts and the matrices. The identifier rules apply to spec
   artifacts and to the `TC` rows the matrices declare. Plans under `plan/` and
   reviews under `reviews/` or `spec/reviews/` carry their own identifiers.

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
5. An existing matrix keeps its `TM` ID wherever it moves. A new root index or
   a new subsystem matrix takes the next free `TM` ID.

### Restructure

A restructure pull request is `git mv` plus link fixes, and the reviewer reads
the diff; Git shows the renames.

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
  Registry table in `spec/spec.md` is not validated by Quire; the
  registry-to-directory agreement is part of the restructure review.

## Consequences

- Every requirement's directory names the subsystem that owns it, the crates
  that implement it, and the one matrix that verifies it.
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
