---
id: FR-345
title: "Check artifact IDs and relocation maps before a spec change merges"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-003
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/ADR-0056
    type: references
---

# FR-345: Check artifact IDs and relocation maps before a spec change merges

## Description

`make spec` SHALL run a repository-local check that fails when two artifacts
claim one identifier, when a test case is declared by more than one matrix
row, when recorded ID blocks overlap, or when a relocation map disagrees with
the tree, and that, given a base revision, fails a structural change whose
identifier set or renames differ from its relocation map. This is the
mechanism [ADR-0056](../decisions/ADR-0056-spec-layout-convention.md) names for
this repository while `quire validate` does not report duplicate
declarations.

## Inputs

- Every spec artifact in the working tree, as ADR-0056 defines it (a Markdown
  document under `spec/` outside `spec/reviews/`), and its first frontmatter
  `id:` line. Files under `plan/`, `reviews/` and `spec/reviews/` are not
  read for identifiers.
- Every document typed `TestMatrix` under `spec/`, and the rows of its
  `## Test Case Summary` table.
- The `## ID Blocks` table of the master-requirements document, when present.
- Every file under `spec/relocations/`.
- Optionally, a base revision named by `SPEC_BASE`, read from Git objects.

## Outputs

- Exit status 0 and one summary line when every check passes.
- A non-zero exit status and one finding per defect on stderr, each naming the
  identifier and every `path:line` involved.

## Behavior

- The check SHALL read only the working tree and, when `SPEC_BASE` is set, the
  Git objects of that revision. It SHALL NOT use the network.
- When two spec artifacts declare the same frontmatter `id`, the check SHALL
  report the identifier and both paths.
- When one `TC` ID leads more than one `Test Case Summary` row, in one matrix
  or across matrices, the check SHALL report the identifier and each
  `matrix:line`.
- When two rows of the `## ID Blocks` table of one family cover a common ID, the
  check SHALL report both rows.
- For each relocation map, the check SHALL require the header row
  `old_path`, `new_path`, `old_id`, `new_id`, rows sorted by `old_path`, each
  `old_path` at most once, each `new_path` present in the tree, and, where
  `new_id` is not `-`, the file at `new_path` declaring `new_id`.
- When `SPEC_BASE` is set and every row of the relocation maps the change adds
  has equal `old_id` and `new_id`, the check SHALL require the frontmatter ID
  set of spec artifacts at `SPEC_BASE` to equal the set in the working tree,
  and every file renamed between `SPEC_BASE` and the working tree to have
  exactly one row in a relocation map the change adds.
- A defect SHALL NOT be reported as a warning: every finding fails the check.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-345-AC-1 | Two spec artifacts declaring one frontmatter `id` fail the check with the identifier and both paths; the same identifier in body prose alone, or declared twice only by files under `plan/`, `reviews/` or `spec/reviews/`, does not. | Test (TC-224) |
| FR-345-AC-2 | A `TC` ID leading two `Test Case Summary` rows fails the check with the identifier and each `matrix:line`, both when the rows are in one matrix and when they are in two; citing the ID in another table does not. | Test (TC-224) |
| FR-345-AC-3 | Two `## ID Blocks` rows of one family with a common ID fail the check naming both rows; adjacent non-overlapping ranges and ranges of different families pass. | Test (TC-224) |
| FR-345-AC-4 | A relocation map with a wrong header, unsorted rows, a repeated `old_path`, a `new_path` absent from the tree, or a `new_path` whose file declares a different `id` than `new_id` fails the check with `path:line` for each defect. | Test (TC-224) |
| FR-345-AC-5 | With `SPEC_BASE` set, a structural change that drops, adds or renumbers an ID, or renames a file with no row in an added relocation map, fails the check; the same moves with a complete, ID-preserving relocation map pass. | Test (TC-224) |
| FR-345-AC-6 | A tree with none of these defects exits 0 with one summary line, and `make spec` fails whenever the check fails. | Test (TC-224) |

## Dependencies

- **Decision**: [ADR-0056](../decisions/ADR-0056-spec-layout-convention.md)
  defines identifiers, ID blocks, collisions, matrices and relocation maps.
- **Stakeholder need**: [StR-003](../stakeholder/StR-003-reviewable-assurance.md);
  a duplicated identifier silently unbinds the tests that trace to it.
