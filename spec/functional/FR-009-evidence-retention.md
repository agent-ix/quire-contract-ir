---
id: FR-009
title: "Delegate current assurance records to Quoin and ix-flow"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---
# FR-009: Delegate current assurance records to Quoin and ix-flow

## Description

The governance contract shall delegate new retention/audit and human decisions
to Quoin and ix-flow respectively, exactly as specified by PGM-01-R09.

## Inputs

An explicit structured domain result, exact producer identities, limitations,
authoritative record references, and a human decision subject.

## Outputs

A Quoin-retained evidence reference plus a separately attributed ix-flow
release decision.

## Behavior

- This repository retains no revision-scoped evidence records of its own. The
  `evidence/` tree that held them, its external checksum files, its corrections
  and the schemas those records named are deleted, under the pre-stable release
  of the preservation constraint decided by the repository owner on 2026-09-02
  and recorded in
  [engineering-assurance#7](https://github.com/agent-ix/engineering-assurance/issues/7).
  Nothing replaces them: no new local record, envelope, verifier or retention
  authority is introduced in their place.
- No claim in this repository rests on those records. They were read and not
  rewritten for the duration of the migration, and they are now gone; nothing
  here asserts that a deleted record still verifies anything.
- Inconclusive, failed, skipped, and unavailable results remain explicit and
  distinct wherever they are produced; CI success does not approve a release.
- Current retention, integrity checking, and audit belong to Quoin, which
  accepts an already-produced structured domain result and never executes its
  producer. Quire exports static definitions without execution. ix-flow owns the
  attributed decision event.
- Published contract and temporal crates acquire no runtime dependency on
  Quire or Quoin. Exact pinned development-time CLIs are used at the boundary,
  as specified by [FR-022](./FR-022-shared-assurance-intake.md).

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-009-AC-1 | The solver fixture remains semantically `inconclusive` after successful schema validation. | Test (TC-006) |
| FR-009-AC-2 | Only the named human can close the release decision. | Inspection (TC-004) |
| FR-009-AC-6 | New retention/audit and human-decision references name Quoin and ix-flow while both Quire and Quoin remain non-executing. | Inspection (TC-025) |

### Retired criteria

`FR-009-AC-4` required the append-only correction record and the claim it
supersedes to remain byte-identical and readable. `FR-009-AC-5` required
historical PGM-01 records to remain byte-identical and readable through an
explicit lossy mapping. Both criteria were about the retained `evidence/` tree.
The repository owner released the evidence-preservation constraint for the
pre-stable phase on 2026-09-02 (see
[engineering-assurance#7](https://github.com/agent-ix/engineering-assurance/issues/7),
"Preservation constraint released for the pre-stable phase"), the tree is
deleted, and both criteria are retired rather than restated more weakly: there
is no retained record left to be byte-identical to, and no claim here survives
that depends on one. Neither identifier is reused.

`FR-009-AC-3` required a repository-local release verifier to enumerate the
`HEAD` tree, match input checksums, and select exactly one current record. That
verifier was the repository-local retention and integrity authority this
migration removes, so the criterion is retired rather than reassigned: intake,
integrity, and audit are Quoin's under
[FR-022](./FR-022-shared-assurance-intake.md), and its `unavailable` outcome is
a first-class state there rather than a local exit code. The identifier is not
reused.

## Dependencies

- **Governed by**: [PGM-01](../program/PGM-01-governance.md).
- The current assurance path is [FR-022](./FR-022-shared-assurance-intake.md).
