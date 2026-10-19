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

Verify NFR-002-AC-1 independently of TC-019's cross-run and cross-platform
determinism analysis. Inspect the public schema and model-name inventory for
Rust-layout, architecture-language, solver, runtime, pointer-width and
operating-system-path vocabulary.

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
array value, and check both for forbidden terms. Derive the exemption set by
running the STD-001 `Code`-row extractor on STD-001 itself; STD-001 governs
registered diagnostic code vocabulary, while TC-456 governs model vocabulary.
Exempt a `Std001Code` public associated constant only when its name equals the
registered code string converted to uppercase ASCII with underscores unchanged
and its string value equals that exact registered code string. A registered row
without a matching constant creates no exemption. Check `Std001Code::REGISTERED`
against the same registry rows: require its array value to equal, element by element in
ASCII-sorted order with no extra or missing entries, the registered code strings
represented by those matching `Std001Code` constants. Exempt each matching
array element only through its corresponding registry row; the name
`REGISTERED` grants no exemption to an arbitrary string in the array. Do not
infer registration from a constant's presence in source or
`Std001Code::REGISTERED`, and do not use a prefix, glob, regular expression or
separately maintained list of exempt names
in the test. The exemption applies only to those exact associated-constant
names and their string values in `Std001Code`, and the corresponding checked
`REGISTERED` elements, not to another exported item or schema name with the
same spelling. Fail if any other enumerated name or string value contains a
forbidden implementation-specific term. Include a positive
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
For the removed-row mutation, use a fixture made from the STD-001 document
with only its `kani_solver_absent` `Code` row removed. Feed this fixture through
the same STD-001 `Code`-row extractor used for the unmodified document; a
hand-built code set does not count. Leave `Std001Code::KANI_SOLVER_ABSENT` and
`Std001Code::REGISTERED` unchanged in the Rust source fixture and run the same
Rust enumerator. The inventory must reject the constant's name and value and
the now-unregistered `REGISTERED` array element.

## Expected Results

The complete public schema and model-name inventory, including string `const`
values and all exported item kinds under their applicable features, has no
forbidden names outside the exact STD-001-derived `Std001Code` exemption set.
Every adverse control and both registration mutations are detected; the benign
control, the registered constants and their values, and the checked `REGISTERED`
array are admitted.

## Status

Planned until the NFR-002-AC-1 inventory test covers the amended scope. Its
existing trace tag alone does not establish that coverage. TC-019 remains
planned for NFR-001's separate determinism measurements.
