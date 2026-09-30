---
id: FR-017
title: "Check schema version and classify trace coverage"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-016
    type: depends_on
---
# FR-017: Check schema version and classify trace coverage

## Description

The library shall reject unsupported schema versions and classify artifact
traces as shallow, deep, uncovered, or orphaned.

## Inputs

Schema identity, current package identities, and artifact trace references.

## Outputs

A supported package or structured version diagnostic, plus a coverage class for
every current requirement and referenced artifact.

## Behavior

Schema 1.1 is the only supported version. The library registers no schema
migration; a later migration is introduced by its own requirement.

Wire version preflight reads only the top-level `schema_version` object before
semantic package decoding. A missing/malformed version retains the existing
grammar/`invalid_wire_format` or `invalid_schema_version` precedence. Any version
other than 1.1 fails `unsupported_schema_version`. No rejected version reaches
identifier, source, requirement, clause, dependency, or expression validation,
and no best-effort field interpretation occurs.

An artifact trace has a unique validated artifact ID, source span, target
requirement reference, and closed depth. The trace has its own source span, the
target reference retains its reference span, and a deep trace's digest retains
its digest-token span. `shallow` asserts an exact current requirement
reference. `deep` additionally carries the canonical current requirement
digest; a mismatch is `stale_trace_digest` at the digest-token span. Artifact
IDs must be unique within one classification input; every occurrence of a
duplicated ID is orphaned and `duplicate_artifact_trace` is emitted at each
later occurrence.

Classification resolves traces in authored order, then emits requirement rows
sorted by the composite key package namespace, requirement ID, and numeric
revision, in that order. Namespace and identifier comparison is Unicode scalar
value order and revision comparison is ascending unsigned numeric order.
Artifact rows sort by artifact ID in Unicode scalar value order. No locale,
host map order, or encoded-byte collation participates. A
valid shallow trace contributes `shallow`; a valid deep trace with a matching
digest contributes `deep`; deep dominates shallow for the same requirement; a
current requirement with neither is `uncovered`. Cross-package, missing,
stale-revision, duplicate-ID, and digest-mismatched artifacts are `orphaned`,
retain respectively `cross_package_reference`,
`orphaned_requirement_reference`, `stale_requirement_revision`,
`duplicate_artifact_trace`, or `stale_trace_digest`, and contribute no coverage.
Artifact orphan reasons are the closed values `cross_package`,
`missing_requirement`, `stale_revision`, `duplicate_artifact`, and
`digest_mismatch`.

The diagnostic-to-orphan-reason mapping is closed and exact:

| Diagnostic code | Artifact orphan reason |
|---|---|
| `cross_package_reference` | `cross_package` |
| `orphaned_requirement_reference` | `missing_requirement` |
| `stale_requirement_revision` | `stale_revision` |
| `duplicate_artifact_trace` | `duplicate_artifact` |
| `stale_trace_digest` | `digest_mismatch` |

Each unique artifact ID produces exactly one artifact row. If an ID occurs
more than once, the classifier collapses all its occurrences into that one
`orphaned`/`duplicate_artifact` row; it emits one
`duplicate_artifact_trace` diagnostic at every occurrence after the first and
retains those diagnostics in authored order. No occurrence of that ID can
contribute coverage.

Coverage returns a report plus ordered diagnostics instead of discarding valid
rows when some artifacts are orphaned. Requirement rows never use `orphaned`;
artifact rows never use `uncovered`. Diagnostics use the trace span and follow
authored trace order. Repeating or permuting unique trace inputs produces the
same sorted rows; diagnostics remain authored-order evidence for invalid input.
Package requirements, requirement clauses, and artifact-trace inputs are also
subject to FR-019's semantic node/depth/collection preflight before
canonicalization or coverage recursion begins.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-017-AC-1 | Version preflight accepts schema 1.1 and rejects every other major or minor with `unsupported_schema_version` before semantic interpretation. | Test (TC-017) |
| FR-017-AC-2 | Shallow, deep, uncovered, and each closed orphan reason have positive/negative fixtures; stale, missing, cross-package, duplicate, digest-mismatched, and over-limit inputs retain distinct diagnostics and cannot make a current requirement appear covered. | Test (TC-017, TC-018) |

## Dependencies

FR-011 defines revision identity; FR-016 defines stable digests.
