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
| Cross-run byte equality | 100% | Any mismatch fails | Repeat the complete corpus twice and compare bytes |
| Cross-platform golden digest equality | 100% | Any mismatch fails | Compare Linux, macOS, and Windows corpus outputs when CI is enabled |
| Diagnostic order equality | 100% | Any reorder fails | Compare ordered diagnostic code/path tuples |

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-001-AC-1 | Two complete corpus runs over fixed inputs and profiles produce byte-identical results. | Test (TC-019) |
| NFR-001-AC-2 | Linux, macOS, and Windows produce identical declared golden digests when cross-platform CI is later enabled. | Test (TC-019) |

## Verification

Golden fixtures, seeded repetition, and canonicalization property tests (TC-017,
TC-019).

Issue #9 pins the canonical profile, byte grammar, domain-separated SHA-256
inputs, structural row ordering, and same-process repetition in TC-017. The
complete twice-run corpus and Linux/macOS/Windows comparison remain TC-019 work
after the issue #10 corpus exists; no CI run is required or claimed by issue #9.
