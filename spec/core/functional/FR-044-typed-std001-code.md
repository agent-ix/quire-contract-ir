---
id: FR-044
title: "Export a typed STD-001 code from the model crate"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/STD-001
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-030
    type: references
---
# FR-044: Export a typed STD-001 code from the model crate

## Contract

```yaml
name: TypedStd001Code
ownership: quire-contract-ir
crate: quire-contract-model
module: code
types: [Std001Code, Std001CodeError]
invariants:
  - a Std001Code is built only by a fallible validating constructor or from a registered code
  - an invalid code form is refused with a typed error and never panics
  - the serialized form is the bare code string
  - a registered code is spelled by one constant, never by a literal at a consumer
```

## Description

The `quire-contract-model` crate shall export `Std001Code`, one typed value for
a stable machine-readable STD-001 code, so a consumer that carries a code
carries this type and not a `String`. This requirement states the type, its
form, its constructor, its wire form and its stability rules. STD-001 lists
the registered codes. FR-030 states how `KaniOutcome` carries the type.

## Inputs

A candidate code string, a `DiagnosticCode`, or a request for a registered code.

## Outputs

A validated `Std001Code`, or a typed `Std001CodeError` and no code.

## Behavior

### Home

`Std001Code` and `Std001CodeError` live in a new `code` module of
`quire-contract-model`, exported by name at the crate root and listed in
FR-019's Public items table. They are not in the `quire-contract-ir` root crate:
the root crate re-exports no model item (FR-039), so a consumer, including
`quire-contract-codegen` and QSL, imports the type from `quire_contract_model`.
QSL already depends on the model crate, so it adds no edge. The type depends on
no QSL, codegen or runtime type.

### Form and constructor

A code is lowercase ASCII snake case: one to 64 bytes, each of `a` to `z`, `0`
to `9` and `_`, beginning with `a` to `z`, with no trailing `_` and no two
adjacent `_`. `Std001Code::new` takes a `&str`, checks its length before it
scans it, and returns `Result<Std001Code, Std001CodeError>`. No other public
constructor takes a string, the type has no `From<String>` or `From<&str>`,
and the representation is private. `as_str` returns the code,
`Display` prints it unchanged, and equality, hashing and ordering are those of
`as_str` bytes. `Std001CodeError` carries the code `invalid_code_form`, a
`Std001Code` itself, registered in STD-001, and holds no copy of the rejected
input, so a one-mebibyte input costs no allocation of that size.

### Registered codes

A code STD-001 registers outside `DiagnosticCode` is one associated constant of
`Std001Code`, named by the code in upper case (for example
`Std001Code::KANI_VACUOUS_PROOF` for `kani_vacuous_proof`). `Std001Code::REGISTERED`
is the sorted `&'static [&'static str]` of those codes. `From<DiagnosticCode>`
for `Std001Code` is infallible and yields the code `DiagnosticCode::as_str`
returns, so one type carries every STD-001 code. `Std001Code::is_registered`
is true for a code in `REGISTERED` or one that `DiagnosticCode::ALL` spells,
and false for any other well-formed code.

A well-formed code that STD-001 does not register is a valid `Std001Code`: the
type states form, not membership, because a consumer registers its own codes in
its own registry (codegen's corpus refusals are codegen's). `is_registered`
is how a consumer asks about membership.

### Serialization

`Std001Code` serializes as the bare JSON string of its code, the bytes the
`String` it replaces serialized to: a `KaniOutcome` serializes unchanged.
Deserialization runs `Std001Code::new`: a JSON value that is not a string, or a
string that is not a well-formed code, is a deserialization error and yields no
value. This is the one place the form is checked on the wire, so a consumer that
reads a stored outcome rejects a malformed `code` where a `String` accepted it.

### Stability

A registered code is never renamed, never reused for another condition and never
removed while a release names it; a changed meaning takes a new code. The form
above is part of the contract: widening it is a breaking change. The constants
and `REGISTERED` are the only spelling of a registered code a consumer may rely
on, and a registry row has one constant and the reverse.

### Break

This change is a prerelease Rust API break, with no compatibility layer. A field
or parameter that carried a code as `String` takes `Std001Code`, and a caller
that passed a literal builds the code with `Std001Code::new` or a constant. The
wire form does not change, apart from the stricter deserialization above.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-044-AC-1 | `Std001Code::new("kani_vacuous_proof")` and a 64-byte code of `a` return a code whose `as_str` equals the input; each of the empty string, a 65-byte string, `Kani_x`, `kani-x`, `_x`, `x_`, `x__y`, `1x`, `kani x`, `kanï` and a one-mebibyte string returns `Std001CodeError` whose code is `invalid_code_form`, with no value and no panic. | Test (TC-442) |
| FR-044-AC-2 | `serde_json::to_string` of `Std001Code::KANI_VACUOUS_PROOF` equals `"kani_vacuous_proof"` with its quotes; that string deserializes to an equal code; and `"Bad"`, `""`, `7` and `null` each fail to deserialize with an error and no value. | Test (TC-442) |
| FR-044-AC-3 | `Std001Code::REGISTERED` equals, in order, the fifteen codes `invalid_code_form`, `kani_backend_absent`, `kani_bound_exhausted`, `kani_bound_invalid`, `kani_capability_missing`, `kani_capability_request_invalid`, `kani_counterexample`, `kani_identity_invalid`, `kani_outcome_invalid`, `kani_population_incomplete`, `kani_population_invalid`, `kani_proved`, `kani_reference_invalid`, `kani_solver_absent`, `kani_vacuous_proof`; each constant's `as_str` equals its lower-case name and passes `Std001Code::new`; every `DiagnosticCode::ALL` entry converts to a code whose `as_str` equals `DiagnosticCode::as_str`; `is_registered` is true for each of those and false for `kani_corpus_identity_collision`. | Test (TC-442) |

## Dependencies

[STD-001](./STD-001-diagnostic-registry.md) registers the codes.
[FR-019](../../model/functional/FR-019-rust-library-interface.md) lists the
model crate's public items. [FR-030](../../kani/functional/FR-030-bounded-kani-domain-and-outcomes.md)
makes `KaniOutcome.code` this type. QSL's `DeclineCode` takes this type for its
`Ir` arm (QSL-351); that arm is QSL's and is not specified here.

## Status

Planned (Linear IR-605). No `code` module exists: `KaniOutcome.code` is a
`String` and `DiagnosticCode` has no `Deserialize`.
