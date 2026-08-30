---
id: FR-009
title: "Retain immutable evidence and human decisions"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---
# FR-009: Retain immutable evidence and human decisions

## Description

The governance contract shall retain revision-scoped evidence and keep automated results separate from human decisions exactly as specified by PGM-01-R09.

## Inputs

An immutable subject revision, command/tool/environment identities, individual results, limitations, and human decision record.

## Outputs

A revision-scoped content-addressed record and a separately authorized release decision.

## Behavior

- Each rerun mints a new `evidence/pgm-01-<short-sha>/` record rather than rewriting a prior record.
- An external checksum file covers the evidence manifest and every retained output.
- The release-only verifier requires checksums for every non-evidence file in the subject tree and checks each digest against both the exact subject-revision Git blob and the current candidate file.
- Every retained output, including the external checksum file, must match its current `HEAD` Git blob; a rerun selects the matching record with the closest ancestor subject.
- Inconclusive, failed, and skipped results remain explicit; CI success does not approve a release.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-009-AC-1 | The solver fixture remains semantically `inconclusive` after successful schema validation. | Test (TC-006) |
| FR-009-AC-2 | Only the named human can close the release decision. | Inspection (TC-004) |
| FR-009-AC-3 | Evidence verification rejects incomplete input coverage, false or drifted input digests, output/checksum drift, unsafe paths, and stale-record selection. | Test (TC-013) |

## Dependencies

- **Governed by**: [PGM-01](../program/PGM-01-governance.md).
