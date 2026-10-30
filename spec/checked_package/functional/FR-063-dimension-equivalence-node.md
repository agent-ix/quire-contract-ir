---
id: FR-063
title: "Encode and admit checked-package dimension equivalence nodes"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-051
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-322-AC-118
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-139
    type: references
  - target: ix://agent-ix/quire-specification/TC-233
    type: references
  - target: ix://agent-ix/quire-specification/TC-280
    type: references
---
# FR-063: Encode and admit checked-package dimension equivalence nodes

## Description

When a checked-package V2 graph contains a `scalar_type` node with semantic
form `dimension_equivalence`, the Contract IR typed writer SHALL encode its
complete QSpec structural id and graph members, and the strict reader SHALL
admit it only under QSpec FR-322-AC-118. For an invalid equivalence, the strict
reader SHALL refuse the package at the first applicable QSpec FR-322 wire,
graph or semantic check, with no partial admitted package. IR
neither infers an equivalence from names nor rewrites a producer's pair or
scale.

## Inputs

The typed V2 graph supplied to the encoder of
[FR-051](./FR-051-encode-checked-package-v2-envelope.md), or untrusted V2
bytes, selected package evidence, lock and limits supplied to the strict
reader of [FR-038](./FR-038-consume-checked-package-v2.md). The authoritative
form is QSpec `ix://agent-ix/quire-specification/FR-322-AC-118` and the
published `proposals/checked-package-v2/schema.json`; IR carries no schema
copy.

## Outputs

Canonical checked-package V2 bytes retaining the typed equivalence node, or
an admitted package retaining that node and its occurrence, or one typed,
located refusal with no partial admitted package.

## Behavior

The typed V2 vocabulary SHALL include `dimension_equivalence` as a closed
`scalar_type` form with a closed dimension-equivalence body. That closed body contains exactly two dimension node ids and a positive rational `root_scale`, while the node retains a `SourceOwner`. The writer preserves the supplied graph, structural id and
occurrence through canonical encoding under [FR-051](./FR-051-encode-checked-package-v2-envelope.md);
the reader retains the admitted node in its typed graph.

The reader SHALL require the pair to name distinct `scalar_type`/`dimension` nodes in structural-id order, with no second equivalence node for that pair. The node is self-typed, have no `declaration`, have `dependencies` exactly the pair, and have one `anchor` occurrence with its source-order ordinal among that source's equivalence declarations; the source owner joins the lock. Reversing the source declaration's operand spelling preserves the normalized pair and scale. A valid pair or scale edit changes the retained body without causing a stale-id refusal.

The closed V2 wire schema SHALL require `root_scale` with positive-integer
`numerator` and `denominator` strings, exactly two `dimension_node_ids`, and
no `declaration` member. A missing scale, zero or negative scale component,
one-entry pair or present declaration SHALL refuse `malformed_wire` at the
node before graph checks. The graph check SHALL require a schema-valid
`root_scale` to be reduced, with the exact ratio determined by the two base
roots. For equivalences
between derived dimensions, it SHALL require equal normalized maps and
the ratio fixed by those base roots. Across a connected equivalence graph,
it SHALL require every cycle's root-scale product to equal one exactly.
It SHALL apply QSpec FR-322's wire, graph and semantic refusal
order; the encoder is a byte-construction API and does not substitute for
reader admission.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-063-AC-1 | PLANNED/UNRUN (IR-1141). An external consumer encodes a typed dimension_equivalence node, then reads it through the public strict V2 reader. The admitted package retains the structural id, exact pair, positive reduced scale, two dependencies, self semantic_type, absent declaration, owner and one anchor occurrence. Canonical re-encoding retains the same complete node. | Test |
| FR-063-AC-2 | Withdrawn by the IR-1141 structural-identity decision under QSpec #517: the reader does not recompute identity from body content or refuse a retained structural id solely because a valid body changed. This id is retained and never reused. | Withdrawn |
| FR-063-AC-3 | PLANNED/UNRUN (IR-1141). QSpec TC-233 dimension-equivalence-rpc-flow-time, dimension-equivalence-rpc-clock-time and dimension-equivalence-flow-clock-time retain respectively 1/1, 1000/1 and 1000/1 root scales over their structurally ordered node pairs. The reader admits their consistent three-package graph as one equivalence class with anchor ordinals in source declaration order. | Test |
| FR-063-AC-4 | Each QSpec TC-280 adverse pair or graph is refused `invalid_semantic_graph` at its equivalence node: descending ids, equal ids, a unit id, a repeated pair, unequal derived-dimension normalized maps, a scale inconsistent with base roots, or a cycle whose exact scale product differs from one. No malformed node is skipped or treated as an unknown form. | Test |
| FR-063-AC-5 | PLANNED/UNRUN (IR-1141). Missing or invalid scale, one-entry pair and present declaration refuse malformed_wire at the node before graph checks. Unreduced scale, missing dependency, non-self semantic_type or owner outside the selected source lock refuses invalid_semantic_graph at its exact node under FR-322 precedence. No stale-key refusal or vector hash comparison occurs. | Test |
| FR-063-AC-6 | The writer and reader use the typed dimension_equivalence form in the V2 graph and envelope. Unknown scalar forms remain closed; no untyped alternate path or copied QSpec schema or fixture is added. | Inspection |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns strict admission,
  structural identity and refusal precedence.
- [FR-051](./FR-051-encode-checked-package-v2-envelope.md) owns the typed
  envelope writer.
- QSpec `ix://agent-ix/quire-specification/FR-322-AC-118` owns the precise
  nominal form and graph meaning; `ix://agent-ix/quire-specification/FR-322-AC-139`
  owns the wire-before-graph refusal oracle, with equivalence cases in
  `ix://agent-ix/quire-specification/TC-233` and
  `ix://agent-ix/quire-specification/TC-280` and cross-form mutations in
  QSpec's `proposals/checked-package-v2/node-identity-vectors.json`.

## Status

Planned. The current IR typed V2 vocabulary does not contain this form.
