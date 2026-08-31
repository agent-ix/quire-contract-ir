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
- The release-only verifier independently enumerates every non-evidence file in the current `HEAD` tree, requires an exact checksum-key match, rejects non-ignored untracked inputs, and checks each digest against both the `HEAD` Git blob and current candidate file. The source revision is provenance only; an ancestor relationship is not required after a squash merge.
- Every retained output, including the external checksum file, matches its current `HEAD` Git blob.
- Exactly one record matches the current candidate.
- The evidence manifest is validated against the published `quire.pgm01-evidence/v1` Draft 7 schema, whose identity and digest are carried in the record.
- Inconclusive, failed, and skipped results remain explicit; CI success does not approve a release.
- An append-only `quire.evidence-correction/v1` record may supersede a false
  claim without rewriting the affected evidence bytes. The verifier validates
  and authenticates every correction, requires each affected record to exist,
  and rejects an affected record as support for a current decision.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-009-AC-1 | The solver fixture remains semantically `inconclusive` after successful schema validation. | Test (TC-006) |
| FR-009-AC-2 | Only the named human can close the release decision. | Inspection (TC-004) |
| FR-009-AC-3 | Evidence verification rejects incomplete, committed-added, or untracked input coverage; false or drifted input digests; output/checksum drift; unsafe paths; schema-invalid manifests; and ambiguous record selection without depending on source-revision ancestry. | Test (TC-013) |
| FR-009-AC-4 | A schema-valid, checksum-authenticated correction makes the verifier reject the affected review-pass record; malformed, unauthenticated, and dangling corrections fail closed. | Test (TC-022) |

## Dependencies

- **Governed by**: [PGM-01](../program/PGM-01-governance.md).
