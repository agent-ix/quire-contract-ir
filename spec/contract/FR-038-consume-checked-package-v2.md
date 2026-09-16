---
id: FR-038
title: "Consume CheckedPackage V2 without widening the frozen V1 reader"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/AD-003
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/106
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
  - target: ix://agent-ix/quire-specification/FR-201
    type: references
  - target: ix://agent-ix/quire-specification/FR-195
    type: references
---
# FR-038: Consume CheckedPackage V2 without widening the frozen V1 reader

## Description

When Contract IR receives checked-package bytes, the consumer shall select the
exact I04 contract version before decoding, admit a `quire.checked-package/v2`
package only after re-deriving every package and nominal node identity, and
lower admitted V2 items independently, while the frozen
`quire.checked-package/v1` reader, public types and lowering stay unchanged.

## Inputs

Untrusted wire bytes; caller-selected read limits; a read context holding the
authoritative raw-artifact digests (from supplied bytes or a verified digest
store) and the reader-supported required features; for migration, an admitted
V1 source package, an admitted V2 target package, the claimed target package
key and optional reconstruction inputs; for lowering, requested node keys and a
lowering profile (supported node tags, bounded-domain requirement, work limit).

The normative producer contract is QSpec
`proposals/checked-package-v2/` at
`agent-ix/quire-specification@5aa00f35056c65948de93ad339540974d35c368a`,
vendored byte-exact under `tests/fixtures/checked-package/` with a
`PROVENANCE` file naming each source path, blob and SHA-256.

## Outputs

- A closed dispatch result: admitted V1 package, admitted V2 package, refused
  (typed code and structural path) or incomplete (limit kind, limit, consumed).
- A closed V2 read result: admitted, refused or incomplete; no partial package.
- A closed `MigrationOutcome`: `relinked { correspondence }` or
  `refused { code, subject }`.
- One V2 lowering record per requested item: `lowered`, `unsupported`,
  `requires_bound`, `invalid_input` or `failed`.

## Behavior

The dispatcher shall measure raw bytes against the byte limit, parse strict JSON
once (duplicate members refuse, nesting charged against the depth limit),
require canonical bytes, read `contract_version`, and route to exactly one
version-specific strict decoder. An unknown version refuses as
`unknown_contract_version`; neither decoder relabels or upgrades the other
version.

The V2 reader shall recompute `package_id` as the SHA-256 of the RFC 8785
canonical bytes of `identity_preimage` under `quire.package.semantic/v2`,
require the preimage's lock members to equal the lock, and require its
`identity_projection` to equal the occurrence-free projection of the graph in
graph order. It shall check each digest's declared domain before its bytes,
each locked source/definition/model digest against the read context, every
required feature against the reported `available` capability and the
reader-supported feature set, every node tag, family form and semantic term,
reference and dependency resolution, cycles outside one explicit
`recursion_group`, and the total source map.

When a node is an enum declaration, enum member, dimension or declared unit, the
V2 reader shall reconstruct the closed nominal preimage, require
`node_id.digest` to equal the SHA-256 of its canonical bytes, require the owner
to join an exact lock selection, and enforce identifier, canonical-integer,
reduced-rational, member/term order, duplicate/zero-exponent, root/non-root
unit, base-dimension and cross-field (`semantic_type`, dependencies, enum
literal body) rules. Any violation refuses as `invalid_semantic_graph`; a
nominal form without its preimage, or a non-nominal form carrying one, refuses
the same way.

When a caller migrates a V1 package, the migrator shall consume the V1 source
only as an admitted package reference and shall not convert, relabel or fill V1
bytes. The migrator shall evaluate refusal causes in
the order `migration_byte_only_reconstruction`, `migration_input_missing`,
`migration_input_ambiguous`, `migration_input_stale`,
`migration_target_incompatible`, reporting only the first failing check and its
subject. A relinked correspondence records both package keys, the exact
reconstruction inputs and one ordered row per V1 node whose basis is
`relinked-identical` exactly when the source and target node digests are equal.

When a caller lowers requested V2 items, the V2 lowerer shall charge work per request and, for each visited reachable node, per node, per body term and per successor edge, return
`invalid_input` for an absent node key, `unsupported` when any reachable node's
tag is outside the profile, `requires_bound` when the profile requires bounds
and a reachable unbounded numeric, text or collection type has no reachable
`bounded_domain` typed by it, and `failed` at the work limit. A lowered record
carries the node, its exact source-map entries, semantic type, reachable
dependency keys, bounding domain keys, reachable claim keys and a
`quire.contract-ir.semantic/v1` digest. A non-lowered record carries no node.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-038-AC-1 | The dispatcher admits the vendored V1 fixture through the V1 path and each vendored V2 fixture through the V2 path; an unknown version refuses as `unknown_contract_version`; the V1 reader refuses V2 bytes and a V1 node carrying `nominal_identity_preimage` (`unknown_member`); the V2 reader refuses V1 bytes; equal digest bytes under `quire.package.semantic/v1` and `/v2` compare unequal; and the V1 golden read/lower result is unchanged. | Test (TC-047) |
| FR-038-AC-2 | The V2 reader refuses malformed, duplicate-member, unknown-member, noncanonical, stale-dependency, cross-domain digest, unknown required capability, unsupported node tag, invalid graph and invalid source-map inputs with exactly those codes, and every vendored adverse structural mutation returns its recorded outcome, before exposing a package. | Test (TC-048) |
| FR-038-AC-3 | Exact byte, depth, node, edge, occurrence, diagnostic and work limits admit a V2 package; each one-over limit returns `incomplete` with that limit kind, the limit and the consumed counter and no package. | Test (TC-048) |
| FR-038-AC-4 | The recomputed V2 package id equals each vendored fixture id; editing a source-map region, occurrence, raw source digest or capability disposition leaves it unchanged, while editing the edition, a selection, a required feature or a node projection changes it and refuses unless mirrored. | Test (TC-048) |
| FR-038-AC-5 | Every vendored node-identity vector re-derives its recorded digest; a package carrying every vector admits; every vendored invalid mutation, an absent or wrong preimage, and each retained-preimage change of enum case, `semantic_type`, dependency or unit target refuses as `invalid_semantic_graph`. | Test (TC-048) |
| FR-038-AC-6 | Both vendored positive migration vectors return their exact relinked correspondence, every vendored refusal vector returns its exact code and subject, and a relabelled or mismatched target refuses as `migration_target_incompatible`. | Test (TC-049) |
| FR-038-AC-7 | Every admitted V2 node family lowers independently with exact source, type, dependency, bound and claim correspondence; missing, unsupported, unbounded and over-work requests return `invalid_input`, `unsupported`, `requires_bound` and `failed` without a node and without changing sibling records. | Test (TC-050) |

## Dependencies

[FR-035](./FR-035-complete-v1-contract-package-lowering.md) owns the frozen V1
reader and lowering this requirement must not widen. QSpec FR-322 (AC-4, AC-8,
AC-10, AC-11, AC-12), FR-201-AC-5 and FR-195 (AC-1 through AC-5) own the
normative V2 wire, identity-domain and lowering semantics; TC-217 names this
repository as their consumer evidence owner.
