---
id: SR-980
title: "integrity review of PR 203 (ADR-0056, FR-345)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/decisions/ADR-0056-spec-layout-convention.md, spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md, spec/test-matrix.md, spec/index.md"
review_set: subset
---
# SR-980: integrity review of PR 203

Former id: SR-592 (cited by the marker on IR-314).

## Summary

Ticket: IR-314. Internal consistency and completeness of ADR-0056 against itself, FR-345 and the tooling it cites.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Structural PR must mint a TM ID for spec/tests.md (and new subsystem matrices), contradicting gate rules 1 and 4 and FR-345-AC-5. | spec/decisions/ADR-0056-spec-layout-convention.md:224-234 |
| FND-002 | high | Adoption step 5 has Codegen/Runtime specify a new FR/TC in the structural PR, which gate rule 1 forbids. | spec/decisions/ADR-0056-spec-layout-convention.md:309-325 |
| FND-003 | medium | Context section reproduces private-repo internals (script behaviour, incident history, unreleased v1 IA doc) in a public repo. | spec/decisions/ADR-0056-spec-layout-convention.md:37-51 |
| FND-004 | low | 'Registered short name' names no registry. | spec/decisions/ADR-0056-spec-layout-convention.md:160-164 |
| FND-005 | low | No rule places withdrawn TC artifacts TC-046/TC-054 that no matrix declares. | spec/decisions/ADR-0056-spec-layout-convention.md:230-232 |

## Finding Detail

**FND-001** (high, confidence high, soundness; spec/decisions/ADR-0056-spec-layout-convention.md:224-234, 241-243, 254-255): Every structural change must add spec/tests.md (a TestMatrixIndex, whose archetype requires frontmatter `id`: reproduced, validate fails 'id is a required property') and usually new subsystem matrices, each taking a new TM ID (Matrices rule 5). Gate rule 1 says the structural PR 'adds no ID' and gate rule 4 / FR-345-AC-5 require the spec-artifact ID set at base and head to be identical. So the restructure the ADR prescribes cannot pass its own gate, and FR-345's SPEC_BASE check will fail it. Fix: exempt the TestMatrixIndex and new-subsystem TM IDs explicitly (e.g. allow added IDs of family TM only, listed in the relocation map), or add them in a prior PR.

**FND-002** (high, confidence high, soundness; spec/decisions/ADR-0056-spec-layout-convention.md:309-325, 241-243): Adoption steps are listed as the content of Codegen's and Runtime's structural pull request ('each in its own structural pull request: ... 5. It specifies ... its own repository-local ID and relocation check'). Specifying it means a new FR and TC ID, which gate rule 1 ('adds no ID, no requirement ... no test case') forbids in that same PR. Fix: say the check's FR is authored and merged before the structural PR (as this repo does with FR-345), and only the code lands with or before the move.

**FND-003** (medium, confidence medium, other; spec/decisions/ADR-0056-spec-layout-convention.md:37-51): quire-contract-ir is public; filament-ide-rs and quire-specification are private (gh repo view). Beyond naming paths and ADR-002, the ADR reproduces a private script's behaviour, a private incident history (ID sequence forked eleven times, parallel TC collision) and the content of an unreleased quire-specification design document ('v1 spec information architecture'). Once merged it is in public history. Fix: state the rules as this ADR's own and cite only the public ecaz tree, or reduce the private bullets to repository names.

**FND-004** (low, confidence medium, ambiguous; spec/decisions/ADR-0056-spec-layout-convention.md:160-164): 'Registered short name' names no registry, so two authors cannot agree whether a short form (QSpec, QSL, IR) is allowed. Name where short names are registered or drop the alternative.

**FND-005** (low, confidence high, coverage; spec/decisions/ADR-0056-spec-layout-convention.md:230-232): spec/contract/TC-046-canonical-backend-replay.md and spec/contract/TC-054-kani-counterexample-replay-crossing.md are withdrawn TC artifacts that no Test Case Summary declares (only prose under TM-002, spec/contract-test-matrix.md:109-112). Matrices rule 4 gives them no directory and gate rule 4 counts only matrix-declared TCs, so the restructure has no rule for where they go. Add a rule for withdrawn TC artifacts.

## Scope

- `ADR-0056-layout` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: Directory layout rules 1-8: spec/spec.md is the only master-requirements document; subsystem directory holds six kind directories; subsystems derived from AD-001; core; STD placement; AD/assurance in spec/assurance/; ADRs in spec/decisions/; reviews in root reviews/, spec/reviews/ does not exist.
- `ADR-0056-registry` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: spec/spec.md carries a ### Subsystem Registry table with exactly these columns: Subsystem | Path | Role | Owning crates/modules | ADs | Owner, one row per subsystem directory, core first.
- `ADR-0056-traceability` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: spec/tests.md is typed TestMatrixIndex. It carries a ## Requirements Traceability table with the columns Subsystem | Requirements | Local Matrix | Status, one row per subsystem.
- `ADR-0056-identifiers` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: IDs are flat and global per repository per family ... never carries a subsystem prefix ... never renumbered by a move ... An unprefixed ID names an artifact in the same repository ... ix://agent-ix/<repo>/<ID> ... next free ID ... A spec artifact is a document under spec/ outside spec/reviews/.
- `ADR-0056-idblocks` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: When two or more authors write specifications in one repository at the same time, each writes only IDs from a block issued to it. ... recorded in the ## ID Blocks table of spec/spec.md in one commit on the default branch, before dispatch.
- `ADR-0056-collision` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: A collision is two spec artifacts declaring the same frontmatter id, or two matrix rows declaring the same TC in a Test Case Summary. make spec fails on either, naming both files.
- `ADR-0056-matrices` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: Each subsystem has exactly one matrix ... The root spec/tests.md is typed TestMatrixIndex ... Every TC is declared by exactly one matrix ... A new root index or a new subsystem matrix takes the next free TM ID.
- `ADR-0056-gate` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: Restructure gate 1-8: structural first (adds no ID), one writer, relocation map TSV old_path/new_path/old_id/new_id, ID-set equality, no meaning change, link check, embedded-path scan, gates unchanged.
- `ADR-0056-tools` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: quire validate ... does not report two files declaring the same id. quire coverage --scope <DIR> takes the repository root ... a subsystem directory passed as --scope ... is refused. The TestMatrixIndex archetype validates the root index's Requirements Traceability columns and mints no test case.
- `ADR-0056-adoption` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: quire-contract-codegen and quire-contract-runtime adopt this layout the same way as this repository, each in its own structural pull request: 1-6.
- `ADR-0056-context` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: Three existing trees already show the target shape: filament-ide-rs ..., quire-specification ..., ecaz ...
- `FR-345` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: make spec SHALL run a repository-local check that fails when two artifacts claim one identifier, when a test case is declared by more than one matrix row, when recorded ID blocks overlap, or when a relocation map disagrees with the tree, and that, given a base revision, fails a structural change whose identifier set or renames differ from its relocation map.
- `FR-345-AC-5` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: With SPEC_BASE set, a structural change that drops, adds or renumbers an ID, or renames a file with no row in an added relocation map, fails the check; the same moves with a complete, ID-preserving relocation map pass.

## Verdict

The layout, registry columns, TestMatrixIndex columns (checked against spec-artifacts-process mappings.yaml: Subsystem | Requirements | Local Matrix | Status), the flat-global identifier rule, the ix://agent-ix/<repo>/<ID> form (matches this repo's existing relationships), ID blocks and collision handling are coherent. The quire 0.33.0 claims reproduce on a scratch tree: `quire validate` exits 0 with two FR files declaring FR-001 and one matrix declaring TC-001 twice; `quire coverage --scope spec/core` is refused (no document root); a nested spec/core/matrix/tests.md mints its TC rows. Two rules contradict the restructure gate, and the context section reproduces private-repo internals.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | Rule 8's same-document-set requirement contradicts the added TM index/matrices that rules 1 and 3 now allow. | spec/decisions/ADR-0056-spec-layout-convention.md:275-279 |

**FND-006** (low, confidence medium, soundness; spec/decisions/ADR-0056-spec-layout-convention.md:275-279, 231-233): Rule 8 requires every enumerating check to read the same document and ID set before and after, but rule 1 and rule 3 now let the structural PR add a TestMatrixIndex and new subsystem matrices with new TM IDs. A matrix-enumerating check (validate_matrix_status.py STATUS_DOCUMENTS, quire's TestMatrix walk) necessarily reads more documents at the head, so rule 8 read literally fails every conforming restructure. Say 'the same set, apart from the added index and matrix rows of the relocation map'.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | resolved |
| FND-002 | fixed | resolved |
| FND-003 | fixed | resolved |
| FND-004 | fixed | resolved |
| FND-005 | fixed | resolved |
| FND-006 | fixed | resolved |
