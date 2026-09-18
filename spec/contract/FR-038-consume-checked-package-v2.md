---
id: FR-038
title: "Consume the CheckedPackage V2 contract and refuse every other version"
type: FR
relationships:
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
  - target: ix://agent-ix/quire-specification/FR-340
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/109
    type: references
---
# FR-038: Consume the CheckedPackage V2 contract and refuse every other version

## Description

When Contract IR receives checked-package bytes, the consumer shall parse
`contract_version` exactly once before any further decoding, admit a
`quire.checked-package/v2` package only after re-deriving every package and
nominal node identity, and lower admitted items independently. Any other
contract version is refused with a typed `unknown_contract_version` code.

## Inputs

Untrusted wire bytes; caller-selected read limits, for which the crate ships one
named finite default appropriate to a single local request — 1048576 bytes, 128
nesting levels, 10000 nodes, 100000 edges, 100000 occurrences, 10000 diagnostics
and 1000000 term-validation visits — as stable API, every member finite so that
the default admits no unbounded read; package evidence holding
the authoritative raw-artifact digests (from supplied bytes or a verified
digest store), the authoritative `sha256-jcs` domain-package digests, and the
reader-supported required features; for lowering, requested node keys and a
lowering profile (supported node tags, bounded-domain requirement, work
limit).

The normative producer contract is QSpec
`proposals/checked-package-v2/` at
`agent-ix/quire-specification@0c7497ee0f7c99b2c6fd69b283c314edbe53a1bb`,
vendored byte-exact under `tests/fixtures/checked-package/` with a
`PROVENANCE` file naming each source path, blob and SHA-256. The vendored set
is `README.md`, `schema.json`, `node-identity-preimage.schema.json`,
`node-identity-vectors.json`, `operation-catalog.json` and the six fixtures
`adverse.json`, `positive-all-families.json`,
`positive-nominal-identities.json`, `positive-operation-identities.json`,
`positive-clause-operations.json` and `positive-control-operations.json`.

## Outputs

- A closed read result: admitted, refused or incomplete (limit kind, limit,
  consumed). A refusal carries a typed code, a structural path, and — for a
  refusal whose code is paired with a catalogued cause — that cause tag and the
  exact node key it is located at, so a caller distinguishes the condition and
  the offending node without parsing prose.
- One lowering record per requested item, drawn from a closed seven-member
  vocabulary: `lowered`, `unsupported`, `requires_bound`, `invalid_input`,
  `failed`, `invalid_body` and `body_incomplete`. The last two are defensive:
  they report a body that refuses or stops at a validation limit during the
  lowering walk rather than guessing past it, so the work count and the closure
  stay exact. A package this reader admitted never yields either, and lowering
  every node of every admitted package is required to produce neither — but a
  consumer written to the vocabulary shall handle all seven, because a record
  kind that exists and is undeclared is a consumer that is incomplete by
  construction.
- The lowering result carries the admitted package's `package_id` alongside the
  records. It is not an aggregate package artifact; see
  [FR-035](./FR-035-complete-v1-contract-package-lowering.md) for the
  `ContractPackage` obligation, which this path does not discharge.

## Behavior

The reader shall measure raw bytes against the byte limit, parse strict JSON
once (duplicate members refuse, nesting charged against the depth limit),
require canonical bytes, and read `contract_version` exactly once. It shall
admit only `quire.checked-package/v2`; any other version, or a missing or
malformed `contract_version`, refuses before any version-specific decoding
begins. This is a refusal control, not a compatibility layer: it never
relabels, converts or widens the input to fit the current contract.

The reader shall recompute `package_id` as the SHA-256 of the RFC 8785
canonical bytes of `identity_preimage` under `quire.package.semantic/v2`,
require the preimage's lock members to equal the lock, and require its
`identity_projection` to equal the occurrence-free projection of the graph in
graph order. It shall check each digest's declared domain before its bytes,
each locked source/definition byte digest against the package evidence's raw
artifact digests and each selected `sha256-jcs` domain package digest against
its separately typed domain package evidence, every required feature against
the reported `available` capability and the reader-supported feature set,
every node tag, family form and semantic term, reference and dependency
resolution, cycles outside one explicit `recursion_group`, and the total
source map.

For `model_selections`, whole-array uniqueness is checked before any entry's
digest is evaluated against evidence, so an array carrying both a repeated
entry and an entry the evidence does not attest has one determined outcome
rather than a position-dependent one: the reader shall first refuse, as
`malformed_wire` at `lock.model_selections`, an array that repeats an entry
anywhere in it (identity, version, digest domain and digest all equal), and
only once no
entry repeats shall it evaluate any entry's digest against the domain package
evidence. An array carrying both a repeated entry and an entry whose digest
the evidence does not attest therefore always refuses as `malformed_wire`,
never as `stale_dependency`, regardless of which defect appears first.

When a node is an enum declaration, enum member, dimension or declared unit, the
reader shall reconstruct the closed nominal preimage, require
`node_id.digest` to equal the SHA-256 of its canonical bytes, require the owner
to join an exact lock selection (source and definition owners by authority and
identity, model owners by domain package identity with a nonempty IR node
identity), and enforce identifier, canonical-integer,
reduced-rational, member/term order, duplicate/zero-exponent, root/non-root
unit, base-dimension and cross-field (`semantic_type`, dependencies, enum
literal body) rules. Any violation refuses as `invalid_semantic_graph`; a
nominal form without its preimage, or a non-nominal form carrying one, refuses
the same way.

A `model` node carries one of the eighteen declared business and systems
meanings as its `semantic_form`: `model_import`, `object_type`, `value_type`,
`variant_type`, `record_value_type`, `event_type`, `state_machine`, `process`,
`persistence_interface`, `namespace`, `field_declaration`,
`operation_declaration`, `clause_member_declaration`, `systems_interface`,
`systems_part`, `systems_port`, `systems_connection` and `systems_allocation`.
An `expression` node additionally carries the `collection` and `value_read`
forms. A named, source-declared node carries `declaration.qualified_name`, the
package-local declaration path; a `literal` term carries `type`, the node key of
its package type; an `application` term carries `operation` and `result_type`
beside `operator` and `arguments`. The reader shall read each of these as a
declared member, refuse a node or term that omits one where the contract
requires it as `invalid_semantic_graph`, and carry every one of them into the
occurrence-free identity projection it compares against
`identity_preimage.identity_projection`, so a package whose preimage omits a
`declaration`, a literal `type` or an application `operation` refuses rather
than admitting a projection that disagrees with the graph.

### Frame bodies

A `state` node of `semantic_form` `frame` carries a frame body, and only a
frame body: `{"term": "frame", "modifies": [...], "creates": [...],
"deletes": [...]}`, with all three members required, each a `uniqueItems`
array of node keys per the wire schema. The reader shall validate a
`state`/`frame` node's `body` against this shape alone and every other node's
`body` against the semantic-term grammar alone. A frame body that omits one of
the three members, carries any further member, repeats an entry within one
member, or carries an entry that is not a node key, refuses as
`invalid_semantic_graph` at the frame body; so does a node of any other tag or
form whose body carries the frame shape. Three empty members are admitted.

When a caller lowers requested items, the lowerer shall charge work per request and, for each visited reachable node, per node, per body term and per successor edge, return
`invalid_input` for an absent node key, `unsupported` when any reachable node's
tag is outside the profile, `requires_bound` when the profile requires bounds
and a reachable unbounded numeric, text or collection type has no reachable
`bounded_domain` typed by it, and `failed` at the work limit. A lowered record
carries the node, its exact source-map entries, semantic type, reachable
dependency keys, bounding domain keys, reachable claim keys and a
`quire.contract-ir.semantic/v1` digest. A non-lowered record carries no node.

The lowerer shall select each request's single record under this total order, so
that a request satisfying two conditions has one determined outcome rather than
a traversal-dependent one:

1. The per-request work unit is charged first. A work limit that unit alone
   exceeds returns `failed` before the requested key is looked up, so a zero
   work limit returns `failed` and not `invalid_input`.
2. An absent node key returns `invalid_input`.
3. Work is then charged per visited node, per body term and per successor edge;
   exceeding the limit at any charge returns `failed`.
4. Over the completed closure, `unsupported` is decided before `requires_bound`
   and wins outright: a closure holding both an out-of-profile tag and an
   unbounded type returns `unsupported`.
5. `requires_bound` is decided last, and only when the profile requires bounds.

Where a record names the "first" offending node, first shall mean the least
`node_id` in ascending key order over the whole closure, not the first node the
traversal reached. A record therefore never depends on visit order.

A reachable type is unbounded exactly when its family and semantic form are a
`scalar_type` of form `integer`, `rational`, `decimal` or `text`, or a
`composite_type` of form `sequence`, `set`, `bag` or `ordered_set`. No other
family and no other form of those two families is unbounded, so `boolean`,
`float32`, `float64`, `dimension`, `unit`, `enum`, `option`, `record`, `tuple`,
`alias` and `reference` never raise `requires_bound`. A type is bounded when the
closure holds a reachable `bounded_domain` node whose `semantic_type` is that
type's key.

A lowered record's `dependencies` shall be every node key reachable from the
requested node excluding the requested node itself, in ascending key order.
The lowerer shall derive `bounds` and `claims` as filtered views of that same
reachable set — `bounded_domain` nodes and `claim` nodes respectively — and not
as a partition of it: every key in `bounds` and every key in `claims` also
appears in
`dependencies`. The `quire.contract-ir.lowered-node/v1` preimage carries all
three lists as written, so an independent re-derivation of `ir_id` that treats
the three as disjoint disagrees byte for byte.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-038-AC-1 | Each vendored fixture admits; an unknown, empty, absent or malformed `contract_version` refuses as `unknown_contract_version` or `malformed_wire` before any version-specific decoding; and the strict parse (duplicate-member, noncanonical) refusals occur before a version is selected. | Test (TC-048) |
| FR-038-AC-2 | The reader refuses malformed, duplicate-member, unknown-member, noncanonical, stale-dependency, cross-domain digest, unknown required capability, unsupported node tag, invalid graph and invalid source-map inputs with exactly those codes, and every vendored adverse structural mutation returns its recorded outcome, before exposing a package; a lock reference or owner carrying `authority`, `revision` or `export` refuses as `unknown_member`, a domain package selection outside `sha256-jcs` as `digest_domain_mismatch`, one whose digest is attested only as a raw artifact as `stale_dependency`, and a `model_export` semantic form as `invalid_semantic_graph`. | Test (TC-048) |
| FR-038-AC-3 | Exact byte, depth, node, edge, occurrence, diagnostic and work limits admit a package; each one-over limit returns `incomplete` with that limit kind, the limit and the consumed counter and no package. | Test (TC-048) |
| FR-038-AC-4 | The recomputed package id equals each vendored fixture id; editing a source-map region, occurrence, raw source digest or capability disposition leaves it unchanged, while editing the edition, a selection, a required feature or a node projection changes it and refuses unless mirrored. | Test (TC-048) |
| FR-038-AC-5 | Every vendored node-identity vector re-derives its recorded digest; a package carrying every vector admits; every vendored invalid mutation, an absent or wrong preimage, and each retained-preimage change of enum case, `semantic_type`, dependency or unit target refuses as `invalid_semantic_graph`; a model owner admits when its identity names a selected domain package and refuses as `invalid_semantic_graph` when it names none or carries an empty node. | Test (TC-048) |
| FR-038-AC-6 | Every admitted node family lowers independently with exact source, type, dependency, bound and claim correspondence; missing, unsupported, unbounded and over-work requests return `invalid_input`, `unsupported`, `requires_bound` and `failed` without a node and without changing sibling records. | Test (TC-050) |
| FR-038-AC-9 | The shipped default read-limit policy is exactly those seven finite values, every member is strictly positive and finite, and each meter is enforced at its own true measured boundary against a real package: the package's exact measured consumption for that meter admits it, and one below that exact value refuses it as `incomplete`, naming that meter and reporting the true consumption. The shipped default is far larger than any vendored fixture, so this boundary is proven against each meter's real measured cost rather than against the default value itself — no vendored fixture approaches that scale, and none is fabricated to do so. | Test (TC-048) |
| FR-038-AC-7 | Lowering every node of every vendored fixture under a profile supporting every tag yields no `invalid_body` and no `body_incomplete` record, and the seven-member record vocabulary is exhaustive: no eighth kind is reachable and each of the seven is named. | Test (TC-052) |
| FR-038-AC-8 | A closure holding both an out-of-profile tag and an unbounded type returns `unsupported`; a zero work limit returns `failed` for an absent key rather than `invalid_input`; each named offending key is the least in ascending order rather than the first visited; each of the eight unbounded forms raises `requires_bound` and each other declared form of those two families does not; and a lowered record's `dependencies` contains every key in its `bounds` and `claims`. | Test (TC-052) |
| FR-038-AC-10 | A `lock.model_selections` entry that repeats an earlier entry's identity, version, digest domain and digest verbatim, mirrored identically into `identity_preimage.model_selections`, refuses as `malformed_wire` at `lock.model_selections`; the same lock-side repeat left unmirrored in the identity preimage refuses earlier, as `stale_dependency` at `lock`, because the preimage/lock equality check runs first; two entries sharing identity and version but differing in digest refuse as `stale_dependency` at `lock.model_selections`, never as `malformed_wire`. | Test (TC-048) |
| FR-038-AC-11 | A `lock.model_selections` array carrying both a repeated entry and an entry whose digest the package evidence does not attest refuses as `malformed_wire` at `lock.model_selections`, never as `stale_dependency`, regardless of whether the repeated entry or the stale entry appears first in the array — the uniqueness check runs over the whole array before any entry's digest is evaluated against evidence, so the outcome does not depend on array position. | Test (TC-048) |
| FR-038-AC-16 | The vendored `tests/fixtures/checked-package/` tree is byte-identical to `proposals/checked-package-v2/` at `0c7497ee0f7c99b2c6fd69b283c314edbe53a1bb` for every vendored path, `PROVENANCE` names that commit and each path's git blob and SHA-256, and each of the five positive fixtures admits and re-derives its recorded `package_id`: `b0b40569b19f00bd06ae08e218f0f77d114ce97cf42d2b6fa7c868a96a18bdad` (all-families), `b70a9f27c9ef49711fb603d56014aa5ce092379cd820c5e62a0154c89877e7b4` (nominal-identities), `dca508e418e70d99bcaf49384ea48c7f909a389d8c5b15541e5fb1bddd426168` (operation-identities), `d011de207a1fe5578b89d185f9394ef6a995c16244b72d63ba2775c6518b5952` (clause-operations) and `c76a26bf468ae66a74ea3f79dde881657b5fc9c0c555535fcc12d59b4cc2b69f` (control-operations). | Test (TC-048) |
| FR-038-AC-17 | Each declared wire member the vendored contract carries is read and enters the identity projection: a declaring node's `declaration.qualified_name`, a `literal` term's `type`, and an `application` term's `operation` and `result_type`. Deleting any one of them from a single node of an otherwise unmodified `positive-operation-identities` package, mirrored into `identity_preimage.identity_projection`, refuses as `invalid_semantic_graph`; deleting it from the graph alone refuses at the projection comparison; and the eighteen `model` forms and the fifteen `expression` forms the contract declares are each admitted as a node form while a nineteenth `model` form and a sixteenth `expression` form refuse as `invalid_semantic_graph`. | Test (TC-048) |

## Dependencies

QSpec FR-322 (AC-4, AC-8, AC-10), FR-201 (AC-2, AC-3) and FR-195 (AC-1 through
AC-5) own the normative V2 wire, identity-domain and lowering semantics;
TC-217 names this repository as their consumer evidence owner.
