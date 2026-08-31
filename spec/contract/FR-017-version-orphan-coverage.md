---
id: FR-017
title: "Handle schema evolution and classify trace coverage"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-016
    type: depends_on
---
# FR-017: Handle schema evolution and classify trace coverage

## Description

The library shall reject unsupported schema versions, apply only registered
explicit migrations, and classify artifact traces as shallow, deep, uncovered,
or orphaned.

## Inputs

Schema identity, migration request, current package identities, and artifact
trace references.

## Outputs

A supported package or structured version diagnostic, plus a coverage class for
every current requirement and referenced artifact.

## Behavior

Schema 1.1 is current. Schema 1.0 remains a supported source version. The only
registered migration is `reference_body_1_0_to_1_1`; it preserves package,
requirement, clause, source, anchor, dependency, and reference semantics while
changing the schema version. Its immutable receipt contains the migration ID,
source and target versions, source package digest, and target package digest.
The source digest is recomputed before migration and the target digest after
validation. A migration whose actual source/target differs from its registered
edge fails `unregistered_migration` and produces no package or receipt.

Wire version preflight reads only the top-level `schema_version` object before
semantic package decoding. A missing/malformed version retains the existing
grammar/`invalid_wire_format` or `invalid_schema_version` precedence. An unknown
major fails `unsupported_schema_version`. Major 1 minor 0 or 1 is supported;
another minor fails `unregistered_migration`. No rejected version reaches
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
subject to FR-019's semantic node/depth/collection preflight before migration,
canonicalization, or coverage recursion begins.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-017-AC-1 | Version preflight rejects unknown majors and unregistered minor/migration paths before semantic interpretation; the registered 1.0-to-1.1 migration alone succeeds and its receipt binds source/target versions and digests. | Test (TC-017) |
| FR-017-AC-2 | Shallow, deep, uncovered, and each closed orphan reason have positive/negative fixtures; stale, missing, cross-package, duplicate, digest-mismatched, and over-limit inputs retain distinct diagnostics and cannot make a current requirement appear covered. | Test (TC-017, TC-018) |

## Dependencies

FR-011 defines revision identity; FR-016 defines stable digests.
