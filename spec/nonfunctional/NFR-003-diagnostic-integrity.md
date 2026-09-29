---
id: NFR-003
title: "Fail closed with reviewable diagnostics"
type: NFR
quality_attribute: safety
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-003
    type: traces_to
---
# NFR-003: Fail closed with reviewable diagnostics

## Statement

Untrusted input shall never trigger permissive schema interpretation, silent
repair, false covered status, or a public panic. Every rejected semantic
condition shall retain a stable code and the narrowest available source span.

## Scope

Parsing, validation, migration, dependency derivation, canonicalization,
coverage classification, and the conformance runner.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Public panic paths for untrusted input | 0 | Any path fails | Negative corpus plus panic-free API review |
| Failure classes with stable code | 100% | Any unnamed class fails | Diagnostic registry/corpus reconciliation |
| Orphans counted as covered | 0 | Any false coverage fails | Orphan fixture assertions |

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-003-AC-1 | Every declared invalid-input class returns a stable diagnostic code and no public panic. | Test (TC-019) |
| NFR-003-AC-2 | No malformed, unsupported, stale, undefined, or orphaned input contributes successful canonical identity or covered status. | Test (TC-019) |

## Verification

Negative fixtures, mutation tests, diagnostic-order tests, and orphan coverage
tests (TC-017 through TC-019).
