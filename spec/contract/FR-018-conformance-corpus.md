---
id: FR-018
title: "Publish the v0.1 schema and conformance corpus"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-003
    type: traces_to
---
# FR-018: Publish the v0.1 schema and conformance corpus

## Description

The repository shall publish a Draft 7 JSON schema, representative valid
packages, targeted invalid packages, expected diagnostics, canonical encodings,
digests, dependency sets, and a reusable conformance runner.

## Inputs

Versioned schema files, fixture manifest, fixture bytes, expected outcomes, and
runner configuration.

## Outputs

Machine-readable per-fixture results with stable diagnostics and a nonzero exit
status for any mismatch.

## Behavior

The fixture manifest lists every fixture exactly once and pins its expected
validity, diagnostic sequence, canonical digest, and dependency identities.
Downstream tools can run the corpus without linking the Rust library.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-018-AC-1 | Every public construct and failure class has at least one positive or negative fixture. | Test (TC-018) |
| FR-018-AC-2 | The runner detects schema, diagnostic, canonical-byte, digest, or dependency expectation drift. | Test (TC-018) |

## Dependencies

FR-011 through FR-017 define the normative corpus behavior.
