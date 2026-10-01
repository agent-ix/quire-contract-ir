---
id: NFR-001
title: "Produce deterministic semantic results"
type: NFR
quality_attribute: reliability
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-002
    type: traces_to
---
# NFR-001: Produce deterministic semantic results

## Statement

The quire-contract-ir library shall reproduce validation results, diagnostics,
canonical bytes, digests, dependency sets, and coverage classes across supported
operating systems and process runs when input bytes, supported profiles, and
dependency versions are fixed.

## Scope

The public Rust library, JSON interface, corpus runner, and golden fixtures.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Cross-run byte equality | 100% | Any mismatch fails: repeat the complete corpus twice and compare the two outputs byte for byte | metamorphic-testing |
| Cross-platform golden digest equality | 100% | Any mismatch fails: compare Linux, macOS, and Windows corpus outputs against the checked-in golden digests when CI is enabled | golden-approval-testing |
| Diagnostic order equality | 100% | Any reorder fails: compare the ordered diagnostic code/path tuples against the checked-in golden outputs | golden-approval-testing |

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-001-AC-1 | Two complete corpus runs over fixed inputs and profiles produce byte-identical results. | Test (TC-019) |
| NFR-001-AC-2 | Linux, macOS, and Windows produce identical declared golden digests when cross-platform CI is later enabled. | Test (TC-019) |

## Verification

Golden fixtures, seeded repetition, and canonicalization property tests (TC-017,
TC-019).
