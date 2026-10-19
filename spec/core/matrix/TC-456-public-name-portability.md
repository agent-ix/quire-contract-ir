---
id: TC-456
title: "Public schema and model names exclude implementation-specific vocabulary"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/NFR-002
    type: verifies
---
# TC-456: Public schema and model names exclude implementation-specific vocabulary

## Description

Inspect the public schema and model-name inventory, independently of TC-019's
cross-run and cross-platform determinism analysis, for
Rust-layout, architecture-language, solver, runtime, pointer-width and
operating-system-path vocabulary. NFR-002-AC-1 and NFR-002-AC-3 verification
credit is withheld, blocked by IR-881.

## Test Procedure

Enumerate every repository-owned public JSON schema's name-bearing property,
definition, title, enum entry and string `const` value. A forbidden name used
as a schema `const` must fail the check. Enumerate every exported item of the
model crate's macro-expanded public API, including feature-gated exports,
re-exports and public items reachable through public modules. Items declared
inside `macro_rules!` bodies, such as `DiagnosticCode` and its variants and
`Std001Code` constants, are in scope. For each exported type, also enumerate
its public fields, enum variants, public associated functions and public
associated constants (including `Std001Code` constants and `REGISTERED`,
`DiagnosticCode::ALL` and `SchemaVersion::V1_1`). Enumerate exported traits,
including `CanonicalBody`, and their public associated items. The current
public modules are `selected_population` and `source_obligation`; a newly
declared public module, feature-gated export or unrecognized public declaration
must fail the inventory until its names are checked. Read each public associated
constant's value as well as its name, including every string element of an
array value, and check both for forbidden terms. Add a `Code`-row extractor over
all seven tables in STD-001: Identity and Reference Codes, Type and Expression
Codes, Version, Canonicalization and Coverage Codes, Semantic Limit Codes,
Typed Code Form, Bounded Kani Cause Codes, and Bounded Kani Outcome Error Code.
The existing `std001_declared_codes` in `tests/it/nfr003_invalid_input.rs`
extracts only the first four tables, so it cannot serve this check unchanged.
Subtract the exact strings spelled by `DiagnosticCode::ALL` from the extracted
STD-001 code strings. Require `Std001Code::REGISTERED` to equal that difference
element by element in ASCII-sorted order, with no extra, missing or duplicate
entries. Require each code string in that difference to have a `Std001Code`
associated constant whose name is the code string converted to uppercase ASCII
with underscores unchanged and whose string value equals that code string.
`REGISTERED` itself and every array element remain subject to the forbidden-term
check except the exact element named below. The only temporary allowlist entry is:

| Associated constant name | String value | Ticket and expiry |
| --- | --- | --- |
| `Std001Code::KANI_SOLVER_ABSENT` | `kani_solver_absent` | IR-881 (rename of the registered code in the Kani lane, with no compatibility alias); removed when IR-881 merges |

Apply this entry only when the exact name and exact string value match each
other and a `kani_solver_absent` row is extracted from STD-001's Bounded Kani
Cause Codes table and survives the `DiagnosticCode::ALL` subtraction. It permits that
constant's name and value and the matching `Std001Code::REGISTERED` element;
it permits no other item, schema name or value. Do not infer an exception from
registration, a constant's presence in source, or `Std001Code::REGISTERED`.
Do not use a glob, prefix or regular expression. Fail if any other enumerated
name or string value contains a forbidden implementation-specific term,
including an unregistered `solver` constant next to `Std001Code`. Include a positive
control showing that a benign `jsonpointer` name is admitted and adverse
controls for a runtime title, pointer-width property, architecture-language
enum name, forbidden schema `const`, feature-gated export, public field, enum
variant, public associated function, public associated constant, exported
trait item and macro-generated public item. Inject each adverse control as a
schema or Rust source fixture and detect it through the inventory's own schema
walker or Rust source enumerator, respectively. Passing a control directly to
`portable_name_violations` without extraction does not count. Also mutate the
Rust source fixture to add an unregistered public `Std001Code` associated
constant with `solver` in its name and value; the inventory must reject it.
Remove the sole allowlist entry in a mutation while leaving STD-001 and the Rust
source fixture unchanged; the inventory must then reject the
`Std001Code::KANI_SOLVER_ABSENT` name, its `kani_solver_absent` value and the
matching `Std001Code::REGISTERED` element. Any STD-001 registry fixture used by
a mutation must pass through the same seven-table `Code`-row extractor as the
unmodified STD-001 document; a hand-built code set does not count. The new
unregistered `solver` constant must fail with the allowlist entry present.
The eventual TC-456 test must run without `NFR-002-AC-1` or `NFR-002-AC-3`
trace tags until IR-881 merges. The TC-456 test PR removes or withholds the
existing `NFR-002-AC-1` tag in `tests/it/public_interface.rs` and the existing
`NFR-002-AC-3` tag in `tests/it/identity.rs`. IR-881 acceptance restores both
tags when the registered-code rename lands and the allowlist entry is removed.

## Expected Results

The complete public schema and model-name inventory, including string `const`
values and all exported item kinds under their applicable features, has no
forbidden names or values outside the single temporary allowlist entry. Every
adverse control and both mutations are detected; the benign control, the exact
allowlisted constant and value, and the checked `REGISTERED` array are admitted.

## Status

Implemented for the schema and model-name inventory, registered-code reconciliation,
and adverse and mutation controls. The temporary entry remains, so NFR-002-AC-1
and NFR-002-AC-3 credit is withheld, blocked by IR-881; their trace tags are
withheld until the rename and allowlist removal. TC-019 remains planned for
NFR-001's separate determinism measurements.
