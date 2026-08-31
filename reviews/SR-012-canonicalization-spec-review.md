---
id: SR-012
title: "Issue 9 canonicalization, migration, and coverage specification review"
type: SpecReview
analysis: architecture-evaluation
scope: "FR-011, FR-016, FR-017, STD-001, NFR-001, NFR-003, TM-002, TC-017, TASK-008"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/9
    type: reviews
---
# Issue 9 canonicalization, migration, and coverage specification review

## Summary

Issue #9 defines one source-free canonical JSON profile, domain-separated
SHA-256 identities, one explicit 1.0-to-1.1 migration, fail-closed version
preflight, and deterministic coverage rows that cannot count orphaned traces.
It does not claim the issue #10 external corpus, cross-platform CI, or schema
publication work.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-085 | high | Current schema wording contradicted FR-011. | FR-011, FR-017 |
| FND-086 | high | Package canonical content did not bind schema version. | FR-016, FR-017 |
| FND-087 | medium | Orphan reasons lacked an exact diagnostic mapping. | FR-017, STD-001 |
| FND-088 | low | Unsupported-version wording overlapped zero-major grammar. | STD-001 |
| FND-089 | low | The NFR matrix omitted the issue #9 verification slice. | TM-002, NFR-001, NFR-003 |
| FND-090 | medium | Envelope example order contradicted key sorting. | FR-016 |
| FND-091 | medium | Deep digest mismatch had no exact span. | FR-017, STD-001 |
| FND-092 | medium | Duplicate artifact output cardinality was ambiguous. | FR-017 |
| FND-093 | medium | Canonical allocation failure lacked an owning acceptance criterion. | FR-016, STD-001, TC-017 |
| FND-094 | medium | Coverage row sorting lacked exact comparison and composite-key rules. | FR-017, NFR-001 |
| FND-095 | medium | The digest mismatch location used two different normative span names. | FR-017, STD-001 |
| FND-096 | medium | The task Plan Delta omitted the resource-failure harness required by FR-016-AC-3. | TASK-008, FR-016, TC-017 |

## Independent Spec Review Disposition

The first independent read-only pass reported five actionable findings,
FND-085 through FND-089. The producer audit then found three additional
ambiguities, FND-090 through FND-092. The closing attempt verified those eight
dispositions and found two new gaps, FND-093 and FND-094. The next closing
attempt found two remaining consistency gaps, FND-095 and FND-096. All twelve
are retained below; none is waived. The final independent read-only pass
verified every disposition against the normative text and returned exactly
`No actionable findings.` The task may proceed to Implement.

| ID | Severity | Finding | Disposition |
|---|---|---|---|
| FND-085 | high | FR-011 froze wire schema 1.0 while FR-017 declared 1.1 current. | FR-011 now distinguishes the issue #6 introduction from FR-017's current version and registered migration source. |
| FND-086 | high | Package canonical content did not explicitly bind the schema version, leaving migration receipt digests ambiguous. | FR-016 requires the major/minor object in package content and states that migration changes package bytes and digest. |
| FND-087 | medium | Closed orphan reasons and diagnostics had no normative one-to-one mapping. | FR-017 now contains the exact diagnostic-to-orphan-reason table. |
| FND-088 | low | The unsupported-version row overlapped the existing zero-major grammar diagnostic. | STD-001 limits `unsupported_schema_version` to nonzero majors other than 1. |
| FND-089 | low | The NFR matrix still marked all issue #9-owned determinism and fail-closed work planned. | TM-002 now distinguishes issue #9's specified TC-017 slice from the planned TC-019 remainder. |
| FND-090 | medium | The displayed envelope member order contradicted the general key-sorting rule. | FR-016 displays and requires the scalar-sorted `kind`, `profile`, `value` order. |
| FND-091 | medium | A deep trace did not identify which span belongs to a digest mismatch. | FR-017 distinguishes trace, target-reference, and digest-token spans and locates `stale_trace_digest` at the latter. |
| FND-092 | medium | Duplicate artifact occurrences left output row cardinality and identity ambiguous. | FR-017 emits exactly one orphan row per unique artifact ID, retains diagnostics for later occurrences, and forbids all duplicate occurrences from coverage. |
| FND-093 | medium | `canonicalization_resource_exhausted` had no owning requirement criterion or forced negative evidence. | FR-016-AC-3 requires a deterministic reservation-failure harness to verify the code, absence of partial public bytes, and absence of a digest. |
| FND-094 | medium | Coverage row sorting omitted its string comparison rule and requirement-reference composite key. | FR-017 now fixes Unicode-scalar ordering for namespace/IDs, ascending numeric revision, the exact composite key, and exclusion of locale/map/byte collation. |
| FND-095 | medium | FR-017 and STD-001 named the deep-digest diagnostic location differently. | Both documents now use the single exact term `digest-token span`. |
| FND-096 | medium | TASK-008 did not carry FR-016-AC-3's deterministic resource-failure harness into its implementation plan. | The Plan Delta now explicitly requires the harness, diagnostic, and absence of partial bytes or digest. |

## Dependencies

- **Upstream**: TASK-007 / issue #8 supplies validated semantic values.
- **Downstream**: issue #10 consumes these identities and coverage results in the external corpus.
