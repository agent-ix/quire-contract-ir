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
  - fixed public conformance registries
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

The stable v0.1 surface is the public API re-exported by `quire_contract_ir`.
Serde deserialization trait implementations are not part of that surface:
untrusted package JSON enters through `ContractPackage::from_json_str` or
`from_json_bytes`, while validated values remain serializable.
Unvalidated JSON enters only through wire/request decoders. Validated identity,
package, declaration, expression, canonical, migration, and coverage types keep
fields private and expose checked constructors plus immutable accessors. There
is no `From`/unchecked constructor from untrusted wire values to validated
types. `ValidationOptions::strict()` is the sole v0.1 option set and cannot
disable limits, diagnostics, version preflight, or definedness.

Package parsing accepts UTF-8 `&str` and byte slices. Invalid UTF-8, unknown
object members, malformed wire shape, and wire nesting above the fixed limit are
`invalid_wire_format`. Parsing is separate from semantic validation and version
preflight precedes semantic conversion. Expression conformance requests decode
to public wire types, then explicitly validate declarations, expression nodes,
expected type, execution point, and clause-root policy. Fallible operations
return typed results. Public diagnostics carry code, severity, message, source
span, semantic path, related identities, and obligation kind. Callers never
need to parse display/debug/panic text.

Canonical APIs require the explicit closed `CanonicalProfile`; v0.1 registers
only `quire.contract.canonical-json/v1`. Migration requires explicit source and
target versions. Coverage accepts immutable traces and returns a complete report
plus ordered diagnostics. No mutable cache, global registry, filesystem path,
process handle, host-width integer, downstream engine type, or schema-library
type appears in the semantic API.

The crate root exports `PUBLIC_CONSTRUCT_TAGS` and `CONFORMANCE_BOUNDARIES` as
sorted fixed-width `&'static [&'static str]` registries and retains
`DiagnosticCode::ALL` as its sorted fixed-width enum registry. Their ordering,
contents, names, and types are stable v0.1 API and are inspected alongside the
other public signatures.

All recursive or collection-bearing untrusted inputs undergo fixed-limit
preflight before recursive conversion. The crate root exports the
wire-independent `MAX_SEMANTIC_NODES: u32 = 25000`,
`MAX_SEMANTIC_DEPTH: u32 = 256`, and
`MAX_SEMANTIC_COLLECTION_ITEMS: u32 = 10000`, plus the parser guard
`MAX_WIRE_JSON_DEPTH: u32 = 576`. A complete operation input may
contain at most that many decoded semantic nodes; nested value-type or other
recursive structure may be at most that deep; and every declaration,
requirement, clause, field, variant, parameter, trace, item, or other semantic
collection may contain at most that many entries. Preflight is iterative and
occurs before recursive validation, canonicalization, migration, or coverage.
The first node, depth, or collection path crossing a limit returns
`semantic_input_too_large` and no partial semantic result. Public decode,
validate, canonicalize, migrate, and classify calls return without panic for
the complete negative corpus. Rust 1.75 builds the library, runner, and tests
with default features; the crate remains `publish = false`.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-019-AC-1 | Compile-time/API fixtures plus public-source signature inspection show wire/request values are distinct from private-field validated values, unknown members are rejected consistently with the published schema, every conversion is fallible, canonical/migration profiles are explicit, the fixed conformance registries, three semantic-limit constants, and wire-depth constant are stable public exports, and forbidden host/downstream/schema-library vocabulary is absent without requiring nightly rustdoc JSON. | Inspection (TC-018) |
| FR-019-AC-2 | The complete negative corpus executes package/expression decode, validation, canonicalization, migration, and coverage through `catch_unwind`; exact-at-limit and one-past-limit type depth, semantic node, and semantic collection cases return the specified result with no public panic, partial result, or message parsing. | Test (TC-018) |

## Dependencies

FR-011 through FR-018 define the operations and results.
