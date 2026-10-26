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
  - target: ix://agent-ix/quire-specification/TC-233
    type: references
  - target: ix://agent-ix/quire-specification/TC-280
    type: references
---
# FR-063: Encode and admit checked-package dimension equivalence nodes

## Description

When a checked-package V2 graph contains a `scalar_type` node with semantic
form `dimension_equivalence`, the Contract IR typed writer SHALL encode its
complete QSpec nominal preimage and graph members, and the strict reader SHALL
admit it only under QSpec FR-322-AC-118. An invalid equivalence SHALL refuse
the package with the QSpec `invalid_semantic_graph` result at the offending
node. IR neither infers an equivalence from names nor rewrites a producer's
pair or scale.

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
`scalar_type` form with a `DimensionEquivalencePreimage` variant. That closed
preimage SHALL contain only version `quire.dimension-equivalence-node/v1`, a
`SourceOwner`, exactly two dimension node ids, and a positive rational
`root_scale`. The writer SHALL preserve the supplied graph, preimage and
occurrence through canonical encoding under [FR-051](./FR-051-encode-checked-package-v2-envelope.md);
the reader SHALL retain the admitted node in its typed graph.

The reader SHALL require the pair to name distinct `scalar_type`/`dimension`
nodes, ordered by ascending node-id digest, with no second
equivalence node for that pair. It SHALL require the node to be self-typed,
have no `declaration`, have `dependencies` exactly the pair, and have one
`anchor` occurrence with its source-order ordinal among that source's
equivalence declarations; the source owner SHALL join the lock.
The nominal key SHALL equal the SHA-256 digest of the JCS preimage, and the
package identity SHALL include the resulting graph projection under
[FR-038](./FR-038-consume-checked-package-v2.md). Reversing the source
declaration's operand spelling SHALL preserve that normalized pair, scale
and key; changing either the pair or scale SHALL change the key.

The reader SHALL require `root_scale` to be present, positive and reduced,
with the exact ratio determined by the two base roots. For equivalences
between derived dimensions, it SHALL require equal normalized maps and
the ratio fixed by those base roots. Across a connected equivalence graph,
it SHALL require every cycle's root-scale product to equal one exactly.
It SHALL apply QSpec FR-322's wire, graph, identity and semantic refusal
order; the encoder is a byte-construction API and does not substitute for
reader admission.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-063-AC-1 | An external consumer builds a complete typed package with one `scalar_type`/`dimension_equivalence` node from the two selected `Time` base dimensions, calls `encode_checked_package_v2`, then reads the bytes through the public strict V2 reader. The reader admits the package and retains the node, its closed nominal preimage, exact two dependencies, self `semantic_type`, absent `declaration`, and one `anchor` occurrence with its source-order ordinal, zero for the sole declaration. The canonical re-encoding retains the same node and package identity. | Test |
| FR-063-AC-2 | The pair is ascending by node-id digest and distinct, the owner is the exact source selected by the lock, and the preimage contains only version `quire.dimension-equivalence-node/v1`, owner, pair, and reduced positive `root_scale`. The node key is the JCS SHA-256 of that preimage. Reversing the source declaration's operands while preserving its meaning produces the same pair, scale and key; changing the pair or scale under a retained key refuses as a stale nominal identity. | Test |
| FR-063-AC-3 | The QSpec TC-233 `dimension-equivalence-rpc-flow-time`, `dimension-equivalence-rpc-clock-time` and `dimension-equivalence-flow-clock-time` cases have respectively `1/1`, `1000/1` and `1000/1` root scales under their published digest-ordered pairs and retain the three original `Time` node keys. The reader admits their consistent three-package graph as one equivalence class, with `anchor` occurrence ordinals following source declaration order. | Test |
| FR-063-AC-4 | Each QSpec TC-280 adverse pair or graph is refused `invalid_semantic_graph` at its equivalence node: descending ids, equal ids, a unit id, a repeated pair, unequal derived-dimension normalized maps, a scale inconsistent with base roots, or a cycle whose exact scale product differs from one. No malformed node is skipped or treated as an unknown form. | Test |
| FR-063-AC-5 | Each QSpec TC-280 adverse member is refused `invalid_semantic_graph` at its equivalence node: absent, zero, negative or unreduced `root_scale`; one missing dependency; non-self `semantic_type`; present `declaration`; owner outside the selected source lock; or a stale key after pair or scale mutation. The reader applies FR-322's stated check order when an earlier independent defect is also present. | Test |
| FR-063-AC-6 | The writer and reader use the typed `dimension_equivalence` form and preimage in the existing V2 graph and envelope. Unknown scalar forms remain closed, and neither an untyped alternate path nor a copied QSpec schema or fixture is added to IR. | Inspection |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns strict admission,
  identity and refusal precedence.
- [FR-051](./FR-051-encode-checked-package-v2-envelope.md) owns the typed
  envelope writer.
- QSpec `ix://agent-ix/quire-specification/FR-322-AC-118` owns the precise
  nominal form, graph meaning and positive/adverse oracle, with published
  cases in `ix://agent-ix/quire-specification/TC-233` and
  `ix://agent-ix/quire-specification/TC-280`.

## Status

Planned. The current IR typed V2 vocabulary does not contain this form.
