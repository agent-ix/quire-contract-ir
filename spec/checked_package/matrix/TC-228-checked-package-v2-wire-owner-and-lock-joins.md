---
id: TC-228
title: "CheckedPackage V2 validates wire owners and lock joins"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
---
# TC-228: CheckedPackage V2 validates wire owners and lock joins

## Description

Verify FR-038-AC-153 through FR-038-AC-155 through the production V2 reader.
Construct packages with selected source and model evidence and owner values
matching QSpec FR-322 and QSL FR-092/FR-094. Read QSpec's two-owner positive
fixtures from its selected checkout; copy no fixture into this repository.

## Test Procedure

1. Admit a declared record, tuple and function with each node's
   `SourceOwner` in the node. Admit a model declaration
   and undeclared clause function with `ModelOwner`. Admit an anonymous type,
   `correspondence`/`source_locus`, a nominal node and an application-keyed
   expression without a node owner. Admit QSpec's two-owner `Point` and `List`
   packages and compare their published structural `Point`, `List`, `Integer` and `three` ids and complete bodies.
2. For each required-owner class, omit `owner` from the node, and separately add a forbidden top-level identity projection. Separately insert `null`, the wrong owner variant, a model
   `version` member, and an owner on an anonymous, nominal or application
   node. Repeat an omitted-owner mutation while an occurrence and the source
   map still name the source unit. Apply the current QSpec adverse owner mutations to fresh positive packages. Keep each authored structural id unchanged and compare owner joins directly. Check exact code,
   cause and RFC 6901 pointer before the owner join or a key check can run.
3. Set a source owner pair and a model owner identity to values absent from
   their respective lock selections. Put two unmatched owners in inverse wire and structural node-id order, and assert the reported node. Repeat with both source
   pairs selected but with a declared node's source-map region changed to the
   other pair while its owner remains fixed. In the selected model document,
   give an otherwise valid model declaration node an absent `owner.node`,
   then one naming a relationship while the node claims an object type. Test
   both object-type subkind mismatches: a `model`/`object_type` node owned by
   an object type with `interfaceFeatures`, and a
   `model`/`systems_interface` node owned by an object type without
   `interfaceFeatures`. Retain well-formed structural ids so a malformed graph id cannot mask the owner join.
   Give an unreachable clause function an absent operation member, then a
   field as its owner node; as positive controls, use an operation member's
   clause and an object type's invariant. Place a duplicate structural id on a lower-id node beside an invalid owner, and measure the exact work limit around one
   selected-document lookup.

## Expected Results

All positive packages admit. `Point` and every `List` group member differ
between owners, while `Integer` and `three` have equal keys. Missing or
forbidden owners refuse `malformed_wire` at the object lacking the member or
at the present `owner`; no occurrence repairs the omission. An added top-level identity projection refuses `unknown_member` at that member; owner mutations without it reach their exact source or model join.
Unmatched source owners and unresolved or wrong-kind model owners, including
both `interfaceFeatures` cross-kind mutations, refuse
`missing_declaration`/`missing-selection` at the lowest-key offending node,
even when it is unreachable and another lower-id node has a duplicate id. A
declaration occurrence whose source differs from its owner refuses
`invalid_package`/`invalid-value` at its source-map entry. The operation
clause and invariant owners pass. One document lookup consumes one validation
visit; a limit one below it returns `incomplete`.

## Status

The in-repo generated owner-schema and join cases run in IR-646. IR-658
implements the projection-owner mismatch refusal. FR-038-AC-176 external
conformance runs through `tests/conformance_qspec/main.rs` for QSpec's
`positive-two-owners-a.json` and `positive-two-owners-b.json` and through the
private `model_members` test for every published model declaration key. Both
read the checkout named by `QUIRE_SPECIFICATION_DIR` through the explicit
`make conformance-qspec` target. No fixture is copied into this repository.
IR-1141 replaces structural-key re-derivation with structural-id shape, uniqueness, owner and graph checks. Any interim recursion-group skip must be retired by the code leg before these cases can back the revised contract.
