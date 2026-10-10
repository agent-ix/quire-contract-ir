---
id: FR-017
title: "Check schema version and classify trace coverage"
type: FR
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
grammar/`invalid_wire_format` or `invalid_schema_version` precedence. Any
well-formed version other than 1.1 fails `unsupported_schema_version`: at
`schema_version.major` when the major is not 1, otherwise at
`schema_version.minor`. No rejected version reaches
identifier, source, requirement, clause, dependency, or expression validation,
and no best-effort field interpretation occurs.

An artifact trace has a unique validated artifact ID, source span, target
requirement reference, and closed depth. The trace has its own source span and
the target reference retains its reference span. `shallow` and `deep` each
refer to the exact current package, requirement ID, and revision. The depth
records the trace's claimed evidence level; classification does not verify the
underlying artifact. A stale revision is orphaned by the same typed resolution
rule for either depth. No requirement digest or parallel freshness record is
retained in a trace. Artifact IDs must be unique within one classification
input; every occurrence of a duplicated ID is orphaned and
`duplicate_artifact_trace` is emitted at each later occurrence.

Classification resolves traces in authored order, then emits requirement rows
sorted by the composite key package namespace, requirement ID, and numeric
revision, in that order. Namespace and identifier comparison is Unicode scalar
value order and revision comparison is ascending unsigned numeric order.
Artifact rows sort by artifact ID in Unicode scalar value order. No locale,
host map order, or encoded-byte collation participates. A
valid shallow trace contributes `shallow`; a valid deep trace contributes
`deep`; deep dominates shallow for the same requirement; a
current requirement with neither is `uncovered`. Cross-package, missing,
stale-revision and duplicate-ID artifacts are `orphaned`, retain respectively
`cross_package_reference`, `orphaned_requirement_reference`,
`stale_requirement_revision`, or `duplicate_artifact_trace`, and contribute no
coverage.
Artifact orphan reasons are the closed values `cross_package`,
`missing_requirement`, `stale_revision`, and `duplicate_artifact`.

The diagnostic-to-orphan-reason mapping is closed and exact:

| Diagnostic code | Artifact orphan reason |
|---|---|
| `cross_package_reference` | `cross_package` |
| `orphaned_requirement_reference` | `missing_requirement` |
| `stale_requirement_revision` | `stale_revision` |
| `duplicate_artifact_trace` | `duplicate_artifact` |

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
| FR-017-AC-1 | Version preflight accepts schema 1.1; schemas 1.0, 1.2 and 2.0 each fail `unsupported_schema_version` at the specified path, and a zero major fails `invalid_schema_version`, all before semantic interpretation. | Test (TC-017) |
| FR-017-AC-2 | Shallow, deep, uncovered, and each closed orphan reason have positive/negative fixtures; stale revision, missing, cross-package, duplicate, and over-limit inputs retain distinct diagnostics and cannot make a current requirement appear covered. Deep traces name the current requirement revision and carry no digest. | Test (TC-017, TC-018) |

## Dependencies

FR-011 defines revision identity.
