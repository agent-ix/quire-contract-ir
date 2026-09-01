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

- Each rerun mints a new `evidence/pgm-01-<short-sha>/` record rather than rewriting a prior record.
- An external checksum file covers the evidence manifest and every retained output.
- The release-only verifier independently enumerates every non-evidence file in the current `HEAD` tree, requires an exact checksum-key match, rejects non-ignored untracked inputs, and checks each digest against both the `HEAD` Git blob and current candidate file. The source revision is provenance only; an ancestor relationship is not required after a squash merge.
- Every retained output, including the external checksum file, matches its current `HEAD` Git blob.
- Exactly one record matches the current candidate.
- Evidence verification exits 1 when candidate evidence is available but
  invalid, and exits 3 when candidate evidence cannot be evaluated because a
  required Git object or worktree input is unavailable.
- The evidence manifest is validated against the published `quire.pgm01-evidence/v1` Draft 7 schema, whose identity and digest are carried in the record.
- Inconclusive, failed, and skipped results remain explicit; CI success does not approve a release.
- An append-only `quire.evidence-correction/v1` record may supersede a false
  claim without rewriting the affected evidence bytes. The verifier validates
  and authenticates every correction, requires each affected record to exist,
  and rejects an affected record as support for a current decision.
- Historical manifests, checksums, corrections, and source bytes are never
  rewritten into a new common schema; unknown fields remain unknown.
- New shared-assurance adoption accepts an already-produced structured domain
  result. Quoin retains, integrity-checks, audits, and reports it without
  execution; Quire exports static definitions without execution; ix-flow owns
  the attributed decision event.
- Published contract and temporal crates acquire no runtime dependency on
  Quire or Quoin. Exact pinned development-time CLIs may be used at the
  boundary after the common-work release gate.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-009-AC-1 | The solver fixture remains semantically `inconclusive` after successful schema validation. | Test (TC-006) |
| FR-009-AC-2 | Only the named human can close the release decision. | Inspection (TC-004) |
| FR-009-AC-3 | Evidence verification rejects incomplete, committed-added, or untracked input coverage; false or drifted input digests; output/checksum drift; unsafe paths; schema-invalid manifests; and ambiguous record selection without depending on source-revision ancestry. It distinguishes invalid available evidence (exit 1) from unavailable evidence (exit 3). | Test (TC-013) |
| FR-009-AC-4 | A schema-valid, checksum-authenticated correction makes the verifier reject the affected review-pass record; malformed, unauthenticated, and dangling corrections fail closed. | Test (TC-022) |
| FR-009-AC-5 | Historical PGM-01 records remain byte-identical and readable through an explicit lossy mapping; no missing field or legacy verdict is synthesized. | Test (TC-024) |
| FR-009-AC-6 | New retention/audit and human-decision references name Quoin and ix-flow while both Quire and Quoin remain non-executing. | Test (TC-025) |

## Dependencies

- **Governed by**: [PGM-01](../program/PGM-01-governance.md).
