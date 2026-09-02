---
id: FR-009
title: "Preserve historical evidence and delegate current assurance records"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---
# FR-009: Preserve historical evidence and delegate current assurance records

## Description

The governance contract shall keep historical revision-scoped records readable
without rewrite and delegate new retention/audit and human decisions to Quoin
and ix-flow respectively, exactly as specified by PGM-01-R09.

## Inputs

Immutable historical bytes or an explicit structured domain result, exact
producer identities, limitations, authoritative record references, and a human
decision subject.

## Outputs

A read-only historical compatibility view or Quoin-retained evidence reference,
plus a separately attributed ix-flow release decision.

## Behavior

- The `evidence/pgm-01-<short-sha>/` records, their external checksum files,
  the `evidence/corrections/` bytes, and the schemas those records name are
  closed historical inputs. Nothing writes to them, nothing appends to them, and
  no new record joins them.
- Those bytes are read only through the Engineering Assurance read-only
  compatibility mapping, which preserves the source digest and reports `lossy`,
  `incompatible`, or `unreadable` rather than synthesizing an absent field.
- Historical manifests, checksums, corrections, and source bytes are never
  rewritten into a new common schema; unknown fields remain unknown.
- Inconclusive, failed, skipped, and unavailable results remain explicit and
  distinct in the mapped view; CI success does not approve a release.
- The append-only `quire.evidence-correction/v1` record that supersedes a false
  claim remains readable in place. It is history, not an active gate: the
  correction bytes are locked and the claim it corrects is a documented fact
  about a merged candidate, not an input to a current decision.
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
| FR-009-AC-4 | The append-only correction record and the claim it supersedes remain byte-identical and readable without a repository-local verifier enforcing them. | Test (TC-022) |
| FR-009-AC-5 | Historical PGM-01 records remain byte-identical and readable through an explicit lossy mapping; no missing field or legacy verdict is synthesized. | Test (TC-024, TC-032) |
| FR-009-AC-6 | New retention/audit and human-decision references name Quoin and ix-flow while both Quire and Quoin remain non-executing. | Inspection (TC-025) |

### Retired criteria

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
