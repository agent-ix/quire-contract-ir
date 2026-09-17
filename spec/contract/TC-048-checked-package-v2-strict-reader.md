---
id: TC-048
title: "CheckedPackage V2 strict reader re-derives package and nominal identities"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
  - target: ix://agent-ix/quire-specification/TC-217
    type: references
---
# TC-048: CheckedPackage V2 strict reader re-derives package and nominal identities

## Description

Verify FR-038-AC-1 through FR-038-AC-5 (QSpec FR-322-AC-4, FR-322-AC-8,
FR-322-AC-10) against the vendored V2 fixtures and node-identity vectors.

## Test Procedure

Dispatch an unknown, empty, absent and malformed `contract_version`, and
independently a malformed document, a duplicate top-level member and
noncanonical bytes, before any version is selected. Admit both vendored V2
positive fixtures. Apply every vendored adverse
structural mutation and compare the outcome. Independently inject malformed
JSON, a duplicate member, an unknown member, noncanonical bytes, an absent or
mismatched context digest, an unreported or unsupported required feature, a
dangling reference and an incomplete source map. Admit at exact byte, depth, node,
edge, occurrence, diagnostic and work limits, then lower each limit by one.
Recompute every fixture's package id, edit excluded and included preimage
members, and re-read. Re-derive each node-identity vector digest; build one
package holding every vector node; apply every vendored invalid mutation with
the retained digest, remove or swap a preimage, and change a retained
preimage's enum case, `semantic_type`, dependency and unit target while
mirroring the identity projection and package id. Re-own the nominal enum
declaration by a model owner (`identity`, `node`) over a locked `sha256-jcs`
domain package and re-read it; then name an unselected package, select none,
empty the node, restore a compiled-model owner or lock shape carrying
`authority`, `revision` or `export`, change the selection's digest domain,
version or evidence domain, and set a model node to `model_export`.

## Expected Results

An unknown, empty or absent `contract_version` refuses as
`unknown_contract_version`; a malformed `contract_version`, document, or a
duplicate or noncanonical top-level member refuses as `malformed_wire`,
`duplicate_member` or `noncanonical_wire` before any version-specific
decoding. Positive fixtures admit with their recorded package ids. Every adverse and
injected case returns its exact refusal code or incomplete accounting with no
package. Excluded edits keep the id and included edits change it. Every vector
digest matches; every nominal mutation and contradictory cross-field join
refuses as `invalid_semantic_graph`. The model-owned package validates
against the vendored schema and admits; each model join mismatch refuses as
`invalid_semantic_graph`, a compiled-model owner or lock shape carrying
`authority`, `revision` or `export` as `unknown_member`, a foreign digest
domain as `digest_domain_mismatch`, an absent or raw-only domain package
digest as `stale_dependency`, and `model_export` as `invalid_semantic_graph`.
