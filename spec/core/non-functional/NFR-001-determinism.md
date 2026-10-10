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

The public Rust library, JSON interface, and conformance corpus.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Cross-run byte equality | 100% | Any mismatch fails: repeat the complete corpus twice and compare the two outputs byte for byte | metamorphic-testing |
| Cross-platform canonical byte equality | 100% | Any mismatch fails: compare canonical bytes produced from the same fixed corpus inputs and profiles on Linux, macOS, and Windows when cross-platform CI is enabled | metamorphic-testing |
| Ordered diagnostic code/path tuple equality | 100% | Any difference fails: compare the complete ordered sequence of diagnostic `(code, path)` tuples from the same fixed inputs and profiles across repeated runs and supported operating systems | metamorphic-testing |

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-001-AC-1 | Two complete corpus runs over fixed inputs and profiles produce byte-identical results. | Test (TC-019) |
| NFR-001-AC-2 | For each fixed corpus input and profile, Linux, macOS, and Windows produce byte-identical canonical output when cross-platform CI is enabled; compare the canonical bytes directly. | Test (TC-019) |
| NFR-001-AC-3 | For each fixed invalid corpus input and profile, repeated runs and supported operating systems produce exactly the same complete ordered sequence of diagnostic `(code, path)` tuples. | Test (TC-019) |

## Verification

Direct output comparison, seeded repetition, and canonicalization property tests
(TC-017, TC-019).
