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
macros: [std001_code]
invariants:
  - a Std001Code guarantees the STD-001 code form and nothing else
  - a Std001Code does not name the registry that issued it
  - a Std001Code is built only by a validating constructor, a registered constant, std001_code! or the infallible conversion from a DiagnosticCode
  - an invalid code form from a runtime string is refused with a typed error and never panics
  - an invalid literal given to std001_code! is a compile error and has no runtime path
  - the serialized form is the bare code string
  - a code STD-001 registers outside DiagnosticCode is spelled by one Std001Code constant, and a DiagnosticCode code by one DiagnosticCode variant
```

## Description

The `quire-contract-model` crate shall export `Std001Code`, one typed value for
a machine-readable code in the STD-001 form, so a consumer that carries a code
carries this type and not a `String`. This requirement states the type, its
form, its constructors, its wire form and its stability rules. STD-001 lists
the registered codes. FR-030 states how `KaniOutcome` carries the type.

## Inputs

A candidate code string, a string literal, a `DiagnosticCode`, or a request for
a registered code.

## Outputs

A validated `Std001Code`, or a typed `Std001CodeError` and no code.

## Behavior

### What the type guarantees

`Std001Code` guarantees form only: the string is a well-formed code (below).
It does not guarantee that STD-001 registers the code, and it does not name the
registry that issued it. The name refers to the form STD-001 fixes, not to
membership. Membership is `is_registered()`, which is true only for a code
STD-001 registers. A code that is not registered is a valid `Std001Code`,
because a consumer that mints codes of its own (codegen's corpus refusals) keeps
them in its own registry. A consumer that carries a `Std001Code` therefore
carries an IR-family code, from IR's registry or from the registry of a producer
that builds `KaniOutcome`s, and `is_registered()` is the only way to tell the
two apart. A consumer that needs the issuing registry, as QSL's `Declined`
`Ir` arm may (QSL's ruling is that a code names the registry that issued it),
must carry that attribution itself, for example as a variant or a field beside
the code, or must refuse a code for which `is_registered()` is false. That
choice is QSL's, and this requirement does not make the `Ir` arm an IR-registry
claim.

### Home

`Std001Code`, `Std001CodeError` and the `std001_code!` macro live in a new `code`
module of `quire-contract-model`, exported by name at the crate root and listed
in FR-019's Public items table. They are not in the `quire-contract-ir` root
crate: the root crate re-exports no model item (FR-039), so a consumer,
including `quire-contract-codegen` and QSL, imports them from
`quire_contract_model`. QSL already depends on the model crate, so it adds no
edge. The type depends on no QSL, codegen or runtime type.

### Form and constructors

A code is lowercase ASCII snake case: one to 64 bytes, each of `a` to `z`, `0`
to `9` and `_`, beginning with `a` to `z`, with no trailing `_` and no two
adjacent `_`. `Std001Code::new` takes a `&str`, checks its length before it
scans it, and returns `Result<Std001Code, Std001CodeError>`; it is the one
constructor for a string known only at run time. `Std001Code::from_static`
takes a `&'static str` and applies the same check as a `const fn`, so it
evaluates in a `const` item. The `std001_code!` macro takes one string literal
and expands to a `const` block over `from_static` that fails const evaluation
on an invalid form, so a literal that is not a well-formed code is a compile
error, and the macro leaves no error path and no panic at run time. A consumer
declares each code it mints from a literal with `std001_code!` and needs no
`.expect()` or unreachable error arm. No other public constructor takes a
string, the only other ways to obtain a code are a registered constant and
`From<DiagnosticCode>` (below), the type has no `From<String>` or `From<&str>`, and the representation
is private. `as_str` returns the code, `Display` prints it unchanged, and
equality, hashing and ordering are those of `as_str` bytes.
`Std001CodeError` carries the code `invalid_code_form`, a `Std001Code` itself,
registered in STD-001, and holds no copy of the rejected input, so a
one-mebibyte input costs no allocation of that size.

### Registered codes

A code STD-001 registers outside `DiagnosticCode` is one associated constant of
`Std001Code`, named by the code in upper case (for example
`Std001Code::KANI_VACUOUS_PROOF` for `kani_vacuous_proof`), usable in a `const`
context. `Std001Code::REGISTERED` is the sorted `&'static [&'static str]` of
those codes. A code STD-001 registers as a `DiagnosticCode` has no constant:
`From<DiagnosticCode>` for `Std001Code` is infallible and yields the code
`DiagnosticCode::as_str` returns, so one type carries every STD-001 code.
`Std001Code::is_registered` is true for a code in `REGISTERED` or one that
`DiagnosticCode::ALL` spells, and false for any other well-formed code.

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
above is part of the contract: widening it is a breaking change. Every STD-001
row has exactly one spelling: a row outside `DiagnosticCode` has one `Std001Code`
constant and `REGISTERED` lists it, and a row in `DiagnosticCode` has one
variant, and no constant or variant exists without a row. The constants,
`REGISTERED` and `DiagnosticCode::ALL` are the only spelling of a registered code
a consumer may rely on.

### Break and order

This change is a prerelease Rust API break, with no compatibility layer. A field
or parameter that carried a code as `String` takes `Std001Code`, and a caller
that passed a literal builds the code with `std001_code!` or a constant. The
wire form does not change, apart from the stricter deserialization above.

The family lowerings still in this repository (`src/kani/arithmetic.rs`,
`collections.rs`, `objects.rs`) emit their codes through the non-success
constructor, so until IR-347 removes them they build those codes with
`std001_code!`, unregistered, and IR-347 deletes them with the lowerings; the
IR-605 code change does not wait for IR-347. `KaniProviderRecord`, whose `cause`
is a `String` today, is QSL's item and is removed from this repository
(FR-039-AC-3); until it is removed its `cause` is a `Std001Code` and no `String`
copy of an outcome's code remains.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-044-AC-1 | `Std001Code::new("kani_vacuous_proof")` and a 64-byte string of `a` return a code whose `as_str` equals the input; each of the empty string, a 65-byte string of `a`, `Kani_x`, `kani-x`, `_x`, `x_`, `x__y`, `1x`, `kani x`, `kanï` and a one-mebibyte string of `a` returns `Std001CodeError` whose code is `invalid_code_form`, with no value and no panic. | Test (TC-442) |
| FR-044-AC-2 | `serde_json::to_string` of `Std001Code::KANI_VACUOUS_PROOF` equals `"kani_vacuous_proof"` with its quotes; that string deserializes to an equal code; and `"Bad"`, `""`, `7` and `null` each fail to deserialize with an error and no value. | Test (TC-442) |
| FR-044-AC-3 | `Std001Code::REGISTERED` equals, in order, the fifteen codes `invalid_code_form`, `kani_backend_absent`, `kani_bound_exhausted`, `kani_bound_invalid`, `kani_capability_missing`, `kani_capability_request_invalid`, `kani_counterexample`, `kani_identity_invalid`, `kani_outcome_invalid`, `kani_population_incomplete`, `kani_population_invalid`, `kani_proved`, `kani_reference_invalid`, `kani_solver_absent`, `kani_vacuous_proof`; each constant's `as_str` equals its lower-case name and passes `Std001Code::new`; every `DiagnosticCode::ALL` entry converts to a code whose `as_str` equals `DiagnosticCode::as_str`; `is_registered` is true for each of those and false for the well-formed `kani_corpus_identity_collision`, which `Std001Code::new` accepts. | Test (TC-442) |
| FR-044-AC-4 | `std001_code!("kani_corpus_identity_collision")` evaluates in a `const` item to a code whose `as_str` equals that literal and whose `is_registered` is false; a probe that passes `std001_code!` the literal `"Bad-Code"` fails to compile, and so does one that passes it a non-literal `&str` variable. | Test (TC-442) |
| FR-044-AC-5 | `Std001Code::from_static`, evaluated in a `const` item, returns `Ok` with a code equal to `Std001Code::new`'s for `kani_vacuous_proof` and returns `Err` with code `invalid_code_form` for the empty string, `Kani_x` and a 65-byte string of `a`, and no input panics. | Test (TC-442) |

## Dependencies

[STD-001](./STD-001-diagnostic-registry.md) registers the codes.
[FR-019](../../model/functional/FR-019-rust-library-interface.md) lists the
model crate's public items. [FR-030](../../kani/functional/FR-030-bounded-kani-domain-and-outcomes.md)
makes `KaniOutcome.code` this type. QSL's `DeclineCode` takes this type for its
`Ir` arm (QSL-351); that arm, and whether it records the issuing registry, is
QSL's and is not specified here beyond the limit stated under "What the type
guarantees".

## Status

Planned (Linear IR-605). No `code` module exists: `KaniOutcome.code` is a
`String` and `DiagnosticCode` has no `Deserialize`.
