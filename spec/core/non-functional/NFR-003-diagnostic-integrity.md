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

Parsing, validation, dependency derivation, canonicalization,
coverage classification, and the conformance runner.

The declared invalid-input classes of NFR-003-AC-1 are the semantic
diagnostic codes STD-001 registers for FR-011 through FR-019, which are the
`DiagnosticCode` values: the four code tables of STD-001 from "Identity and
Reference Codes" through "Semantic Limit Codes". STD-001 states that this
registry owns those codes and that a new failure class requires a registry row.
Three groups of STD-001 and FR-020 codes are outside the declared classes:

- the bounded-Kani cause and outcome error codes (`kani_*`, including
  `kani_outcome_invalid`): they describe a `KaniOutcome` that FR-030 builds,
  not a rejected package, expression or coverage input;
- `invalid_code_form` (FR-044): it reports a candidate code string that is not
  a well-formed code and carries no span; TC-442 asserts it;
- the six runner operational codes of FR-020 (`invalid_invocation`,
  `invalid_corpus`, `unsupported_profile`, `unsafe_path`, `fixture_io`,
  `resource_exhausted`): STD-001 states they are "neither `DiagnosticCode`
  values nor semantic diagnostic shapes", and NFR-003's statement concerns
  rejected semantic conditions; TC-018 asserts them.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Public panic paths for untrusted input | 0 | Any path fails | Negative corpus plus panic-free API review |
| Failure classes with stable code | 100% | Any unnamed class fails | Diagnostic registry/corpus reconciliation |
| Coverage diagnostic locus accuracy | 100% | Any coverage reference error without its exact target-reference span, or any later duplicate-ID error without its exact occurrence source span, fails | unit-testing |
| Orphans counted as covered | 0 | Any false coverage fails | Orphan fixture assertions |

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-003-AC-1 | Every declared invalid-input class returns a stable diagnostic code and no public panic. | Test (TC-454) |
| NFR-003-AC-2 | No malformed, unsupported, stale, undefined, or orphaned input contributes successful canonical identity or covered status. | Test (TC-454) |
| NFR-003-AC-3 | Given artifact traces whose source and target-reference spans differ, each cross-package, missing-requirement, and stale-revision diagnostic retains the exact target-reference span of its trace; each `duplicate_artifact_trace` diagnostic retains the exact source span of the later duplicate occurrence. | Test (TC-017) |

## Verification

Negative fixtures, mutation tests, diagnostic-order tests, and orphan coverage
tests (TC-017, TC-018, TC-454).
