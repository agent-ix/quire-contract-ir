---
id: TC-230
title: "CheckedPackage V2 reads quantity literals and unit type references"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
---
# TC-230: CheckedPackage V2 reads quantity literals and unit type references

## Description

Verify FR-038-AC-299 through FR-038-AC-302 against QSpec
FR-322-AC-79 through FR-322-AC-81 at the public V2 reader boundary.
Consume the selected QSpec checkout without copying its fixtures or schemas.

## Test Procedure

1. Construct QSpec's QL-01..04 quantity literal cases with a declared
   `scalar_type`/`unit` node. Admit the canonical integer and reduced-fraction
   magnitudes. Change the magnitude to every recorded noncanonical spelling,
   the literal type to another family, and an integer literal's type to the
   unit. Compare the reader's code, cause and literal-member locus.
2. Construct QSpec's MU-01..06 model-member cases from a selected document
   with `typeRef` `ix://quire/unit/ms`, and checked units `ms` and `s`. Read
   scalar and set fields plus an operation parameter and result through the
   production member reader. Change the name to `ghost`, and supply another
   unit and an integer argument; compare each named outcome.
3. Construct QSpec's NS-01..03 declaration-name cases. Admit one unit,
   dimension and record of the same qualified name; repeat the unit or type
   name in its own namespace and compare the later-node refusal.
4. Run the selected-checkout QSpec conformance target and compare strict
   coverage with the main-branch baseline. Verify that the four new IR
   criteria are traced and no new unbacked or contradicted row appears.

## Expected Results

Canonical quantity literals and unit type references admit without a new
model-owned unit type node. Noncanonical magnitudes refuse `invalid_package`/
`invalid-value`, wrong literal and argument families refuse `ill_typed`/
`operator-ineligible`, missing units refuse `missing_declaration`/
`missing-name`, and duplicate names refuse `ambiguous_declaration`/
`ambiguous-name` in their own namespace. Every test binds its criterion with
`#[trace]`.

## Status

Planned for IR-938. Tests and implementation follow this spec change.
