---
id: FR-020
title: "Expose versioned JSON and conformance-runner interfaces"
type: FR
object: interface
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-002
    type: traces_to
---
# FR-020: Expose versioned JSON and conformance-runner interfaces

## Contract

```yaml
name: ContractIrJsonConformanceApi
version: quire-contract-ir-v0.1
ownership: quire-contract-ir
inputs:
  - UTF-8 JSON bytes
  - optional fixture manifest path
  - explicit schema and canonicalization profile identities
outputs:
  - JSON Lines conformance results
  - stable process exit classification
invariants:
  - output vocabulary is independent of Rust type names and debug formatting
  - standard output remains machine-readable
  - operational errors are written to standard error
compatibility:
  schema: versioned and fail-closed
  licensing: MIT OR Apache-2.0
  publication: disabled pending a later human release decision
```

## Description

The repository shall expose a versioned JSON package interface and a
process-level conformance runner whose output is independent of Rust type names
and debug formatting.

## Inputs

UTF-8 JSON bytes, an optional fixture-manifest path, and explicit schema and
canonicalization profile identities.

## Outputs

JSON Lines results containing fixture ID, validity, ordered diagnostics,
canonical digest, dependency identities, tool identity, and exit classification.

## Behavior

The runner writes machine-readable results to standard output and operational
errors to standard error. Exit 0 means every expectation matched, exit 1 means a
conformance mismatch, and exit 2 means invocation or environment failure.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-020-AC-1 | An independent process can validate the canonical corpus and compare every declared result without linking this crate. | Interface test (TC-018) |
| FR-020-AC-2 | Runner output and exit classes are stable, documented, and covered by positive and negative fixtures. | Interface test (TC-018) |

## Dependencies

FR-018 defines corpus content; PGM-01 defines tool and evidence identity.
