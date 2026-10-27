---
id: FR-038
title: "Consume the CheckedPackage V2 contract and refuse every other version"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-codegen/FR-033
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-013
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-358
    type: references
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/AD-003
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-66
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-120
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-121
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-122
    type: references
  - target: ix://agent-ix/quire-specification/FR-330-AC-4
    type: references
  - target: ix://agent-ix/quire-specification/FR-152
    type: references
  - target: ix://agent-ix/quire-specification/FR-154
    type: references
  - target: ix://agent-ix/quire-specification/FR-043
    type: references
  - target: ix://agent-ix/filament-core-data/FR-094
    type: references
  - target: ix://agent-ix/filament-core-data/FR-095
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-092
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-094
    type: references
  - target: ix://agent-ix/quire-specification/FR-201
    type: references
  - target: ix://agent-ix/quire-specification/FR-195
    type: references
  - target: ix://agent-ix/quire-specification/FR-340
    type: references
  - target: ix://agent-ix/quire-specification/FR-370
    type: references
  - target: ix://agent-ix/quire-specification/FR-481
    type: references
  - target: ix://agent-ix/quire-specification/FR-415
    type: references
  - target: ix://agent-ix/quire-specification/FR-440
    type: references
  - target: ix://agent-ix/quire-specification/FR-464
    type: references
  - target: ix://agent-ix/quire-specification/FR-465
    type: references
  - target: ix://agent-ix/quire-specification/FR-466
    type: references
  - target: ix://agent-ix/quire-specification/FR-271
    type: references
  - target: ix://agent-ix/quire-specification/FR-272
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-040
    type: references
  - target: ix://agent-ix/quire-contract-ir/AD-006
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-357
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
named finite default appropriate to a single local request — 1048576 bytes,
10000 nodes, 100000 edges, 100000 occurrences, 10000 diagnostics and 1000000
term-validation visits — as stable API, every member finite so that the default
admits no unbounded read, and no member a nesting depth, the reader having no
depth limit (see "Reading"); package evidence holding
each selected domain package's Semantic IR 2.0.0 document
supplied as bytes under its `sha256-jcs` digest, each selected dependency's
admitted V2 package supplied under its library identity, and the reader-supported
required features; for lowering, requested node keys and a
lowering profile (supported node tags, bounded-domain requirement, work
limit).

The normative producer contract is the QSpec I04 CheckedPackage interface,
owned by `agent-ix/quire-specification`. This repository states the wire shape
it admits in its own reader — `crates/quire-contract-model/src/checked_package/`
— and holds no copy of the upstream artifacts.

## Outputs

- A closed read result: admitted, refused or incomplete (limit kind, limit,
  consumed, and the location of the value whose charge failed). A refusal
  carries a typed code and the location of the value it is about, so a caller
  distinguishes the condition and locates it without parsing prose. An
  `unknown_contract_version` refusal also carries the `contract_version`
  string the reader read. A frame-body refusal additionally carries the cause
  tag FR-322's `DiagnosticCausePairing` pairs with that code — `missing-name`
  with `missing_declaration`, `malformed-declaration` with
  `invalid_model_binding` — and the node key of the offending entry or, for a
  canonical-order defect, of the frame node itself, so a caller need not
  re-derive from the path alone why admission failed or at which node.
  The public model-intake refusal conversion retains its existing code, cause
  and selection-row pointer. Declaration identity/origin retention below is
  implemented at the private intake return and checked by crate-local tests,
  not part of this public output.
  A `stale_dependency` refusal with cause `content-mismatch` or
  `byte-digest-mismatch` also carries the two typed digest identities it compares, as
  `expected` and `actual` ("Digest mismatches: content and byte").
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
- The lowering result carries the records alongside the call's canonical
  `ContractPackage`, which records the admitted package's `package_id` as its
  source identity; see
  [FR-035](./FR-035-complete-v1-contract-package-lowering.md).
- A typed accessor on the admitted package, `model_object_fields`, returning a
  model object type's effective fields with their derived member types, integer
  bounds as values; see "Typed accessor for a model object type's fields"
  (FR-038-AC-136 through FR-038-AC-144, IR-628).
- A typed accessor on the admitted package for a scalar application's ordered
  operands. Each entry carries its argument ordinal, a typed child identity,
  and its own inclusive integer range; see "Typed scalar application operands"
  (FR-038-AC-159 through FR-038-AC-164).
- A typed accessor on the admitted package for a `value`/`parameter` node's
  declared name and level, `parameter_declaration`; see "Typed parameter
  declaration" (FR-038-AC-258 through FR-038-AC-261, IR-832).
- A typed accessor on the admitted package for the destination end of a
  `quire.op.model.reaches` application, `relationship_end`; see "Typed relationship
  end of a reaches application" (FR-038-AC-262 through FR-038-AC-268, IR-851).
- A typed accessor on the admitted package for the finite model-object population
  a `quire.op.model.reaches` application walks, `reaches_population`, returning the
  selector FR-053 reads; see "Typed population of a reaches application"
  (FR-038-AC-269 through FR-038-AC-280, IR-852).

- A typed composite operand accessor, `composite_application_operands`,
  returning authentic positional children, authored shapes and introduced positions
  with exact decimal bounds, explicit unbounded/Whole descriptors and typed refusals
  (implemented, AC-177 through AC-182), together with the admitted comparison
  operator (implemented, FR-038-AC-246); see "Typed authored domains of composite
  equality and inequality operands".

## Behavior

### Locating refusals and limits

Every location the reader reports is an RFC 6901 JSON pointer into the
document it was given. The reader builds it from its own position while it
walks the document: each object member name is escaped (`~` as `~0`, `/` as
`~1`) and each array element is named by its index. It is never rebuilt from
member names afterwards. Every pointer resolves in that document. A refusal
about a value points at that value. A refusal about a member the document
lacks points at the object that lacks it. The empty pointer names the whole
document.

A refusal about the byte stream rather than a value carries no pointer. These
are malformed JSON and non-canonical bytes. The one refusal that carries a
second pointer, `document_pointer`, is about a number inside a supplied model
document and not about the package's bytes ("A model document is a value inside
a supplied document"); its `noncanonical_wire` code does not make it a refusal
about the package's byte stream. A repeated member points at that
member. A closed-schema decode refusal points where the decoder stopped: at
an unknown member, at the object missing a required member, or at a value of
the wrong kind. Inside a nominal identity preimage, and inside its owner,
serde reads the members from a buffer it does not track. There the reader
decodes the variant the preimage's `version` (or the owner's `kind`) selects,
so the pointer still names the member at fault. Inside a nominal preimage or
its owner, a missing member or a value of the wrong kind refuses as
`invalid_semantic_graph`, the code every nominal preimage defect carries,
rather than `malformed_wire`; an unknown member there refuses as
`unknown_member`, as it does anywhere else.

A refusal located at a graph node points at the node's member the failed
check read. Examples are its `node_id`, its `dependencies` (or one entry of
them), its `declaration.qualified_name`, one frame entry, one `aggregate`
member, or an `operation` law, mode or member. A reference that resolves to no
node points at that reference. A stale preimage or lock mirror points at the
first value, in document order, where the retained copy differs from the one
the reader derived. Within a `lock.model_selections` defect class, the
pointer names the first entry of that class in array order. The array
position never decides the code. A cycle without one shared
`recursion_group` points at the lowest-positioned member at fault: at its
`recursion_group` when it carries a different one, else at the node, which
lacks the member. Where this specification names the location of a refusal
as a dotted member path, such as `lock.model_selections`, the returned
pointer names that member at its concrete indices, or the object that lacks
it.

An `unknown_contract_version` refusal points at `/contract_version` and
carries the version string read there.

An incomplete outcome carries the pointer of the value whose charge failed,
for every limit except the byte limit. The byte limit is charged before any
value is parsed. The others are charged as follows:

- The node limit at the first node past it.
- The edge limit at the dependency that took the count past it.
- The occurrence limit at the source-map entry or region that did.
- The diagnostic limit at the first entry past it.
- The work limit at the value whose validation took the meter past it: a
  node body, a nominal preimage member, an `operation`, a graph edge's
  member, or a diagnostic detail.

### Reading

The reader shall measure raw bytes against the byte limit, read the document
as strict JSON (duplicate members refuse), require canonical bytes, and read
`contract_version` exactly once. The reader has no depth limit, as merged QSpec
FR-322 states ("Identity and validation" and "Body grammar", FR-322-AC-41): the
closed body grammar (see "The flat wire" below) fixes the JSON depth of every
package whatever its node count, so depth is never a limit kind. No outcome of the
V2 reader names a depth, `CheckedPackageReadLimits` has no depth member (so
`CheckedPackageLimit` has no `Depth` variant, a breaking change for a consumer that
builds it), and no constant of the V2 reader
(`crates/quire-contract-model/src/checked_package/`) caps a value a caller supplies
by a nesting depth. The v1 limits of FR-019 and FR-023 are a
different surface and are unchanged. A body nested beyond the grammar, within what the
strict parse reads, refuses `malformed_wire` at the first value outside it, at the body
grammar of the order of checks, and is never an `incomplete` outcome. A document nested
past the strict parse's own recursion limit (`serde_json`'s default of 128, which
returns an error before it recurses deeper) refuses `malformed_wire` with no pointer,
as malformed JSON does (FR-038-AC-24), at the strict parse, never `incomplete`, and
without a stack overflow (FR-038-AC-117). The reader parses the document under that
limit, with neither `disable_recursion_limit` nor `serde_stacker`. A syntax or
duplicate-member defect anywhere in the document refuses first. No walk of the V2
reader after the strict parse (the body grammar, the term walks, the preimage, the
closure and dependency walks and the lowering walk) recurses on the call stack at any
depth: each runs on an explicit heap stack, a term walk in document pre-order on the
shared `quire-walk` crate's, so a chain of nodes 100000 deep is read and lowered on a
thread whose stack is 256 KiB. The closed-schema decoder shall decode the
document directly into the typed V2 wire. It shall not build or retain a
full-document `serde_json::Value` tree or allocate a second full-document
canonical byte buffer. A node `body` or a diagnostic `details` term may remain a `Value` inside the typed wire; its
recursion stays bounded by the strict parse's limit of 128 levels. A body
nested between the grammar and the parse's limit (a window of about 90 to
126 JSON levels) refuses at the body grammar on a 256 KiB stack in a debug
build; a shipped projection at that depth is an unknown wire member and
refuses before identity validation on that stack. Direct decoding shall retain the same strict syntax and
duplicate-member refusal, canonical-byte decision, closed-wire losslessness
and unknown-member refusal, first-refusal precedence and JSON pointers,
identity checks, source-map content and admitted typed public API. For the
flat 100000-node chain, the additional reader peak resident memory shall
meet FR-038-AC-214 under its isolated measurement procedure; fixture
generation, caller-owned input bytes, selected evidence, lowering and
runtime startup are outside that measured call.

Every value an admitted package holds nests to a depth the grammar fixes, so its
clone, comparison, `Debug` rendering, lowering and drop
need no stack that follows the package's size, and the V2 reader has no `stacker`,
`serde_stacker` or `on_stack_for`. It shall
admit only `quire.checked-package/v2`; any other version, or a missing or
malformed `contract_version`, refuses before any version-specific decoding
begins. This is a refusal control, not a compatibility layer: it never
relabels, converts or widens the input to fit the current contract.

Each closed wire vocabulary is decoded into its own enum exactly once, at
intake: the node family (`node_tag`), each family's semantic forms
(`semantic_form`, decoded together with the family, so a form of the wrong
family cannot be represented), the lock selection `role` and the capability
report `disposition`, alongside the diagnostic stage, code and cause and the
occurrence role already typed on the wire. A selection role or disposition
outside its vocabulary is a wire-shape refusal (`malformed_wire` at that
value); an unknown family or form keeps its own graph refusal below.
Past intake no decision that depends on a node's family or form, a
selection role or a disposition compares a wire string: each matches the
decoded enum exhaustively, listing every form, with no catch-all arm, so
adding a member is a compile error at each place that must decide what it
means. The semantic term grammar inside a node's `body` (the `term` tag, an
operation member's `kind`, a mode's `kind`) is not one of these enums: the
body validators read it from the JSON at the wire edge. Admission retains the
validated body-reference adjacency and term counts for lowering; lowering
uses those admitted facts rather than parsing a body again for each request.

The node-family vocabulary includes `hybrid` with exactly the
`hybrid_model`, `hybrid_mode` and `hybrid_transition` forms of QSpec FR-481.
The typed family/form decoder admits those three combinations. The strict
published hybrid schema refuses `hybrid_flow` and any form outside this
three-form set as `malformed_wire` before the hybrid body step; it does not
infer a form from the body. The checked package's public
typed wire exposes the new `HybridForm` enum; FR-019 lists that type in its
public-item inventory. No separate hybrid accessor is introduced.

The reader SHALL recompute `package_id` as the SHA-256 of the RFC 8785
canonical bytes of the seven-member wire `identity_preimage` with one derived
`identity_projection` member added, under `quire.package.semantic/v2`.
The derived member SHALL contain every graph node in graph order with only
`occurrences` excluded, including complete nominal bodies and optional owner,
declaration, recursion and nominal-preimage fields. The reader SHALL encode
this borrowed graph view through `CheckedPackageIdentityPreimageRefV2` and
`quire_canonical::Encode`, without retaining another body collection.
The wire `CheckedPackageIdentityPreimageV2` SHALL retain only `version`,
`edition`, `profile_selections`, `definition_selections`, `model_selections`,
`required_features` and `dependency_selections`; its lock selections SHALL
equal the corresponding lock members. The reader SHALL refuse an extra immediate selection member as `malformed_wire`
at that member's pointer. In particular, a shipped `identity_projection`,
including an empty array, refuses at
`/identity_preimage/identity_projection` during closed-wire intake, before
package identity or graph validation. There is no old-wire compatibility path. Closed node version, tag and form
checks SHALL use the existing header decoder before hashing, so the published
unsupported-tag and wrong-form vectors retain their first refusal. It shall check each digest's declared domain before its bytes,
each selected domain package by reading the document the
evidence supplies under its `sha256-jcs` digest (see "Model-owned members"
below), every required feature against
the reported `available` capability and the reader-supported feature set,
every node tag, family form and semantic term, reference and dependency
resolution, cycles outside one explicit `recursion_group`, and the total
source map.

For `model_selections`, whole-array uniqueness is checked before any entry's
digest is checked against the document the evidence supplies, so an array
carrying both a repeated entry and an entry whose document is missing or does
not match has one determined outcome
rather than a position-dependent one: the reader shall first refuse, as
`malformed_wire` at `lock.model_selections`, an array that repeats an entry
anywhere in it (identity, digest domain and digest all equal), and
only once no
entry repeats shall it check any entry's digest against the domain package
evidence. An array carrying both a repeated entry and an entry whose digest
no supplied document satisfies therefore always refuses as `malformed_wire`,
never as `stale_dependency`, regardless of which defect appears first.

Every other `model_selections` defect is decided the same way.

The reader shall select the array's single refusal under this total order over
defect classes, so that an array carrying two defects has one determined
outcome rather than a position-dependent one:

1. A repeated entry anywhere in the array refuses as `malformed_wire` at
   `lock.model_selections`.
2. An entry whose `digest_domain` is not `sha256-jcs` refuses as
   `digest_domain_mismatch` at `lock.model_selections`.
3. An entry with an empty `identity` or a `digest` that is not a SHA-256 hex
   digest refuses as `malformed_wire` at `lock.model_selections`.
4. An entry whose selection evidence FR-322 step 1 does not admit (see
   "Model-owned members" below) refuses with that step's code and cause:
   `missing_import`/`missing-selection` at the entry's `digest` when the
   evidence supplies no document under it, `noncanonical_wire` at its `digest`
   with a `document_pointer` and a cause (FR-038-AC-109) when the document holds a number whose exact value its RFC 8785 encoding loses (quire-specification:FR-272), among them one whose text denotes a magnitude past 2^53 (this
   precedes the digest comparison), `stale_dependency`/`content-mismatch` at
   its `digest` when the document reads and its RFC 8785 SHA-256 is another
   digest than the row's, `stale_dependency`/`byte-digest-mismatch` at its
   `digest` when the bytes do not read and their plain SHA-256 is another
   digest than the row's (both carry the two digests; see "Digest mismatches:
   content and byte"), `invalid_model_binding`/`wrong-model-selection` at its
   `identity` when the document names another identity or when the bytes do
   not read and the row's digest is their plain SHA-256, and the first FR-154 declaration refusal
   of the document at the entry. Two entries naming one `identity` but
   different digests are one identity selected twice: the later entry refuses
   `stale_dependency` at its `digest`, whatever documents the evidence
   supplies, before any document is read.

For an operation in a selected domain document, FR-154-AC-14 validates each
parameter's declared `name` before any frame entry reads the operation. A
missing, non-string, empty, syntactically invalid or identity-mismatched name
refuses `invalid_model_binding`/`malformed-declaration` at that selection row.
Repeated parameter identities refuse `invalid_model_binding`/
`conflicting-binding` first, suppressing the name checks of both colliding
parameters. The private intake refusal retains the operation's identity,
origin and `TypeDefinition.operations[k].params[s].name` member path for a
name defect; the public refusal has the row pointer above. For a repeated
identity the private refusal retains the shared parameter identity and later
parameter's `TypeDefinition.operations[k].params[s]` path without an origin.

A selection is bound by `identity` alone (see "Selections bind by identity"
below), and the nominal `model` owner joins `lock.model_selections` by
identity, so the lock must hold at most one row per model identity for that
join to be sound. The classes guarantee it: two rows of one identity are
either a repeat (class 1) or differ in `digest` (class 4), and both refuse.
The guarantee is scoped to one lock: the nominal `Model`-owned node key is
derived from content alone, so two different packages that each select one
document of the same identity derive the same node key for the same content.
That is the intended content-only identity (QSpec STD-145), not a defect.
QSpec's `ModelRef` has no `version` member, so there is no second selection of
one identity "at another version" for the lock to contain: a row that carries
`version` is refused by its shape (FR-038-AC-62), before any class above.

Each class is evaluated over the whole array before the next class is
evaluated over any of it. The refusal an array draws is therefore the code of
the least class it carries a defect of, whatever order the defective entries
appear in. Within one class the reader draws no distinction: every entry of a
class refuses with that class's own code at the one array path, so which entry
of a class is named is not an observable of the contract. An exact repeated
entry is also a same-identity pair; class 1 is evaluated first, so it refuses
as `malformed_wire` and never as the class 4 `stale_dependency` of a pair that
differs in `digest`.

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

A model owner is QSpec's `ModelOwner` (FR-322, FR-322-AC-28; STD-145 made
its identity content-only). Its closed shape is `{kind, identity, node}`:
`kind` is `model`, `identity` the selected domain package's identity and
`node` the IR node identity inside that package, each a nonempty string, with
no `version` member. This one owner shape keys both places QSpec uses it: the
owner of a model-owned nominal preimage (enum declaration, dimension, unit),
and the owner of a `ModelDeclarationNode`, the structural node preimage by
which a model declaration node of a selected domain package is keyed (QSpec
`node-identity-preimage.schema.json`). The reader shall derive every
model-owned node key, in both places, from the owner's three members alone,
so a node key depends only on content, never on the version of the domain
package that declares it, and shall recover a declaring node's owner (step 2
of "Model-owned members") and compare a reference's member type against those
version-free keys. The lock's `model_selections` row carries no `version`
either (QSpec `ModelRef`, "Selections bind by identity"), so the owner's
`identity` joins the row by identity and nothing else. The reader shall
refuse an owner of kind `model` that carries a `version` member, or any member
outside `kind`, `identity` and `node`, as `unknown_member` at that member, as
every closed shape does; this is the closed shape QSpec publishes, not a
reader for an earlier one.

A `model` node carries one of the fifteen business and systems meanings
[FR-040](./FR-040-admit-frame-entries-and-state-clauses.md) names as its
`semantic_form`.
An `expression` node additionally carries the `collection` and `value_read`
forms. A named, source-declared node carries `declaration.qualified_name`, the
package-local declaration path; a `literal` term carries `type`, the node key of
its package type; an `application` term carries `operation` and `result_type`
beside `operator` and `arguments`. The reader shall read each of these as a
declared member, refuse a node or term that omits one where the contract
requires it as `invalid_semantic_graph`, and carry every one of them into the
occurrence-free identity projection derived for package hashing. No second
wire projection can omit or substitute a graph member.

### Canonical encoding of the wire types

The V2 wire types encode to RFC 8785 canonical bytes through the first-party
`quire-canonical` crate, which this repository depends on at `branch = "main"`
as a git dependency, with its source allow-listed in `deny.toml`. The crate
owns the encoder; this repository holds no copy of it, adds no wrapper type
around it and writes no `DEPTH` literal. Every identity digest this repository
computes is computed through `quire-canonical`, and no encoder of this
repository's own remains: the `package_id`, a nominal preimage's digest, an
application and a structural node key, a lowered node's `ir_id`, a lowered
package's bytes and `package_id`, and a selected model document's digest (see
"Every identity digest is computed through quire-canonical"). This requirement
also changes how the reader checks that a document is canonical (see "Canonical
bytes are quire-canonical's bytes"). `CanonicalWriter`, `canonical_envelope_bytes`
and `digest_json` do not exist (FR-038-AC-91), and the v1 model's canonical
objects and output-mapping identity material take the same encoder (FR-016,
FR-034).

`quire-canonical` has two encode paths, and each type takes the one its depth
allows. `#[derive(FixedShape)]` is valid only for a type whose JSON nests to a
depth fixed by the type itself; serde then recurses once per fixed level. A
type with a member whose depth follows the input implements
`quire_canonical::Encode` instead. Every `serde_json::Value` in the wire is
such a member, because an in-memory `Value` nests as deep as it is built (an
admitted package's `body` and `details` nest to a depth the body grammar fixes). The target is that `quire-canonical` provides the iterative
`Encode` for `serde_json::Value`, behind a `serde_json` feature, as the one
owner for this repository, `quire-contract-codegen` and `quire-spec-language`,
and that this repository's `Encode` types delegate their `Value` members to it.
That upstream change is quire-canonical #7, merged as
b4bb97a5fe0a946e9d980e6466c7ecf95c6e62f1. The same change adds a
public non-recursive drop helper for a deep `serde_json::Value`, because
`serde_json`'s own drop recurses and an in-memory `Value` is built at any depth;
the reader drops a deep `Value` through that helper and writes no drop of its own.
`Value` members encode through that `Encode`, and this repository has no walker of
its own: there is no `value_to_vec` in `checked_package/v2/encode.rs`
(FR-038-AC-91). The assignment follows from the types:

| Type | Path | Reason |
|---|---|---|
| `CheckedSemanticId` | `FixedShape` | three strings |
| `CheckedSourceMapEntry` | `FixedShape` | `CheckedNodeId`, `CheckedOccurrenceRole`, an integer and `CheckedSourceRegion`s of `CheckedArtifactRef`s: every member nests to a depth fixed by its type |
| `CheckedCapability` | `FixedShape` | a string and the closed `CheckedCapabilityDisposition` |
| `CheckedPackageLockV2` | `FixedShape` | arrays of `CheckedArtifactRef`, `CheckedSelection`, `CheckedDomainPackageRef`, `CheckedDependencySelection` and strings: no `Value` and no recursive member |
| `CheckedPackageIdentityPreimageV2` | `Encode` | seven fixed-depth selection members; no node bodies |
| `CheckedPackageIdentityPreimageRefV2` | `Encode` | borrowed graph nodes supply the derived hash projection; node bodies delegate to iterative `Value` encoding |
| `CheckedSemanticGraphV2` | `Encode` | every `CheckedSemanticNodeV2` has a `body` that is a `Value` |
| `CheckedDiagnosticsV2` | `Encode` | every `CheckedDiagnosticV2` carries `details: Vec<Value>` |
| the application node preimage (a node's body and its recursion group's members) | `Encode` | the body is a `Value` |
| `LoweredNodePreimage` | `Encode` | it holds the lowered node, whose `body` is a `Value` |
| `ContractPackagePreimage` | `Encode` | it holds every lowered node and dependency node |
| `NominalIdentityPreimage` | `FixedShape` | four closed preimage structs of strings, `CheckedRational`s and `DimensionTerm`s, its owner a `NominalOwner` |
| the structural node preimage of a model declaration node | `FixedShape` | a fixed body (`aggregate` with no members), `null` for the members the shape declares null, and a `ModelOwner` of three strings |
| the bound identity envelope of FR-023 (`quire.contract.bound-identity/v1`) | `FixedShape` | a profile, a canonical profile, a package digest and the clause identity, declaration digest and expression digest of each binding: depth fixed by the type |

Each fixed-depth type derives `FixedShape`, and so does every type it holds, so
a member that later grows a recursive or `Value` member stops compiling instead
of recursing. The types held by the four `FixedShape` types are the node
identity `CheckedNodeId`, `CheckedOccurrenceRole`, `CheckedSourceRegion`, the
artifact-reference types `CheckedArtifactRef` and `CheckedSourceRef` (the
reference shape of FR-038-AC-46 through FR-038-AC-61), `CheckedSelection`, `CheckedSelectionRole`, `CheckedCapabilityDisposition`,
`CheckedDomainPackageRef` and `CheckedDependencySelection`. The `Encode`
types delegate their fixed-depth members to `Writer::serialize`, which
requires `FixedShape`, so these derive it as well: `CheckedOccurrence`,
`CheckedDeclaration`, `NominalIdentityPreimage` with `EnumDeclarationPreimage`,
`EnumMemberPreimage`, `DimensionPreimage` and `UnitPreimage`, `NominalOwner`,
`DimensionTerm`, `CheckedRational`, `CheckedDiagnosticStage`,
`CheckedDiagnosticCode`, `CheckedDiagnosticCause`, `CheckedSourceRegion` and
the artifact-reference type of `CheckedDiagnosticsV2.catalog`. Types are named
by role where a pending change renames them; the code change for this
requirement is ordered after the one that lands the artifact-reference shape,
so it derives on the types that exist then. The derive is not used with a serde
attribute that replaces a field's serialization (`into`, `serialize_with`,
`with`, `remote`). A type that holds a `Value`, directly or through another
type, never derives `FixedShape`; `CheckedNodeProjectionV2`,
`CheckedSemanticNodeV2` and `CheckedDiagnosticV2` share the `Encode` path of the
types that hold them. A type that has an `Encode` implementation of its own
writes its members as `Writer` events in its own field order, delegating a
fixed-depth member to `Writer::serialize` and a `Value` member to the encode of
a `Value` (`quire-canonical`'s `Encode` for `serde_json::Value`, quire-canonical
#7, merged as b4bb97a5fe0a946e9d980e6466c7ecf95c6e62f1).

This repository holds no walker over a
`Value`: the iterative encode of a `Value`, its number rule (an integer past
2^53 is refused, any other number is its `f64`) and its member order belong to
`quire-canonical`, and FR-038-AC-77 and FR-038-AC-78 are verified against that
`Encode`. Encoding
adds no depth limit of its own: a body nested deeper than the body grammar
admits, built in memory, still encodes. An encode refuses for the byte ceiling and for
any other cause `quire-canonical` reports (a number it has no encoding for, an
object buffer past its fixed bound, an allocation failure, and its other
errors, which it may extend); lowering reports every such refusal as a `failed`
record for `bytes` (see below). The byte
ceiling is the read's byte limit.

The canonical bytes are unchanged for every value whose wire shape is unchanged
and that the reader admitted before this requirement and admits after it.
FR-038-AC-153 adds mandatory owner members to declared and model-owned
structural nodes; their keys, lowered ids and package bytes therefore change.
The owner-free nominal fixture retains its byte-exact pre-owner golden. The
authoritative golden for owner-bearing fixtures comes from QSpec's published
positive packages and `model-member-type-vectors.json`; it is asserted by the
FR-038-AC-176 conformance row, not minted by this reader. For every V2 type above the bytes
`quire-canonical` produces equal the bytes of the type's `serde_json` canonical
form (`to_value` then `to_vec`, members in sorted order) for every in-repo
fixture and for the crafted values of FR-038-AC-74 and FR-038-AC-75 (strings
with non-ASCII and astral characters, and the integers 9007199254740992 and
-9007199254740992), and `package_id` is the SHA-256 of exactly those bytes
with no domain label hashed in, so every `package_id` recorded for an unchanged
wire fixture still recomputes. The same holds for every node key and for the
lowered package when the identity preimage is unchanged: each application and nominal
node key recorded in such a fixture still recomputes, and a lowered node's
`ir_id` and the lowered package's bytes and `package_id` are the ones the
lowering produced before the move (FR-038-AC-89). A structural node whose key
changes under this requirement is checked against a test-side derivation oracle
that constructs QSL FR-092's preimage and hashes its canonical bytes; its lowered
`ir_id` is checked from a separately constructed lowered-node preimage. An
application whose referenced structural key changes is likewise checked from
its test-side application preimage. Those
derived expectations are not an authoritative external golden. This statement is about V2
only: the v1 model's canonical objects
change spelling (FR-016) and so do their digests.

### Every identity digest is computed through quire-canonical

Each digest below is the SHA-256 of `quire-canonical`'s bytes for its preimage,
computed by `quire-canonical`, and by no function of this repository's own that
serializes through `serde_json`:

- `package_id`, over `identity_preimage`;
- a nominal node key, over `NominalIdentityPreimage`, and an application node
  key, over the application node preimage, and a structural node key, over a
  model declaration node's structural preimage;
- a lowered node's `ir_id`, over `LoweredNodePreimage`, and the lowered
  package's bytes and `package_id`, over `ContractPackagePreimage`;
- the order of a dimension's terms, which is the order of each term's
  canonical bytes, not of its `serde_json::to_vec` bytes;
- a selected model document's digest, over the document read through
  `quire_canonical::read` (see "A model document is a value inside a supplied
  document").

The assignment table above says which path each preimage takes: the preimages
whose depth follows their input (the application, lowered node and lowered
package preimages) implement `Encode`, and the fixed-depth ones (the nominal and
structural preimages and the bound identity envelope) derive `FixedShape`. No
digest here hashes a domain label ahead of the bytes: each is the SHA-256 of the
canonical bytes alone, as before.

Every encode runs under a byte ceiling: the reader's byte limit, `limits.bytes`,
for each digest the reader computes while reading, and for the lowering of an
admitted package the byte limit it was read under, which the admitted package
retains. An encode refusal while reading refuses the document with the code and
location its check already uses for a preimage that cannot be encoded:
`malformed_wire` at `/identity_preimage` for `package_id`, and
`invalid_semantic_graph` at the node for a node key. A lowered node whose
preimage's canonical bytes exceed the retained limit is reported `failed`, and a
lowered package whose bytes exceed it is reported `failed` for every requested
record and carries no package bytes or id; the `expect` that the package encode
makes today is replaced by that outcome, so lowering never panics on an encode
refusal (FR-038-AC-95). A ceiling equal to the canonical length admits; one byte
under it refuses.

`CheckedPackageV2::lower_independent_records` returns records without an
aggregate `ContractPackage`. For each requested key, its record shall equal
the record from a separate one-key `lower` call under the same profile and
retained byte ceiling. The operation shares graph indexing and component
closure work across its requests, but an aggregate package's byte-ceiling
failure does not replace a one-key result that fits. Each request still bears
its full work charge and reports the same terminal outcome as its one-key
call (FR-038-AC-309).

`NominalIdentityPreimage::digest`, which a
caller may call on a preimage it built, takes the caller's configured byte
limit as an argument and returns the encoder's refusal when the preimage's
canonical bytes exceed it; it takes neither `u64::MAX` nor a fixed cap of its
own (FR-038-AC-90).

A `failed` record names the limit it failed. The record carries the limit kind
beside `limit` and `consumed`, as the closed `CheckedPackageLimit` of an
`incomplete` outcome (FR-038-AC-3) already names it. For `work`, `limit` is the
caller's work ceiling and `consumed` the counter at the failed charge. For
`bytes`, `limit` is the retained byte limit. When the refusal is
`quire-canonical`'s canonical-bytes limit (`LimitKind::CanonicalBytes`),
`consumed` is that refusal's `required`: the length of the canonical output
written so far, including the bytes being written, when the encode refused. It is
greater than `limit` and at most the full canonical length of the value. It can
be less than the full length (a refusal inside the encoding), and is equal to it
at a ceiling one byte under the full length, where the closing of the outermost
object is counted in one final step, so there `required` is also `limit + 1`.
For every other encoder refusal (an object buffer past the encoder's fixed object
bound, an allocation failure, a number `quire-canonical` has no encoding for),
`consumed` is `limit + 1`, the smallest count that exceeds `limit`, saturating at
`u64::MAX`. Every encoder
refusal during lowering is therefore a `failed` record for `bytes`, a total
mapping that adds no record kind, returns no error beside the records and never
panics. The other refusals are unreachable for an admitted package at the default
limits: the reader refuses a number past 2^53 (FR-038-AC-79, FR-038-AC-93), so an
admitted body holds none; the object bound is `u32::MAX` bytes, so it needs an
object buffer over 4 GiB, which a retained limit of `u32::MAX` or less excludes
and the default of 1 MiB does; an allocation failure is a property of the host,
not of the package. A byte-ceiling failure is never reported as a `work` failure.
A `failed` record has `consumed` above `limit`, with one exception: a retained
limit of `u64::MAX` has no greater count, no encoding that fits in memory exceeds
it, so no canonical-bytes refusal is reachable there, and a refusal of another
cause records `consumed` equal to `limit`, `u64::MAX`.

A lowered node over the ceiling fails only its own request: that record is
`failed` for `bytes`, and the sibling records are unchanged, meaning they are the
records the same call returns with the failed node's request removed. A lowered
package over the ceiling is the one exception to sibling independence
(FR-035 Behavior, FR-035-AC-3, FR-038-AC-6): every requested record is `failed`
for `bytes` with the retained `limit` and the package's `consumed`, because no
package exists to carry a lowered node, which overrides the disposition each
record would have had alone; the lowering returns no lowered node, no dependency
node, no package bytes and no id, and no requested record is left `lowered` in a
package that was not produced. The independence of per-node outcomes
(`invalid_input`, `unsupported`, `requires_bound`, a work or byte `failed` of one
node) is unchanged.

### Canonical bytes are quire-canonical's bytes

This requirement changes what the reader accepts as canonical bytes, and gives
each input it changes one code. A document is canonical when its bytes equal `quire-canonical`'s bytes for
the value read, and the reader makes that check through `quire-canonical`,
without recursing over the document, in the place it checks canonical bytes
today: after the strict syntax and duplicate-member checks and before the
closed-schema decode, the header check, the `package_id` recomputation and every
grammar check. A document that is not canonical in that sense, or that
`quire-canonical` refuses to encode, refuses `noncanonical_wire` with no
pointer, the code and form of every other refusal about the byte stream and the
one the QSpec FR-322 refusal vocabulary names for it. Three inputs are
decided differently as a result:

- An integer whose magnitude exceeds 2^53 anywhere in the document, such as
  9007199254740993 in a node body, refuses `noncanonical_wire`. The v2 literal
  grammar admits any `i64` or `u64` today and the document is canonical by
  `serde_json`'s spelling, so it is admitted today; exact integers past 2^53
  travel as decimal strings. The integer 9007199254740992 is canonical and is
  not refused for its size.
- A whole float such as `2.0`, whose RFC 8785 spelling is `2`, refuses
  `noncanonical_wire` at intake. It refuses today as a literal grammar defect
  once the document is canonical, so the code it returns changes.
- An object whose member names are ordered by UTF-8 bytes but not by UTF-16
  code units (a member name holding a scalar above U+FFFF beside one in U+E000
  to U+FFFF) refuses `noncanonical_wire`, and the same names in UTF-16 order
  pass the canonical-bytes check; the reverse holds today. Passing that check
  admits nothing by itself: the closed body grammar still decides the document.

The order of checks is: strict syntax and duplicate member; canonical
bytes; closed-schema decode and header; the body grammar, as part of strict wire
validation (merged QSpec FR-322 "Identity and validation" and "Body grammar", FR-322-AC-40:
a body outside the grammar refuses `malformed_wire` before any identity is recomputed,
FR-038-AC-116); `package_id` and node-key recomputation; then the graph checks. The
reader runs the body-grammar check ahead of the `package_id` recomputation, so a body
outside the grammar in a package whose identity is stale refuses at the body, not at
the identity check (FR-038-AC-116).
The `package_id` recomputation therefore runs on a
value every number of which `quire-canonical` encodes, and a more specific
grammar refusal is never decided by an encoder refusal. An encode refusal at the
recomputation is not reachable from a document that passed intake; a caller who
meets one (an in-memory value) receives `quire-canonical`'s error.

### The flat wire

This reader enforces the closed body grammar of merged QSpec FR-322
"Body grammar" (FR-322-AC-39 through FR-322-AC-41; FR-341-AC-6; FR-370-AC-5 and
FR-370-AC-12; QSpec TC-233, TC-303 and TC-427 BG-02), and this section states what
that requires of this reader without copying the grammar. Where the reader and
that text differ, the text is this reader's requirement; a reading of a nested
application in "Operation identity", "Application node keys" and the temporal and
state-clause sections is superseded by it. The
grammar covers every node `body` and every diagnostic `details` term, though a
`state`/`frame` body and a `correspondence`/`abstraction_relation` body are
validated against flat shapes of their own (FR-040, FR-346) and an application
standing in either is refused as below. It is strict wire validation: it is decided over the decoded document, in document
pre-order, after the closed-schema decode and before any identity is recomputed
(the order of checks above).

- An application of any operator class other than `case` nested inside a node
  body or inside another term (an application's `arguments`, an `aggregate`'s
  members, a `binding` value) refuses `malformed_wire` at the nested
  application. This holds for every such class, `temporal`, `temporal_formula`,
  `temporal_fairness`, `quire.op.state.clause` and the ordinary classes alike.
  So do an `aggregate` inside a Group's members and a `binding` as a body root.
  The same meaning written with each composite subterm as its own node reached
  by `reference` is not refused for its shape. FR-038-AC-114.
- A nested `case` application is the one named exception: it refuses
  `ill_typed` with cause `operator-ineligible` at the nested application's
  `operator`, and an application of the `temporal_formula`,
  `temporal_fairness` or `case` class in a `details` term refuses the same way
  at that application's `operator`, as FR-322 "Body grammar" and FR-370-AC-12
  state. An application of any other class in a `details` term is outside the
  grammar and refuses `malformed_wire` at that application. A non-`case`
  application at the body root of a node that its class does not place there
  refuses `ill_typed`/`operator-ineligible` at the node that holds it, never
  `malformed_wire`. FR-038-AC-115.
- When one body or `details` term holds several offending constructs, the
  refusal is the first in document pre-order, outermost first, as FR-322
  states: a `details` term that is a `temporal_formula` application holding a
  nested `case` argument refuses at `.../details/{d}/operator`, and a `details`
  aggregate of a `temporal_formula` application and then a `case` application
  refuses at `.../details/{d}/members/0/operator`. The walk that decides the
  nested `case` therefore does not run ahead of an outer construct in pre-order.
  FR-038-AC-115.
- The walk is iterative: it reads the body in document pre-order on a heap
  stack and never recurses on the call stack, so no refusal and no admission
  depends on a chain of nodes' length, and the preimage of an application node
  (FR-038-AC-88) is built over the grammar's fixed depth without recursion.
  Rewriting the references to a node's own `recursion_group` into
  `group_reference` terms visits each term once and clones no enclosing term.
  FR-038-AC-117.
- The V2 reader has no depth limit and nothing in it caps a caller's value by
  depth; the v1 limits of FR-019 and FR-023 are unchanged. FR-038-AC-117.
- The refusal of a nested `case` is decided in the same document pre-order walk
  as the `malformed_wire` refusals, ahead of every identity check, so that FR-322's
  first-in-pre-order rule holds across both. Merged FR-440 "Reader joins" lists the
  same refusal among the operation step's joins, and the merged texts do not
  reconcile the two stages; this stage is this reader's reading, which keeps one
  pre-order across the body. FR-038-AC-115 and FR-038-AC-116.
- Each of the five `body_grammar_mutations` of QSpec's `adverse.json` refuses
  `malformed_wire`, and the expected-failure list of FR-038-AC-112 holds none of
  them. FR-038-AC-118.

### A model document is a value inside a supplied document

A selected domain package's Semantic IR 2.0.0 document is evidence the caller
supplies under its `sha256-jcs` digest, and the reader recomputes that digest
(FR-322 step 1) through `quire-canonical`: the document is read with
`quire_canonical::read`, which keeps each number's text, and encoded from that
reading, under the reader's byte limit. It is not parsed a second time into a
`serde_json::Value`, so there is one reader for the document and the decision
below is made on the number's text before any rounding to a double. Two
refusals about it are different in kind and are not two forms of one refusal:

- A refusal about the checked package's own byte stream (malformed JSON,
  non-canonical bytes: FR-038-AC-79) is about the document the reader was handed
  as the package. It carries no pointer, because no value of that document is
  the fault: the bytes as a whole are.
- A refusal about a value located inside a supplied domain document is about
  one number of that document (any number whose text denotes a magnitude past
  2^53, however spelled), which is not the package. It carries the RFC 6901
  pointer of that number into the model document.

Before the digest is computed, the reader refuses a model document that holds,
at any depth, a number whose text denotes a value of magnitude greater than 2^53
(9007199254740992), however it is spelled: `9007199254740993`,
`9.007199254740993e15`, `1e20` and an integer spelling past the 64-bit range
are all refused, and `9007199254740992`, `-9007199254740992` and
`9.007199254740992e15` are admitted. The decision is on the text, so a
spelling a double would round to 2^53 is still refused. RFC 8785 has no exact
number past 2^53, so two different documents would otherwise share one digest.
This is this repository's own rule, not one a consumer or the specification
requires of it: IR refuses because two documents that differ in such a number
must not share a digest. It is a reader change: a bound such as an integer
type's `Int[lo, hi]` past 2^53 in a model document, which this reader accepts
exactly today, is refused. The refusal is `noncanonical_wire`, located at the
selection row's `digest` (`/lock/model_selections/<i>/digest`) and carrying a
`document_pointer`, the pointer of the first such number in document order, except
that a number with no finite double is named when the reader reaches it (see the
first-fault rule below) (`/package/count` for a member `count` of the object `package` at the document's
root). It is decided at the place the digest comparison is: after the supplied
document is read and before any digest is computed, so it precedes the
`content-mismatch` and the identity refusals of the same row. The document is
read and encoded under the reader's byte limit: bytes that exceed it return
`incomplete` for `bytes` with that limit and the document's length as consumed
and no pointer, as every byte-limit outcome is (FR-038-AC-26); the `work` limit
charged for the same document is located at the row (FR-038-AC-30), and the two
are different limits with different locations.

The same check refuses a number whose exact value its RFC 8785 encoding loses,
by the requirements of the QSpec native `noncanonical_wire` diagnostic
(quire-specification:FR-271) and its two causes (quire-specification:FR-272):
`inexact-integer` for a number whose text denotes a whole value past 2^53,
however spelled, which is every number FR-038-AC-93 refuses, and
`inexact-number` for any other number that FR-272 defines as inexact; a number
that matches both reports `inexact-integer`. A model document is read once by
`quire_canonical::read`, which keeps each number's text next to its double, and
is digested through `quire-canonical`; because the reader refuses a number whose
text is not the exact spelling of its double's value, two model documents that
differ only in such a number (`0.1000000000000000000001` and `0.1`,
`9007199254740993.5` and `9007199254740994.0`) no longer share one digest: the
first of each pair is refused. A
number past the double range (`1e400`, `-1e400`) is an `inexact-integer`
(FR-038-AC-110; see the QSL ruling below). `quire_canonical::read` refuses such
a number with `ReadError::NumberOutOfRange`, which carries the number's RFC 6901
pointer and source text, and `admit_document` maps it to that refusal, with the
cause `inexact-integer` when the text denotes a whole value past 2^53 and
`inexact-number` otherwise (a literal such as 400 nines followed by `.5`);
every other read failure but the byte limit is a document with no RFC 8785
form, decided under the raw-bytes rule of "Digest mismatches: content and
byte" below (`stale_dependency`/`byte-digest-mismatch` at the row's `digest`
under any digest but the bytes' own plain SHA-256, which is
`invalid_model_binding`/`wrong-model-selection` at the row's `identity`).

When one document carries several faults, the first fault `quire_canonical::read`
returns decides, as QSL's merged FR-056 says (QSL #625). The read checks, in
this order: the byte limit; that the whole input is UTF-8; then one pass in
document order, in which a number past the double range is raised when it is
read, a repeated member name only when its object closes, and any other
malformation (truncation, a lone surrogate escape) at its offset. It stops at the
first fault. So invalid UTF-8 anywhere in the bytes is the first fault even after
an out-of-range number (`[1e400,"<0xFF>"]` refuses `byte-digest-mismatch` with no
pointer), whereas a lone surrogate escape after the number is read after it
(`[1e400,"\ud800"]` names `/0`). An
out-of-range number is named ahead of every earlier inexact number, which is
decided only after a read that finds no fault (`{"b":0.1000000000000000000001,"a":[1e400]}`
and `{"b":9007199254740993,"a":[1e400]}` name `/a/0` as `inexact-integer`). A
repeated name in an object that closes before the out-of-range number decides
first and the bytes refuse `byte-digest-mismatch` (`[{"a":1,"a":2},1e400]`); a
repeated name in the object that still holds the number open is not yet
detected, so the number is named (`{"a":1,"a":2,"n":1e400}` names `/n`); a
truncation after the number is read after it (`[1e400` names `/0`); and `1e-400`,
which the read accepts, decides nothing ahead of a repeated name
(`{"a":1,"a":2,"n":1e-400}` refuses `byte-digest-mismatch`). The "first in
document order" of FR-038-AC-93 and FR-038-AC-110, and of this note's
`document_pointer` sentence, is the first number, among the numbers this
reader decides on their text after a read that succeeds, in document order; a
number with no finite double is named when the read reaches it, as QSL's
FR-056 excepts it. That ordering of the faults is QSL's rule, which this reader
implements through the one read; it is not an IR choice. The `byte-digest-mismatch`
outcome of a raw-path document is the outcome under a digest that is not the
document's raw digest; under its own raw digest QSL's merged FR-056 refuses
unparseable bytes `invalid_model_binding`/`wrong-model-selection`, and so does
this reader (FR-038-AC-216). A number below the double range (`1e-400`) reads as zero with
its text kept, so it is an `inexact-number` through the rule above. The reader decides each number of a model document from
its text, through no `serde_json` value. The package document is a different
read: it is parsed through `serde_json` and its bytes compared with the
canonical bytes of the value read, so the crate's own manifest declares
`serde_json` with the feature `float_roundtrip` (FR-038-AC-111), which makes
that parse of a shortest round-trip text exact whatever other crates in the
build turn on. `CheckedPackageRefusalCause` carries the members
`inexact-integer` and `inexact-number`. The refusal is the one of FR-038-AC-93,
located at the row's `digest` with the `document_pointer` of the first such
number in document order, and carries the cause (FR-038-AC-109 through
FR-038-AC-111). The package's
own byte stream already refuses such a number, because its text is not the
canonical bytes of the value read, and carries neither cause nor pointer
(FR-038-AC-79). The whole-number rule of FR-038-AC-93 is unchanged. QSL
never emits such a number, so this change has no lockstep with QSL. The text a
number is compared with is the one `quire-canonical` writes for it, whose
tie-breaking is the even last digit, as QSL's merged intake rule (FR-056-AC-13
and FR-056-AC-14, quire-spec-language #617, merged as
8244dd2b4a63b312ca36a8f254cdc096590eb9b9) takes it, so the two readers agree on
every number with a finite IEEE 754 double. For a number with no finite double,
such as `1e400` and `-1e400`, QSL's merged FR-056 (AC-2 and TC-145) says it
refuses `noncanonical_wire`/`inexact-integer` (`1e-400`: `inexact-number`) with the
number's `document_pointer`, as QSpec FR-272 (`inexact-integer`, "however
spelled") does, unless an earlier reader fault decides, and names the first fault
`read` returns when faults coexist (QSL #625). `quire-canonical`'s `read` carries
the number's pointer and lexeme and IR maps it (IR-555), so the two readers agree
on these documents too, and on the raw-digest case above (FR-038-AC-216). QSL's rule also checks the members of an invocation or snapshot
document it admits (QSL FR-106-AC-11); this reader reads no invocation or
snapshot document, only the selected model documents and the package document,
so that clause has no counterpart here.

### Artifact references

A definition is named by its identity alone. QSpec's `DefinitionRef` is the
closed shape `{authority, identity}` (FR-322 `lock`; QSpec's Wave B Q9 change dropped a
definition's revision and digest, and STD-150 dropped a source's revision), and this reader's
`CheckedArtifactRef` is exactly that shape: `authority` and `identity`, each a
nonempty string, and no other member. A definition reference carries no
`revision`, no `digest_domain`, no `digest` and no `export`: the content
identity of a package lives in its `package_id`, which the reader recomputes,
and not in the references it holds. The reader reads a definition reference at
each place the wire has one:

- `lock.edition.definition` and each `lock.profile_selections[].definition`,
  the `definition` of a `Selection`, and the same two in `identity_preimage`;
- each entry of `lock.definition_selections`, and the same rows mirrored in
  `identity_preimage.definition_selections`;
- `diagnostics.catalog`;
- each `operation.laws[].definition` of an `application` term;
- each `law_roles` entry of the operation catalog (the catalog's owner,
  `quire-verification-contracts`, publishes them as `{authority, identity}`;
  this reader holds no copy of it).

A raw source is the one reference that keeps a byte digest, because the lock
binds a source document to the exact bytes the producer read. QSpec's
`RawSourceRef` is the closed shape `{authority, identity, digest_domain,
digest}`: `digest_domain` is exactly `quire.source.bytes/v1` and `digest` a
SHA-256 hex digest of the source bytes, and there is no `revision`. This
reader's source reference type is `CheckedSourceRef`. It is the member type of
`lock.sources` and the `source` of every `source_map` region and every
`diagnostics.entries[].loci[]` region. A source row's digest is not part of
`identity_preimage`, so it never enters `package_id` (FR-038-AC-4). Dropping
`revision` from a source reference also changes the canonical bytes of a
lowered `ContractPackage` (FR-035-AC-5), whose source-map entries embed the
region `source`; the amendment changes that output, and FR-035's rows are
re-verified when IR-530 lands.

Both shapes are closed under `deny_unknown_fields`, so there is one refusal
for a reference of the earlier shape and no reader for it. This is a refusal
control, not a compatibility layer: the reader never relabels, drops or
ignores the extra member to fit the current contract. The refusal code depends
on where the reference sits, because the reader decodes two places differently:

- The `lock`, `identity_preimage`, `diagnostics` and `source_map` members are
  decoded with the typed wire decode. A definition reference carrying
  `revision`, `digest_domain`, `digest` or `export`, or a source reference
  carrying `revision` or `export`, refuses as `unknown_member` at the first such
  member in document order. A member that is absent or of the wrong kind
  refuses as `malformed_wire`, at the reference when absent and at the value
  otherwise. All of this precedes every check below.
- An `operation.laws[].definition` is inside an application node's `operation`,
  which the reader decodes as its own closed shape, so an extra, absent or
  wrong-kind member there refuses as `invalid_semantic_graph`, as FR-038-AC-36
  fixes for the other closed shapes in a node body, and not as `unknown_member`
  or `malformed_wire`.

After the decode, the checks run in this order. A definition reference at a
`lock` or `diagnostics.catalog` site with an empty `authority` or `identity`
refuses as `malformed_wire` at that member. A `lock.sources` row is checked on
its own: a `digest_domain` other than `quire.source.bytes/v1` refuses as
`digest_domain_mismatch` at `digest_domain` first, so a row with another domain
and also an empty member or non-hex digest refuses for the domain; then an empty
`authority` or `identity`, or a `digest` that is not 64 lowercase hex digits,
refuses as `malformed_wire` at that member. A `source_map` region or
`diagnostics.entries[].loci[]` `source` is not checked as a row: it must equal a
`lock.sources` row, and one that does not refuses as `invalid_source_map`
whatever member differs. The row checks apply to `lock.sources` only, because a
region that equals a lock row is already a checked row.

Two references are equal when `authority` and `identity` are equal, and the
reader compares nothing else about a definition. The joins read that way:

- A source or definition nominal owner joins `lock.sources` or
  `lock.definition_selections` by `authority` and `identity`, as before.
- The `definition` of an `operation` law of a value role (`integer_division`,
  `ieee_profile`, `text_profile`) is admitted only when its pair is one of the
  role's catalogued definitions and is a row of `lock.definition_selections`:
  a pair the role does not catalogue refuses `invalid_package` with cause
  `operation-law-mismatch`, and a catalogued pair the lock does not select
  refuses `operation-law-unselected`, each at that law's `definition`.
- The `definition` of a law of a profile role (`temporal_profile`,
  `protocol_profile`) is admitted only when its pair is the pair of the lock's
  `profile_selections` row of that role; any other pair refuses
  `operation-law-unselected` at that law's `definition`.
- A `source_map` region or `diagnostics.entries[].loci[]` `source` is admitted
  only when it equals a `lock.sources` row in all four members; any other
  refuses `invalid_source_map` at that `source`.
- `lock` and `identity_preimage` carry equal definition rows, and a pair that
  differs refuses `stale_dependency` at the first differing value, as every
  lock mirror does.

The reader reads the catalog's `law_roles` entries as `{authority, identity}`
and matches by those two members. The production catalog is a build-time input
and not a caller's bytes, so an entry of another shape is never a package
refusal. The catalog read is a function over supplied bytes that returns an
error naming the unreadable entry, so that a test can supply bytes in which an
entry carries `revision`; the production catalog is read through it once, and
only that one-time read of the build's own bytes may stop the process
(FR-038-AC-58).

### Selections bind by identity

QSpec's `ModelRef` is the closed shape `{identity, digest_domain, digest}` and
its `DependencySelection` the closed shape `{identity, package_id}` (FR-322
`lock` and `identity_preimage`; QSpec `schema.json`), and this reader's
`CheckedDomainPackageRef` and `CheckedDependencySelection` are exactly those
shapes. Neither carries a `version`: the content identity of what a selection
names lives in its `digest` or `package_id`, which the reader checks against
the supplied evidence, and not in a version label the reader could compare with
nothing authoritative. The same rows are mirrored in `identity_preimage`, so
the version member is absent from the bytes that enter `package_id` as well.
QSpec's `ModelOwner`, `DefinitionRef` and `RawSourceRef` already carry none
(FR-038-AC-45, "Artifact references").

A selection binds to its evidence, to the owner that names it and to the rest
of the lock by `identity`, with the content digest as the only other
comparison:

- A `model_selections` row binds the document the evidence supplies under its
  `digest` when that document's own `package` identity equals the row's
  `identity`. Step 1 reads the document's `package.identity` alone:
  `package.version` is neither required nor read, so a document with no
  `package.version`, or a non-string one, admits when its identity and digest
  match. A model
  owner, and the owner of a model declaration node, joins the row whose
  `identity` equals the owner's `identity`.
- A `dependency_selections` entry binds the package the evidence supplies under
  its `identity` when that package's recomputed `package_id` equals the
  entry's. No version is compared.
- Two selections of one kind are the same selection when their `identity` is
  equal, so the lock holds at most one entry per identity. The two kinds refuse
  a second entry of one identity under different rules. For
  `model_selections` it is class 1 (an exact repeat, `malformed_wire`) or class 4
  (a different digest, `stale_dependency`); the `stale_dependency` code is this
  reader's own choice, kept from FR-038-AC-10, because QSpec's reader assigns no
  code to a repeated model identity (its FR-321 checker refuses it, which is not
  a reader code). For `dependency_selections` it is `invalid_package` with cause
  `conflicting-definition`, which QSpec FR-322 states ("Dependency selections").

No version evidence remains in the lock row: the reader holds no member that
records, mirrors or checks the version of a selected domain package or library.
A package whose `model_selections` row or `dependency_selections` entry carries
a `version` member, in the lock or in the identity preimage, is refused by the
closed shape as `unknown_member` at that member when it otherwise has every
required member (FR-038-AC-62, FR-038-AC-63); a dependency entry that also lacks
`package_id` is `malformed_wire` at the entry ("Dependency selections").
This is a refusal control, not a compatibility layer: the reader never drops,
ignores, relabels or compares the extra member, and keeps no reader for the
earlier shape.

### Model-owned members

A `field` or `operation` member whose declaring node is a model declaration
node resolves through the domain package the lock selects, because that node's
body is `aggregate{[]}` and names no member (QSpec FR-322 "Model-owned
members" and "Reference conformance"; FR-154 for the declaration read). The
reader runs FR-322's four steps:

1. **Selection evidence**, before graph admission, for each `model_selections`
   row in lock order: the digest domain, the document supplied under the
   digest, the check that the document holds no number whose text denotes a magnitude past 2^53, the
   recomputed RFC 8785 SHA-256 and the document's own identity, in
   that order (class 4 above). The reader then reads the
   document's object types, record value types, systems interfaces, integer
   value types and relationships, node by node in ascending IR node identity,
   and refuses with the first declaration refusal at the row
   (`/lock/model_selections/<i>`).
   Within one node its failures report in FR-154's table order (a `typeRef`
   naming no node before a multiplicity with `lower > upper`), and a reference
   to a node refused for its own object id, kind or repeated identity is not
   reported again: that node's own refusal stands for it. A reference outcome
   does not depend on the order the nodes are read in: a `typeRef` naming a
   relationship is `invalid_model_binding`/`malformed-declaration` wherever
   its declaring node sorts. A node with no `identity` is
   `malformed-declaration`, never a `conflicting-binding` with another such
   node.
2. **Owner recovery**, at the application: the declaring node's key is matched
   to one selected declaration's model declaration node key, which the reader
   shall derive under the content-only `ModelOwner` (the
   `ModelOwner` paragraph above), with no domain package version; no match refuses
   `missing_declaration`/`missing-selection` at the member's `declaration`.
3. **Resolution** among the declaring type's exposed effective members, own
   and inherited; none refuses `ill_typed`/`operator-ineligible` at the
   member's `name`, two refuse `ambiguous_declaration`/`ambiguous-name`.
4. **Member type**, compared with the application's `result_type` by node key.

   An operation call's argument types are compared with its parameters' by node
   key, except that an argument whose record conforms to a record-typed
   parameter's record is admitted ([FR-347](./FR-347-admit-conforming-records-as-operation-call-arguments.md)).

#### Selected relationship declarations (IR-661)

The reader SHALL read relationships in the Semantic IR 2.0.0 shape owned by
FCD, rather than applying the owner-nested field and operation identity rule
to them. FCD [FR-095, "Node identities"](https://github.com/agent-ix/filament-core-data/blob/main/spec/functional/FR-095-mint-package-identity-and-provenance.md#node-identities)
places a relationship at
`ix://<org>/<name>/relationship/<Name>-<verb>-<TargetName>` in the selected
package's global `relationship` slot. This reader allocates relationship
identity to FCD FR-095's specific closed-slot rule and the relationship row of
[contracts-v1, Identity minting](https://github.com/agent-ix/filament-core-data/blob/main/docs/semantic-data-system/contracts-v1.md#identity-minting-issue-87).
There is an upstream prose conflict: QSpec FR-154's generic member-identity
paragraph says every member is owner-nested, and FCD contracts-v1's paragraph
before the table includes relationships among members with no slot. Those
paragraphs have not been reconciled with the specific relationship slot. For
this Semantic IR reader, the specific relationship rule governs that case;
the generic owner-nested rule governs fields, operations and parameters. This
is an explicit authority allocation for IR-661, not a claim that the upstream
prose already agrees. The terminal identity segment is an identity, not an end role; the reader SHALL NOT derive a role by splitting it.
Fields and operations retain `<owner type identity>/<member>` identities, and
operation parameters retain `<owner operation identity>/<parameter>` identities.
An owner-nested relationship identity, a foreign-package relationship identity,
or an identity outside the global relationship slot is
`invalid_model_binding`/`malformed-declaration`. A relationship still backs
`relation`/`relationship`, never `model`/`object_type`, and all declarations
share the existing package-wide identity uniqueness check.

FCD [FR-094, "Relationships"](https://github.com/agent-ix/filament-core-data/blob/main/spec/functional/FR-094-lower-relationships-operations-and-clauses.md)
and its [semantic IR schema, relationship end definitions](https://github.com/agent-ix/filament-core-data/blob/main/schema/semantic/v1/semantic-ir.schema.json)
are the producer authority for the whole relationship declaration. The reader
SHALL validate its required `direction`, `category`, `composite` and `origin`
against those authoritative schema definitions, including the origin's
referenced common-schema branch and closed shape. An absent, null,
wrong-typed, unsupported-enum or otherwise schema-invalid non-end member
refuses `invalid_model_binding`/`malformed-declaration` in step 1 at the
selection row with its typed code, cause and RFC 6901 pointer. This public
conversion does not retain FCD origin or declaration identity; the private
intake return's planned retention is governed below. A generated origin has
no source span. The schema is
referenced, never copied into this repository. The reader SHALL
read each end's `type` and `multiplicity`, the required non-empty string
`sourceEnd.role`, and `targetEnd.role` only when present. An absent target role
is valid and exposes no inverse name; a present null, empty or non-string
role, or an absent source role, refuses
`invalid_model_binding`/`malformed-declaration`. The reader SHALL NOT supply a
default role or check a role against the frontend's inverse registry. The
frontend owns agreement with that registry. The source end's type names its
owning type and the target end's type names the target declaration in the same
selected document. A source end naming a different owning type refuses
`invalid_model_binding`/`malformed-declaration`. An end type naming no
declaration refuses
`missing_declaration`/`missing-name`; an end type naming a declaration of the
wrong meaning, including another relationship, refuses
`invalid_model_binding`/`malformed-declaration`. A malformed end or
multiplicity refuses `invalid_model_binding`/`malformed-declaration`; a
well-shaped multiplicity with `lower > upper` refuses
`invalid_model_binding`/`unpreserved-model-meaning`.

A V2 `relationship_end` member SHALL have the existing member shape
`{kind: "relationship_end", declaration, name}`, with `declaration` naming
the relationship's `relation`/`relationship` graph node. Its `ModelOwner`
selects the actual FCD relationship by selected package identity and global
relationship identity; it does not select the source or target object node.
This is the declaring-node rule of merged QSpec
[FR-322, Properties and Model-owned members step 2](https://github.com/agent-ix/quire-specification/blob/main/spec/objects/interfaces/FR-322-checked-package-artifact.md),
together with merged QSL
[FR-094, Model declaration nodes](https://github.com/agent-ix/quire-spec-language/blob/main/spec/functional/FR-094-key-model-owned-reference-population-and-quantity-nodes.md#model-declaration-nodes):
"A `RelationshipRecord` is `relation` / `relationship`" and a checked node
names it "only through a `relationship_end` member". The relationship node
keeps its content-only owner and empty body. Step 2 recovers a valid object's
own declaration when the member names that object node; it does not fail
expected-kind recovery. After that recovery this amendment adds a
relationship-member binding check: a recovered declaration that is not a
relationship refuses `ill_typed`/`operator-ineligible` at
`member.declaration`, before role lookup. This mirrors QSpec FR-322-AC-30's
ineligible cross-kind member case and never redirects lookup to object
members. The earlier owner join remains FR-038-AC-155's: an owner's missing
selection, undeclared node or mismatched node kind is still
`missing_declaration`/`missing-selection` at the owner's node, independently
of this new member check.

At the application's model-member check, the reader SHALL resolve the
member's `name` against this selected relationship's declared roles, byte for
byte. Named-end lookup uses `sourceEnd.role` for the forward source-to-target
end and, only when present, `targetEnd.role` for the inverse target-to-source
end. Two matching end roles refuse `ambiguous_declaration`/`ambiguous-name`.
A requested name matching no declared applicable end role, including an
absent inverse, refuses `missing_declaration`/`missing-name`; it SHALL NOT
select the forward end or create an inverse. This lookup alone
grants no navigation eligibility. Merged QSpec
[FR-152, "Navigation"](https://github.com/agent-ix/quire-specification/blob/main/spec/functional/type-model/FR-152-bind-systems-model-structures.md#navigation)
owns traversal-direction eligibility, destination-multiplicity result
derivation and its typed refusals. The reader SHALL apply that canonical
rule, not maintain a second direction or multiplicity table. Its IR binding
is: source role chooses the source receiver endpoint and target destination;
target role chooses the target receiver endpoint and source destination. A
receiver qualifies through the endpoint's effective view, including an
inherited relationship on a subtype, under `quire.model.complete/v1` and
FR-152; it need not be the exact declared endpoint type. An unrelated receiver
refuses `ill_typed`/`operator-ineligible`. Role lookup precedes those checks,
so an absent inverse remains a missing name even if traversal would also be
ineligible. The navigation application's `result_type` SHALL name the
anonymous type node the canonical derivation yields; a different node refuses
`ill_typed`/`operator-ineligible`. A present inverse never overrides FR-152's
direction eligibility.

For `quire.op.model.reaches`, the same recovered relationship node and named
end select the edge. The static edge owner is the resolved receiver endpoint,
not the relationship graph node: neither operand is a reference to the
relationship declaration. The reader SHALL check the two reference operands
and resolved destination edge against QSpec FR-322 "Reaches over a field"'s
reference-edge admissibility and
[FR-043, Behavior](https://github.com/agent-ix/quire-specification/blob/main/spec/functional/foundation/FR-043-evaluate-finite-graph-relations.md#behavior)'s
static-edge conformance and homogeneous traversal rule, using that endpoint
as the object owner. Thus the endpoint mapping adapts the relationship-end
case of the predicate; it does not apply the field-only equality between
`member.declaration` and the receiver object node. Role, endpoint and
direction resolution follow the binding above; FR-152 remains the authority
for the destination end's typed multiplicity. Merged QSpec
FR-322-AC-57 (RN-18 and RN-20 through RN-27) and FR-152 "Reachability over a
relationship end" admit a destination end of the receiver endpoint's own type
whether it is single-valued (`[0,1]`, `[1,1]`) or finite multi-valued
(`Set`- or `Bag`-shaped, unique or not). The reader SHALL therefore admit
`quire.op.model.reaches` over any such end with the same node shape: operands
0 and 1 stay `Reference` operands checked as above, and the result stays the
catalog's Boolean node. The reader derives no extra node member from the end's
multiplicity and carries no charge information, because the traversal charges
belong to the consumer of the node. The checks run in the order stated by
FR-322-AC-57: the receiver-endpoint conformance check of operand 0
(`arguments/0`) and, for `reaches`, of operand 1 (`arguments/1`), each
conforming to the receiver endpoint type directly or through a supertype;
then direction; then destination multiplicity; then type. An `unbounded`
upper bound or an `ordered` destination end refuses
`unsupported_construct`/`expression-form` at `member.name`, and a destination
end of another type than the receiver endpoint refuses
`ill_typed`/`operator-ineligible` at `member.name`. A package with two
defects therefore reports the earlier check: an unrelated operand 1 over a
wrong-direction, unbounded, ordered or other-type end refuses at `arguments/1`.
The implementation slice moves the operand-1 check ahead of direction to
match this order and compares operand 1 with the receiver endpoint type
instead of the destination type; that changes which refusal wins for a package with more than
one `reaches` defect, and leaves every single-defect outcome unchanged. An
otherwise well-formed edge or pair of operands that fails reference-edge
admissibility refuses `ill_typed`/`operator-ineligible`. The result remains
the catalog's Boolean node, not navigation's destination type. Runtime
traversal remains FR-043's owner responsibility; this reader checks the static
package binding only.

These declaration checks run in step 1 above, including for an otherwise
unused relationship. Their outer pointer is `/lock/model_selections/<i>`.
The reader SHALL retain the selected declaration's authentic
identity and valid origin in the private intake refusal under the retention contract below. A nested relationship
refusal shall retain that relationship's context, not its enclosing object type.
The current public conversion preserves code, cause and the selection-row pointer;
it does not expose declaration identity or origin. A selected-document declaration
identity is not a checked graph-node digest: `locus` keeps its existing separate
meaning. Private metadata retention is PLANNED / UNRUN until implemented and tested.
They keep FR-154's table order and
member-path order within a row: malformed declaration before unresolved end type, duplicate identity
before reversed multiplicity, and unresolved end type before reversed
multiplicity. A reference to a declaration refused for its identity or kind
keeps that declaration's own refusal, independent of node order; it SHALL NOT
be recast as a missing end type. Selected-document parsing and each
relationship node and member read are charged through the existing work meter
at the selection row. Exhaustion of that selection-stage charge returns
`incomplete` there and no partial package. The complete read also charges
operation validation and later graph-body reference edges in its existing
order. A total-reader work limit one below the measured successful total
reports the first unpayable charge at its own stage pointer, which may be a
graph argument target; it does not necessarily report the selection row.

### Declaration-refusal identity and origin retention

When FR-154 model intake refuses a located declaration, the reader SHALL retain
the authentic FCD node identity and the node's valid FCD origin in the private
`SelectionRefusal` returned as `SelectionFailure::Refused` from `admit_selection`.
The selected,
content-authenticated domain package document shall supply the metadata before
its owned document is released. The private refusal shall own its retained values
so that document release does not invalidate them. It shall keep the existing
`refusal` code/cause and optional selection-row `member` unchanged.

The reader shall populate the private refusal with `declaration_identity: Option<Box<str>>` and
`declaration_origin: Option<IntakeDeclarationOrigin>`. `IntakeDeclarationOrigin`
is an IR-owned private typed representation of FCD
[`common.schema.json`, `$defs.origin`](https://github.com/agent-ix/filament-core-data/blob/main/schema/semantic/v1/common.schema.json).
It shall distinguish exactly these forms:

- `Source { source_identity: Box<str>, path: Box<str>, start_line: u64,
  start_column: u64, end_line: Option<u64>, end_column: Option<u64> }`.
- `Generated { generator_identity: Box<str>, generator_version: Box<str>,
  input_identities: Vec<Box<str>> }`.

The reader shall preserve every valid supplied origin member's value and
optional-member presence exactly according to the referenced FCD schema.
Existing selected-document exact-number admission precedes declaration intake and already
refuses integer magnitudes above 2^53. The reader shall introduce no smaller numeric
ceiling, paired-end requirement or span-order rule for retention. It shall retain
generated version text and input identity order/multiplicity exactly, with no source span.
This is preservation of typed member values, not a copy of document formatting.

When an offending node supplies a string identity, the reader shall retain that
exact identity, including one refused for its object-id spelling. When identity
is missing or non-string, the reader shall retain None. The reader SHALL NOT infer or repair identity from
an artifact title, member lookup, selection-row pointer, graph digest or enclosing
owner; FCD supplies no separate artifact-id field. When a field's own declaration
or type reference fails, the reader shall retain that field's identity/origin;
when an operation's own declaration or return type fails, it shall retain that
operation's identity/origin; when a parameter's own declaration or type reference
fails, it shall retain that parameter's identity/origin. An unavailable nested
identity or origin shall remain None, never replaced by the containing operation
or object type. A relationship-end defect shall retain the relationship's own
identity/origin. An unresolved reference shall identify its containing declaration,
not fabricate the absent target. These rules allocate refusal context for existing
intake forms, not admission of new declaration forms.

When origin is missing or malformed, including both branches, partial branches,
unknown members or invalid members, the reader shall retain None and shall not
salvage a partial branch.
A same-identity type group means two or more type declarations sharing one
supplied identity. For every such group,
the reader shall retain that common identity when available. It shall retain an
origin only when every candidate has a valid origin and all those typed origins
are equal in every supplied value and optional-member presence. Any missing or
malformed candidate origin, or any different valid origin, requires None. This
rule applies whether the group's existing refusal is conflicting-binding or
malformed-declaration; the reader shall not select a candidate winner or change
the first refusal. This conservative ambiguity rule is an IR-local refinement
of the multi-node case left unspecified by FR-154's per-node retention rule.

A relationship identity repeated sequentially, including under different owners,
is not a same-identity type group. For that duplicate refusal the reader shall
retain the later actual offending relationship's supplied identity and its own
valid origin, or None for its missing/malformed origin, in the existing
node/member-path order. It shall not substitute the earlier relationship or
containing owner's metadata.

When byte, digest, package-identity or shape admission fails before identifying a
declaration, the reader shall retain None for both fields. Existing distinct
`SelectionFailure::Limit` and `SelectionFailure::InexactNumber` results shall keep
their current values and precedence. The reader shall keep declaration checks in their existing
node/table/member-path order, with the same code/cause/member choice and first-refusal behavior.
It shall acquire metadata from the existing read and charge it to the existing
bounded document/read/work accounting. No second parse, raised ceiling, unbounded
diagnostic collection, default origin or diagnostic-prose parsing is allocated.

The reader shall compare full valid typed origin values in production when
resolving a same-identity type group's origin ambiguity. The owned metadata shall
belong to `SelectionRefusal`, carried by `SelectionFailure::Refused` through
`admit_selection`, and survive that function's release of the original document.
At the existing `SelectionFailure::Refused(refused)` mapping arm in
`checked_package/v2/mod.rs`, the reader shall release that owned metadata by the
end of the arm while preserving its existing `ValidationFailure` projection.
Inspection passes only if the production group comparison determines origin
retention and the actual owned metadata is moved into that refusal and released
at that mapping arm, by explicit ownership teardown or ordinary destruction of
its scoped owner. A read whose result affects neither origin retention nor an
ownership transfer is not consumption; reading a field solely to silence lint
fails the inspection. A blanket `dead_code` allowance or a new observer/public
API also fails it. Its feature-off model-consumer Clippy gate remains mandatory.
The CODE slice shall verify this ownership lifecycle and full typed group
comparison in `model_members.rs` and its private `intake_origin.rs` module.
It shall pass the workspace all-target and feature-off model-consumer Clippy
lanes and the pre-PR and premerge gates. No public metadata consumer or new
API is allocated by this contract.

The retention boundary is the private intake return above. The current conversion
of that return to `ValidationFailure` carries code, path and cause only. The
reader SHALL NOT add declaration metadata to `CheckedPackageRefusal` or the
IR-owned dispatch result under this contract. Existing `locus`, expected-node-key,
`contract_version` and `document_pointer` behavior elsewhere remains unchanged. Public reader metadata observability
is not claimed by this contract.

Open item: a future concrete typed driver, CG or runtime consumer needing these
values would require a separately owned consumer contract and ticket. No such
consumer or public API is allocated here.
Private origin/identity lifetime, refusal-constructor and intake-return assertions
remain CODE gates, not evidence supplied by this specification.

The new relationship-member binding checks run at the operation step, after
the existing catalog shape, operand-count and family checks: owner recovery,
relationship declaration-kind check, role lookup, receiver effective endpoint,
direction, destination multiplicity and operation-specific type comparison.
This is the order to implement for IR-661, not an assertion that today's
reader already performs relationship-end checks. A `reaches` operand-1
conformance or edge-admissibility refusal points to `.../body/arguments/1`
or `.../operation/member/name`, respectively.
The member's `declaration` and `name` refusals point to those members under
`/semantic_graph/nodes/{n}/body/operation/member`; the operand refusal points
to `/semantic_graph/nodes/{n}/body/arguments/0`, and a mismatched result points
to `/semantic_graph/nodes/{n}/body/result_type`. Direction and destination
multiplicity eligibility refuse at `.../operation/member/name`, the value
that selected that traversal. These operation refusals retain the calling
application node's key; selected-document declaration refusals keep the
selection-row pointer; private intake declaration metadata is governed above. Each ancestor or conformance edge followed and each relationship
member visited during resolution is charged through the same selected-row
work meter as other model-member visits; no endpoint or role lookup bypasses
the bounded read. These rules introduce no new limit or budget domain.

Every ancestor edge followed, member visited and redefinition pair compared
in step 3 for an operation, the build of the field tables and the table entries of every field
resolution (a read, a frame entry, an abstraction relation's field entry) ("Typed accessor for a model object type's fields", which replace
the step 3 walk for a field), the conformance walk of `conforming_reference`, and the parse and
read of each selected document (one unit per 1024 document bytes, then one per
node and member) are charged to the `work` limit at the row of the selection
they belong to, so an exhausted limit is `incomplete` with the pointer
`/lock/model_selections/<i>`. Each model declaration node key computed in step
2 is one validation visit at its row.

### Dependency selections

`lock.dependency_selections` and `identity_preimage.dependency_selections` are
the same array of closed `DependencySelection` entries, `{identity,
package_id}`, one per library identity, in strictly ascending UTF-8 byte order
of `identity` (QSpec FR-322, STD-105). `package_id` is the dependency's own
`quire.package.semantic/v2` `{domain, algorithm, digest}`. Every entry's
`package_id` enters the importing package's `package_id`. The reader checks the
lock's array, in this order, each check over the whole array before the next:

1. A `package_id` whose `domain` is not `quire.package.semantic/v2` refuses
   `digest_domain_mismatch` at that entry's `package_id.domain`.
2. An empty `identity`, an `algorithm` other than `sha256` or a `digest` that
   is not a SHA-256 hex digest refuses `malformed_wire` at that member.
3. An entry repeating an earlier entry's `identity` refuses `invalid_package`
   with cause `conflicting-definition` at the repeating entry.
4. An entry whose `identity` is not strictly after its predecessor's in UTF-8
   byte order refuses `invalid_package` with cause `invalid-value` at that
   entry.

An entry that lacks a required member and carries a member outside the closed
shape (a `Selection` or `DefinitionRef` where a `DependencySelection` belongs)
refuses `malformed_wire` at the entry; an entry carrying every required member
and one more, `version` included, refuses `unknown_member` at the extra member
(FR-038-AC-63).

Every entry is also bound to the admitted `CheckedPackageV2` the package
evidence supplies for its `identity` (QSpec FR-322 `dependency_selections`).
After the array checks above and the domain package evidence checks, the reader
takes the entries in lock order and refuses the first that fails: no package
supplied for the `identity` refuses `missing_import` with cause
`missing-selection` at the entry; a package whose own `package_id` is not the
entry's refuses `stale_dependency` with cause `content-mismatch` at the
entry's `package_id.digest`, carrying the entry's `package_id.digest` and the
package's own recomputed one (the reader recomputed it from the package's
identity preimage when it admitted the package, so this is a pinned identity
against a recomputed one). The entry's `identity` and `package_id` are the
only binding: the entry names no version, and none is compared with anything
the consumer supplies. This binding runs before any node is read, so before any
`dependency_reference` check. A consumer therefore supplies every
`dependency_selections` entry's package, under its library identity, with
`insert_dependency_package`.

### Digest mismatches: content and byte

A `stale_dependency` refusal says that what the package selects or pins is not
what the reader found. Its cause says which comparison failed, and follows
what was compared, as merged QSpec FR-272 (its `stale_dependency` row; QSpec
#187) and FR-154 define (this repository holds no copy of either):

- `content-mismatch` is a selected or pinned content identity against the
  identity the reader recomputed from what was supplied. Its contract has
  three comparison sites: a selected model document that reads as strict JSON and whose
  RFC 8785 SHA-256 is another digest than the row's (FR-154 check 3,
  FR-322 step 1), at the row's `digest`; a `dependency_selections`
  entry whose supplied package recomputes to another `package_id` than the
  entry's, at the entry's `package_id.digest`; and this package's declared
  `package_id.digest` against the identity recomputed from its own
  `identity_preimage`, at `/package_id/digest` (QSpec FR-272). All three
  comparisons return `stale_dependency`/`content-mismatch` with the complete
  selected or declared identity as `expected` and the recomputed identity as
  `actual` (IR-238; FR-038-AC-215, AC-217, AC-218).
- `byte-digest-mismatch` is a raw-bytes digest against its declared digest.
  It has one site in this reader: the supplied bytes of a selected model
  document that `quire_canonical::read` refuses (not strict JSON, a repeated
  member name, a lone surrogate escape, and every other fault except the byte
  limit and a number past the double range, which have their own outcomes).
  Such bytes have no RFC 8785 form, so check 3 compares the row's digest with
  the plain SHA-256 of the supplied bytes, at the row's `digest`.
- When those bytes are selected under their own plain SHA-256, check 3 passes
  and no identity can be read for check 4, so the refusal is
  `invalid_model_binding`/`wrong-model-selection` at the row's `identity`, as
  merged QSL FR-056 states for the same case. No declaration is admitted.

This reader reads no source bytes, no `quire.source.bytes/v1` entry and no
replay `byte_provision`, so QSpec's source-bytes and `byte_provision` rules
(FR-323) have no site here. The nearest IR input, a model document or a
dependency package the evidence does not supply, already refuses
`missing_import`/`missing-selection`, at the row's `digest` or the entry: the
pointer names the member of the package that requests it, and the requested
digest record is the row that member belongs to (FR-038-AC-27, AC-37).

Both mismatches carry `expected` and `actual` as `CheckedSemanticId` values:
each has its own domain, the `sha256` algorithm and a lowercase 64-character
hexadecimal digest. Both are present or both absent; every other refusal has
neither. For a readable selected model document, `content-mismatch` carries
the row's selected `sha256-jcs` digest as `expected` and the document's
recomputed RFC 8785 `sha256-jcs` digest as `actual`. For a dependency package,
`content-mismatch` carries the entry's complete
`quire.package.semantic/v2` `package_id` as `expected` and the admitted
package's complete recomputed `package_id` as `actual`. For unreadable model
bytes, `byte-digest-mismatch` carries the row's selected `sha256-jcs` digest
as `expected` and the plain SHA-256 of the supplied bytes under
`raw-artifact-digest` as `actual`. QSL FR-056's raw-byte digest comparison
chooses which *refusal* follows a failed read; even equal digest bytes do not
make the raw identity a `sha256-jcs` identity, admit a declaration or erase
the domain difference. Two typed identities with equal algorithm and digest
but different domains compare unequal (QSpec FR-201), including that
`sha256-jcs`/`raw-artifact-digest` pair.

`CheckedPackageRefusal` gains two public `Option<CheckedSemanticId>` members,
`expected` and `actual`, beside `document_pointer`; the latter is a location
inside a supplied document, whereas the two new members are identities and
neither can be recovered from the package pointer. The existing
`CheckedPackageRefusalCause` remains a `Copy` tag and gains `ContentMismatch`.
This is an IR public-layout choice, not QSpec's cause representation: QSpec
FR-272's cause identity is the tuple (`code`, cause tag, required payload).
For either mismatch, its required payload is the ordered pair of complete
`expected` and `actual` identities. A consumer projecting an IR refusal into
that QSpec cause key SHALL include the pair and compare each identity's
domain, algorithm and digest; comparing the `cause` tag alone compares only
tags. Two mismatch refusals with the same code and tag but different pairs
have different cause keys. The whole `CheckedPackageRefusal` value retains
its derived componentwise equality, including the pair and its location;
location is not part of the QSpec cause key. A missing member of the required
pair is not a valid mismatch refusal and is never defaulted.

This is a break in the crate's public API and behaviour, with no
compatibility layer: `CheckedPackageRefusal` gains two public members (an
external struct literal stops compiling, as with `document_pointer`),
`CheckedPackageRefusalCause` gains a member (an exhaustive match stops
compiling), a selected document that reads and a dependency entry that
recomputes differently change from `byte-digest-mismatch` to
`content-mismatch`, and unparseable bytes under their own plain SHA-256 change
from `stale_dependency` to `invalid_model_binding`. The refusal is a Rust
value this crate does not serialize, so no wire form changes.

### Local protocol references

The V2 reader SHALL admit `node_tag: "protocol"` with
`semantic_form: "protocol_reference"` as `ProtocolForm::ProtocolReference`.
This carrier has one or more occurrences, all of role `expression`, no
declaration, owner, nominal identity preimage or recursion group, and a closed body
`{term: "reference", target: NodeRef}`. The body target is an exact typed
`quire.checked-semantic-node/v1` key in this package's own
`semantic_graph.nodes`. The carrier's `dependencies` SHALL contain exactly
that target, and its `semantic_type` SHALL equal the target protocol node's
`semantic_type`. The body and target participate in the existing structural
node-key preimage and package identity projection. The carrier has no
separate collection or package selector; a selected dependency cannot satisfy
its local target. A `dependency_reference` body or an extra body member fails
the closed V2 schema.

Every source reference to the same target SHALL share the one carrier node,
with a distinct occurrence ordinal and its own `source_map` row keyed by
(`node_id`, `expression`, `ordinal`) naming that reference's exact source
regions. Occurrences and source-map rows enter neither the structural key
preimage nor the package identity projection. Adding a second source reference
and its source correspondence SHALL preserve the carrier `node_id` and
`package_id`; it SHALL NOT introduce a discriminator, a different identity
preimage or another carrier node with the same key.

The reader SHALL refuse an empty occurrence array or an occurrence of a role
other than `expression` before target resolution. An empty occurrence array
SHALL refuse `invalid_source_map` at the carrier's `/occurrences`, preserving
the general graph occurrence check. A repeated occurrence ordinal SHALL
refuse `invalid_source_map` at the later
`/semantic_graph/nodes/{n}/occurrences/{j}`; a repeated source-map key SHALL
refuse `invalid_source_map` at the later `/source_map/{i}`. Two actual graph
nodes with the same `node_id` SHALL still refuse `invalid_semantic_graph`.
These rules implement the landed QSpec FR-322-AC-66; the repeated-reference
reader change and its tagged native public-reader acceptance controls are implemented
and have passed (IR-785); the aggregate release gate remains separate.

The graph-shape stage SHALL compare the sole dependency with the body target
without resolving it and refuse a mismatch or extra dependency as
`invalid_semantic_graph` at the carrier's `/dependencies`. After graph-shape
and stale-node-key checks, the declaration stage SHALL join carriers in
ascending node-id digest order, before frame and operation checks. A target
absent from the local graph refuses `missing_declaration`/`missing-name` at
`/semantic_graph/nodes/{i}/body/target`, even if a selected dependency holds
that key. A present target whose node tag is not `protocol`, or whose
`semantic_type` differs, refuses `ill_typed`/`operator-ineligible` at that
same pointer. The locus is the carrier's typed node key under this reader's
ordinary node-refusal convention. Generic unresolved-reference validation
SHALL NOT precede this carrier join.

### Hybrid node bodies and reader step

The reader SHALL admit QSpec FR-481's `hybrid`/`hybrid_model`,
`hybrid`/`hybrid_mode` and `hybrid`/`hybrid_transition` nodes, and a hybrid
claim as a `temporal`/`temporal_clause` node whose `over` parameter is typed
by a hybrid model. The published QSpec schema owns their closed wire layouts,
including ordered bindings, required and forbidden owner/declaration fields,
occurrence roles, minimum nonempty lists, literal kinds and allowed class and
reset-relation spellings. Strict wire validation SHALL refuse a layout defect
as `malformed_wire` at the defective member before any body step. A model's
continuous variables are `value`/`parameter` nodes typed
`scalar_type`/`rational`; the class literal is producer-attested and is not
reconstructed from the rational rate value. The reader SHALL retain the exact
body references and ordering of variables, modes, initial entries, rates,
transitions and resets. It SHALL NOT synthesize a missing node or infer an
unstated initial entry other than FR-481's single default.

Before this body step, the existing graph-shape check SHALL compare each
hybrid node's `dependencies` with the unique digest-ascending set of its
body's reference targets and refuse a mismatch as `invalid_semantic_graph`
at `/dependencies`. The derived-shape key stage SHALL check each hybrid node
against QSpec FR-481's `quire.structural-node/v1` preimage: the node is its
own `semantic_type`, has no recursion group or nominal preimage, and its key
includes the exact declaration, owner and body (null declaration and absent
owner for a transition). A stale key refuses `invalid_package`/
`stale-node-key` before the hybrid step. The complete nodes enter the
package's derived identity projection, so changing a mode, rate, guard,
reset, variable class or initial entry changes the containing model,
model-typed `over` parameter, claim and package identities along their
reference paths.

The hybrid body step SHALL run after the step-relation step and before the
protocol step, as QSpec FR-322 "Body steps" orders them. It visits models
in ascending node-id digest order. For each model, it SHALL stop at the first
defect in this order, at the member or target the check names:

1. A repeated mode reference or an explicitly written default `initial`
   refuses `invalid_semantic_graph` at the later mode entry or `initial`.
2. Variable targets in variable order must be distinct rational parameters
   at their positional levels; a wrong type, level or repeated name refuses
   `invalid_model_binding`/`malformed-declaration` at the target.
3. Mode targets in body order must be `hybrid_mode` nodes declared under the
   model's qualified name. A mismatch refuses
   `invalid_model_binding`/`malformed-declaration` at the target. For each
   mode in that order, the rate count must equal the variable count; then
   the invariant must be Boolean, each rate rational or one of the model's
   variable nodes, and each transition target a `hybrid_transition` node.
   These checks refuse at `rates`, the invariant or rate target, or the
   transition target with FR-481's respective `invalid_model_binding` or
   `ill_typed` code and cause. Each transition in body order then checks
   target-mode and reset-variable indices, Boolean guard, conflicting reset
   entries, and rational or model-variable reset values, with FR-481's
   `missing_declaration`/`missing-name`, `ill_typed`/
   `non-boolean-root`, `invalid_package`/`conflicting-definition`, or
   `ill_typed`/`type-mismatch` at the named target, guard or entry.
4. Initial entries in body order check the mode index, then Boolean
   condition, refusing `missing_declaration`/`missing-name` or
   `ill_typed`/`non-boolean-root` at the binding.
5. Parameter reads in model terms must resolve to the model's variable
   nodes, outside called function bodies, and called functions must use
   Boolean or numeric parameter and result types. A foreign read refuses
   `missing_declaration`/`missing-name` at the term's binding; a wrong
   function type refuses `ill_typed`/`type-mismatch` at the call.
6. A `clock` variable must have the literal rational `1` as its rate in
   every mode; the first failure in variable order refuses
   `invalid_package`/`invalid-value` at its class literal. A model with no
   `hybrid` variable refuses the same at `variables`. The converse is not
   checked: a variable classed `hybrid` may have literal-one rates.

After all model checks, an unreferenced hybrid mode or transition refuses
`invalid_semantic_graph` at that node in ascending node-id digest order.
Then hybrid claims are visited in ascending node-id digest order. The
temporal step has already checked their temporal placement, profile and
interval fit, including FR-481's timed `[q, *]` only under
`quire.temporal.timed/v1`; a profile mismatch refuses there before the
hybrid step. For each surviving claim, the hybrid step SHALL check, in
order: (7) `model-time` or `model-steps` clock, typed `each` trigger, and
Boolean guard and correctly typed capture initializers; (8) fairness
members naming a mode or transition in this model; (9) parameter reads in
`holds`, guards and captures restricted to the model variables and the
claim's specified activation and capture levels. These checks SHALL use
FR-481's codes, causes and loci: an invalid clock is
`invalid_package`/`invalid-value` at `/body/arguments/1`, a mistyped
trigger/guard/capture is `ill_typed`/`type-mismatch` at that term, and
unresolved fairness or reads are `missing_declaration`/`missing-name` at
the member or formula node. A model or claim defect SHALL win over a
protocol-step defect; an earlier body-step defect SHALL win over a hybrid
defect.

The reader SHALL consume QSpec TC-966's published hybrid vectors from the
selected QSpec checkout. It SHALL admit the positive `Braking` and `Tank`
graphs, including a variable-to-variable rate or reset and a
`hybrid`-classed literal-one rate, and match the strict-wire and rebuilt-key
negative vectors by code, cause and locus. It SHALL neither copy those
artifacts into this repository nor claim the process-provider or result-trace
checks owned by QSpec FR-481 as IR reader evidence.

### Timed-model nodes and reader step

The V2 reader SHALL admit the `timed_model` node family and exactly the
`timed_model`, `time_source`, `clock_place`, `clock_reset`, `clock_guard`,
`time_invariant`, `urgency` and `clock_constraint` forms owned by QSpec
FR-322-AC-120. The published QSpec schema fixes each form's closed wire
shape and occurrence roles; each timed node's outer `semantic_type` is its
own key, while the structural preimage carries a null `semantic_type`, as
FR-322-AC-122 states. The IR reader SHALL
decode the family and form once with the other node kinds. Strict wire
validation SHALL refuse an unknown form or malformed member before identity
or body validation. The canonical writer SHALL emit each complete timed node
once in `semantic_graph.nodes`, without a second body in the wire identity
preimage.

For the published body layout, a `timed_model` root carries the ordered
bindings `model`, `time_dimension`, `source`, `clock_places`, `resets`,
`guards`, `invariants`, `urgencies`, and `constraints`. A tick `time_source`
carries `kind`, `unit`, `model`, `operation`, and `period`; its explicit
`model` reference equals the root model, and its operation references a
`state`/`operation_anchor` whose context is that model. A `clock_place` carries
`model`, `path`, `resolved_path`, and `guards`. The resolved-path table has one
ordered row per path step and retains the declaring type, member identity,
member type, collection kind and bounds; the terminal member type is the
native `Clock` marker. These members are checked package data and are validated
against their path steps before semantic graph admission. For a source-owned
model node, the path's root name comes from its declaration's qualified name;
for a selected model-owned node, it comes from the explicit `owner.node`
identity. The reader compares this identity directly with each resolved row's
declaring type and member identity; it does not infer a type from a display
name or consult source text.

Graph-shape validation SHALL compare each timed node's dependency set with
its body references before the key stage. The structural-key stage SHALL
derive its `quire.structural-node/v1` key from the published QSpec preimage,
including the owner where present, declaration and complete body; the derived
package identity SHALL include that node with only source occurrences
excluded. A stale key or dependency mismatch SHALL refuse at the existing
key or graph-shape stage before the timed-model body step. Reordering only
source occurrences SHALL leave both identities unchanged.

The timed-model body step SHALL run after the hybrid step and before the
protocol step. It SHALL visit timed nodes in ascending retained node-id
digest order and reconstruct each selected model from its one root, source,
places and timed clauses. It SHALL check root presence, ownership, every
target's presence and form, place uniqueness and canonical path, and exact
time quantity and unit validity. It SHALL use QSpec FR-322's paired causes:
`invalid_package`/`missing-timed-model`, `invalid_package`/
`timed-node-reference`, `invalid_package`/`duplicate-clock-place`,
`invalid_package`/`malformed-clock-place`, and `ill_typed`/
`invalid-time-quantity`, with the failing node and member pointer. A timed
defect SHALL win over a protocol or later operation defect; a hybrid or
earlier body-step defect SHALL win over a timed defect. The writer SHALL
preserve the admitted source's node order, exact quantity data and selected
model identity on read-back.
The reader tests SHALL use QSpec's `positive-timed-model-nodes.json` and
`timed-model-node-vectors.json` through `QUIRE_SPECIFICATION_DIR`, without storing a copy
of either fixture in this repository.

### Protocol node bodies and reader step

When a V2 package contains a `protocol` node other than the
`protocol_reference` carrier, the reader SHALL apply the exact body, key,
declaration and dependency rules of QSpec FR-464, FR-465 and FR-466 to its
`protocol_clause`, `role`, `channel`, `queue`, `obligation`, `control` or
`compensation` form. Each body root is a `quire.op.protocol.control`
application with one `protocol_profile` law equal to the selected lock row,
`null` mode, empty leaves, Boolean result and semantic types, and a
`profile_operator` member. The selected finite global protocol profile
defines exactly `protocol`, `role`, `channel`, `queue`, `finish`, `terminal`,
`sequence`, `choice`, `parallel`, `repeat`, `await`, `check`, `send`, `receive`,
`attempt`, `effect`, `event`, `commit`, `fence` and `compensate`. The reader
SHALL compare profile identities exactly: `quire.protocol.finite-global/v1`
and `quire.protocol.complete/v1` are distinct values, never aliases.

The reader SHALL preserve the QSpec layouts as fixed, named argument lists,
including the thirteen clause and thirteen compensation bindings, the
role/channel/queue/finish/terminal declarations and the thirteen control
operators. It SHALL distinguish ordered lists from sets sorted by node-id
digest, sorted case and branch records from ordered captures, optional Groups
of zero or one member from required Groups, and enumerated text and interval
terms from unrestricted terms. A member kind other than `profile_operator`,
or an operator outside the selected profile, refuses `invalid_package`/
`operation-member-mismatch`. A wrong binding name, shape, position or
enumerated spelling refuses `malformed_wire`; a repeated or misordered set or sorted
record refuses `invalid_semantic_graph`. Node keys SHALL continue to derive
from the authored application preimage, including exact declaration paths and
reference targets, rather than from source order or occurrences.

After the state-graph, temporal, abstraction, refinement, terminal,
step-relation, hybrid and timed-model steps and before the operation step,
the reader SHALL run the protocol step
in QSpec FR-464 order: placement;
operator and law; declarations; member nodes; membership; then clauses.
Each `protocol`/`protocol_clause` node SHALL carry the exact local
`owner_model` edge of [FR-064](./FR-064-bind-protocol-clause-to-owner-model.md).
The reader SHALL use that identity-bound edge, rather than the `over`
parameter's type or a step row, to decide which selected model owns the
clause and which clauses contribute refinement classes.
Within a clause it SHALL check references, activation and captures, the
derived causal relation and progress, then slots. Within a slot it SHALL
check forbidden `over` and model-state reads before binder scope and
decision facts. After every protocol node passes these checks, it SHALL
compare each present refinement `class_catalog` with the complete classes
reconstructed from the selected model declarations and admitted local or
directly selected dependency protocol graph. It SHALL compare concrete
before abstract classes, and for each class its node association, ordered
binder types and source-default memory model as QSpec FR-375 and FR-464
specify. The refinement `semantic_type` resolves to its selected concrete
package's canonical `model_root`; each clause's distinct `owner_model` resolves
through the selected declaration and package to its owned object-type node.
The reader SHALL derive the concrete inventory from every admitted clause
owned by that declaration, including clauses absent from `steps`, and SHALL
exclude foreign or unselected clauses. It SHALL derive the abstract inventory
from the selected abstract model or exact selected protocol clause, including
a directly selected dependency clause. It SHALL compare the entire inventory,
not just entries mentioned by refinement rows, and SHALL preserve distinct
`fork` and `join` spellings on one `parallel` node. An absent required catalog,
or a catalog present for a model-only refinement with no protocol or memory
class, refuses `invalid_semantic_graph` at the catalog member. A wrong-kind or
wrong-subject catalog node, changed node association, or wrong ordered binder
signature refuses `invalid_model_binding`/`malformed-declaration` at the
catalog entry. An omitted class refuses `invalid_semantic_graph` at the
affected catalog array; an extra class refuses that code at its entry. A
duplicate or misordered entry refuses earlier under refinement canonical
order. The refinement step's key, order and row-to-catalog checks precede
the protocol step; a protocol-body defect precedes the catalog graph join.
For each selected source `parallel`, the reconstruction SHALL include
`memory drain` with `drain: store-buffer-flush` for every source-default
model it selects among `sc`, `tso` and `ra`, plus `memory flush` for `tso`.
These entries have no declaration node or binder types. The reader SHALL
compare their class spelling, source-default model and drain member against
the admitted source `parallel`; it SHALL NOT derive them from a later request
memory override or silently skip a memory entry because no step row names it.
Where one subject selects multiple source-default models, QSpec TC-310
requires one `memory drain` entry per distinct selected model, keyed by
(`class`, `memory_model`), even when two `parallel` controls select the same
model. A declared class is keyed by its class spelling alone. Entries order
by the UTF-8 bytes of `class`, then by the UTF-8 bytes of `memory_model` for
equal memory-class spellings: drains for `ra`, `sc`, `tso`, followed by the
`tso` flush. A duplicate key or an out-of-order entry refuses
`invalid_semantic_graph` at the later duplicate or first out-of-order entry
in the earlier refinement canonical-order step. A missing or extra distinct
memory class refuses at the later catalog graph join. The reader SHALL
admit the complete ordered multi-default catalog, independently of step rows.

For a protocol clause activated `on each (x: T)`, the reader SHALL associate
the `activate <protocol>` class with the protocol clause node itself, not the
trigger node. Its `binder_types` SHALL contain `[T]` when `T` is not a record;
for a record `T`, it SHALL contain one key per effective field in order. The
effective order walks the source `supertypes` array in authored order, then
the type's own `fields` in authored order. A diamond field appears once at
its first position; a redefining field replaces the hidden field at that
position, with hiding applied once over the whole walk. Changing source
supertype order may therefore change `binder_types`, even when the same
fields and nodes remain. An ambiguous same-name field without a redefinition
refuses `ambiguous_declaration`/`ambiguous-name` at the catalog entry's
`binder_types` pointer
(`/semantic_graph/nodes/{n}/body/class_catalog/{side}/{i}/binder_types`)
when its binder record is requested. A clause activated
`on origin` contributes no `activate` entry. A wrong clause association or
wrong or reordered binder type refuses `invalid_model_binding`/
`malformed-declaration` at the catalog entry; an omitted required entry
refuses `invalid_semantic_graph` at the catalog array, and an extra entry
for `on origin` refuses that code at the extra entry.
The step SHALL use the typed refusal code, cause, pointer
and node locus that QSpec FR-464 through FR-466 state; an earlier body-step
defect SHALL win over a protocol defect, and a
protocol defect SHALL win over an operation-step defect.

For an admitted protocol clause, the reader SHALL derive its activation
relation solely from that clause's `activation` and `requirements` arguments.
It SHALL produce one row per distinct required temporal clause in ascending
node-id digest order: the protocol clause key, `origin` or `each` with its
trigger key, and the required temporal clause key. An empty requirements set
produces no row. Two source `requires temporal` declarations naming the same
clause encode as one canonical `requirements` set entry and produce one row;
two duplicate entries on the wire refuse under the protocol clause's set
rule before any relation is exposed. A temporal clause with a matching clock
or profile that is not required produces no row. A refused package exposes
no relation.

`CheckedPackageV2::protocol_activation_relation(&self)` SHALL expose the
derived relation from an admitted package as a `Vec<CheckedProtocolActivationRow>`.
It SHALL derive the rows from the admitted graph on each call, without adding
a wire member, identity input or separately persisted relation. Rows SHALL be
ordered first by protocol clause node-id digest and then by required temporal
clause node-id digest. `CheckedProtocolActivationRow` SHALL have exactly the
public fields `clause: CheckedNodeId`, `activation:
CheckedProtocolActivation` and `required_temporal: CheckedNodeId`.
`CheckedProtocolActivation` SHALL have exactly the variants `Origin` and
`Each { trigger: CheckedNodeId }`. These values are node keys from the admitted
graph; the method SHALL return an empty vector when no clause has a required
temporal clause. Repeated calls and calls on an equal admitted package SHALL
return equal vectors without changing the package or charging reader work.

### Dependency references

A `{term: "dependency_reference", package, node}` term (QSpec FR-322) names a
declaration of a selected dependency. It is a member of the closed semantic
term vocabulary, decoded once with the other term tags. `package` is a
`quire.package.semantic/v2` SHA-256 identity and `node` a node key; a term that
is not exactly that shape refuses `invalid_semantic_graph` at `package`, at
`node`, or, for a missing or extra member, at the term.

The term is a callee only: argument 0 of a `quire.op.function.call`
application. It enters the application node's key as it stands on the wire, so
a change to either `package` or `node` changes the node id, and it contributes
no entry to the node's `dependencies`; listing its target there refuses
`invalid_semantic_graph`. The `function` operand family of a call is met by a
`dependency_reference` callee.

The reader checks each term at its place in its node's pre-order term walk,
wherever it stands (an application argument, a binding value, an aggregate
member or the body root), in step 7 of the operation stage, after the
application's own identity, law, mode and member checks and before its operand
checks, by these three checks in this order:

1. `package` equal to no `dependency_selections` entry's `package_id` refuses
   `missing_declaration` with cause `missing-selection` at the term's
   `package`.
2. `node` naming no node of the supplied dependency package, or a node that
   carries no `declaration`, refuses `missing_declaration` with cause
   `missing-name` at the term's `node`.
3. A term that is not argument 0 of a `quire.op.function.call`, a `node` that
   is not a `function` node, or a function with a package-dependent signature
   refuses `ill_typed` with cause `operator-ineligible` at the term.

A function's signature is package-independent when no node in the transitive
closure of its signature type nodes carries a `declaration` or a `ModelOwner`.
The signature type roots are the function's own `semantic_type` (its result
type) and the `semantic_type` of each `value`/`parameter` node referenced by
its body's `parameters` binding (QSpec FR-322-AC-86). Other function
dependencies and body references do not enter the signature. The closure
follows each root type node's `dependencies`. A `model` or
`relation` node, and a node with a nominal preimage owned by a domain package,
is a `ModelOwner` carrier. A refusal carries the calling node as its locus.
Each supplied dependency's graph is indexed once when the lock stage binds it,
one unit of `work` per node at the entry's pointer; each lookup is one unit at
the term, and the closure walk charges each parameter binding reference and
each type node visited.

Documented limits, inherited from the reader's operation stage: a call's
`result_type` and `semantic_type` are not compared with the dependency
function's result type (FR-322's `return:0`), and the number and types of a
call's arguments are not compared with the function's parameters. What is
checked is that the callee names a declared `function` node of the supplied
dependency whose signature is package-independent, and where the term stands.

### Closed vocabularies are decoded once

Every closed vocabulary the reader depends on is decoded to an enum where its
wire text is read, and everything past that read matches the enum, exhaustively
and without a catch-all arm, so a member added to a vocabulary is a compile
error at each site that must decide what it means. The vocabularies are the
node family and its forms, selection role, capability disposition, the
semantic term's `term` tag, a literal's `value_kind`, an application's
`operator`, and the operation catalog's law roles, mode kinds, member kinds
and constraint kinds. The bounded-Kani modules under `src/kani/` read Kani's transcript text the same way: the check
kind and the Boolean decoded-value comment are decoded once, where the
transcript is read.

The operation catalog is the owner of its own words, and the reader holds none
of them as a second list. Every application operator class, operation member
kind and constraint kind the catalog declares is a member of the matching enum,
so that reading the catalog never fails on a word the catalog declares. The
catalog's operator classes include `case`, `temporal_formula` and
`temporal_fairness`, its member kinds include `temporal_interval` and
`fairness`, and its constraint kinds include `union_arms`; each is a member of
its enum. The catalog's `union` and `temporal` families, the `arm_body` result
form and the operand family names are catalog strings that the reader indexes
and does not decode into an enum. The `union` family is a member of the
`any_value`, `any_term` and `structural_kind` groups and the `temporal` family
a member of `any_term` only. A word outside a vocabulary still refuses: in a
catalog it is an error of the fallible catalog read of FR-038-AC-58, which
names the unreadable entry or word and never stops the process for bytes it
cannot decode, and as an application `operator` in a package it refuses
`invalid_semantic_graph` at the term. The catalog's `law_roles` entries carry no
`revision`, and the catalog is readable only when FR-038-AC-46 through
FR-038-AC-61 and FR-038-AC-65 are both implemented; the two code changes move
the catalog lock together (IR-530 for the references, IR-503 for the words),
and neither is merged against a lock that the other leaves unreadable.

### Catalog words: admitted, read and catalogued, never evaluated

Decoding a word gives the reader its vocabulary, and admission gives it the
checks of a catalogued entry; neither gives it the meaning. Three of the new
words are operator classes: `case` (the union `case`, QSpec FR-440) and
`temporal_formula` and `temporal_fairness` (the temporal formula and fairness
operators, QSpec FR-370). The catalog carries them on sixteen temporal
identities (fifteen `temporal_formula`, one `temporal_fairness`) and on
`quire.op.control.case`, and the member kinds `temporal_interval` and
`fairness` and the constraint kind `union_arms` appear only on those entries.
The positive fixtures of QSpec's checked-package V2 proposal are the
conformance bar of this reader: `positive-all-families.json` carries a temporal
clause over `temporal`/`formula` nodes applying `quire.op.temporal.holds` and
`quire.op.temporal.eventually` with a `temporal_interval` member, and
`positive-union-nodes.json` carries `quire.op.control.case` (the fixtures are
QSpec's and are read from there, never copied into this repository; FR-038-AC-107
reads them). A reader that refuses what those fixtures contain does not conform,
so each word is admitted and read, and none is evaluated:

| Word class | Catalogued on | Disposition |
| --- | --- | --- |
| operator `case` | `quire.op.control.case` (operand `union`, rest `binder`, result `arm_body`, constraint `union_arms`) | admitted at the body root of an `expression`/`case` node; `union_arms` enforced |
| operator `temporal_formula` | the fifteen `quire.op.temporal.*` identities other than `clause` and `fair` (member `temporal_interval` on the eight interval operators, none on the other seven) | admitted at the body root of a `temporal`/`formula` node |
| operator `temporal_fairness` | `quire.op.temporal.fair` (member `fairness`) | admitted at the body root of a `temporal`/`fairness` node |
| member kinds `temporal_interval`, `fairness`; constraint kind `union_arms` | only the entries above | decoded, and each checked as its own rule below |

Each entry is checked as every catalogued entry is: the `operation` object is
read for its closed wire shape, then the identity, the operator class, the laws,
the mode, the member, the leaves and the operands, in the order the section above
fixes, with these entry-specific rules and no `unsupported_construct`:

- **No law, no mode, no leaves.** None of the sixteen temporal entries nor
  `quire.op.control.case` catalogues a law, a mode or a leaf source, so a law, a
  mode or a leaf on one refuses as it does on any entry that catalogues none.
- **The `temporal_interval` member** (QSpec FR-370). On each of the eight
  interval operators (`eventually`, `always`, `once`, `historically`, `until`,
  `release`, `since`, `triggered`) the member is `{kind: temporal_interval,
  interval}`, as QSpec's V2 schema binds it, with `interval` the closed object
  `{lower, upper}` (an interval operator), `{lower, upper: null}` (a
  lower-bounded operator) or `null` (an unbounded operator), or the timed form of the next member rule; a `null` member is
  not an interval and refuses `operation-member-mismatch` at the application. In
  the integer form each of `lower` and
  `upper` is a non-negative integer string, matching `^(0|[1-9][0-9]*)$` exactly:
  `"0"`, `"3"` and `"10"` match; `"-1"`, `"1.5"`, `"01"`, `"+1"`, `""` and `"3x"` do
  not. QSpec FR-370 (merged) makes the integer-form bounds non-negative
  decimal integer strings in the published schema (`NonNegativeIntegerString`,
  `^(0|[1-9][0-9]*)$`) and states that a bound outside its form's pattern, including a
  negative integer bound such as `{lower: "-1", upper: null}`, refuses
  `invalid_package`/`invalid-value` at that bound's pointer during strict wire
  validation, before any step of FR-370, so it is `invalid-value` under every
  profile and never `operation-member-mismatch` (FR-370-AC-9; the merged
  `adverse.json` has the case `negative-temporal-interval-bound` at
  `/semantic_graph/nodes/29/body/operation/member/interval/lower`). This reader
  refuses every bound outside the pattern, a negative one and a malformed one alike
  (`"1.5"`, `"01"`, `"+1"`, `""`, `"3x"`, a non-string), as
  `invalid_package`/`invalid-value` at schema validation, which in this reader is strict wire validation
  (the flat wire pass, which runs after the strict parse, the canonical-bytes check
  and the decode and before the `package_id` recomputation and every other identity
  check, the temporal step and the operation step; it follows the flat wire's
  nested-application refusals, the spec being silent on the order of those two wire
  checks, an IR reading, and reads the nodes in position order), first in member order (`lower`, then `upper`),
  located at the bound
  (`/semantic_graph/nodes/{n}/body/operation/member/interval/lower` or `.../upper`).
  There is no asymmetry between a negative and a malformed bound: both are failures
  of the schema pattern, refused in that early stage, and neither is the shape
  refusal below. A member of another kind, or an integer-form `interval` that
  holds a third member, refuses `invalid_package` with cause
  `operation-member-mismatch` at `operation.member` (an IR reading: merged text
  states no cause for it, and the published schema's closed interval `oneOf` fails it
  at strict wire validation, which this reader reaches at the operation step instead;
  the timed form's member-set defects are the next member rule's), and a `null` member on an
  interval-capable operator refuses `invalid_package`/`operation-member-mismatch` at
  the application, `/semantic_graph/nodes/{n}/body` (merged FR-370). A closed interval whose
  `lower` is greater than its `upper` refuses `invalid_package` with cause
  `invalid-value` (QSpec FR-370: at the application, here at the application's
  body root, `/semantic_graph/nodes/{n}/body`). The two bounds are compared as
  integers of unbounded size, by sign, then digit count, then digits, never as
  strings and never through a fixed-width integer, so `{9, 10}` admits, `{10, 9}`
  refuses and a pair beyond 2^64 compares as its value does. On the other seven
  `temporal_formula` entries and on `quire.op.temporal.clause`, any member
  refuses `operation-member-mismatch` at `operation.member`.
- **The timed interval form** (QSpec FR-370 and FR-370-AC-8, FR-255; implemented,
  FR-038-AC-119 through FR-038-AC-122). A fourth shape of `interval` is the closed object `{lower, upper,
  lower_end, upper_end}`, the timed form, which merged FR-370 uses only under
  `quire.temporal.timed/v1`: `lower` and `upper` are the exact rationals
  `{numerator, denominator}` of the published schema's `NonNegativeRational`
  (`numerator` matching `^(0|[1-9][0-9]*)$`, `denominator` matching `^[1-9][0-9]*$`),
  `lower_end` and `upper_end` each `closed` or `open`, and there is no `null` upper
  bound. The reader tells the forms apart by the member set of `interval`: exactly
  `lower` and `upper` is the integer form, exactly `lower`, `upper`, `lower_end` and
  `upper_end` the timed form; each bound is judged against its own form's pattern,
  and an interval with any other member set has no form, its bounds judged against
  the integer pattern (an IR reading, as today's reader judges them). A timed-form
  interval is read in these stages, in this order across the package:
  1. **Bound pattern, in strict wire validation** (the flat wire pass, before the
     `package_id` recomputation and every other identity check). Merged FR-370 refuses a bound
     outside its form's pattern `invalid_package`/`invalid-value` "during strict wire
     validation, before any step of this requirement runs"; here that is `numerator`
     negative or malformed (`"-1"`, `"01"`), `denominator` zero, negative or
     malformed (`"0"`, `"-2"`), and a bound that is not the closed object of those
     two members or is a JSON integer, and, in the four-member form, `null` in place
     of `upper`, first in member order, `lower` then `upper`, under every profile. A
     negative bound is merged FR-370-AC-9's case; the others are the same rule applied
     to the rational pattern, and `null` in the upper position is an IR reading
     that follows from the member-set rule above. The locus is the bound,
     `/semantic_graph/nodes/{n}/body/operation/member/interval/lower` or `.../upper`:
     an IR reading of "that bound's pointer" (a schema validator would name
     `.../lower/numerator`, and this crate's rational check of a nominal preimage
     names `.../numerator` today).
  2. **Lowest terms, a graph check.** A bound that matches the pattern and is not in
     lowest terms (`{numerator: "2", denominator: "4"}`) refuses
     `invalid_semantic_graph`, as every non-normalized rational does (merged FR-370
     and FR-322-AC-10). It runs after strict wire validation of every node and after the
     `package_id` and node-key recomputation, as the graph checks do, as the last of
     the graph checks of the nodes that run before the frame step, after the application
     dependency join and every other `invalid_semantic_graph` check of the nodes
     those checks make, and ahead of the frame and state-clause step and the
     temporal step; it runs before the model-selection owners step (the derivation
     of each selected model declaration's node key, which can refuse
     `invalid_semantic_graph` at `/lock/model_selections/{i}`), an IR reading, so a
     package with a non-reduced bound and such a selection defect refuses at the bound, at the first such bound in ascending
     `node_id` digest order and
     then member order, located at the bound (the same pointer as stage 1). So a
     package with a non-reduced bound in one node and another defect of those earlier
     graph checks in another node refuses at the other defect, while the frame
     step's own `invalid_semantic_graph` refusals (a frame array out of canonical
     order, at the frame body, FR-038-AC-14) are later and lose to the bound. So a
     pattern failure in any node, in any bound, is refused `invalid-value` ahead of a
     non-reduced bound in any node; the stage, the position among the graph checks and
     the locus are an IR reading, merged text stating only the code. The GCD work is charged to the work limit at the node's
     body, as the reduced-rational check of a nominal preimage member charges its
     own, so a package over the work limit returns `incomplete` naming it
     (FR-038-AC-3) and not a refusal of the bound.
  3. **Profile fit** (below, the temporal step): under `quire.temporal.timed/v1` the
     timed form admits; under every other profile it refuses
     `operation-member-mismatch` at the operator's application (merged FR-370-AC-8).
  4. **Interval bounds** (the temporal step, after profile fit, with the integer
     form's `lower > upper` refusal): merged FR-370 refuses `lower > upper`, and an
     open end with `lower = upper`, `invalid_package`/`invalid-value` at the
     application, `/semantic_graph/nodes/{n}/body`. The rationals compare exactly, as
     integers of unbounded size by cross-multiplication, never through a float or a
     fixed-width integer, and that work is charged to the work limit as in stage 2.
     A closed interval with equal bounds, `[3, 3]`, is punctual and admits.
  5. **Member-set defects, at the operation step.** A four-member interval whose
     `lower_end` or `upper_end` is neither `closed` nor `open`, and an interval of
     any other member set whose bounds pass the integer pattern (a missing end, a
     fifth member), refuse `invalid_package`/`operation-member-mismatch` at
     `operation.member`. This is an IR reading that diverges from the published
     schema: its closed interval `oneOf` and its `enum [closed, open]` fail each of
     these at strict wire validation, and this reader refuses them later, as it does
     an integer-form interval with a third member. The temporal step skips such an
     interval, so another interval's profile-fit or bounds defect, in this clause or a
     later one, is reported before it.
- **The `fairness` member** (QSpec FR-370). On `quire.op.temporal.fair` the
  member is the closed object `{kind: fairness, fairness_kind, granularity,
  declaration, name}` with `fairness_kind` `weak` or `strong`, `granularity`
  `whole` or `each`, and `declaration` and `name` naming a declaring `model` node
  and an operation as an `operation` `field` member names its declaration. A
  missing member, a member of another kind, an unknown member name or a value
  outside those words refuses `operation-member-mismatch` at `operation.member`.
  A `fairness` member on any other entry refuses the same way.
- **Operands.** The operands are the catalog's: `holds` takes one `boolean`
  operand; `true`, `false` and `fair` none; `not` one `temporal`; `and`, `or` and
  `implies` two `temporal`; `eventually`, `always`, `once` and `historically` one
  `temporal`; `until`, `release`, `since` and `triggered` two; and
  `quire.op.control.case` one `union` operand followed by one rest `binder`
  argument per arm. A `reference` to a `temporal`/`formula` node fits a
  `temporal` operand, as for the clause. Operand count and family refuse
  `ill_typed`/`operator-ineligible` at `arguments` or at the argument, as for
  every entry.
- **The `case` result and `union_arms`.** `quire.op.control.case` is checked
  against QSpec FR-440: its arguments are the scrutinee followed by exactly one
  arm per member of the scrutinee's union, in that union's member declaration
  order, each arm a `binding` named by its member whose value is an `aggregate`
  of a binder aggregate and the arm body; each binder aggregate holds as many
  `reference` terms to `value`/`parameter` nodes as the member's payload arity,
  each binder's `semantic_type` equal to the payload type at its position; and
  the application's `result_type` is the type every arm body has. A `case` whose
  arms are out of member order, omit or repeat a member, carry a binder count
  that differs from the arity, type a binder other than its payload position or
  carry an arm body of another type than the `result_type` (an arm body whose type
  the reader cannot resolve, such as an aggregate or an untyped literal, is treated
  as not of the `result_type`, the conservative IR reading, unlike an operand the
  reader leaves undecided in "Recursive compared types") refuses
  `ill_typed`/`operator-ineligible` located at the `case` node, as QSpec FR-440
  reader join 3 words it. Throughout this section "at the node" is the node's own
  pointer, `/semantic_graph/nodes/{n}`, and "at the application" is its body root,
  `/semantic_graph/nodes/{n}/body`. The `union_arms` constraint is enforced where
  it is reached and never skipped in the constraint match.

Four node forms carry these words and are admitted with them, each a closed
`semantic_form` of its node tag decoded at the form gate: `temporal`/`fairness`,
`composite_type`/`union`, `value`/`union_value` and `expression`/`case`. A union
type node is a source-declared node whose body is an `aggregate` of one `binding`
per member, each value an `aggregate` of `reference` terms to payload type
nodes (a `model`/`record_value_type` declaration node is one,
[FR-347](./FR-347-admit-conforming-records-as-operation-call-arguments.md)); a duplicated member name refuses `invalid_package` with cause
`duplicate-member` and a payload reference to a node that is not a type refuses
`ill_typed`/`operator-ineligible`, each at the union node. A union type or union value
body that is not that shape (an empty union with no member, a binding whose value
is not an `aggregate`, a union value with other than one binding) is malformed and
refuses `invalid_semantic_graph` at the node's `body`, as every other malformed
node body does (an IR reading, pending a QSpec ruling: merged QSpec FR-322 and FR-440 state no refusal for a malformed union body). A union value node's
`semantic_type` is a union type node and its body is one `binding` naming a
member of that union over exactly the member's payload terms, each of the
member's payload type; a binding naming no member, a wrong payload count or a
payload of another type refuses `ill_typed` with cause `type-mismatch` at the
node (QSpec FR-440 reader joins 1 and 2). Neither `duplicate-member` nor
`type-mismatch` is a cause of the reader's refusal types today, and the code
change adds both with the pairings FR-440 states and no other. Merged QSpec FR-322
(FR-322-AC-47) pairs
`ill_typed`/`type-mismatch` (FR-440's union value join) and admits `type-mismatch`
in the V2 `Diagnostic` schema only with `ill_typed`; it likewise admits
`missing-selection` only with `missing_import` or `missing_declaration`, which the
reader's `missing-selection` pairs already follow. The operand family of a
`composite_type`/`union` node is `union`.
`quire.op.structural.eq` and `quire.op.structural.ne` over two values of one union
admit and over values of two different unions refuse `ill_typed`/
`operator-ineligible` by their `same_type` constraint (QSpec FR-440-AC-6). The
leaves of a comparison over a union follow merged QSpec FR-322 "Structural leaf
walk" and FR-322-AC-45 and AC-46 and FR-440: the leaf segments gain `member:<Ident>`,
naming the union member; a
`structural.eq` or `structural.ne` leaf path through a union is `member:<Name>`
followed by `position:i`, `i` the index within that member's ordered payload even
for a single payload, and then the `field:`, `position:` and `inner` segments into
the payload type, with one leaf per member whose payload reaches a `text` type,
members in declaration order and none for a payload-less member. A comparison over a union whose payloads reach
no `text` type admits with `leaves` empty, as for any compared type that reaches
none. A union member key (QSpec FR-441) is
derived by a reader and never carried; this reader derives none, since it selects
no arm. An `expression`/`case` node is keyed by the application-node preimage
like every application node, and the other three forms carry the `node_id` every
node carries.

**The temporal step.** QSpec FR-370 "Reader order" puts a temporal step after
the state step (FR-040) and before any operation refusal, and this reader runs it
there, replacing the earlier placement of these applications in the operation
step and the term walk. Merged QSpec FR-370 and FR-440 state the rules this section cites unless a
sentence says it is an IR reading or pending. Strict wire validation (here the
interval bound-pattern refusal, `flat_wire::check_interval_bounds`) precedes the step. The step reports the first defect
of this order: placement of every temporal application and of every reference to a
`temporal`/`formula` or `temporal`/`fairness` node (1), then every diagnostic
`details` term in entry order and then `details` order, then the clause checks (2).
Placement
(1) is checked over the whole graph first, in ascending `node_id` digest order;
the clause checks (2) then run clause by clause, in ascending `node_id` digest
order of the clause node, and each clause is taken through its four checks (`over`,
fairness resolution, profile fit, interval bounds) in that order before the next
clause is read, as QSpec FR-370 "Reader order" nests them, so a lower-digest
clause's profile-fit defect is reported ahead of a higher-digest clause's `over`
defect:

1. **Placement of every temporal application**, in ascending `node_id` digest
   order, for every node of the graph. The classes are `temporal`,
   `temporal_formula` and `temporal_fairness` (QSpec FR-370), judged by the
   term's `operator`, whatever identity it names. A `temporal`/`temporal_clause`
   node's body root is a `quire.op.temporal.clause` application and that
   application stands nowhere else; a `temporal`/`formula` node's body root is a
   `temporal_formula` application and such an application stands nowhere else; a
   `temporal`/`fairness` node's body root is a `quire.op.temporal.fair`
   application and such an application stands nowhere else (QSpec's V2 schema
   binds each form to its body). So a body that is not that application, as an
   aggregate, a literal or an application of another class, refuses, and so does
   an application of these classes as the body root of a node of another form. An
   application of these classes nested as an element of another application's
   `arguments`, in a `binding` value or in an `aggregate` is no placement
   defect: it is outside the body grammar and refuses `malformed_wire` at the
   nested application at strict wire validation, ahead of this step ("The flat
   wire"; FR-038-AC-114). A `temporal`/`formula` node is referenced only from
   a clause's formula argument or as an operand of a `temporal_formula`
   application, and a `temporal`/`fairness` node only from a clause's fairness
   argument (QSpec FR-370 "Placement and profile fit"); a `reference` to one from
   any other term refuses. Each refuses `ill_typed` with cause
   `operator-ineligible`, located at the node that holds the misplaced
   application or, for a reference, at the referencing node: merged QSpec FR-370
   states that a misplaced reference "is located at the node whose body holds the
   reference, not at the formula or fairness node it names" and that a misplaced
   application is refused `ill_typed`/`operator-ineligible` at the node that holds
   it (FR-370-AC-12).
2. **Each clause** (the clause checks, in this order for one clause):
   - **Profile identity** (first, "Unknown profile" below).
   - **`over`.** Its `over` argument is a `reference` to a node that is among the
     clause node's declared `dependencies` and is a `value`/`parameter` node, else
     `missing_declaration`/`missing-name` for a target that is no declared
     dependency and `invalid_model_binding`/`malformed-declaration` for a declared
     dependency of another tag or form, the pairing this repository's FR-040 gives a
     frame entry's reference (QSpec FR-370 "Reader order" states the check and these
     two pairings).
   - **Fairness resolution.** Merged QSpec FR-370 "Fairness resolution" and
     FR-370-AC-11 resolve a fairness
     member's `name` by FR-322 "Model-owned members" steps 2 and 3 among the
     effective operation members of its `declaration` node, the set QSpec FR-362
     resolves a fairness constraint against (own and inherited, redefinitions
     applied), and state, for each fairness member reached through the clause's
     fairness argument, in this order: (1) the `declaration` target is a `model` node
     of `semantic_form` `object_type`, else `invalid_model_binding`/
     `malformed-declaration` located at the target, with path
     `/semantic_graph/nodes/{fairness node}/body/operation/member/declaration` and the
     `declaration` target's key as locus ("Path and locus" below); (2) the owner is
     recovered, else FR-322 step 2's own refusal, `missing_declaration`/
     `missing-selection` for a node no selected owner matches and `invalid_package`/
     `stale-node-key` for a matched node that is not its own preimage; (3) exactly one
     effective operation member matches `name`: two or more refuse
     `ambiguous_declaration`/`ambiguous-name` and none refuses
     `missing_declaration`/`missing-name`. An operation the declaring type only
     inherits is admitted and resolves to its most derived redefinition: FR-342's
     refusal of an inherited-only operation belongs to an operation anchor and does
     not apply. A `declaration` that names no node at all is a target that is
     no `model`/`object_type` node, so step 1 refuses it
     `invalid_model_binding`/`malformed-declaration` located at the target, as merged
     FR-370 words step 1; it is not `missing-name`.
   - **Profile fit** (below).
   - **Interval bounds**, the `lower > upper` refusal of the member rule above
     and, for the timed form, its `lower > upper` and open-end-with-equal-bounds
     refusals ("The timed interval form"), which run after
     profile fit. The refusal is a property of the member, not of
     a clause, so a formula node no clause reaches is checked too, in a sweep after
     every clause has been read, in ascending `node_id` digest order (an IR reading;
     a formula's interval is an interval all the same). A negative bound is not part of this stage: it is
     a schema-pattern failure, refused in strict wire validation, before any
     temporal step, so `{lower: "-1", upper: null}` under a bounded profile refuses
     `invalid-value` at `lower` and never `operation-member-mismatch`, and
     `{lower: "0", upper: "-2"}` refuses at `upper` as a negative bound, not as
     `lower > upper` (merged FR-370-AC-9 and the stage order of its "Reader order":
     strict wire validation precedes the temporal step).

   The temporal step reads only a well-formed member: an interval member whose
   shape the member rules above refuse (a member of another kind, an interval with
   a third member) and a fairness member that is not the closed object are skipped
   by profile fit and bounds, and the operation step then refuses them
   `invalid_package`/`operation-member-mismatch` at `operation.member`. A bound
   outside the pattern (`{lower: "1.5", upper: "0"}`) never reaches the step: it is
   refused `invalid-value` at the bound in strict wire validation, first in member order. A
   `null` member on an interval-capable operator refuses
   `invalid_package`/`operation-member-mismatch` at the application, merged FR-370.
3. **Profile fit.** The clause's one `temporal_profile` law names the profile.
   The known profiles are the five members of QSpec FR-250's Values table
   (merged text): `quire.temporal.event-position.false-extension/v1`,
   `quire.temporal.fixed-sample.false-extension/v1`,
   `quire.temporal.timestamped-event.finite-window/v1`,
   `quire.temporal.infinite-trace/v1` and `quire.temporal.timed/v1`. A clause whose
   `laws` is not exactly one law of role `temporal_profile` has no profile to check:
   the temporal step skips that clause's profile check and profile fit, and its law
   defect is the operation step's (`operation-law-missing`,
   `operation-law-mismatch`), as merged QSpec FR-370 "Profile check" states. The fit
   of the interval members per profile is merged QSpec FR-370's table and
   FR-370-AC-3, and any other shape, or
   a non-empty fairness argument the profile does not admit, refuses `invalid_package`/
   `operation-member-mismatch` at the application:
   - `quire.temporal.infinite-trace/v1` admits `interval: null`, `{lower, upper:
     null}` and `{lower, upper}`, and any number of fairness references.
   - `quire.temporal.timed/v1` admits `interval: null` (an unbounded operator) and
     the timed form above, and any number of fairness references; `{lower, upper:
     null}` and an integer-form `{lower, upper}` refuse `operation-member-mismatch`
     under it (merged FR-370 states a timed interval has a finite upper bound and
     exact rational bounds). The timed form of merged
     FR-370, `{lower, upper, lower_end, upper_end}`, is read as "The timed interval
     form" in the member rules above states, and under `timed/v1` every timed form
     whose bounds and ends are well formed admits (`[a, b]`, `(a, b]`, `[a, b)` and
     `(a, b)`, with the bound checks of that form); it is admitted under no other
     profile and refuses `operation-member-mismatch` at the operator's application
     there (merged FR-370-AC-8). FR-038-AC-119 through FR-038-AC-122.
   - The three bounded profiles (`event-position.false-extension`,
     `fixed-sample.false-extension`, `timestamped-event.finite-window`) admit
     integer `{lower, upper}` only: a `null` interval or a `{lower, upper: null}` interval
     refuses `invalid_package` with cause `operation-member-mismatch` at the
     operator's application, and a non-empty fairness argument refuses the same
     way at the clause's application.

   **Unknown profile.** Merged QSpec FR-370 "Profile check" and FR-370-AC-10: a clause's `temporal_profile` law is
   checked first among that clause's checks and must name one of the five FR-250
   members, its `identity` equal byte for byte. A law whose `definition` is not one
   refuses `unknown_profile`, located at
   `/semantic_graph/nodes/{n}/body/operation/laws/0/definition` for the clause node
   `n`, with the pairings of merged QSpec FR-272.

   **Cause.** Merged QSpec FR-370: the cause is `wrong-selection-role` when the same
   `DefinitionRef` is the `definition` of a `Selection` of the package's own lock (its
   `edition` or a `profile_selections` row) whose `role` is not `temporal_profile`
   (here the `lock.profile_selections` rows and the `lock.edition` row, whose roles are
   `language`, `edition`, `profile`, `binding_contract`, `protocol_profile`), and
   `unsupported-selection` otherwise. The rule is decidable from the package alone: this reader holds no
   catalog and no identity list beyond FR-250's five members, so an identity the
   package does not select under another role, such as the in-repo fixtures'
   `quire.fixture.temporal-profile/v1`, refuses `unsupported-selection`, and
   `quire.protocol.complete/v1` or `quire.package.composed/v1` refuses
   `wrong-selection-role` only when the package itself selects it in another role.
   A known profile is matched by its FR-250 identity label, an exact string
   (QSpec FR-250 "Identity and comparison"; the label is the key); the
   `authority` of the definition reference takes no part, so a law with a member's
   identity and a different `authority` is still a known profile and later refuses
   `operation-law-unselected` at FR-038-AC-57's join (merged FR-370 matches `identity`
   alone: "its `identity` equals one of FR-250's five values byte for byte"). This check precedes FR-038-AC-57's join of the law to the lock's
   `temporal_profile` row, which still refuses `operation-law-unselected` for a
   known FR-250 member that is not the selected one. An unknown identity is never read as bounded or as unbounded.
   `unknown_profile` is a code of QSpec FR-271's diagnostic code vocabulary and not
   yet a variant of the reader's `CheckedPackageRefusalCode`, and neither
   `unsupported-selection` nor `wrong-selection-role` is a variant of its
   `CheckedPackageRefusalCause` today (measured): the code change adds the code
   `UnknownProfile` and both causes, with these pairings and no other. A formula tree is the set of `temporal`/`formula`
   nodes reachable from the clause's formula argument by operand references.

**Path and locus.** Every refusal of the temporal step carries both a path (an
RFC 6901 pointer) and a locus (a node key), as QSpec FR-047 requires of every
refusal ("an exact locus") and as this repository's own FR-040 pairs them for a
frame entry (the path is on the referencing member, the locus is the declaring node's
key); merged QSpec FR-370 states "located at the target" for a fairness declaration
and "located at the node whose body holds the reference" for a misplaced reference. "At the node" and "at the application" above name
a path (`/semantic_graph/nodes/{n}` and its `/body`); unless a row below says
otherwise, the locus is the key of the node that path is on.

| Refusal | Path | Locus |
| --- | --- | --- |
| placement, misplaced application | `/semantic_graph/nodes/{holder}` | the holder's key |
| placement, misplaced reference | `/semantic_graph/nodes/{referencing node}` | the referencing node's key |
| `unknown_profile` | `/semantic_graph/nodes/{clause}/body/operation/laws/0/definition` | the clause's key |
| `over` not a declared dependency, no node (`missing-name`) | `/semantic_graph/nodes/{clause}/body/arguments/0` | the clause's key when it names no node, otherwise the target's key |
| `over` not a `value`/`parameter` (`malformed-declaration`) | `/semantic_graph/nodes/{clause}/body/arguments/0` | the target's key |
| fairness `declaration` not a `model`/`object_type` node, including a target that names no node (`malformed-declaration`, FR-370 "Fairness resolution" step 1, "located at the target") | `/semantic_graph/nodes/{fairness node}/body/operation/member/declaration` | the `declaration` target's key as named (`member.declaration`) |
| fairness `name` matches no effective operation (`missing-name`) or two or more (`ambiguous-name`) | `/semantic_graph/nodes/{fairness node}/body/operation/member/name` | the `declaration` target's key |
| fairness declaring node's owner not recovered | `/semantic_graph/nodes/{fairness node}/body/operation/member/declaration` | the fairness node's key (the calling node, as FR-038 states for a model-owner refusal) |
| a `details` term references a `temporal`/`formula`, `temporal`/`fairness` or `expression`/`case` node (merged QSpec FR-370 and FR-440, #182; a union or union value node reference is admitted) | `/diagnostics/entries/{e}/details/{d}` | the holder is a diagnostic entry, not a node: no node key; the entry's pointer is the locus |
| an application of the `temporal_formula`, `temporal_fairness` or `case` class in a `details` term (merged QSpec FR-370 and FR-322 "Body grammar", #182 and #183); an application of any other class there is `malformed_wire` at that application ("The flat wire") | `/diagnostics/entries/{e}/details/{d}/operator` (and below for a nested one) | no node key; the entry's pointer is the locus |
| an application of any class other than `case` nested in a node body or another term, an `aggregate` inside a Group's members, a `binding` as a body root (`malformed_wire`, strict wire validation, merged QSpec FR-322 "Body grammar", FR-341-AC-6, FR-370-AC-5) | the nested value, `/semantic_graph/nodes/{holder}/body/...` | the holder's key |
| an `expression` node whose `semantic_form` contradicts its root application's operator class (`invalid_semantic_graph`) | `/semantic_graph/nodes/{n}/body` | the node's key |
| a `case` application nested in another term (merged QSpec FR-440 and FR-322 "Body grammar", #182; refused in strict wire validation, `flat_wire`) | `/semantic_graph/nodes/{holder}/body/.../operator` of the nested application | the holder's key |
| a `case` application as the body root of a node that is not an `expression` node (merged FR-440 join 1; the operation step, not the temporal step) | `/semantic_graph/nodes/{holder}` | the holder's key |
| a `null` member on an interval-capable operator (`operation-member-mismatch`, merged FR-370) | `/semantic_graph/nodes/{n}/body` | the node's key |
| profile fit, interval | `/semantic_graph/nodes/{formula node}/body` | the formula node's key |
| profile fit, fairness argument | `/semantic_graph/nodes/{clause}/body` | the clause's key |
| `lower > upper` | `/semantic_graph/nodes/{formula node}/body` | the formula node's key |
| a bound outside the non-negative integer pattern, negative or malformed (`invalid-value`, strict wire validation, first in member order) | `/semantic_graph/nodes/{formula node}/body/operation/member/interval/lower` or `.../upper` | the formula node's key |
| a timed bound outside the rational pattern (`invalid-value`, strict wire validation, first in member order; the pointer is an IR reading, a schema validator naming `.../lower/numerator`) | `/semantic_graph/nodes/{formula node}/body/operation/member/interval/lower` or `.../upper` | the formula node's key |
| a timed bound not in lowest terms (`invalid_semantic_graph`, graph check after strict wire validation and the identity recomputation; an IR reading of stage and locus) | `/semantic_graph/nodes/{formula node}/body/operation/member/interval/lower` or `.../upper` | the formula node's key |
| a timed interval under a profile other than `timed/v1` (`operation-member-mismatch`) | `/semantic_graph/nodes/{formula node}/body` | the formula node's key |
| a timed-form member-set defect: an end neither `closed` nor `open`, a missing end, a fifth member (`operation-member-mismatch`, operation step; an IR reading that diverges from the published schema) | `/semantic_graph/nodes/{formula node}/body/operation/member` | the formula node's key |
| a timed `lower > upper`, or an open end with equal bounds (`invalid-value`) | `/semantic_graph/nodes/{formula node}/body` | the formula node's key |

The paths AC-97, AC-102, AC-103, AC-104, AC-108 and AC-119 through AC-122 give, and the loci they name,
are the rows of this table. The table follows the loci merged FR-370 states (the
target, the referencing node, the clause node for the law); the other pairings are IR
readings, pending a QSpec ruling.

One row is a deliberate departure from this repository's FR-040, which makes the locus the entry's
declaring node key (the key the entry names, as `frame.rs` carries it): an `over` that
names no node puts the locus on the holder (the clause) rather than on the named key,
because a key that names no node cannot locate a node in the graph. A fairness
`declaration` that names no node is refused at the target as named, as merged FR-370
step 1 says, and uses the named key. Every other row that names a target uses the
target's key, as FR-040 does.

**Merged QSpec text.** QSpec PRs #181, #182, #183 and #184 are merged: FR-322 (the `type-mismatch`,
`missing-selection`, `unsupported-selection` and `wrong-selection-role` pairings, the
`member:<Ident>` leaf segment, "Structural leaf walk", AC-45 through AC-47), FR-370
(interval forms and non-negative bound pattern, profile table, "Profile check",
"Fairness resolution", placement, "Reader order", AC-3, AC-4, AC-8 through AC-12),
FR-440 (case placement, union leaves, AC-7, AC-8) and the V2 schemas. The sentences
above that cite them are no longer pending. What the merged text does NOT state, and
stays an IR reading or pending a QSpec ruling, is listed in the PR description: a
malformed union or union value body; the path and locus pairing of the refusals
beyond the loci quoted above; the refusal of a fairness `declaration` that names no
node is no longer on this list (merged step 1 covers it); the unreached-formula
`lower > upper` sweep; and an arm body of unresolvable type. The recursion-leaf
entry for a record, tuple or union cycle that reaches `text` is merged FR-322
"Structural leaf walk" text (#182), not an IR reading. Merged FR-322 "Body grammar"
(#183, #184) also states the flat wire: an application of any operator class other
than `case` nested inside a node body or another term is `malformed_wire` at the
nested application, at strict wire validation and ahead of every identity check; only
an application at a body root its class does not place it in refuses
`ill_typed`/`operator-ineligible` at that node; and the first offending construct in
document pre-order, outermost first, is reported. "The flat wire" states what that
requires of this reader (FR-038-AC-114 through FR-038-AC-118); this section's placement
and `details` rules are read with it, and a nested non-`case` application is never a
placement or `details` defect here.

Operand family and count and `result_type` stay the operation step's
(FR-370-AC-7), so `holds` over a `reference` to a `temporal`/`formula` node
refuses `ill_typed`/`operator-ineligible` there, after the temporal step, and an
identity the catalog does not hold or an `operator` that differs from the
entry's class still refuses `unknown-operation` or `operation-class-mismatch`
there, never ahead of a placement defect. A `case` application is not temporal.
Merged QSpec FR-440 "Reader joins" 1 and FR-440-AC-7 state that a
`quire.op.control.case` application stands only as the body root of an
`expression`/`case` node and that one at the body root of any other node refuses
`ill_typed`/`operator-ineligible` at the node that holds it. Merged FR-440 checks its
joins, case placement first among them, at FR-322's operation step, in ascending
`node_id` digest order, each node at its place in that order, and this reader follows
it: case placement is NOT part of the temporal step's placement pass, which covers
the three temporal classes only. The order consequence: a defect of the temporal step
(placement, `details`, clause checks) is reported before any operation-step defect,
whatever the digest order, so it is reported before a `case` placement defect; among
operation-step defects the lowest `node_id` digest is reported, and case placement is
the first check of its node's joins. This replaces the earlier IR reading that put
`case` in the one placement pass. Two further outcomes, merged QSpec text (#182 and
#183: FR-322 "Body grammar" and FR-440; the nested case is refused in strict wire
validation (`flat_wire`), where nested applications are refused, ahead of every
identity check (FR-038-AC-116), and not by the operation step): a
`case` application nested inside another term refuses `ill_typed`/`operator-ineligible`
at the nested application's `operator`, FR-322's named exception to `malformed_wire`
(a nested application of any other class is `malformed_wire`, not placement, which
refuses at the holding node only for an application at a body root), where merged
FR-440 words it as not a failure of FR-322's body grammar; and, as merged FR-440
and FR-322 state, an `expression` node whose `semantic_form` contradicts its root
application's operator class (for example an `expression`/`case` node whose body is
not a `case` application, or an `expression` node of another form whose body root is
a `case` application) refuses `invalid_semantic_graph` at the node's `body`, not
`ill_typed`/`operator-ineligible`. A `case` application at the body root of a node
that is not an `expression` node at all is the placement refusal above.

**Diagnostic details.** The `details` references below are checked after the placement of
applications and references and before the clause checks, in entry order and then
`details` order (merged QSpec FR-370 "Reader order", #182). The `details`
applications are not: they are decided at strict wire validation, in the body grammar
walk ahead of every identity check ("The flat wire"):
- a `details` term that REFERENCES a `temporal`/`formula` or `temporal`/`fairness`
  node refuses `ill_typed`/`operator-ineligible` at that entry,
  `/diagnostics/entries/{e}/details/{d}`; so does a reference to an
  `expression`/`case` node (merged FR-370-AC-12 states it with the same code, cause
  and path);
- a `details` term that REFERENCES a `composite_type`/`union` or `value`/`union_value`
  node is an ordinary type or value reference and is ADMITTED;
- an APPLICATION of operator class `temporal_formula`, `temporal_fairness` or `case`
  inside a `details` term refuses `ill_typed`/`operator-ineligible` at that
  application's `operator` (`/diagnostics/entries/{e}/details/{d}/operator`, and
  below it for a nested one, the first in document pre-order, outermost first),
  since a `details` term is no node's body (merged FR-370-AC-12 and FR-322 "Body
  grammar"). A `case` is an application, so it refuses here. An application of any
  other class there, `temporal` among them, is outside the grammar and refuses
  `malformed_wire` at that application, at the same stage (FR-038-AC-115).

**No evaluation.** Nothing in this repository evaluates an application. The
reader admits and checks a catalogued entry by shape, and the lowerer carries an
admitted node as data: it gates a closure by node tag (FR-038-AC-8) and reads no
operator, so a closure holding a `temporal`, `composite_type`, `value` or
`expression` node lowers when the caller's profile supports its tag and returns
`unsupported` naming that tag when it does not. Temporal, fairness and `case`
semantics are evaluated downstream, by contract generation and the runtime, and
an application they cannot evaluate is refused downstream at evaluation time with
a typed unsupported cause of their own, not by this reader (note, not a
requirement of this specification: `quire-contract-codegen`'s generation outcome
`Unsupported` with error code `UnsupportedExpression` is the typed cause that
exists today, and whether it is reached for these operators is not measured
here). IR-549 removed `CheckedPackageRefusalCode::UnsupportedConstruct` and
`CheckedPackageRefusalCause::ExpressionForm` for blanket rejection of temporal
and case applications; admitting those forms does not require evaluating them.
IR-661 separately requires those code/cause variants only for relationship
navigation whose destination has an unbounded upper or `ordered: true`, as
QSpec FR-152 Navigation states. These additions are PLANNED / UNRUN and do
not restore blanket temporal/case rejection. The diagnostics wire vocabulary
`CheckedDiagnosticCode::UnsupportedConstruct` stays, because QSpec's V2 schema
enumerates `unsupported_construct` as a diagnostics `code`.

**IR-549 historical breaking change for consumers, shipped inside its held branch.**
The four new node forms, the then-removed refusal code `UnsupportedConstruct` and the new public
`CheckedPackageRefusalCode` variant `UnknownProfile` are visible to every consumer
of this crate: a consumer that classifies every `CheckedNodeKind` or matches
`CheckedPackageRefusalCode` exhaustively fails to build or test until it handles
them, and an added variant breaks such a match as a removed one does.
`UnsupportedConstruct` exists only on the branch of the held artifact-reference
change (#253), not on a merged `main`, so this change ships inside that branch
before it merges and no merged consumer ever sees the code.

The in-repo `v2_all_families` clause selects the temporal profile
`quire.fixture.temporal-profile/v1` (`temporal_profile_law` in
`tests/it/support/checked_package.rs`), which is not a known profile and would
refuse `unknown_profile`. The code change moves that selection, in the law and in
the lock's `profile_selections` row, to `quire.temporal.event-position.false-extension/v1`,
a bounded profile, which fits the fixture's closed interval `{0, 3}`; the rows that
build on the fixture are re-verified by that change.

`quire.op.temporal.clause` is catalogued with the same rules as every entry.
It is catalogued with operator class `temporal`, one `temporal_profile`
law, no member, no rest and the six fixed operands `reference`, `text`,
`aggregate`, `aggregate`, `aggregate`, `temporal`, in that order. The reader
checks it as it checks every entry: any other argument count refuses
`ill_typed`/`operator-ineligible` at `arguments`; an argument whose family
resolves and does not fit its operand family refuses the same way at that
argument; and a supplied member of any kind, including a `profile_operator`
member, which the entry no longer admits, refuses `invalid_package`/
`operation-member-mismatch`. The six arguments of QSpec FR-370, in order, are:
a `reference` to the `value`/`parameter` node of the clause's `over`
parameter; a `text` literal; three `aggregate` terms; and a `reference` to a
`temporal`/`formula` node. The first is not family-resolved: it matches the
`reference` operand when it is a `reference` term, whatever the type of the
parameter it names, because resolving it through the parameter's
`semantic_type` would give the parameter's own family. The second resolves to
`text`. The three aggregates resolve to no family and are not checked, as every
aggregate argument is today. The sixth resolves to family `temporal` when it is
a `reference` to a `temporal`/`formula` node, and to `boolean` when it
references a Boolean node, which does not fit. A `reference` to a
`temporal`/`formula` node fits the `temporal` operand and every `any_term`
position, and no `boolean` or `any_value` operand position. The `over`
resolution, the fairness members and the profile fit are the temporal step's
(above); the clause's activation and captures aggregates are not checked beyond
their shape here. A
clause whose formula is a `reference` to a `temporal`/`formula` node whose body
root is a `temporal_formula` application is an admitted package, so the clause's
own check passing is seen end to end through a package read as well as at the
operation check of the clause node (FR-038-AC-68, FR-038-AC-96). The in-repo
all-families fixture, already rewritten to the six-operand clause shape, holds a
`temporal`/`formula` node whose body is an empty `aggregate` (`v2_all_families` in
`tests/it/support/checked_package.rs`); that body is not a `temporal_formula`
application, so the temporal step refuses it. The code change makes the formula
node `a2a2` the clause's formula root, an application of `quire.op.temporal.eventually`
with a `temporal_interval` member `{0, 3}`, and adds the formula node `a5a5`, an
application of `quire.op.temporal.holds` over a Boolean literal typed at the added
`scalar_type`/`boolean` node `a6a6`; it also adds the `scalar_type`/`integer` node
`a3a3` and the `value`/`parameter` node `a4a4`, which becomes the clause's `over`
argument in place of its earlier target because the clause check requires a
`value`/`parameter` dependency (FR-038-AC-103). The fixture thus grows by four
nodes beyond the formula node already there, all built from this
crate's own vocabulary and not copied from QSpec, so the rows that build on it
(FR-035, FR-040, FR-344, TC-044, TC-047, TC-048, TC-050, TC-052, TC-053, TC-056
and TC-222) are re-verified by the code change, whose node, edge and occurrence
one-over locations move with the added nodes.

QSpec FR-440 and FR-370 own the encodings; this section states what the reader
checks of them. The evaluation of the union and temporal words is not this
repository's; IR-506 and IR-507 (union) and IR-510 and IR-7 (temporal) are the
tickets that earlier text named for their semantics.

### Parameter and compound-unit nodes, and the application dependency join

A `value` node may carry the `parameter` form and a `scalar_type` node the
`compound_unit` form: the parameter node QSL FR-092 builds for each binder
(a function parameter, a `let` name, a query, `count`, `sum`, `fold` or
`reduce` binder) and the anonymous quantity type QSL FR-094 builds for a
product, quotient or power of units. QSpec FR-322's published form list
does not yet name either; QSL proposes both, and the reader admits them in
exactly the shapes QSL fixes.

A `value`/`parameter` node carries no `declaration`. Its `semantic_type` is
its binder's type: a `scalar_type`, `composite_type` or `bounded_domain`
node other than itself, or a `model`/`record_value_type` declaration node
([FR-347](./FR-347-admit-conforming-records-as-operation-call-arguments.md)). Its body is exactly `aggregate{[binding "name" = a
`text` literal typed at a `scalar_type`/`text` node whose value is an
identifier, binding "level" = an `integer` literal typed at a
`scalar_type`/`integer` node whose value is a canonical non-negative decimal
string]}`, and its `dependencies` is empty. A body of any other shape refuses
as `invalid_semantic_graph` at the node's `body`, as
[FR-040](./FR-040-admit-frame-entries-and-state-clauses.md) fixes; a
`declaration`, a dependency or itself as its type refuses at that member.

A `scalar_type`/`compound_unit` node carries no `declaration` and is its own
semantic type. Its body is an `aggregate` of terms, each exactly
`aggregate{[binding "unit" = reference to a root `scalar_type`/`unit` node
(its nominal preimage has no target unit), binding "exponent" = an `integer`
literal typed at a `scalar_type`/`integer` node whose value is a canonical
nonzero decimal string]}`, strictly ascending by unit node key. Its
`dependencies` is exactly those unit keys in that order. The empty body is
the dimensionless unit.

A node whose body is an `application` term has
`dependencies` exactly the unique, digest-ascending reference targets and
operation member declarations of its body (FR-322
`application_node_preimage`); a `result_type` or literal `type` is not a
dependency. An application stands only at a body root ("The flat wire"), so the
join applies to the nodes whose body root is one. A member `declaration` that is not
a node key is left to the operation stage's refusal.

Any violation refuses as `invalid_semantic_graph` at the member that breaks
the rule (`semantic_graph.nodes.body`, `.dependencies` or `.semantic_type`),
located at the offending node, at the first such node in ascending node-id
digest order. A `declaration` on either form refuses as the declaration rule
above does. These are graph-shape refusals: they precede the stale-key
stage. The reader re-derives neither form's node key: QSL keys both by its
proposed `quire.structural-node/v1` preimage, which QSpec does not publish.

### Wire owner on checked nodes

FR-038-AC-153 through FR-038-AC-155 own the implemented owner-wire
boundary. IR-630 extends structural key validation to read that actual owner
and authenticate every structural node, grouped or not. The requirements
below replace the interim recursion-group skip; their new runtime controls
are PLANNED and UNRUN. The completed owner boundary alone is not evidence
that these key checks run.

The optional `owner` member of `CheckedSemanticNodeV2` has QSpec FR-322's closed `NodeOwner` shape:
`SourceOwner` `{kind: "source", authority, identity}` or `ModelOwner`
`{kind: "model", identity, node}`. The derived hash projection includes the node's owner, so changing it
changes `package_id`. A node is structural for this
presence rule when it has no `nominal_identity_preimage` and its body root is
not `application`. Every declared structural node SHALL carry the declaring
unit's `SourceOwner`. An undeclared `model`/`object_type`,
`model`/`systems_interface`, `relation`/`relationship`,
`relation`/`population` or `function` node
SHALL carry its `ModelOwner`. Every other node SHALL omit `owner`: this
includes anonymous types, undeclared values and parameters, nominal nodes
whose owner is in their nominal preimage, and application-keyed nodes. A
`DefinitionOwner` is not a node owner. The reader SHALL derive no missing
owner from occurrences, the source map, the lock, or node position.

The strict wire/schema step SHALL refuse a missing required `owner` as
`malformed_wire` at the node object that lacks it. It SHALL refuse an owner on
a node that must omit one, a wrong owner kind, `null`, a wrong member type or
an unknown owner member as `malformed_wire` at `owner`. A shipped projection
refuses at its member under the single-body rule before any projection-owner
contents can be inspected. There is no admission route for an owner-less
declared or model-owned structural node.

The owner-join step SHALL visit owner-bearing nodes in ascending retained
node-id digest order, before any derived-shape key check. A `SourceOwner` SHALL
match a `lock.sources` row by (`authority`, `identity`). Every source-map
region of that node's `declaration` occurrence SHALL name that same source
pair; a mismatch SHALL refuse `invalid_package`/`invalid-value` at the
occurrence's source-map entry, even when the other source is lock-selected.
A `ModelOwner` SHALL match a `lock.model_selections` row by `identity`, and
its `node` SHALL resolve in that selection's admitted document to the kind
the node represents: `model`/`object_type` requires an object type without
`interfaceFeatures`, `model`/`systems_interface` requires an object type with
`interfaceFeatures`, `relation`/`relationship` requires a relationship, and
`relation`/`population` requires a raw population record whose `identity` is
the owner's `node` in the selected document's `populations` list.
An operation clause function requires an operation member; an invariant clause
function requires an object type. The reader SHALL check this
for every model-owned node whether or not any other node references it, and
charge one validation visit for each document lookup. A missing selection,
declaration, member or matching kind SHALL refuse
`missing_declaration`/`missing-selection` at that node's `node_id`.

An owner that passes this join proceeds to the structural key stage below,
which SHALL read that actual owner for every structural key.
`correspondence`/`abstraction_relation` remains under FR-346's distinct
FR-451 key check. `source_locus` remains a structural form, and
application-keyed nodes remain owner-free.

### Declared population references

QSpec FR-322 "Population references" and FR-322-AC-82/83 own this reader
boundary. A declared population is one `relation`/`population` structural
node with a required `ModelOwner`, no declaration or nominal preimage, its
own `node_id` as `semantic_type`, an empty aggregate body and no dependencies.
The reader re-derives its `ModelDeclarationNode` key from the owner and
checks the same fixed members as every other model declaration node. The
owner's `identity` selects a domain document, and its `node` must equal the
`identity` of a raw population record in that document's `populations` list;
otherwise the owner join refuses `missing_declaration`/`missing-selection` at
the node id.
A selected `populations` entry that lacks an `identity` string or carries a
`node` member refuses `invalid_model_binding`/`malformed-declaration` at the
model selection before the owner join.
The declaration key is the two UTF-8 strings `owner.identity` and
`owner.node`, not the digest of this reference node. The node carries no
member type, extent or maximum. An absent owner refuses `malformed_wire`
before key re-derivation or the owner join.

The reader admits a reference to that node as operand 0 of
`quire.op.model.all_instances` or `quire.op.model.lookup`, includes it in the
application's dependencies, and distinguishes it from a `value`/`parameter`
binding typed by `bounded_domain`/`model_population` using the referenced
node's tag and form. Both are population-family operands in these two
positions. A unit-authored maximum belongs to the application's
`result_type` collection bounds, never to the population reference. A
population reference in a `collection_kind` operand or in a reaches
`relationship_end.population` link, or as the declaring node of a `field`,
`operation` or `relationship_end` member, refuses
`ill_typed`/`operator-ineligible` at the offending operand or member. The
existing `model_population` binding remains valid in its own positions.

### Anonymous structural node bodies and keys

Measured at the pre-IR-627 `origin/main` baseline (IR-627's earlier text was
re-measured): the reader re-derived three families of node key, the nominal
keys ("Every identity digest is computed through quire-canonical"), the
application keys ("Application node keys") and the model declaration node keys
(owner recovery, step 2 of "Model-owned members", which refuses
`stale-node-key` for a fixed member that does not hold). No baseline stage re-derived the
key of an anonymous structural node (`bounded_domain` of any form, and the
self-typed `scalar_type` and `composite_type` forms), so a node of that kind
was admitted with whatever `node_id` it carried. Step 4 of "Model-owned
members" compares a member's derived type with the application's `result_type`
by the digest of the node key alone: the reader derives the key of `Int[lo, hi]`
from the declared bounds (`MemberType::node_key`, `pub(super)`, so the package
exports no function for it) but never reads the body of the node that
`result_type` names, and an `Option`, a collection, a `Reference` or an
operation parameter type is compared by its key alone in the same way. A package whose `Int[0, 1000]`-keyed `bounded_domain`/
`integer_range` node carries the body bounds `0` and `10`, or `0` and `5000`,
under the unchanged `node_id`, with the package identity recomputed from
the tampered graph, therefore admitted before IR-627. A consumer that takes the range from the
node body (quire-contract-codegen's field range and state domain, which a
verification harness then assumes) assumes a range the declaration does not
state, and a narrower one yields a verdict that is unsound.

The preimage of an anonymous node is QSL's: QSpec FR-322 step 4 says each
anonymous member-type node is keyed "exactly as QSL FR-092 and FR-094 key it",
and QSpec publishes the `quire.structural-node/v1` preimage itself only for the
model declaration node (the `ModelOwner` form). The reader already derives, from
that QSL preimage, the keys of the ten node shapes `MemberType::node_key`
covers. This section does not state a preimage, copy a QSL or QSpec file, or
infer a preimage for any form the reader does not already derive. It decides
what needs no new preimage and gates the rest.

**Decided: re-derive the key of every node shape the reader already derives.**
The shapes, enumerated from `MemberType::node_key`, are `scalar_type` of form
`boolean` and `integer`; `composite_type` of form `reference`, `option`, `set`,
`bag`, `sequence` and `ordered_set`; and `bounded_domain` of form
`integer_range` and `collection_bounds`. Each of the following is its own requirement:

1. When a node of one of these shapes is outside a labelled group and carries
   no `declaration`, the reader shall re-derive its `node_id` from the node's
   own `node_tag`,
   `semantic_form`, `semantic_type` and body.
2. The reader shall derive that key by the one function `MemberType::node_key`
   calls, and by no second function.
3. When the body of such a node is not exactly the closed body of its form,
   the reader shall treat the node as having no derivable key.
4. When the stored `node_id` of such a node differs from the derived key, or
   the node has no derivable key, the reader shall refuse the package as
   `invalid_package` with cause `stale-node-key` at that node's `node_id`: the
   cause the reader already uses for a key that does not match its node.
5. When a `scalar_type` or `composite_type` node of these shapes is not its own
   `semantic_type`, the reader shall refuse it as item 4 does, because the
   preimage records that member as `null` and the key cannot show a
   re-pointed one. The `semantic_type` of a `bounded_domain` node is in the
   preimage, so the key covers it.

The closed bodies are: `scalar_type` `aggregate{[]}`; `reference`, `option` and
the four collections an `aggregate` of exactly one `reference`; `integer_range`
and `collection_bounds` an `aggregate` of exactly the bindings `min` then `max`,
each an `integer` literal typed at the node keyed as the `scalar_type`/`integer`
node and valued by a decimal string of the grammar of its form: for
`integer_range`, `^(0|-?[1-9][0-9]*)$` (an optional minus sign, no leading
zero, no `+`, and `-0` refused), and for `collection_bounds`,
`^(0|[1-9][0-9]*)$` (non-negative, since a collection bound is a count). The
reader's existing non-negative grammar (`is_non_negative_integer`) is the second;
the first is a new grammar and is not `is_nonzero_integer`. A string outside its
form's grammar leaves the body not closed, so the node has no derivable key.
The bounds are not parsed to a fixed-width integer or a float: they enter the
preimage as their strings, so there is no range limit other than the reader's
byte limit, a preimage past which refuses `invalid_semantic_graph` at the node
as an encoder refusal does today, and an `i128`-wide bound and one past it are
compared exactly. This requirement does not order `min` against `max`. The
literal `type` is part of the preimage, so a literal typed at any node other
than the derived `Integer` key changes the key.

This rule replaces an arithmetic comparison of body bounds against the member
type, on the merits. The comparison would reach only the node a model-owned
member read names, and would leave open a range reached through an `Option`, a
collection, an operation parameter's argument key, a state field's body target,
or a scalar operand's bound, each of which the consumer reads without passing
through that member read. Re-derivation checks the body of every node of the
ten shapes against its own key, so a node whose key equals a declared key holds
exactly the range that key denotes; a node of those shapes re-pointed at
another node changes its own key; and the existing digest comparisons of
"Model-owned members" step 4 and of operand and parameter types then compare
keys that are true. A separate arithmetic check would restate a subset of this
and is not specified.

What closes, and what stays open. The rule closes every route to a tampered
range whose referrers are all of the ten shapes. It does not close a route
through a gated form: a referrer that is not of the ten shapes can still be
re-pointed at a genuinely keyed narrower node. The cases are a `value`/
`parameter` node's `semantic_type`, which quire-contract-codegen's scalar
operand route reads (`operand_type_node`, `check_parameters`, `checked_bounds`);
an `alias`, `record`, `tuple` or `union` composite over a range; and a declared,
named `bounded_domain` (`type Small = Int[0, 9]`), which a state field's domain
reads. The implemented owner-free ungrouped stage remains a partial defence.
IR-630's planned extension below closes owner-bearing, declared and labelled
group routes by authenticating the complete structural preimage.
FR-038-AC-132 keeps authoritative conformance evidence at its owning checkout.

### Structural keys inside and outside labelled groups (IR-630)

This is a planned extension of the implemented owner-free, ungrouped stage.
The runtime controls of AC-131 and AC-145 through AC-150 are PLANNED and UNRUN;
old residual-admission tests and their trace tags do not prove the amended
refusals. No structural node SHALL be skipped because it carries an owner,
a declaration or a `recursion_group`, or because it lies on a names cycle.

**Reader group membership and order.** The reader SHALL follow the published
QSpec FR-322 reader contract: all nodes carrying one retained
`recursion_group` label are that labelled group's members, in their graph
array order. The member's ordinal is its position among those members and
size is their count. A node outside a names-graph strongly connected component
still participates when it carries that label. A lone label also selects a
group preimage; it does not select the ordinary ungrouped preimage.

QSL FR-092 computes its producer's groups from the names graph and orders
content classes by refinement of anonymous and full shapes. That producer
order and the reader's wire member order are different authorities: QSL emits
its content order on the wire, whereas this reader derives ordinals from that
wire. The reader SHALL NOT introduce SCC membership or content-order refusal
as an additional admission rule. A permutation with all node keys, labels,
references, projection and package identity correctly recomputed for the wire
order is judged by FR-322's checks; permutation alone is not a refusal.
Existing graph-shape and reference-cycle checks remain independent. A
`dependencies` edge or frame entry does not become a QSL names-graph edge;
no names-graph skip predicate is used by this key stage.

**One preimage authority.** The structural key stage SHALL authenticate every
node without a nominal preimage and without an application in its body,
including declared nodes, model declarations, model-owned clause functions
and structural correspondence forms. The sole exception is
`correspondence`/`abstraction_relation`, whose identity remains owned by
FR-451. Nominal and application identities retain their existing authorities.
The structural preimage SHALL be QSL FR-092's, with model-owned nodes governed
by FR-094. Its `owner`, when present, SHALL be the actual joined wire owner;
its declaration SHALL be the node's actual declaration, with only
`qualified_name` entering the preimage. Missing owners are never inferred.
`semantic_type` is null exactly for a self-typed node, unless a non-self type
names a member of the same group, when it is a group reference.

The production authority used here SHALL also be the authority called by
`MemberType::node_key`, not a parallel reader encoder. It SHALL support the
actual owner, declaration and group context while preserving the existing
anonymous preimages. Application members contribute their existing
application-node preimages to a group's local digest array; this does not
create a second application key algorithm. Model declarations retain their
closed self-typed, empty-body, ungrouped shape. The ten derived shapes retain
their closed bodies, self-type requirements and exact string-bound grammars;
a malformed shape has no derivable key. No range, bound, element type or field
type is authenticated by merely trusting a retained node key.

**First pass: current node keys.** After the existing owner joins, graph-shape
and application key checks, the reader SHALL visit every structural node in
ascending retained node-id digest order. An ungrouped node's preimage has
`recursion` null. For a labelled member, build its group-local preimage with
`recursion` `{size, ordinal}` and group references at the positions naming
members of that labelled group; the structural preimage's `recursion.group`
is then the retained label. Body references and a non-self `semantic_type`
that names a member use the member's wire ordinal. References outside the
labelled group remain node identities; dependency references remain external
package references. Compare the resulting key with the retained key. A stale
key or a closed shape with no derivable key SHALL refuse
`invalid_package`/`stale-node-key` at that node's `node_id`. The existing
expected-node-key retention of AC-183 applies when there is a derivable key.

**Second pass: group labels.** Only after every first-pass structural key is
current, the reader SHALL recompute each group's digest by QSL FR-092's
specified array of group-local preimage digests in wire ordinal order. A
structural local preimage omits `recursion.group`; an application local
preimage is its application-node preimage. This array includes every labelled
member, not only structural members or members on a cycle. Compare its digest
with the retained label. A mismatch SHALL refuse
`invalid_package`/`stale-node-key` at the group's least retained node-id member.
Visit groups in ascending order of that least member, so two failing labels
have a determined first refusal. Both passes precede nominal keys and every
declaration, frame, state, temporal, abstraction and operation step. The first
pass precedes the second even when a second-pass offender has a lower key.

**Finite, fallible work.** Group membership collection, each node validation,
each local-preimage term visit, each member digest and each group digest SHALL
be charged to the existing work meter before the charged work proceeds.
Each membership entry and each node-key visit costs one unit; each local
preimage term visit and each digest computation costs one unit. Walking uses
explicit heap stacks, with no stack recursion following graph size. Canonical
encoding SHALL use `quire-canonical` under the selected byte ceiling; the
existing encoder-refusal behavior remains. No new limit kind, raised default,
extra cancellation API or unbounded refinement pass is introduced. An unpaid
charge SHALL immediately return the existing work-incomplete result with the
first unpayable value's pointer and no partial package, and stop all remaining
group work. Derived state is local to the read and discarded on refusal or
incomplete outcome; an abandoned read exposes no partial admitted package.

**Retired interim admissions.** The former AC-146 acceptance of an ordinary
ungrouped key plus a lone label, and AC-147 acceptance of an off-cycle
same-label node with its ordinary ungrouped key, are retired. Both now select
labelled preimages and can refuse stale keys even with unchanged bodies.
The former AC-148 tamper and forged-cycle admissions are refusal regressions.
A correctly keyed owner-free self-option group (QSL G1), and correctly keyed
`List` and `Tree` groups, remain positive controls. Adding a cycle or label to
an old key does not authenticate its body.

**Trust root.** The member type step 4 derives comes from the domain document
that the caller's evidence supplies under the `model_selections` digest, whose
RFC 8785 digest and identity the reader recomputes (FR-038-AC-27). The check is
sound relative to that evidence. A tamperer who re-points `model_selections` at
a document declaring a narrower range changes `package_id` and, with no such
document in the caller's evidence, is refused `missing_import`/`missing-selection`
at the row's `digest` (FR-038-AC-27); which document is authoritative for a
lock is the caller's decision and is outside this reader, which admits a package
against whatever evidence it is given.

**Implemented baseline and planned extension.** The existing re-derivation
excludes owner-bearing and declared structural nodes and most grouped forms,
and skips six derived shapes on same-label names cycles. Labelled off-cycle
nodes of the ten shapes still use ordinary ungrouped keys in that baseline.
Those exclusions and skip choices are implementation limits, not the amended
contract. IR-630's two-pass extension above SHALL replace them, retaining
nominal, application and FR-451 identity ownership. Until that code and its
new controls are run, those old admission paths remain an implementation gap.
The owner join remains ahead of both key passes, including when an owner
defect is on a higher-key node than a stale structural key.

**Fixtures carry derived keys.** Measured against QSpec's `origin/main`
(2b2dd28; a stale local checkout differs): the fixtures FR-038-AC-107 reads
live in QSpec's checkout (`proposals/checked-package-v2/fixtures/`), not in this
repository. The undeclared nodes of a derived shape, each under a placeholder
key, are: in `positive-all-families.json`, node 24 (`scalar_type`/`integer`,
`7f7f...`) and node 26 (`composite_type`/`reference`, `b2b2...`); in
`positive-clause-operations.json`, node 0 (`boolean`, `aaaa...`), node 7
(`integer`, `7f7f...`) and node 12 (`reference`, `b2b2...`); in
`positive-union-nodes.json`, node 0 (`integer`, `a1a1...`). Two nodes of
`positive-all-families.json` carry a `declaration` and are therefore gated
forms, not decided shapes: node 0 (`boolean`, `aaaa...`, `Example::Flag`) and
node 2 (`integer_range`, `cccc...`, `Example::Small`). None of those keys is the
key QSL FR-092 gives the shape. The in-repo fixtures
(`tests/it/support/checked_package.rs`) are not the same fixtures but fail the
same rule: their `aaaa` boolean has a `literal` body, not `aggregate{[]}`, and
their `bbbb` option is typed at `aaaa` with an empty aggregate body, so items 1
to 5 of the decided rule refuse them whatever their key. The decided stage
refuses each undeclared derived-shape node `stale-node-key`, and no
tolerance for a placeholder key is specified: a placeholder key is the broken
state the check exists to catch. FR-038-AC-107 and FR-038-AC-112 describe what
the reader does today and stay as they are until the code lands. Each
requirement below is its own statement:

1. A conformant positive fixture shall carry, on every node of the ten shapes,
   the key the reader derives for it (FR-038-AC-133).
2. The in-repo fixtures are owned by this repository's code change, which shall
   regenerate their derived-shape keys and also change their bodies and
   `semantic_type` to the closed forms (FR-038-AC-134).
3. The QSpec fixtures are owned by the QSpec owner (QSL-635, IR-627-Q5); the code change
   for FR-038-AC-123 through FR-038-AC-130 shall land after those fixtures
   carry derived keys, because its stage would otherwise make
   `make conformance-qspec` fail on AC-107, and this repository does not copy
   or locally regenerate them.
4. The `adverse.json` mutations of AC-112 shall reach their recorded refusals
   with derived keys on the base package (FR-038-AC-135). Measured over the
   eleven entries of `adverse.json` (six `structural_mutations`, five
   `body_grammar_mutations`): the pointers are `/contract_version`,
   `/lock/sources/0/digest_domain`, `/package_id/domain`,
   `/semantic_graph/nodes/0/node_tag` and `.../semantic_form` (node 0 is the
   declared boolean `Example::Flag`, a gated form the stage does not take;
   `unknown-semantic-node-tag` is refused at the node-tag decode, and
   `wrong-node-family-form` replaces the form with `protocol_clause`),
   `/semantic_graph/nodes/29/body/operation/member/interval/lower` (a temporal
   formula, `negative-temporal-interval-bound`), `/semantic_graph/nodes/1/body[/members]`
   (a `record`, a gated form; four entries) and
   `/semantic_graph/nodes/5/body/arguments/0` (a function), so no listed
   mutation lies inside the body of a node of the ten shapes; with real keys on
   the base each keeps its recorded stage, and one the new stage pre-empts as
   `stale-node-key` fails the harness as AC-112 already says. The package that
   AC-113's `dependency-selection-vectors.json` names as its base is the same
   `positive-all-families.json`, so its recorded `package_id` changes when
   QSL-635 regenerates the base.

Open questions (IR-627-Q1 to Q5), for the owner of QSL FR-092 and FR-094 and
for the QSpec owner of FR-322:

1. **Q1.** Is `quire.structural-node/v1`, as QSL FR-092 and FR-094 publish it,
   the authoritative preimage of each anonymous and each declared structural
   node, and does it publish golden vectors for the forms the reader does not
   yet derive (the gated list above)?
2. **Q2.** How does IR depend on that preimage without copying it: golden
   vectors read from the QSL or QSpec checkout at a `make conformance-*`
   target (as FR-038-AC-112 and AC-113 read QSpec's `adverse.json`), or a
   published type in `quire-canonical`? Neither copies a file into this
   repository.
3. **Q3.** Is `invalid_package`/`stale-node-key` the refusal for a structural
   key that differs from its body, or does QSpec want a distinct cause?
4. **Q4.** Resolved for this reader's IR-630 specification: wire owners are
   supplied by the implemented owner boundary. The published FR-322 reader
   contract uses all nodes sharing a retained label and graph-position
   ordinals, then authenticates both keys and the group label. QSL FR-092's
   names-graph SCC and content refinement describe the producer; they do not
   add a reader rejection. The earlier SCC skip and ordinary-key-plus-label
   admissions are superseded by the two-pass requirements above. An owner-free
   group such as QSL G1 needs no inferred owner. Whether a source checker
   produces G1 is separate from whether this reader authenticates its wire
   representation. Runtime implementation and qualification remain planned.
5. **Q5.** QSpec's checkout holds derived-shape nodes under placeholder keys
   (`aaaa...` boolean, `7f7f...` integer, `b2b2...` reference, `a1a1...` integer
   in the union fixture), which QSL FR-092's preimage does not produce, in these
   files under `proposals/checked-package-v2/`, measured before QSL-635
   refreshed the QSpec fixtures. IR reads three: `fixtures/positive-all-families.json`,
   `fixtures/positive-clause-operations.json` and `fixtures/positive-union-nodes.json`
   (FR-038-AC-107), applies `fixtures/adverse.json` to the first
   (FR-038-AC-112), and reads `dependency-selection-vectors.json`
   (FR-038-AC-113), whose base is the first and whose recorded `package_id`
   changes when the base is regenerated. IR reads none of
   `fixtures/positive-control-operations.json`,
   `fixtures/positive-operation-identities.json` and `node-identity-vectors.json`
   (the in-repo tests that read the vectors were removed, AGE-1961), which also
   carry placeholders (`b2b2...` in the vectors). The QSpec-side ticket is
   QSL-635 (QSL lane): regenerate every positive fixture and the vectors with
   keys from QSL's own derivation, recompute each projection and `package_id`,
   and add a negative fixture. Which IR release waits on it is Q5's question:
   the IR code change for FR-038-AC-123 through FR-038-AC-130 does not merge
   before QSL-635's fixtures, or a conformance lane is held citing QSL-635. This
   specification change does not wait.

   This Q5 inventory is a historical measurement of that earlier checkout:
   FR-038-AC-176 now plans to read all nine published positive fixtures,
   including `positive-control-operations.json` and
   `positive-operation-identities.json`, from the current QSpec checkout.

IR-628 (a typed accessor for a model object type's effective fields) is decided
in "Typed accessor for a model object type's fields" below. It does not remove
the need for the re-derivation above, because any node a consumer reads can be
tampered; it removes the need to read the model declaration path's range from a
node at all.

### Typed accessor for a model object type's fields

Measured at `origin/main` (IR-628, whose own text is a claim and was
re-measured). A model declaration node's body is `aggregate{[]}` and carries no
member (QSpec FR-322 "Model-owned members"), so an admitted
`CheckedPackageV2` holds no field of a model object type in any node. The field
set and each field's declared type are in the selected domain package document,
which the reader parses into a `DomainModel` (`model_members.rs`) while it
admits the package and then drops: `CheckedPackageV2` keeps only `wire`, `kinds`
and the byte limit. The resolution (`DomainModel::resolve`, `field_type`,
`slot_type`, `MemberType`, `MemberType::node_key`, `ModelOwners::recover`) is
`pub(super)` or private, so the admitted package exports no function that
returns a field, a member type or an integer bound as a value. The only place a
consumer finds a field's range is the `result_type` of an
`expression`/`query` node that reads the field (`check_model_member` in
`operations.rs` compares that node's key with the derived member type's key),
which gives no range to a field no clause reads and no list of fields. The
derivation already exists and is already the one admission checks; the missing
piece is a typed way out of it and the package keeping what it derives from.

**Decided: one accessor on the admitted package, over the retained derivation.**
Each of the following is its own requirement:

1. At admission, the reader shall compute, for every object type and systems interface declaration of each
   admitted `lock.model_selections` document, a field table holding that
   declaration's exposed effective fields (the members of `MemberKind::Field`
   that FR-322 step 3 exposes: its own and inherited fields, less every field a
   redefinition hides and every redefining field of a less derived owner), each
   with the member type `DomainModel::field_type` derives, and its hidden
   entries.
2. The reader shall build the table of an object type from its own fields and
   the tables of its declared supertypes, each declaration built once.
3. The reader shall not re-walk the ancestors of each object type to build its
   table.
4. Where declared supertypes form a cycle (two types that extend each other, or
   a type that extends itself), the reader shall build one table per strongly
   connected component, shared by its members, over the owners that
   `DomainModel::ancestors`' reached-set rule gives today: the component and
   every type reachable from it, with a member of a cycle among its own
   ancestors.
5. The reader shall not refuse a document for a supertype cycle, which admits
   today (`check_supertype` checks only that the supertype names an object type
   of the document).
6. The exposed fields of each table, for a cyclic and an acyclic document alike,
   shall equal the member set that `DomainModel::resolve` returns today for
   `MemberKind::Field` over that type: the same fields, the same hidden fields,
   by the same redefinition rules (including that a redefining member of an
   owner that is its own ancestor hides itself and its target).
7. The reader shall use the table for the field case of `DomainModel::resolve`,
   so that `resolve` selects the named field from the table and by no second walk
   of the declarations; the `MemberKind::Operation` case of `resolve` is not
   changed.
8. The reader shall charge the build of each table to the `work` limit at the
   row of its selection, so that an exhausted limit is `incomplete` with the
   pointer `/lock/model_selections/<i>`.
9. The charge of one table (one per component, not per member of a cycle) shall
   be one unit per own field of each member, one per declared supertype edge of
   each member, and one per exposed or hidden entry copied from the table of a
   supertype outside the component. Each declared edge incurs this copied-entry
   charge, including repeated edges and both arms of a diamond.
10. Every resolution of a field through `DomainModel::resolve` (a read,
    including a relationship edge read, a frame entry, or an abstraction
    relation's field entry; a state clause resolves only operations) shall
    charge, in place of the step 3 walk, one unit per exposed or hidden entry of
    the table it selects from, at the selection's row.
11. The reader shall build and charge the tables as the last act of the lock
    stage: after step 1 has admitted every `lock.model_selections` row, in lock
    order of the rows, and before the graph stage (`validate_graph`).
12. The reader shall retain the field tables, and an owned index from each
    selected declaration's model declaration node key to its selection row, form
    and node identity, in the admitted `CheckedPackageV2`.
13. Where the exposed fields of an object type include two of one name, the
    reader shall retain that table as ambiguous, carrying the name.
14. The reader shall not refuse admission for an ambiguous table.
15. `CheckedPackageV2` shall export `model_object_fields(&self, node:
    &CheckedNodeId) -> Result<CheckedModelObjectFields,
    CheckedModelFieldsError>`, which takes no evidence argument.
16. When the graph holds no node whose `node_id` equals `node`, the accessor
    shall return `CheckedModelFieldsError::UnknownNode`.
17. When the node exists and is not an object type's model declaration node, the
   accessor shall return `CheckedModelFieldsError::NotModelObjectType`. A node is
   an object type's model declaration node when its `node_id` digest is the
   retained index's key of an object type (not a `systems_interface` or
   `relationship`) declaration, its `node_tag` and `semantic_form` are that
   declaration's, and its fixed members hold (`semantic_type` is itself, no
   `declaration`, no `recursion_group`, body `aggregate{[]}`): the rule of
   `ModelOwners::recover` (FR-322 step 2), shared with it and not restated by a
   second function.
18. When the node is an object type's model declaration node and its retained
    table is not ambiguous, the accessor shall return that table's fields.
19. When the retained table of the node is ambiguous, the accessor shall return
    `CheckedModelFieldsError::AmbiguousField` carrying the smallest ambiguous
    name in bytewise UTF-8 order, for the whole call.
20. The accessor shall return the fields in ascending order of field name
    (bytewise UTF-8), each name once.
21. The member type of each returned field shall be the one
    `DomainModel::field_type` derived at admission, the function
    `check_model_member` calls, so that a field's reported type is the type whose
    node key admission compares with a read's `result_type`.
22. The member type shall be the closed enum `CheckedMemberType` with the variants
    `Boolean`, `Integer`, `String`, `Uuid`, `Timestamp`, `IntRange` carrying `lower` and `upper` as `i128`
    values, `Reference` carrying the `CheckedNodeId` of the referenced object
    type's model declaration node, `Option` carrying its element type, and
    `Collection` carrying the closed enum `CheckedCollectionKind` (`Set`, `Bag`,
    `Sequence`, `OrderedSet`), its element type and its bounds as
    `Option<(u64, u64)>`; it mirrors `MemberType` variant for variant.
23. Where the tables of FR-322 step 4 give a field no type (a declared type that
    is a rational, decimal, float, text or systems interface, a
    value type not bound as `Int[lo, hi]` (including one whose bound lies
    outside `i128`), a type of another FR-208 meaning, or a collection with a
    positive lower bound and no upper bound), the accessor shall return the
    field with `member_type` `None`.
24. A field with `member_type` `None` shall not make the call refuse.
25. A bound outside `i128` shall give `member_type` `None`.
26. A bound outside `i128` shall never be saturated, wrapped or truncated into
    an `IntRange`.
27. The accessor shall read no node body other than the passed node's own fixed
    members (item 17).
28. The accessor shall read no `result_type`.
29. `CheckedModelObjectFields` shall expose `fields()`, a slice of
    `CheckedModelField` in the order of item 20.
30. `CheckedModelObjectFields` shall expose `field(&str) ->
    Option<&CheckedModelField>`, which is `None` for an absent field and not an
    error.
31. `CheckedModelField` shall expose `name()` and `member_type()`
    (`Option<&CheckedMemberType>`).
32. The public enums `CheckedMemberType`, `CheckedCollectionKind` and
    `CheckedModelFieldsError` shall be closed, with no `#[non_exhaustive]`, as
    every public enum of the crate is (AD-006): a variant added later stops a
    consumer's exhaustive match from compiling, which is intended.
33. The structs `CheckedModelObjectFields` and `CheckedModelField` shall keep
    their fields private and expose the immutable accessors of items 29 to 31,
    as validated types of the crate do (FR-019).
34. The accessor shall not panic.
35. The accessor shall charge no work limit, because it performs no resolution:
    it reads the retained table (items 1 to 14).
36. The accessor shall return equal results for equal calls and for a clone of
    the package.
37. The retained tables and index shall not take part in `PartialEq` for
    `CheckedPackageV2`: equality stays the equality of the admitted content,
    which fixes each document through its digest.

**Cost, stated.** The charge of items 8 and 9 is proportional to the total size
of the tables, the sum over object types and systems interfaces of (own
fields + supertype edges + entries copied), not to an ancestor re-walk per type (which would charge about N²/2 edges for a
chain of N before any member). It is still quadratic in the depth of an
inheritance chain whose types each declare a field, because the table of a
type at depth k holds k exposed fields, so the output itself is quadratic: for a
chain of N types with one field each the charge is N(N+1)/2 + N − 1 (the first
type 1, the type at depth k ≥ 2 charges k + 1: one own field, one edge and k − 1
copied entries). A flat model, or one whose types declare few fields, costs
linearly in its fields. Retained memory is at most one table entry per charged
unit. The packages that change outcome are those that admit today and whose
selected documents' total table charge, added to their present work, exceeds the
caller's `work` limit: under `CheckedPackageReadLimits::bounded()` (work
1,000,000), an unread chain of N = 1500 types with one field each charges
1,500 × 1,501 / 2 + 1,499 = 1,127,249 and is `incomplete` at
`/lock/model_selections/<i>`, where it admits today; a chain of N = 1000 charges
1,000 × 1,001 / 2 + 999 = 501,499 and still admits (its document charge is
linear). A cyclic document is charged once per component: a type that extends
itself with one field charges 2 (one own field, one edge, nothing copied), two
types that extend each other with one field each charge 4 (two own fields, two
edges), and both tables hold both fields. A field resolution (a read, a frame entry, an abstraction relation's field entry) no longer charges
the step 3 walk: it charges one unit per entry of the table it selects from
(item 10), so a read-heavy package's work count falls (a package `incomplete`
today for its reads can admit) while the table charge can raise it; the existing
exact-limit tests of packages with field reads need their limits re-measured by
the code change. The step 3 walk of `MemberKind::Operation` is not changed. This
is owner-visible in IR-628-Q2.

Note (not a requirement). The code change that adds the five new public type
names (`CheckedModelObjectFields`, `CheckedModelField`, `CheckedMemberType`,
`CheckedCollectionKind`, `CheckedModelFieldsError`) is also the change that
lists them in FR-019's Public items table, row `checked_package` (V2 reader).
No gate enforces that table today: FR-019-AC-5's inventory test (TC-058) is
planned, and when it lands it fails on a name the table omits.

**What the accessor does after admission.** The reader authenticates every
selected model declaration's closed, empty body before exporting the accessor.
An unread `model`/`object_type` declaration whose retained body changes or
whose key matches no selected declaration therefore cannot supply an admitted
package: key authentication or owner joining refuses first. The accessor
applies FR-322 step 2 (`ModelOwners::recover`) to an admitted node at the call
(item 17), returns fields only for a recovered object type, and uses the
document's retained table rather than the node body for those fields. Admitted
non-object nodes still return `NotModelObjectType`; an absent graph id returns
`UnknownNode`.

**Why a typed enum with `None` and not a refusal for other kinds.** A model
object with one `Text` field and one `Int[0, 1000]` field is common. A refusal
for the whole object would hide the range of the `Int` field from a consumer
that wants only the ranges, and the consumer would still have to re-derive the
field set to find out which field refused. Returning every effective field, with
`None` where the tables derive no type, gives the consumer the field set (which
no read gives) and lets it decide per field (quire-contract-codegen records a
field with no range and its reason). The accessor invents no type for a form the
reader derives none for: a rational, text or compound-unit field has no
`MemberType` and no preimage here (the gated forms of "Anonymous structural node
bodies and keys"), so it is `None` and not a guessed variant. A variant per
unsupported FR-208 meaning would copy the document's type system into the public
surface and break on every new meaning; the closed enum holds exactly what
`MemberType` holds. `UnknownNode` and `NotModelObjectType` are separate because a
caller that holds an id from outside the package (a mistyped id) and a caller
that holds a node that is not a model object type (a fixture-built object
carrying body members, which the consumer reads from the body) take different
paths.

**Why an ambiguous name refuses the whole call and a `None` type does not.**
`None` is a legitimate declared type outside the tables: the field exists, has one
meaning, and the consumer decides what to do with a type it does not model.
An ambiguous name is a defect of the model document: the field set itself has no
defined content, since two exposed members claim one name, and admission refuses
the same condition at any read (`ambiguous_declaration`/`ambiguous-name`, step 3).
A partial list that dropped or guessed the ambiguous name would let a consumer
assume a field set that admission would not resolve; a refusal naming the name
tells the consumer exactly which declaration to fix. The asymmetry is therefore
deliberate, and the consumer that needs the other fields has the document's own
refusal to act on.

**What the accessor guarantees, stated exactly.** The bounds it returns are the
bounds of the type `field_type` derived from the selected document at admission,
and admission checks every read of that field against the key of that same
type. The accessor therefore depends on no node body: the IR-627 tamper (an
`Int[0, 1000]`-keyed node with body bounds 0 and 10) does not reach it, and it is
sound on a reader without IR-627's stage. It is sound relative to the same
evidence IR-627's "Trust root" paragraph names: the document is the one the
caller's evidence supplied under the `model_selections` digest, whose RFC 8785
digest and identity the reader recomputed (FR-038-AC-27), so a package re-pointed
at a document declaring a narrower range, admitted with that document in
evidence, returns that document's bounds (FR-038-AC-129); which document is
authoritative is the caller's decision. For a field no read names, admission
checked only the selected document's digest, identity and declarations (step 1),
not that field's type against a node, because no node names it; the accessor's
answer for that field is the document's, derived by the same function. This
accessor does not make a node body trustworthy and does not close any gated
route of IR-627 (a state field's body target, a scalar operand's bound, a
declared `bounded_domain`): a consumer that reads those routes from a node body
is still bound by IR-627's gate. It also does not extend to an object that is not
a model declaration node, whose members are in its body and are checked by
`check_field_member` by name only.

**What quire-contract-codegen can then retire.** Measured in CG's `origin/main`
(FR-015 "IR-624 (planned)" section, FR-024 Q-5), which this repository does not
edit. For the model declaration path only: the read of a field's range from the
`result_type` of a read (`field_range` over an empty-bodied model declaration
node), with the three causes it adds for that read (`FieldNotRead`,
`ConflictingFieldReads`, and `MemberDisagreesWithRead` for that path); the
`NoRead` record reason of FR-015-AC-81 (every field of a model declaration has a
declared type, so a field is `None` or has a range, never "no read"); and the
state-field list from the request alone, since the accessor supplies the field
set. The accessor lifts the gate "IR-627 or IR-628" that CG's text puts on
FR-015-AC-77 through FR-015-AC-81 and FR-024-AC-31 through FR-024-AC-35 for the
model declaration path, with no dependency on IR-627's stage, but it does not
satisfy them as CG wrote them: FR-015-AC-77 reads the range "from the reads'
`result_type`", FR-015-AC-78 defines `FieldNotRead`, `ConflictingFieldReads`
and `MemberDisagreesWithRead`, and FR-015-AC-81 defines `NoRead`, which are the
route this accessor replaces. CG shall AMEND FR-015-AC-77, AC-78 and AC-81 (and
the FR-024 criteria that restate them) to the accessor route, not only ungate
them. CG keeps the body path for objects that are not
model declaration nodes, and keeps its gate on IR-627 for any range it reads from
a node body (the scalar operand route, a state field's body target, a declared
`bounded_domain`). This specification does not edit CG's text. The downstream
ticket to file at merge (the lead files it, in the CG team): "CG: read a model
declaration object's field set and ranges from
`CheckedPackageV2::model_object_fields`; amend FR-015-AC-77, AC-78 and AC-81 and
the FR-024 criteria that restate them, retiring the `result_type` route, its
three causes and the `NoRead` reason for that path; lift the IR-628 arm of the
GATED label on FR-015-AC-77 through AC-81 and FR-024-AC-31 through AC-35; keep
the IR-627 gate on every route that reads a node body", blocked by the IR-628
code change.

Open questions, for the owner of IR (decided here on the merits and not
settled until the owner confirms):

1. **IR-628-Q1.** Should each field also carry the node key of its member type
   (`CheckedNodeId`), so a consumer can join a field to the node a read names?
   Decided no: a `Reference` already carries its target's key, but the key of
   an `IntRange`, `Option` or `Collection` is an anonymous structural key that
   admission derives only for a type a read names; deriving it for every field
   would add an encoder derivation (refusable at the byte limit) to the table
   at admission, and would make that unpublished QSL preimage's output part of
   the public surface when the consumer needs the value, which it has.
2. **IR-628-Q2.** Who pays for the field tables? Decided: admission, once, for
   every declared object type of every selected document (not only the unread
   ones), built incrementally from supertypes' tables and charged per the
   "Cost, stated" paragraph (items 8 and 9); the accessor charges nothing (item
   35); every field resolution (read, frame entry, abstraction relation's field entry) charges one unit per table entry in place of the step 3
   walk (item 10), so reads cost less and the exact-limit tests need
   re-measuring. Supertype cycles are not refused (item 5): they get one shared
   table per component, equal to what `resolve` returns today (item 6), which
   is an owner-visible choice (the alternative, refusing a cycle at step 1,
   changes what admits today and is not taken here). The cost is quadratic in the
   depth of a chain whose types each declare a field, so a package that admits
   today can become `incomplete`: under `bounded()` an unread chain of 1500
   such types does (1,127,249 units against 1,000,000), one of 1000 does not.
   The alternative the owner may prefer is to retain only own fields and
   supertype edges and resolve at call under a caller-supplied limit, which
   changes no admission outcome and adds a limit parameter; not chosen, because
   it gives the accessor a failure mode (`incomplete`) a consumer must handle
   per call.
3. **IR-628-Q3.** Should `systems_interface` and `relationship` declarations
   have an accessor? Decided no: no consumer reads them (CG's frame object is an
   object type); they return `NotModelObjectType`.

### Typed clause-context integer comparisons

For a caller-supplied `state`/`state_clause` node id and one authentic `claim`
occurrence of that node, `CheckedPackageV2` shall expose a typed accessor for
one direct integer comparison in its Boolean condition. The clause kind shall
be `invariant`, `precondition` or `postcondition`. The accessor shall follow
the clause body's third argument, a reference to the comparison application
node, and admit only the catalogued identities `quire.op.integer.eq`, `.ne`,
`.lt`, `.le`, `.gt` and `.ge`, each with exactly two arguments. It shall return
the actual clause id, supplied claim occurrence, clause kind, comparison node
id, closed comparison operator and two typed operand entries in authored
`body.arguments` order with ordinals 0 and 1. At least one operand shall be a
read. The accessor does not evaluate the predicate, perform bound-oracle
admission or construct a codegen obligation.

The clause body, parameter order and checked signature join are the committed
[QSpec FR-341](ix://agent-ix/quire-specification/FR-341) contract; QSL
[FR-093](ix://agent-ix/quire-spec-language/FR-093) and
[FR-105](ix://agent-ix/quire-spec-language/FR-105) supply its expression and
state-clause lowering. The downstream census, clause kinds and observations
are those of committed [CG FR-008](ix://agent-ix/quire-contract-codegen/FR-008).

An eligible read is one of two provenance forms. A `StateField` read is a
reference to `quire.op.record.project` with a `field` member naming its
selected `model`/`object_type` declaration and field name, over a
`quire.op.model.deref` whose sole reference is the clause's level-0 `self`
parameter from the FR-341 parameter aggregate. An `OperationInput` read is a
direct reference to one of that aggregate's operation-parameter nodes. The
reader's FR-341 signature join fixes those slots after `self` and, when
present, `result`; the `result` slot is not an operation input. The accessor
shall use the aggregate position and the reader-checked anchor/signature
join, not the parameter's spelling, to identify the slot. Its typed read
result shall carry `StateField { project: CheckedNodeId, declaration:
CheckedNodeId, field: Box<str> }` or `OperationInput { parameter:
CheckedNodeId, ordinal: u64 }` provenance. The `OperationInput` ordinal is
its position among operation parameters, starting at zero. A `let` alias,
second-level project, unrelated parameter, result slot or other expression
shall return typed `IneligibleOperand`, even if it has a matching name or
integer type.

The typed provenance is the authoritative input to CG's declaration-kind
mapping for this V2 seam: CG maps `StateField` to its FR-008 `State` census
kind and `OperationInput` to `Input`. IR owns the distinction by the
FR-341 clause aggregate and checked field member; CG owns its own census
kind names and alias rendering. Neither component shall infer V1
Input/State kind from a V2 parameter name, expression node id, source
region or JSON body decoded in CG. No V2 per-use metadata or V1 kind wire
member is added. An invariant has only `self` and therefore no
`OperationInput`; preconditions and postconditions may carry operation
inputs in their checked signatures.

The accessor shall derive observation from the path starting at the supplied
clause's condition. A direct `StateField` is `Current` in an invariant or
precondition and `Post` in a postcondition. One `quire.op.state.pre` wrapper
around a `StateField` in a postcondition is `Pre`. A direct `OperationInput`
is `Current` in an eligible clause. Other wrappers, including `pre` in an
invariant or precondition and `pre` around an operation input, are
`IneligibleOperand`. A content-addressed project reached directly and
through `pre` in one postcondition retains the same project id but gives
its two entries `Post` and `Pre` respectively. No snapshot label belongs
to the project node globally.

An eligible literal is either an inline integer `literal` term at one
comparison argument or a reference to a `value`/`literal` graph node whose
body is an integer literal. Its exact canonical decimal value shall fit
`i128`; its range is the singleton `(value, value)`. The typed identity is
`GraphChild(CheckedNodeId)` for a referenced literal and
`InlineLiteral { comparison: CheckedNodeId, clause: CheckedNodeId,
claim: CheckedOccurrence, ordinal: u64 }` for an inline term. These are
positions, not new graph nodes or child expression occurrences. An
out-of-`i128` literal returns `RangeOutOfI128`, with no narrowed value.
A comparison with two literals returns `NoRead`, without a partial result.

Each read entry shall carry its typed provenance, observation and exact
finite inclusive `(lower, upper)` bounds as `i128`. A `StateField` range
comes from the admitted selected model's effective field type, through the
same retained table as `model_object_fields` and the read's admission-checked
member join. An `OperationInput` range comes from the selected model
operation parameter type that the FR-341 signature join compared with the
parameter's `semantic_type`, not from an unverified type-node body. Only a
finite `IntRange` is eligible. An unbounded `Integer` yields
`UnboundedRange`. An absent or unrepresentable model member type yields
defensive `MissingRange`.
No endpoint is rounded, clamped or inferred from an unverified field body.
The reader's model-member join does not admit a direct field read whose
derived member type is absent, so field `MissingRange` is defensive on a
post-admission inconsistency. Every failure returns no partial result.

The closed typed refusal shall distinguish `UnknownClause`,
`NotStateClause`, `MissingClaimOccurrence`, `UnsupportedClauseKind`,
`MissingComparison`, `UnknownOperator`, `IneligibleOperator`,
`InvalidComparisonShape`, `IneligibleOperand`, `MissingChild`, `NoRead`,
`MissingRange`, `UnboundedRange` and `RangeOutOfI128`. Every refusal shall
carry the supplied clause id and claim occurrence plus a deterministic path
into that clause's condition and, when reached, the actual child id. This
structural locus does not claim a unique child source region. Refusal order
shall be clause existence, clause form, claim occurrence, clause kind,
condition reference, comparison identity/arity, then operand 0 before
operand 1; within an operand, shape and child joins precede type and
bounds, and `NoRead` follows both well-formed literal operands. Malformed
post-admission graph mutations shall return the applicable typed refusal
without panic or a partial result. Reader admission and its refusal order
do not change.

A `claim` occurrence identifies the supplied clause occurrence, but V2
reference edges contain node ids without child occurrence keys. The
accessor therefore does not select or return a comparison, project or
literal `expression` occurrence from that claim, nor assert that a child's
source-map region is uniquely associated with it. Source-map regions remain
available by a separate lookup. An external Rust consumer shall use public
typed inputs and outputs without decoding `body` JSON. Repeated calls and a
cloned package shall return equal results without changing package equality
or any wire metadata.

### Typed scalar application operands

When a caller supplies an admitted application node id and one of that node's
`CheckedOccurrence` keys, `CheckedPackageV2::scalar_application_operands` shall
return a typed result in `body.arguments` order for the integer `add`,
`sub`, `mul` and `negate` operation identities published in
`quire-verification-contracts/contracts/checked-operation-catalog-v1.json`
and used by QSL [FR-357](ix://agent-ix/quire-spec-language/FR-357). The accessor does not
evaluate the operation or mint a CG obligation identity. Its input and output
use public Rust types, not `serde_json::Value`; CG consumes the typed result
without decoding the package's `body` itself.
Composite equality operands have structural domains and are outside this
scalar accessor; their typed accessor is a planned follow-on.

Each returned entry holds the zero-based `ordinal`, a child identity, and the
inclusive `(lower, upper)` operand range as exact `i128` values. The accessor
returns entries only when every operand has finite bounds and both endpoints
fit `i128`; package admission alone does not impose either condition. A
reference to a graph child carries that child's `CheckedNodeId`: a parameter
reference names the `value`/`parameter` node, while a reference to a subterm
names the subterm's own application node. Its range is the admitted integer
range of that value's semantic type, except that a `value`/`literal` child has the
singleton range of its integer value. An inline integer literal has the same
singleton range. Its identity is a distinct typed
`InlineLiteral { application: CheckedNodeId, occurrence: CheckedOccurrence,
ordinal: u64 }` variant, not a fabricated graph node id. This distinguishes
two literal positions and two occurrences of the same application. The
`GraphChild(CheckedNodeId)` variant identifies a referenced node. The
accessor returns no occurrence regions or serialized term fragments.

The closed typed error distinguishes `UnknownNode`, `NotApplication`,
`UnknownOperator`, `IneligibleOperator`, `MissingOccurrence`, `MissingChild`,
`MissingRange`, `UnboundedRange` and `RangeOutOfI128`. A missing occurrence
refuses even if another occurrence of the same node exists. A dangling
reference or a missing bound refuses instead of returning a partial operand
list or guessing a range. An admitted operand with a valid unbounded `Integer`
type returns `UnboundedRange`; an admitted integer-range endpoint, graph
literal value or inline literal value outside `i128` returns `RangeOutOfI128`.
Neither case narrows, saturates or truncates the value, and neither returns a
partial operand list. An admitted package ordinarily rules out a dangling
child; the accessor still returns a typed error if its internal graph cannot
resolve one. It never panics on any input. These accessor errors do not change
package admission or the V2 reader's refusal order.

The admitted flat wire permits both a reference to a `value`/`literal` graph
node and an inline literal term inside `arguments`. QSL FR-357 requires an
operator-level literal's singleton range; the inline selector above gives
that term a distinct identity without inventing a graph node. ADR-013 O-09
identifies the *application obligation* by application node id and occurrence
key; the operand identity is not a new obligation preimage member. QSL
confirmed the two-variant operand identity for this seam while its FR-357 and
ADR-013 wording is being amended (QSL-641).

### Typed parameter declaration

A consumer that binds a call's named arguments to a scalar application's
ordered operands needs each parameter operand's declared name, which
`scalar_application_operands` does not return and no public accessor reads from
the node's `body`. `CheckedPackageV2::parameter_declaration(&self, node:
&CheckedNodeId) -> Result<CheckedParameterDeclaration,
CheckedParameterDeclarationError>` shall return it as typed Rust values, so the
consumer does not decode the package's `body`. The IR-owned public types, not
QSL wire types, are:

| Public type | Exposed fields or closed variants |
| --- | --- |
| `CheckedParameterDeclaration` | `node: CheckedNodeId`, `name: Box<str>`, `level: Box<str>` |
| `CheckedParameterDeclarationError` | Closed variants `UnknownNode`, `NotParameter` and `MalformedBody` |

These types are exported at the model root and listed in
[FR-019](../../model/functional/FR-019-rust-library-interface.md)'s Public items
table, which lists only exported items (FR-019-AC-5).

`node` is the supplied id. `name` is the value of the `"name"` binding's text
literal and `level` is the canonical non-negative decimal text of the `"level"`
binding's integer literal, both exactly as the reader admitted them under
"Parameter and compound-unit nodes, and the application dependency join". The
level is returned as that text so that the accessor adds no integer bound the
reader does not impose. The result exposes no `serde_json::Value`, body
fragment, occurrence or region, and no semantic type, function or value.

The accessor shall identify a parameter by node id alone and shall return no
enclosing-function identity. QSL [FR-092](ix://agent-ix/quire-spec-language/FR-092)
("Parameter nodes") gives a parameter node no `owner`, no `declaration` and no
function identity, so that its key does not depend on the function that
declares it: two functions whose parameters have the same name, level and
type share one node (QSL FR-092-AC-5), and the node therefore cannot say which
function it belongs to. The declared name is a property of the node, so the
same node id always returns the same name and level. Two parameter nodes of one
name that differ in level or type are distinct nodes with distinct ids; the
`level` member is what distinguishes the binders of a repeated name within one
scope chain (QSL FR-092: a reference to a parameter node of level `k` denotes
the one enclosing binder at level `k`).

An unknown id shall refuse `UnknownNode`. A node whose tag and form are not
`value`/`parameter` shall refuse `NotParameter`, whatever its body. A parameter
node whose body fails any of these checks shall refuse `MalformedBody`, a state
only a crate-internal post-admission mutation can produce: the body is an
aggregate of exactly the `"name"` and `"level"` bindings; the `"name"` binding is
a `text` literal typed at a `scalar_type`/`text` node whose value is an
identifier; the `"level"` binding is an `integer` literal typed at a
`scalar_type`/`integer` node whose value is a canonical non-negative decimal
string. The accessor shall not otherwise re-check the package.

Each refusal shall return only the typed error and no partial declaration.
The accessor shall not panic on any input. The accessor shall not change
package admission or the V2 reader's refusal order.

Binding concrete values is the caller's join and not an IR API: the caller
reads a call's operands with `scalar_application_operands`, reads each
`GraphChild` parameter's declared name with this accessor, and maps its own
named argument to the operand ordinal whose parameter has that name. The IR
provides no lookup from a name to a parameter node or an operand ordinal, no
value store, and no evaluation. A caller whose operands include two parameter
nodes of one name tells them apart by the returned `level` and node id.

### Typed relationship end of a reaches application

Measured at `origin/main` (IR-851, whose own text is a claim and was
re-measured). The reader resolves a `quire.op.model.reaches` application's
named relationship end while it admits the package (`check_relationship_member`
in `operations.rs`) and then drops the result. The end record
(`RelationshipEnd`: role, type reference and multiplicity), the relationship
declaration and the direction are `pub(super)`; `RetainedModels` keeps only the
declaration index, the object types' field tables and the operation parameter
types, so an admitted `CheckedPackageV2` holds no relationship end. The only
public multiplicity-bearing accessor is `model_object_fields`, which returns an
object type's edge FIELDS; a relationship end is not a field. A consumer that
must draw a finite number of targets for a reachability claim (the
bounded-harness consumer, CG FR-015) has no typed way to read the end's bounds,
and decoding the application's `body` and the selected domain package document
would duplicate the reader's derivation.

`CheckedPackageV2::relationship_end(&self, application: &CheckedNodeId) ->
Result<CheckedRelationshipEnd, CheckedRelationshipEndError>` shall return the
destination end the reader resolved for that application, as typed Rust values.
`application` is the node whose body root is the `quire.op.model.reaches`
application (the reachability expression), not the `state_clause` or other node
that encloses it; a consumer that holds a clause and an argument path supplies
the id of the application at that path. The IR-owned public types are:

| Public type | Exposed fields or closed variants |
| --- | --- |
| `CheckedRelationshipEnd` | `application: CheckedNodeId`, `relationship: CheckedNodeId`, `role: Option<Box<str>>`, `destination_type: CheckedNodeId`, `lower: u64`, `upper: Option<u64>`, `ordered: bool`, `unique: bool`, `forward: bool` |
| `CheckedRelationshipEndError` | Closed variants `UnknownNode`, `NotReachesApplication` and `UnresolvedEnd` |

These types are exported at the model root and listed in
[FR-019](../../model/functional/FR-019-rust-library-interface.md)'s Public items
table, which lists only exported items (FR-019-AC-5). The code change that
exports them adds that row; this specification change does not, because the
table lists exported items only.

**Selection.** The accessor reads the application's `operation.member`
(`declaration` and `name`) and resolves them through the same rule as
admission: the relationship named by `declaration`, then the end whose authored
role equals `name`. Admission's direction rule applies unchanged: when the
relationship's source end has role `name`, `forward` is true and the
destination is the target end; when its target end has role `name`, `forward` is
false and the destination is the source end (the destination rule of
`navigation_type`, FR-152).

**Retention.** The reader shall retain, for each selected relationship
declaration, the record admission consults: both ends' role, type reference and
multiplicity, the direction, and the derived model declaration node id of each
end's type (the id admission computes from the selected document's identity,
the end type's object-type or systems-interface form and the byte limit). The
accessor shall not recompute that id. The reader shall keep this record outside
package equality, shall charge no work beyond what admission already charges,
and shall not change its refusal order. The accessor shall read this record and
add no second derivation.

**Result.** `application` is the supplied id and `relationship` is the
`operation.member.declaration` node id the application names, so a consumer can
name the edge. `role` is the destination end's authored role, `None` when that
end carries none (a target role may be omitted, FR-038-AC-166); it is the role
the data holds and is never synthesised. `destination_type` is the identity of
the declared destination end's type, the model declaration node of the
destination object type (or systems interface) that admission derived; it is
the declared end type to which every operand conforms, not the type of a
particular operand, so a subtype operand does not change it. The node need not
be present in the graph: the reaches path computes it and does not look it up
(the result is Boolean), so the accessor returns the derived id whether or not
the graph holds that node. `lower`, `upper` and `unique` are the destination
end's declared multiplicity and `ordered` its declared ordering, with no change
of value or type: `upper` is `None` when the declaration is unbounded. The
result carries no member type, charge, population, bound, evaluation or source
text, and no `serde_json::Value`.

**What an admitted application yields.** The reaches branch refuses an
unbounded or an ordered destination end (FR-038-AC-255), a destination end of
another type than the receiver endpoint (AC-256) and a traversal against the
declared direction (AC-254), so no node that survives admission has such an
end. For every admitted reaches application the accessor therefore returns
`ordered` false, `upper` `Some` and a `destination_type` that is the receiver
endpoint's own declared type. The accessor does not itself re-check those
properties and promises nothing beyond them: it returns the declared `lower`,
`upper` and `unique` (for example `{0,3}` unique, `{0,3}` not unique, `{2,3}`,
`{0,1}`, `{1,1}`).

An unknown id shall refuse `UnknownNode`. A node whose body root is not a
`quire.op.model.reaches` application shall refuse `NotReachesApplication`,
whatever its tag, including a `quire.op.model.navigate` application over a
`relationship_end` member (a relationship-end application that is not a
reachability application), a relationship declaration node, a field read and an
application of any other operation. A reaches application whose
`operation.member` no longer resolves as admission resolved it shall refuse
`UnresolvedEnd`: a member that is absent, whose `kind` is not `relationship_end`,
whose `declaration` is not a node id or does not resolve to a retained
relationship, or whose `name` is not text, matches no role or matches both roles
of it. `UnresolvedEnd` is a catch-all in the way `MalformedBody` is for
`parameter_declaration`. Admission refuses each of those states
(`missing_declaration`, `ill_typed`), so `UnresolvedEnd` is reachable only from a
package that a crate-internal post-admission mutation changed; no reader-admitted
package returns it. Each refusal shall return only the typed error and no
partial end. The accessor shall not panic on any input, shall charge no work,
shall not change package admission or the V2 reader's refusal order, and shall
not mutate the package.

A relationship end is not an edge field. [FR-053](../../model/functional/FR-053-selected-population-binding.md)
and its AC-5 name `model_object_fields` as the sole authority for model
edge-FIELD multiplicity; that sentence is about fields and stays true, so
FR-053 is not amended. `relationship_end` is the authority for relationship-end
multiplicity and `model_object_fields` returns no relationship end. This
accessor exposes no population, population binding or bound; selecting a
population remains FR-053's `select_population`, none of whose items is
re-exported from the `quire-contract-ir` root. The population
of a reaches application is read by `reaches_population` ("Typed population of a
reaches application").

### Typed population of a reaches application

Measured at `origin/main` (IR-852, whose own text is a claim and was
re-measured). A `quire.op.model.reaches` application carries no population: its
body keys are the closed set `term`, `operator`, `operation`, `result_type` and
`arguments`, its `operation.member` is `{kind, declaration, name}`, and the
selected domain package's population has no maximum, so
[FR-053](../../model/functional/FR-053-selected-population-binding.md)'s
`select_population` can return `Finite` only for a `bounded_domain` /
`model_population` node whose `Type` occurrence a caller names. Nothing in the
admitted package tells a consumer which such node bounds the population a
reachability walks, and the IR must not choose one by object type: a
`Population<E>[N]` parameter type of the same `E` and `N` is the same
anonymous node (FR-053-AC-2), and a different `N` for the same `E` is another.
The producing language therefore names the node explicitly, and the reader
checks the name.

**The link.** A `quire.op.model.reaches` application MAY carry one additional
key in `operation.member`, `population`, whose value is the node id of one
`bounded_domain` / `model_population` node. The producer omits the key, and
never writes `null`, when the walked population has no authored bound; the
producer rule is owned by the QSL lowering and is not restated here. The V2
reader shall refuse a present `population` that is not a node id, `null`
included, as the reader check below states. The key is added to no other object
of the application: the application body, its `operation` object, its operands,
its `result_type` and its catalog arity are unchanged, and the key is not a
second operand. The member-shape rule that admits the key belongs to the QSpec
checked-package application rule (FR-322), which states the key and its checks in FR-322-AC-73, and their order in FR-322's
section "Population link of a `reaches`". Where
FR-322 states a refusal, the reader check below follows
its code, cause and pointer, except that a difference of outcome, code, pointer or
stage (FR-322's schema stage against this reader's operation stage, which has no
member schema, and FR-322's derived-shape check against this reader's lack of one)
is stated where it arises.

**The accessor.** `CheckedPackageV2::reaches_population(&self, application:
&CheckedNodeId) -> Result<Option<CheckedSelectedPopulationSelector>,
CheckedReachesPopulationError>` shall return, for the node whose body root is
the `quire.op.model.reaches` application, the selector FR-053 reads.
`application` is the application node, as for `relationship_end`, not the
`state_clause` that encloses it: one claim can hold several reachabilities over
different populations, so a claim has no one population, and a consumer
supplies the id of the application at the argument path it is bounding.
`CheckedSelectedPopulationSelector` is the FR-053 type; this requirement adds no
second selector type.

- When the application's `operation.member` has no `population` key,
  `reaches_population` shall return `Ok(None)`: the walked population has no
  authored bound, the caller holds the population declaration identity from the
  domain package, and the unbounded outcome of `select_population`
  (`Declaration`) is the caller's. `reaches_population` shall not infer a
  population from an object type, a count of population nodes or the absence of
  a node.
- Otherwise `reaches_population` shall return `Ok(Some(selector))` with
  `package_id` the package's own identity, `binding` the `population` node id
  exactly as the member holds it, and `occurrence` the `Type` occurrence of that
  node with the least ordinal. The selector is exactly what FR-053's `Admitted`
  source accepts. Because the reader check below refuses a population node whose
  `max` FR-053 cannot read, `select_population` over the selector returns
  `Finite` with the node's own `max`, `minimum` 0 and the object type the node's
  `semantic_type` chain names. `reaches_population` shall itself read no `max`,
  shall charge no work and shall add no second derivation of the object type.

`CheckedReachesPopulationError` has the closed variants `UnknownNode`,
`NotReachesApplication` and `UnresolvedPopulation`. When called with an id
absent from the graph, `reaches_population` shall refuse `UnknownNode`. When
called with a node whose body root is not a `quire.op.model.reaches`
application, `reaches_population` shall refuse `NotReachesApplication`,
whatever its tag, including a `quire.op.model.navigate` application. When called
with a reaches application whose `population` key no longer resolves as
admission resolved it, `reaches_population` shall refuse `UnresolvedPopulation`:
a value that is not a node id, a node id absent from the graph, a node that is
not a `bounded_domain` / `model_population` node, or a node with no `Type`
occurrence. Admission refuses each of those states, so `UnresolvedPopulation` is
reachable only from a package that a crate-internal post-admission mutation
changed. Each refusal shall return only the typed error and no partial selector.
`reaches_population` shall not panic on any input, shall not change package
admission, and shall not mutate the package.

**The reader check.** The V2 reader shall run the `population` check of a reaches
application as the last step of `check_relationship_member`, immediately after
that function's `result_type` check. Every check the reader makes today for the
application up to and including that `result_type` check therefore precedes it,
and `check_mode_type` and `check_leaf_count`, which run after
`check_relationship_member` returns, follow it. The structural dependency join
(AC-276) and the application-key stage run before every operation-stage check.
For `reaches` and `navigate` the catalog gives no mode and no leaves, so a mode
or a leaf on such an application is refused by the identity stage before
`check_relationship_member`, and `check_mode_type` and `check_leaf_count` can
never refuse such an application; they follow the `population` check without
effect.
A package without the key, and a package whose existing defect the reader refuses
today, therefore yield their present outcome at the present pointer and in the
present refusal order. The reader shall refuse with existing code and cause pairs
only, at the member's `population` pointer, and shall apply these rules in this
order:

- When a `quire.op.model.navigate` application carries a `population` key, the
  reader shall refuse `ill_typed`/`operator-ineligible`, before it resolves the
  value; this is the first population rule, so the refusal does not depend on
  what the value names.
- When the value of a reaches application's key is not a node id (`null`
  included), or is a node id with no node in the graph, the reader shall refuse
  `ill_typed`/`operator-ineligible`. An absent node is `ill_typed` and not
  `missing_declaration`: graph shape compares `dependencies` without resolving
  them, so an unresolved population is a failed population link and not an
  unselected declaration. For a value that is not a node id FR-322's
  schema stage refuses (the V2 schemas type the key as a node id,
  FR-322-AC-73); this reader has no member schema (`operation.member` is
  an opaque value), so it refuses at its operation stage with
  `ill_typed`/`operator-ineligible`, where FR-322's schema failure is
  refused `malformed_wire`. The difference is code and stage.
- When the node is not a `bounded_domain` / `model_population` node, the
  reader shall refuse `ill_typed`/`operator-ineligible`.
- When the node has no `Type` occurrence, the reader shall refuse
  `ill_typed`/`operator-ineligible`. FR-322-AC-73 is silent on this; the
  refusal exists because `reaches_population` returns the least `Type`
  ordinal.
- When the node's `max` is one that FR-053's `population_maximum` rejects (the
  member is missing or duplicated, is not an integer
  `literal`, or the value is negative or exceeds `u64`), the reader shall refuse
  `ill_typed`/`operator-ineligible`. A `max` member that is not a `binding` is not a
  valid body term, so the graph reader refuses it earlier as `invalid_semantic_graph`
  at the member (`/semantic_graph/nodes/N/body/members/M`) and it never reaches this
  check. The reader shall read `max` through
  `population_maximum` itself, so that a package the reader admits is one for
  which `select_population` is `Finite`; a `max` of 0 is admitted. For a `max` that
  is absent (a `max` that is not a binding is FR-322's absent case), negative or
  not an integer literal (FR-322's model-population shape
  rule in "Derived type forms and forwarding", whose negative-bound vector is
  RM-POP-MAX, FR-322-AC-62, and which FR-322's section "Population link of a
  `reaches`" applies), FR-322 refuses `invalid_package`/`invalid-derived-shape`
  at the population node, and the same check refuses a population node whose
  `semantic_type` is not `set<reference<T>>`. For a non-binding `max`, this reader
  instead refuses `invalid_semantic_graph` at
  `/semantic_graph/nodes/N/body/members/M` before the population check. This
  reader has no derived-shape check for a `model_population` node (that is
  FR-038-AC-131, still PLANNED), so it
  cannot produce that refusal, and until AC-131 lands it refuses an absent,
  non-literal or negative `max` `ill_typed`/`operator-ineligible` at
  `member.population`. It does not check the form of the node's `semantic_type`
  either (FR-053's `population_object_type` follows the first member target
  twice and checks neither form), so a node whose `semantic_type` is not
  `set<reference<T>>` but whose chain still names `E` is admitted here where
  FR-322 refuses it. The differences are in outcome, code, pointer and stage, and
  end when AC-131 lands; the code of FR-322 then governs, and no published vector exercises these
  cases.
- When the node's object type is not the declared type of the receiver end that
  the member's `name` selects (the destination end's type, which a reaches
  application shares with the receiver end, FR-038-AC-256), the reader shall
  refuse `ill_typed`/`operator-ineligible`. The reader shall derive the object
  type through FR-053's `population_object_type` itself, so that there is one
  derivation, and shall compare it by its derived model declaration node id, the
  id `relationship_end` reports as `destination_type`; a subtype of the declared
  type does not conform.

For an application of any operation other than `navigate` and `reaches`,
`check_relationship_member` does not run, and the application keeps its present
outcome for a `population` key in its member: a member kind whose shape is
closed today (`type_argument`, `temporal_interval`, `fairness`, `state_clause`)
refuses the key as it does today, and every other member kind (`field`, `position`,
`element`, `operation`, `profile_operator`) is opaque today and admits it unread. FR-322's section "Population link of a `reaches`" has the schema
refuse the key on every other member kind; this reader does not add that refusal.

No new refusal code, cause or pointer vocabulary is introduced.

**Dependencies.** The application-node dependency join
(`application_dependencies`) shall add the `population` node id to the
application's unique, digest-ascending dependency list when the application is a
`quire.op.model.reaches` or a `quire.op.model.navigate` application and its
member carries a node id there, as it already adds the member `declaration`. The
join shall add nothing for an application of any other operation. A package whose
`dependencies` omit that id, or list a population id that no such application
names, shall be refused by the join with its existing refusal
(`invalid_semantic_graph`, no cause), at the structural stage. A `population`
value that is not a node id is skipped by the join and refused by the reader
check above, as for `declaration`. Because the join precedes the operation
stage, a navigate application carrying a `population` key reaches the reader
check of the previous paragraph only when its `dependencies` list the id.

**What is not added.** The accessor exposes no bound, no member type, no
evaluation and no `serde_json::Value`; the bound stays FR-053's
`CheckedSelectedPopulation::maximum`. `CheckedSelectedPopulationSelector` stays
reachable by its module path only and no `selected_population` item is
re-exported from the `quire-contract-ir` root (FR-053-AC-5). The new error type
is exported at the model root and listed in
[FR-019](../../model/functional/FR-019-rust-library-interface.md)'s Public items
table by the code change that exports it, not by this specification change.
`relationship_end` is unchanged and its result carries no population. FR-038-AC-252's statement that a reaches node carries no charge or multiplicity member is unchanged by the `population` key, which is neither.

### Typed authored domains of composite equality and inequality operands

When supplied an admitted `quire.op.structural.eq` or `quire.op.structural.ne`
application node id, an occurrence belonging to that node, and a finite accessor work limit,
`CheckedPackageV2::composite_application_operands` shall return the application's
authentic operands and their package-authored domain structure through public
Rust types. The accessor shall admit only these two operation identities and the
package's admitted structural-kind operand families. Its eligibility does not
expand the owning checked-operation catalog. The existing scalar accessor's
clauses, four operations, types and errors remain unchanged.

The public call is
`composite_application_operands(&self, application: &CheckedNodeId,
occurrence: &CheckedOccurrence, work_limit: u64) ->
Result<CheckedCompositeOperands, CheckedCompositeOperandError>`.
The following are planned public IR-owned API types, not QSL wire types:

| Public type | Exposed fields or closed variants |
| --- | --- |
| `CheckedCompositeOperands` | `application: CheckedNodeId`, `occurrence: CheckedOccurrence`, `operator: CheckedCompositeOperator`, `operands: Vec<CheckedCompositeOperand>`, `consumed_work: u64` |
| `CheckedCompositeOperator` | Closed IR-owned variants `Eq` and `Ne`; the consumer maps these to its own equality operator vocabulary |
| `CheckedCompositeOperand` | `ordinal: u64`, `child: CheckedScalarOperandChild`, `semantic_type: CheckedNodeId`, `domain: CheckedCompositeOperandDomain` |
| `CheckedCompositeOperandDomain` | `Literal`; `Parameter { shape: Vec<CheckedCompositeShapeEntry>, positions: Vec<CheckedCompositeDomainPosition> }` |
| `CheckedCompositeShapeEntry` | `type_node: CheckedNodeId`, `kind: CheckedNodeKind`, `path: Vec<u32>`, `edges: Vec<CheckedCompositeChildEdge>` |
| `CheckedCompositeChildEdge` | `target: CheckedNodeId`, `ordinal: Option<u32>`, `name: Option<Box<str>>`, `optional_presence: bool`; an ordinal is present for a field, tuple position or inner edge, and absent only for path-neutral alias/bounded-domain forwarding; a name is present only for a record field; optional_presence is false except for the prescribed optional-presence record wrapper |
| `CheckedCompositeDomainPosition` | `key: CheckedCompositeDomainKey`, `type_node: CheckedNodeId`, `authored: CheckedAuthoredCompositeDomain` |
| `CheckedCompositeDomainKey` | `Node { node: CheckedNodeId, path: Vec<u32> }` only |
| `CheckedCanonicalIntegerBound` | Validated admitted canonical decimal text exposed by `as_str()`; construction restricted to reader/accessor content |
| `CheckedAuthoredCompositeDomain` | `IntegerRange { lower: CheckedCanonicalIntegerBound, upper: CheckedCanonicalIntegerBound }`; `UnboundedInteger`; `Collection { kind: CheckedCollectionKind, minimum: CheckedCanonicalIntegerBound, maximum: CheckedCanonicalIntegerBound }`; `UnboundedCollection { kind: CheckedCollectionKind }`; `Enum { ordered: bool, members: Vec<Box<str>> }`; `UnboundedDepth { declaration: CheckedNodeId, reentry_paths: Vec<Vec<u32>> }`; `Whole { kind: CheckedNodeKind }` |
| `CheckedUnsupportedCompositeOperand` | `InlineInteger`, `InlineNonInteger`, `ApplicationSubterm`, `NonStructuralType`, `NonliteralGraphValue` |
| `CheckedCompositeOperandError` | The twelve variants and payloads below |

None of these types shall expose `serde_json::Value` or a serialized term body.
The result's `operator` shall be `CheckedCompositeOperator::Eq` exactly when the
selected application's admitted operation identity is `quire.op.structural.eq`,
and `CheckedCompositeOperator::Ne` exactly when it is `quire.op.structural.ne`.
For a catalogued operation identity other than `quire.op.structural.eq` or
`quire.op.structural.ne`, on an application that passes the node, application
and occurrence checks, the accessor shall refuse `IneligibleOperator`. The
existing refusal order and variants are unchanged; no other successful operator
value is possible. The operator shall be returned together
with the operands; the consumer shall not need a separate operation lookup.
The accessor already resolves the application node under the graph lookup's
one-unit work charge before reading its operation identity. Selecting the typed
operator from that identity shall add no work charge and shall leave
`consumed_work`, the existing refusal variants, and operand order unchanged.
The result shall retain the actual application and supplied occurrence, and one
entry per `body.arguments` position in that order, with ordinals `0..n-1`.
Each entry shall retain its admitted semantic type and actual child identity.
Repeated references shall remain separate entries. A parameter reference shall
carry its own `GraphChild` identity, not its type or application identity.

A supported graph literal shall be a finite closed value subtree whose root has
an admitted structural-kind semantic type, in the record-value, tuple-value,
collection-value or option-value forms. A `union_value`, at the operand root or
within its graph subtree, shall refuse `UnsupportedDomain` with the enclosing
argument ordinal and its actual union semantic-type node; closedness shall not
turn that unsupported domain into `NonliteralGraphValue`. Missing or malformed
union type metadata shall first refuse `MalformedDomain` with its specified
authentic available type locus, never an invented union id. Its other descendants
may use the admitted literal and enum-value forms as well. A parameter reference,
application subterm,
unsupported value form or value-cycle within it shall refuse `UnsupportedOperand`
with reason `NonliteralGraphValue`; the enclosing argument ordinal is retained.
The accessor shall establish closure through a bounded structural walk under the
same work limit without evaluating an expression. The value tag alone shall not
establish closure. A closed graph literal shall retain its own `GraphChild` id
and `Literal` disposition with no introduced positions. It shall not copy or
reconstruct the literal value, or replace that singleton with its wider type's
free positions. QSL owns the downstream parity domain and encoding decisions.

For a record-value, tuple-value, collection-value or option-value subtree, one
referenced `expression`/`conversion` node with a body-root
`quire.op.numeric.narrow` application SHALL be recognized as a closed integer
literal only under the following complete rule. Its `operator` is `convert`;
`operation` has exactly identity `quire.op.numeric.narrow`, `laws: []`,
`leaves: []`, `mode: null`, and a `type_argument` member whose `declaration`
equals both the application's `result_type` and the expression node's
`semantic_type`. That id resolves to a `bounded_domain/integer_range` node
whose `semantic_type` resolves to `scalar_type/integer` and whose body is an
aggregate of exactly two bindings, `min` then `max`, each holding a
canonical decimal `integer` literal typed at that same scalar Integer node,
with `min <= max`.
The application has exactly one argument, a reference to a `value`/`literal`
node whose body is an integer literal typed at that same scalar Integer node
and whose `semantic_type` equals that node. The literal's canonical decimal
value must lie inclusively in the target range. The accessor SHALL compare
signed decimal text by sign, length and digits without narrowing through a
fixed-width integer or float, and SHALL retain the enclosing operand's
original `GraphChild`, `semantic_type` and `Literal` disposition, with no
introduced positions or copied value. This recognizes QSL FR-093's checked
`Coerce` spelling, including values beyond `i128`; it is not an evaluation of
an arbitrary numeric application or an admission rule for the reader.

If a node identifies as `quire.op.numeric.narrow` but any required wrapper,
target, literal type, canonical spelling, bound order or range-membership
condition above fails, the accessor SHALL refuse `MalformedDomain` with the
enclosing argument ordinal. Its `type_node` is the first valid typed id at
the failing locus: the wrapper's `semantic_type` for an invalid wrapper or
missing/mismatched target; the target range node for invalid endpoints or a
value outside them; the literal's declared type for a type mismatch, and
`None` when that locus has no decodable typed id. A valid wrapper whose sole
argument reaches a parameter, another application (including another narrow),
or a value cycle SHALL retain `UnsupportedOperand` with reason
`NonliteralGraphValue` and the first offending node's semantic type. A
different operation remains a nonliteral graph value. Tuple, Option,
collection and enum-value descendants retain their existing value-form paths;
the exception applies only where one of those paths reaches the exact wrapper.
The reader's admission and first-refusal order do not change.

The closed-value walk SHALL charge every graph resolution, visited node and
traversed term/member edge through its existing work meter before inspection.
Before validating or comparing a present endpoint, it SHALL charge its full
UTF-8 byte length once per logical visit even when comparison could stop at
the sign or an early digit; copying an endpoint SHALL NOT charge those bytes
again. It SHALL likewise charge the referenced literal's full decimal byte
length once before comparison. These bytes are examined, never copied into
the result. A failed charge SHALL return
`WorkLimit` with the first attempted cumulative charge before a later shape
or range error, without a partial result. The exact successful work limit
SHALL succeed and one less SHALL fail. The exception adds no public type,
second domain derivation, recursion ceiling, or harness draw value.

The accessor shall refuse top-level inline integer terms as `UnsupportedOperand`
with reason `InlineInteger`, and every other top-level inline literal kind as
`UnsupportedOperand` with reason `InlineNonInteger` (including `none`). It shall
fabricate neither a graph id nor an inline literal value. This is a defensive
accessor rule for integer terms: the owning catalog's `structural_kind` family
excludes Integer, and reader admission already refuses an integer operand of
`structural.eq` or `structural.ne`: both catalog entries require structural_kind.
No successfully admitted integer-inline equality or inequality fixture is
claimed here. The scalar accessor's eligible arithmetic literals remain governed
by AC-159 through AC-164; that does not allocate scalar equality support.
A future catalog/reader allocation that admits integer-inline equality must first
specify its exact authored decimal value and identity without `i128` narrowing.
Application subterms shall refuse reason `ApplicationSubterm`; nonstructural
operand types shall refuse reason `NonStructuralType`. This accessor does not
promise replay support for subterms merely because a reader admits them elsewhere.

For a parameter, the accessor shall retain a finite projection of its admitted
type structure using an explicit heap stack. Each shape entry shall retain its
actual type node, closed kind, path and ordered child edges. Record fields shall
use admitted body member order; tuples shall use positional order; Option and
collection elements shall use child zero. A direct record field reference names
that field's type. For the optional-presence form, the field binding's aggregate
contains exactly one `optional` binding referencing an Option type; the accessor
shall follow that reference as the field type, retaining `optional_presence=true`.
The aggregate and `optional` binding shall add no child-index segment. The field
ordinal adds one segment, and the Option inner edge adds zero: root
`List { head; tail?: List }` reenters List at `[1,0]`. A malformed optional wrapper
shall refuse `MalformedDomain`, not silently add a path segment or skip the field.
A direct reference to an Option type shall retain `optional_presence=false` and
follow the same field/inner path. Alias and bounded-domain forwarding shall
retain their source shape node and use a path-neutral edge.

The accessor shall root every introduced `Node` key at the actual parameter node,
with its child-index path, never at a type or application node. Shared types
reached through distinct paths shall retain those paths separately. The following
type-walk mapping is exhaustive over the eleven scalar and seven bounded-domain
forms; it does not assert that every form can occur in a reader-admitted
structural comparison:

| Type form | Introduced descriptor and source node |
| --- | --- |
| `scalar_type/boolean` | No position; retain its shape |
| `scalar_type/integer` | `UnboundedInteger` at this node, unless the same path is already represented by an enclosing `integer_range` |
| `scalar_type/enum` | `Enum` at this declaration node, retaining `EnumDeclarationPreimage.ordered` and `members` in preimage order; an unordered preimage is byte-ascending, while an ordered preimage retains its semantic order |
| `scalar_type/rational`, `scalar_type/decimal`, `scalar_type/float32`, `scalar_type/float64`, `scalar_type/text`, `scalar_type/dimension`, `scalar_type/unit`, `scalar_type/compound_unit` | One `Whole` position at this node, unless an enclosing bounded-domain Whole at the same path represents it |
| `bounded_domain/integer_range` | One `IntegerRange` at this wrapper with its exact admitted min/max; forward to its semantic type at the same path without adding `UnboundedInteger` |
| `bounded_domain/collection_bounds` | One `Collection` at this wrapper with exact admitted min/max and resolved collection kind; forward to its collection type without a second collection position, and visit its element at child zero |
| `bounded_domain/rational_range`, `bounded_domain/decimal_range`, `bounded_domain/float_rounding`, `bounded_domain/text_bounds` | One `Whole` at the outermost such wrapper on this path; retain path-neutral forwarded shape nodes, but suppress additional Whole positions at that same path |
| `bounded_domain/model_population` | One Node-keyed `Whole` at this population-typed leaf; no population member walk or `Population` key |

The public reader shall admit structural-comparison projection controls only
for the nine non-IEEE scalar forms and six non-IEEE bounded-domain forms in
this table, each inside a valid structural compared type. A compared type
reaching `scalar_type/float32`, `scalar_type/float64` or
`bounded_domain/float_rounding` shall instead refuse
`ill_typed`/`operator-ineligible` at `operation.leaves` before this accessor
can be called on an admitted package, as QSpec FR-322's operation contract and
AC-231 require. The three IEEE
descriptor mappings remain part of the exhaustive type-walk contract and
shall be verified at the isolated private type-walk seam, without claiming
public reader admission or an accessor result for those rejected comparisons.

A `text_bounds` leaf shall therefore introduce exactly one `Whole` position whose
`type_node` and kind identify the `text_bounds` wrapper, not a second position at
its underlying text node. The source identity retains the whole admitted type,
not just its length ceiling. Nested bounded wrappers at one path shall retain all
shape nodes and use the outermost applicable wrapper as that path's descriptor
source; contradictory wrapper/type structure shall refuse `MalformedDomain`.

For composite forms, Record and Tuple shall retain their ordered edges and walk
their children. Option shall introduce no position of its own and walk child
zero. Sequence, Set, Bag and OrderedSet shall introduce `UnboundedCollection` at
their type node when no collection-bounds wrapper supplies the position, and walk
child zero. Alias shall forward at the same path. Reference shall introduce one
`Whole` position at its type node, without walking an object or population.
Union payload projection remains outside this allocation and shall refuse
`UnsupportedDomain`. This is accessor eligibility, not a new admission refusal.

Recursion detection shall use the current ancestor chain, not a global visited
set. On a record/tuple declaration reentry, the accessor shall introduce one
`UnboundedDepth` key at that declaration's first-entry path and stop that branch.
Its descriptor shall retain all distinct encountered reentry paths, in
lexicographic `Vec<u32>` order, at that one key. Root List's Depth key is `[]`
and its reentry path is `[1,0]`; these are distinct facts. Root
`Tree { left?: Tree; right?: Tree }` has one Depth key `[]` and reentry paths
`[0,0]` and `[1,0]`. A shared nonrecursive sibling shall not acquire Depth.
A forwarding or Option/collection-only cycle with no record/tuple anchor shall
refuse `MalformedDomain`. The result shall contain each introduced key once.

Integer and collection endpoints shall retain their admitted canonical decimal
text. The accessor shall not convert them through `i128`, `u64`, `usize` or float,
order or repair them, or supply a maximum for an unbounded position. An authored
collection minimum remains visible. This projection shall not claim to construct
a valid QSL `FiniteBound` or to cover the whole domain of a Whole leaf.

The accessor shall return no partial result and use exactly this closed public
error enum. Every `ordinal` below is the enclosing argument position. A
`type_node: Option<CheckedNodeId>` is `Some` exactly when a valid typed target
identity is available, including a missing referenced type; absent or malformed
identity is `None`, never a fabricated node.

| Variant | Exact fields | Condition |
| --- | --- | --- |
| `UnknownNode` | `node: CheckedNodeId` | Requested application absent |
| `NotApplication` | `node: CheckedNodeId` | Requested node is not an application |
| `MissingOccurrence` | `application: CheckedNodeId, occurrence: CheckedOccurrence` | Supplied occurrence absent from application |
| `UnknownOperator` | `application: CheckedNodeId` | Missing or unknown catalog identity |
| `IneligibleOperator` | `application: CheckedNodeId` | Known identity other than structural.eq or structural.ne |
| `MissingChild` | `ordinal: u64, child: CheckedNodeId` | Valid typed reference target names an absent value child |
| `MalformedChild` | `ordinal: u64` | Reference target is absent or cannot decode as CheckedNodeId; no child identity is invented |
| `UnsupportedOperand` | `ordinal: u64, type_node: Option<CheckedNodeId>, reason: CheckedUnsupportedCompositeOperand` | The unsupported forms named above |
| `MalformedDomain` | `ordinal: u64, type_node: Option<CheckedNodeId>` | Missing type, malformed closed structure/optional wrapper or unanchored type cycle |
| `UnsupportedDomain` | `ordinal: u64, type_node: CheckedNodeId` | Union payload projection |
| `PositionOutOfRange` | `ordinal: Option<u64>, type_node: Option<CheckedNodeId>` | Argument ordinal or child path index cannot fit its specified wire width; ordinal is None only when that ordinal itself cannot fit |
| `WorkLimit` | `limit: u64, consumed: u64` | First unpayable charge or counter overflow |

For `UnsupportedOperand`, `type_node` shall identify the inline term's declared
type, the application subterm's result type, or the first offending graph value's
semantic type, whichever source caused that refusal. For `MalformedDomain`, it
shall identify the first missing or malformed type node; when an optional field
has no valid type reference it shall identify the containing record node.
`UnsupportedDomain` shall retain the actual union type node. A failed child-index
conversion shall retain its containing type node and enclosing ordinal; a failed
argument-ordinal conversion occurs before type resolution and shall retain None
for both optional fields. No other error variant carries an unstated locus field.

The accessor shall check node, application, occurrence and operation in that
order, then operands in argument order and type positions in depth-first child
order. Within an operand it shall recognize an inline term first, then an
application subterm, then resolve a graph reference and its semantic type before
family eligibility and parameter/literal projection. Thus a defensive inline
integer case returns InlineInteger rather than NonStructuralType. It shall charge before inspecting the charged item; an exhausted charge
shall take precedence at that point. Crate-internal post-admission mutations may
exercise conditions ordinarily rejected by the reader. Diagnostic prose shall
not classify an error. Recoverable input conditions shall not panic.

The accessor shall charge one unit per logical graph-node resolution attempt
(including a missing-node result), per visited shape/position, per traversed
edge, per retained path element and per UTF-8 byte copied as bound text or an
identifier. Lookup scan length, index hits and cache strategy shall not change
these logical charges. Repeated visits at distinct paths shall be charged for
each path. The finite admitted graph bounds lookup storage; the caller work limit
bounds semantic traversal and retained projection growth. The accessor shall use
checked counter, ordinal and path-index arithmetic. `consumed` shall be the
attempted cumulative charge; on counter overflow it shall be `u64::MAX` and shall
not authorize another visit. Zero work shall refuse before any node inspection.
A sufficient budget shall yield deterministic content and consumed_work; its
exact successful consumed_work budget shall succeed and one less shall refuse.
This limit is not a harness recursion depth, draw maximum or read-admission limit.
Package equality and reader refusal order shall remain unchanged.

IR shall expose only authored structure and introduced positions. It shall
neither accept, select, store nor validate harness draw values, B-4 substitutions
or CG proof ceilings. CG owns actual draw bounds; QSL owns domain admission,
Bounds ordering and the single parity encoder. The cycle-free model shall not
acquire a QSL dependency; owner integration belongs in the root bridge. This
accessor shall mint no obligation identity, incoming-key validator or coverage
verdict. Its declared type/position output does not prove harness coverage.

The `structural.ne` extension (IR-690) is implemented by the same accessor as
`structural.eq`; an admitted inequality application returns its authentic
operands. The accessor uses the same retained admission-derived lookup index and the existing
reader family/catalog/reference-resolution owners used for structural.eq.
The extension shall add no second domain derivation, caller-drawn bounds,
recursion ceiling, schema, encoder or public type. Each operation shall retain
its own actual application identity and selected occurrence; corresponding
operand types and child positions shall follow the same authored-domain rules
above, without treating inequality as a complement domain or negating bounds.
The extension shall preserve reader admission and first-refusal order and the
existing per-logical-visit work accounting. Equality behavior and scalar
accessor eligibility shall remain unchanged.

This closes only the declared operand-accessor eligibility gap for NotEqual in
[CG FR-033](ix://agent-ix/quire-contract-codegen/FR-033) AC-12 and
[QSL FR-358](ix://agent-ix/quire-spec-language/FR-358). It shall not claim that
these consumers, their runtime replay, whole-domain coverage or all scalar
Eq/Ne operations are enabled. Catalog/reader admission remains a prerequisite;
this extension shall not admit an operand or operation refused by that owner.

### Application node keys

[FR-046](../../model/functional/FR-046-derive-candidate-application-node-identity.md)
owns the implemented public pre-read candidate-key helper. The reader and helper
shall share the authoritative application preimage construction and canonical
encoder. This sharing retains the reader's existing authenticated all-node
graph-order context, refusal translation, and one work-unit charge before
preimage construction and hashing; it does not run public draft-context
validation in the reader or add its refusals or charges. A helper candidate is
not admission evidence, and a changed body retaining that key is still subject
to the reader's stale-key check.

A node whose `body` is an `application` term, and every `value`/`union_value`
node even when its body is an aggregate without an application, carries a `node_id` that is the
SHA-256 of the RFC 8785 bytes of its own preimage, `{version, node_tag,
semantic_form, semantic_type, declaration, recursion, body}`, with `version`
`quire.application-node/v1` (QSpec FR-322 `application_node_preimage`). The
preimage additionally contains `owner_model` exactly for a
`protocol`/`protocol_clause` node as
[FR-064](./FR-064-bind-protocol-clause-to-owner-model.md) specifies; it is
absent for every other application node. The
`recursion` member is `{size, ordinal}` of the node within its `recursion_group`,
the members of the group taken in graph order, and `null` outside a group. Each
`reference` in the body to a member of the node's own group becomes
`{term: "group_reference", ordinal}`, wherever the term stands (an application
argument, a binding value or an aggregate member), and a reference to a node
outside the group stays as it was. The reader re-derives the key for each such
node and refuses a retained key that differs as `invalid_package` with cause
`stale-node-key` at the node's `node_id`; the production-derived expected key
is retained by `expected_node_id()` while `locus` retains the supplied key. A
node whose `node_id` is that digest is not refused for its key. The
`value`/`union_value` selection includes a nullary member whose body is an
aggregate containing one named binding with an empty payload aggregate and a
payload-bearing member whose body holds only references. Neither construction
acquires an invented application, operator, operation identity or owner. A
`record_value` or `tuple_value` body with no application remains on its existing
structural-key path. The public FR-046 draft helper remains scoped to body-root
applications; the reader uses its authoritative preimage construction and
encoder for the additional `union_value` selection without treating a draft
helper result as admission evidence. This is the key check that FR-038-AC-35
exercises for a `dependency_reference` callee. This section fixes what the
preimage is (FR-038-AC-88); "Every identity digest is computed through
quire-canonical" fixes who encodes and hashes it, and FR-038-AC-89 pins that the
key is the one the reader derived before the move, so the two do not overlap.
The reader charges the selected node's existing one work unit before preimage
construction and hashes under the reader's existing canonical byte ceiling.
If encoding exceeds that ceiling, it returns the existing typed
`invalid_semantic_graph` refusal at `node_id` without an expected key; an
unpayable work charge returns `incomplete`/`work` before any later stale key or
encoding failure. Strict wire, package identity, owner join and graph-shape
checks retain their existing precedence; this key check precedes the derived
structural-key, declaration, frame, operation and generic body-reference edge
checks. Group membership is
the complete graph-order group, including structural nodes, and references to
members are rewritten at every body position, including nested payload
aggregate members and binding values. The reader does not add a second
preimage encoder or derive a key from the error text.

### Operation identity, laws, mode, member and operands

The `operation` of a node whose body root is an `application` term is read as its
own closed shape. A value that is not an object of that shape, such as a bare
string, refuses `invalid_semantic_graph` at the application's `operation`. Past
that read the reader looks `operation.identity` up in the operation catalog and
checks the application against the entry it finds. Each refusal below is
`invalid_package` unless it names another code, and is located at the member that
carries the defect. Each criterion below is observed at the operation check of
that node, which the unit tests call on a graph of the node and the nodes it
names, as FR-038-AC-68 observes the clause check; a package read refuses the
same fixtures earlier or by another code where the body grammar decides them
first.

QSpec FR-322 fixes the order of the checks, under `quire.native.diagnostics/v1`:
`unknown-operation`, `operation-class-mismatch`, `operation-law-missing`,
`operation-law-mismatch`, `operation-law-unselected`, `operation-mode-mismatch`
and `operation-member-mismatch`; then each argument application under the same
order; then `operation-mode-type-mismatch` and `operator-ineligible`; then the
leaves. The reader runs the first seven in that order, then the argument
`dependency_reference` walk, then `operator-ineligible` and then
`operation-mode-type-mismatch`, so it reverses the last pair: a stated deviation
from QSpec FR-322, not a settled rule, and no criterion below fixes the order of the
reversed pair. It checks an application at a node's body root, the only place one
stands: a nested application never reaches this check, because it is refused at
strict wire validation (`malformed_wire`, or for a nested `case`
`ill_typed`/`operator-ineligible` at its `operator`; "The flat wire").

QSpec requires `mode` and `member` as members of `operation`, each nullable.
When either member is omitted from that closed object, the reader SHALL refuse
`invalid_semantic_graph`, with no cause, at the application's `operation`
object, before the catalog identity, law, mode and member semantic checks.
Explicit `null` is a present wire member: it SHALL continue through those
semantic checks, which admit it where the catalog requires no mode or member
and refuse it with the existing mode or member mismatch where one is required.
The required-presence correction is implemented and exercised through
`CheckedPackageV2::read` by
`operation_mode_and_member_are_required_even_when_null_is_admitted`. The test
refreshes application/node and package identities for each omission, checks
shape refusal ahead of an unknown catalog identity, and restores explicit
`null` to demonstrate admission. The rule concerns the containing
`operation` object's `mode` and `member` only; it does not amend the distinct
operation-leaf shape or its mode checks. Earlier package-read stages retain
their existing precedence. The other deviations named in this section are
stated deviations, not scheduled defects.

- **Identity and operator class.** An identity the catalog does not list refuses
  `unknown-operation` at `operation.identity`, and an `operator` other than the
  entry's operator class refuses `operation-class-mismatch` at `operator`. An
  application that names a catalogued identity under its catalogued operator
  class, with the laws, mode, member and operands its entry requires, is not
  refused for either.
- **Laws.** An entry that catalogues a law role and an application that supplies
  no law refuses `operation-law-missing` at `operation.laws`. An entry that
  catalogues no law and an application that supplies one refuses
  `operation-law-mismatch` at `operation.laws/0`. A law whose `role` is not the
  role the entry catalogues at that position refuses `operation-law-mismatch` at
  the law's `role`, whatever its `definition` is, so the role is checked in its
  own right. The `definition` of a law of the right role is "Artifact
  references" (FR-038-AC-56, FR-038-AC-57).
- **Mode.** An entry that catalogues a mode kind and an application that carries
  no mode (`mode` `null`) refuses `operation-mode-mismatch` at `operation.mode`.
  For a mode kind the catalog lists as type-pinned (`rounding`, `text_profile`),
  the reader reads each fixed operand whose resolved family fits its position and
  whose type binds a value for that kind, a `bounded_domain` such as a
  `decimal_range` binding `rounding`, reached by a `reference` to it or by a
  `literal` whose `type` names it, and a mode value other than the bound one
  refuses `operation-mode-type-mismatch` at `operation.mode/value`. QSpec
  FR-322 also takes the pin from the result type and from a type with no binding
  (`exact` for `rounding`, the strict default of QSpec AD-005 and FR-140 for an
absent rounding spelling, which FR-322 applies to the pin); the reader checks the operand types that carry a
  binding only, a stated deviation. The modes of a leaf are "Operation leaves".
- **Member.** An entry that catalogues a member kind and an application that
  carries no member (`member` `null`) refuses `operation-member-mismatch` at
  `operation.member`. An entry that catalogues a `field` member, whose member
  names a field that its
  declaring record type does not declare, refuses `ill_typed` with cause
  `operator-ineligible` at `operation.member.name`. An application of
  `quire.op.model.reaches_field` whose member `declaration` names no node of the
  graph refuses `ill_typed`/`operator-ineligible` at `operation.member.declaration`.
- **Arity.** An application that supplies any number of arguments other than its
  entry's fixed operands, the entry admitting no rest operand, refuses
  `ill_typed`/`operator-ineligible` at `arguments`; with a rest operand, fewer
  than the fixed operands refuses the same way.
- **Operand families.** A `literal` argument resolves to its declared `type`, as a
  `reference` resolves to its target, and each takes part in the operand-family
  and mode-pin checks. An
  argument whose family does not fit its operand position refuses
  `ill_typed`/`operator-ineligible` at that argument. A clause application, the
  application of an entry whose result is `clause`, has family `clause` and fits
  no `boolean` operand position.
- **IEEE literal type and catalog result.** Independently of an enclosing
  operand position, a `float32_bits` literal requires a declared `type` in the
  `float32` family, and a `float64_bits` literal requires one in the `float64`
  family. A mismatch refuses `ill_typed`/`operator-ineligible` at that
  literal's `type`. For `quire.op.ieee.to_float32` and
  `quire.op.ieee.to_float64`, the catalog's `member` result form requires the
  `result_type` to name the same type node as `operation.member.declaration`;
  this is the IEEE instance of **Catalogued member results** below. The
  catalogued `member_family` constraint requires that
  type to have the `float32` and `float64` family respectively. A violation
  refuses `ill_typed`/`operator-ineligible` at the application's `result_type`.
  These checks occur at the existing operation-admission stage after strict
  wire and identity checks, with the published validation order and resource
  accounting unchanged. They do not admit an unlisted same-width IEEE
  conversion identity.
- **Catalogued member results.** For every catalogued entry whose result form is
  `member`, the entry's member kind selects the result rule. When that kind is
  `type_argument`, the application's `result_type` SHALL name exactly the type
  node named by `operation.member.declaration`. The closed `type_argument`
  member shape is `{kind: type_argument, declaration: NodeId}`: a missing,
  malformed or extra member, including `type_argument` in place of
  `declaration`, refuses `invalid_package`/`operation-member-mismatch` at
  `operation.member`, with the application node as locus. There is no alternate
  spelling. When the entry has a `member_family` constraint and the reader
  resolves a family for the declared type, that family SHALL fit the catalogue
  constraint. When the reader does not resolve a family for that type, this
  family check does not refuse it; the result/declaration equality still applies.
  For this check, `compound_unit`, `alias` and `model_population` have no
  ruled family under the FR-322 deferral below: generic bounded-domain
  forwarding of `model_population` to its set base does not settle its
  population family. In particular this preserves admission of
  `quire.op.quantity.convert` into a `compound_unit`. An equality or resolved-family
  mismatch refuses `ill_typed`/`operator-ineligible` at the application's
  `result_type`, with the application node as locus. After the earlier wire,
  identity, operation member-shape and operand checks, this result check runs
  before `result_inner`, state-clause result, model-member, mode-type and leaf
  checks. Thus a wrong result and wrong rounding-mode type on the same
  `quantity.convert` application refuse at `result_type` first; the published
  resource accounting is unchanged. Selection follows the catalogue entry
  shape alone (`result: member` and `member: type_argument`). Every other
  catalogued operation, including `model.lookup`, `model.all_instances` and
  other member kinds, keeps its prior admission outcome. `collection.fold` and
  `collection.reduce` have no `member_family` and receive only the
  result-type/declaration equality check. A catalogue entry with another
  result form is outside this rule. A member
  kind that disagrees with the wire member refuses earlier with
  `invalid_package`/`operation-member-mismatch` at `operation.member`.
  Other
  `member` kinds (`field`, `position`, `element`, `relationship_end` and
  `operation`) retain their member-specific result rules; their declarations
  need not themselves be the result type.
- **Same type.** A `same_type` constraint compares the type node each constrained
  operand resolves to, not the operand nodes: two operands of one type admit
  whichever nodes carry them, and the first operand whose resolved type differs
  from its predecessor's refuses `ill_typed`/`operator-ineligible` at that
  argument.
- **Operand families by form.** The operand family a node denotes is read from its
  tag and form alone: for `scalar_type`, the forms `boolean`, `integer`,
  `rational`, `decimal`, `float32`, `float64`, `text` and `enum`, each its own
  name; for `composite_type`, the forms `option`, `sequence`, `set`, `bag`,
  `ordered_set`, `record`, `tuple` and `reference`, each its own name;
  `expression`/`reference`, `reference`; `function`/`pure_function`,
  `function`/`predicate` and `function`/`recursive_function`, `function`;
  `model`/`object_type` and `model`/`systems_interface`, `object`; and
  `relation`/`population`, `population`; and `temporal`/`formula`, `temporal`
  (FR-038-AC-68; the reader gains this pair with the IR-503 code change, PR 253,
  so the unit test of this table gains that case there). Every other tag and form
  has none of its own. An enum's family is `enum` by form, refined to
  `ordered_enum` for an ordered enum (FR-038-AC-42). A node is type-shaped, so
  that an argument naming it names a type and not a value of its `semantic_type`,
  exactly when its tag is `scalar_type`, `composite_type`, `bounded_domain`,
  `relation` or `function`, or it is an `expression`/`reference` or a
  `temporal`/`formula`.

  The declared `scalar_type`/`unit` is also a quantity type: a reference to a
  value typed by that node has family `quantity`. The current closed
  `literal.value_kind` vocabulary has no quantity member, so a unit-typed
  literal is not a positive quantity case. An `aggregate` argument has family `aggregate` from its
  term, without resolving a type node. Neither may become an unchecked
  argument merely because its family is absent from the old table. The
  catalog's `same_family` comparison uses the resolved family of each named
  argument, including a quantity branch of `quire.op.control.if`; the twelve
  `quire.op.quantity.*` entries use their published operand positions. These
  are checks of the published operation catalog, not new operations.

  Three emitted shapes need an upstream FR-322 family ruling before this
  reader claims their admission semantics: QSL FR-094 emits a
  `scalar_type`/`compound_unit` as a quantity type and a
  `bounded_domain`/`model_population` over `Set<Reference<T>>`, while FR-322's
  current family paragraph names `unit` but not `compound_unit`, and its
  bounded-domain forwarding would classify that population node by its set
  base. FR-322 also does not say whether `composite_type`/`alias` forwards to
  its named enum; the present reader stops at the alias. These are not
  silently classified as `quantity`, `population` or `ordered_enum` by this
  amendment. The later implementation and its tests shall use the upstream
  ruling for each shape rather than infer a family from the node name.

### Operation leaves

For a catalog entry that names a leaf source, QSpec FR-322 has `operation.leaves`
list, in declaration order, one `{path, laws, mode}` entry for every `text`
leaf of the compared type: the first operand's
type, the inner type of the first operand, or, for a `set`, `bag` or
`ordered_set` result only, that result's inner type; any other result, a
`sequence` included, expects none. The reader walks that type through aliases
and bounded domains, the fields of a `record`, the positions of a `tuple` and
the inner type of an `option`, `sequence`, `set`, `bag` or `ordered_set`, and
counts its `text` leaves. A type with none expects none, so an empty `leaves`
is admitted; fewer supplied leaves than derived leaves (the text leaves and,
for a type that reaches itself, its recursion leaves) refuse as
`operation-law-missing` at `operation.leaves`.

The leaves supplied must be exactly the derived ones: the text leaves and the
recursion leaves described under Recursive compared types. The rules in this
and the next paragraph are about text leaves. The expected path of a
text leaf is its `field:<name>`, `position:<n>` and `inner` segments from the
compared type, and each text leaf carries exactly one law, of role `text_profile`
and a definition the operation catalog lists for that role. More supplied
text leaves than text leaves derived, a leaf at another path or out of declaration order,
and a leaf whose laws are not that one law each refuse as `invalid_package`
with cause `operation-law-mismatch`, at the first extra leaf, or at the `path`
or `laws` of the first leaf that differs. A leaf whose law the lock does not
select then refuses `operation-law-unselected` at that law's `definition`. An
entry that names no leaf source takes no leaves, and a supplied one refuses
`operation-law-mismatch` at `operation.leaves/0`. The expected paths are
derived one at a time, never listed ahead, and only after the leaf count is
settled; the number of text leaves is memoised per type node, where a node's
entry applies (Recursive compared types states where, for a type that reaches
itself). For a type none of whose nodes reaches an open composite, deriving them
costs the supplied leaves times the nesting depth times the width of a node's
fields, since a sibling holding no text is entered and skipped; for a type that
reaches itself the work budget alone bounds the cost. Each visit is
charged to the work budget. The component search that decides where a
memoised count applies costs one work unit per reachable type node, charged to
the same meter. A compared type that names a node that is not in
the graph or is not shaped as its form requires refuses `ill_typed` with cause
`operator-ineligible` at `operation.leaves`; so does a text leaf whose type,
through its aliases and bounded domains, binds no `text_profile`. A compared
type that reaches itself through a record or tuple is admitted (Recursive
compared types, below). Each type node whose entry applies is counted once
however many fields name it
and each visit is charged to the work budget, so nesting depth is bounded by
that budget and a type too large for it is refused as the budget is.

For an application's operation validation, resolving a structural type through
an `alias` or a non-`model_population` bounded domain shall charge one work
unit for each distinct forwarding node first entered. The charge precedes
reading that node's `semantic_type`; the terminal structural node contributes
no forwarding charge. The resolution memo belongs to that one application's
operation validation, including its union-arm constraint, inner-type checks,
result-inner selection, and both passes of its leaf walk. A later lookup of a
memoised forwarding node uses the recorded resolution without charging that
node or its already resolved suffix again. A second application has a fresh
memo, even when it names the same chain in the same package read. This charge
is additional to the existing visit and `text_profile` pin charges; a pin's
own first-visit memo and charge remain as stated above. A failed charge returns
the reader's typed `incomplete`/`work` outcome at the operation value being
validated, including from a union-arm predicate or a leaf edge; it shall not
be turned into an absent type, an `ill_typed` refusal, or an unchecked leaf.
The memoized count follows the existing rule that a type node's count applies
once even when many fields name it, and makes the work charged for structural
resolution proportional to the distinct forwarding nodes examined by one
application, rather than the number of repeated references to their suffixes.

For a record field, a direct `reference` value names the field's type. The
QSL-emitted optional-presence form instead has a field `binding` whose value is
an `aggregate` with exactly one member: a `binding` named `optional` whose
value is a `reference` to an `option` type node. The leaf walk follows that
reference as the field's type and descends the option through its usual `inner`
edge. The field contributes `field:<name>` and the option contributes `inner`;
the aggregate and its `optional` binding contribute no path segment. A direct
field reference to an option type remains a valid field value; its option also
contributes `inner` when walked.
An aggregate field value with an absent, extra or differently named member, a
member that is not a binding, or an `optional` binding whose value does not
reference an option type, is not a traversable record-field edge and refuses
`ill_typed` with cause `operator-ineligible` at `operation.leaves` during the
operation check. Each of these is permitted by the flat body grammar as a
record field value; a well-formed package carrying one reaches this check after
its identity members have been re-derived.

Once the shape and laws hold, each leaf's `mode` must be of kind `text_profile`
with a value the catalog lists, else `operation-mode-mismatch` at the leaf's
`mode` (absent), `mode/kind` or `mode/value`; a listed value other than the
profile the leaf's type pins refuses `operation-mode-type-mismatch` at
`mode/value`. An entry whose result is not a `set`, `bag` or `ordered_set`
expects no leaves under `result_inner`, so a supplied leaf refuses as the
extra leaf it is.

#### Recursive compared types

Equality, and every other operation that names a leaf source, over a compared
type that reaches itself is admitted, with the structural meaning QSL
FR-093-AC-11 gives it: two values are equal when their shapes agree and
their corresponding leaves are equal, however deep the values nest, and the
finite leaf list describes the type's unfolding rather than listing it. The
leaves a cyclic type expects are these. The derivation keeps the **open
composites**: the `record` and `tuple` nodes it is inside, after resolving
aliases and bounded domains, between entering one's fields or positions and
leaving them. An edge (a `field`, a `position` or an `inner`) whose target is
an open composite is a **reentry**. The
derivation does not follow a reentry: it contributes no text leaf, only the
recursion leaf below where text is reachable. Every
other path is walked and counted as before, so each distinct path to a text
leaf is one expected leaf; a record or tuple that two sibling fields both name
is not a cycle, and its leaves are derived under each path. The open set holds
the composites on the current path only, not every composite visited. A
composite from which no `text` type is reachable expects nothing, not even a
recursion leaf, so an equality over a recursive record with no text field
admits with `leaves` empty.

A composite's text-leaf count depends on which composites are open where it is
entered, so a node's memoised count applies at a use only when no record or
tuple reachable from that node is open at that use. With `X { t: Text; n?: Y }`,
`Y { u: Text; x?: X }` and `Wrap { y: Y; x: X }`, `Y` counts 2 under `field:y`;
under `field:x` the composite `X` is open and reachable from `Y`, so `Y` is
counted again, as 1 there, and `Wrap` has four text leaves. A count memoised
once per node regardless of the open set gives five, and one that turns a
reentry into 0 but keeps that memo gives three.

A cycle that passes through no record, tuple or union is not admitted. The
derivation also notes each option, `sequence`, `set`, `bag` and `ordered_set`
node it enters, with the number of open composites at that point. The note is
held only while the node is on the current path and is dropped when the walk
leaves it, so a node a sibling field names later is not a revisit (node keys
are content digests, and one `Option` of `Text` serves every optional text
field: `R { a?: Text; b?: Text }` admits). Reaching a node again while its
note is held, with no record or tuple entered between the two visits, refuses
`ill_typed` with cause `operator-ineligible` at `operation.leaves` (`T` as an
`Option` of itself, or a `Sequence` of itself). QSL never writes a leaf for one, and there is no
composite to anchor a recursion leaf's `d`. A cycle through a record, tuple or
union that also passes through an option or collection is a recursive composite
and is admitted as above; an independent cycle through none of the three
elsewhere in the same type is still refused.

A `composite_type`/`union` node is such a composite, and each of its members
anchors in the walk as a record's field or a tuple's position does (merged QSpec
FR-322 "Structural leaf walk", FR-322-AC-46 and FR-440-AC-8: a cycle through a union
is a recursion leaf and is admitted, never `operator-ineligible`; recursive unions
such as `List = Cons(Integer, List) | Nil` are core, and a union cycle counts in the
reader's visited set as IR-506 holds). The path of an edge into a union member's
payload is `member:<Name>` then `position:i`, `i` the index within the member's
ordered payload even for a single payload, and a reentry into an open union is a
reentry like any other: a cycle through a union is never record-free and never
refused for it, and expects one recursion leaf where a `text` type is reachable,
`d` counted as for a record. Merged FR-322's "Structural leaf walk" (#182, which
restored the entry #181 had dropped) lists a `recursion:<d>` entry for a record, a
tuple and a union cycle that reaches `text` (QSL FR-093, Text leaves, rule 3), as
this repository's FR-038-AC-70 through FR-038-AC-72 require.

At each reentry into a composite from which a `text` type is reachable, the
derivation expects one **recursion leaf**, `{path: p + "recursion:d", laws: [],
mode: null}`, where `p` is the path of the reentry edge and `d` is the decimal
count of segments the path held when the reentered composite was entered (QSL
FR-093, Text leaves, rule 3). A reentry into a composite that reaches no `text`
type expects none and admits none. The leaves derived are the text leaves and
these recursion leaves, in the order the walk reaches them, a recursion leaf at
the position of its reentry. A supplied entry whose last path segment is
`recursion:` followed by a decimal is a recursion leaf; every other entry is a
text leaf. A recursion leaf is required, so a type has one `operation.leaves`
and a meaning has one node key and one package id: the application node key
hashes the whole `operation`, leaves included, and a list written without a
required recursion leaf, or with one where none is derived, is refused as
below and is not admitted under a key of its own.

The checks run in this order. The supplied leaves are counted against the
derived ones, text and recursion: fewer refuses `operation-law-missing` at
`operation.leaves`, the same refusal a missing text leaf gets, since a missing
recursion leaf is a derived leaf the list lacks (so `Node` with its text leaf
alone, or with the recursion leaf alone, refuses it). One pass over the supplied
list then follows the derivation in order, and at each place the next entry must
be the leaf derived there. The first entry the pass cannot place refuses
`invalid_package`/`operation-law-mismatch`: a recursion leaf, wherever it
sits and whatever the cause (a wrong prefix, another `d`, a second one, one at a
reentry into a composite that reaches no `text` type, one where the place
holds a text leaf), at its `path`; a text leaf at another path at its `path`;
and a text leaf for which no place is left at `operation.leaves/<i>`, the
index into the supplied list, recursion leaves counted. A recursion leaf that
carries a law refuses at its `laws`. After the shape and laws of every leaf
hold and the law selection is settled, a recursion leaf that carries a mode
refuses `operation-mode-mismatch` at its `mode`, and the text leaves' modes
are checked as above.

The count and the path derivation are iterative over an explicit stack, as
they already were for a type that does not reach itself: a cycle costs heap,
never call stack, and no recursion on the call stack is used at any depth. A
reentry is found by the open set, not by exhausting a depth limit. Each node
visit, and each reentry edge examined, is charged one unit to the work budget
as every visit is; emitting the recursion leaf there, and consuming it in the
pass, costs no unit beyond the one that reentry edge was charged. A cyclic type whose leaves or width exceed the budget
refuses as the budget does (`incomplete` for `work`), so the cost of a type
such as `n` records that each hold text and an optional field of every other
record is bounded by the budget and not by the `(n - 1)!` leaves it unfolds
to. The memoised count applies where the paragraph above says. No depth
limit applies to the cycle itself, since each composite is open at most once
at a time and a path is no longer than the composites it passes.

This reader does not match the reference reader in four respects, which
remain open. The QSpec reference reader finds a compared type undecidable
when its walk reaches any node a second time, and refuses it
`operator-ineligible`, and FR-322 is silent on cycles (QSpec STD-129 asks for
the ruling and is unruled); this reader admits a compared type that reaches
itself through a record or tuple as above, which QSL FR-093-AC-11 requires and
which QSL's lowering emits. This is a deviation from the reference reader, not
an alignment with it. The recursion leaf is QSL's ADR-013 QC-24 proposal; QSpec
has not adopted it (STD-129 asks whether it is) and QSpec's published leaf
segment schema rejects the `recursion:<d>` segment. This reader requires it
where text is reachable along a cycle, as QSL FR-093's Text leaves rule 3 writes
it, so every package QSL's lowering writes admits; that requirement is a stated
deviation from QSpec's schema, held until that ruling. A `float32` or `float64` leaf counts as no text leaf, where the
reference reader finds the type undecidable. A compared type this reader cannot
resolve from the first operand (an operand it does not type, such as an untyped
literal or an aggregate), and a
`set`, `bag` or `ordered_set` result whose inner type does not resolve, are not
decided and their leaves are not checked, where the reference reader refuses
`operator-ineligible`. Refusing them would refuse every operand this reader
does not yet type, which is a wider set than the reference reader's
undecidable types, so that is left until operand typing is complete.

The planned IR-491 residual rule applies only after the earlier package,
reference, operation and operand checks have succeeded. The catalog names
exactly the following leaf-source cases; the reader SHALL use the selected
type in this table and SHALL NOT treat failure to resolve a required type as
an empty leaf list:

| Catalogued operation | Leaf source | Selected compared type after earlier checks |
| --- | --- | --- |
| `structural.eq`, `structural.ne` | `operand:0` | The first operand's declared type: a `literal`'s `type`, a `reference`'s target when it is type-shaped, or otherwise that target node's `semantic_type`. A reference to an expression node with an application at its body root selects that node's `semantic_type`, which must equal the body's `result_type`, or for a record result conform to it ([FR-347](./FR-347-admit-conforming-records-as-operation-call-arguments.md) "Result positions"); an application cannot occur inline in `arguments`. Its family must already fit `structural_kind`, and `same_type` must hold. |
| `collection.contains` | `inner:0` | The element type of the first operand's `sequence`, `set`, `bag` or `ordered_set` type, through permitted alias and bounded-domain forwarding. The first operand must already fit `collection_kind` and its second operand must satisfy the catalogued `inner_type` constraint. Selection adds no leading `inner` path segment. |
| `collection.set`, `.bag`, `.ordered_set`, `.map`, `.flat_map`, `.flatten`, `.convert` | `result_inner` | The result's element type only when the admitted `result_type` resolves to `set`, `bag` or `ordered_set`. A valid `sequence` result expects no leaves, even if its element type is text. Selection adds no leading `inner` segment. |

A `binding` or `aggregate` as operand 0 has the catalogued `binder` or
`aggregate` family, so it cannot satisfy the first two rows' operand family;
a `dependency_reference` outside a call's callee position is rejected by the
earlier reference walk. A `frame` or `abstraction_relation` is not a value of
the required family. These terms are not an admitted positive control for an
unresolved compared type. A malformed `reference`, `literal` or `application`
is likewise subject to its earlier graph, literal or operand check. After
those checks, a required compared or set-like result element type that still
cannot be resolved refuses `ill_typed`/`operator-ineligible` at
`operation.leaves`, with either an empty or a supplied leaf list. A valid
non-set-like result, including a `sequence`, has an empty expected list; a
supplied leaf refuses `operation-law-mismatch` at its first extra entry.
The compared type's own node and member walk, text-profile pin, law and mode
rules above remain in force; this selection rule does not relax their work
charges or refusal order. Leaf-list construction is separate from type
eligibility: a `structural.eq`, `structural.ne` or `collection.contains`
compared type that contains Float32 or Float64 at any depth refuses
`ill_typed`/`operator-ineligible` at `operation.leaves`, even though the IEEE
node contributes no text leaf. A `result_inner` operation yielding a
`set`, `bag` or `ordered_set` whose element type contains IEEE is likewise
ineligible under QSpec FR-144's collection-type rule; it does not admit by
having zero text leaves. A valid `sequence` of IEEE values yielded by a
`result_inner` operation still expects no leaves. QSL FR-093's rule that a
float contributes no *text* leaf is consistent with its separate upstream
equality and collection-type admission checks.

### Frame bodies

A `state` node of `semantic_form` `frame` carries a frame body, and only a
frame body: `{"term": "frame", "modifies": [...], "creates": [...],
"deletes": [...], "deletes_scoped": [...]}`, with all four members required and each a
`uniqueItems` array per the wire schema: `creates` and `deletes` of node
keys, `modifies` of the `FrameModifiesEntry` values and `deletes_scoped` of the
`FrameDeletesScopedEntry` values
[FR-040](./FR-040-admit-frame-entries-and-state-clauses.md) defines. The reader shall validate a
`state`/`frame` node's `body` against this shape alone, a
`correspondence`/`abstraction_relation` node's `body` against the shape of
[FR-346](./FR-346-admit-abstraction-relation-body.md) alone, and every other
node's `body` against the semantic-term grammar alone. A frame body that omits one of
the four members, carries any further member, or does not carry this closed
shape refuses as `invalid_semantic_graph` at the frame body; so does a node of
any other tag or form whose body carries the frame shape. Within one member
array, an entry that repeats another entry of the same array refuses as
`invalid_semantic_graph` at the second occurrence's entry, and a `creates`,
`deletes` or `modifies` entry that is not a node key (or, for `modifies`, a
`FrameModifiesEntry`) refuses the same way at that entry (the path of the
entry under `semantic_graph.nodes.body.modifies`, `.creates` or `.deletes`); an entry
naming a node key outside the reader's node-identity domain refuses as
`digest_domain_mismatch` at that entry. `deletes_scoped` entries are never node
keys: the node-key and node-identity-domain clauses do not apply to them, and
FR-040 owns their shape, repeat and order refusals. Four empty members are
admitted. The shape, ordering, conflict and state-step rules of the `modifies`
field entries' `scope` and of `deletes_scoped` are
[FR-040](./FR-040-admit-frame-entries-and-state-clauses.md)'s.

The frame-body-semantics stage charges no work of its own: every entry it
walks was already parsed, shape-validated and charged once by the per-node
body-validation loop that ran before it, so its cost is already bounded by
the same work limit that bounded that earlier walk. A caller cannot grow
this stage's cost independently of an already-charged input.

The declaration stage ends with two name checks. A node carrying both a
`declaration` and a nominal identity preimage that fixes a name (an enum
declaration, a dimension or a declared unit) shall declare exactly that
preimage's `qualified_declaration`, else the reader refuses as
`invalid_package` with cause `declaration-nominal-mismatch`. Names are unique
within the types, dimensions and units namespaces: a unit, dimension and type
may share a qualified name. Two nodes in one namespace declaring the same
`qualified_name` refuse as `ambiguous_declaration` with cause
`ambiguous-name`, located at the later node in node-id order. QSpec FR-322
orders both before any frame
or operation refusal but not against each other; this reader checks the
nominal join first, each at the first offending node in ascending node-id
digest order.

Once every node's identity is known, the reader shall evaluate frame-body
semantics as its own stage, in reader order graph-shape, stale-key,
declaration, frame, operation — immediately after declaration checks and
before the graph's dependency and body-reference edges are resolved, so a
frame's own `dependencies` entry naming no real node is this stage's own
refusal rather than the graph's generic unresolved-reference refusal. A
`creates` or `deletes` entry names a declared dependency of `model` form
`object_type` or `process`; a `modifies` entry's eligibility and field-name
resolution are [FR-040](./FR-040-admit-frame-entries-and-state-clauses.md)'s. This is the closed eligibility table over the family/form pairs
the contract's node taxonomy admits; a member and an entry's family/form pair
outside it is never admitted, regardless of the entry's own grammar validity.
An entry naming a digest that is not among the frame node's own
`dependencies` — including one declared but resolving to no node anywhere in
the graph — refuses as `missing_declaration` with cause `missing-name`,
located at that entry's own key. An entry that does resolve against a
declared dependency, but to a family/form pair the entry's member does not
admit, refuses as `invalid_model_binding` with cause `malformed-declaration`,
located at that entry's own key. Within one member array, entries shall
appear in strictly ascending order of their order key (for a `creates` or
`deletes` entry its digest, for a `modifies` entry FR-040's order key); an array not in that order refuses
as `invalid_semantic_graph` at the frame body path, located at the frame node
itself, carrying no cause.

A frame node carrying more than one defect refuses for exactly one of them,
selected by one precedence: any meaning-join defect (`missing_declaration` or
`invalid_model_binding`) outranks a canonical-order defect outright, so a
frame whose member order is wrong and whose entries are also ineligible
refuses for the meaning-join defect, never the order defect. Among
meaning-join defects, the member the defect's entry sits in decides first —
`modifies` before `creates` before `deletes` — and, within one member,
ascending entry digest breaks the tie, except that a `conflicting-binding`
defect of two scoped `modifies` entries ranks after every other meaning-join
defect in any member (FR-040-AC-25); the selected defect therefore never
depends on where in its array an entry sits or on which member an author
wrote a defect into. A package carrying more than one defective frame node
refuses at the frame the reader reaches first in ascending `node_id` digest
order, reporting only that frame's own selected defect; a later frame's
defect, however it would otherwise rank, is never reached or compared
against an earlier frame's.

When a caller lowers requested items, the lowerer shall charge work per request and, for each visited reachable node, per node, per body term and per successor edge, return
`invalid_input` for an absent node key, `unsupported` when any reachable node's
tag is outside the profile, `requires_bound` when the profile requires bounds
and a reachable type is unbounded at a position or is recursive (both defined
below), and `failed` at the work limit. A lowered record
carries the node, its exact source-map entries, semantic type, reachable
dependency keys, bounding domain keys, reachable claim keys and a
`quire.contract-ir.semantic/v1` digest. A non-lowered record carries no node.
Reusing an admitted graph or closure does not discount any request's work:
each request reports the same consumed counter and terminal outcome it would
report when traversed independently. A shared closure is computed once for
each distinct graph component needed by a lowering call, while the records
and package still carry each request's full exact dependency population.

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

A reachable type is unbounded by form exactly when its family and semantic form
are a `scalar_type` of form `integer`, `rational`, `decimal` or `text`, or a
`composite_type` of form `sequence`, `set`, `bag` or `ordered_set`. No other
family and no other form of those two families is unbounded by form, so `boolean`,
`float32`, `float64`, `dimension`, `enum`, `option`, `record`, `tuple`,
`alias` and `reference` raise `requires_bound` only as a recursive type. A
quantity is its own class, not one of the eight forms, and is checked apart from
them.

A quantity is a `scalar_type` of form `unit` or `compound_unit`, or a
`bounded_domain` whose base chain ends at one. Its magnitude is an unbounded
rational that no bounding form makes finite, so a consumer classifies it as
unbounded (QSL FR-097-AC-2: a quantity takes no finite bound). A quantity is
checked by a separate position predicate and never through the typed path
below: `requires_bound` raises it only when a position (below) is typed at a
quantity, whether directly or through a `bounded_domain` base chain, and then
names the `unit` or `compound_unit` node at the end of that chain. The base
chain continues through every `bounded_domain` form other than
`model_population`; a chain that returns to a domain already on it ends without
a quantity. No `bounded_domain` covers it. A `unit` declaration node lowered as the requested
node, the unit nodes a `compound_unit` lists in `dependencies`, a unit named
only by a `literal.type` annotation, and an `application`'s `result_type` that
is a quantity therefore do not raise it, and no node is a quantity position
merely because it types at, depends on or is requested as a unit.

`requires_bound` tests only a type that a reachable node is typed at: a node
other than the type itself names it through `semantic_type`, `dependencies` or
a body `reference` target or `application` `result_type`, or it is the
requested node. A type reached only as a `literal.type` annotation, such as the
`text` node QSL FR-092 gives the name literal of every parameter (FR-322
requires each literal to carry a `type`), is in the closure and in
`dependencies` but is not a value's type, so it never raises `requires_bound`.

A type that is unbounded by form is unbounded at a position when either of these
holds, and a `bounded_domain` elsewhere in the closure never covers a position:

- A `composite_type` node names it as an element or field type.
- A `value`/`parameter` node's own `semantic_type` is it.

So `{n: Integer, k: Int[0,9]}` raises `requires_bound` naming `integer` although
the `integer_range` for `k` is over the same `integer` node; `Sequence<Integer>[0,3]`
does too; and `(x + 1) + n` over `x: Int[0,9]` and `n: Integer` raises it for
`n`. A `bounded_domain`'s own base type, the literals of its bounds, an
`application`'s `result_type` and its own `semantic_type` are not positions, so
`Sequence<Boolean>[0,3]` and `x + 1` over `x: Int[0,9]` lower under a
bounds-required profile. A type that is unbounded by form and is typed at by any
other reachable node, or is the requested node, but is named at no position is
bounded when the closure holds a reachable `bounded_domain` node
whose `semantic_type` is that type's key.

A `scalar_type` or `composite_type` node that declares a `recursion_group` is a
recursive type, whatever its form. Its depth is unbounded and no `bounded_domain`
form bounds depth, so it raises `requires_bound` naming itself, whatever bounds
its fields carry. The label alone decides: the reader admits a label on a node
outside any cycle, and a recursive type reached only as a `literal.type`
annotation also raises it.

Where more than one reachable type raises `requires_bound`, the record names the
least offending node key overall, positional or recursive.

A lowered record's `dependencies` shall be every node key reachable from the
requested node excluding the requested node itself, in ascending key order.
The lowerer shall derive `bounds` and `claims` as filtered views of that same
reachable set — `bounded_domain` nodes and `claim` nodes respectively — and not
as a partition of it: every key in `bounds` and every key in `claims` also
appears in
`dependencies`. The `quire.contract-ir.lowered-node/v1` preimage carries all
three lists as written, so an independent re-derivation of `ir_id` that treats
the three as disjoint disagrees byte for byte.

### Native String, UUID and Timestamp in the V2 reader

The reader SHALL retain `scalar_type` forms `string`, `uuid` and `timestamp`
as three distinct types, separate from `text`, when a selected Semantic IR
document declares a field, parameter or result with the corresponding native
`typeRef`. It SHALL retain the same distinction for literals of `value_kind`
`string`, `uuid` and `timestamp` and for the families checked at an application.
The selected document's existing `typeRef` member supplies the declaration;
this rule adds no new source-document member or second selection path.

For each family the reader SHALL recognize only the catalogued `eq` and `ne`
identities as native comparisons. They take two operands of that one family,
yield `boolean`, and carry no law, mode, member or leaves. `string` compares
exact Unicode scalar sequences without normalization or case folding;
`uuid` and `timestamp` compare their canonical payloads. The reader SHALL
refuse a foreign family in either operand position of each native comparison,
or an ordering or arithmetic identity over any of the three families, as
`ill_typed`/`operator-ineligible` at the first offending argument.
A `text_profile` law on a native comparison SHALL refuse as
`invalid_package`/`operation-law-mismatch` at that law, and a `text_profile`
mode SHALL refuse as `invalid_package`/`operation-mode-mismatch` at the mode.
A `string` operand of `quire.op.text.eq` SHALL refuse as
`invalid_package`/`operation-mode-type-mismatch` at the mode value; it does
not acquire a text profile from its spelling.

A native literal's `type` SHALL resolve to its own family. A mismatch SHALL
refuse as `ill_typed`/`operator-ineligible` at that literal's `type` before
its `value` is examined. For a correctly typed literal, `string.value` SHALL
be a JSON string retaining its exact Unicode scalar sequence; `uuid.value`
SHALL be lowercase ASCII hexadecimal in `8-4-4-4-12` groups; and
`timestamp.value` SHALL be the canonical decimal spelling of a signed
128-bit nanosecond count, from
`-170141183460469231731687303715884105728` through
`170141183460469231731687303715884105727`. An incorrect value shape or
noncanonical spelling SHALL refuse as `invalid_package`/`invalid-value` at
that literal's `value`, without repair or coercion. Earlier wire, graph,
identity, declaration, frame and resource refusals retain their existing
precedence; the native literal and operation checks run in the existing
operation step under the same bounded work accounting.

### Quantity literals and checked-unit type references

The V2 reader SHALL admit a `literal` with `value_kind: "quantity"` only when
its `type` names a `scalar_type`/`unit` node. That node supplies the `quantity`
operand family. The literal `value` is a canonical exact rational magnitude:
an integer `n` or a reduced fraction `n/d` with a positive denominator,
following QSpec FR-322's scalar-literal rule. A literal of another kind typed
by a unit, or a quantity literal typed by another family, SHALL refuse
`ill_typed`/`operator-ineligible` at the literal type. A quantity magnitude
outside that canonical grammar SHALL refuse `invalid_package`/`invalid-value`
at the value. No conversion, rounding or normalization repairs the wire.

A selected model document's field, operation parameter or operation result
`typeRef` may name `ix://quire/unit/<name>`, where `<name>` is the checked
package unit declaration's qualified-name segments joined with `/`. The
reader SHALL resolve it to that unit's existing `scalar_type`/`unit` node,
through the same multiplicity and presence tables as other member types; it
introduces no new type node. An unresolved unit name SHALL refuse
`missing_declaration`/`missing-name`, and an argument typed by another unit or
family SHALL refuse `ill_typed`/`operator-ineligible`.

Declaration names SHALL be unique within each of the `types`, `dimensions`
and `units` namespaces. The same qualified name may occur once in each
namespace. Two nodes of one namespace with that name SHALL refuse
`ambiguous_declaration`/`ambiguous-name` at the later node in node-id order.
The reader SHALL decide this before operation admission.

### Retain a production-derived expected node key

When the reader refuses a graph node as `invalid_package` with cause
`stale-node-key` because its retained `node_id` differs from a key the reader
actually derived, `CheckedPackageRefusal` shall privately retain that derived
identity and expose `expected_node_id() -> Option<&CheckedNodeId>`. The value
has the fixed `quire.checked-semantic-node/v1` domain and a 64-character
lowercase SHA-256 digest. It comes from the structural-key or application-key
computation already performed at that refusal site; retaining it adds bounded
bytes, no second derivation and no new work charge. `locus` remains the original
stored node id, never the expected one.

When no key is derivable, the accessor returns `None`, even if the refusal has
the same `invalid_package`/`stale-node-key` pair. That includes a malformed
closed shape; any of the eight self-typed forms (`boolean`, `integer`,
`reference`, `option`, `set`, `bag`, `sequence`, `ordered_set`) whose
`semantic_type` differs from its retained `node_id`; a non-self-typed shape
whose required semantic type is inconsistent; `collection_bounds` whose
`semantic_type` names no collection type; and a model-declaration node rejected
by FR-322 step 2 for its fixed-member shape. Earlier wire/schema refusals and
canonical encoder failures likewise retain no expected key. Callers shall
fail closed on absence rather than infer a key from the cause.

The field participates in the existing derived `Clone`, `Eq` and `PartialEq`:
refusals differing only in their expected keys compare unequal. Derived `Debug`
may print the key. The type has no `Display` or serialized refusal form; this
change adds neither. `code`, `path`, `cause` and `locus` remain public and
mutable, and a caller may clone a refusal. Consequently, the expected key is
authentic only when paired with the original `locus` and other fields returned
by the reader; a clone whose public fields the caller changes does not carry
an authenticated substitution instruction. The key is never inserted into a
document pointer or `locus` text.

The six production refusal constructors in `checked_package/common.rs` shall
initialize genuine absence through a crate-private construction path. The
private expected-key field forbids downstream struct literals. IR's six
integration-test refusal literals (five shared helpers and one selected-model
number helper) shall become independent, test-authored expected values checked
field by field against actual reader refusals, including the new accessor;
reader output compared only with itself is not an oracle. This preserves about
126 helper assertions across 13 test files, including the two current
struct-update expressions, without a public synthetic-refusal constructor.
Known downstream test literals migrate with their owners at their IR lock
update: `quire-contract-codegen/src/kani/generate/outcome.rs` and
`quire-driver/tests/drive.rs` can use a genuine malformed-byte refusal with no
path; `quire-contract-codegen/src/oracle/scalar/mod.rs` requests the impossible combination
`invalid_semantic_graph` with no path and needs a redesigned assertion. These
are test migrations, not an IR compatibility API. Note to the CG IR-664 and
driver lock-refresh owners: verify their QSL and driver forwarding paths retain
an authentic typed key or absence without diagnostic-prose parsing. Existing
code, cause, pointer, locus, first-refusal order, byte/work
limits and the public key-derivation surface remain unchanged.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-038-AC-1 | Each positive V2 package fixture this repository builds from its own public vocabulary admits; an unknown, empty, absent or malformed `contract_version` refuses as `unknown_contract_version` or `malformed_wire` before any version-specific decoding; and the strict parse (duplicate-member, noncanonical) refusals occur before a version is selected. | Test (TC-048) |
| FR-038-AC-2 | The reader refuses malformed, duplicate-member, unknown-member, noncanonical, stale-dependency, cross-domain digest, unknown required capability, unsupported node tag, invalid graph and invalid source-map inputs with exactly those codes, and every authored adverse structural mutation returns its declared outcome, before exposing a package; a domain package selection or model owner carrying `authority`, `revision`, `export` or `version` refuses as `unknown_member`, a domain package selection outside `sha256-jcs` as `digest_domain_mismatch`, one with no supplied document as `missing_import` with cause `missing-selection`, and a `model_export` semantic form as `invalid_semantic_graph`. In an otherwise valid package, replacing `/semantic_graph/nodes/0/occurrences/0/role` with the literal string `unknown field` refuses as `malformed_wire`, not `unknown_member`: a value outside the closed occurrence-role vocabulary is a malformed value even when its text resembles a decoder error. A genuine extra member still refuses as `unknown_member`, and a required member of the wrong kind still refuses as `malformed_wire`; the value's text never selects the refusal code. | Test (TC-048) |
| FR-038-AC-3 | Exact byte, node, edge, occurrence, diagnostic and work limits admit a package; each one-over limit returns `incomplete` with that limit kind, the limit and the consumed counter and no package; no limit kind is a depth (FR-038-AC-117). | Test (TC-048) |
| FR-038-AC-4 | The package id is derived from the seven wire selections plus every graph node with only occurrences excluded. Published positive identities are preserved. A full nominal-body mutation with the nominal key kept changes the derived id and refuses stale_dependency/content-mismatch at /package_id/digest under the original id; node reordering changes identity while occurrences remain excluded. | Test |
| FR-038-AC-5 | Every nominal preimage of the in-repo nominal fixture re-derives the node key it is keyed by and round-trips its own wire form; every authored invalid nominal mutation, an absent or wrong preimage, and each retained-preimage change of enum case, `semantic_type`, dependency or unit target refuses as `invalid_semantic_graph`; a model owner admits when its identity names a selected domain package and refuses as `invalid_semantic_graph` when it names none or carries an empty node. | Test (TC-048) |
| FR-038-AC-6 | Every admitted node family lowers independently with exact source, type, dependency, bound and claim correspondence; missing, unsupported, unbounded and over-work requests return `invalid_input`, `unsupported`, `requires_bound` and `failed` without a node and without changing sibling records, except that a package over the byte ceiling fails every requested record (FR-038-AC-95). | Test (TC-050) |
| FR-038-AC-9 | The shipped default read-limit policy is exactly those six finite values, every member is strictly positive and finite, and each meter is enforced at its own true measured boundary against a real package: the package's exact measured consumption for that meter admits it, and one below that exact value refuses it as `incomplete`, naming that meter and reporting the true consumption. The shipped default is far larger than any fixture, so this boundary is proven against each meter's real measured cost rather than against the default value itself; no fixture approaches that scale, and none is fabricated to do so. | Test (TC-048) |
| FR-038-AC-7 | Lowering every node of every positive fixture under a profile supporting every tag yields no `invalid_body` and no `body_incomplete` record, and the seven-member record vocabulary is exhaustive: no eighth kind is reachable and each of the seven is named. | Test (TC-052) |
| FR-038-AC-8 | A closure holding both an out-of-profile tag and an unbounded type returns `unsupported`; a zero work limit returns `failed` for an absent key rather than `invalid_input`; each named offending key is the least in ascending order rather than the first visited; each of the eight unbounded forms raises `requires_bound` and each other declared form of those two families, carrying no `recursion_group`, does not; and a lowered record's `dependencies` contains every key in its `bounds` and `claims`. | Test (TC-052) |
| FR-038-AC-10 | A `lock.model_selections` entry that repeats an earlier entry's identity, digest domain and digest verbatim, mirrored identically into `identity_preimage.model_selections`, refuses as `malformed_wire` at `lock.model_selections`; the same lock-side repeat left unmirrored in the identity preimage refuses earlier, as `stale_dependency` at `lock`, because the preimage/lock equality check runs first; two entries sharing an identity but differing in digest are not a repeat and are owned by FR-038-AC-20 (`stale_dependency`, never `malformed_wire`). | Test (TC-048) |
| FR-038-AC-11 | A `lock.model_selections` array carrying both a repeated entry and an entry whose digest no document the package evidence supplies satisfies refuses as `malformed_wire` at `lock.model_selections`, never as `stale_dependency`, regardless of whether the repeated entry or the stale entry appears first in the array — the uniqueness check runs over the whole array before any entry's digest is checked against the supplied documents, so the outcome does not depend on array position. | Test (TC-048) |
| FR-038-AC-12 | A `creates` or `deletes` entry's eligibility is exactly `model`/`object_type` and `model`/`process`, and this eligibility predicate is checked against every `(member, tag, form)` triple the closed node taxonomy can produce, not sampled; the `modifies` entry shape and its eligibility are [FR-040](./FR-040-admit-frame-entries-and-state-clauses.md)'s. The refusal an ineligible triple drives — `invalid_model_binding` with cause `malformed-declaration`, located at the offending entry — is verified end-to-end through the reader for a representative ineligible triple in each of `creates` and `deletes` and every node family; the reader's mapping from an ineligible predicate result to that refusal depends only on which member the entry sits in, never on its tag or form, so this sample together with the exhaustive predicate check covers the full triple space without driving every one of it through the reader. Each of the two eligible triples the published all-families fixture's own frame body does not already exercise (`process` in `creates`, `object_type` in `deletes`) is independently shown admitted. | Test (TC-053) |
| FR-038-AC-13 | An entry naming a digest that is not among the frame node's own `dependencies` refuses as `missing_declaration` with cause `missing-name`, located at that entry, whether the digest resolves to no node the frame declared as a dependency (a real node elsewhere in the graph) or to no node anywhere in the graph at all — both conditions are the same refusal, never `invalid_model_binding` and never the graph's generic unresolved-reference `invalid_semantic_graph`. | Test (TC-053) |
| FR-038-AC-14 | A frame node carrying both a meaning-join defect and a canonical-order defect refuses for the meaning-join defect; among two meaning-join defects in different members, the earlier member (`modifies` before `creates` before `deletes`) is selected regardless of which defect's entry digest is lower, except that a `conflicting-binding` defect ranks after every other meaning-join defect (FR-040-AC-25); among two meaning-join defects in the same member, the lower entry digest is selected; a member array not in strictly ascending digest order, with no meaning-join defect present, refuses as `invalid_semantic_graph` at the frame body path, located at the frame node itself and carrying no cause. | Test (TC-053) |
| FR-038-AC-15 | A package carrying two defective `state`/`frame` nodes refuses at the one with the lower `node_id` digest, reporting only that frame's own defect, even when the other frame's defect would otherwise outrank it under FR-038-AC-14's precedence — the visit order is ascending node-id digest across frames, and the reader reports the first defective frame it reaches rather than comparing every frame's defect. | Test (TC-053) |
| FR-038-AC-17 | Each required declaration, literal type, application operation and result_type is read from the graph and enters the derived hash projection. With the graph-derived package id refreshed to reach the member check, deleting one refuses invalid_semantic_graph at the existing node declaration or body pointer before admission. All fifteen expression forms remain admitted through the same closed taxonomy. | Test |
| FR-038-AC-18 | For a keyed input that reaches the graph-reference stage, a self-typed node's own body-root `literal.type` naming itself is exempt from the reference-cycle check and needs no declared `recursion_group`. The same self-reference nested in an `aggregate` member, `binding` value or `application` argument is not exempt and refuses as `invalid_semantic_graph` at `/semantic_graph/nodes/<n>` (the node object lacking `recursion_group`), as do a self-typed node's `reference` body and `application.result_type` naming itself. Public admission authenticates content-derived structural and application keys before that stage; a body embedding its own final key has no constructible honest preimage. Public tests assert that first stale-key refusal, and a private graph-stage test verifies the conditional carve-out. | Test (TC-048) |
| FR-038-AC-19 | A `lock.model_selections` array carrying two entries of different defect classes refuses for the earlier class under the stated total order, at `lock.model_selections`, regardless of which of the two entries appears first in the array: an entry outside `sha256-jcs` beside an entry of empty `identity` refuses as `digest_domain_mismatch`, and an entry of empty `identity` beside an entry whose digest no supplied document satisfies refuses as `malformed_wire`. Both orderings of each pairing are pinned and refuse with the same code, so a refusal decided by array position rather than by defect class fails this criterion rather than passing it as "some refusal occurred". | Test (TC-048) |
| FR-038-AC-20 | A selection binds by `identity`, so a `lock.model_selections` array holds at most one entry per identity: two entries naming the same `identity` and differing in `digest` refuse as `stale_dependency` at the later entry's `digest` (`/lock/model_selections/<i>/digest`), whichever of the two entries appears first in the array, even when both entries are independently well-formed and each names a document the package evidence supplies, and never as `malformed_wire`; the pair is refused before any document is read, so it refuses the same way, in both orders, when an entry of the pair or an earlier row has no supplied document, a forged one or one naming another identity, and never as that document's own `missing_import`, `content-mismatch`, `byte-digest-mismatch` or `invalid_model_binding` refusal; an array whose two entries name different identities still admits (each other check passing). An entry carries no `version`, so the identity is the whole locator (FR-038-AC-62). A same-identity, different-digest pair is decided by the total order above, so it refuses `digest_domain_mismatch` when one of its entries is outside `sha256-jcs`, whichever of the two carries that second defect. | Test (TC-048) |
| FR-038-AC-21 | A node whose declared `qualified_name` differs from its nominal preimage's `qualified_declaration` refuses as `invalid_package` with cause `declaration-nominal-mismatch` at that node; two nodes in the same namespace declaring one name refuse as `ambiguous_declaration` with cause `ambiguous-name` at the later node in node-id order, while the same name across namespaces admits; a package carrying both defects refuses for the nominal mismatch; and a package also carrying a frame or an operation defect still refuses for its declaration defect. | Test (TC-048, TC-230) |
| FR-038-AC-22 | A QSL-shaped package holding a function whose body applies an operation to its parameters, each a `value`/`parameter` node, and a `scalar_type`/`compound_unit` node, admits and lowers; a parameter with a negative, non-canonical or missing level, an empty, non-identifier or wrongly typed name, or an extra binding refuses as `invalid_semantic_graph` at the node's `body`; a parameter with a dependency, itself as its type or a `declaration`, and a compound unit with a zero exponent, a term naming a non-unit, a repeated unit, a dependency list other than its unit keys or a type other than itself, each refuses as `invalid_semantic_graph` at the member it breaks; the empty compound unit admits. | Test (TC-048) |
| FR-038-AC-23 | An application node whose `dependencies` omits a body reference target, lists them out of digest order, repeats one, or adds its `result_type` refuses as `invalid_semantic_graph` at `semantic_graph.nodes.dependencies`, located at that node. | Test (TC-048) |
| FR-038-AC-24 | Every refusal about a value carries the RFC 6901 pointer of that value, built from the reader's own position: member names escaped (`~` as `~0`, `/` as `~1`), array elements by index, resolving in the document read — an unknown member (including one whose name holds `~` or `/`) at that member, a repeated member at that member, a missing member at the object lacking it, a wrongly typed value at that value, a stale mirror at the first differing value, and each graph, lock, source-map, capability and diagnostic refusal at the member its failed check read; malformed JSON and non-canonical bytes carry no pointer. The `unknown field` occurrence-role value of FR-038-AC-2 points exactly at `/semantic_graph/nodes/0/occurrences/0/role`; the genuine extra-member and wrong-kind controls each point at the member or value they change. No refusal code changes. | Test (TC-048) |
| FR-038-AC-25 | An `unknown_contract_version` refusal points at `/contract_version` and carries the exact `contract_version` string the reader read, including an empty one; no other refusal carries a version. | Test (TC-048) |
| FR-038-AC-26 | Each one-over limit other than the byte limit returns `incomplete` carrying the RFC 6901 pointer of the value whose charge failed, which resolves in the document read: nodes at the first node past it, edges at the dependency and occurrences at the source-map entry or region that took the count past it, diagnostics at the first entry past it, and work at the value whose validation took the meter past it; the byte limit carries none. | Test (TC-048) |
| FR-038-AC-27 | A domain package selection is admitted only by the document the evidence supplies under its digest: a document supplied under no digest of the row refuses `missing_import`/`missing-selection` at the row's `digest`; a document holding a number whose text denotes a magnitude past 2^53 refuses `noncanonical_wire` at the `digest` before any digest is computed (FR-038-AC-93); a document whose RFC 8785 SHA-256 is not the digest it was supplied under refuses `stale_dependency`/`content-mismatch` at the `digest` (and bytes that do not read, `byte-digest-mismatch`, FR-038-AC-216); a document naming another identity refuses `invalid_model_binding`/`wrong-model-selection` at the row's `identity`; and a matching document admits. The document's `package.version` is neither required nor read: a matching document with no `package.version`, or a non-string one, admits, and the same package supplied as two documents at different versions admits when each is selected in its own package under its own digest (FR-038-AC-64). | Test (TC-048) |
| FR-038-AC-28 | A domain package document's declarations refuse at the row, in FR-154's order: a node whose object id is invalid, whose `kind` names no construct, or that shares its identity, refuses for itself and any reference to it reports that refusal, never `missing_declaration`/`missing-name`, wherever the two sort; a node failing two rows reports the earlier (a dangling `typeRef` before a multiplicity with `lower > upper`, a malformed member before both); a `typeRef` naming a relationship refuses `invalid_model_binding`/`malformed-declaration` wherever its declaring node sorts; two nodes with no identity refuse `malformed-declaration`, never `conflicting-binding`. | Test (TC-048) |
| FR-038-AC-29 | A package whose lock selects a domain package document with declared types, and whose graph holds a `dispatch_call` on one of its operations, admits; the same call naming an operation the document does not declare refuses `ill_typed`/`operator-ineligible` at the member's `name`; an inherited field resolves on a subtype and a subtype conforms to its supertype in either operand order; a field typed at an `Int[lo, hi]` value type has the integer-range member type `[lo, hi]`. | Test (TC-048) |
| FR-038-AC-30 | Reading a selected domain package and resolving a model-owned member are charged to the `work` limit: a limit one below the work a read used returns `incomplete` for `work` with the pointer `/lock/model_selections/<i>` of the row, and the exact work admits. | Test (TC-048) |
| FR-038-AC-31 | A package whose lock and identity preimage carry the same `dependency_selections` of two `DependencySelection` entries `{identity, package_id}`, one per identity in ascending identity order, admits when each selected dependency's admitted package is supplied, and both members read back as the supplied entries; changing one entry's `package_id` changes the package's `package_id`. | Test (TC-048) |
| FR-038-AC-32 | A `dependency_selections` entry whose `package_id.domain` is another digest domain refuses `digest_domain_mismatch` at that `domain`; one with an empty `identity`, a short `digest` or a bare-digest `package_id` refuses `malformed_wire` at that member; one that lacks `package_id` or `identity` while carrying a `Selection` or `DefinitionRef` member refuses `malformed_wire` at the entry. | Test (TC-048) |
| FR-038-AC-33 | A `dependency_selections` entry repeating an earlier entry's `identity`, adjacent or not, refuses `invalid_package`/`conflicting-definition` at the repeating entry; an entry not strictly after its predecessor in UTF-8 byte order refuses `invalid_package`/`invalid-value` at that entry; entries in UTF-8 byte order where UTF-16 code-unit order differs admit. | Test (TC-048) |
| FR-038-AC-35 | A package whose `function` call has a `dependency_reference` callee to a declared function of the supplied dependency admits, carries the term verbatim in the node body and lists no `dependencies` entry for it; listing the target refuses `invalid_semantic_graph` at the node's `dependencies`; changing only the term's `package`, or only its `node`, without re-deriving the key refuses `invalid_package`/`stale-node-key`, and re-derived they give distinct node ids. | Test (TC-048) |
| FR-038-AC-36 | A `dependency_reference` with a bare-digest, other-domain or short-digest `package` refuses `invalid_semantic_graph` at the `package`, one in another node domain at the `node`, and one with a missing or extra member at the term; a well-formed term that is not argument 0 of a `quire.op.function.call` (a second argument, an argument of another operation, an aggregate member or a node body root) refuses `ill_typed`/`operator-ineligible` at the term. | Test (TC-048) |
| FR-038-AC-37 | A `dependency_selections` entry with no package supplied for its `identity` refuses `missing_import`/`missing-selection` at the entry, and one whose package has another `package_id` `stale_dependency`/`content-mismatch` at its `package_id.digest`; a term whose `package` no entry names refuses `missing_declaration`/`missing-selection` at the `package`, a `node` naming no node or a node without a `declaration` `missing_declaration`/`missing-name` at the `node`, and a `node` naming a declared node that is no function `ill_typed`/`operator-ineligible`, each carrying the calling node as its locus. | Test (TC-048) |
| FR-038-AC-293 | Given an otherwise valid package with direct selections for two admitted dependencies whose own direct selections reach the same transitive identity with different package ids, the V2 reader refuses `invalid_package`/`conflicting-definition` before graph-reference validation. Its `path` and typed `related_path` name the two exact importer-rooted selection paths, each following `/lock/dependency_selections/<i>` through the admitted package selected by the preceding entry; neither path is encoded in prose. The same refusal holds when the importer has one direct selection whose identity recurs in that admitted child's transitive closure with a different package id. | Test (TC-048) |
| FR-038-AC-38 | A dependency function whose parameter is a declared record, whose result is a declared record, whose parameter is a `Set` of one, a tuple holding one, or a `Reference` to a `model` node refuses `ill_typed`/`operator-ineligible` at the callee; the same function over a `Set` of a bounded integer admits. | Test (TC-048) |
| FR-038-AC-39 | Under a bounds-required profile, `x + 1` over a parameter `x` typed at an `integer_range` domain over `integer` lowers although the parameter's name literal is annotated with the unbounded `text` type, and the annotation stays in the closure and in `dependencies`; a parameter typed at an unbounded `integer` or `rational` type refuses `requires_bound` naming that type. | Test (TC-050) |
| FR-038-AC-40 | Under a bounds-required profile a `bounded_domain` over a shared unbounded type does not bound another position naming it: `{n: Integer, k: Int[0,9]}`, `Sequence<Integer>[0,3]` beside `Int[0,9]`, and `(x + 1) + n` over `x: Int[0,9]` and an `n` typed at `integer` each return `requires_bound` naming `integer`, while `Sequence<Boolean>[0,3]`, a record of bounded collections and ranged fields, and `x + 1` over `x: Int[0,9]` lower. | Test (TC-050) |
| FR-038-AC-41 | Under a bounds-required profile a `scalar_type` or `composite_type` node declaring a `recursion_group` returns `requires_bound` naming the least offending node key, although every field of the recursive record is bounded. | Test (TC-050) |
| FR-038-AC-42 | An enum's operand family is `ordered_enum` when its nominal preimage is `ordered` and `enum` otherwise (QSpec FR-322): `quire.op.enum.lt`, `le`, `gt` and `ge` over two operands of an ordered enum, member literals or parameters, admit; the same operations over an unordered enum refuse `ill_typed`/`operator-ineligible` at the first argument (QSpec FR-141-AC-5), and over operands of two different enums at the second; `quire.op.enum.eq` and `ne` admit over either. | Test (TC-048) |
| FR-038-AC-43 | `structural.eq` over an all-integer record, and `collection.contains` over a set of integers, admit with `leaves` empty, because the compared type has no text leaf; the same operations over a record with a nested `text` field, or a set of `text`, refuse `invalid_package`/`operation-law-missing` at `operation.leaves` when `leaves` is empty, and one supplied leaf over two text fields refuses the same way; `collection.flatten` to a `sequence` of text admits with `leaves` empty, while a `set` of text result refuses; a compared type that names a missing node, or that reaches an option or collection again with no record or tuple between the two visits, refuses `ill_typed`/`operator-ineligible` at `operation.leaves`, while a compared type that reaches itself through a record or tuple admits (FR-038-AC-70 through FR-038-AC-72); and a chain of 12 record levels of 4 fields naming the next level is decided inside the work budget. | Test (TC-048) |
| FR-038-AC-44 | `operation.leaves` is exactly the derived leaves, the text leaves and, for a type that reaches itself, its recursion leaves (FR-038-AC-70, FR-038-AC-71): a record with `a`, `b` and `c` (an option of text) fields over a `text_bounds` text type that binds the `nfc` profile admits the leaves `["field:a"]`, `["field:b"]` and `["field:c", "inner"]`, a tuple the `position:<n>` segments, and `collection.contains` over a set of text the one empty path, each carrying exactly one catalogued `text_profile` law the lock selects and the mode `{kind: text_profile, value: nfc}`; a text leaf whose type binds no profile refuses `ill_typed`/`operator-ineligible` at `operation.leaves`, with the leaves supplied or not; a leaf with no mode refuses `operation-mode-mismatch` at its `mode`, one of another kind at `mode/kind`, one of an uncatalogued value at `mode/value`, and a catalogued value other than the pinned one `operation-mode-type-mismatch` at `mode/value`; `collection.flatten` to a `sequence` of text refuses one supplied leaf `operation-law-mismatch` at `operation.leaves/0` and admits none; an entry with a law the lock does not select and a supplied leaf but no leaf source refuses `operation-law-mismatch`, not `operation-law-unselected`; two leaves over an all-integer record, one leaf more than the text leaves, and an entry that names no leaf source with a leaf refuse `invalid_package`/`operation-law-mismatch` at the first extra leaf; three unrelated paths, a wrong or missing `inner` segment, two leaves out of order and a wrong segment kind refuse the same way at that leaf's `path`; a leaf with no law, two laws, a law of another role and a law outside the catalogued `text_profile` definitions refuse the same way at that leaf's `laws`; a leaf law the lock does not select refuses `operation-law-unselected` at its `definition`; and 16 levels of 10 fields all naming the next level over text refuse `operation-law-missing` without listing its 10^16 leaves, while the one leaf of a 16-level path with nine integer fields per level admits at its exact path and refuses at a wrong segment. | Test (TC-048) |
| FR-038-AC-45 | A package a producer emits under the content-only `ModelOwner` identity (QSpec FR-322-AC-28), whose model-owned nominal preimages carry the owner `{kind: model, identity, node}` and no `version`, admits when the owner's identity names a selected domain package; its model-owned node keys are the SHA-256 of that preimage's canonical bytes and do not change when only the selected row's `digest` changes (the lock row carries no `version` to change, FR-038-AC-62). A package whose lock selects a domain package and whose graph holds a model declaration node keyed under the version-free `ModelOwner` (`ModelDeclarationNode`) resolves a model-owned `field` or `operation` member against it and admits, the declaration node's key is unchanged when only the selected row's `digest` changes, and the same node keyed under another domain package's identity refuses `missing_declaration`/`missing-selection` at the member's `declaration`. An owner of kind `model` that carries a `version` member refuses as `unknown_member` at that member, and one with an empty `identity` or `node` refuses as `invalid_semantic_graph`. | Test (TC-048) |
| FR-038-AC-46 | A package whose definition references are exactly `{authority, identity}` with nonempty strings, at `lock.edition.definition`, each `lock.profile_selections[].definition`, each `lock.definition_selections` entry, the same three in `identity_preimage`, `diagnostics.catalog` and each `operation.laws[].definition`, admits, each other check passing. | Test (TC-048) |
| FR-038-AC-47 | A package whose source references are exactly `{authority, identity, digest_domain, digest}`, at each `lock.sources` row, each `source_map` region `source` and each `diagnostics.entries[].loci[].source`, admits, each other check passing. | Test (TC-048) |
| FR-038-AC-48 | A definition reference at a lock, `identity_preimage` or `diagnostics.catalog` site that carries `revision`, `digest_domain`, `digest` or `export` refuses as `unknown_member` at the first such member in document order and returns no package; a reference of the earlier five-member shape (`authority`, `identity`, `revision`, `digest_domain`, `digest`) refuses at its first extra member and is never read, relabeled or admitted. | Test (TC-048) |
| FR-038-AC-49 | An `operation.laws[].definition` that carries a member beyond `authority` and `identity`, lacks one or holds one of the wrong kind refuses as `invalid_semantic_graph` at that member, or at the `definition` when a member is absent, as every closed shape inside an `operation` does (FR-038-AC-36), never as `unknown_member` or `malformed_wire`. | Test (TC-048) |
| FR-038-AC-50 | A `lock.sources` row, `source_map` region `source` or `diagnostics.entries[].loci[].source` that carries `revision` or `export` refuses as `unknown_member` at that member and returns no package. | Test (TC-048) |
| FR-038-AC-51 | A definition reference at a lock, `identity_preimage` or `diagnostics.catalog` site whose `authority` or `identity` is absent or not a string refuses as `malformed_wire` at the value, or at the reference when the member is absent; at a `lock` or `diagnostics.catalog` site an empty `authority` or `identity` refuses as `malformed_wire` at that member. | Test (TC-048) |
| FR-038-AC-52 | A source reference at a `lock.sources` row, `source_map` region or `diagnostics.entries[].loci[]` whose `authority`, `identity`, `digest_domain` or `digest` is absent or not a string refuses as `malformed_wire` at the value, or at the reference when the member is absent, before any domain or equality check. | Test (TC-048) |
| FR-038-AC-53 | A `lock.sources` row whose `digest_domain` is another string refuses as `digest_domain_mismatch` at `digest_domain`, ahead of the row's empty-member and digest-form checks, so a row with another domain and an empty `authority` or a non-hex `digest` refuses `digest_domain_mismatch`. | Test (TC-048) |
| FR-038-AC-54 | A `lock.sources` row, whose `digest_domain` is `quire.source.bytes/v1`, with an empty `authority` or `identity` or a `digest` that is not 64 lowercase hex digits refuses as `malformed_wire` at that member. | Test (TC-048) |
| FR-038-AC-55 | A `source_map` region `source` or `diagnostics.entries[].loci[].source` that is well formed but equals no `lock.sources` row in all four members refuses as `invalid_source_map` at that `source`, whatever member differs (including a wrong `digest_domain`, an empty member or a non-hex `digest`); the row-level refusals of FR-038-AC-53 and FR-038-AC-54 apply to `lock.sources` only. | Test (TC-048) |
| FR-038-AC-56 | An `operation.laws[].definition` of a value role admits only when its `{authority, identity}` pair is catalogued for the role and is a `lock.definition_selections` row, compared by those two members alone: a pair the role does not catalogue, including an empty one, refuses `invalid_package` with cause `operation-law-mismatch`, and a catalogued pair the lock does not select refuses `operation-law-unselected`, each at that law's `definition`. | Test (TC-048) |
| FR-038-AC-57 | An `operation.laws[].definition` of a profile role (`temporal_profile` or `protocol_profile`) admits only when its `{authority, identity}` pair equals the pair of the lock's `profile_selections` row of that role; any other pair refuses `operation-law-unselected` at that law's `definition`. | Test (TC-048) |
| FR-038-AC-58 | The operation catalog read over supplied bytes whose `law_roles` entries are exactly `{authority, identity}` returns the catalog, and over bytes in which an entry carries `revision`, `digest_domain` or `digest` returns an error naming the unreadable entry instead of a catalog and without a panic; this is a catalog read failure and not a package refusal. | Test (TC-048) |
| FR-038-AC-59 | A `lock.definition_selections` row that differs from its `identity_preimage` mirror in `authority` or `identity` refuses as `stale_dependency` at the first differing value. | Test (TC-048) |
| FR-038-AC-60 | A package whose one definition row's `identity` is edited in the lock and the preimage and whose `package_id` is re-derived admits under the new `package_id`, and the same edit under the old `package_id` refuses as `stale_dependency`. | Test (TC-048) |
| FR-038-AC-61 | A `source` or `definition` nominal owner whose `{authority, identity}` pair is the pair of no `lock.sources` or `lock.definition_selections` row respectively refuses as `invalid_semantic_graph`. | Test (TC-048) |
| FR-038-AC-62 | A package whose `lock.model_selections` rows are exactly `{identity, digest_domain, digest}`, mirrored identically in `identity_preimage.model_selections`, admits, each other check passing; a row carrying every one of those members plus `version`, in the lock or in the identity preimage, refuses as `unknown_member` at that `version` member, the first in document order, and returns no package (a row that also lacks a required member is the missing-member case below, and this criterion pins no outcome for that combination); a row lacking `identity`, `digest_domain` or `digest`, or holding one of the wrong kind, refuses as `malformed_wire` at the row or the value; and the reader never reads, drops or compares the extra `version`. | Test (TC-048) |
| FR-038-AC-63 | A package whose `lock.dependency_selections` entries are exactly `{identity, package_id}`, mirrored identically in `identity_preimage.dependency_selections`, admits, each other check passing; an entry carrying both `identity` and `package_id` plus `version`, in the lock or in the identity preimage, alone or beside an otherwise well-formed entry, refuses as `unknown_member` at that `version` member, the first in document order, and returns no package; an old-shape entry `{identity, version}` that lacks `package_id` refuses as `malformed_wire` at the entry; and the reader never reads, drops or compares the extra `version`. | Test (TC-048) |
| FR-038-AC-64 | Selections bind by identity and content digest alone: a `model_selections` row admits when the document supplied under its `digest` names the row's `identity`, so two documents of one package at different versions, each selected in its own package under its own digest, both admit and yield the same model-owned node keys; a model owner joins the row whose `identity` equals its own, and an owner naming another identity refuses as `invalid_semantic_graph`; and a `dependency_selections` entry admits when the package supplied under its `identity` has the entry's `package_id`, with no version supplied or compared, and refuses `missing_import`/`missing-selection` for a package supplied under another identity and `stale_dependency`/`content-mismatch` for another `package_id`. | Test (TC-048) |
| FR-038-AC-65 | Every application operator class, operation member kind and constraint kind the operation catalog declares decodes to a member of its closed vocabulary, so the production catalog, whose `law_roles` entries carry no `revision`, reads once FR-038-AC-46 through FR-038-AC-61 are also implemented; each of `case`, `temporal_formula`, `temporal_fairness`, `temporal_interval`, `fairness` and `union_arms` converts wire string to enum member and back to the same string; catalog bytes that name an operator class, member kind or constraint kind outside its vocabulary return the typed error of the catalog read of FR-038-AC-58, naming the word, instead of a catalog; and a package application whose `operator` is outside the closed operator vocabulary refuses `invalid_semantic_graph` at that term. | Test (TC-048) |
| FR-038-AC-67 | An unknown identity refuses `unknown-operation` and an `operator` that differs from the catalogued class refuses `operation-class-mismatch`, each ahead of every other check of the entry at the operation step, so `quire.op.control.case` under the operator `unary` refuses `operation-class-mismatch`; and of two defective body-root nodes the one with the lower `node_id` digest is reported when both defects are operation refusals. A placement defect of the temporal step (FR-038-AC-102) is reported ahead of every operation refusal, whatever the digest order. (Amended by IR-549: the earlier text ordered both refusals ahead of blanket temporal/case `unsupported_construct`, which IR-549 removed, and put the placement of these applications inside the operation step.) | Test (TC-048) |
| FR-038-AC-68 | At the operation check of a `quire.op.temporal.clause` application, observed on that node as a unit-level check of the node's operation step and, since IR-549, also as an admitted package whose formula node is a `temporal_formula` application (FR-038-AC-96), an application with the one selected `temporal_profile` law, no member and six arguments (a `reference` to a `value`/`parameter` node of any type, a `text` literal, three `aggregate` terms and a `reference` to a `temporal`/`formula` node) passes; five or seven arguments refuse `ill_typed`/`operator-ineligible` at `arguments`; a first argument that is not a `reference` term and a sixth that references a Boolean node each refuse the same way at that argument; a `reference` to a `temporal`/`formula` node fits the sixth operand and an `any_term` position and no `boolean` operand position; and a member of kind `profile_operator` or any other kind refuses `invalid_package`/`operation-member-mismatch` at `operation.member`. | Test (TC-048) |
| FR-038-AC-69 | A member of kind `temporal_interval` or `fairness` on an application whose catalogued entry has none of the operator classes `case`, `temporal_formula` and `temporal_fairness` and a member of another kind or none, such as `quire.op.boolean.not` or `quire.op.temporal.clause`, refuses `invalid_package`/`operation-member-mismatch` at the member, and, since IR-549, so does a `fairness` member on a `temporal_formula` identity such as `quire.op.temporal.holds`, which the earlier text refused `unsupported_construct` at `operator`; the operator refusal is gone and every member that disagrees with its entry refuses alike. | Test (TC-048) |
| FR-038-AC-70 | `structural.eq` over `record Node { label: Text[0, 8; nfc]; next?: Node; }` admits with the leaf `["field:label"]` carrying one catalogued `text_profile` law the lock selects and the mode `{kind: text_profile, value: nfc}` followed by the recursion leaf `["field:next", "inner", "recursion:0"]` with no laws and no mode, and the same comparison with the text leaf alone refuses `invalid_package`/`operation-law-missing` at `operation.leaves`, as does the recursion leaf alone; over `Option<Node>` it admits the leaves `["inner", "field:label"]` and `["inner", "field:next", "inner", "recursion:1"]`; over a mutually recursive pair `A { name: Text[0, 8; binary-utf8]; b?: B }` and `B { tag: Text[0, 4; nfc]; a?: A }` in one package, compared at `A` and compared at `B`, each leaf with the mode its own type pins, it admits `["field:name"]`, `["field:b", "inner", "field:tag"]` and the recursion leaf `["field:b", "inner", "field:a", "inner", "recursion:0"]` at `A`, and `["field:tag"]`, `["field:a", "inner", "field:name"]` and the recursion leaf `["field:a", "inner", "field:b", "inner", "recursion:0"]` at `B`; over `Two { x: Node; y: Node }` it admits exactly the text leaves `["field:x", "field:label"]` and `["field:y", "field:label"]` each followed by its own recursion leaf, `["field:x", "field:next", "inner", "recursion:1"]` and `["field:y", "field:next", "inner", "recursion:1"]`, and refuses the list lacking either recursion leaf `operation-law-missing`; over `Tree2 { label: Text[0, 8; binary-utf8]; kids: Sequence<Tree2>[0, 3]; }` it admits `["field:label"]` and the recursion leaf `["field:kids", "inner", "recursion:0"]`; over a declared tuple `Pair` of a `Text[0, 8; nfc]` and an `Option<Pair>` it admits `["position:0"]` and the recursion leaf `["position:1", "inner", "recursion:0"]`; over `X { t: Text[0, 8; nfc]; n?: Y }`, `Y { u: Text[0, 8; nfc]; x?: X }` and `Wrap { y: Y; x: X }` compared at `Wrap` it admits exactly the four text leaves `["field:y", "field:u"]`, `["field:y", "field:x", "inner", "field:t"]`, `["field:x", "field:t"]` and `["field:x", "field:n", "inner", "field:u"]` with the two recursion leaves its reentries derive in their places, `["field:y", "field:x", "inner", "field:n", "inner", "recursion:1"]` after the second and `["field:x", "field:n", "inner", "field:x", "inner", "recursion:1"]` after the fourth, so a count memoised for `Y` under `field:y` is not reused under `field:x`, where `X` is open, and a list of the four text leaves alone refuses `operation-law-missing` at `operation.leaves` while a list of all six and one further text leaf refuses `operation-law-mismatch` at the seventh entry; over a record `R { a?: Text[0, 8; nfc]; b?: Text[0, 8; nfc] }`, whose two optional fields name one option node, it admits `["field:a", "inner"]` and `["field:b", "inner"]`; over a recursive record that reaches no `text` type, such as a `List` of integers, `structural.eq` and `collection.contains` admit with `leaves` empty; and the `Node` comparison with a second text leaf supplied after `["field:label"]` and the recursion leaf refuses `operation-law-mismatch` at that extra entry, `operation.leaves/2`, so a cyclic type is no longer refused `ill_typed`/`operator-ineligible` at `operation.leaves` and no longer admits unchecked leaves. | Test (TC-048) |
| FR-038-AC-71 | Over `Node`, the recursion leaf placed before the text leaf, a recursion leaf at `["field:next", "recursion:0"]`, one `["field:next", "inner", "recursion:1"]` with the wrong `d`, and a second recursion leaf after the first each refuse `invalid_package`/`operation-law-mismatch` at that leaf's `path`, one that carries a law refuses the same way at its `laws`, and one that carries a mode refuses `operation-mode-mismatch` at its `mode`; a recursion leaf at a reentry of an integer `List`, which reaches no `text` type, refuses `operation-law-mismatch` at that leaf's `path`; a text leaf inside a recursive record whose type binds no `text_profile` still refuses `ill_typed`/`operator-ineligible` at `operation.leaves`, with the leaves supplied or not; `T` = `Option<T>`, a `Sequence` of itself, and a record that holds a field of such a type each refuse `ill_typed`/`operator-ineligible` at `operation.leaves` under a work limit of 1000, so the refusal is not `incomplete` for `work`, whereas `R { x: Option<R> }` admits; and a leaf law the lock does not select inside a recursive record refuses `operation-law-unselected` at its `definition`. | Test (TC-048) |
| FR-038-AC-72 | The leaf count and derivation over a cyclic compared type are iterative and decided by the work budget: a cycle of 20000 record nodes, each holding an integer field and naming the next, the last naming the first, with the last also holding one `text` field, admits with its one 20000-segment leaf and its recursion leaf under byte, node, edge and work limits raised to admit it (the default byte limit of 1 MiB is below the package's size), on a thread whose stack is 256 KiB; ten records `R0` to `R9` that each hold a text field and an optional field naming every other record, compared at `R0` with `leaves` empty, return `incomplete` for `work` at `operation.leaves` under the default read limits, rather than a listing of their leaves or any other refusal; and for a ring of 12 records, each holding one text field and an optional field naming the next, compared at the first with its 12 text leaves and its one recursion leaf (the last record's field reenters the first), a work limit one below the work a read used returns `incomplete` for `work` and the exact work admits. | Test (TC-048) |
| FR-038-AC-73 | Under a bounds-required profile a position typed at a quantity returns `requires_bound` naming the `unit` or `compound_unit` node: a record whose field is typed at a `unit` node, a parameter typed at a `compound_unit` node, `Sequence<Quantity>[0,3]` whose element type is a `unit` node, and a record whose field is typed at a reachable `bounded_domain` over a `unit` node, which names the `unit` node rather than the domain, each return it; while a `unit` declaration lowered as the requested node, a `compound_unit` node lowered as the requested node with its unit nodes in `dependencies`, a record with no quantity position whose closure reaches a unit only as a `literal.type` annotation, and an application whose `result_type` is a quantity over parameters typed at a bounded type each lower. | Test (TC-050) |
| FR-038-AC-74 | For every in-repo positive fixture, canonical encoding of its seven selection fields agrees with serde JSON. The borrowed full hash input agrees with an independently assembled occurrence-free graph projection and hashes to the fixture package_id.digest without a domain label. Crafted node projections preserve non-ASCII and astral strings and the integers 9007199254740992 and -9007199254740992. Changing a hash-covered graph or selection member under the original package id refuses stale_dependency at /package_id/digest. | Test |
| FR-038-AC-75 | For every in-repo positive fixture, the bytes `quire_canonical::to_vec` returns for its `CheckedSemanticGraphV2` equal the bytes of `serde_json::to_vec(&serde_json::to_value(&graph))`; and for a graph whose nodes carry a nominal identity preimage of each of the four versions, a `declaration`, a `recursion_group` and bodies of object, array, string, integer, boolean and null values, with strings holding non-ASCII and astral characters (for instance U+00E9 and U+1F600), member names ordered the same by UTF-8 bytes and by UTF-16 code units, and the integers 9007199254740992, -9007199254740992, 0 and -1, the bytes are equal as well. | Test (TC-048) |
| FR-038-AC-76 | For every in-repo positive fixture, the bytes `quire_canonical::to_vec` returns for each of `CheckedPackageLockV2`, `CheckedSourceMapEntry`, `CheckedCapability`, `CheckedSemanticId` and `CheckedDiagnosticsV2` equal the bytes of `serde_json::to_vec(&serde_json::to_value(&value))`, one assertion per type; the diagnostics value carries entries whose `details` hold nested objects and arrays and whose `loci` are non-empty. | Test (TC-048) |
| FR-038-AC-77 | A CheckedSemanticGraphV2 node body, a standalone CheckedNodeProjectionV2 body and a CheckedDiagnosticsV2 details value nested 100000 levels deep each encode on a 256 KiB stack to independently written expected text without stack overflow. The borrowed package hash input and selection-only canonical envelope likewise encode deep graph bodies without retaining a second wire body. | Test |
| FR-038-AC-78 | The encodes of FR-038-AC-77, run under a byte ceiling of the encoded text's exact length, return the bytes; run under a ceiling one byte lower, return the byte-limit error with no bytes; a body nested 20000 levels deep encodes and is not refused for its depth; and a body holding the integer 9007199254740993 returns the encoder's refusal naming that value with no bytes. | Test (TC-048) |
| FR-038-AC-79 | A package document holding the integer 9007199254740993 in a node body refuses `noncanonical_wire` with no pointer, as does one holding -9007199254740993, one holding the float `2.0` in a body, and one whose body object lists a member named with U+E000 before one named with U+10000 (UTF-8 byte order), each refused before any grammar, `package_id` or graph refusal the same document also earns; the same document with 9007199254740992, with `2` in place of `2.0`, and with U+10000 before U+E000 (UTF-16 code-unit order) is not refused `noncanonical_wire`. | Test (TC-048) |
| FR-038-AC-80 | `quire-canonical` is a `branch = "main"` git dependency of this repository's manifests with its source in the `allow-git` list of `deny.toml`, and `make deny` passes; `CheckedSemanticId`, `CheckedSourceMapEntry`, `CheckedCapability` and `CheckedPackageLockV2`, and every type the three `Encode` types and these four hold that does not itself hold a `Value` (`CheckedOccurrence`, `CheckedDeclaration`, `NominalIdentityPreimage`, its four preimage structs, `NominalOwner`, `DimensionTerm`, `CheckedRational`, `CheckedDiagnosticStage`, `CheckedDiagnosticCode`, `CheckedDiagnosticCause`, the node identity, selection, domain-package, dependency-selection, source-region types, `CheckedArtifactRef` and `CheckedSourceRef`), derive `FixedShape`; `CheckedPackageIdentityPreimageV2`, `CheckedSemanticGraphV2` and `CheckedDiagnosticsV2` implement `quire_canonical::Encode`, and they and the types that hold a `Value` do not implement `FixedShape`; the crate's source holds no hand-written `impl FixedShape`, no `const DEPTH` and no wrapper type around a `quire-canonical` type; and no copy of `quire-canonical` source or of its published vectors is in the repository, each checked by a test that reads the manifests, `deny.toml` and crate source. | Test (TC-048) |
| FR-038-AC-81 | Observed at the operation check of a body-root application, an application whose `operation.identity` the operation catalog does not list refuses `invalid_package` with cause `unknown-operation` at `operation.identity`; `quire.op.integer.add`, a `binary` operation, under the operator `unary` refuses `invalid_package`/`operation-class-mismatch` at `operator`; and `quire.op.integer.add` under `binary` with two operands, no law, no mode and no member admits, so neither refusal is a refusal of every application. Their order against every other check of the entry is FR-038-AC-67's. An application whose `operation` is a bare string, not an object of the closed operation shape, refuses `invalid_semantic_graph` at that application's `operation`. Implemented required-presence correction: through `CheckedPackageV2::read`, independently omitting `operation.mode` or `operation.member` from an otherwise valid application refuses `invalid_semantic_graph`, with no cause, at `/semantic_graph/nodes/{i}/body/operation`, before catalog semantic checks; the otherwise identical application with both fields present as `null` admits when its catalog entry requires neither. Re-derive all affected application/node and package identities after each mutation so a stale identity cannot mask the operation shape refusal. Explicit-null required-mode and required-member semantic refusals remain AC-83 and AC-84's. | Test (TC-048) |
| FR-038-AC-82 | Observed at the operation check, `quire.op.integer.div`, which catalogues the one law role `integer_division`, with no law refuses `invalid_package`/`operation-law-missing` at `operation.laws`; `quire.op.integer.add`, which catalogues no law, with one law refuses `invalid_package`/`operation-law-mismatch` at `operation.laws/0`; and `quire.op.integer.div` with one law whose `role` is the word `not_integer_division` and whose `definition` is a catalogued `integer_division` definition refuses `invalid_package`/`operation-law-mismatch` at that law's `role`, so the role is checked in its own right and not only through the definition. | Test (TC-048) |
| FR-038-AC-83 | Observed at the operation check, `quire.op.decimal.add`, which catalogues the mode kind `rounding`, with `mode` `null` refuses `invalid_package`/`operation-mode-mismatch` at `operation.mode`; with the mode `{kind: rounding, value: toward-zero}` over a first operand typed at a `bounded_domain` over a `decimal` whose body binds `rounding` to `nearest-even`, reached by a `reference` to the domain and, separately, by a `literal` whose `type` names it, it refuses `invalid_package`/`operation-mode-type-mismatch` at `operation.mode/value`. | Test (TC-048) |
| FR-038-AC-84 | Observed at the operation check, `quire.op.quantity.convert`, which catalogues the member kind `type_argument`, with its `rounding` mode supplied and `member` `null` refuses `invalid_package`/`operation-member-mismatch` at `operation.member`; `quire.op.record.project` with a `field` member whose `name` is a field the declaring record type does not declare refuses `ill_typed`/`operator-ineligible` at `operation.member.name`; and `quire.op.model.reaches_field` with a `field` member whose `declaration` names no node of the graph refuses `ill_typed`/`operator-ineligible` at `operation.member.declaration`. | Test (TC-048) |
| FR-038-AC-85 | Observed at the operation check, `quire.op.integer.add`, whose entry has two fixed operands and no rest operand, with three arguments refuses `ill_typed`/`operator-ineligible` at `arguments`; with two `literal`s typed at an `integer` node it admits, and with a first `literal` typed at a `boolean` node it refuses the same way at `arguments/0`; `quire.op.boolean.not` over an application of `quire.op.state.clause`, whose catalogued result is `clause`, refuses the same way at `arguments/0`. | Test (TC-048) |
| FR-038-AC-86 | Observed at the operation check, `quire.op.structural.eq`, whose `same_type` constraint covers both operands, admits two distinct `value`/`parameter` nodes typed at one record type, two `literal`s typed at one type, and refuses `ill_typed`/`operator-ineligible` at `arguments/1` for parameters typed at two record types and `literal`s typed at two types, so the constraint compares the types the operands resolve to and not the operand nodes. | Test (TC-048) |
| FR-038-AC-87 | The operand family of a node is exactly, by tag and form: `scalar_type` `boolean`, `integer`, `rational`, `decimal`, `float32`, `float64`, `text` and `enum`, and `composite_type` `option`, `sequence`, `set`, `bag`, `ordered_set`, `record`, `tuple` and `reference`, each its own name; `expression`/`reference`, `reference`; `function` `pure_function`, `predicate` and `recursive_function`, `function`; `model` `object_type` and `systems_interface`, `object`; and `relation`/`population`, `population`; and `temporal`/`formula`, `temporal` (FR-038-AC-68); and, since IR-549, `composite_type`/`union`, `union` (FR-038-AC-99); every other kind of the closed node taxonomy has none, and the pairs are compared whole, so a form moved to another family or added to the list fails it. A kind is type-shaped exactly when its tag is `scalar_type`, `composite_type`, `bounded_domain`, `relation` or `function`, or it is `expression`/`reference` or `temporal`/`formula`, checked for every kind of the taxonomy. The `temporal`/`formula` pair arrives with the IR-503 code change; until it lands the pairs the unit test compares do not include it. | Test (TC-048) |
| FR-038-AC-88 | The preimage of an application node that is member 1 of a `recursion_group` of two, whose body is `quire.op.integer.add` over a `reference` to member 0 and a `reference` to a node outside the group, with `declaration` `{qualified_name: [pkg, total]}`, serializes, observed at the key check and with members in sorted order, to exactly this object: `body` (`arguments` the `group_reference` `{ordinal: 0, term: group_reference}` then the `reference` to the outside node, `operation` `{identity: quire.op.integer.add, laws: [], leaves: [], member: null, mode: null}`, `operator` `binary`, `result_type`, `term` `application`), `declaration` `{qualified_name: [pkg, total]}`, `node_tag` `function`, `recursion` `{ordinal: 1, size: 2}`, `semantic_form` `function`, `semantic_type` and `version` `quire.application-node/v1`; a `reference` to a group member becomes a `group_reference` in an aggregate member, a binding value and the body-root application's arguments, a reference outside the group stays, and `recursion` is `{ordinal, size}` of the node in its group; an application node whose `node_id` is not the SHA-256 of that preimage refuses `invalid_package`/`stale-node-key` at its `node_id`, and one whose `node_id` is that digest admits. | Test (TC-048) |
| FR-038-AC-89 | Every identity digest of an unchanged identity preimage recomputes to the value an in-repo fixture recorded before the move to `quire-canonical`: the nominal fixture keeps exact recorded node keys, every lowered `ir_id`, lowered package `package_id` and canonical bytes, and the all-families and operation fixtures keep their recorded node-key/`ir_id` pairs where the old key equals an independent derivation from that node's unchanged preimage. For every owner-free structural node whose key changes under IR-627, the test keeps the row and compares its key with a test-side SHA-256 over canonical bytes of QSL FR-092's structural preimage constructed from the wire node, and compares its lowered `ir_id` with a test-side SHA-256 over canonical bytes of its lowered-node preimage. An application whose referenced structural key changes keeps its row and is compared with a test-side application preimage and lowered-node preimage in the same way. These are derivation oracles, not authoritative external goldens. Nodes newly bearing a mandatory owner under AC-153 have owner/projection equality and owner joins checked in TC-228; its test builders compute owner-bearing keys from their preimages, while reader re-derivation of those keys remains IR-630. Their pre-owner golden is inapplicable. Authoritative structural-key and owner-bearing goldens are the recorded digests of QSpec's published positive packages and `model-member-type-vectors.json`, consumed directly from its checkout by FR-038-AC-176. A test-side derivation alone does not replace those recorded values. The canonical bytes of one preimage of each kind (a nominal preimage of each of its four versions, an application, a structural, a lowered node and a lowered package preimage) equal an expected byte string written out in the test from the preimage's JSON text, not computed by a call into the code under test, with the recorded digests as the second oracle when the preimage is unchanged. | Test (TC-048, TC-228) |
| FR-038-AC-90 | `NominalIdentityPreimage::digest` takes a byte limit: called with the limit equal to the canonical byte length of a preimage of each of its four versions it returns the digest the node key holds, and one byte lower it returns the encoder's byte-limit refusal and no digest; the reader passes its configured `limits.bytes` to it, and no call in the crate source passes `u64::MAX` or a literal cap, each checked by a test that reads the crate source. | Test (TC-048) |
| FR-038-AC-91 | The files `canonical.rs`, `binding.rs`, `output_mapping.rs` and everything under `checked_package/` hold none of the symbols `CanonicalWriter`, `canonical_envelope_bytes`, `digest_json` and `serde_json_canonicalizer`, and no `serde_json::to_vec` or `serde_json::to_value` call whose result reaches a digest, a node key, a lowered preimage, the order of a dimension's terms, the selected-document digest, output-mapping identity material or the bound identity envelope, each checked by a test that reads those files and counts the symbols and calls, which finds none. The same scan finds no `value_to_vec` and no `impl` of `Encode` for `serde_json::Value`, and the manifest enables `quire-canonical`'s `serde_json` feature (`quire-canonical` #7, merged). | Test (TC-048) |
| FR-038-AC-92 | The application node preimage, `LoweredNodePreimage` and `ContractPackagePreimage` implement `quire_canonical::Encode` and none derives `FixedShape`; `NominalIdentityPreimage` with its four preimage structs, the structural node preimage and the bound identity envelope derive `FixedShape` and hold no `Value`; no `FixedShape` implementation is hand-written for any of them, each checked by a test that reads the crate source. | Test (TC-048) |
| FR-038-AC-93 | A selected model document holding the number `9007199254740993`, `-9007199254740993`, `9.007199254740993e15`, `1e20` or `18446744073709551617` at `/package/count` refuses `noncanonical_wire` at `/lock/model_selections/0/digest` with `document_pointer` equal to `/package/count`, whether the row selects the document's own digest or another digest, so the refusal precedes `content-mismatch`; a document holding such numbers at `/b` and then `/a/0` names `/b`, the first in document order (except that a number with no finite double is named when the reader reaches it, FR-038-AC-110); the same document holding `9007199254740992`, `-9007199254740992`, `9.007199254740992e15` or `0.5` at that pointer is not refused for it and is digested; an integer value type whose upper bound is `9007199254740993` in a document that is otherwise admitted refuses the same way, where today it is admitted (the reader change); and a package document refused `noncanonical_wire` for its own bytes (FR-038-AC-79) carries no pointer and no `document_pointer`. | Test (TC-048) |
| FR-038-AC-94 | A selected model document whose bytes are exactly `limits.bytes` long is read, digested and matches its row, with the package itself shorter than that limit; read with a `limits.bytes` one lower the same package returns `incomplete` for `bytes` with that limit and the document's length reported and no pointer, as FR-038-AC-26 states for the byte limit, while a `work` limit one below the work of the same read still returns the row pointer `/lock/model_selections/0` (FR-038-AC-30). | Test (TC-048) |
| FR-038-AC-95 | Lowering encodes under the byte limit the package was read under: through the unit seam `identify_node`, which takes the ceiling as an argument as FR-034-AC-6's package step does, a node whose retained limit equals the length of its preimage is identified and with the retained limit one byte lower is refused, and that refusal's record is `failed` for the `bytes` limit with `limit` equal to the retained limit and `consumed` equal to the `required` of encoding the same preimage under that limit, which is greater than `limit` and at most the preimage's canonical length, and the same equality holds at a ceiling chosen for the fixture inside a string value of at least two bytes (not at a byte the encoder writes alone, such as a quote, colon, comma or brace, nor inside an escape run), roughly mid-way through the encoding, where `consumed` is neither `limit + 1` nor the full length; a node preimage built in memory holding an integer past 2^53, which no admitted package holds and which the reader would have refused, so the in-memory preimage skips the reader, is refused by the same seam as `failed` for the `bytes` limit with `consumed` equal to `limit + 1`, under a ceiling at or above the length of the encoding up to that number and below `u64::MAX`; through a whole call of `lower` at a ceiling the package fits under but one of the requested nodes' preimages does not, that node's record is `failed` as above and the sibling records are equal to the records the same call returns with that request removed, with the siblings `lowered` in both calls, so the comparison cannot pass because every record failed; a lowered package whose bytes exceed the ceiling, checked through a whole call of `lower` at a ceiling one byte below the package's canonical length, makes every requested record `failed` for the `bytes` limit with `limit` equal to the retained limit and `consumed` equal to the `required` of encoding the package under that limit, returns no lowered node, no dependency node, no package bytes and no id, and does not panic, while a call at a ceiling equal to that length lowers; a work-budget failure still records the `work` limit; no `failed` record has `consumed` at or below its `limit` unless the limit is `u64::MAX`; and the crate source holds no `expect` or `unwrap` on the package encode, checked by a test that reads `lower.rs`. | Test (TC-048) |
| FR-038-AC-96 | Each of the fifteen `temporal_formula` identities (`holds`, `true`, `false`, `not`, `and`, `or`, `implies`, `eventually`, `always`, `once`, `historically`, `until`, `release`, `since`, `triggered`) admits as the body root of a `temporal`/`formula` node, with operator `temporal_formula`, no law, `mode` `null`, no leaves, its catalogued member (`temporal_interval` with a closed interval on the eight interval operators, `null` on the other seven, under a clause selecting a bounded `temporal_profile`) and its catalogued operands (`holds` over one `boolean` operand, `true` and `false` over none, `not` and the four unary interval operators over one `reference` to a `temporal`/`formula` node, `and`, `or`, `implies` and the four binary interval operators over two); each formula node is referenced from a clause's formula argument or a formula operand; `quire.op.temporal.fair` admits as the body root of a `temporal`/`fairness` node with a well-formed `fairness` member and no operand, referenced from the fairness argument of a clause selecting `quire.temporal.infinite-trace/v1` (a bounded profile admits no fairness argument); and a package holding a `quire.op.temporal.clause` node whose sixth argument references a `temporal`/`formula` node applying `quire.op.temporal.eventually` with the member `{kind: temporal_interval, interval: {lower: "0", upper: "3"}}` over a second formula node applying `quire.op.temporal.holds` admits with every node key and its `package_id` re-derived, and lowers under a profile that supports the `temporal` tag and returns `unsupported` naming `temporal` under one that does not. | Test (TC-048) |
| FR-038-AC-97 | On each of the eight interval operators the member `{kind: temporal_interval, interval}` admits, under `quire.temporal.infinite-trace/v1`, with `interval` `{lower: "0", upper: "3"}`, `{lower: "9", upper: "10"}`, `{lower: "2", upper: null}` and `null`; `{lower: "-1", upper: "3"}` refuses `invalid_package`/`invalid-value` at the bound (`/semantic_graph/nodes/{n}/body/operation/member/interval/lower`), `{lower: "0", upper: "-2"}` refuses the same way at `.../interval/upper`, and `{lower: "-5", upper: "-2"}` refuses at `.../interval/lower`, the first bound in member order, each in strict wire validation, before any identity check and any temporal step and under every profile (merged QSpec FR-370-AC-9 and the schema's non-negative bound pattern); every other bound outside the pattern (`"1.5"`, `"01"`, `"+1"`, `""`, `"3x"`, and the JSON integer `0`, which is no string) refuses `invalid_package`/`invalid-value` at that bound, `{lower: "1.5", upper: "0"}` at `.../interval/lower`, in the same early stage and first in member order (`lower`, then `upper`), never `operation-member-mismatch` (merged FR-370: "a bound outside its form's pattern ... refuses `invalid_package`/`invalid-value` at that bound's pointer during strict wire validation"); `{lower: "3", upper: "0"}` and `{lower: "10", upper: "9"}` refuse `invalid_package`/`invalid-value` at the application (`/semantic_graph/nodes/{n}/body`), also when the formula node is reached by no clause (the sweep after every clause, an IR reading) and `{lower: "18446744073709551617", upper: "18446744073709551616"}` refuses the same way, so the comparison is numeric and neither lexicographic nor fixed-width; a member of kind `fairness` and an interval holding a third member each refuse `invalid_package`/`operation-member-mismatch` at `operation.member` (an IR reading, merged text silent), and a `null` member on an interval-capable operator refuses `invalid_package`/`operation-member-mismatch` at the application (`/semantic_graph/nodes/{n}/body`, merged FR-370); and any member on `quire.op.temporal.holds`, `quire.op.temporal.not` or `quire.op.temporal.clause` refuses `operation-member-mismatch` at `operation.member`. | Test (TC-048) |
| FR-038-AC-98 | The `fairness` member of `quire.op.temporal.fair` admits as `{kind: fairness, fairness_kind: weak, granularity: whole, declaration, name}` and as `strong` with `each`, with a `declaration` and `name` that resolve (the resolution refusals are FR-038-AC-103's); a `fairness_kind` of `medium`, a `granularity` of `part`, a member lacking `name`, a member holding an extra member, a `null` member and a member of kind `temporal_interval` each refuse `invalid_package`/`operation-member-mismatch` at `operation.member`; and a `fairness` member on `quire.op.boolean.not` refuses the same way. | Test (TC-048) |
| FR-038-AC-99 | A package holding a `composite_type`/`union` node `Shape` with the members `Circle(Integer)`, `Rect(Integer, Integer)` and `Empty`, a `value`/`union_value` node `Shape::Rect(2, 3)` and an `expression`/`case` node whose body root applies `quire.op.control.case` over the scrutinee and the arms `Circle`, `Rect` and `Empty` in member declaration order, the binders `r`, `w` and `h` as `value`/`parameter` nodes typed `Integer` and every arm body of the `result_type`, admits, the three forms decoded and the operand family of the `Shape` node `union`; the same `case` with its arms out of member order, with the `Empty` arm omitted, with the `Circle` arm repeated, with the `Rect` binder aggregate holding one reference, with the `Circle` binder typed `Text` and with one arm body of another type than the `result_type` each refuse `ill_typed`/`operator-ineligible` at the `case` node (`/semantic_graph/nodes/{n}`); a union with no member and a union value with two bindings each refuse `invalid_semantic_graph` at the node's `body`; a union type with `Circle` twice refuses `invalid_package`/`duplicate-member` and a payload reference to a node that is not a type refuses `ill_typed`/`operator-ineligible`, each at the union node; and a union value naming `Triangle`, a `Rect` with one payload term and a `Circle` over a `Text` payload each refuse `ill_typed`/`type-mismatch` at the node. | Test (TC-048) |
| FR-038-AC-100 | Placement in both directions: an application of operator class `temporal`, `temporal_formula` or `temporal_fairness`, naming any identity, refuses `ill_typed`/`operator-ineligible` at the node that holds it when it is the body root of a node of another form (a `temporal_formula` application in a `function` node, a `quire.op.temporal.fair` application in a `temporal`/`formula` node), while the same application as an element of another application's `arguments`, a `binding` value or inside an `aggregate` refuses `malformed_wire` at the nested application at strict wire validation, ahead of every step here (FR-038-AC-114); a `temporal`/`formula` node, a `temporal`/`fairness` node and a `temporal`/`temporal_clause` node whose body is an empty `aggregate`, a `literal`, or an application of another class refuses the same way at the node; a `reference` to a `temporal`/`formula` node from a `function` node's body, from a clause's fairness argument or from a `case` argument, and a `reference` to a `temporal`/`fairness` node from a clause's formula argument or from a `temporal_formula` operand, refuses the same way at the node that holds the reference; a `case` application refuses `ill_typed`/`operator-ineligible` at the node that holds it when it is the body root of a node that is not an `expression` node, at the operation step and so after every temporal-step defect whatever the digest order (merged FR-440 join 1; not in the temporal placement pass), and at its own `operator` (`/semantic_graph/nodes/{n}/body/.../operator`) when it is nested in another term (merged FR-322 "Body grammar"); an `expression` node whose `semantic_form` contradicts its root application's operator class (an `expression`/`case` node whose body is not a `case` application, an `expression` node of another form whose body root is a `case` application) refuses `invalid_semantic_graph` at the node's `body` (merged QSpec FR-440 and FR-322); a `diagnostics.entries[].details[]` term that references a `temporal`/`formula`, `temporal`/`fairness` or `expression`/`case` node refuses `ill_typed`/`operator-ineligible` at that entry (`/diagnostics/entries/{e}/details/{d}`; the `case` node reference is merged FR-370-AC-12), a `details` reference to a `composite_type`/`union` or `value`/`union_value` node is an ordinary reference and admits, and an application of the `temporal_formula`, `temporal_fairness` or `case` class as the root of or nested in a `details` term refuses `ill_typed`/`operator-ineligible` at that application's `operator` (`/diagnostics/entries/{e}/details/{d}/operator`, and `.../members/0/operator` for the first member of a `details` aggregate; merged FR-370-AC-12 and FR-322 "Body grammar"), the first in document pre-order, outermost first, while an application of any other class there refuses `malformed_wire` at that application (FR-038-AC-115); and each application, node and reference at its own place admits (FR-038-AC-96, FR-038-AC-99), so none of the refusals is a refusal of every such term. The in-repo `v2_all_families` formula node, whose body is an empty `aggregate`, is refused as above and the code change replaces its body. | Test (TC-048) |
| FR-038-AC-101 | Admitted temporal/formula and expression/case applications are not blanket-refused `unsupported_construct`/`expression-form`. IR-661 adds `CheckedPackageRefusalCode::UnsupportedConstruct` and `CheckedPackageRefusalCause::ExpressionForm` only for FR-152's unsupported unbounded or ordered relationship-navigation destination (PLANNED / UNRUN; AC-170); this does not restore the original temporal/case refusal; the diagnostics wire vocabulary `CheckedDiagnosticCode` still carries `unsupported_construct`, so a `diagnostics.entries[]` entry whose `code` is `unsupported_construct` reads as QSpec's schema allows; no application is evaluated by the reader or the lowerer, so an admitted `temporal`/`formula` node and an admitted `expression`/`case` node lower, under a profile that supports their tags, to nodes whose body equals the admitted body and whose `ir_id` is derived from it as for every node, and a profile lacking the tag returns `unsupported` naming it (FR-038-AC-8). | Test (TC-048) |
| FR-038-AC-102 | The temporal step runs after the frame and state-clause step and before the operation step, placement first: a package holding one node with a placement defect (a `temporal_formula` application as the body root of a `function` node) and a second with a lower `node_id` digest whose operation identity is unknown refuses for the placement defect, and so does the same placement defect beside a lower-digest node whose `operator` differs from its entry's class (`quire.op.control.case` under `unary`), in both digest orders each, the placement defect taken in the shape of a `temporal_formula` application in a `function` node; two placement defects are reported at the lower `node_id` digest; a placement defect is reported ahead of any clause defect in either digest order; within one clause an `over` defect is reported ahead of a fairness-resolution defect, ahead of a profile-fit defect, ahead of an interval-bounds defect, each adjacent pair built as two defects of one clause and in both orders of the two defects' positions; and across two clauses the lower-digest clause's later-stage defect (a profile-fit defect) is reported ahead of the higher-digest clause's earlier-stage defect (an `over` defect), in both digest orders of the two clauses. The temporal step skips a member whose shape the operation step refuses (a wrong-kind member, an interval with a third member), and a `null` member on an interval operator under a bounded profile refuses `invalid_package`/`operation-member-mismatch` at the application. Every bound outside the schema pattern, negative or malformed (`{lower: "1.5", upper: "0"}` included), is a schema-pattern failure, refused `invalid_package`/`invalid-value` at the bound in strict wire validation, before placement and every temporal step, first in member order, under every profile (merged QSpec FR-370-AC-9; there is no asymmetry between a negative and a malformed bound): `{lower: "0", upper: "-2"}` refuses at `.../interval/upper`, `{lower: "-1", upper: "-3"}` at `.../interval/lower`, and `{lower: "-1", upper: null}` refuses at `.../interval/lower` under a bounded profile, under `quire.temporal.infinite-trace/v1` and under a clause that also holds a placement defect at a lower-digest node, never `operation-member-mismatch`; `lower > upper` stays in the bounds step, after profile fit, so `{lower: "3", upper: "0"}` under a bounded profile in a clause whose profile fit also fails refuses the profile-fit defect first. | Test (TC-048) |
| FR-038-AC-103 | A clause whose `over` argument is a `reference` to a `value`/`parameter` node among its `dependencies` admits; one whose `over` references a `value`/`parameter` node that is not among its `dependencies` refuses `missing_declaration`/`missing-name`, and one that references a declared dependency that is a `scalar_type` node refuses `invalid_model_binding`/`malformed-declaration`, each with path `/semantic_graph/nodes/{clause}/body/arguments/0` and the locus of the "Path and locus" table; a fairness member whose `declaration` and `name` resolve to an operation of a `model`/`object_type` declaration node admits (FR-370 "Fairness resolution" steps 1 to 3, in that order), one whose `name` is no operation of that node refuses `missing_declaration`/`missing-name` with path `/semantic_graph/nodes/{fairness node}/body/operation/member/name` and the `declaration` target's key as locus, and one whose `declaration` names a node that is not a `model`/`object_type` declaration (a `scalar_type` node, a `model`/`value_type` node, or a key that names no node) refuses `invalid_model_binding`/`malformed-declaration` with path `/semantic_graph/nodes/{fairness node}/body/operation/member/declaration` and the `declaration` target's key as named as locus; the `over`-not-among-`dependencies` case is observed at the unit level of the temporal step, as AC-68 observes the clause check, because the application-node dependency join refuses a package that names a non-dependency reference first, while the package-level half (an `over` that names no node) is read through the reader; a `name` that matches two exposed operation members of the declaration node refuses `ambiguous_declaration`/`ambiguous-name`, one that the node only inherits admits and resolves to its most-derived redefinition, and a declaring node whose owner is not recovered refuses with that resolution's own refusal (`missing_declaration`/`missing-selection` or `invalid_package`/`stale-node-key`), the ambiguous name with path `.../member/name` and the `declaration` target's key as locus and the unrecovered owner with path `.../member/declaration` and the fairness node's key (merged QSpec FR-370-AC-11 and FR-370 "Fairness resolution" state the ambiguous, inherited-admitted, unrecovered-owner and malformed-declaration outcomes; the loci of the name and owner rows are an IR reading). | Test (TC-048) |
| FR-038-AC-104 | Under a clause whose `temporal_profile` law names `quire.temporal.event-position.false-extension/v1`, `quire.temporal.fixed-sample.false-extension/v1` or `quire.temporal.timestamped-event.finite-window/v1`, every interval operator of its formula tree with a closed interval admits, and one with a `null` interval or `{lower, upper: null}` refuses `invalid_package`/`operation-member-mismatch` at that operator's application; a non-empty fairness argument refuses `operation-member-mismatch` at the clause's application and an empty one admits; under `quire.temporal.infinite-trace/v1` the same `null` interval, `{lower, upper: null}` and a non-empty fairness argument admit; under `quire.temporal.timed/v1` a `null` interval admits and `{lower, upper: null}` and an integer-form closed `{lower, upper}` each refuse `operation-member-mismatch` at that operator's application (merged QSpec FR-370-AC-3: a timed interval is the timed form; under `timed/v1` only `null` and the timed form are admitted, confirmed by the QSL ruling relayed 2026-10-03), and a non-empty fairness argument admits under it as under infinite-trace (only the three bounded profiles refuse one, as above; merged QSpec FR-370-AC-4); the timed form itself, `{lower, upper, lower_end, upper_end}`, is FR-038-AC-119 through FR-038-AC-122's (merged FR-370-AC-8), and this criterion's fit rows are unchanged by it; and the profile fit reads only the formula nodes reachable from that clause. | Test (TC-048) |
| FR-038-AC-108 | Each of the five members of QSpec FR-250's Values table admits as a clause's `temporal_profile` law (the lock's `temporal_profile` row naming it). A clause whose `temporal_profile` law `definition` is not one of the five refuses `unknown_profile` at `/semantic_graph/nodes/{n}/body/operation/laws/0/definition`, ahead of the same clause's `over` defect and of any defect of a higher-digest clause, and is not read as bounded (a `null` interval under it refuses `unknown_profile` and not `operation-member-mismatch`). The cause, merged QSpec FR-370 "Profile check" and FR-370-AC-10, is decided from the package alone: `wrong-selection-role` when the same `{authority, identity}` is a row of the package's own `lock.profile_selections` (or the `lock.edition` row) under a `role` other than `temporal_profile`, `unsupported-selection` otherwise. A known profile is matched by identity label alone, `authority` taking no part: a law naming a FR-250 member's identity under a different `authority` is a known profile and refuses `operation-law-unselected` at AC-57's join, not `unknown_profile`. So `quire.fixture.temporal-profile/v1` selected nowhere else, an empty identity and a differently spelled `quire.temporal.infinite-trace/v1` refuse `unsupported-selection`; `quire.package.composed/v1` named by the law and selected in the lock as a `profile_selections` row of role `binding_contract` refuses `wrong-selection-role`, and so does `quire.protocol.complete/v1` selected as a `protocol_profile` row; and the same two identities named by the law with no such row in the lock refuse `unsupported-selection`. A clause whose `laws` is not exactly one law of role `temporal_profile` (none, or two) skips the profile check and its profile fit, and its law defect refuses at the operation step (`operation-law-missing`, `operation-law-mismatch`; merged QSpec FR-370 "Profile check"). The refusal is the reader's `CheckedPackageRefusalCode::UnknownProfile` with the causes `unsupported-selection` and `wrong-selection-role` in `CheckedPackageRefusalCause`, a code and two causes the code change adds. | Test (TC-048) |
| FR-038-AC-105 | The temporal and case operands are checked at the operation step, after the temporal step: `quire.op.temporal.holds` over a `reference` to a `temporal`/`formula` node, and over a Boolean-family `reference`, admits only the latter, the former refusing `ill_typed`/`operator-ineligible` at the argument; `quire.op.temporal.until` with one argument, `quire.op.temporal.not` with two, `quire.op.temporal.true` with one, and `quire.op.temporal.and` over a Boolean-family `reference` each refuse `ill_typed`/`operator-ineligible` at `arguments` or at the argument; and `quire.op.temporal.not` over a `reference` to a `temporal`/`formula` node admits. | Test (TC-048) |
| FR-038-AC-106 | `quire.op.structural.eq` over two values of the `Shape` union admits, and over a `Shape` value and a value of another union refuses `ill_typed`/`operator-ineligible` at `arguments/1` by its `same_type` constraint (QSpec FR-440-AC-6); over a union whose payload types reach no `text` type, `leaves` empty admits; and over `union Label { Named(Text[0, 8; nfc]), Tagged(Integer, Text[0, 8; nfc]), Empty }`, a comparison admits with exactly the leaves `["member:Named", "position:0"]` and `["member:Tagged", "position:1"]` in member declaration order, and, following FR-038-AC-70's convention, a list holding only the `Named` leaf, and a list holding only the `Tagged` leaf, each refuse `invalid_package`/`operation-law-missing` at `operation.leaves`, while the list of both leaves followed by the `Named` leaf again, and the list of both followed by a leaf `["member:Empty"]`, each refuse `invalid_package`/`operation-law-mismatch` at `operation.leaves/2` (the `member:<Ident>` segment is merged QSpec FR-322-AC-45); and over the recursive unions `union IntList { Cons(Integer, IntList), Nil }` and `union TextList { Cons(Text[0, 8; nfc], TextList), Nil }`, `structural.eq` admits (a cycle through a union is never refused `ill_typed`/`operator-ineligible`, merged QSpec FR-322-AC-46 and FR-440-AC-8): over `IntList`, which reaches no `text`, with `leaves` empty; over `TextList` with exactly the leaves `["member:Cons", "position:0"]` and the recursion leaf `["member:Cons", "position:1", "recursion:0"]` (`d` is 0, the segments the path held when `TextList` was entered, as for a record's `recursion:<d>`), a list lacking the recursion leaf refusing `operation-law-missing` at `operation.leaves`, one whose recursion leaf reads `recursion:1` refusing `operation-law-mismatch` at that leaf's `path`, and a recursion leaf at a reentry of `IntList` refusing `operation-law-mismatch` at its `path`. | Test (TC-048) |
| FR-038-AC-107 | Read from the authoritative `quire-specification` checkout, never copied into this repository, the QSpec positive fixtures `positive-all-families.json` (the clause, `quire.op.temporal.holds` and `quire.op.temporal.eventually` with the interval `{lower: "0", upper: "3"}`), `positive-clause-operations.json` and `positive-union-nodes.json` (`quire.op.control.case`) each admit end to end with their recorded `package_id`. The test is named `tc_048_qspec_positive_fixtures_admit`, lives in its own test target, outside the run of `make test`, which has no checkout; it is run by a dedicated target, `make conformance-qspec`, which reads the root of a `quire-specification` checkout from the environment variable `QUIRE_SPECIFICATION_DIR` and fails, not skips, when the variable is unset, when it names a path that does not hold `proposals/checked-package-v2/fixtures/`, or when a fixture does not admit. The criterion is verified by that target alone and never counted from `make test`; wiring the target into CI is a separate decision this specification does not make. Nothing is copied from QSpec into this repository. The self-built packages of FR-038-AC-96 and FR-038-AC-99 reproduce the fixtures' features: a clause with the `temporal_profile` law `quire.temporal.event-position.false-extension/v1`, formula nodes whose body root is the application, a `case` scrutinee that is a `value`/`parameter` node typed at the union, union value payloads that are references to value nodes, arm binders that are `value`/`parameter` nodes whose body is the `{name, level}` aggregate, and the `case` node's `dependencies` as the digest-ascending reference targets. | Test (TC-048) |
| FR-038-AC-109 | A selected model document holding at `/package/ratio` the number `0.1000000000000000000001`, `9007199254740993.5`, `-0.1000000000000000000001`, `4.9e-324` or `1e-400` refuses `noncanonical_wire` at `/lock/model_selections/0/digest` with `document_pointer` equal to `/package/ratio` and cause `inexact-number` (quire-specification:FR-272), whether the row selects the document's own digest or another digest, so the refusal precedes `content-mismatch`; the same document holding `0.1`, `0.5`, `1.5`, `-0.25`, `5e-324`, `2.5e-10`, `1.0`, `-0` or `1e2` at that pointer (`1.0`, `-0` and `1e2` are spelled other than the text `quire-canonical` writes, `1`, `0` and `100`, and each is admitted because the value is the same), each a number whose exact decimal value equals that of its double's shortest round-trip text, is not refused for it and is digested; the text that decides is the one `quire-canonical` writes for the number, which where two shortest texts are equally close to the double is the one with the even last digit, so a document holding `1125899906842624.2`, `1500000000000000.2` or `2.9802322387695312e-8` at that pointer (each the even-digit text of a tie) is not refused and one holding `1125899906842624.3`, `1500000000000000.3` or `2.9802322387695313e-8` refuses `inexact-number`, and the two texts are compared by their digits and scale, never as doubles; two documents that differ only in `0.1` and `0.1000000000000000000001` at that pointer are not both admitted, the second refusing as above where today both digest alike; the decision is made on the number's text read by `quire-canonical`, through no `serde_json` value, so it is the same whatever features other crates in the build turn on. | Test (TC-048) |
| FR-038-AC-110 | The cause of each refusal FR-038-AC-93 names is `inexact-integer`, with the `document_pointer` AC-93 gives, for `9007199254740993`, `-9007199254740993`, `9.007199254740993e15`, `1e20`, `18446744073709551617`, `9007199254740993.0`, `1e400` and `-1e400`, and for an integer value type whose upper bound is `9007199254740993`; `9007199254740993.0` is `inexact-integer` and never `inexact-number` although its text also differs from its double's shortest round-trip text, and `1e400` and `-1e400` refuse `noncanonical_wire` at that `digest` with the `document_pointer` of the number; `9007199254740992`, `-9007199254740992` and `9.007199254740992e15` are unchanged and admitted, while `9007199254740992.5` (whose nearest double is `9007199254740992`) refuses `inexact-number`; a document holding an inexact number at `/b` and then a whole number past 2^53 at `/a/0` names `/b` with `inexact-number`, the first in document order, and the reverse order names the whole number with `inexact-integer`, except that a number with no finite double is named when the reader reaches it, and so can be named ahead of an earlier inexact number; when faults coexist the first fault `quire_canonical::read` returns decides (QSL FR-056), so `{"b":0.1000000000000000000001,"a":[1e400]}` and `{"b":9007199254740993,"a":[1e400]}` each refuse `noncanonical_wire`/`inexact-integer` with `document_pointer` `/a/0`, `{"a":1,"a":2,"n":1e400}` names `/n`, `{"n":1e400,"a":1,"a":2}` names `/n` and `[1e400` names `/0`, while `[1e400,"<0xFF>"]` (invalid UTF-8, which the read checks over the whole input first) refuses with no `document_pointer`, and, as this reader's reading, `[{"a":1,"a":2},1e400]`, `{"a":1,"a":2,"n":1e-400}` and `[1e400,"<0xFF>"]` offered under a digest that is not their raw digest refuse `stale_dependency`/`byte-digest-mismatch` at the row's `digest` because a reader fault other than an out-of-range number comes first; `1e-400` and `-1e-400` refuse `inexact-number`. | Test (TC-048) |
| FR-038-AC-111 | A package document holding the number `0.1000000000000000000001` or `9007199254740993.5` in a node body refuses `noncanonical_wire` with no pointer, no `document_pointer` and no cause, as it does today, each refused before any grammar, `package_id` or graph refusal the same document also earns; the same document holding `0.1` in that place is not refused `noncanonical_wire`; the package document is parsed through `serde_json`, so the manifest of the crate that holds the reader declares `serde_json` with the feature `float_roundtrip`, which makes the parse of a shortest round-trip text exact, so that a package document holding `1.2793061557049685`, `1.2106592671318679` or `1.3567384036451073` in a node body (each its double's shortest round-trip text, which `serde_json` without that feature reads as a neighbouring double and so would re-encode to different bytes) is not refused `noncanonical_wire`, whatever other crates in the build turn on. | Test (TC-048) |
| FR-038-AC-112 | `make conformance-qspec` reads `proposals/checked-package-v2/fixtures/adverse.json` from the checkout named by `QUIRE_SPECIFICATION_DIR` (never copied into this repository) and applies every mutation of its `structural_mutations` and `body_grammar_mutations` lists, each at its `pointer` with its `replacement`, to a fresh copy of the `positive-all-families.json` package, reading the result with the production reader; each mutation refuses with exactly the code its `outcome` names (`refused:<code>`, and the cause after a `/` where it gives one). The harness refreshes no identity after a mutation: `node_id`, the references to it, the seven wire selection fields in `identity_preimage` and `package_id` stay as the fixture holds them; the reader derives the logical identity projection from the mutated `semantic_graph.nodes` rather than reading a shipped projection, as QSpec's TC-427 BG-02 applies the same mutations, because merged FR-322 has the reader validate the wire, body grammar included, ahead of every identity check, so a mutation under test is the first check that can refuse; a mutation the reader refuses at an identity check (`stale-node-key`, `stale_dependency`, `invalid_semantic_graph`) instead of its recorded code fails. For each `body_grammar_mutations` entry the harness also applies its `flattened` replacement at the same `pointer` in a fresh copy as a positive control, and the reader does not refuse that package `malformed_wire`; it may refuse it at a later identity check, the package being unrefreshed. The run fails, never skips, when the variable is unset or empty, when the file is missing or not JSON, when a list is absent or empty, when a `body_grammar_mutations` entry has no `flattened` member, or when a mutation does not refuse as recorded. A mutation the reader does not yet refuse as recorded is named in the harness's expected-failure list, each entry holding the mutation `id`, the refusal the reader gives it today, which differs from the recorded `outcome`, and the open ticket that owns the missing refusal; the run passes only while every listed id's refusal equals its listed one, and fails when a listed id is absent from `adverse.json`, when a listed id's refusal equals the recorded `outcome` (a stale entry, until it is deleted), when it equals neither, and when an unlisted mutation does not refuse as recorded. | Test (TC-048) |
| FR-038-AC-113 | `make conformance-qspec` reads `proposals/checked-package-v2/dependency-selection-vectors.json` from the checkout named by `QUIRE_SPECIFICATION_DIR` (never copied into this repository) and replaces `dependency_selections` with the file's version-free `{identity, package_id}` entries in both the `lock` and the `identity_preimage` of its `base` fixture, as QSpec's README states, then derives the `package_id` by calling this repository's own derivation, the reader's `CheckedPackageIdentityPreimageRefV2` built from the seven wire selection fields and `semantic_graph.nodes` in graph order, excluding only occurrences, and encoded through `quire-canonical` (FR-038-AC-89), so a harness that hashes the JSON itself cannot pass; the result equals the file's recorded `package_id`, and the `base` fixture unchanged derives its own different `package_id`, so an entry carrying a `version` member, which is refused `unknown_member` (FR-038-AC-62 through FR-038-AC-64), is never the input of the recomputed identity. The run fails, never skips, when the variable is unset or empty, when the file is missing or not JSON, or when the recomputed identity differs from the file's. | Test (TC-048) |
| FR-038-AC-114 | A node body in which an application of any operator class other than `case` stands as an element of another application's `arguments`, as a member of an `aggregate` or as the value of a `binding` refuses `malformed_wire` at that nested application, for `quire.op.function.call`, `quire.op.state.clause` and an application of each of the `temporal`, `temporal_formula` and `temporal_fairness` classes, in each of the three positions, at a pointer such as `/semantic_graph/nodes/{n}/body/arguments/0`; an `aggregate` inside a Group's members, a `binding` whose value is an `aggregate` inside an `aggregate`'s members where the stratum admits only a Leaf value, a `binding` whose value is a `binding`, a `binding` or a Tuple inside a Tuple's members (a Tuple's members are each a Leaf or a Group) and a `binding` as a body root refuse `malformed_wire` at that value; and the same meaning with each composite subterm as its own node reached by `reference` is not refused `malformed_wire`. | Test (TC-048) |
| FR-038-AC-115 | A `case` application nested inside another term refuses `ill_typed`/`operator-ineligible` at the nested application's `operator`; a non-`case` application at the body root of a node its class does not place it in refuses `ill_typed`/`operator-ineligible` at that node and not `malformed_wire`; and where one body or `details` term holds several offending constructs the refusal is at the first in document pre-order, outermost first: a `details` term that is a `temporal_formula` application with a nested `case` argument refuses at `/diagnostics/entries/{e}/details/{d}/operator`, a `details` aggregate of a `temporal_formula` application and then a `case` application refuses at `/diagnostics/entries/{e}/details/{d}/members/0/operator`, a node body whose `arguments/0` is a nested `case` application and whose `arguments/1` is a nested `quire.op.function.call` application refuses `ill_typed`/`operator-ineligible` at `.../body/arguments/0/operator` and, with the two swapped, `malformed_wire` at `.../body/arguments/0` (an IR reading: merged FR-322 gives the pre-order rule, merged FR-440 decides a nested `case` at the operation step, and the two are unreconciled), and an application of a class other than `temporal_formula`, `temporal_fairness` and `case` in a `details` term refuses `malformed_wire` at that application. | Test (TC-048) |
| FR-038-AC-116 | A package whose `node_id`s, `identity_preimage` and `package_id` are all stale and whose one node body holds a nested non-`case` application refuses `malformed_wire` at the nested application, and not `stale-node-key`, `stale_dependency` or `invalid_semantic_graph`, and one whose body holds a nested `case` application refuses `ill_typed`/`operator-ineligible` at its `operator` and not at an identity check (an IR reading, as in FR-038-AC-115); the same package with that body flattened and its identities left stale refuses at an identity check and not `malformed_wire`. | Test (TC-048) |
| FR-038-AC-117 | `CheckedPackageReadLimits` has no depth member and `CheckedPackageLimit` has no `Depth` variant, and no source file under `crates/quire-contract-model/src/checked_package/` holds `MAXIMUM_DEPTH`, another `MAX_*DEPTH` constant, `stacker`, `serde_stacker` or `on_stack_for` (the crate's v1 modules, their limits and its manifest are out of this scan: FR-019 and FR-023 keep them); a package holding a chain of 100000 nodes, each referencing the previous (a 100000-deep expression, as merged QSpec FR-322-AC-41 and TC-427 BG-03 build it), read under byte, node, edge, occurrence, diagnostic and work limits all sized so that none decides the outcome, on a thread whose stack is 256 KiB, is admitted and lowered from its last node with no outcome naming a depth and no stack overflow, and has the same JSON nesting depth as a package of one level; and an otherwise canonical document whose node body nests a term 300 levels deep, past the strict parse's recursion limit of 128, read on a thread whose stack is 256 KiB, refuses `malformed_wire` with no pointer at the strict parse, ahead of the canonical-bytes check and the body grammar, and is never `incomplete`, while the same document nested 20 levels deep, and one nested within the strict parse's limit but far past the grammar (a body of 61 aggregates, 126 JSON levels), each refuse `malformed_wire` at the first value outside the body grammar (FR-038-AC-114), on a 256 KiB thread in a debug build with no stack overflow. | Test (TC-048) |
| FR-038-AC-118 | Each of the five `body_grammar_mutations` of QSpec's `adverse.json` (`application-in-application-arguments`, `application-in-aggregate-members`, `application-in-binding-value`, `aggregate-in-group-members` and `binding-as-body-root`) refuses `malformed_wire` as its recorded `outcome` names, through the harness of FR-038-AC-112, whose expected-failure list holds none of them. | Test (TC-048) |
| FR-038-AC-119 | On each of the eight interval operators under a clause whose `temporal_profile` law names `quire.temporal.timed/v1`, the timed form `{lower, upper, lower_end, upper_end}` admits with bounds `{numerator: "0", denominator: "1"}` and `{numerator: "3", denominator: "1"}` for each of the four end pairs `closed`/`closed`, `closed`/`open`, `open`/`closed` and `open`/`open`; it admits with `lower` `{numerator: "1", denominator: "2"}`, with `lower` and `upper` both `{numerator: "3", denominator: "1"}` and both ends `closed` (a punctual `[3, 3]`), and with `lower` `{numerator: "0", denominator: "1"}`; and a `null` interval on the same operators admits (merged FR-370-AC-3 and FR-370-AC-8). | Test (TC-048) |
| FR-038-AC-120 | Each of these timed-form intervals refuses `invalid_package`/`invalid-value` at the bound, `/semantic_graph/nodes/{n}/body/operation/member/interval/lower` (or `.../upper` where it is the `upper` that fails), in strict wire validation, before the `package_id` recomputation, any node-key check and any temporal step (so also in a package whose `package_id` and node ids are all stale), with the bound of the first failing member in member order (`lower` then `upper`) and the same refusal under `quire.temporal.timed/v1`, `quire.temporal.infinite-trace/v1` and each of the three bounded profiles: `lower` with `numerator` `"-1"` (merged FR-370-AC-9's negative numerator), `lower` with `numerator` `"01"`, `upper` with `denominator` `"0"`, `upper` with `denominator` `"-2"`, `lower` a JSON integer `0`, `lower` the string `"3"`, and `upper` `null` with the four members present (an IR reading from the member-set rule: merged FR-370 states the upper bound is always present); a timed form with `lower` numerator `"-1"` and `upper` denominator `"0"` refuses at `lower`; the pointer is an IR reading, the published schema naming `.../lower/numerator` and the crate's rational check of a nominal preimage `.../numerator`; and a package whose first node holds a non-reduced bound and whose later node holds any of these pattern failures refuses `invalid-value` at the later node's bound. | Test (TC-048) |
| FR-038-AC-121 | Under `quire.temporal.timed/v1`, `lower` `{numerator: "2", denominator: "4"}` refuses `invalid_semantic_graph` at `.../interval/lower` (the code is merged FR-370 and FR-322-AC-10's; the stage after strict wire validation and the identity recomputation, before the model-selection owners step, and the locus are an IR reading), before the temporal step, so a package holding it beside a profile-fit defect in a lower-digest node refuses `invalid_semantic_graph`; a non-reduced `lower` `{numerator: "2", denominator: "4"}` with `upper` numerator `"-1"` refuses `invalid_package`/`invalid-value` at `.../interval/upper`, a negative `lower` numerator `"-1"` with a non-reduced `upper` refuses `invalid-value` at `.../interval/lower`, and a non-reduced `upper` alone refuses `invalid_semantic_graph` at `.../interval/upper`; `(3, 3]`, `[3, 3)` and `(3, 3)` with both bounds `{numerator: "3", denominator: "1"}`, and `lower` `{numerator: "5", denominator: "2"}` over `upper` `{numerator: "2", denominator: "1"}`, refuse `invalid_package`/`invalid-value` at `/semantic_graph/nodes/{n}/body` (merged FR-370 and FR-370-AC-8), while `[3, 3]` admits; `lower` `{numerator: "1", denominator: "2"}` with `upper` `{numerator: "2", denominator: "3"}` admits and the two swapped refuses; and, both ends `closed`, `lower` `{numerator: "18446744073709551617", denominator: "3"}` with `upper` `{numerator: "18446744073709551616", denominator: "3"}` refuses `invalid-value` at the body, which a float comparison, rounding both numerators to 2^64, gets wrong by admitting it, and the two bounds swapped admit, which a checked parse into 64 bits gets wrong; that beyond-2^64 package, `lower` `{numerator: "18446744073709551617", denominator: "3"}` and `upper` `{numerator: "18446744073709551616", denominator: "3"}`, read with a work limit that the GCD or cross-multiplication of its bounds takes past returns `incomplete` naming the `work` limit (FR-038-AC-3) and no refusal of the interval. | Test (TC-048) |
| FR-038-AC-122 | The timed form `{lower, upper, lower_end, upper_end}` with bounds `{numerator: "0", denominator: "1"}` and `{numerator: "3", denominator: "1"}` and both ends `closed` refuses `invalid_package`/`operation-member-mismatch` at `/semantic_graph/nodes/{n}/body` under `quire.temporal.infinite-trace/v1` and under each of the three bounded profiles (merged FR-370-AC-8), and admits under `quire.temporal.timed/v1`, whose refusal of an integer-form `{lower, upper}` and of `{lower, upper: null}` is FR-038-AC-104's; four-member intervals with valid rational bounds and `lower_end` `"half"`, or `upper_end` `"half"`, each refuse `invalid_package`/`operation-member-mismatch` at `operation.member`, as do `{lower: "0", upper: "3", lower_end: "closed"}` (a missing end, integer-string bounds), `{lower: "0", upper: "3", lower_end: "closed", upper_end: "closed", extra: "x"}` (a fifth member, integer-string bounds), each an IR reading that diverges from the published schema, whose closed interval `oneOf` and end `enum` fail them at strict wire validation, and each reported at the operation step after the temporal step, so a profile-fit defect in a higher-digest clause is reported first; and the same missing-end and fifth-member intervals with the rational-object bounds of the timed form refuse `invalid-value` at `.../interval/lower` instead, an interval of another member set having no form, its bounds judged against the integer pattern (an IR reading). | Test (TC-048) |
| FR-038-AC-123 | Tamper regression (IR-627; implemented for the member-read, scalar-operand, state-field body-target and unreferenced rows). Over a package whose model-owned field read names a `bounded_domain`/`integer_range` node keyed as `Int[0, 1000]` (the field declared `Int[0, 1000]` in the selected domain document), the unmutated package admits; the same package with that node's `max` binding changed to `10`, and separately to `5000`, the `node_id` kept and `package_id` recomputed through `quire-canonical` in the test from the seven selection fields and the mutated graph nodes in graph order, excluding only occurrences, each refuses `invalid_package`/`stale-node-key` at that node's `node_id` and returns no package. Mutation rows: `max` changed to `10`; `max` changed to `5000`; `min` changed to `1`; `min` and `max` swapped. The same four rows over an `Int[0, 1000]` node that no member read names (a scalar operand's bound and a state field's body target) refuse identically, so a reader that only compared the node a member read names admits those and fails. | Test (TC-226) |
| FR-038-AC-124 | Closed body of each derived shape (IR-627; implemented; every refused body is keyed again by its own derivation in the test, so only the closed-body rule can refuse it. The earlier assertion that membership in a recursion group itself leaves no derivable key is superseded by the planned AC-145 through AC-150 group derivation). An `integer_range` or `collection_bounds` node whose body is an `aggregate` of exactly the bindings `min` then `max`, each an `integer` literal typed at the `Integer`-keyed node with a string of its form's grammar (`^(0\|-?[1-9][0-9]*)$` for `integer_range`, `^(0\|[1-9][0-9]*)$` for `collection_bounds`), and whose key is the derived one, admits; an `integer_range` `min` of `"-5"` admits. A body with `max` absent, a third binding, the order `max`, `min`, a `max` of `"01"`, `"+5"`, `"-0"`, `"-01"` or `""` on either form, a `collection_bounds` `min` of `"-1"`, a `max` literal typed at the node keyed as `Boolean` or at any node other than the `Integer`-keyed one, or a `max` that is a `text` literal, each refuses `invalid_package`/`stale-node-key` at that node's `node_id`. A `scalar_type`/`integer` node with a non-empty body, a `reference`, `option` or collection node with no member, two members or a member that is not a `reference`, each refuses identically. | Test (TC-226) |
| FR-038-AC-125 | Self-typing and `semantic_type` (IR-627; implemented; an `integer_range` node whose `semantic_type` is not the `Integer`-keyed node also has no derivable key, even when its key is derived over that `semantic_type`). A `scalar_type` or `composite_type` node of a derived shape whose `semantic_type` is another node refuses `invalid_package`/`stale-node-key` at its `node_id`, although its key is unchanged. A `collection_bounds` node whose `semantic_type` is re-pointed at a collection of another element range, an `integer_range` node whose `semantic_type` is re-pointed at a node other than the `Integer`-keyed one, and a `collection_bounds` node whose `min` and `max` literal `type` is re-pointed at a genuinely keyed node of another type, each keep the stored `node_id` and each refuse the same way, because `semantic_type` and the literal `type` are in the preimage. A rekeyed `integer_range` typed at Boolean and a rekeyed `collection_bounds` typed at Boolean likewise refuse `invalid_package`/`stale-node-key` at the fresh `node_id`: a bounded form has no derivable key when its semantic type is not respectively the Integer node or a collection (`set`, `bag`, `sequence`, `ordered_set`) node. | Test (TC-226) |
| FR-038-AC-126 | Indirect redirection (IR-627; implemented over the in-repo fixture and selected-document field and operation contexts). Over fields and operations declared `Option<Int[0, 1000]>`, `Set<Int[0, 1000]>`, `Sequence<Int[0, 1000]>`, `Reference<O>` and an operation parameter typed `Int[0, 1000]`, a package in which the `option`, a collection or `reference` node keeps its stored key while its body `reference` is re-pointed at a genuinely keyed `Int[0, 10]` node, and one in which the parameter's type node holds the body `max` `10`, each refuses `invalid_package`/`stale-node-key` at the re-pointed or tampered node's `node_id`, and never `ill_typed`. A reader that re-derived only `integer_range` nodes admits the re-pointed `option`, collection and `reference` rows and fails. | Test (TC-226) |
| FR-038-AC-127 | Stage and order (IR-627 baseline implemented; extended IR-630 two-pass controls PLANNED and UNRUN). The derived-shape re-derivation, both IR-630 passes included, runs after the graph-shape stage and the application key stage and before the nominal key stage and every declaration, frame, state, temporal, abstraction and operation step. Owner joins precede both passes: an owner defect wins over a stale structural key even on a lower-key node. Within IR-630 the complete first pass precedes group-label comparisons. A package holding a graph-shape defect and a tampered node reports the graph-shape defect; one holding a stale application key and a tampered node reports the stale application key; one holding a tampered node and an `ill_typed` defect in an application that reads it reports the tampered node; one holding two tampered nodes reports the one whose own `node_id` digest is lower. Each report is `invalid_package`/`stale-node-key` at that node's `node_id` and never at the reading application. | Test (TC-226) |
| FR-038-AC-128 | Exact bounds at the extremes (IR-627; implemented over declared fields and in-repo fixture nodes). Over fields declared `Int[0, 0]` and `Int[-170141183460469231731687303715884105728, 170141183460469231731687303715884105727]` (the `i128` extremes), a node holding the declared bounds, the `i128` minimum as `min` included, admits and a node whose `max` differs from the declared bound by one in either direction refuses as AC-123 does. A comparison through a lossy float conversion would admit `"170141183460469231731687303715884105726"` against the `i128` maximum and fails this row. | Test (TC-226) |
| FR-038-AC-129 | Trust root (IR-627; implemented). A package whose `model_selections` row is re-pointed at a document declaring `Int[0, 10]`, whose node is re-keyed to `Int[0, 10]` and whose `package_id` is recomputed refuses `missing_import`/`missing-selection` at the row's `digest` when the evidence holds only the original document (FR-038-AC-27). The same package admits when the caller's evidence also holds the re-pointed document, because the reader admits a package against the evidence it is given, and the test records that admission as the stated limit of the trust root. | Test (TC-226) |
| FR-038-AC-130 | One derivation (IR-627; implemented by the unit test of `v2/derived_keys.rs`). For each of the ten derived shapes the key the admission stage derives from a node's own body equals the key `MemberType::node_key` derives from the matching member type, and a node whose body is built from the derived key's own preimage admits. A test derives both for each shape over the bounds of AC-128 and compares them. | Test (TC-226) |
| FR-038-AC-131 | PLANNED; new IR-630 controls UNRUN. Every structural form, declared or not and owner-bearing or not, is re-derived from its actual node members under the one QSL FR-092/FR-094 production authority. Nominal and application nodes retain their own key authorities; only correspondence/abstraction_relation is excluded from structural derivation. One-body-value mutations retaining old keys cover rational_range, decimal_range, float_rounding, text_bounds, model_population, compound_unit, text, parameter, union, record, tuple, value forms and owner-bearing clause functions. Each reaches invalid_package/stale-node-key at the least first-pass stale node_id. Existing owner-free ungrouped controls remain baseline evidence only; they do not prove the extended rows. | Test (TC-226) |
| FR-038-AC-132 | No copy (IR-627 checkout-reference harness implemented; IR-630 extended conformance PLANNED and UNRUN). Authoritative vectors for owner-bearing, declared and labelled-group forms remain at their owning QSL or QSpec checkout and are read there, not copied into this repository. A conformance read fails closed when its source is absent or malformed, as FR-038-AC-112 and AC-176 require. Test-side derivation controls do not replace published goldens. | Test (TC-226) |
| FR-038-AC-133 | Positive fixtures carry derived keys (IR-627; planned, ungated; supersedes FR-038-AC-107 for derived-shape nodes, which stands as built until then). The three fixtures AC-107 reads (`positive-all-families.json`, `positive-clause-operations.json`, `positive-union-nodes.json`) admit end to end with the derived key on every derived-shape node. The same fixture with the key of any one undeclared derived-shape node replaced by a placeholder, and every reference to it left in place, refuses; the nodes are `positive-all-families.json` nodes 24 (`integer`, `7f7f...`) and 26 (`reference`, `b2b2...`), `positive-clause-operations.json` nodes 0 (`boolean`, `aaaa...`), 7 (`integer`, `7f7f...`) and 12 (`reference`, `b2b2...`), and `positive-union-nodes.json` node 0 (`integer`, `a1a1...`), and `positive-all-families.json` nodes 0 (`boolean`, declared `Example::Flag`) and 2 (`integer_range`, declared `Example::Small`) are gated forms this row does not use. The refusal is `invalid_package`/`stale-node-key` at that node's `node_id`, and no placeholder key is tolerated. Owned by QSpec for those three fixtures and for the recorded `package_id` of `dependency-selection-vectors.json` (AC-113), which IR reads and whose base is `positive-all-families.json` (QSL-635, IR-627-Q5; QSL-635 also covers `positive-control-operations.json`, `positive-operation-identities.json` and `node-identity-vectors.json`, which IR does not read); `make conformance-qspec` fails on the three until they conform, so the IR code change merges after QSL-635's regenerated fixtures or a conformance lane is held citing QSL-635. QSpec also owns the conformance counterpart of the tamper regression row of TC-226: a negative fixture whose `integer_range` node keeps its id under a changed bound and whose `package_id` is recomputed from the seven selection fields and the mutated graph nodes in graph order, excluding only occurrences, which the reader refuses `stale-node-key` at that node (read from the checkout, never copied here). | Test (TC-226) |
| FR-038-AC-134 | In-repo fixtures carry derived keys (IR-627; implemented). Every in-repo fixture package (`tests/it/support/checked_package.rs`) carries on each undeclared node of the ten shapes the closed body of its form, its own key as `semantic_type` for a `scalar_type` or `composite_type`, and the derived key, and admits. The migration changes more than keys: the `aaaa` boolean's `literal` body becomes `aggregate{[]}`, and the `bbbb` option, typed at `aaaa` with an empty aggregate body, becomes self-typed with a body of one `reference`. The same package with the `aaaa` placeholder key restored on its boolean node refuses `invalid_package`/`stale-node-key` at that node's `node_id`, and with the `bbbb` body or `semantic_type` left unmigrated refuses the same way. Owned by this repository's code change. | Test (TC-226) |
| FR-038-AC-135 | Adverse mutations still reach their stage (IR-627; planned, ungated; supersedes AC-112's base package when the code lands). With `positive-all-families.json` carrying derived keys, each `structural_mutations` and `body_grammar_mutations` entry of `adverse.json` refuses with exactly its recorded `outcome`, among them `wrong-node-family-form` as `invalid_semantic_graph`, `negative-temporal-interval-bound` as `invalid_package` and the five `malformed_wire` body-grammar entries (eleven entries in all: six `structural_mutations`, five `body_grammar_mutations`), and none is refused `stale-node-key`; `dependency-selection-vectors.json` over the regenerated base carries its regenerated `package_id`. The harness fails, as AC-112 says, on a mutation refused at an identity check instead of its recorded code. | Test (TC-226) |
| FR-038-AC-136 | Fields as values (IR-628; implemented, ungated). Over an admitted package whose `model`/`object_type` node is a model declaration node with the empty body (the shape QSL emits), selected from a domain document declaring `balance: Int[0, 1000]` and `audit: Int[0, 1000]`, with a read of `balance` only, `model_object_fields` returns exactly two fields, `audit` then `balance`, each with `member_type` `Some(IntRange { lower: 0, upper: 1000 })` as `i128` values; `field("audit")` is `Some` although no read names it, and `field("missing")` is `None`. Mutation rows: a reader that takes the fields from the node body returns none; one that takes them from reads' `result_type` omits `audit`; one that returns the order of the document's declaration or of a hash map breaks the ascending-name row (a document declaring `zeta` before `alpha`). | Test (TC-227) |
| FR-038-AC-137 | Effective set is the one admission resolves (IR-628; implemented, ungated). Over a document with a supertype chain, an inherited field, a field redefined by a subtype with a narrower `Int[lo, hi]`, a most-derived redefiner two owners deep and an own field, the accessor returns each exposed name once, with the most-derived redefiner's type, and no hidden field; for every returned field, a package whose read of it is keyed by the independent recomputation of the returned type's key admits, and the same read keyed by the hidden base field's type refuses `ill_typed`/`operator-ineligible` (step 4). Mutation rows: inherited fields omitted; a redefined base field returned beside its redefiner; the base type returned for a redefined field; a less derived redefiner returned. | Test (TC-227) |
| FR-038-AC-138 | Every member kind (IR-628; implemented, ungated). Fields declared `Boolean`, `Integer`, `Int[0, 0]`, `Int[-5, 5]`, `Int[0, 9007199254740993]`, the `i128` extremes as `Int[lo, hi]`, a `Reference` to another object type, an optional `Int[0, 1000]`, a `Set`, `Bag`, `Sequence` and `OrderedSet` of `Int[0, 1000]`, a bounded collection `Set<Int[0, 1000]>[1, 4]` and an optional `Set` return `Boolean`, `Integer`, `IntRange` with the exact `i128` values, `Reference` carrying the key of the referenced model declaration node, `Option`, `Collection` with the matching `CheckedCollectionKind`, element and `Some((1, 4))`. Fields declared `Text`, `Rational`, `Decimal`, a systems interface type, a value type not bound as `Int[lo, hi]`, `Int[0, 170141183460469231731687303715884105728]` and `Int[-170141183460469231731687303715884105729, 0]` (one past each `i128` extreme), and a collection of lower bound 1 with no upper bound are present with `member_type` `None`, and the call returns the other fields (`Ok`, never a refusal). Mutation rows: bounds narrowed through `i64` (the `i128` extremes differ) or `f64` (2^53 + 1 differs); a bound past `i128` saturated, wrapped or truncated into an `IntRange`; `Option` dropped; a `Collection` kind swapped; a `None` field omitted; a `None` field refusing the call. | Test (TC-227) |
| FR-038-AC-139 | Absence and refusal semantics (IR-628; implemented, ungated). `UnknownNode` for an id whose digest is no node of the graph; `NotModelObjectType` for an admitted `scalar_type` node, a `systems_interface` declaration node and a `relationship` declaration node; a selected `model`/`object_type` declaration whose retained body is changed from `aggregate{[]}` refuses admission as `invalid_package`/`stale-node-key`, and an unselected model owner refuses at owner joining before the accessor is available; `AmbiguousField` carrying `x` for an object type whose document gives two unhidden effective fields named `x` (own and inherited, no redefinition), in package A, which no read of `x` names and which admits, the call returning the error for the whole call even though the object also declares a valid `Int[0, 1000]` field `y`; package B, the same document with a read of `x`, is refused at admission `ambiguous_declaration`/`ambiguous-name` and yields no package to call; `field("absent")` is `None` and the call is `Ok`. An intact selected declaration admits and returns its fields; changing only its unread closed body refuses at the retained node key, with no admitted package to call. Mutation rows: `UnknownNode` and `NotModelObjectType` collapsed into one; ambiguity resolved by returning the first or last field, or the other fields; an absent field returned as an error; the changed-body declaration being admitted or returning fields. | Test (TC-227) |
| FR-038-AC-140 | Independent of any node body (IR-628; implemented, ungated; holds before and after IR-627's stage lands). A crate-internal test over an admitted package (the module's own tests can build a `CheckedPackageV2` from an admitted one) replaces, after admission, the body bounds of the `Int[0, 1000]` node that a read of the field names with `0` and `10`, and separately `0` and `5000`, and the accessor returns `IntRange { lower: 0, upper: 1000 }` for the field, equal to its result before the replacement. Mutation row: an accessor that takes the bounds from the node a read names, or from any node body, returns `10` or `5000`. This row stays catchable once IR-627's stage refuses such a package at admission, because the replacement happens after admission. | Test (TC-227) |
| FR-038-AC-141 | One derivation with admission (IR-628; implemented, ungated). For each field of AC-138 whose type is a range, `Reference`, `Option` or collection, a read keyed by the key the test recomputes through `quire-canonical` from the accessor's returned type admits, and the same read keyed from the perturbed type refuses `ill_typed`/`operator-ineligible` at the read. The perturbation is: for an `IntRange`, `upper - 1` when `upper > lower`, `upper + 1` when `lower == upper < i128::MAX`, and `lower - 1` when `lower == upper == i128::MAX`; for a collection's bounds, `upper - 1` when `upper > lower`, `upper + 1` when `lower == upper < u64::MAX`, and `lower - 1` when `lower == upper == u64::MAX`; so every perturbed type node stays well-formed (`min <= max`) and is refused at the read as `ill_typed`, never for its own shape; for a `Reference`, the key of another object type's model declaration node; for `Boolean` and `Integer`, the other scalar; for an `Option` or a collection, the same wrapper over the perturbed element. Mutation rows: an accessor that derives types by its own copy of `field_type` which differs in the optional wrapper, a collection's bounds, or a `Reference` target. | Test (TC-227) |
| FR-038-AC-142 | Public shape and stability (IR-628; implemented, ungated). A compile-time API fixture shows `model_object_fields` takes `&self` and one `&CheckedNodeId` and no evidence; an exhaustive match over `CheckedMemberType`, `CheckedCollectionKind` and `CheckedModelFieldsError` with no wildcard arm compiles; code constructing `CheckedModelObjectFields` or `CheckedModelField` by struct literal fails to compile; the criterion is the compile fixtures: the exhaustive matches compile from an external crate (`tests/it`) only while the enums are closed. Mutation rows: `#[non_exhaustive]` added to an enum (the wildcard-free match fails to compile); a public field on either struct (the struct-literal fixture compiles); a variant added to an enum (the exhaustive match fails to compile). | Test (TC-227) |
| FR-038-AC-143 | Pure, total, bounded (IR-628; implemented, ungated). Two calls on one package return equal results; a clone of the package returns an equal result; two admissions of the same bytes under different byte limits (both admitting) are equal as packages and return equal results; a package admitted (under a `work` limit set high enough) from a document with an inheritance chain of 200 object types and 100 redefinitions of one field has a call that returns the fields without panic or stack overflow, and the call does no resolution (a counter on the table-building function of items 2 and 7 reads the same value before and after the call); a node of AC-139's refusal cases returns its error and never panics. Equality of two packages admitted from the same bytes is unchanged by the retained tables. Mutation rows: the accessor panics on an unwrap of a missing owner; it recurses over a 20,000-type chain on a 256 KiB thread stack; it resolves per call; equality of packages gains the retained tables (compared with a crate-internal package holding the same admitted content and cleared tables). | Test (TC-227) |
| FR-038-AC-144 | Resolution is charged once, at admission (IR-628; implemented, ungated). Cost model of items 8 and 9, with the arithmetic. For a chain of N object types, the first declaring one field and each later one declaring one field and extending the previous, the charge is 1 for the first type and k + 1 for the type at depth k >= 2 (one own field, one supertype edge, k - 1 copied entries), in all N(N + 1)/2 + N - 1: 13 for N = 4 (1 + 3 + 4 + 5) and 43 for N = 8. A crate-internal test reads the table-building counter and asserts 13 and 43. For the same chains, the smallest whole-read `work` limit that admits is M; M - 1 returns `incomplete` at the final selected-owner charge (`/lock/model_selections/0`), which follows graph structural-key validation. Separately, the limit one below the selected chain's last table-building charge returns `incomplete` at `/lock/model_selections/0`, preserving the table-before-graph precedence; these are distinct charges at the same pointer. A stale-key chain control finds the first budget that reaches its graph-stage stale-key refusal and shows that one below it returns work-`incomplete` at that node's `/semantic_graph/nodes/<n>/node_id` structural-key charge. Under `CheckedPackageReadLimits::bounded()` (work 1,000,000), a chain of N = 1000 (charge 1,000 x 1,001 / 2 + 999 = 501,499) admits and a chain of N = 1500 (1,500 x 1,501 / 2 + 1,499 = 1,127,249) returns `incomplete` at `/lock/model_selections/0`. Precedence (item 11), over one package each: a table charge that exhausts the limit together with a graph-stage refusal (a stale node key) returns `incomplete`, because tables precede `validate_graph`; a step 1 refusal at row 1 together with a table charge that exhausts the limit at row 0 returns the step 1 refusal, because every row's step 1 precedes any table charge. An object type with an ambiguous name (AC-139, package A) admits, and its table is charged as any other. Read charge (item 10): a read of a field of the N = 4 chain's deepest type charges 4 units (the four entries of its table, no hidden entries), a read of the root charges 1, and a read of an operation charges the unchanged step 3 walk; a frame entry naming a field and an abstraction relation's field entry on that deepest type each charge the same 4 units, a relationship edge read charges the entries of its table, and none runs the ancestor walk for a field (a counter on the walk stays at its pre-resolution value); a state clause resolves only operations and is unchanged. Cycles (items 4 to 6, 9), each pinned against what `resolve` returns today (the expected tables are recorded by running the pre-change `resolve` over each document before the code change and stored as the test's expected values, so they are not asserted from reading the code): a type `A extends A` declaring one field `f` admits, its table is `{f}` and the charge is 2; types `A extends B` with field `a` and `B extends A` with field `b` admit, the tables of `A` and `B` are the one shared table `{a, b}` and the charge is 4; `A extends A` declaring `f` and `g` where `g` redefines `f` admits and its table equals what `resolve` returns for `A` today (by reading `resolve`, `g` hides `f` and, since `A` is its own ancestor, itself, so neither is exposed); a document with a third type `C extends A` (outside the cycle) has a table equal to `resolve`'s for `C`, charged with the entries copied from the cycle's table. Mutation rows: tables computed after admission or per call (the limit never exhausts); the charge omitted for an unread object (the one-below limit admits); tables built after the graph stage (the precedence row returns the graph refusal); a per-type ancestor re-walk (the counter exceeds 13 for N = 4); an ambiguous table refusing admission; a field resolution (read, frame entry, abstraction field entry) that still runs the walk or charges nothing; a cycle refused at admission, built without termination, or built per member (the charge exceeds 4 for the two-cycle); a cyclic table differing from `resolve`'s. | Test (TC-227) |
| FR-038-AC-145 | PLANNED (IR-630); UNRUN. Correctly keyed and labelled record List { next?: List; }, record Tree { kids: Sequence<Tree>[0, 3]; } and the owner-free QSL G1 self-option group admit through both structural-key and group-digest passes. Every structural member is authenticated, including the declared record, option, sequence and collection_bounds nodes. The source-owned positives use actual joined wire owners. An application member retains its separate application key check and contributes its local preimage to the group digest. A reader that still skips any member fails the tamper controls of AC-148. | Test (TC-226) |
| FR-038-AC-146 | PLANNED (IR-630); UNRUN; retires ordinary-ungrouped-key-plus-label admission. Boolean, Integer, integer_range and reference retain their existing closed-shape checks and are never skipped because of a label or cycle. Adding a lone label to a correctly keyed ungrouped Int[0, 1000] node while retaining its key, mirroring projection and recomputing package identity now refuses invalid_package/stale-node-key at its node_id, with or without changing max to 10. The same stale-key mutations over Boolean, Integer and reference refuse. Correctly rekeyed and labelled nodes are judged by both published passes and existing shape checks, without an invented SCC-membership refusal. | Test (TC-226) |
| FR-038-AC-147 | PLANNED (IR-630); UNRUN; retires off-cycle-same-label-ordinary-key admission. All nodes sharing a retained label participate in its wire-order group preimages, even off the names cycle. Adding an off-cycle Set<Int[0, 1000]> node carrying the Tree group label and its unchanged ordinary ungrouped key, then mirroring projection and refreshing package identity, refuses invalid_package/stale-node-key at the least first-pass stale node_id; it cannot admit by verifying the extra node as ungrouped. A lone labelled acyclic collection_bounds node with its old key also refuses. Dependencies do not supply group ordinals or a skip exemption; existing dependency/reference graph checks retain their precedence. A fully rekeyed and relabelled group containing an off-cycle member is judged by FR-322, not refused merely for being off-cycle. | Test (TC-226) |
| FR-038-AC-148 | PLANNED (IR-630); UNRUN; replaces former residual admissions with refusal regressions. On correctly keyed Tree, keep node keys while separately changing collection_bounds max to 5, min to 1, its semantic_type to another group member, or Sequence<Tree> body reference to another member; mirror projection and refresh package identity. Each refuses invalid_package/stale-node-key under the two-pass order. Forge a self-option by starting from a correctly keyed ungrouped Option<Integer>, changing its body reference to its own retained node_id and adding a label, retaining the old key, mirroring projection and recomputing package identity: the production reader refuses stale-node-key at that option node_id. Also refuse the old bounded-set/collection_bounds forged cycle and the integer_range-to-collection_bounds retag under retained keys. A reader that accepts old keys on these cycles fails; genuine G1 remains the positive counterpart. | Test (TC-226) |
| FR-038-AC-149 | PLANNED (IR-630); UNRUN. Group membership, every structural-key visit, local-preimage term visits and member/group digest computations are charged before work as specified above, including owner-bearing and off-cycle labelled members. For a fully valid Tree read, independently account for these visits and find the exact smallest total work M that admits; work M-1 returns work-incomplete at the first unpaid value with limit, consumed count and pointer, and exposes no package. Separately, select a charge inside group-local preimage or group-digest work on the valid Tree, with at least one later group visit or digest still pending, and set the existing work ceiling so that this selected charge is denied. The result is work-incomplete at that failed charge's pointer with no admitted package; observations of actual group visits and digest computations show none after the denied charge. A genuine mutant that continues the production group walk after this unpaid charge makes the pending visit or digest observable and fails this control; restoring immediate propagation of the charge error restores the control. These internal-group exhaustion and mutant/restored controls are PLANNED and UNRUN, separately from whole-read M/M-1. A crate-local counter also proves that every labelled member contributes. The walk is iterative and encoding remains fallible under existing finite byte/work limits, with no added or raised ceiling. | Test (TC-226) |
| FR-038-AC-150 | PLANNED (IR-630); UNRUN; the prior owner gate and SCC skip are retired. First authenticate each structural node in ascending retained node-id digest order using actual joined owner/declaration, null recursion outside a labelled group, or its wire-order group-local preimage with recursion.group equal to the retained label. Then recompute every label from all member-local preimage digests in wire ordinal order. A stale node key reports invalid_package/stale-node-key at that node_id; a stale label reports that refusal at the least retained member, with failing groups ordered by least member. A package with both a higher-key stale structural node and a lower-key failing group label reports the first-pass node. An owner mutation with old keys, a retained-key wire permutation, and a package rekeyed under an incorrect label each refuse at their specified pass. A complete wire permutation with every key, label, reference, projection and package identity correctly recomputed is judged by the published wire-order contract and is not refused merely for differing from producer content order. No content-order recomputation or SCC admission rule is added. | Test (TC-226) |
| FR-038-AC-151 | Implemented by IR-644 and verified by TC-048. Over the QSL-shaped recursive `List` record with an integer field and `next` encoded as `binding(next, aggregate([binding(optional, reference Option<List>)]))`, `structural.eq` over two `List` values admits with `leaves` empty, and the leaf derivation terminates through the option's `inner` edge at the record reentry. Replacing that integer field with a text field whose profile is selected admits exactly its text leaf followed by `["field:next", "inner", "recursion:0"]`; omission of that recursion leaf refuses `invalid_package`/`operation-law-missing` at `operation.leaves`. Adding a healthy text sibling after `next` retains its own leaf after the recursion leaf in declaration order, proving that the wrapper consumes exactly one field edge. A direct field `reference` to `Option<List>` also admits with the same `field:next`, `inner` path. | Test (TC-048) |
| FR-038-AC-152 | Implemented by IR-644 and verified by TC-048. With the rest of the `List` comparison well formed and its identity members recomputed after each mutation, replace only the value of `next` by an `aggregate` with no member, two `optional` members, a member named otherwise, a non-binding member, or an `optional` binding whose value is not a reference to an option type. Each mutation is admitted by the flat body grammar; after identity re-derivation, each reaches the operation check and refuses `ill_typed`/`operator-ineligible` at `operation.leaves`, rather than silently omitting `next` or deriving a leaf from a different member. | Test (TC-048) |
| FR-038-AC-153 | A declared record, tuple and function each carries its source owner in the node and equal identity projection; QSpec's two-owner packages for `Point` and recursive `List` admit with distinct `Point` ids, distinct `List` group labels and member ids, and equal builtin `Integer` and application-keyed `three` ids under the two owners. An undeclared model declaration and clause function each carries its model owner, while an anonymous type, `source_locus` node and application-keyed node carry no owner. | Test (TC-228) |
| FR-038-AC-154 | Omitting a required node owner, inserting null, using the wrong owner kind, adding version to ModelOwner or placing an owner on an owner-free node refuses malformed_wire at the node that lacks a required owner or at the present owner, before identity validation. No missing owner is reconstructed from occurrences, source map or lock. A shipped identity projection is refused by AC-310 before any obsolete projection-owner comparison. | Test |
| FR-038-AC-155 | Before structural key re-derivation, a `SourceOwner` absent from `lock.sources` refuses `missing_declaration`/`missing-selection` at its node's `node_id`; a source-map region of its `declaration` occurrence naming another source pair refuses `invalid_package`/`invalid-value` at that source-map entry even when both pairs are lock-selected. A `ModelOwner` whose `identity` is unselected, whose `node` names no declaration, or whose `node` names the wrong kind refuses `missing_declaration`/`missing-selection` at its node's `node_id`, even if nothing reaches that node: a relationship cannot back `model`/`object_type`, an object type with `interfaceFeatures` cannot back `model`/`object_type`, an object type without `interfaceFeatures` cannot back `model`/`systems_interface`, and a field cannot back an operation clause. An operation member's clause and an object type's invariant pass. Two owner defects select the lowest retained node-id digest, ahead of a stale key on a lower-key node. Each selected-document lookup charges one validation visit. | Test (TC-228) |
| FR-038-AC-159 | For admitted `quire.op.integer.add`, `quire.op.integer.sub`, `quire.op.integer.mul` and `quire.op.integer.negate` applications whose every operand has finite bounds representable in `i128`, `scalar_application_operands` returns one entry per argument, in wire argument order, with ordinals `0..n-1`, typed identities and exact inclusive `i128` ranges; the unary operation returns one entry and the binary operations return two. Swapping two arguments swaps their entries and ordinals without sorting by child id. | Test (TC-048) |
| FR-038-AC-160 | A reference to a `value`/`parameter` returns `GraphChild` carrying that parameter's node id and its admitted integer-range bounds; a reference to an application subterm returns `GraphChild` carrying that subterm's own node id and result range. Two distinct children of the same integer type remain distinct identities. An admitted graph literal reference carries the literal node id and singleton `(value, value)`, independent of a wider declared type bound. | Test (TC-048) |
| FR-038-AC-161 | For an admitted inline integer literal argument of value `7`, the result carries `InlineLiteral` naming the application node id, the supplied occurrence key and that argument's ordinal, with range `(7, 7)`; another position or occurrence yields a distinct identity. No `CheckedNodeId` is fabricated for the inline term. | Test (TC-048) |
| FR-038-AC-162 | An unknown graph node, a known non-application node, an absent occurrence key, a dangling referenced child and a child without a readable integer range produce the distinct typed errors `UnknownNode`, `NotApplication`, `MissingOccurrence`, `MissingChild` and `MissingRange`, respectively. An admitted reference to unbounded `Integer` returns `UnboundedRange`; an admitted integer-range endpoint, graph literal or inline literal outside `i128` returns `RangeOutOfI128`. Each refusal returns no partial list, narrowed value or panic. The dangling-child row may use a crate-internal mutation after admission because the reader ordinarily rejects it. | Test (TC-048) |
| FR-038-AC-163 | A crate-internal mutation of an admitted application to an operation identity absent from the catalog returns `UnknownOperator`; an admitted application with a catalogued identity outside the four eligible integer operations returns `IneligibleOperator`. The accessor does not interpret a non-integer operation as an eligible scalar parity claim. | Test (TC-048) |
| FR-038-AC-164 | An external Rust API fixture calls the accessor with `&CheckedNodeId` and `&CheckedOccurrence`, exhaustively matches the child-identity and error enums, and reads each ordinal, node id or inline selector and range through typed fields; it does not read `graph().nodes[*].body` or deserialize JSON. Repeated calls and a cloned admitted package return equal results, and neither call changes package equality. | Test (TC-048) |
| FR-038-AC-165 | PLANNED / UNRUN (IR-661). A selected, independently authored FCD-shaped document with a global `relationship` slot identity admits, while an owner-nested relationship, a foreign-package identity and a wrong-slot identity each refuse `invalid_model_binding`/`malformed-declaration`. Owner-nested fields, operations and operation parameters still admit; moving a field or operation into the global relationship slot refuses `invalid_model_binding`/`malformed-declaration`. | Test (TC-048) |
| FR-038-AC-166 | PLANNED / UNRUN (IR-661). Changing only an authored role makes the old name refuse `missing_declaration`/`missing-name` and the new name resolve on the same selected relationship. Destination-type and multiplicity changes are observed through the admitted navigation result nodes of AC-170, not through an unspecified retention accessor. Omitting only `targetEnd.role` admits the document with only its forward named end. Removing `sourceEnd.role`, or setting either present role to null, empty or a non-string, refuses `invalid_model_binding`/`malformed-declaration`; no role is synthesized or checked against an inverse registry. | Test (TC-048) |
| FR-038-AC-167 | PLANNED / UNRUN (IR-661). An unresolved `sourceEnd.type` or `targetEnd.type` refuses `missing_declaration`/`missing-name`; a type naming a relationship or another declaration of the wrong meaning refuses `invalid_model_binding`/`malformed-declaration`, independent of declaration order. A source end naming another owning type, a malformed end or multiplicity refuses `invalid_model_binding`/`malformed-declaration`; `lower > upper` alone refuses `invalid_model_binding`/`unpreserved-model-meaning`. Each declaration refusal keeps its typed code, cause and selection-row pointer. This criterion verifies refusal semantics only; private metadata retention is verified separately by the dedicated criteria under the declaration-refusal retention contract. | Test (TC-048) |
| FR-038-AC-168 | PLANNED / UNRUN (IR-661). Repeating a relationship identity, including under two different owners, refuses `invalid_model_binding`/`conflicting-binding` at the selection row. Two missing relationship identities are malformed declarations, not conflicting identities. A malformed role beats an unresolved end type and reversed multiplicity on that relationship; an unresolved end type beats reversed multiplicity. Referencing a node refused for identity or kind keeps its own refusal rather than reporting a missing end type. With other limits sufficient, the measured successful selection-stage work admits that stage; one below its cumulative charge threshold returns `incomplete` at `/lock/model_selections/<i>` with no admitted package. This is separate from AC-171's total-reader work limit. | Test (TC-048) |
| FR-038-AC-169 | PLANNED / UNRUN (IR-661). `relationship_end.declaration` names the selected relationship's `relation`/`relationship` graph node and resolves through its content-only `ModelOwner`, with an empty body; a valid source or target object node used as its declaration refuses `ill_typed`/`operator-ineligible` at `member.declaration` after owner recovery and before role lookup, without invalidating that object owner. Source role lookup yields the forward end and a present target role yields the inverse end, by exact role bytes rather than the terminal identity segment. An unknown role or absent inverse refuses `missing_declaration`/`missing-name` at the member's `name` before receiver, direction or result checks; two matching roles refuse `ambiguous_declaration`/`ambiguous-name`. A wrong-kind or missing relationship owner keeps the existing owner-join refusal, with no role fallback. | Test (TC-048) |
| FR-038-AC-170 | PLANNED / UNRUN (IR-661). Forward and inverse navigation choose opposite receiver/destination endpoints and admit the canonical result nodes specified by QSpec FR-152 Navigation. The independently authored finite cases in TC-048 exercise Reference, Option, Set and Bag results; changing only destination type or bounds changes which result node admits. An inherited end admits on a subtype receiver with the same destination type; an unrelated object receiver refuses `ill_typed`/`operator-ineligible`. Wrong result or direction refuses `ill_typed`/`operator-ineligible` at the declared IR path. An eligible direction with an unbounded or ordered destination refuses `unsupported_construct`/`expression-form` at `member.name`, using the relationship-navigation-only variants named in AC-101. A named inverse on a source-to-target relationship refuses direction before unsupported destination multiplicity. | Test (TC-048) |
| FR-038-AC-171 | PLANNED / UNRUN (IR-661). For `quire.op.model.navigate`, the new relationship-end checks follow the specified operation-step order, explicit calling-node locus and RFC 6901 paths: owner recovery and declaration-kind eligibility before roles; roles before receiver, direction, multiplicity and result comparison. A missing role combined with a wrong receiver, disallowed direction and wrong result reports `missing_declaration`/`missing-name` at `.../operation/member/name`; a valid role with wrong receiver reports at `.../body/arguments/0`; a mismatched result reports at `.../body/result_type`. A successful complete read admits at its measured exact total work limit. One less returns `incomplete` at the first unpayable charge in the existing stage order, without a partial package: operation validation precedes later graph-body reference-edge charges, so the pointer may be `.../body/arguments/0/target`, not the selection row. Selection-stage-minus-one has the separate selection-row oracle of AC-168. | Test (TC-048) |
| FR-038-AC-172 | PLANNED / UNRUN (IR-661). A homogeneous self-relationship with a resolved forward or inverse end and an eligible Reference or Option destination admits `quire.op.model.reaches` with two references conforming to its static endpoint owner and Boolean result, including subtype operands. An unrelated operand, heterogeneous destination or otherwise ineligible Set/Bag edge refuses `ill_typed`/`operator-ineligible` at its operand or member-name path. Missing role and object-node declaration produce AC-169's respective typed outcomes; a wrong result node refuses at `body.result_type`. No operand is treated as Reference to the relationship graph node. FR-038-AC-252 supersedes the otherwise-ineligible Set/Bag clause for `quire.op.model.reaches`. | Test (TC-048) |
| FR-038-AC-173 | PLANNED / UNRUN (IR-661). Removing direction, category, composite or origin, setting each to null or a wrong type, using an unsupported direction/category value, or supplying a malformed common-schema origin branch refuses `invalid_model_binding`/`malformed-declaration` at the selection row before application resolution. Code/cause/pointer and check order remain unchanged. Schema-valid source and generated origins admit without a default direction. This criterion verifies origin validation and admission/refusal semantics only; private metadata retention is verified separately by the dedicated criteria under the declaration-refusal retention contract. | Test (TC-048) |
| FR-038-AC-174 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 identity retention). When FR-154 intake selects an offending declaration with a supplied string identity, the reader retains that exact FCD identity in the private refusal returned by `admit_selection`, including refused object-id spelling. Missing or non-string identity yields None; no graph digest, inferred owner, repaired string or separate artifact id supplies it. | Test (TC-048) |
| FR-038-AC-175 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 private constructors). Each private `SelectionRefusal` constructor and declaration-to-`SelectionFailure::Refused` conversion preserves the selected retained metadata record through the private `admit_selection` return after document release. Independently dropping available identity/origin or substituting a different declaration's record fails the typed return assertion. Pre-declaration constructors preserve genuine metadata absence. | Test (TC-048) |
| FR-038-AC-176 | IMPLEMENTED (IR-654 code; external conformance target passes). With `QUIRE_SPECIFICATION_DIR` naming the authoritative QSpec checkout, `make conformance-qspec` requires all nine `positive-*.json` packages under `proposals/checked-package-v2/fixtures/`: `positive-all-families.json`, `positive-clause-operations.json`, `positive-control-operations.json`, `positive-nominal-identities.json`, `positive-operation-identities.json`, `positive-recursive-records.json`, `positive-two-owners-a.json`, `positive-two-owners-b.json` and `positive-union-nodes.json`. Each admits through the production reader with its published `package_id`. For a package selecting `acme/orders`, the harness supplies `proposals/checked-package-v2/domain-package-acme-orders.json` from the same checkout under its selected `sha256-jcs` digest and the reader checks the bytes against that selection; this extends AC-107's existing positive read for packages with `model_selections`, not a second skipped read. A missing or malformed document, absent required positive fixture, unknown selected model identity, missing selection digest or non-admitted package fails the run. For the two-owner fixtures, read each node id and owner back through the admitted package's graph accessor and compare with the published graph and its derived identity-projection entries: the corresponding declared structural nodes have different ids across source owners, as AC-153 requires. This existing control checks published wire/admission. The planned IR-630 extension passes only when those production reads additionally authenticate the actual-owner structural keys and group labels. That extension is UNRUN, and the existing trace tags are not evidence for it. For every `model_declaration_nodes[*].preimage` in `proposals/checked-package-v2/model-member-type-vectors.json`, compare its `sha256` with the digest from the reader's production `declaration_key` path (`StructuralPreimage`/`structural_key` in `model_members.rs`), or admit its published wire node through that same production derivation; a harness that only hashes the vector's JSON preimage cannot pass. `node-identity-vectors.json` in that proposal remains the separate nominal/application-vector source; `model-effective-declaration-vectors.json` describes another model artifact and is not the structural-key oracle. No fixture document, source pin or digest catalog is copied into this repository. Live QSL emission followed by IR reading belongs to integration work outside this criterion. | Test (TC-048) |
| FR-038-AC-177 | Implemented by `composite_application_operands`; verified by the TC-048 composite operand tests. An admitted structural.eq over supported parameter references returns the actual application and supplied occurrence with two typed operands in argument order, ordinals 0 and 1. Swapping arguments swaps entries; repeating a parameter retains two entries. A second authentic occurrence is retained exactly; an absent occurrence refuses MissingOccurrence with the supplied application and occurrence. | Test (TC-048) |
| FR-038-AC-178 | Implemented by `composite_application_operands`; verified by the TC-048 composite operand tests. A supported closed graph composite literal retains its own GraphChild id and Literal disposition with no introduced positions even under a wider type. A value subtree reading a parameter refuses UnsupportedOperand with enclosing ordinal and reason NonliteralGraphValue; an application subterm refuses reason ApplicationSubterm. Defensive inline integer and noninteger term cases refuse reasons InlineInteger and InlineNonInteger, without fabricated identity or value. No publicly admitted structural.eq or structural.ne integer-inline success is claimed: both owning catalog entries exclude Integer from structural_kind; future success requires an owning eligibility/value specification. | Test (TC-048) |
| FR-038-AC-179 | Implemented by `composite_application_operands`; verified by the TC-048 composite operand tests. A record containing bounded Sequence of integer-range elements returns its exact field/collection/element paths, Node keys rooted at the operand, and both collection endpoints. Changing only one path's source type changes that descriptor. Bounds beyond i128/u64 retain canonical decimal text; unbounded integer/collection have explicit unbounded descriptors without guessed maxima. | Test (TC-048) |
| FR-038-AC-180 | Implementation exists in `composite_application_operands`; the corrected 18-form code evidence is PLANNED/UNRUN (IR-491), and the prior 18-form public-admission claim does not discharge it. Tuple, direct Option and optional-presence record edges preserve the specified ordinals without wrapper segments; List with optional tail has Depth key [] and reentry [1,0]. Tree with two optional self-fields retains both [0,0] and [1,0] at Depth key []; shared nonrecursive siblings retain distinct paths without Depth. Boolean introduces no position. Enum retains ordered flag and exact preimage member order, including unordered Blue, Green, Red. Each of the eleven scalar and seven bounded-domain forms has its specified type-walk descriptor or absence, including one Whole at a `text_bounds` wrapper and a Node-keyed Whole without Population key for population-typed leaves. A public reader-admitted comparison over each of the fifteen non-IEEE forms returns its independently enumerated introduced-key set; isolated private type-walk unit controls over `float32`, `float64` and `float_rounding` return their tabled descriptors. Public structural comparisons reaching any of those three forms instead refuse `ill_typed`/`operator-ineligible` at `operation.leaves` under AC-231; no public accessor result is claimed for them. | Test (TC-048) |
| FR-038-AC-181 | Implemented by `composite_application_operands`; verified by the TC-048 composite operand tests. The public IneligibleOperator assertion uses a reader-admitted `boolean.and` application outside eq/ne (AC-199). Every one of the twelve specified error variants exposes exactly its tabled payload and no partial result or panic. Unsupported union, malformed optional wrapper and unanchored cycle carry the actual enclosing ordinal/type locus. Zero and one below measured successful work refuse WorkLimit; that exact budget succeeds. Checked-index failure returns PositionOutOfRange with authentic available ordinal/type metadata. Equivalent package node order and lookup strategy leave logical consumed_work unchanged; an earlier operand refusal or exhausted charge beats a later defect. | Test (TC-048) |
| FR-038-AC-182 | Implemented by `composite_application_operands`; verified by the TC-048 composite operand tests. An external consumer calls the named public accessor and exhaustively matches all named output, descriptor, edge, key and error types using the specified fields without body JSON or a QSL model dependency. Repeated/clone calls yield equal content and unchanged package equality. API inspection finds no harness-bound inputs, coverage verdict or obligation encoder; structural.ne eligibility is verified separately by AC-197 through AC-201. | Test, Inspection (TC-048) |
| FR-038-AC-183 | Implemented and verified by TC-048 and TC-226 (IR-680 code). For an unreferenced owner-free, ungrouped node in each of the `derived_key` closed-shape and `UngroupedPreimage` branches, a retained node id that differs from the production-derived key refuses `invalid_package`/`stale-node-key` at the original `node_id`, keeps that original typed id in `locus`, and returns the distinct derived typed key through `expected_node_id()`. A fresh package whose unreferenced node is rekeyed to that value and whose package identity is refreshed admits; one changed preimage member changes the expected digest. For each of the eight self-typed forms, a changed `node_id` can yield a key only when its `semantic_type` changes with it; a stale `semantic_type`, a non-self-typed shape with inconsistent semantic type, a `collection_bounds` over a noncollection type, and a malformed closed shape yield `None`. AC-185 governs shared retention and absence. | Test (TC-048, TC-226) |
| FR-038-AC-184 | Implemented and verified by TC-048 (IR-680 code). An unreferenced application node whose retained key differs from the digest already computed by `validate_application_keys` refuses `invalid_package`/`stale-node-key` at its original `node_id`; `locus` remains that original typed id and `expected_node_id()` returns the distinct computed typed key. A fresh package rekeyed to that value, with package identity refreshed and no referrer to cascade stale keys, admits; changing one application preimage member changes the expected digest. The value comes from the existing application computation, not a second hash or a diagnostic string. AC-185 governs shared retention and absence. | Test (TC-048) |
| FR-038-AC-185 | Implemented and verified by TC-048 (IR-680 code). The expected key is a private, bounded `Option<CheckedNodeId>` exposed only by `expected_node_id() -> Option<&CheckedNodeId>`; it participates in derived `Clone`/`Eq`/`PartialEq`, may appear in derived `Debug`, and is absent from Display, document pointer, locus text and serialized forms. A fixed-member model declaration shape failure, a canonical encoding failure and a pre-key-stage refusal return `None`; every IR refusal constructor and public V2/dispatch forwarder preserves an authentic value or absence, and IR's independent test-authored expected values check the actual reader field by field. Existing code, cause, path, locus, first-refusal order, byte/work charges and public key-derivation surface remain unchanged; no public synthetic-refusal constructor or derive/rekey API is added. The key is meaningful only with the reader-returned original `locus`, since callers can mutate public fields on a cloned refusal. | Test, Inspection (TC-048) |
| FR-038-AC-186 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 source origin). Given a located refusal with a valid FCD Source origin, the reader returns its exact typed source member values and optional-member presence at the private intake boundary, including all four end-coordinate presence combinations and the existing admitted 2^53 boundary. Retention adds no paired-end or span-order constraint. | Test (TC-048) |
| FR-038-AC-187 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 generated origin). Given a located refusal with a valid FCD Generated origin, the reader returns exact generator identity/version text and input identity sequence at the private intake boundary. Input order/repetition remains unchanged and no source span is present. | Test (TC-048) |
| FR-038-AC-188 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 selected nested context). When an existing field declaration/type reference, operation declaration/return type, parameter declaration/type reference or relationship-end check refuses, the reader retains the actual offending field, operation, parameter or relationship's supplied identity and valid origin. Missing nested metadata remains None; an enclosing owner or unresolved target cannot substitute. | Test (TC-048) |
| FR-038-AC-189 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 origin absence). Given a located refusal with missing or malformed origin, the reader returns None for origin while independently retaining any available declaration identity. Both-branch, partial-branch, unknown-member and invalid-member cases cannot yield a salvaged branch or default origin. | Test (TC-048) |
| FR-038-AC-190 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 same-identity type-group refinement). For every same-identity type group as defined above, including malformed-declaration and conflicting-binding refusals, the reader retains its common supplied identity and retains origin only if every candidate has a valid origin and all typed values and optional-member presence agree. Any missing, malformed or different valid candidate origin yields None. Candidate permutation cannot select a winner. This IR-local ambiguity refinement does not apply type-group consensus to a sequential relationship refusal. | Test (TC-048) |
| FR-038-AC-191 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 refusal precedence). Retaining metadata does not change the existing first code, cause or selection-row member in node/table/member-path order. Combining declaration defects preserves that exact first refusal; distinct Limit and InexactNumber alternatives retain their current payloads and precedence. | Test (TC-048) |
| FR-038-AC-192 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 bounded retention). Retained metadata is acquired and released within the existing bounded document/read/work accounting, without a second parse, raised ceiling or unbounded diagnostic collection. Exact sufficient work succeeds; the first unpayable charge preserves the existing incomplete result and no partial package. | Test (TC-048) |
| FR-038-AC-193 | IMPLEMENTED; source Inspection completed (IR-663 public boundary exclusion). Inspection of the IR-owned model-intake conversion confirms that it projects code/path/cause to ValidationFailure and adds no declaration metadata field to CheckedPackageRefusal or the IR-owned dispatch result. Unrelated locus, expected-node-key, contract-version and document-pointer behavior remains intact. | Inspection (TC-048) |
| FR-038-AC-194 | IMPLEMENTED; source Inspection completed (IR-663 production ownership). Inspection passes only when full typed-origin equality determines production group-origin retention, metadata is owned by SelectionRefusal through SelectionFailure::Refused after admit_selection releases the document, and that owner is released by the end of the existing SelectionFailure::Refused(refused) mapping arm to ValidationFailure in checked_package/v2/mod.rs. Explicit teardown or ordinary scoped-owner destruction passes; a field read affecting neither origin retention nor ownership transfer, a read solely to silence lint, a blanket dead_code allowance or a new observer/public API fails. | Inspection (TC-048) |
| FR-038-AC-195 | The repository lint gate runs both Clippy lanes with warnings denied: workspace all-targets and the model consumer with test-only fault injection disabled. CI invokes that same gate. A compiler failure fails the gate; a suppression or fake observer does not satisfy it. | Analysis (`make lint`) |
| FR-038-AC-196 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 sequential relationship context). A relationship identity repeated sequentially, including under two different owners, retains the later actual offending relationship's supplied identity and its own valid origin under the existing node/member-path order. Its missing or malformed origin yields None. The earlier relationship or enclosing owner cannot substitute, and type-group origin consensus does not apply. | Test (TC-048) |
| FR-038-AC-197 | Implemented (IR-690) by `composite_application_operands`; TC-048 checks an admitted structural.ne application over supported parameter and closed graph-value operands, its actual application, selected authentic occurrence and positional child entries, and exact ordered identities when arguments are swapped or repeated. | Test (TC-048) |
| FR-038-AC-198 | Implemented (IR-690); TC-048 compares independently admitted structural.eq and structural.ne operand projections with the same authored type/child structures, including canonical bounds, optional/reentry paths, enum order and closed-literal disposition. The shared projection introduces no complement, drawn maximum or altered domain. | Test (TC-048) |
| FR-038-AC-199 | Implemented (IR-690); TC-048 checks MissingOccurrence with the supplied application and occurrence, exact refusal variants and ordinal/type-node payloads for unsupported inline/subterm and union operands, and a reader-admitted `boolean.and` IneligibleOperator target. Crate-internal admitted-then-mutated Ne probes check malformed domains, nonstructural operands and first-defect order; these defensive probes are not reader-admission evidence. | Test (TC-048) |
| FR-038-AC-200 | Implemented (IR-690); TC-048 independently counts the shared logical-work charges and checks exact-limit success, one-less and zero WorkLimit payloads, equal eq/ne work, and equal work after reversing admitted graph-node order. | Test (TC-048) |
| FR-038-AC-201 | Implemented (IR-690); source inspection finds one shared retained-index/family/catalog/reference-resolution and domain-projection owner for eq/ne, with the existing public signature/types and scalar/admission boundaries unchanged; no per-call uncharged index, second traversal owner, depth ceiling, encoder or new dependency was introduced. | Inspection (TC-048) |
| FR-038-AC-202 | Implemented (IR-703); tagged local controls exist. For an admitted invariant, precondition or postcondition clause with an authentic claim occurrence and a direct condition reference to a binary integer.eq/ne/lt/le/gt/ge application, the typed accessor returns the actual clause id, claim, kind, comparison id and closed operator; the comparison id alone supplies no observation. | Test (TC-445) |
| FR-038-AC-203 | Implemented (IR-703); tagged local controls exist. For two eligible reads, the accessor returns exactly ordinals 0 and 1 in authored order, preserving distinct StateField or OperationInput provenance and exact child identities; reversing arguments reverses entries without sorting or deduplication. CG maps only typed StateField to State and OperationInput to Input for its FR-008 census; no name-based V1 kind inference occurs. | Test (TC-445) |
| FR-038-AC-204 | Implemented (IR-703); the tagged QSL producer-origin case runs only under the named conformance gate. For a fresh QSL-produced and IR-admitted ConfigVersion postcondition `self.versionNumber = pre(self.versionNumber)`, the shared record.project id appears in both entries, the direct path is Post and the state.pre path is Pre. Synthetic hand-built topology is separately labelled and cannot discharge this producer-origin assertion. | Test (TC-445) |
| FR-038-AC-205 | Implemented (IR-703); tagged local controls exist. A read whose selected model field or operation-parameter type supplies a finite integer range returns its exact inclusive i128 endpoints, including i128::MIN and i128::MAX; a literal in either position returns an exact i128 singleton with its graph or inline identity. Unbounded Integer, an out-of-i128 literal, and defensive unavailable model member type produce their distinct typed refusals without truncation, body-derived guess or partial result. | Test (TC-445) |
| FR-038-AC-206 | Implemented (IR-703); tagged local controls exist. The accessor accepts direct self.field reads and operation-parameter references in the reader-checked FR-341 aggregate, plus one state.pre wrapper around a StateField in a postcondition. Direct StateField is Current in invariant/precondition and Post in postcondition; direct OperationInput is Current. An alias, result slot, unrelated parameter, nested project or other wrapper is IneligibleOperand. A read plus literal is eligible; two literals return NoRead. | Test (TC-445) |
| FR-038-AC-207 | Implemented (IR-703); tagged local controls exist. Unknown clause, wrong node form, absent claim, unsupported clause kind, missing comparison, unknown or ineligible operator, invalid arity, missing child and malformed operand yield their respective typed refusals in the stated first-defect order. Each carries clause id, claim, structural condition/operand path and reached child id when available, without a claimed unique child source region or partial result; malformed post-admission graph never panics. | Test (TC-445) |
| FR-038-AC-208 | Implemented (IR-703); tagged local controls exist. Two clause claim occurrences over one content-addressed comparison produce equal contextual operands apart from the supplied claim. The accessor returns no comparison, project or literal expression occurrence and claims no claim-to-child source-map association; a separate source-map lookup may enumerate regions. | Test (TC-445) |
| FR-038-AC-209 | Implemented (IR-703); tagged local controls exist. An external Rust consumer calls the public accessor with &CheckedNodeId and &CheckedOccurrence and exhaustively matches closed operator, provenance, observation, operand identity and refusal types without body JSON. Repeated and cloned calls return equal values and preserve package equality and admission; CG needs no private V2 decoder or new per-use wire member. | Test (TC-445) |
| FR-038-AC-210 | Implemented (IR-484). For a valid self-typed declared `unit` node, references to distinct values typed by that node resolve to `quantity`: all twelve catalogued `quire.op.quantity.*` entries resolve each required quantity position and reject a Boolean at the first one, independent of their later result, member and dimension obligations; `quantity.add` and `quantity.eq` with valid same-unit operands also admit end to end. Replacing a required quantity argument by Boolean refuses `ill_typed`/`operator-ineligible` at that argument; placing a declared-unit quantity in the first `integer.add` position also refuses at `arguments/0`, so a missing quantity family cannot turn the rejection into a skip. `quire.op.quantity.pow` refuses a quantity rather than an integer exponent at `arguments/1`. The operation identity and package identity are re-derived after every mutation, and the positive controls admit through the public reader. | Test (TC-048) |
| FR-038-AC-211 | Implemented (IR-484). `quire.op.control.if` over a Boolean condition and two values of the same declared-unit quantity type admits, while replacing the second branch with an Integer value refuses `ill_typed`/`operator-ineligible` at `arguments/2` under its `same_family` constraint. A valid `aggregate` in each of the three aggregate positions of `quire.op.temporal.clause` and in the first position of `quire.op.state.clause` fits `aggregate` at the operation step after earlier shape checks. Placing an aggregate in the first `boolean.and` argument refuses `ill_typed`/`operator-ineligible` at `arguments/0`, so a missing aggregate family cannot make the check pass. Replacing the first `state.clause` aggregate by a Boolean reference instead refuses `invalid_semantic_graph` at that node's body during the earlier structural stage, never as an operation-family refusal. Each control supplies the other catalogued arguments, law and member and has a freshly derived identity. | Test (TC-048) |
| FR-038-AC-212 | Implemented (IR-484). A body-root or argument literal whose `value_kind` is `integer` and whose `type` names an Integer node admits when the enclosing operation is otherwise valid; retaining that value kind while changing only its declared type to a Boolean node and re-deriving identities refuses `ill_typed`/`operator-ineligible` at the literal's `type`, even if the enclosing operand position accepts either family. An enum literal accepts either `enum` or `ordered_enum` type according to its nominal declaration, and a `none` literal accepts an option type; each mismatched kind/type pair, including an integer literal declared at a unit type, refuses at its own `type`. | Test (TC-048) |
| FR-038-AC-213 | Implemented (IR-484). `quire.op.quantity.convert` over a declared-unit quantity and a selected quantity `type_argument` admits mode `{kind: rounding, value: exact}`; `toward-zero` on otherwise identical operands and result refuses `invalid_package`/`operation-mode-type-mismatch` at `operation.mode/value`, even though the unit type has no explicit rounding binding. The result type alone pins this conversion to the declared-unit default `exact`. The same mode check applies to `quire.op.collection.sum.quantity` when its quantity binder and member are valid. | Test (TC-048) |
| FR-038-AC-222 | Implemented (IR-742). Through `CheckedPackageV2::read`, a `float32_bits` literal declared at a Float32 type and a `float64_bits` literal declared at a Float64 type each admit in otherwise valid packages. Retaining each value kind and changing only its declared `type` to the opposite IEEE width, while re-deriving the affected node and package identities, refuses `ill_typed`/`operator-ineligible` at that literal's `type`, including in an operand position whose catalogued family would otherwise accept the declared type. Neither mutation is admitted or refused merely at an enclosing operand. This extends FR-038-AC-212's general literal-family rule without changing its integer, enum or option cases. | Test (TC-048) |
| FR-038-AC-223 | Implemented (IR-742). Through `CheckedPackageV2::read`, valid `quire.op.ieee.to_float32` over Float64 and `quire.op.ieee.to_float64` over Float32 admit when each `type_argument` and `result_type` name the same correctly typed destination node. For each identity, changing `result_type` and the enclosing application's root-node `semantic_type` together to the source-width node while retaining the valid destination `type_argument`, and re-deriving all affected identities, refuses `ill_typed`/`operator-ineligible` at the application's `result_type` by the `member` result form; changing `type_argument`, `result_type` and that root-node `semantic_type` together to the source-width node likewise refuses there by the catalogued `member_family` constraint. In both cases the root `result_type` equals its node's `semantic_type`, so that separate root mismatch rule cannot decide the result. Each adverse package supplies an otherwise valid source operand and law, and is not allowed to reach a same-width descriptor through the public reader. | Test (TC-048) |
| FR-038-AC-247 | Implemented (IR-225). Through `CheckedPackageV2::read`, a catalogued `result: member`/`member: type_argument` application whose member has a valid `declaration` admits only when `result_type` equals that declaration, regardless of whether its family resolves or the entry has a `member_family` constraint. A mismatch refuses `ill_typed`/`operator-ineligible` at `/semantic_graph/nodes/{i}/body/result_type`, with the application node as locus. `collection.fold` and `collection.reduce` admit equal results without a family constraint and refuse unequal results. `model.lookup` and `model.all_instances` have other result forms and remain admitted. `quire.op.ieee.to_rational` over each of Float32 and Float64 admits with rational declaration and rational result; float/rational and rational/float declaration/result pairs refuse at that pointer, with root `semantic_type` matched to `result_type` and all affected identities re-derived. AC-250 also requires equality refusals for two distinct non-IEEE catalogue entries, `quire.op.numeric.narrow` and `quire.op.quantity.convert`. After earlier member-shape and operand checks, this result refusal precedes `result_inner`, state-clause result, model-member, mode-type and leaf checks; a simultaneous operand defect wins over a result defect, while a simultaneous genuine rounding-mode type defect on `quantity.convert` does not displace the `result_type` refusal. | Test (TC-048) |
| FR-038-AC-248 | Implemented (IR-225). For a catalogued `type_argument` member result with `member_family`, a declared type whose family the reader resolves under the FR-322 ruling must fit the catalogue family; a resolved mismatch refuses `ill_typed`/`operator-ineligible` at `/semantic_graph/nodes/{i}/body/result_type`, with the application node as locus. `quire.op.ieee.to_rational` over both Float32 and Float64 refuses a same-width float declaration and float result, even though equality holds. A valid `quire.op.quantity.convert` into a `compound_unit` with equal member declaration and result admits: its unresolved family does not cause a refusal under the FR-038 deferral of `compound_unit`, `model_population` and `alias` family rulings to FR-322. An otherwise valid `quire.op.collection.size` whose `type_argument` declaration is a `model_population` node and whose `result_type` equals that declaration admits: this result check skips the unresolved family even though the generic bounded-domain walk reaches the `Set<Reference<T>>` base and reports `set`, which does not fit the entry's `integer` member-family constraint. | Test (TC-048) |
| FR-038-AC-249 | Implemented (IR-225). A `type_argument` member missing `declaration`, carrying a malformed declaration, or carrying `type_argument` instead of `declaration` refuses `invalid_package`/`operation-member-mismatch` at `/semantic_graph/nodes/{i}/body/operation/member`, with the application node as locus. The code PR changes the eleven existing IR fixtures using `member.type_argument` to the closed `member.declaration` spelling; no alternate read is admitted. | Test (TC-048) |
| FR-038-AC-250 | Implemented (IR-225). Through the public reader, valid `quire.op.numeric.narrow` and `quire.op.quantity.convert` applications each admit with equal member declaration and `result_type`, and each refuses a mismatched `result_type` as `ill_typed`/`operator-ineligible` at `/semantic_graph/nodes/{i}/body/result_type` with the application node as locus, after re-deriving affected identities. These two non-IEEE catalogue entries demonstrate entry-driven selection beyond the IEEE conversion identities. The `field`, `position`, `element`, `relationship_end` and `operation` member-result kinds do not acquire the type-argument equality rule; in particular, valid `tuple.project` and `collection.index` packages with member declarations distinct from their `result_type` remain admitted. | Test (TC-048) |
| FR-038-AC-251 | Implemented (IR-225; amended from an expected-set shape guard because the reader selects from the current catalogue entry). Member-result checking is selected by the catalogue entry shape alone (`result: member` and `member: type_argument`). Every other catalogued operation, including `model.lookup`, `model.all_instances` and the other member kinds, keeps its pre-IR-225 admission outcome; `collection.fold` and `collection.reduce`, which have no `member_family`, are checked for result-type/declaration equality only. A catalogue-wide differential test reads every entry from the same published catalogue as the reader, requires admission for each constructible control, and names the exact unavailable fixture set. | Test (TC-048) |
| FR-038-AC-214 | CODE-GATED (IR-575; isolated 2026-10-10 measurement passed, final repo gates pending). The production `CheckedPackageV2::read` path directly decodes the closed wire into the typed package without ever constructing a full-document `serde_json::Value` tree or allocating a second full-document canonical byte buffer; `Value` terms inside the typed wire remain permitted. Source inspection and the real-reader adverse corpus confirm unchanged duplicate-member, noncanonical-byte, unknown-member and first-refusal code/pointer outcomes, unchanged identity and source-map content, and the same public typed result. In three fresh Linux x86_64 reader processes built with the repository's debug `cargo test --locked --test` profile and Rust's default System allocator, using this repository's canonical 100000-reference-chain fixture prebuilt by a separate generator process, `CheckedPackageV2::read` admits the full package under nonbinding limits and uses at most 1.50 GiB (1572864 KiB) additional peak resident memory during the call. Each reader loads input bytes and prepares evidence before recording baseline VmRSS and VmHWM. A run whose pre-read VmHWM differs from pre-read VmRSS is invalid setup and must be recorded and retried in a fresh process, never counted as passing. For each valid run, sample VmRSS during `read`, record VmHWM immediately after return while retaining the admitted package, and compute additional peak as `max(sampled peak VmRSS, post-read VmHWM) - pre-read VmRSS`. Fixture generation, lowering and runtime startup lie outside the interval. Every valid run records input size, admitted node count, baseline/peak/post RSS and HWM, elapsed read time, compiler version, allocator and platform; an over-ceiling run fails. The existing AC-117 admission and stack obligations remain. | Test, Inspection (TC-048) |
| FR-038-AC-215 | A selected model document that reads as strict JSON, supplied under its row's `digest`, then edited in one member value so that its RFC 8785 SHA-256 changes, refuses `stale_dependency`/`content-mismatch` at `/lock/model_selections/0/digest` with `expected` equal to `{domain: "sha256-jcs", algorithm: "sha256", digest: <row digest>}` and `actual` equal to the edited document's RFC 8785 identity under that same domain and algorithm (the expected digest bytes are written out in the test, not recomputed by the reader's digest function), before identity or declaration checks and never as `byte-digest-mismatch`; the unedited document and a whitespace-respelled copy under their RFC 8785 digest admit; a parseable document selected under the plain SHA-256 of its raw bytes when that differs from its RFC 8785 SHA-256 refuses `content-mismatch`, not `wrong-model-selection`; a document holding a number FR-038-AC-93 or FR-038-AC-109 refuses `noncanonical_wire` first. | Test (TC-048) |
| FR-038-AC-216 | Bytes of a selected model document that `quire_canonical::read` refuses (a truncated array, invalid UTF-8, `{"a":1,"a":2}`, a lone surrogate escape), supplied under a row `digest` other than their plain SHA-256, refuse `stale_dependency`/`byte-digest-mismatch` at `/lock/model_selections/0/digest` with `expected` `{domain: "sha256-jcs", algorithm: "sha256", digest: <row digest>}` and `actual` `{domain: "raw-artifact-digest", algorithm: "sha256", digest: <plain SHA-256 of supplied bytes>}`; the same bytes supplied under the row whose digest bytes equal their plain SHA-256 refuse `invalid_model_binding`/`wrong-model-selection` at `/lock/model_selections/0/identity` with neither payload, admit no declaration and read no identity from the bytes. Equal digest bytes in those different domains remain unequal typed identities. The outcomes of FR-038-AC-93, AC-109 and AC-110 for a number past the double range and of FR-038-AC-26 for the byte limit are unchanged by the raw digest. | Test (TC-048) |
| FR-038-AC-217 | A `dependency_selections` entry whose supplied admitted package recomputes to another `package_id` than the entry's refuses `stale_dependency`/`content-mismatch` at `/lock/dependency_selections/0/package_id/digest` with `expected` equal to the entry's complete `CheckedSemanticId` (`quire.package.semantic/v2`, `sha256`, its digest) and `actual` equal to the supplied package's complete recomputed `package_id`, and never `byte-digest-mismatch`; no package supplied refuses `missing_import`/`missing-selection` at `/lock/dependency_selections/0` with neither payload, and a supplied package with the selected `package_id` admits. | Test (TC-048) |
| FR-038-AC-218 | The `expected` and `actual` members of `CheckedPackageRefusal` are either both absent or both complete `CheckedSemanticId` values with exact domain, `sha256` algorithm and lowercase 64-character hexadecimal digest, and are present exactly for `stale_dependency`/`content-mismatch` and `stale_dependency`/`byte-digest-mismatch`. A `noncanonical_wire`, `missing_import`/`missing-selection`, `digest_domain_mismatch`, `invalid_model_binding`/`wrong-model-selection`, `invalid_semantic_graph` and `malformed_wire` refusal each carries neither. `ContentMismatch` is paired only with `StaleDependency`; two mismatch refusals with the same code and cause tag but different expected/actual identities have different QSpec FR-272 cause keys, while changing only path leaves that cause key unchanged. Typed identities with equal algorithm and digest but different `sha256-jcs` and `raw-artifact-digest` domains compare unequal. | Test (TC-048) |
| FR-038-AC-224 | PLANNED/UNRUN (IR-490). A private operation-stage meter probe over an alias or non-population bounded-domain chain of N distinct forwarding nodes charges exactly N structural-resolution units on its first successful lookup, zero on another lookup of the same head or an already resolved suffix within that application's operation validation, and N again for the same chain in a second application. The terminal type adds zero forwarding units; the first lookup's exact limit succeeds and a limit one below returns typed Work before the failed forwarding node's `semantic_type` is read. For a 300-field record naming different entries of one 300-alias chain, measure total operation work W under a nonbinding limit: W reaches the ordinary `operation-law-missing` decision and W-1 returns typed Work when the required leaves are missing; this case measures the count pass only. In a separate version with every derived leaf and its selected law supplied, both leaf passes execute and the application admits at its measured total work limit W; W-1 returns typed Work. Across those two passes each of the 300 structural forwarding nodes charges once, while leaf-visit and `text_profile` pin charges remain independent and repeated pin lookups retain their separate first-visit charge rule. For each of `union_arms`, `inner:0`, `result_inner`, and a leaf edge, an otherwise valid operation whose remaining work limit is exhausted by structural forwarding returns the reader's `incomplete`/`work` at that operation, with no `ill_typed`/`operator-ineligible`, skipped constraint, or unchecked leaf. | Test (TC-048) |
| FR-038-AC-225 | IMPLEMENTED (IR-739; tagged reader and conformance tests passed). `CheckedPackageV2::read` admits QSpec's current `positive-union-nodes.json`, read directly from an authoritative checkout by `make conformance-qspec`, with both its payload-bearing `Shape::Rect` and nullary `Shape::Empty` aggregate-bodied `value`/`union_value` nodes keyed under `quire.application-node/v1`. The test compares each retained key with an independently fixed QSpec preimage/key vector, its exact aggregate body and the fixture's recorded `package_id`; an ordinary application still admits, while application-free `record_value` and `tuple_value` controls retain structural-key selection. A private key-helper success alone is insufficient. | Test (TC-048) |
| FR-038-AC-226 | IMPLEMENTED (IR-739; tagged reader and conformance tests passed). For each of the two authentic QSpec union values of AC-225, changing its retained key to the prior structural key, rewriting every reference and source-map occurrence of that key, mirroring the identity projection and recomputing only the package identity needed to reach node-key validation, yields `invalid_package`/`stale-node-key` at that node's `node_id`; `locus` is the supplied typed key and `expected_node_id()` is the distinct current application-derived typed key. The untampered package admits, so an earlier graph or package refusal cannot pass this control. Changing one key-covered body member without rekeying also refuses at that node. | Test (TC-048) |
| FR-038-AC-227 | IMPLEMENTED (IR-739; tagged reader and conformance tests passed). In a reader-admissible recursion group containing a `union_value` and a nonapplication node, the union value's preimage uses the complete graph-order `{size, ordinal}` and rewrites each reference to an in-group member in its nested payload aggregate to `group_reference`, while an outside reference remains unchanged; the current keyed package admits and a retained stale key refuses with the distinct production-derived expected key. At the private key-stage seam, for the same preimage, a canonical byte ceiling equal to its independently measured byte length accepts its key and one less refuses the existing typed `invalid_semantic_graph` at `node_id` with no expected key; exhausted work refuses `incomplete`/`work` before hashing. The public reader's earlier whole-package byte limit remains in force. Existing strict-wire, package-identity, owner-join and graph-shape refusals keep their precedence ahead of the key stage; the derived-structural, declaration, frame, operation and generic reference-edge stages follow it. No copied encoder, alternate ceiling or public candidate-helper admission claim is introduced. | Test (TC-048) |
| FR-038-AC-228 | PLANNED/UNRUN (IR-491). Through `CheckedPackageV2::read`, otherwise valid `structural.eq` and `structural.ne` packages whose first operand is, separately, a reference to a structural value, a reference directly to its type node, a reference to an expression node with an application at its body root whose `result_type` equals that node's `semantic_type`, or an option-typed `none` literal, each derive the leaves of the selected declared type. An all-integer record admits with `leaves: []`, and a record with one pinned text field admits with exactly that field's selected-law and mode leaf, while removing the required leaf refuses `invalid_package`/`operation-law-missing` at `operation.leaves`. Replacing the first operand's reference with an inline `application` refuses `malformed_wire` at `arguments/0` before leaf checking. Each package's operand and result identities are independently re-derived; changing a declared type does not silently retain the original comparison. | Test (TC-048) |
| FR-038-AC-229 | PLANNED/UNRUN (IR-491). For `structural.eq`, `structural.ne` and `collection.contains`, a first operand replaced with a `binding`, `aggregate`, `frame` or `abstraction_relation` never admits by skipping type resolution: after any earlier applicable grammar/reference check, it refuses `ill_typed`/`operator-ineligible` at `arguments/0` if it reaches the operand-family check. A `dependency_reference` in that position is refused by the earlier dependency-reference placement check. Separately, a first operand whose family fits but whose required compared or `inner:0` type remains unresolved after earlier checks refuses `ill_typed`/`operator-ineligible` at `operation.leaves`, with `leaves` empty or nonempty; exhausted structural-resolution work remains `incomplete`/`work`. | Test (TC-048) |
| FR-038-AC-230 | PLANNED/UNRUN (IR-491). For every catalogued `result_inner` identity (`collection.set`, `.bag`, `.ordered_set`, `.map`, `.flat_map`, `.flatten`, `.convert`), a result of `set`, `bag` or `ordered_set` over pinned text admits exactly the selected element-type leaf at the empty path and refuses an omitted one as `operation-law-missing` at `operation.leaves`; where the catalogued result form permits a valid `sequence` result over that text, it admits with `leaves: []` and refuses one supplied leaf as `operation-law-mismatch` at `operation.leaves/0`. A set-like result whose required element type remains unresolved after earlier checks refuses `ill_typed`/`operator-ineligible` at `operation.leaves` rather than admitting either leaf list; exhausted structural-resolution work remains typed Work. Each positive control admits through the public reader with current catalog and lock selections. | Test (TC-048) |
| FR-038-AC-231 | PLANNED/UNRUN (IR-491). An otherwise valid `structural.eq` or `structural.ne` over a record, tuple, option or sequence whose selected compared type reaches Float32 or Float64 at any depth, and `collection.contains` over a sequence whose element type does, each refuse `ill_typed`/`operator-ineligible` at `operation.leaves` with `leaves: []` and with a supplied text leaf; changing only that nested IEEE type to an integer type and re-deriving identities admits with its exact empty leaf list. A valid `collection.flatten` result of `sequence<Float32>` or `sequence<Float64>` admits with no leaves because `result_inner` selects no compared type for a sequence result. A `set`, `bag` or `ordered_set` of an IEEE-bearing element is not a positive control: its element type is ineligible under QSpec FR-144. Each adverse comparison is first independently shown to pass strict wire, graph and identity admission without the comparison, so an earlier malformed package cannot satisfy the refusal. | Test (TC-048) |
| FR-038-AC-232 | PLANNED/UNRUN (IR-691). A fresh QSL FR-093 checked and emitted `record Point { x: Int[0,9]; }` value `Point { x: 3 }` is admitted by the public V2 reader; its `record_value` binding references a `numeric.narrow` conversion whose sole argument references the canonical Integer literal `3`, and whose member declaration, result type and semantic type name the same `Int[0,9]` range. In an admitted structural.eq or structural.ne over that value, `composite_application_operands` returns the original record GraphChild and semantic type with Literal domain and no introduced positions. At the private post-admission seam, an equivalent graph with canonical integer literal and range endpoints beyond i128 has the same result; no copied or narrowed value is returned. | Test (TC-048) |
| FR-038-AC-233 | PLANNED/UNRUN (IR-691). In crate-internal admitted-then-mutated controls, altering only the narrow node's operator, laws, leaves, mode, member kind/declaration, result type, semantic type or argument count/term; the range's presence, Integer base, endpoint binding/literal shape, endpoint literal type, endpoint grammar/order; or the operand literal's value kind, declared/semantic type or canonical spelling returns `MalformedDomain` with the enclosing ordinal and first valid type locus specified above. For both positive and negative ranges beyond i128, a canonical literal one below min or one above max returns `MalformedDomain` at the actual range node, with no rounded, clamped or partial Literal result; min and max themselves succeed. The negative controls use explicitly authored signed decimal strings on both sides of each bound, so a positive-only or lexicographic comparator fails. | Test (TC-048) |
| FR-038-AC-234 | PLANNED/UNRUN (IR-691). A narrow whose argument reaches a parameter, another application including a second narrow, or a value cycle returns `UnsupportedOperand`/`NonliteralGraphValue` at the first offending node with the enclosing ordinal; an unrelated application remains refused. A direct parameter operand retains its Parameter projection. Existing closed tuple, Option, collection and enum-value descendants remain accepted by their value-form paths, and a narrow integer field inside a supported composite receives only the stated narrow exception. Reader admission and first-refusal outcomes for the unchanged packages are identical before and after the accessor change. | Test (TC-048) |
| FR-038-AC-235 | PLANNED/UNRUN (IR-691). Independently count the exact logical graph, term/member, examined-endpoint-byte and examined-literal-byte charges for a small admitted narrow-record fixture. Its measured successful work limit succeeds; one less and zero return `WorkLimit` with the first attempted cumulative charge and no partial operands. At the private seam, a range with long canonical endpoints charges every endpoint byte once per logical visit before comparison even when its first sign or digit decides the order: the independently counted exact limit succeeds, and one less returns `WorkLimit`. A budget exhausted immediately before a malformed wrapper, out-of-range literal or later operand defect returns `WorkLimit` first. Reordering admitted graph nodes leaves the result and consumed work equal. No public accessor type, scalar eligibility, reader admission rule, second domain owner or fixed-width decimal conversion is introduced. | Test (TC-048), Inspection |
| FR-038-AC-236 | IMPLEMENTED (IR-777). A selected Semantic IR model declaration of a nonempty variant type, with a self-typed declaration node and exact model owner, is admitted through `CheckedPackageV2::read`. Its declared type is accepted as a field, parameter and result type, including under an optional or bounded sequence field. Changing the owner to an absent or different declaration refuses at the declaration's `node_id` with `missing_declaration`/`missing-selection`. | Test (TC-048) |
| FR-038-AC-237 | IMPLEMENTED (IR-777). A value typed by the selected variant accepts an `enum_value` whose `case` is an exact selected UTF-8 case name, including a name with a space, digit prefix or non-ASCII character. Distinct case names or variant declarations yield distinct re-derived value keys, and `enum.eq` and `enum.ne` admit otherwise valid operands of the same variant type. `enum.lt` and `structural.eq` refuse that unordered variant at the first ineligible argument. | Test (TC-048) |
| FR-038-AC-238 | IMPLEMENTED (IR-777). An empty selected case declaration refuses `invalid_model_binding`/`malformed-declaration` at the selected model row, while an `enum_value` naming an absent case refuses `invalid_semantic_graph` at its `case`. Supplying a document with a different case set under the selection digest refuses `stale_dependency` at that digest. Ordinary scalar enum values retain their identifier-only case rule. | Test (TC-048) |
| FR-038-AC-239 | PLANNED/UNRUN (IR-782). A package with a `protocol`/`protocol_reference` carrier whose closed reference body targets an exact local `protocol` node, with one `expression` occurrence, the same semantic type and exactly that target as its dependency, admits through the public V2 reader as `ProtocolForm::ProtocolReference`. Both a `protocol_clause` target and a different protocol form (`role` or `control`) admit; a reader that accepts only `protocol_clause` fails the second control. For each admitted target, the structural key, identity projection and package id are independently recomputed; changing the target and re-deriving these identities changes the carrier key. The admitted package retains one graph node collection and its source correspondence. | Test (TC-048) |
| FR-038-AC-240 | PLANNED/UNRUN (IR-782). An extra dependency or a dependency unequal to the carrier's body target refuses `invalid_semantic_graph` at `/semantic_graph/nodes/{i}/dependencies` in graph-shape validation; a stale carrier key with matching dependency refuses `invalid_package`/`stale-node-key` before the local target join. With both body target and sole dependency changed together and identities independently recomputed, an absent local target refuses `missing_declaration`/`missing-name` at `/semantic_graph/nodes/{i}/body/target`, even if an admitted selected dependency contains that key. A present non-`protocol` target, or a protocol target with unequal semantic type, refuses `ill_typed`/`operator-ineligible` at that same pointer. A carrier target defect wins over a frame or operation defect, after the earlier graph-shape and stale-key checks. | Test (TC-048) |
| FR-038-AC-241 | PLANNED/UNRUN (IR-782). A carrier with a non-reference or `dependency_reference` body or an extra body member fails the closed V2 schema. A carrier with a declaration, owner, nominal preimage or recursion group refuses at its offending member before target resolution. Multiple valid `expression` occurrences are admitted as specified by FR-038-AC-294; they are not a forbidden carrier member. An ordinary node holding a generic reference remains an ordinary node; no second protocol-reference form or separate collection is admitted. | Test (TC-048) |
| FR-038-AC-294 | Implemented; tagged native public-reader controls passed (IR-785). Starting with an otherwise admitted public-reader package containing one local protocol target and one carrier with `expression` ordinal 0, add a second source reference to that target as `expression` ordinal 1 and its own exact source-map row. Both packages admit through `CheckedPackageV2::read`, retain exactly one carrier, and have equal independently recomputed carrier `node_id` and `package_id`; the admitted carrier retains both distinct ordinals and both source correspondences. A third occurrence with a further distinct ordinal and source-map row also admits with those same identities, so a fixed two-occurrence maximum fails this criterion. The source regions of the references are distinct. The body stays exactly `{term: "reference", target}` and no identity discriminator is added. | Test |
| FR-038-AC-295 | Implemented; tagged native public-reader controls passed (IR-785). From the admitting repeated-reference package, independently duplicate an occurrence ordinal or a source-map key while retaining all otherwise valid identities. The public reader refuses `invalid_source_map` respectively at the later `/semantic_graph/nodes/{n}/occurrences/{j}` or `/source_map/{i}` pointer. An empty carrier occurrence array refuses `invalid_source_map` at `/semantic_graph/nodes/{n}/occurrences`; changing either occurrence role away from `expression` refuses `invalid_semantic_graph` at that occurrence's `/role` pointer before target resolution. Adding a second actual graph node with the carrier's `node_id` refuses `invalid_semantic_graph` at the later node's `/node_id`. Each case is a separate wire mutation, so an earlier duplicate or stale-key defect cannot mask the asserted refusal. Existing closed-body, forbidden-member, exact target, type and dependency controls remain unchanged. | Test |
| FR-038-AC-242 | IMPLEMENTED (IR-800; tagged reader tests passed). A selected Semantic IR document with fields, parameters and results declared at `ix://quire/native/String`, `ix://quire/native/UUID` and `ix://quire/native/Timestamp` admits through `CheckedPackageV2::read` with distinct self-typed `scalar_type`/`string`, `/uuid` and `/timestamp` nodes and matching member types; the same String declaration never resolves as `text` or requires a text profile. Each kind's correctly typed literal admits with the exact canonical payload retained in its keyed body. | Test (TC-048) |
| FR-038-AC-243 | IMPLEMENTED (IR-800; tagged reader tests passed). The production catalog and closed V2 vocabularies admit exactly `quire.op.string.eq`, `.ne`, `quire.op.uuid.eq`, `.ne`, and `quire.op.timestamp.eq`, `.ne` as native comparisons; each public-reader package with same-family operands, Boolean result and no law, mode, member or leaves admits, and equal/unequal canonical payload controls retain their separate operand identities. For each of the six identities, replacing argument 0 and then argument 1, separately, with a foreign-family operand (`text` for `string`, `string` for `uuid`, `uuid` for `timestamp`) refuses `ill_typed`/`operator-ineligible` at the changed argument. `quire.op.integer.lt` and `quire.op.integer.add` over each of `string`, `uuid` and `timestamp` operands likewise refuse at `arguments/0`; neither a one-sided family check nor an ordering/arithmetic special case for one native family passes. | Test (TC-048) |
| FR-038-AC-244 | IMPLEMENTED (IR-800; tagged reader tests passed). Adding a selected `text_profile` law to each native comparison refuses `invalid_package`/`operation-law-mismatch` at that law; adding a `text_profile` mode refuses `invalid_package`/`operation-mode-mismatch` at the mode; placing a `string` operand in `quire.op.text.eq` refuses `invalid_package`/`operation-mode-type-mismatch` at the mode value. An arity defect or another wrong-family operand in that text comparison refuses `ill_typed`/`operator-ineligible` at the arguments or first offending argument before the String-in-text-operation mode refusal. None admits by treating `string` as `text` or by ignoring a forbidden law or mode. | Test (TC-048) |
| FR-038-AC-245 | IMPLEMENTED (IR-800; tagged reader tests passed). For each native `value_kind`, a literal typed at another family refuses `ill_typed`/`operator-ineligible` at its `type` even if its `value` is also malformed. With the right family, a non-string String value, an uppercase, hyphenless or malformed UUID, and a Timestamp value of `-0`, `+1`, `01`, empty text, outside signed-128 range or another noncanonical spelling refuse `invalid_package`/`invalid-value` at that literal's `value`; exact String Unicode sequences, UUID nil/max and both Timestamp endpoints admit. Recompute every affected node key and package identity so no stale-key refusal can stand in for these decisions. | Test (TC-048) |
| FR-038-AC-252 | Implemented (IR-824). This criterion supersedes the Set/Bag clause of FR-038-AC-172 (an "otherwise ineligible Set/Bag edge" refused `ill_typed`/`operator-ineligible`); no other outcome of AC-172 changes. Merged QSpec FR-322-AC-57 and FR-152 "Reachability over a relationship end" (RN-18, RN-20 to RN-27) admit `quire.op.model.reaches` over a finite multi-valued destination end. For a homogeneous self-relationship with a resolved forward or inverse end whose destination multiplicity is finite and not `[0,1]` or `[1,1]` (`{0,3}` unique, `{0,3}` not unique, `{2,3}`), the application admits with two `Reference` operands conforming to the receiver endpoint's own type (including a subtype operand 1) and the catalog's Boolean `result_type`; operand types, node members and result are those of the single-valued case, and the node carries no charge or multiplicity member. The optional `population` member key is specified separately by FR-038-AC-269 through FR-038-AC-280; it is not a charge or multiplicity member and does not change this criterion. | Test (TC-048) |
| FR-038-AC-253 | Implemented (IR-824). For `quire.op.model.reaches` over a finite multi-valued destination end, an operand 1 unrelated to the receiver endpoint type refuses `ill_typed`/`operator-ineligible` at `arguments/1`, and an operand 1 that is a subtype of it admits. The receiver-endpoint checks of operand 0 (`arguments/0`) and operand 1 (`arguments/1`) run before direction, multiplicity and type (QSpec FR-322-AC-57), so an unrelated operand 1 together with a wrong-direction, `unbounded`, `ordered` or other-type end refuses at `arguments/1`; single-defect outcomes are unchanged. | Test (TC-048) |
| FR-038-AC-254 | Implemented (IR-824). For `quire.op.model.reaches` with operands conforming to the receiver endpoint type, a named end traversed against the declared direction refuses `ill_typed`/`operator-ineligible` at `member.name`, decided before multiplicity: the same end also made `unbounded` refuses with the direction outcome (QSpec RN-27). | Test (TC-048) |
| FR-038-AC-255 | Implemented (IR-824). For `quire.op.model.reaches` over an eligible direction, an `unbounded` upper bound or an `ordered` destination end refuses `unsupported_construct`/`expression-form` at `member.name`, decided before type: an `unbounded` end of another type refuses with this outcome (QSpec RN-23, RN-24, RN-26). | Test (TC-048) |
| FR-038-AC-256 | Implemented (IR-824). For `quire.op.model.reaches` over an eligible direction and a finite end, a destination end whose type differs from the receiver endpoint's, single-valued or multi-valued, refuses `ill_typed`/`operator-ineligible` at `member.name` (QSpec RN-25). | Test (TC-048) |
| FR-038-AC-257 | Implemented (IR-824). A `quire.op.model.reaches` application over an admitted finite multi-valued end whose `result_type` names a navigation result type instead of the catalog's Boolean node refuses `ill_typed`/`operator-ineligible` at `body.result_type`. | Test (TC-048) |
| FR-038-AC-258 | Implemented (IR-832). For an admitted `value`/`parameter` node, `parameter_declaration` returns `Ok` whose `node` equals the supplied id, whose `name` equals the node's admitted `"name"` text literal and whose `level` equals the canonical decimal text of its admitted `"level"` integer literal; for parameters named `a` at level `0` and `b` at level `1` it returns exactly `("a", "0")` and `("b", "1")`, and a level of `10` is returned as `"10"` and not read as an integer. | Test (TC-048) |
| FR-038-AC-259 | Implemented (IR-832). `parameter_declaration` over an id absent from the graph refuses `UnknownNode`; over an existing `value`/`literal`, an `application` node and a `function` node it refuses `NotParameter` for each; over a parameter node whose `body` a crate-internal post-admission mutation changed to lack the `"name"` binding, to give `"name"` a non-text value, to give `"name"` a non-identifier value, to type the name literal at a node that is not `scalar_type`/`text`, and to give `"level"` the non-canonical value `"010"`, it refuses `MalformedBody` for each. Every refusal returns only the typed error and no partial declaration. | Test (TC-048) |
| FR-038-AC-260 | Implemented (IR-832). Two admitted parameter nodes named `x` at levels `0` and `1` have distinct ids, and `parameter_declaration` returns `level` `"0"` for one and `"1"` for the other with `name` `x` for both, so the accessor separates equal names by level and not by an enclosing function. The result type has no member naming a function, owner or declaration, which a compiled field-by-field construction of `CheckedParameterDeclaration` shows. | Test (TC-048) |
| FR-038-AC-261 | Implemented (IR-832). For an admitted `quire.op.integer.add` application whose two operands reference parameters `a` and `b`, each typed at `Int[0, 9]` (a finite `bounded_domain`/`integer_range`, so that `scalar_application_operands` returns `Ok` and not `UnboundedRange`), calling `parameter_declaration` on each `GraphChild` id returned by `scalar_application_operands` yields `a` then `b` in operand ordinal order, and the same application with its arguments swapped in a separately admitted package yields `b` then `a`. An `InlineLiteral` operand carries no operand `CheckedNodeId`: the only id its `CheckedScalarOperandChild::InlineLiteral` holds is `application`, the containing application's, and `parameter_declaration` over that id refuses `NotParameter`. | Test (TC-048) |
| FR-038-AC-262 | Implemented (IR-851). When `relationship_end` is called with the id of an admitted `quire.op.model.reaches` application over a bidirectional homogeneous relationship (the direction under which both a forward and an inverse application admit) whose source end is `{1,1}` and whose target end is `{2,3}` unique with role `peer`, it returns `Ok` with `application` equal to the supplied id, `relationship` equal to the relationship node the application names, `forward` true when the application names the source role, and `lower` 2, `upper` `Some(3)`, `unique` true, `ordered` false and `role` `Some("peer")`; for a second admitted application over the same relationship naming the target role it returns `forward` false and the source end's `lower` 1, `upper` `Some(1)`, `unique` false and its own `role`, so a swap of the two ends fails. `destination_type` equals the model declaration node id of the destination end's object type, derived independently of the accessor. | Test (TC-048) |
| FR-038-AC-263 | Implemented (IR-851). For admitted reaches applications over a destination end of `{0,3}` unique, `{0,3}` not unique, `{2,3}`, `{0,1}` and `{1,1}`, `relationship_end` returns exactly that `lower`, `upper` and `unique` with `ordered` false and `upper` `Some`, and for a destination end that omits its role it returns `role` `None`; an `unbounded` or `ordered` destination end is refused at admission by FR-038-AC-255, so no application exists to be read. For an admitted application whose operand 1 is a subtype of the receiver endpoint type, `destination_type` is the declared end type's derived declaration node id and not the subtype's, including when the graph holds no node with that id, and the result is `Ok`. | Test (TC-048) |
| FR-038-AC-264 | Implemented (IR-851). `relationship_end` over an id absent from the graph refuses `UnknownNode`; over an admitted `quire.op.model.navigate` application with a `relationship_end` member, an application of `quire.op.integer.add`, the relationship declaration node, an object type's declaration node, a field-read node and a `value`/`literal` node it refuses `NotReachesApplication` for each. Every refusal returns only the typed error and no partial end. | Test (TC-048) |
| FR-038-AC-265 | Implemented (IR-851). When a crate-internal post-admission mutation of an admitted reaches application removes its `operation.member`, sets the member `kind` to a value other than `relationship_end`, sets its `declaration` to a non-node-id value, points it at a node that resolves to no retained relationship, sets its `name` to a non-text value, to a text equal to neither role or to a text equal to both roles, `relationship_end` refuses `UnresolvedEnd` for each and returns no partial end; the defects that the reader can see in a package given to it are refused at admission as before. | Test (TC-048) |
| FR-038-AC-266 | Implemented (IR-851). When `relationship_end` is called twice on an admitted reaches application and on a clone of the admitted package, the results are equal, the package equals its pre-call self, the accessor charges no work, and every package of FR-038-AC-252 through FR-038-AC-257 yields its pre-change admission or refusal outcome at the same pointer and the same refusal order. | Test (TC-048) |
| FR-038-AC-267 | Implemented (IR-851). An external Rust API fixture calls `relationship_end` with `&CheckedNodeId`, exhaustively matches `CheckedRelationshipEndError` without a wildcard arm and constructs `CheckedRelationshipEnd` field by field with exactly the nine fields of the table, so a tenth public field or a `serde_json::Value`, `CheckedMemberType` or population member fails to compile. The FR-019 Public items row for the two new types is added by the code change that exports them, not by this specification change. | Test (TC-048) |
| FR-038-AC-268 | Implemented (IR-851). `model_object_fields` over the object type named by a relationship returns the same fields as before this change and no relationship end, and no `selected_population` item is `pub use`d at the model crate root or re-exported from the `quire-contract-ir` root (FR-053-AC-5); `selected_population` stays reachable only by its module path. | Test (TC-048) |
| FR-038-AC-269 | Implemented (IR-852). When `reaches_population` is called with the id of an admitted `quire.op.model.reaches` application whose `operation.member` carries `population` naming a `bounded_domain` / `model_population` node of the receiver end's declared type with a `max` of 3 and two `Type` occurrences at ordinals 1 and 4, it returns `Ok(Some(selector))` with `package_id` the package's identity, `binding` the node id the member holds and `occurrence` the `Type` occurrence at ordinal 1, and `select_population` over `Admitted` with that selector returns `Finite` (AC-279 makes this hold for every admitted population link) with `maximum` 3, `minimum` 0 and the object type the receiver end's declared type derives, each expected value derived from the fixture and not from the accessor. | Test (TC-048) |
| FR-038-AC-270 | Implemented (IR-852). When `reaches_population` is called with an admitted reaches application whose `operation.member` has no `population` key it returns `Ok(None)`, also in a package that holds a `model_population` node of the receiver end's declared type; two reaches applications in one package naming the same population node return equal selectors, and two naming populations of different bounds return selectors that differ in `binding`. | Test (TC-048) |
| FR-038-AC-271 | Implemented (IR-852). `reaches_population` over an id absent from the graph refuses `UnknownNode`; over an admitted `quire.op.model.navigate` application, an application of `quire.op.integer.add`, the relationship declaration node, a `model_population` node and a `value`/`literal` node it refuses `NotReachesApplication` for each; and when a crate-internal post-admission mutation of an admitted reaches application sets `population` to a non-node-id value, to a node id absent from the graph, to the id of a node that is not a `model_population` node, or to the id of a `model_population` node whose `Type` occurrences are removed, it refuses `UnresolvedPopulation` for each. Every refusal returns only the typed error and no partial selector. | Test (TC-048) |
| FR-038-AC-272 | Implemented (IR-852). A reaches application whose `operation.member.population` is not a node id (a number, an object and `null`), or is a node id with no node in the graph and listed in `dependencies`, is refused by the reader with `ill_typed`/`operator-ineligible` at the member's `population` pointer, for each, and none is refused `missing_declaration`. | Test (TC-048) |
| FR-038-AC-273 | Implemented (IR-852). A reaches application whose `operation.member.population` names a node that is not a `bounded_domain` / `model_population` node, or a `model_population` node with no `Type` occurrence, is refused by the reader with `ill_typed`/`operator-ineligible` at the member's `population` pointer, for each of the two. | Test (TC-048) |
| FR-038-AC-274 | Implemented (IR-852). A reaches application whose `operation.member.population` names a `model_population` node over an object type other than the declared type of the selected receiver end, including a subtype of that type, is refused by the reader with `ill_typed`/`operator-ineligible` at the member's `population` pointer, and one over exactly that type is admitted. | Test (TC-048) |
| FR-038-AC-275 | Implemented (IR-852). A `quire.op.model.navigate` application whose `operation.member` carries a `population` key that names a node id its `dependencies` list, whatever the node is (an absent node, an integer-range node or a valid population node), is refused by the reader with `ill_typed`/`operator-ineligible` at the member's `population` pointer, at the operation stage; the same application whose `dependencies` omit the id is refused earlier by the dependency join (`invalid_semantic_graph`, no cause). An application of an operation other than `navigate` or `reaches` keeps its present outcome for a `population` key: `type_argument`, `temporal_interval`, `fairness` and `state_clause` members refuse as today and `field`, `position` and `element` members admit it unread. | Test (TC-048) |
| FR-038-AC-276 | Implemented (IR-852). A reaches or navigate application whose `operation.member.population` is a node id is admitted by the dependency join only when its `dependencies` hold that id in the unique, digest-ascending list; a package that omits that id, and a package that lists a population id no reaches or navigate application names, are each refused at the structural stage with `invalid_semantic_graph` and no cause, and a reaches application without the key keeps the dependency list it has today. | Test (TC-048) |
| FR-038-AC-277 | Implemented (IR-852). When `reaches_population` is called twice on an admitted reaches application and on a clone of the admitted package, the results are equal, the package equals its pre-call self and no work is charged; every package of FR-038-AC-252 through FR-038-AC-257 and FR-038-AC-262 through FR-038-AC-268 that carries no `population` key yields its pre-change admission or refusal outcome at the same pointer. | Test (TC-048) |
| FR-038-AC-278 | Implemented (IR-852). An external Rust API fixture calls `reaches_population` with `&CheckedNodeId`, binds the result as `Result<Option<CheckedSelectedPopulationSelector>, CheckedReachesPopulationError>` and exhaustively matches `CheckedReachesPopulationError` without a wildcard arm, so a fourth variant fails to compile; no `selected_population` item is `pub use`d at the model crate root or re-exported from the `quire-contract-ir` root (FR-053-AC-5). The FR-019 Public items row for the new error type is added by the code change that exports it, not by this specification change. | Test (TC-048) |
| FR-038-AC-279 | Implemented (IR-852). A reaches application whose `operation.member.population` names a `model_population` node over the receiver end's declared type with a missing `max`, two `max` members, a `max` that is not an integer `literal`, a negative `max` and a `max` above `u64` spelled as the decimal string `"18446744073709551616"` (a JSON number that large is refused earlier as `noncanonical_wire`) is refused by the reader with `ill_typed`/`operator-ineligible` at the member's `population` pointer, for each of the five (the missing, non-literal and negative cases differ from the code FR-322 gives them at the node, as the reader check states); a population node whose `max` member is not a `binding` is refused earlier by the graph reader as `invalid_semantic_graph` at `/semantic_graph/nodes/N/body/members/M` and never reaches the population check; a `max` of 0 and a `max` given as a decimal string are admitted and yield `Finite` from `select_population` with that maximum. | Test (TC-048) |
| FR-038-AC-280 | Implemented (IR-852). A reaches application with an invalid `population` (a node id absent from the graph, listed in its `dependencies`) and an earlier defect in `check_relationship_member` (an operand 1 that does not conform to the receiver end) is refused for the operand at its `arguments` pointer, not at `population`; a reaches application that carries a mode or a leaf is refused by the identity stage (`operation-mode-mismatch` for the mode, a law mismatch for the leaf) whatever its `population` when the `population` is a node id listed in the application's `dependencies`, because the catalog gives `quire.op.model.reaches` neither; an invalid `population` id missing from `dependencies` is refused earlier by the dependency join (`invalid_semantic_graph`, no cause), which runs before the identity stage. | Test (TC-048) |
| FR-038-AC-281 | A reader-admitted protocol member has a `quire.op.protocol.control` body root, exactly one `protocol_profile` law matching the selected lock row, `null` mode, no leaves, Boolean result and semantic types, and a `profile_operator` member. Under the published `quire.protocol.finite-global/v1` selection the admitted operators are exactly `protocol`, `role`, `channel`, `queue`, `finish`, `terminal`, `sequence`, `choice`, `parallel`, `repeat`, `await`, `check`, `send`, `receive`, `attempt`, `effect`, `event`, `commit`, `fence` and `compensate`; an unknown operator refuses `operation-member-mismatch`. A law selected as `quire.protocol.complete/v1` does not satisfy a `finite-global/v1` row, nor the reverse. A missing or mismatched law refuses with its catalogued law cause; a member whose kind is not `profile_operator` refuses `invalid_package`/`operation-member-mismatch` at the application, before any member-body check. An `await` or `compensate` without a selected `temporal_profile` refuses `missing_declaration`/`missing-selection` because it reads a clock role. | Test (TC-048) |
| FR-038-AC-282 | Each of the seven protocol member forms admits only the FR-464, FR-465 or FR-466 fixed argument names, order and term shapes for its selected operator: thirteen clause bindings, the role, channel, queue, finish and terminal layouts, thirteen control layouts and thirteen compensation bindings. A missing, extra, reordered, wrongly shaped or misspelled binding refuses `malformed_wire`; an optional Group admits zero or one member only, an interval has non-negative integer bounds with lower no greater than upper, and a set or sorted-record list repeated or out of its specified ascending order refuses `invalid_semantic_graph`. An ordered `sequence` or capture list retains its authored order. | Test (TC-048) |
| FR-038-AC-283 | A protocol node derives its `quire.application-node/v1` key from its exact tag, form, Boolean type, declaration path, `null` recursion and body, with occurrences excluded; a second reference, source reformatting or declaring roles and other set-backed members in another source order preserves the key and package id, while a changed name, referenced target, literal, ordered capture or `sequence` child changes both. Role, channel, finish, terminal, control and compensation declarations use the fixed qualified-name segments of QSpec FR-464; queues have no declaration and generated occurrences, while other protocol members have declaration occurrences. A wrong path or occurrence role refuses at the node. | Test (TC-048) |
| FR-038-AC-284 | The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending node-id digest. A package with both a refinement-step and protocol defect reports the refinement defect; one with both a state-step and protocol defect reports the state defect; one with both a protocol and operation defect reports the protocol defect. A nested protocol application refuses `malformed_wire` at strict wire validation before this step. | Test (TC-048) |
| FR-038-AC-285 | Every role, channel, queue, obligation and compensation is listed by exactly one owning protocol clause, and every control is held once in its root control tree with the declaration path of its holder followed by its own name. A member listed twice or unattached, a control held twice or by nothing, a channel without exactly one queue, or a queue attached to another clause refuses `invalid_semantic_graph` at that member. A target not listed in its referencing node's `dependencies` refuses `missing_declaration`/`missing-name`; a target of the wrong tag or form refuses `invalid_model_binding`/`malformed-declaration`, except a relationship target of the wrong export refuses `wrong-export`. | Test (TC-048) |
| FR-038-AC-286 | A role admits the FR-465 static and replicated shapes only, including a model of an eligible form, replication maximum at least one, a reference-typed binder and an `until` term exactly for an `until` lifetime; a channel admits only same-clause role endpoints, an eligible message type, explicit delivery and ordering, and a FIFO key with equality-stable type; a queue admits one channel, literal or symbolic capacity, explicit overflow and a loss selector exactly for `loss`. The reader admits the positive variants and refuses the wrong form, missing premise, wrong binder type, non-Boolean lifetime, noncanonical capacity or key and ineligible equality with the FR-465 code, cause and locus. | Test (TC-048) |
| FR-038-AC-287 | A clause names exactly one `finish` obligation whose closing-record binder and Boolean constraint read only the binders available after the root control; its optional `terminal` obligation admits `when` with one Boolean condition or `any` with none. A missing or extra condition refuses `malformed_wire`, a non-Boolean condition or constraint refuses `ill_typed`/`non-boolean-root`, an unjoined branch binder read by finish refuses `missing_declaration`/`missing-name`, and a terminal target in the finish position or an unattached finish refuses by the FR-464 membership rules. | Test (TC-048) |
| FR-038-AC-288 | Each of the thirteen control operators admits its FR-466 ordered arguments and local reference, type, bounds, label and ordering checks: `sequence`, `choice`, `parallel`, `repeat`, `await`, `check`, `send`, `receive`, `attempt`, `effect`, `event`, `commit` and `fence`. A choice or parallel branch list with fewer than two entries, repeated labels, a receive naming a non-send, an effect naming a non-attempt, an await naming an ineligible event, a non-Boolean guard or constraint, an invalid join count or an ineligible memory or ordering spelling refuses with the FR-466 code, cause and locus, without changing the accepted meaning of another operator. | Test (TC-048) |
| FR-038-AC-289 | A `compensate` body admits the thirteen FR-466 bindings in order, including distinct registration and activation captures, a trigger, interval, at-least-one attempt, retry relation, optional commit and recovery predicate. `commit never` is an absent optional Group, not a fabricated commit. The reader refuses a forward target that is not an `effect`, a commit target that is not a `commit`, an inverted interval or zero attempts, a commit that dominates its forward effect, and a read outside any of the five compensation stages with the corresponding FR-466 code, cause and locus; a commit in one case of a `choice` or in a non-all parallel branch before the effect is admitted when it does not dominate that effect, and the package creates no runtime compensation instance. | Test (TC-048) |
| FR-038-AC-290 | For each clause the reader constructs the QSpec FR-464 derived causal relation from control containment, sequencing, receive/send, effect/attempt and await edges, not source order or timestamps. A receive before its send or an await anchored at a later control refuses `invalid_package`/`definition-cycle` naming the cycle; a same pair on independent parallel branches admits. A repeat whose body can complete with no progress on a path refuses `unsupported_construct`/`declaration-form`, while a repeat with a real step admits with or without a maximum and loop proof. | Test (TC-048) |
| FR-038-AC-291 | Binder levels start at the introducing construct's entry count; protocol `over` is level 0, an `each` trigger follows, captures advance in their authored order, and a control binder is at the size of its readable input set. The reader admits reads within the FR-464/466 stage tables and refuses a wrong level, duplicate readable name, forward or foreign binder, or read after a branch, choice, repeat or await path where the binder is unavailable with the specified code, cause, stage and owner. In a binders-only slot, including each `choice` or `repeat` `visible` entry and guard, a direct read of `over` or model state refuses `ill_typed`/`operator-ineligible` at the read before binder-scope or decision-fact checks. | Test (TC-048) |
| FR-038-AC-292 | At a `choice` or `repeat` decision the reader admits only QSpec FR-466 visible facts owned by the deciding role and listed individually in `visible`; a field fact from another role, a readable but ineligible `send`, `effect` or `commit` binder or protocol activation binder, a fact reachable only inside a composite entry, or a readable fact omitted from `visible` refuses `wrong_snapshot`/`wrong-observation` with the specified `other-role`, `ineligible-kind` or `unlisted` reason and a retained binder and field. In each of the four `choice`/`repeat` `visible`/guard slots a direct read of protocol input `over` refuses `ill_typed`/`operator-ineligible` at the read, naming the slot, never `wrong_snapshot`. A bare read of any record binder, including the protocol activation binder, with no field projection refuses `ill_typed`/`operator-ineligible` at that bare read with the slot detail after scope and model-state checks and before decision-fact checks; it never fabricates a field or `wrong_snapshot` detail. A read of the finish binder or a compensation-template binder in any of those slots refuses `missing_declaration`/`missing-name` with `read` `stage` or `foreign` respectively, never `wrong_snapshot`. A slot holding both an `over` read and a binder read reports the `over` refusal first; a slot holding an out-of-scope binder read and an in-scope ineligible binder read reports `missing-name` first; a slot holding a bare record binder read and an in-scope ineligible field read reports the bare-read refusal first in either operand order. | Test (TC-048; QSpec FR-466-AC-14/16, TC-948 DF7g/DF7h) |
| FR-038-AC-324 | For an admitted protocol clause the reader derives one activation-relation row per distinct required temporal clause from the clause's `activation` and `requirements` alone, in ascending required-clause node-id digest order. `CheckedPackageV2::protocol_activation_relation` returns the rows for all local clauses in ascending clause node-id digest order, with exactly the public row fields and `Origin`/`Each` variants specified above, and with no separate wire or identity member; repeated calls and equal admitted packages return equal rows without changing the package or charging reader work. The row contains the protocol clause key, `origin` or `each` with the trigger key, and the required temporal clause key. No requirement gives no row; two source `requires temporal` declarations encoded as one canonical set entry give one row; two duplicate wire entries refuse `invalid_semantic_graph` under the clause's set rule and expose no relation; a second distinct requirement gives two rows; a temporal clause sharing the clock but not required gives no row. A missing `requirements` or `each` trigger dependency refuses `missing_declaration`/`missing-name` at that target and exposes no partial relation. | Test (TC-048; QSpec TC-946) |
| FR-038-AC-325 | The protocol step compares every present refinement `class_catalog` against the complete concrete and abstract classes reconstructed from selected model declarations and the admitted local or directly selected dependency protocol graph, including distinct `fork` and `join` classes on one parallel node, checked binder types and source-default memory classes, even with no step row. The concrete refinement `semantic_type` resolves to a selected package `model_root`, while an owned clause's separate `owner_model` resolves to an object-type node of a model owned by that selected declaration; classes from that clause count even without a step row. Foreign, unselected and wrong-kind owners cannot complete the selected inventory. A source subject with `sc`, `tso` and `ra` defaults admits three distinct `memory drain` entries with `drain: store-buffer-flush`, keyed by (`class`, `memory_model`) and ordered `ra`, `sc`, `tso`, followed by `memory flush` for `tso`; repeated selection of one default adds no entry. An `on each (x: T)` clause admits `activate <protocol>` associated with that clause node and binder types `[T]` for non-record `T`, or the effective fields of record `T` in source supertype order then own declaration order, retaining a diamond field once and placing a redefining field where the hidden field stood; reversing source supertype order changes that binder order. An `on origin` clause admits no activate entry. An ambiguous same-name inherited field without redefinition refuses `ambiguous_declaration`/`ambiguous-name` at `/semantic_graph/nodes/{n}/body/class_catalog/{side}/{i}/binder_types`. A wrong subject, clause/node association, or wrong or reordered binder signature refuses `invalid_model_binding`/`malformed-declaration` at the catalog entry; a missing or extra class refuses `invalid_semantic_graph` at the catalog array or entry; duplicate or misordered catalog keys refuse at the earlier refinement canonical-order check, with memory keys compared by class then model. An absent required or forbidden present catalog refuses `invalid_semantic_graph` at its member. A protocol-body defect reports before this graph join; a refinement-step row/catalog defect reports before either. A later request memory override does not change checked-package catalog facts or package identity. | Test (TC-048; QSpec TC-310) |
| FR-038-AC-246 | Implemented (IR-803). For a reader-admitted `quire.op.structural.eq` application, `composite_application_operands` returns `CheckedCompositeOperator::Eq` together with its operands; for an otherwise equivalent reader-admitted `quire.op.structural.ne` application it returns `CheckedCompositeOperator::Ne` together with its operands. Between those cases, only the returned operator and the selected application's identity and occurrence may differ; operands (including order, child identities, semantic types and domains), ordinals and `consumed_work` are equal. The AC-182 public-fields inspection in `tests/it/checked_package_v2_composite_operands.rs` (`inspect_public_fields`) must exhaustively match both `CheckedCompositeOperator` variants, without a wildcard arm or `#[non_exhaustive]`. The consumer needs no separate operation lookup. TC-048 composite operand tests verify both cases. | Test (TC-048) |
| FR-038-AC-303 | A self-typed `relation`/`population` node with no declaration, nominal preimage or recursion group, an empty aggregate body, empty dependencies and a `ModelOwner` admits when its key matches the closed `ModelDeclarationNode` preimage. The two QSpec P7/P8 preimages yield their distinct published keys; changing either owner string changes the key. An absent owner on the node refuses `malformed_wire` before key or join checks, and a wrong fixed member refuses by the existing model-declaration shape rule. | Test (TC-229) |
| FR-038-AC-304 | After FR-154 admission, the reader joins a population reference's `owner.node` to the raw `identity` of a `populations` record in the document selected by `owner.identity`. A record without an `identity` string, with a `node` member, with an unresolved `kind` construct, or with a construct whose meaning is not population refuses `invalid_model_binding`/`malformed-declaration` at the selection before the join. The QSpec `counters` and `archive` owners admit; an unknown node, an object-type node claimed as a population, and a population node claimed as an object type or relationship each refuse `missing_declaration`/`missing-selection` at the first offending node id. The reference node's digest never substitutes for the declaration key. | Test (TC-229) |
| FR-038-AC-305 | `quire.op.model.all_instances` and `quire.op.model.lookup` each admit a reference to a declared population as operand 0 and list it in dependencies; the same declared population is one reference node across both reads, while the two named populations yield distinct reference, application and package identities. A `value`/`parameter` typed `Population<O>[N]` remains an admitted operand 0. A unit maximum is represented only in the application's result-type collection bounds, not in the declared population node. | Test (TC-229) |
| FR-038-AC-306 | A declared population reference used as a `collection_kind` operand, as the `population` link of a reaches `relationship_end`, or as the declaring node of a `field`, `operation` or `relationship_end` member refuses `ill_typed`/`operator-ineligible` at that operand or member. An otherwise identical binding typed by `bounded_domain`/`model_population` keeps its existing outcome. | Test (TC-229) |
| FR-038-AC-307 | The reader consumes QSpec's selected-checkout published population-reference vectors and operation fixture directly, without copying them into this repository: it admits the amended positive package and checks its package, application and operand identities; it refuses the retired ownerless relation/population operand as `malformed_wire`; and it matches the `population_reference_cases` adverse outcomes by code, cause and locus. | Test (TC-229) |
| FR-038-AC-299 | A `quantity` literal typed by a `scalar_type`/`unit` node admits with family `quantity` for the canonical magnitudes `3`, `0`, `-5`, `1/1000` and `-7/2`. Each of `03`, `+3`, `2/4`, `4/2`, `1/0`, `0/5`, `3/1`, `1/-2`, ` 3`, `2.5` and the empty value refuses `invalid_package`/`invalid-value` at `value`. A quantity literal typed by an integer, string or Boolean node, and an integer literal typed by a unit node, refuse `ill_typed`/`operator-ineligible` at `type`; the V2 schemas enumerate `quantity`. | Test (TC-230) |
| FR-038-AC-300 | A field, operation parameter and operation result `typeRef` `ix://quire/unit/ms` resolve to the checked package's existing `ms` unit node and retain `Quantity<ms>` under scalar, optional and set multiplicities. A missing `ix://quire/unit/ghost` refuses `missing_declaration`/`missing-name`; a `Quantity<s>` or integer argument to a parameter requiring `Quantity<ms>` refuses `ill_typed`/`operator-ineligible`. No unit type node is synthesized from the model document. | Test (TC-230) |
| FR-038-AC-301 | A unit, dimension and record with the same qualified declaration name admit as three namespaces; two units or two types with that name refuse `ambiguous_declaration`/`ambiguous-name` at the later node in node-id order before operation checks. | Test (TC-230) |
| FR-038-AC-302 | TC-230 consumes QSpec's selected-checkout QL-01..04, MU-01..06 and NS-01..03 cases directly or constructs their recorded wire shapes through the public reader, checks accepted and refused outcomes by code, cause and locus, and compares strict coverage with the main-branch baseline. | Test (TC-230) |
| FR-038-AC-308 | For an admitted graph with K distinct requested leaves over one shared R-node closure, at K=2,4,8 and R=4,8,16, the reader validates each body once and retains the resolved adjacency and body-term count; lowering performs no further body grammar validation, reuses closure reachability per strongly connected component, and does not scan the full closure separately to decide each candidate bound. Per-request node, term and edge charges, the consumed value at a work-limit stop, each record and the complete package canonical bytes equal the independently lowered baseline. Admission performs R+K body grammar validations for the R shared nodes and K distinct leaves, and lowering performs zero additional validations; a grouped recursive component and a stop within the shared closure preserve the same outcomes. | Test (TC-050) |
| FR-038-AC-309 | `CheckedPackageV2::lower_independent_records` returns one record per requested key in input order, each byte-for-byte equal to the record from `lower` called with only that key and the same profile and retained byte ceiling. For a shared R-node closure under K requests, graph/source indexes and component closures are built once and body validation is not repeated. When every one-key package fits but their union exceeds the byte ceiling, this method retains each one-key record while `lower` on the union reports its existing whole-call `failed` result. | Test (TC-048, TC-050) |
| FR-038-AC-310 | Both published QSpec shipped_projection_cases (complete occurrence-free graph and empty array) fail the schema and refuse malformed_wire at /identity_preimage/identity_projection, before identity validation. Every other extra immediate member of the seven-field wire selection object refuses malformed_wire at that member's pointer; nested selection records retain their own rules. Removing that member admits the published package with its recorded package_id. Reader and canonical writer ship each complete body once in semantic_graph.nodes; the hash preimage includes every complete node except occurrences in graph order. | Test |
| FR-038-AC-311 | The typed V2 node-family/form vocabulary admits exactly `hybrid`/`hybrid_model`, `hybrid`/`hybrid_mode` and `hybrid`/`hybrid_transition` under the `hybrid` tag. QSpec TC-966's transition with a declaration, mode occurrence of role `claim`, reordered model bindings, class `continuous`, reference target index, mode with no rates, unknown `hybrid_flow` form and ownerless model each refuse `malformed_wire` at strict wire validation, before hybrid body checks; the valid `Braking` and `Tank` nodes reach those checks. | Test (TC-048) |
| FR-038-AC-312 | The reader admits TC-966's `Braking` model, modes, transition and `SlowEnough` claim with their published structural keys, self semantic types and exact dependencies; a rebuilt rate-term edit changes the referenced mode, model, model-typed `over` parameter, claim and package identities, while rekeying an unchanged node preserves its key. A stale hybrid node key or mismatched dependency list refuses at the prior key or graph-shape stage, before a hybrid body defect. | Test (TC-048) |
| FR-038-AC-313 | With dependencies, keys and package id rebuilt after each edit, the reader runs QSpec FR-481 model checks 1 through 4 in order: repeated mode/default initial, variable type/level/name, mode declaration/rate count/term types/transition index and reset conflict, then initial mode and condition. TC-966's HY-03 through HY-08, HY-17's rebuilt-key initial-entry cases and HY-18's rebuilt-key reset-conflict case match their published code, cause and locus; when two model checks fail, the earlier check wins. HY-11's repeated declared mode name is refused by earlier name validation, and HY-18's invalid `eq` relation is refused by strict wire validation, both before these model checks. | Test (TC-048) |
| FR-038-AC-314 | Model term reads and called-function types are checked after initial entries; clock rates and the required hybrid variable are checked after reads. TC-966's foreign `w` read, nonliteral clock rate, all-clock `Braking` and all-clock `Timer` refuse with the published code, cause and locus. TC-966's `Tank`, whose rate and reset value reference another variable and whose `hybrid` variable has a literal-one rate, admits. An unreferenced hybrid transition refuses after all model checks and before claim checks. | Test (TC-048) |
| FR-038-AC-315 | The temporal step precedes the hybrid step: a hybrid claim's integer or timed interval is admitted only under its matching FR-370 profile, including timed `[q, *]` under `quire.temporal.timed/v1` and integer `[2, *]` under `quire.temporal.infinite-trace/v1`. A wrong interval form refuses `invalid_package`/`operation-member-mismatch` there before a fairness defect. The surviving `SlowEnough`, `Slows` and `Recent` claims are read from their published graph nodes with their respective clock, activation, capture, fairness and read levels. | Test (TC-048) |
| FR-038-AC-316 | After all models and orphan modes/transitions, hybrid claims are checked in node-id digest order for clock/activation/guard/capture form, fairness, then model-variable/activation/capture reads. TC-966's `wall` clock, mistyped `each` trigger, fairness member naming `Halted`, `holds` reading `h` and `holds` reading a parameter at the wrong level refuse with FR-481's code, cause and locus; a valid claim and a later protocol defect yield the protocol refusal, while a hybrid defect and a protocol defect yield the hybrid refusal. | Test (TC-048) |
| FR-038-AC-317 | TC-048 reads QSpec TC-966's published `hybrid-node-vectors.json` and `positive-all-families.json` from the selected QSpec checkout, without a local copy. Strict-wire cases are checked before identity, rebuilt-key semantic cases after it, with exact code, cause and locus. The IR reader does not substitute its own process-provider or host trace checks for QSpec's HY-15 and HY-16. | Test (TC-048) |
| FR-038-AC-318 | The public reader admits exactly QSpec FR-322-AC-120's eight `timed_model` forms with the ordered root, tick-source and clock-place members stated above. Each admitted form retains its published semantic type, occurrence roles, owner, declaration and complete body; the root's time dimension matches the source unit's nominal dimension; a tick source model and operation-anchor context match the root model; each resolved-path row matches its path step and selected model declaration, including collection bounds and terminal `Clock`. An unknown form or malformed outer/body member refuses at strict wire validation before any timed-model body check. The writer emits one complete body per node and no wire identity projection. | Test (TC-048; QSpec TM-06..TM-21) |
| FR-038-AC-319 | For a timed model with source, place, reset, guard, invariant, urgency and constraint nodes, graph-shape and structural-key checks compare each node's exact dependencies and QSpec preimage before the timed-model step. Changing one meaning-bearing body or owner member and rebuilding dependent keys changes the affected node and package identities; keeping a stale key or dependency list refuses at the earlier stage. Reordering only source occurrences changes neither identity. | Test (TC-048) |
| FR-038-AC-320 | The timed-model step reconstructs the selected model from exactly one root and its referenced nodes. A missing root, dangling or wrong-form target, repeated place, noncanonical path, and invalid or non-time quantity each refuse with FR-322-AC-120's paired code and cause at the failing member, visiting timed nodes in ascending node-id digest order. The valid QSpec TC-367 CK-25 model admits and retains its normalized constraints. | Test (TC-048) |
| FR-038-AC-321 | A package with both a timed-model defect and a protocol or operation defect reports the timed defect; one with a hybrid defect and a timed defect reports the hybrid defect. Strict-wire, graph-shape and stale-key defects continue to precede either body step. | Test (TC-048) |
| FR-038-AC-322 | The public V2 writer round-trips an admitted timed model through the reader with the same complete node order, semantic identities, selected model and exact normalized quantities, and emits no `identity_preimage.identity_projection` member. | Test (TC-048) |
| FR-038-AC-323 | For each timed root, the `time_dimension` reference names the exact selected base `Time` dimension node, and the source unit's `dimension_node_id` equals that node by identity. A tick source that owns a clock place refuses `invalid_package`/`clock-without-dense-time` at the place's model reference target. A source unit whose dimension is base `Length`, a derived non-Time dimension or a nominal derived-Time dimension refuses `ill_typed`/`invalid-time-quantity` at the source unit reference target `/semantic_graph/nodes/{source}/body/members/1/value/target`, for both dense and tick sources; dimension equivalence, matching terms, names or declaration order do not replace identity equality. | Test (TC-367; TM-23..TM-29) |

FR-038-AC-66 is retired and its ID is not reused (ADR-0056). It required that every
application of operator class `case`, `temporal_formula` or `temporal_fairness` be
refused `unsupported_construct`/`expression-form`, which IR-549 lifts: those classes
are admitted at their node roots (FR-038-AC-96 through FR-038-AC-100) and the
reader does not blanket-refuse those temporal/case forms (FR-038-AC-101).
IR-661's planned relationship-navigation-only refusal does not reinstate AC-66.

## Dependencies

The typed composite operand projection supplies package-authored positions to
[CG FR-033](ix://agent-ix/quire-contract-codegen/FR-033).
[QSL FR-358](ix://agent-ix/quire-spec-language/FR-358) owns replay domain
admission and [QSL ADR-013](ix://agent-ix/quire-spec-language/ADR-013) owns
parity encoding; their drawn proof-bound values are not IR accessor outputs.

[QSpec FR-464](ix://agent-ix/quire-specification/FR-464) owns the twenty
protocol operators, the clause body, keys, membership, binder rules and
protocol reader order. [QSpec FR-465](ix://agent-ix/quire-specification/FR-465)
owns role, channel, queue, finish and terminal bodies;
[QSpec FR-466](ix://agent-ix/quire-specification/FR-466) owns control and
compensation bodies. QSpec owns both checked-package V2 schemas, the
TC-946 through TC-948 vectors and their fixtures. IR consumes those artifacts
by checkout path and does not publish copies of them.

QSpec FR-370 (temporal clause body and temporal operation identities) and FR-440
(union, union value and case nodes) own the encodings that FR-038-AC-96 through
FR-038-AC-101 admit and check. QSpec FR-322 "Body grammar" (AC-39 through AC-41), FR-341-AC-6 and FR-370 (AC-5, AC-12) own the flat wire and the absence of a depth limit that FR-038-AC-114 through FR-038-AC-118 check (QSpec TC-233, TC-303 and TC-427 BG-02, BG-03). QSpec FR-322 (AC-4, AC-8, AC-10, AC-28, AC-35 through AC-37), FR-201 (AC-2, AC-3) and FR-195 (AC-1 through
AC-5) own the normative V2 wire, identity-domain and lowering semantics;
quire-specification:TC-217 names this repository as their consumer evidence
owner. The first-party `agent-ix/quire-canonical` crate owns the RFC 8785
encoder that "Canonical encoding of the wire types" and "Every identity digest
is computed through quire-canonical" consume (FR-038-AC-74 through
FR-038-AC-80, FR-038-AC-89 through FR-038-AC-95 and FR-038-AC-109 through FR-038-AC-111), including its
`Encode` for `serde_json::Value` (quire-canonical #7, merged).
[FR-040](./FR-040-admit-frame-entries-and-state-clauses.md) owns the
QSpec FR-340 `modifies` entry shape, scope and `deletes_scoped`, the FR-341 state clause and FR-342
operation anchor bodies, and the `model` form set.
