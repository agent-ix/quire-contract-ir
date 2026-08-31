---
id: FR-019
title: "Expose a stable Rust semantic-model interface"
type: FR
object: interface
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
---
# FR-019: Expose a stable Rust semantic-model interface

## Contract

```yaml
name: ContractIrRustApi
version: quire-contract-ir-v0.1
ownership: quire-contract-ir
inputs:
  - unvalidated package values
  - validation options
  - explicit schema and canonicalization profiles
outputs:
  - immutable validated packages
  - ordered structured diagnostics
  - canonical bytes and digests
  - dependency and coverage classifications
invariants:
  - wire values remain distinct from validated semantic values
  - untrusted input has no public panic path
  - no downstream engine type appears in the public contract
compatibility:
  msrv: Rust 1.75
  licensing: MIT OR Apache-2.0
  publication: disabled pending a later human release decision
```

## Description

The crate shall expose construction, validation, dependency derivation,
canonicalization, digest, migration, and coverage-classification operations
without exposing mutable internal caches or downstream engine types.

## Inputs

Owned or borrowed contract data, validation options, and explicit supported
schema/canonicalization profiles.

## Outputs

Immutable validated packages, ordered diagnostics, canonical bytes, digests,
dependency sets, and coverage classifications.

## Behavior

Parsing is separate from semantic validation. Fallible operations return typed
results. Public diagnostics carry code, severity, message, source span, semantic
path, and related identities. No public operation panics for untrusted input.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-019-AC-1 | The public API separates unvalidated wire values from validated semantic values and exposes no unchecked conversion for untrusted input. | API inspection (TC-018) |
| FR-019-AC-2 | Every untrusted-input failure is expressible through the stable diagnostic model without panic text parsing. | API and negative-test inspection (TC-018) |

## Dependencies

FR-011 through FR-018 define the operations and results.
