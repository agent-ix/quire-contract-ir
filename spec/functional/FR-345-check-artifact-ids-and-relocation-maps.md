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

`make spec` SHALL run a repository-local check that fails when two spec
artifacts claim one identifier, when a test case is declared by more than one
matrix row, when recorded ID blocks overlap, or when a relocation map disagrees
with the tree, and that, when the change adds a relocation map, fails it if its
identifier set or renames differ from that map. The identifier, block,
collision and relocation-map rules it enforces are those of
[ADR-0056](../decisions/ADR-0056-spec-layout-convention.md).

## Inputs

- Every spec artifact in the working tree, as ADR-0056 defines it (a Markdown
  document under `spec/` outside `spec/reviews/`), and its first frontmatter
  `id:` line. Files under `plan/`, `reviews/` and `spec/reviews/` are not
  read for identifiers.
- Every document typed `TestMatrix` under `spec/`, and the rows of its
  `## Test Case Summary` table.
- The `## ID Blocks` table of the master-requirements document, when present.
- Every file under `spec/relocations/`.
- A base revision named by `SPEC_BASE`, read from Git objects, when set.

## Outputs

- Exit status 0 and one summary line when every check passes.
- A non-zero exit status and one finding per defect on stderr, each naming the
  identifier and every `path:line` involved.

## Behavior

- When two spec artifacts declare the same frontmatter `id`, the check SHALL
  report the identifier and both paths.
- When one `TC` ID leads more than one `Test Case Summary` row, in one matrix
  or across matrices, the check SHALL report the identifier and each
  `matrix:line`.
- When two rows of the `## ID Blocks` table of one family cover a common ID, the
  check SHALL report both rows.
- For each relocation map, the check SHALL require the header row
  `old_path`, `new_path`, `old_id`, `new_id`; rows sorted by `old_path` then
  `new_path`; each `old_path` other than `-` at most once; each `new_path`
  present in the tree; where `new_id` is not `-`, the file at `new_path`
  declaring `new_id`; and, where `old_path` is `-`, `old_id` also `-` and
  `new_id` a `TM` ID.
- When `SPEC_BASE` is set and the change adds at least one relocation map, the
  check SHALL take the spec-artifact ID set at `SPEC_BASE`, replace each
  `old_id` by its `new_id` for every added-map row where both are IDs, and add
  every `new_id` whose `old_id` is `-`, and SHALL require the result to equal
  the spec-artifact ID set in the working tree. The check SHALL also require every
  file renamed between `SPEC_BASE` and the working tree to have exactly one row
  in an added map.
- The check SHALL compare identifier sets across revisions only when
  `SPEC_BASE` is set and the change adds a relocation map.
- A defect SHALL NOT be reported as a warning: every finding fails the check.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-345-AC-1 | Two spec artifacts declaring one frontmatter `id` fail the check with the identifier and both paths; the same identifier in body prose alone, or declared twice only by files under `plan/`, `reviews/` or `spec/reviews/`, does not. | Test (TC-224) |
| FR-345-AC-2 | A `TC` ID leading two `Test Case Summary` rows fails the check with the identifier and each `matrix:line`, both when the rows are in one matrix and when they are in two; citing the ID in another table does not. | Test (TC-224) |
| FR-345-AC-3 | Two `## ID Blocks` rows of one family with a common ID fail the check naming both rows; adjacent non-overlapping ranges and ranges of different families pass. | Test (TC-224) |
| FR-345-AC-4 | A relocation map with a wrong header, unsorted rows, a repeated non-`-` `old_path`, a `new_path` absent from the tree, a `new_path` whose file declares a different `id` than `new_id`, or an added-file row whose `old_id` is not `-` or whose `new_id` is not a `TM` ID fails the check with `path:line` for each defect. | Test (TC-224) |
| FR-345-AC-5 | With `SPEC_BASE` set and an added relocation map, a change that drops an ID, adds a non-`TM` ID, adds a `TM` ID with no map row, renumbers an ID with no map row, or renames a file with no map row fails the check; the same moves with a complete map pass, including a new root index and subsystem matrices listed with `old_id` `-` and a collision renumbering listed as an `old_id`/`new_id` pair. | Test (TC-224) |
| FR-345-AC-6 | With `SPEC_BASE` set and no added relocation map, an ordinary spec change that adds requirement IDs passes the check; a tree with none of the defects above exits 0 with one summary line; and `make spec` fails whenever the check fails. | Test (TC-224) |

## Dependencies

- **Decision**: [ADR-0056](../decisions/ADR-0056-spec-layout-convention.md)
  defines identifiers, ID blocks, collisions, matrices and relocation maps.
- **Stakeholder need**: [StR-003](../stakeholder/StR-003-reviewable-assurance.md);
  a duplicated identifier silently unbinds the tests that trace to it.
