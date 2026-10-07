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
   `SourceOwner` in the node and identity projection. Admit a model declaration
   and undeclared clause function with `ModelOwner`. Admit an anonymous type,
   `correspondence`/`source_locus`, a nominal node and an application-keyed
   expression without a node owner. Admit QSpec's two-owner `Point` and `List`
   packages and compare their independently computed `Point`, `List`,
   `Integer` and application-keyed `three` identities.
2. For each required-owner class, omit `owner` from the node, then from its
   projection. Separately insert `null`, the wrong owner variant, a model
   `version` member, and an owner on an anonymous, nominal or application
   node. Repeat an omitted-owner mutation while an occurrence and the source
   map still name the source unit. Apply QSpec `adverse.json`'s
   `projection_owner_mutations` entries
   `projection-source-owner-differs-from-node` and
   `projection-model-owner-differs-from-node` to their selected nodes in fresh
   `positive-all-families` packages. Keep each node's owner unchanged and
   recompute the package id over the changed projection. Check exact code,
   cause and RFC 6901 pointer before the owner join or a key check can run.
3. Set a source owner pair and a model owner identity to values absent from
   their respective lock selections. Put two unmatched owners in inverse wire
   and digest order, and assert the reported node. Repeat with both source
   pairs selected but with a declared node's source-map region changed to the
   other pair while its owner remains fixed. In the selected model document,
   give an otherwise valid model declaration node an absent `owner.node`,
   then one naming a relationship while the node claims an object type. Test
   both object-type subkind mismatches: a `model`/`object_type` node owned by
   an object type with `interfaceFeatures`, and a
   `model`/`systems_interface` node owned by an object type without
   `interfaceFeatures`. Key each mutated node under its own preimage and
   refresh its identity projection and package id so a stale-key refusal
   cannot mask the owner join.
   Give an unreachable clause function an absent operation member, then a
   field as its owner node; as positive controls, use an operation member's
   clause and an object type's invariant. Place a stale key on a lower-key
   node beside an invalid owner, and measure the exact work limit around one
   selected-document lookup.

## Expected Results

All positive packages admit. `Point` and every `List` group member differ
between owners, while `Integer` and `three` have equal keys. Missing or
forbidden owners refuse `malformed_wire` at the object lacking the member or
at the present `owner`; no occurrence repairs the omission. A projection
owner differing from its node refuses `invalid_package`/`invalid-value` at
`/identity_preimage/identity_projection/{i}/owner` for both the source-owner
and model-owner mutations, even after recomputing the package id.
Unmatched source owners and unresolved or wrong-kind model owners, including
both `interfaceFeatures` cross-kind mutations, refuse
`missing_declaration`/`missing-selection` at the lowest-key offending node,
even when it is unreachable and another lower-key node has a stale key. A
declaration occurrence whose source differs from its owner refuses
`invalid_package`/`invalid-value` at its source-map entry. The operation
clause and invariant owners pass. One document lookup consumes one validation
visit; a limit one below it returns `incomplete`.

## Status

The in-repo generated owner-schema and join cases run in IR-646. IR-658
implements the projection-owner mismatch refusal. The external conformance
row remains **PLANNED / UNRUN** under FR-038-AC-176: read QSpec's
`proposals/checked-package-v2/fixtures/positive-two-owners-a.json` and
`positive-two-owners-b.json`, and the model declaration keys in
`proposals/checked-package-v2/model-member-type-vectors.json`, directly from
the checkout named by `QUIRE_SPECIFICATION_DIR`. The row is absent from the
executable conformance target until IR-654 code lands; its absence is pending
coverage, not a passing skip. No fixture is copied into this repository.
Complete structural-key re-derivation and retirement of the interim
recursion-group skip belong to IR-630.
