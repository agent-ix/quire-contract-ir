# CheckedPackage V2 public interchange

This directory is the normative public transport contract for I04 immutable
checked packages. It is intentionally a schema-and-vector boundary, not a
native Rust package API: QSL remains free to use internal parser/CST/package
structures, but any external consumer receives only the versioned contract.

## Contract contents

- `schema.json` defines the closed `quire.checked-package/v2` wire shape.
- `node-identity-preimage.schema.json` defines the closed RFC 8785 preimages for
  complete-V1 enum, dimension and declared-unit node identities.
- `node-identity-vectors.json` supplies known JCS SHA-256 vectors for those
  preimages.
- `fixtures/` contains independently authored positive package documents and
  structural refusal/mutation cases. Exact reader-limit boundaries,
  duplicate-member/noncanonical raw-byte cases and production-reader outcomes
  remain TC-217 Contract IR consumer evidence; this schema qualification does
  not claim them complete.
- `model-effective-declaration.schema.json` and
  `model-effective-declaration-vectors.json` define the QSL-derived
  `quire.model.effective-declaration/v1`, `quire.model.effective-view/v1` and
  `quire.model.object-universe/v1` preimages selected by
  `quire.model.complete/v1`, with JCS SHA-256 vectors, the reference key order
  and invalid mutations. They add no CheckedPackage V2 wire member.
- The checked semantic graph is a closed `quire.checked-semantic-graph/v2`
  tagged union. It is source-language-neutral transport, not Contract IR and
  not an alternative language semantic authority.

## Identity rules

The package key is the selected contract version and a
`quire.package.semantic/v2` digest over a non-circular JCS preimage. The
preimage is exactly the `quire.checked-package-id/v2` object containing ordered
edition, profile, definition, model, feature and dependency selections plus the
checked semantic graph's typed-key ordered identity projection. It excludes
package identity itself, source-occurrence/source-map data, raw-byte artifacts,
presentation, runtime inputs, installed backends, diagnostic wording and reader
limits.

Raw source documents, native definition documents, `sha256-jcs` domain packages,
checked semantic graph/package identities and generated Contract IR identities
remain separately typed. Equal hash bytes never authorize substitution across
those domains.

For the nominal scalar node forms covered by the node-preimage schema, the
`quire.checked-semantic-node/v1` digest is SHA-256 of the exact RFC 8785 JCS
preimage. An owner subject must match an admitted source, definition or model
selection; source/definition owners use authority plus identity, and model
owners use the domain package identity plus the IR node identity. Revisions,
versions and digests remain package
lock evidence but are excluded from the semantic node preimage, allowing an
unchanged declaration to retain identity across unrelated owner revisions.
Qualified names are arrays of ASCII identifier segments. Arbitrary-precision
integers/rationals use canonical decimal strings with no leading zero and no
`-0`, a positive denominator, greatest-common-divisor one and exactly `0/1` for
zero. Unordered enum members
and dimension terms are ascending by their canonical JCS key; ordered enum
members retain declaration order. Duplicate/zero-exponent terms refuse.

The JSON Schema fixes structural shape; the strict reader additionally enforces
these semantic constraints. A targetless root unit has scale `1/1` and offset
`0/1`; every non-root unit has a non-null same-dimension target, nonzero scale,
and an acyclic path to the unique root. Derived-dimension terms reference base
dimension keys. Cross-field joins are exact: an enum member's `semantic_type`
and sole dependency equal its preimage declaration key and its enum literal
body equals the preimage case; a dimension is self-typed and depends exactly on
its term keys; a unit's `semantic_type` equals its dimension key and its
dependencies are exactly that dimension plus its non-null target. Changing a
body/type/dependency while retaining the nominal preimage therefore refuses.
Runtime compound-unit values are evaluator-owned
`quire.value.compound-unit/v1` identities defined with the complete-value
profile; they are not I04 graph nodes. The concrete RFC 6902
patches in `invalid_mutations` retain the base vector's old node digest and must
all refuse as `invalid_semantic_graph`.

## Reader boundary

A reader selects exactly this version, rejects duplicate and unknown members,
checks canonical bytes and all identity joins before exposing a graph, and
reports malformed/stale/cross-domain/unknown-capability input as typed refusal.
For a node form covered by the node-preimage schema, it reconstructs the closed
preimage from the graph and lock, hashes its JCS bytes and requires exact
`node_id` equality; stale keys are `invalid_semantic_graph`.
Its outcome is the closed sum type admitted/refused/incomplete. It enforces
caller-provided byte/depth/node/edge/occurrence/diagnostic/work limits;
exhaustion is incomplete, not an empty graph or a successful package.

The QSpec qualification crate validates schema/vector correspondence only. The
Contract IR consumer owns the production Rust strict reader and lowering path.
