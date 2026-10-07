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
  - target: ix://agent-ix/quire-specification/FR-440
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
  Located model-declaration refusals additionally retain authentic declaration
  identity and valid typed origin under the PLANNED / UNRUN retention contract
  below; missing/malformed origin and pre-node unavailable identity stay absent.
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

- A planned typed composite operand accessor, `composite_application_operands`,
  returning authentic positional children, authored shapes and introduced positions
  with exact decimal bounds, explicit unbounded/Whole descriptors and typed refusals;
  see "Typed authored domains of composite equality operands" (AC-177 through AC-182).

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
thread whose stack is 256 KiB. The closed-schema decode does not read a node
`body`, an identity-projection `body` or a diagnostic `details` term, which can
nest as deep as the strict parse reads: they are taken out of the document before it
and put into the decoded wire after it, and the lossless-decode check compares the
long arrays one element at a time, so a body nested between the grammar and the
parse's limit (a window of about 90 to 126 JSON levels) refuses at the body grammar on
a 256 KiB stack in a debug build, and no copy of the whole document is built beside
it. The recursion over such a value that remains is bounded by the strict parse's
limit of 128 levels: the parse's own, its drop, and the comparison of an
identity-projection body (which the body grammar does not check) with its node's body
when the projection is stale, which compares the `Value`s and serializes them with
`serde_json::to_value` once per level before the iterative `first_difference` locates
the member; a stale projection body nested at the parse's limit refuses on a 256 KiB
stack in a debug build. Every value an admitted package holds nests to a
depth the grammar fixes, so its clone, comparison, `Debug` rendering, lowering and drop
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
body validators read it from the JSON at the wire edge, including when
lowering re-walks an admitted body through the same validator.

The reader shall recompute `package_id` as the SHA-256 of the RFC 8785
canonical bytes of `identity_preimage` under `quire.package.semantic/v2`,
produced by encoding `CheckedPackageIdentityPreimageV2` through
`quire_canonical::Encode` (see "Canonical encoding of the wire types" below)
and not through `serde_json`, require the preimage's lock members to equal the lock, and require its
`identity_projection` to equal the occurrence-free projection of the graph in
graph order. It shall check each digest's declared domain before its bytes,
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
   precedes the digest comparison), `stale_dependency`/`byte-digest-mismatch`
   at its `digest` when the document's RFC 8785 SHA-256 is another digest,
   `invalid_model_binding`/`wrong-model-selection` at its `identity` when the
   document names another identity, and the first FR-154 declaration refusal
   of the document at the entry. Two entries naming one `identity` but
   different digests are one identity selected twice: the later entry refuses
   `stale_dependency` at its `digest`, whatever documents the evidence
   supplies, before any document is read.

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
occurrence-free identity projection it compares against
`identity_preimage.identity_projection`, so a package whose preimage omits a
`declaration`, a literal `type` or an application `operation` refuses rather
than admitting a projection that disagrees with the graph.

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
| `CheckedPackageIdentityPreimageV2` | `Encode` | `identity_projection` holds `CheckedNodeProjectionV2`, whose `body` is a `Value` |
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
`byte-digest-mismatch` and the identity refusals of the same row. The document is
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
every other read failure but the byte limit is
`stale_dependency`/`byte-digest-mismatch` at the row's `digest`.

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
outcome of a raw-path document is pinned here only under a digest that is not the
document's raw digest. QSL's FR-056 refuses unparseable bytes selected under
their own raw digest `invalid_model_binding`/`wrong-model-selection`, and this
reader's reading of that case is tracked as IR-578 and not specified here. A number below the double range (`1e-400`) reads as zero with
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
on these documents too, except for the raw-digest case above (IR-578). QSL's rule also checks the members of an invocation or snapshot
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
   document's object types, systems interfaces, integer value types and
   relationships, node by node in ascending IR node identity, and refuses with
   the first declaration refusal at the row (`/lock/model_selections/<i>`).
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
selection row with its typed code, cause and RFC 6901 pointer. The refusal
does not retain FCD origin, sourceIdentity, path, span or artifact metadata;
a generated origin has no source span. The schema is
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
for the destination end's typed multiplicity. An otherwise well-formed edge
or pair of operands that fails reference-edge admissibility refuses
`ill_typed`/`operator-ineligible`. The result remains the catalog's Boolean
node, not navigation's destination type. Runtime traversal remains FR-043's
owner responsibility; this reader checks the static package binding only.

These declaration checks run in step 1 above, including for an otherwise
unused relationship. Their outer pointer is `/lock/model_selections/<i>`;
the planned refusal-retention contract retains code, cause and that pointer,
plus the authentic identity and valid origin of the declaration actually refused.
A nested relationship refusal shall name that relationship, not its enclosing
object type or an inferred owner. A selected-document declaration identity is
not a checked graph-node digest: `locus` retains its existing separate meaning
and is populated only where the reader actually determines a graph-node key.
These metadata requirements are PLANNED / UNRUN until implemented and tested.
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

When model intake refuses a located declaration, the reader SHALL retain the
authentic IR node identity and the node's valid origin through the public refusal
boundary. This requirement is PLANNED / UNRUN diagnostic metadata work; it does
not assert the current implementation retains these values. The selected and
content-authenticated document shall supply the values. The reader SHALL NOT
infer them from a name lookup, display text, selection-row pointer, graph digest,
enclosing owner or another declaration. An invalid member of a nested declaration
shall retain that declaration's own available identity and valid origin.

The planned public `CheckedPackageRefusal` field additions are
`declaration_identity: Option<Box<str>>` and
`declaration_origin: Option<CheckedDeclarationOrigin>`. The identity field shall
be present whenever the refusal is located at an authentic model declaration
identity. Its absence shall represent unavailable identity, never an empty string
or a reconstructed identity. Byte-stream or package-admission refusals made
before a declaration is identified shall retain absence rather than invent a
node. Existing `code`, `cause`, `path`, `locus`, `contract_version` and
`document_pointer` shall retain their meanings and values.

`CheckedDeclarationOrigin` is a new IR-owned typed public API allocation, not an
existing upstream API or an untyped JSON/prose payload. Its two forms are:

- `Source { source_identity: Box<str>, path: Box<str>, start_line: u64,
  start_column: u64, end_line: Option<u64>, end_column: Option<u64> }`.
- `Generated { generator_identity: Box<str>, generator_version: Box<str>,
  input_identities: Vec<Box<str>> }`.

The reader SHALL preserve every admitted origin member's value exactly, including
identity/path/version strings, coordinates, optional-member presence and generated
input identity order and multiplicity. Source start coordinates are positive;
source end coordinates are independently optional and positive when supplied.
Retention shall add no coordinate maximum, paired-end requirement or span-order
rule beyond the owning origin validation and existing selected-document exact-number
admission. That admission already refuses integer magnitudes above 2^53 before
model declarations; `u64` represents every admitted positive coordinate, including
2^53, without a new numeric cutoff. Generated origin shall carry no invented
source identity, path, coordinate or span. Version text shall remain the supplied
text, not a normalized semantic-version rendering. Input identities shall retain
their admitted nonempty sequence; no sorting or deduplication is permitted.

When the origin is missing or malformed, the reader SHALL retain no origin.
This includes a value containing both origin branches, an invalid branch or
missing required branch member; the reader shall not salvage a partial branch or
manufacture coordinates. Origin retention shall not add a new refusal, alter an
existing code/cause or reorder admission/declaration checks. Existing malformed
relationship-origin refusal remains authoritative; other refusal paths shall not
be reclassified merely to populate metadata. A valid source origin's locus is
retained as supplied; a generated origin has no source span. Neither shall be
converted into the different raw-byte `CheckedSourceRegion` representation.

Both new fields shall be explicit members of every public refusal constructor;
`Option` represents authentic unavailability, not a defaulted or omitted API member.
The selected refusal and its owned metadata shall survive declaration selection,
private `SelectionRefusal` conversion, selected-document release and every public
reader/dispatch forwarding boundary without loss. The selected offending node
shall supply the metadata before its document is released. All refusal constructors
shall explicitly distinguish authentic available declaration metadata from absence;
no default, compatibility layer, fabricated locus or cause parsing is allocated.
Consumers that retain `CheckedPackageRefusal`, including a driver handoff refusal,
shall retain the full typed record rather than flatten its metadata into a message.

Existing selection admission, node/table/member-path refusal precedence, ordinary
error codes and incomplete/no-partial-package behavior remain unchanged. Metadata
retention and its allocation shall use the existing bounded read/work accounting;
no raised ceiling, unbounded diagnostic collection or second document parse is
permitted. Actual origin and constructor/consumer assertions are owed CODE gates,
not evidence supplied by specification publication.

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
selection-row pointer and declaration metadata above. Each ancestor or conformance edge followed and each relationship
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
entry's refuses `stale_dependency` with cause `byte-digest-mismatch` at the
entry's `package_id.digest`. The entry's `identity` and `package_id` are the
only binding: the entry names no version, and none is compared with anything
the consumer supplies. This binding runs before any node is read, so before any
`dependency_reference` check. A consumer therefore supplies every
`dependency_selections` entry's package, under its library identity, with
`insert_dependency_package`.

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
FR-322 does not fix which nodes of a `function` node are its signature; this
reader takes the function's own `semantic_type` (its result type), and from
the nodes its `dependencies` list and the nodes its own body references (a
non-application function body need not list them), the `semantic_type` of each
`parameter` node and each type node itself. The closure follows `dependencies`,
`semantic_type` and the `reference` terms of each node's body. A `model` or
`relation` node, and a node with a nominal preimage owned by a domain package,
is a `ModelOwner` carrier. A refusal carries the calling node as its locus.
Each supplied dependency's graph is indexed once when the lock stage binds it,
one unit of `work` per node at the entry's pointer; each lookup is one unit at
the term, and the closure walk is one unit per node visited and per body term
walked.

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
nodes; a duplicated member name refuses `invalid_package` with cause
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
node other than itself. Its body is exactly `aggregate{[binding "name" = a
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

FR-038-AC-153 through FR-038-AC-155 specify the planned IR-646 owner-wire
reader. They do not lift FR-038-AC-145 through FR-038-AC-149's current
recursion-group skip or the soundness limit it records. IR-630 owns the later
complete structural-key stage, including in-group re-derivation and retirement
or amendment of those interim criteria and TC-226. QSpec FR-322's derived-shape
rule and QSL FR-092/FR-094 supply that later stage's preimage authority;
IR-646 only exposes and validates the owner it will read.

The optional `owner` member of `CheckedSemanticNodeV2` and of its
`identity_projection` entry has QSpec FR-322's closed `NodeOwner` shape:
`SourceOwner` `{kind: "source", authority, identity}` or `ModelOwner`
`{kind: "model", identity, node}`. The projection carries exactly the node's
owner, so changing it changes `package_id`. A node is structural for this
presence rule when it has no `nominal_identity_preimage` and its body root is
not `application`. Every declared structural node SHALL carry the declaring
unit's `SourceOwner`. An undeclared `model`/`object_type`,
`model`/`systems_interface`, `relation`/`relationship` or `function` node
SHALL carry its `ModelOwner`. Every other node SHALL omit `owner`: this
includes anonymous types, undeclared values and parameters, nominal nodes
whose owner is in their nominal preimage, and application-keyed nodes. A
`DefinitionOwner` is not a node owner. The reader SHALL derive no missing
owner from occurrences, the source map, the lock, or node position.

The strict wire/schema step SHALL refuse a missing required `owner` as
`malformed_wire` at the node or projection object that lacks it. It SHALL
refuse an owner on a node that must omit one, a wrong owner kind, `null`, a
wrong member type, or an unknown owner member as `malformed_wire` at `owner`;
the same rule applies to each projection entry. If a well-shaped projection
owner differs from its node's owner, the reader SHALL refuse
`invalid_package`/`invalid-value`
at `/identity_preimage/identity_projection/{i}/owner` after the schema step and
before the owner join, even when the package id is recomputed over the changed
projection. This applies to both `SourceOwner` and `ModelOwner`; the projection
cannot substitute a different owner for the node. There is no admission route
for an owner-less declared or model-owned structural node.

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
`interfaceFeatures`, and `relation`/`relationship` requires a relationship.
An operation clause function requires an operation member; an invariant clause
function requires an object type. The reader SHALL check this
for every model-owned node whether or not any other node references it, and
charge one validation visit for each document lookup. A missing selection,
declaration, member or matching kind SHALL refuse
`missing_declaration`/`missing-selection` at that node's `node_id`.

An owner that passes this join proceeds to the existing reader stages;
IR-630 will expand the key stage to read this member for every structural
key, with `correspondence`/`abstraction_relation` remaining under FR-346's
distinct FR-451 key check. `source_locus` remains a structural form, and
application-keyed nodes remain owner-free.

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
under the unchanged `node_id`, with `identity_projection` and `package_id`
recomputed, therefore admitted before IR-627. A consumer that takes the range from the
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

1. When a node of one of these shapes carries no `declaration` and is not
   skipped (the next section), the reader shall re-derive its `node_id` from the node's own `node_tag`,
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
reads. A second open route is a node of a skippable shape that sits on a names
cycle (see "Derived-shape nodes inside a recursion group" below), which is not
the node of a range: `integer_range` is never skipped.
The owner-free ungrouped portion of FR-038-AC-131 closes its listed routes;
owner-bearing, declared and grouped routes remain open until IR-630 completes
FR-038-AC-131 and FR-038-AC-150. FR-038-AC-132 keeps their evidence at the
authoritative checkout, so the present stage is a partial defence.

### Derived-shape nodes inside a recursion group

Measured at `origin/main` of QSL and of QSpec (IR-627, whose own text and the
relayed owner answer are claims and were re-measured). A node of a derived
shape can sit in a recursion group: QSpec FR-143 "Recursion rule" admits a
recursive record through a `?` field, an `Option` or a collection whose minimum
is `0`, and never through a tuple position, and QSL FR-092 "Recursion groups"
(its sections "An in-group node's preimage" and "The group order") keys the
`Option` or `Sequence` node of such a record, and the `collection_bounds` node
over it (vector G9, whose `semantic_type` is a `group_reference`), as members of
the record's group. The earlier reading of this section, that such a node is
refused because the decided preimage holds `recursion` `null`, is SUPERSEDED:
a derived-shape node inside a recursion group is legitimate, and the reader
shall not refuse it for being in one.

What QSL FR-092 requires of an in-group key, to the extent measured: a group is
the strongly connected component of the graph in which a node names another by
a body `reference`, its `semantic_type`, a `result_type`, a literal `type` or a
member `declaration`; a structural member's preimage carries `recursion`
`{size, ordinal, group}` and an application member's `{size, ordinal}`; each
position that names a member of the node's own group, `semantic_type` included,
holds `{term: "group_reference", ordinal}`; and `group` is a digest over the
group-local preimages of every member. FR-092 reads a member's ordinal from its
position among the group's nodes in the package's graph order, which QSL's
emission sets to the group order (its "The group order" section); this reader
derives an application member's ordinal the same way and recomputes no content
order.

**Why the decided stage cannot verify a node of a real group.** The group digest
binds the group-local preimage of every member. A group that holds a declared
record or function (every group that source can form: QSpec FR-143 admits
recursion only through a named field, and QSL FR-092 "Groups that collide" says
each group holds a declared record or function) has a member whose preimage
carries `declaration` and the `SourceOwner` of its unit (FR-092 "The
`quire.structural-node/v1` preimage"; QSpec `node-identity-preimage.schema.json`).
A `CheckedSemanticNodeV2` now carries `owner` (FR-038-AC-153), but this
owner-free re-derivation stage does not yet put it in a structural preimage.
IR-630 owns the owner-bearing and grouped preimages, so this stage cannot
re-derive those members. A group with no declared member, as in QSL
FR-092's vector G1 (an `option` over itself, which the key function keys and
no declaration forms), needs no owner and is derivable from the wire alone; QSL
FR-092 does not say the checker can produce one from source.

**What can be in a group.** Of the ten shapes, six can legitimately be a member
of a group: `option`, `set`, `bag`, `sequence` and `ordered_set` (the node that a
recursive field names) and `collection_bounds` (it names the collection at its
`semantic_type`, as G9 does). Four cannot, and are verified whatever label they
carry: `scalar_type` `boolean` and `integer` and `bounded_domain`
`integer_range` name only the `Integer` node, which names nothing but itself, so
no path leads from them back to a node that names them; and `composite_type`
`reference` names a model declaration node, whose body is empty so that
`Reference` attributes "key without a cycle" (QSL FR-094, the empty-body
paragraph after its model declaration table). A range read from a
node whose wire form is `integer_range` is therefore verified in every package,
with no owner needed (the wire-form caveat is in the limit below).

**Decided: skip, do not refuse.** The names graph the reader uses is its own
cycle graph (`validate_graph`) with the edges of `dependencies` left out: a node
names another by its `semantic_type` (a self-typed node excluded), a body
`reference` target, a literal `type`, an application's `result_type` or a frame
entry. QSL FR-092's graph differs by also holding a member `declaration` and not
holding a frame entry. Neither difference changes a component that holds a
skippable shape: a type node's body holds only `reference` terms and literals,
and a member `declaration` names a model node whose body is empty. The
re-derivation stage shall skip a node if and only if all of these hold: its wire
form (`node_tag` and `semantic_form`) is one of the six shapes; it carries a
`recursion_group`; it lies on a cycle of the names graph; and every node of the
strongly connected component that holds it carries that same `recursion_group`.
Nodes carrying the label outside the component do not matter. The stage shall
neither refuse nor verify the key of a skipped node. A node of a skippable
shape that carries a label and fails any of the other three conditions is
verified as an ungrouped node: its key is derived with
`recursion` `null`, and it is refused `stale-node-key` when the stored key
differs and admitted when it holds the ungrouped key. That choice keeps the
admission of a lone label (`tc_048_package_id_covers_exactly_the_identity_preimage`)
and refuses a tampered node that carries one. A node of the four shapes is
verified whatever it carries, and one whose body is not closed has no derivable
key (item 3), so a body reference that would forge a names cycle through it
refuses. The reader keeps deriving the keys of application nodes inside a group
as the "Application node keys" section states, and a skipped node still passes
every other check the reader makes of it, among them FR-038-AC-18's refusal of
a cycle outside a declared `recursion_group`.

**Stated soundness limit.** Until the gate of FR-038-AC-150 lifts (IR-630,
blocked by QSL-638), the reader
does not verify the key, and therefore not the body, of a skipped node. A
changed `max` of the `Sequence<Tree>[0, 3]` node, with its `node_id` kept, is
admitted by this stage. The limit includes a forged group: a tamperer who
re-points a collection node's body `reference` at a `collection_bounds` node
that names the collection at its `semantic_type`, and labels both, makes a names
cycle of two nodes whose members all carry the label, and the stage
skips both (FR-038-AC-148). So a `collection_bounds` count, and the element type
of an `option` or a collection, are unverified by this reader for any node that a
tamperer can place on a names cycle; they rest on the package producer. A
range is not reached through a node whose wire form is `integer_range`, which is
never skipped, but the form is a wire member that a skipped node's key does not
verify: `integer_range` and `collection_bounds` share one body form, so a node
keyed as `Int[0, 1000]` can be retagged `collection_bounds`, have its `max`
changed and its `semantic_type` re-pointed at a labelled collection whose body
references it, and then lies on a forged names cycle and is skipped
(FR-038-AC-148). A consumer that reads a range by wire form reads none from that
node; a consumer that takes the field type from the domain document and the
node from the graph by key is handed a node whose body is not the type it
names. The reader cannot close this without the owner: a cross-check of a
skipped node's key against the member types of the selected domain documents
would need the field tables that FR-038-AC-136 to AC-144 only plan, and it
would still not cover a key the domain documents do not name. In-group
re-derivation (FR-038-AC-150) verifies every skipped node's tag, form and body
through its key, so it closes this route with the rest. The limit does not make
`dependencies` a route, because those edges are not in the names
graph (FR-038-AC-147).

**Consumer consequence.** quire-contract-codegen's lowering already returns
`requires_bound` under a bounds-required profile for a `scalar_type` or
`composite_type` node that declares a `recursion_group` (FR-038-AC-41), so a
legitimate recursive record is not what exposes a consumer. Its
`is_recursive_type` ignores `bounded_domain`, so a `collection_bounds` count read
through a forged group reaches the consumer unverified. The element-range route
is the same: a skipped node's body is not checked for closedness, so a skipped
collection's body `reference` can be re-pointed, or given a second reference, at
another genuinely keyed range, and a consumer that reads an element range
through that collection gets a forged one. Under a bounds-required profile
FR-038-AC-41 catches both, because a forged group labels a `composite_type`
node and that node returns `requires_bound`; under any other profile neither is
caught by this reader. No codegen requirement
backs a rule that a consumer must not trust such a count; a downstream
quire-contract-codegen requirement is needed for it (IR-624, in codegen's
FR-015 section, is the place to propose it) and this specification places no
obligation on that repository.

**Gated: in-group re-derivation.** Once the reader derives the preimage of each
declared member of a group, including its `SourceOwner` (IR-627-Q1 and Q4;
the follow-up is IR-630, blocked by QSL-638, with a spec amendment first), it
shall re-derive every structural node of that group by the QSL FR-092 rules
above, by the function of item 2, and by no second function (FR-038-AC-150). A
group with no declared member is not gated on the owner (Q4); it waits only for
the reader to compute the group digest.

**Trust root.** The member type step 4 derives comes from the domain document
that the caller's evidence supplies under the `model_selections` digest, whose
RFC 8785 digest and identity the reader recomputes (FR-038-AC-27). The check is
sound relative to that evidence. A tamperer who re-points `model_selections` at
a document declaring a narrower range changes `package_id` and, with no such
document in the caller's evidence, is refused `missing_import`/`missing-selection`
at the row's `digest` (FR-038-AC-27); which document is authoritative for a
lock is the caller's decision and is outside this reader, which admits a package
against whatever evidence it is given.

**Owner-free stage and IR-630 boundary.** This stage re-derives every
ungrouped structural node whose wire `owner` and `declaration` are absent,
except nominal, application, model, relation and FR-451
`abstraction_relation` nodes, which have their own key or owner checks. For
forms beyond the ten closed shapes above it uses QSL FR-092's
`quire.structural-node/v1` preimage with `semantic_type` null only for a
self-typed node, `declaration` null, `recursion` null and the wire body.
The stage skips every owner-bearing structural node, including an undeclared
model-owned function, because an owner-free preimage cannot authenticate its
body or key. IR-630 must re-derive those owner-bearing keys and the grouped
keys under their owner and group-local preimages. Until then a changed body on
an owner-bearing node can retain its old `node_id` and pass this stage if the
other reader checks admit it. That admission is a stated limit, not evidence
of FR-092 owner-key validation. Nominal, application and FR-451 key stages
retain their separate obligations.

**First-refusal order.** The reader shall run the decided re-derivation as its
own stage in the stale-key stage of `validate_graph`: after the graph-shape
stage ("Parameter and compound-unit nodes, and the application dependency
join") and the application key stage, and before the nominal key stage and
before every declaration, frame, state, temporal, abstraction and operation
step. Within that stage the reader shall visit nodes in ascending digest order
of the node's own `node_id`, charging one work unit per node, skipped or
not (a skipped node is still charged, FR-038-AC-149; the skip predicate is
FR-038-AC-145), and shall report
the first stale node at its own `node_id`. Because the stage does not run
inside the operation step, a tampered node is reported at itself and never at
an application that reads it, and ahead of any `ill_typed` defect of such an
application.

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
4. **Q4.** ANSWERED in part (QSL owner answer relayed 2026-10-05, QSL-635, and
   re-measured at `origin/main` of QSL and QSpec). A derived-shape node may sit
   in a `recursion_group` (QSpec FR-143 "Recursion rule"; QSL FR-092 "Recursion
   groups", vector G9), and members with equal full signatures are one node
   (FR-092 "The group order", item 5), so a group holds no two nodes of one key.
   The earlier IR reading, that such a node is refused, is superseded: the
   decided stage skips it ("Derived-shape nodes inside a recursion group").
   The `SourceOwner` question (how a reader recovers a declared group member's
   owner, which the group digest of FR-092 "An in-group node's preimage" binds
   and `CheckedSemanticNodeV2` does not carry) is ANSWERED by the QSL owner per
   the relay of 2026-10-05, QSL-638, option (a): the v2 wire carries `owner`, in
   the preimage member's JSON shape, on exactly the nodes whose FR-092 preimage
   has one (declared nodes, and model-owned nodes with `ModelOwner`) and
   nowhere else, with QSpec FR-322 and the QSL emitter changed under QSL-638
   and this reader reading it; the IR follow-up is IR-630, blocked by QSL-638.
   Still open, for the owners of QSL FR-092 and QSpec FR-322: whether a reader must recompute the content order of "The group order" or
   may read each ordinal from graph order, as FR-092 states for FR-322, in which
   case a package whose members are re-keyed under a permuted wire order admits
   (a stated limit of FR-038-AC-150, not a refusal). QSL FR-092's vector G1 (an
   `option` over itself) forms a group with no declared member, so the owner is
   not needed for it; does the checker form such a group from source, or only
   the key function (an owner-free group is derivable from the wire alone)?
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
    `Boolean`, `Integer`, `IntRange` carrying `lower` and `upper` as `i128`
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

**What the accessor does for a node admission did not check.** Admission runs
FR-322 step 2 (`ModelOwners::recover`) only at a node that a read, frame entry,
state clause or relation names, so an unread `model`/`object_type` node with a
non-selected key or a non-empty body is admitted. The accessor does not widen
admission to sweep every model node, because that would refuse packages
that admit today, a behavior change this requirement does not need: what is at
stake is only whether the accessor's answer for a node is the document's. The accessor therefore
applies the same step-2 rule at the call, to the node it is passed (item 17). It
returns the fields only for a node that rule recovers, so a node that a read of
it would refuse at step 2 is `NotModelObjectType`, and the result for a recovered
node is the document's table and no node body. Two admitted packages that differ
only in an unread model node's body can therefore return `Ok` and
`NotModelObjectType`; the spec states this, and does not claim the answer is
independent of the passed node's own fixed members.

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

### Typed authored domains of composite equality operands

When supplied an admitted `quire.op.structural.eq` application node id, an
occurrence belonging to that node, and a finite accessor work limit,
`CheckedPackageV2::composite_application_operands` shall return the application's
authentic operands and their package-authored domain structure through public
Rust types. The accessor shall admit only this operation identity and the
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
| `CheckedCompositeOperands` | `application: CheckedNodeId`, `occurrence: CheckedOccurrence`, `operands: Vec<CheckedCompositeOperand>`, `consumed_work: u64` |
| `CheckedCompositeOperand` | `ordinal: u64`, `child: CheckedScalarOperandChild`, `semantic_type: CheckedNodeId`, `domain: CheckedCompositeOperandDomain` |
| `CheckedCompositeOperandDomain` | `Literal`; `Parameter { shape: Vec<CheckedCompositeShapeEntry>, positions: Vec<CheckedCompositeDomainPosition> }` |
| `CheckedCompositeShapeEntry` | `type_node: CheckedNodeId`, `kind: CheckedNodeKind`, `path: Vec<u32>`, `edges: Vec<CheckedCompositeChildEdge>` |
| `CheckedCompositeChildEdge` | `target: CheckedNodeId`, `ordinal: Option<u32>`, `name: Option<Box<str>>`, `optional_presence: bool`; an ordinal is present for a field, tuple position or inner edge, and absent only for path-neutral alias/bounded-domain forwarding; a name is present only for a record field; optional_presence is false except for the prescribed optional-presence record wrapper |
| `CheckedCompositeDomainPosition` | `key: CheckedCompositeDomainKey`, `type_node: CheckedNodeId`, `authored: CheckedAuthoredCompositeDomain` |
| `CheckedCompositeDomainKey` | `Node { node: CheckedNodeId, path: Vec<u32> }` only |
| `CheckedCanonicalIntegerBound` | Validated admitted canonical decimal text exposed by `as_str()`; construction restricted to reader/accessor content |
| `CheckedAuthoredCompositeDomain` | `IntegerRange { lower: CheckedCanonicalIntegerBound, upper: CheckedCanonicalIntegerBound }`; `UnboundedInteger`; `Collection { kind: CheckedCollectionKind, minimum: CheckedCanonicalIntegerBound, maximum: CheckedCanonicalIntegerBound }`; `UnboundedCollection { kind: CheckedCollectionKind }`; `Enum { ordered: bool, members: Vec<Box<str>> }`; `UnboundedDepth { declaration: CheckedNodeId, reentry_paths: Vec<Vec<u32>> }`; `Whole { kind: CheckedNodeKind }` |
| `CheckedUnsupportedCompositeOperand` | `InlineInteger`, `InlineNonInteger`, `ApplicationSubterm`, `NonStructuralType`, `NonliteralGraphValue` |
| `CheckedCompositeOperandError` | The eleven variants and payloads below |

None of these types shall expose `serde_json::Value` or a serialized term body.
The result shall retain the actual application and supplied occurrence, and one
entry per `body.arguments` position in that order, with ordinals `0..n-1`.
Each entry shall retain its admitted semantic type and actual child identity.
Repeated references shall remain separate entries. A parameter reference shall
carry its own `GraphChild` identity, not its type or application identity.

A supported graph literal shall be a finite closed value subtree whose root has
an admitted structural-kind semantic type, in the record-value, tuple-value,
collection-value or option-value forms. Its descendants may use the admitted
literal and enum-value forms as well. A parameter reference, application subterm,
unsupported value form or value-cycle within it shall refuse `UnsupportedOperand`
with reason `NonliteralGraphValue`; the enclosing argument ordinal is retained.
The accessor shall establish closure through a bounded structural walk under the
same work limit without evaluating an expression. The value tag alone shall not
establish closure. A closed graph literal shall retain its own `GraphChild` id
and `Literal` disposition with no introduced positions. It shall not copy or
reconstruct the literal value, or replace that singleton with its wider type's
free positions. QSL owns the downstream parity domain and encoding decisions.

The accessor shall refuse top-level inline integer terms as `UnsupportedOperand`
with reason `InlineInteger`, and every other top-level inline literal kind as
`UnsupportedOperand` with reason `InlineNonInteger` (including `none`). It shall
fabricate neither a graph id nor an inline literal value. This is a defensive
accessor rule for integer terms: the owning catalog's `structural_kind` family
excludes Integer, and reader admission already refuses an integer operand of
`structural.eq`. No successfully admitted integer-inline equality fixture is
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
mapping is exhaustive over the eleven scalar and seven bounded-domain forms:

| Admitted form | Introduced descriptor and source node |
| --- | --- |
| `scalar_type/boolean` | No position; retain its shape |
| `scalar_type/integer` | `UnboundedInteger` at this node, unless the same path is already represented by an enclosing `integer_range` |
| `scalar_type/enum` | `Enum` at this declaration node, retaining `EnumDeclarationPreimage.ordered` and `members` in preimage order; an unordered preimage is byte-ascending, while an ordered preimage retains its semantic order |
| `scalar_type/rational`, `scalar_type/decimal`, `scalar_type/float32`, `scalar_type/float64`, `scalar_type/text`, `scalar_type/dimension`, `scalar_type/unit`, `scalar_type/compound_unit` | One `Whole` position at this node, unless an enclosing bounded-domain Whole at the same path represents it |
| `bounded_domain/integer_range` | One `IntegerRange` at this wrapper with its exact admitted min/max; forward to its semantic type at the same path without adding `UnboundedInteger` |
| `bounded_domain/collection_bounds` | One `Collection` at this wrapper with exact admitted min/max and resolved collection kind; forward to its collection type without a second collection position, and visit its element at child zero |
| `bounded_domain/rational_range`, `bounded_domain/decimal_range`, `bounded_domain/float_rounding`, `bounded_domain/text_bounds` | One `Whole` at the outermost such wrapper on this path; retain path-neutral forwarded shape nodes, but suppress additional Whole positions at that same path |
| `bounded_domain/model_population` | One Node-keyed `Whole` at this population-typed leaf; no population member walk or `Population` key |

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
| `IneligibleOperator` | `application: CheckedNodeId` | Known identity other than structural.eq, including structural.ne |
| `MissingChild` | `ordinal: u64, child: CheckedNodeId` | Referenced value child absent |
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

`structural.ne` remains ineligible here. [CG FR-033](ix://agent-ix/quire-contract-codegen/FR-033)
AC-12 and [QSL FR-358](ix://agent-ix/quire-spec-language/FR-358) include NotEqual;
that arm still requires an owning follow-on allocation for its operand accessor.
This allocation does not claim to enable all Eq/Ne or scalar equality claims.

### Application node keys

A node whose `body` is an `application` term carries a `node_id` that is the
SHA-256 of the RFC 8785 bytes of its own preimage, `{version, node_tag,
semantic_form, semantic_type, declaration, recursion, body}`, with `version`
`quire.application-node/v1` (QSpec FR-322 `application_node_preimage`). The
`recursion` member is `{size, ordinal}` of the node within its `recursion_group`,
the members of the group taken in graph order, and `null` outside a group. Each
`reference` in the body to a member of the node's own group becomes
`{term: "group_reference", ordinal}`, wherever the term stands (an application
argument, a binding value or an aggregate member), and a reference to a node
outside the group stays as it was. The reader re-derives the key for each such
node and refuses a retained key that differs as `invalid_package` with cause
`stale-node-key` at the node's `node_id`; a node whose `node_id` is that digest
is not refused for its key. This is the key check that FR-038-AC-35 exercises
for a `dependency_reference` callee, stated for every node whose body root is an
application term, the only place an application stands ("The flat wire"). This
section fixes what the
preimage is (FR-038-AC-88); "Every identity digest is computed through
quire-canonical" fixes who encodes and hashes it, and FR-038-AC-89 pins that the
key is the one the reader derived before the move, so the two do not overlap.
Each criterion below is observed
at the key check, which the unit tests call directly on a graph of application
nodes.

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

QSpec requires `mode` and `member` as members of `operation`, each nullable. The
reader decodes each as optional, so an omitted `mode` or `member` is read as
absent, as `null` is. The criteria below are stated for `null`, and the omission
is a leniency the reader has and this specification does not endorse; it is an
existing defect, tracked as IR-539, and is not changed here. The other
deviations named in this section are stated deviations, not scheduled defects.

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

### Frame bodies

A `state` node of `semantic_form` `frame` carries a frame body, and only a
frame body: `{"term": "frame", "modifies": [...], "creates": [...],
"deletes": [...]}`, with all three members required and each a
`uniqueItems` array per the wire schema: `creates` and `deletes` of node
keys, and `modifies` of the `FrameModifiesEntry` values
[FR-040](./FR-040-admit-frame-entries-and-state-clauses.md) defines. The reader shall validate a
`state`/`frame` node's `body` against this shape alone, a
`correspondence`/`abstraction_relation` node's `body` against the shape of
[FR-346](./FR-346-admit-abstraction-relation-body.md) alone, and every other
node's `body` against the semantic-term grammar alone. A frame body that omits one of
the three members, carries any further member, or does not carry this closed
shape refuses as `invalid_semantic_graph` at the frame body; so does a node of
any other tag or form whose body carries the frame shape. Within one member
array, an entry that repeats another entry of the same array, or that is not a
node key, refuses as `invalid_semantic_graph` at that member's own path
(`semantic_graph.nodes.body.modifies`, `.creates` or `.deletes`); an entry
naming a node key outside the reader's node-identity domain refuses as
`digest_domain_mismatch` at that same member path. Three empty members are
admitted.

The frame-body-semantics stage charges no work of its own: every entry it
walks was already parsed, shape-validated and charged once by the per-node
body-validation loop that ran before it, so its cost is already bounded by
the same work limit that bounded that earlier walk. A caller cannot grow
this stage's cost independently of an already-charged input.

The declaration stage ends with two name checks. A node carrying both a
`declaration` and a nominal identity preimage that fixes a name (an enum
declaration, a dimension or a declared unit) shall declare exactly that
preimage's `qualified_declaration`, else the reader refuses as
`invalid_package` with cause `declaration-nominal-mismatch`. No two nodes
shall declare the same `qualified_name`, else the reader refuses as
`ambiguous_declaration` with cause `ambiguous-name`, located at the
lowest-digest node sharing the name; the refusal names that one node, not
every holder. QSpec FR-322 orders both before any frame
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
ascending entry digest breaks the tie; the selected defect therefore never
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
| FR-038-AC-2 | The reader refuses malformed, duplicate-member, unknown-member, noncanonical, stale-dependency, cross-domain digest, unknown required capability, unsupported node tag, invalid graph and invalid source-map inputs with exactly those codes, and every authored adverse structural mutation returns its declared outcome, before exposing a package; a domain package selection or model owner carrying `authority`, `revision`, `export` or `version` refuses as `unknown_member`, a domain package selection outside `sha256-jcs` as `digest_domain_mismatch`, one with no supplied document as `missing_import` with cause `missing-selection`, and a `model_export` semantic form as `invalid_semantic_graph`. | Test (TC-048) |
| FR-038-AC-3 | Exact byte, node, edge, occurrence, diagnostic and work limits admit a package; each one-over limit returns `incomplete` with that limit kind, the limit and the consumed counter and no package; no limit kind is a depth (FR-038-AC-117). | Test (TC-048) |
| FR-038-AC-4 | The recomputed package id equals each positive fixture's declared id; editing a source-map region, occurrence, raw source digest or capability disposition leaves it unchanged, while editing the edition, a selection, a required feature or a node projection changes it and refuses unless mirrored. | Test (TC-048) |
| FR-038-AC-5 | Every nominal preimage of the in-repo nominal fixture re-derives the node key it is keyed by and round-trips its own wire form; every authored invalid nominal mutation, an absent or wrong preimage, and each retained-preimage change of enum case, `semantic_type`, dependency or unit target refuses as `invalid_semantic_graph`; a model owner admits when its identity names a selected domain package and refuses as `invalid_semantic_graph` when it names none or carries an empty node. | Test (TC-048) |
| FR-038-AC-6 | Every admitted node family lowers independently with exact source, type, dependency, bound and claim correspondence; missing, unsupported, unbounded and over-work requests return `invalid_input`, `unsupported`, `requires_bound` and `failed` without a node and without changing sibling records, except that a package over the byte ceiling fails every requested record (FR-038-AC-95). | Test (TC-050) |
| FR-038-AC-9 | The shipped default read-limit policy is exactly those six finite values, every member is strictly positive and finite, and each meter is enforced at its own true measured boundary against a real package: the package's exact measured consumption for that meter admits it, and one below that exact value refuses it as `incomplete`, naming that meter and reporting the true consumption. The shipped default is far larger than any fixture, so this boundary is proven against each meter's real measured cost rather than against the default value itself; no fixture approaches that scale, and none is fabricated to do so. | Test (TC-048) |
| FR-038-AC-7 | Lowering every node of every positive fixture under a profile supporting every tag yields no `invalid_body` and no `body_incomplete` record, and the seven-member record vocabulary is exhaustive: no eighth kind is reachable and each of the seven is named. | Test (TC-052) |
| FR-038-AC-8 | A closure holding both an out-of-profile tag and an unbounded type returns `unsupported`; a zero work limit returns `failed` for an absent key rather than `invalid_input`; each named offending key is the least in ascending order rather than the first visited; each of the eight unbounded forms raises `requires_bound` and each other declared form of those two families, carrying no `recursion_group`, does not; and a lowered record's `dependencies` contains every key in its `bounds` and `claims`. | Test (TC-052) |
| FR-038-AC-10 | A `lock.model_selections` entry that repeats an earlier entry's identity, digest domain and digest verbatim, mirrored identically into `identity_preimage.model_selections`, refuses as `malformed_wire` at `lock.model_selections`; the same lock-side repeat left unmirrored in the identity preimage refuses earlier, as `stale_dependency` at `lock`, because the preimage/lock equality check runs first; two entries sharing an identity but differing in digest are not a repeat and are owned by FR-038-AC-20 (`stale_dependency`, never `malformed_wire`). | Test (TC-048) |
| FR-038-AC-11 | A `lock.model_selections` array carrying both a repeated entry and an entry whose digest no document the package evidence supplies satisfies refuses as `malformed_wire` at `lock.model_selections`, never as `stale_dependency`, regardless of whether the repeated entry or the stale entry appears first in the array — the uniqueness check runs over the whole array before any entry's digest is checked against the supplied documents, so the outcome does not depend on array position. | Test (TC-048) |
| FR-038-AC-12 | A `creates` or `deletes` entry's eligibility is exactly `model`/`object_type` and `model`/`process`, and this eligibility predicate is checked against every `(member, tag, form)` triple the closed node taxonomy can produce, not sampled; the `modifies` entry shape and its eligibility are [FR-040](./FR-040-admit-frame-entries-and-state-clauses.md)'s. The refusal an ineligible triple drives — `invalid_model_binding` with cause `malformed-declaration`, located at the offending entry — is verified end-to-end through the reader for a representative ineligible triple in each of `creates` and `deletes` and every node family; the reader's mapping from an ineligible predicate result to that refusal depends only on which member the entry sits in, never on its tag or form, so this sample together with the exhaustive predicate check covers the full triple space without driving every one of it through the reader. Each of the two eligible triples the published all-families fixture's own frame body does not already exercise (`process` in `creates`, `object_type` in `deletes`) is independently shown admitted. | Test (TC-053) |
| FR-038-AC-13 | An entry naming a digest that is not among the frame node's own `dependencies` refuses as `missing_declaration` with cause `missing-name`, located at that entry, whether the digest resolves to no node the frame declared as a dependency (a real node elsewhere in the graph) or to no node anywhere in the graph at all — both conditions are the same refusal, never `invalid_model_binding` and never the graph's generic unresolved-reference `invalid_semantic_graph`. | Test (TC-053) |
| FR-038-AC-14 | A frame node carrying both a meaning-join defect and a canonical-order defect refuses for the meaning-join defect; among two meaning-join defects in different members, the earlier member (`modifies` before `creates` before `deletes`) is selected regardless of which defect's entry digest is lower; among two meaning-join defects in the same member, the lower entry digest is selected; a member array not in strictly ascending digest order, with no meaning-join defect present, refuses as `invalid_semantic_graph` at the frame body path, located at the frame node itself and carrying no cause. | Test (TC-053) |
| FR-038-AC-15 | A package carrying two defective `state`/`frame` nodes refuses at the one with the lower `node_id` digest, reporting only that frame's own defect, even when the other frame's defect would otherwise outrank it under FR-038-AC-14's precedence — the visit order is ascending node-id digest across frames, and the reader reports the first defective frame it reaches rather than comparing every frame's defect. | Test (TC-053) |
| FR-038-AC-17 | Each declared wire member the V2 contract carries is read and enters the identity projection: a declaring node's `declaration.qualified_name`, a `literal` term's `type`, and an `application` term's `operation` and `result_type`. Deleting any one of them from a single node of an otherwise unmodified `positive-operation-identities` package refuses as `invalid_semantic_graph`, whether or not the deletion is mirrored into `identity_preimage.identity_projection`: a missing `declaration` refuses at `semantic_graph.nodes.declaration` and a missing `literal.type`, `application.operation` or `application.result_type` refuses at `semantic_graph.nodes.body`, because each check applies to the graph node's own closed member set unconditionally, before the projection comparison is reached — mirroring the deletion into the preimage changes nothing, since the graph node's own defect refuses first either way; and the fifteen `expression` forms the contract declares are each admitted as a node form while a sixteenth `expression` form refuses as `invalid_semantic_graph`; the admitted `model` forms are [FR-040](./FR-040-admit-frame-entries-and-state-clauses.md)'s. | Test (TC-048) |
| FR-038-AC-18 | A self-typed node's own body-root `literal.type` — the literal that is the node's body — naming itself is exempt from the reference-cycle check and admits with no declared `recursion_group`. The same `literal.type` self-reference nested one level deeper, inside that node's own `aggregate` member, `binding` value or `application` argument, is not exempt, and refuses as `invalid_semantic_graph` at `semantic_graph.nodes.recursion_group` for want of a declared `recursion_group`, exactly like a self-typed node's `reference` body or `application.result_type` naming itself. | Test (TC-048) |
| FR-038-AC-19 | A `lock.model_selections` array carrying two entries of different defect classes refuses for the earlier class under the stated total order, at `lock.model_selections`, regardless of which of the two entries appears first in the array: an entry outside `sha256-jcs` beside an entry of empty `identity` refuses as `digest_domain_mismatch`, and an entry of empty `identity` beside an entry whose digest no supplied document satisfies refuses as `malformed_wire`. Both orderings of each pairing are pinned and refuse with the same code, so a refusal decided by array position rather than by defect class fails this criterion rather than passing it as "some refusal occurred". | Test (TC-048) |
| FR-038-AC-20 | A selection binds by `identity`, so a `lock.model_selections` array holds at most one entry per identity: two entries naming the same `identity` and differing in `digest` refuse as `stale_dependency` at the later entry's `digest` (`/lock/model_selections/<i>/digest`), whichever of the two entries appears first in the array, even when both entries are independently well-formed and each names a document the package evidence supplies, and never as `malformed_wire`; the pair is refused before any document is read, so it refuses the same way, in both orders, when an entry of the pair or an earlier row has no supplied document, a forged one or one naming another identity, and never as that document's own `missing_import`, `byte-digest-mismatch` or `invalid_model_binding` refusal; an array whose two entries name different identities still admits (each other check passing). An entry carries no `version`, so the identity is the whole locator (FR-038-AC-62). A same-identity, different-digest pair is decided by the total order above, so it refuses `digest_domain_mismatch` when one of its entries is outside `sha256-jcs`, whichever of the two carries that second defect. | Test (TC-048) |
| FR-038-AC-21 | A node whose declared `qualified_name` differs from its nominal preimage's `qualified_declaration` refuses as `invalid_package` with cause `declaration-nominal-mismatch` at that node; two nodes declaring one name refuse as `ambiguous_declaration` with cause `ambiguous-name` at the lower-digest of the two; a package carrying both defects refuses for the nominal mismatch; and a package also carrying a frame or an operation defect still refuses for its declaration defect. | Test (TC-048) |
| FR-038-AC-22 | A QSL-shaped package holding a function whose body applies an operation to its parameters, each a `value`/`parameter` node, and a `scalar_type`/`compound_unit` node, admits and lowers; a parameter with a negative, non-canonical or missing level, an empty, non-identifier or wrongly typed name, or an extra binding refuses as `invalid_semantic_graph` at the node's `body`; a parameter with a dependency, itself as its type or a `declaration`, and a compound unit with a zero exponent, a term naming a non-unit, a repeated unit, a dependency list other than its unit keys or a type other than itself, each refuses as `invalid_semantic_graph` at the member it breaks; the empty compound unit admits. | Test (TC-048) |
| FR-038-AC-23 | An application node whose `dependencies` omits a body reference target, lists them out of digest order, repeats one, or adds its `result_type` refuses as `invalid_semantic_graph` at `semantic_graph.nodes.dependencies`, located at that node. | Test (TC-048) |
| FR-038-AC-24 | Every refusal about a value carries the RFC 6901 pointer of that value, built from the reader's own position: member names escaped (`~` as `~0`, `/` as `~1`), array elements by index, resolving in the document read — an unknown member (including one whose name holds `~` or `/`) at that member, a repeated member at that member, a missing member at the object lacking it, a wrongly typed value at that value, a stale mirror at the first differing value, and each graph, lock, source-map, capability and diagnostic refusal at the member its failed check read; malformed JSON and non-canonical bytes carry no pointer. No refusal code changes. | Test (TC-048) |
| FR-038-AC-25 | An `unknown_contract_version` refusal points at `/contract_version` and carries the exact `contract_version` string the reader read, including an empty one; no other refusal carries a version. | Test (TC-048) |
| FR-038-AC-26 | Each one-over limit other than the byte limit returns `incomplete` carrying the RFC 6901 pointer of the value whose charge failed, which resolves in the document read: nodes at the first node past it, edges at the dependency and occurrences at the source-map entry or region that took the count past it, diagnostics at the first entry past it, and work at the value whose validation took the meter past it; the byte limit carries none. | Test (TC-048) |
| FR-038-AC-27 | A domain package selection is admitted only by the document the evidence supplies under its digest: a document supplied under no digest of the row refuses `missing_import`/`missing-selection` at the row's `digest`; a document holding a number whose text denotes a magnitude past 2^53 refuses `noncanonical_wire` at the `digest` before any digest is computed (FR-038-AC-93); a document whose RFC 8785 SHA-256 is not the digest it was supplied under refuses `stale_dependency`/`byte-digest-mismatch` at the `digest`; a document naming another identity refuses `invalid_model_binding`/`wrong-model-selection` at the row's `identity`; and a matching document admits. The document's `package.version` is neither required nor read: a matching document with no `package.version`, or a non-string one, admits, and the same package supplied as two documents at different versions admits when each is selected in its own package under its own digest (FR-038-AC-64). | Test (TC-048) |
| FR-038-AC-28 | A domain package document's declarations refuse at the row, in FR-154's order: a node whose object id is invalid, whose `kind` names no construct, or that shares its identity, refuses for itself and any reference to it reports that refusal, never `missing_declaration`/`missing-name`, wherever the two sort; a node failing two rows reports the earlier (a dangling `typeRef` before a multiplicity with `lower > upper`, a malformed member before both); a `typeRef` naming a relationship refuses `invalid_model_binding`/`malformed-declaration` wherever its declaring node sorts; two nodes with no identity refuse `malformed-declaration`, never `conflicting-binding`. | Test (TC-048) |
| FR-038-AC-29 | A package whose lock selects a domain package document with declared types, and whose graph holds a `dispatch_call` on one of its operations, admits; the same call naming an operation the document does not declare refuses `ill_typed`/`operator-ineligible` at the member's `name`; an inherited field resolves on a subtype and a subtype conforms to its supertype in either operand order; a field typed at an `Int[lo, hi]` value type has the integer-range member type `[lo, hi]`. | Test (TC-048) |
| FR-038-AC-30 | Reading a selected domain package and resolving a model-owned member are charged to the `work` limit: a limit one below the work a read used returns `incomplete` for `work` with the pointer `/lock/model_selections/<i>` of the row, and the exact work admits. | Test (TC-048) |
| FR-038-AC-31 | A package whose lock and identity preimage carry the same `dependency_selections` of two `DependencySelection` entries `{identity, package_id}`, one per identity in ascending identity order, admits when each selected dependency's admitted package is supplied, and both members read back as the supplied entries; changing one entry's `package_id` changes the package's `package_id`. | Test (TC-048) |
| FR-038-AC-32 | A `dependency_selections` entry whose `package_id.domain` is another digest domain refuses `digest_domain_mismatch` at that `domain`; one with an empty `identity`, a short `digest` or a bare-digest `package_id` refuses `malformed_wire` at that member; one that lacks `package_id` or `identity` while carrying a `Selection` or `DefinitionRef` member refuses `malformed_wire` at the entry. | Test (TC-048) |
| FR-038-AC-33 | A `dependency_selections` entry repeating an earlier entry's `identity`, adjacent or not, refuses `invalid_package`/`conflicting-definition` at the repeating entry; an entry not strictly after its predecessor in UTF-8 byte order refuses `invalid_package`/`invalid-value` at that entry; entries in UTF-8 byte order where UTF-16 code-unit order differs admit. | Test (TC-048) |
| FR-038-AC-35 | A package whose `function` call has a `dependency_reference` callee to a declared function of the supplied dependency admits, carries the term verbatim in the node body and lists no `dependencies` entry for it; listing the target refuses `invalid_semantic_graph` at the node's `dependencies`; changing only the term's `package`, or only its `node`, without re-deriving the key refuses `invalid_package`/`stale-node-key`, and re-derived they give distinct node ids. | Test (TC-048) |
| FR-038-AC-36 | A `dependency_reference` with a bare-digest, other-domain or short-digest `package` refuses `invalid_semantic_graph` at the `package`, one in another node domain at the `node`, and one with a missing or extra member at the term; a well-formed term that is not argument 0 of a `quire.op.function.call` (a second argument, an argument of another operation, an aggregate member or a node body root) refuses `ill_typed`/`operator-ineligible` at the term. | Test (TC-048) |
| FR-038-AC-37 | A `dependency_selections` entry with no package supplied for its `identity` refuses `missing_import`/`missing-selection` at the entry, and one whose package has another `package_id` `stale_dependency`/`byte-digest-mismatch` at its `package_id.digest`; a term whose `package` no entry names refuses `missing_declaration`/`missing-selection` at the `package`, a `node` naming no node or a node without a `declaration` `missing_declaration`/`missing-name` at the `node`, and a `node` naming a declared node that is no function `ill_typed`/`operator-ineligible`, each carrying the calling node as its locus. | Test (TC-048) |
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
| FR-038-AC-64 | Selections bind by identity and content digest alone: a `model_selections` row admits when the document supplied under its `digest` names the row's `identity`, so two documents of one package at different versions, each selected in its own package under its own digest, both admit and yield the same model-owned node keys; a model owner joins the row whose `identity` equals its own, and an owner naming another identity refuses as `invalid_semantic_graph`; and a `dependency_selections` entry admits when the package supplied under its `identity` has the entry's `package_id`, with no version supplied or compared, and refuses `missing_import`/`missing-selection` for a package supplied under another identity and `stale_dependency`/`byte-digest-mismatch` for another `package_id`. | Test (TC-048) |
| FR-038-AC-65 | Every application operator class, operation member kind and constraint kind the operation catalog declares decodes to a member of its closed vocabulary, so the production catalog, whose `law_roles` entries carry no `revision`, reads once FR-038-AC-46 through FR-038-AC-61 are also implemented; each of `case`, `temporal_formula`, `temporal_fairness`, `temporal_interval`, `fairness` and `union_arms` converts wire string to enum member and back to the same string; catalog bytes that name an operator class, member kind or constraint kind outside its vocabulary return the typed error of the catalog read of FR-038-AC-58, naming the word, instead of a catalog; and a package application whose `operator` is outside the closed operator vocabulary refuses `invalid_semantic_graph` at that term. | Test (TC-048) |
| FR-038-AC-67 | An unknown identity refuses `unknown-operation` and an `operator` that differs from the catalogued class refuses `operation-class-mismatch`, each ahead of every other check of the entry at the operation step, so `quire.op.control.case` under the operator `unary` refuses `operation-class-mismatch`; and of two defective body-root nodes the one with the lower `node_id` digest is reported when both defects are operation refusals. A placement defect of the temporal step (FR-038-AC-102) is reported ahead of every operation refusal, whatever the digest order. (Amended by IR-549: the earlier text ordered both refusals ahead of blanket temporal/case `unsupported_construct`, which IR-549 removed, and put the placement of these applications inside the operation step.) | Test (TC-048) |
| FR-038-AC-68 | At the operation check of a `quire.op.temporal.clause` application, observed on that node as a unit-level check of the node's operation step and, since IR-549, also as an admitted package whose formula node is a `temporal_formula` application (FR-038-AC-96), an application with the one selected `temporal_profile` law, no member and six arguments (a `reference` to a `value`/`parameter` node of any type, a `text` literal, three `aggregate` terms and a `reference` to a `temporal`/`formula` node) passes; five or seven arguments refuse `ill_typed`/`operator-ineligible` at `arguments`; a first argument that is not a `reference` term and a sixth that references a Boolean node each refuse the same way at that argument; a `reference` to a `temporal`/`formula` node fits the sixth operand and an `any_term` position and no `boolean` operand position; and a member of kind `profile_operator` or any other kind refuses `invalid_package`/`operation-member-mismatch` at `operation.member`. | Test (TC-048) |
| FR-038-AC-69 | A member of kind `temporal_interval` or `fairness` on an application whose catalogued entry has none of the operator classes `case`, `temporal_formula` and `temporal_fairness` and a member of another kind or none, such as `quire.op.boolean.not` or `quire.op.temporal.clause`, refuses `invalid_package`/`operation-member-mismatch` at the member, and, since IR-549, so does a `fairness` member on a `temporal_formula` identity such as `quire.op.temporal.holds`, which the earlier text refused `unsupported_construct` at `operator`; the operator refusal is gone and every member that disagrees with its entry refuses alike. | Test (TC-048) |
| FR-038-AC-70 | `structural.eq` over `record Node { label: Text[0, 8; nfc]; next?: Node; }` admits with the leaf `["field:label"]` carrying one catalogued `text_profile` law the lock selects and the mode `{kind: text_profile, value: nfc}` followed by the recursion leaf `["field:next", "inner", "recursion:0"]` with no laws and no mode, and the same comparison with the text leaf alone refuses `invalid_package`/`operation-law-missing` at `operation.leaves`, as does the recursion leaf alone; over `Option<Node>` it admits the leaves `["inner", "field:label"]` and `["inner", "field:next", "inner", "recursion:1"]`; over a mutually recursive pair `A { name: Text[0, 8; binary-utf8]; b?: B }` and `B { tag: Text[0, 4; nfc]; a?: A }` in one package, compared at `A` and compared at `B`, each leaf with the mode its own type pins, it admits `["field:name"]`, `["field:b", "inner", "field:tag"]` and the recursion leaf `["field:b", "inner", "field:a", "inner", "recursion:0"]` at `A`, and `["field:tag"]`, `["field:a", "inner", "field:name"]` and the recursion leaf `["field:a", "inner", "field:b", "inner", "recursion:0"]` at `B`; over `Two { x: Node; y: Node }` it admits exactly the text leaves `["field:x", "field:label"]` and `["field:y", "field:label"]` each followed by its own recursion leaf, `["field:x", "field:next", "inner", "recursion:1"]` and `["field:y", "field:next", "inner", "recursion:1"]`, and refuses the list lacking either recursion leaf `operation-law-missing`; over `Tree2 { label: Text[0, 8; binary-utf8]; kids: Sequence<Tree2>[0, 3]; }` it admits `["field:label"]` and the recursion leaf `["field:kids", "inner", "recursion:0"]`; over a declared tuple `Pair` of a `Text[0, 8; nfc]` and an `Option<Pair>` it admits `["position:0"]` and the recursion leaf `["position:1", "inner", "recursion:0"]`; over `X { t: Text[0, 8; nfc]; n?: Y }`, `Y { u: Text[0, 8; nfc]; x?: X }` and `Wrap { y: Y; x: X }` compared at `Wrap` it admits exactly the four text leaves `["field:y", "field:u"]`, `["field:y", "field:x", "inner", "field:t"]`, `["field:x", "field:t"]` and `["field:x", "field:n", "inner", "field:u"]` with the two recursion leaves its reentries derive in their places, `["field:y", "field:x", "inner", "field:n", "inner", "recursion:1"]` after the second and `["field:x", "field:n", "inner", "field:x", "inner", "recursion:1"]` after the fourth, so a count memoised for `Y` under `field:y` is not reused under `field:x`, where `X` is open, and a list of the four text leaves alone refuses `operation-law-missing` at `operation.leaves` while a list of all six and one further text leaf refuses `operation-law-mismatch` at the seventh entry; over a record `R { a?: Text[0, 8; nfc]; b?: Text[0, 8; nfc] }`, whose two optional fields name one option node, it admits `["field:a", "inner"]` and `["field:b", "inner"]`; over a recursive record that reaches no `text` type, such as a `List` of integers, `structural.eq` and `collection.contains` admit with `leaves` empty; and the `Node` comparison with a second text leaf supplied after `["field:label"]` and the recursion leaf refuses `operation-law-mismatch` at that extra entry, `operation.leaves/2`, so a cyclic type is no longer refused `ill_typed`/`operator-ineligible` at `operation.leaves` and no longer admits unchecked leaves. | Test (TC-048) |
| FR-038-AC-71 | Over `Node`, the recursion leaf placed before the text leaf, a recursion leaf at `["field:next", "recursion:0"]`, one `["field:next", "inner", "recursion:1"]` with the wrong `d`, and a second recursion leaf after the first each refuse `invalid_package`/`operation-law-mismatch` at that leaf's `path`, one that carries a law refuses the same way at its `laws`, and one that carries a mode refuses `operation-mode-mismatch` at its `mode`; a recursion leaf at a reentry of an integer `List`, which reaches no `text` type, refuses `operation-law-mismatch` at that leaf's `path`; a text leaf inside a recursive record whose type binds no `text_profile` still refuses `ill_typed`/`operator-ineligible` at `operation.leaves`, with the leaves supplied or not; `T` = `Option<T>`, a `Sequence` of itself, and a record that holds a field of such a type each refuse `ill_typed`/`operator-ineligible` at `operation.leaves` under a work limit of 1000, so the refusal is not `incomplete` for `work`, whereas `R { x: Option<R> }` admits; and a leaf law the lock does not select inside a recursive record refuses `operation-law-unselected` at its `definition`. | Test (TC-048) |
| FR-038-AC-72 | The leaf count and derivation over a cyclic compared type are iterative and decided by the work budget: a cycle of 20000 record nodes, each holding an integer field and naming the next, the last naming the first, with the last also holding one `text` field, admits with its one 20000-segment leaf and its recursion leaf under byte, node, edge and work limits raised to admit it (the default byte limit of 1 MiB is below the package's size), on a thread whose stack is 256 KiB; ten records `R0` to `R9` that each hold a text field and an optional field naming every other record, compared at `R0` with `leaves` empty, return `incomplete` for `work` at `operation.leaves` under the default read limits, rather than a listing of their leaves or any other refusal; and for a ring of 12 records, each holding one text field and an optional field naming the next, compared at the first with its 12 text leaves and its one recursion leaf (the last record's field reenters the first), a work limit one below the work a read used returns `incomplete` for `work` and the exact work admits. | Test (TC-048) |
| FR-038-AC-73 | Under a bounds-required profile a position typed at a quantity returns `requires_bound` naming the `unit` or `compound_unit` node: a record whose field is typed at a `unit` node, a parameter typed at a `compound_unit` node, `Sequence<Quantity>[0,3]` whose element type is a `unit` node, and a record whose field is typed at a reachable `bounded_domain` over a `unit` node, which names the `unit` node rather than the domain, each return it; while a `unit` declaration lowered as the requested node, a `compound_unit` node lowered as the requested node with its unit nodes in `dependencies`, a record with no quantity position whose closure reaches a unit only as a `literal.type` annotation, and an application whose `result_type` is a quantity over parameters typed at a bounded type each lower. | Test (TC-050) |
| FR-038-AC-74 | For every in-repo positive fixture, the bytes `quire_canonical::to_vec` returns for its `CheckedPackageIdentityPreimageV2` equal the bytes of `serde_json::to_vec(&serde_json::to_value(&preimage))`, also for a preimage whose projection bodies hold non-ASCII and astral strings and the integers 9007199254740992 and -9007199254740992, and `quire_canonical::sha256` over it equals the fixture's `package_id.digest`; a fixture whose preimage differs in one member from the one its `package_id` was computed over refuses `stale_dependency` at `/package_id/digest`. | Test (TC-048) |
| FR-038-AC-75 | For every in-repo positive fixture, the bytes `quire_canonical::to_vec` returns for its `CheckedSemanticGraphV2` equal the bytes of `serde_json::to_vec(&serde_json::to_value(&graph))`; and for a graph whose nodes carry a nominal identity preimage of each of the four versions, a `declaration`, a `recursion_group` and bodies of object, array, string, integer, boolean and null values, with strings holding non-ASCII and astral characters (for instance U+00E9 and U+1F600), member names ordered the same by UTF-8 bytes and by UTF-16 code units, and the integers 9007199254740992, -9007199254740992, 0 and -1, the bytes are equal as well. | Test (TC-048) |
| FR-038-AC-76 | For every in-repo positive fixture, the bytes `quire_canonical::to_vec` returns for each of `CheckedPackageLockV2`, `CheckedSourceMapEntry`, `CheckedCapability`, `CheckedSemanticId` and `CheckedDiagnosticsV2` equal the bytes of `serde_json::to_vec(&serde_json::to_value(&value))`, one assertion per type; the diagnostics value carries entries whose `details` hold nested objects and arrays and whose `loci` are non-empty. | Test (TC-048) |
| FR-038-AC-77 | A `CheckedSemanticGraphV2` holding one node whose `body` is a `Value` nested 100000 levels deep, a `CheckedPackageIdentityPreimageV2` whose one projection holds the same body, and a `CheckedDiagnosticsV2` whose one entry holds it in `details`, each encode on a thread whose stack is 256 KiB, return the bytes of the expected text (the nesting written out by repetition, not by `serde_json`) and complete without a stack overflow. | Test (TC-048) |
| FR-038-AC-78 | The encodes of FR-038-AC-77, run under a byte ceiling of the encoded text's exact length, return the bytes; run under a ceiling one byte lower, return the byte-limit error with no bytes; a body nested 20000 levels deep encodes and is not refused for its depth; and a body holding the integer 9007199254740993 returns the encoder's refusal naming that value with no bytes. | Test (TC-048) |
| FR-038-AC-79 | A package document holding the integer 9007199254740993 in a node body refuses `noncanonical_wire` with no pointer, as does one holding -9007199254740993, one holding the float `2.0` in a body, and one whose body object lists a member named with U+E000 before one named with U+10000 (UTF-8 byte order), each refused before any grammar, `package_id` or graph refusal the same document also earns; the same document with 9007199254740992, with `2` in place of `2.0`, and with U+10000 before U+E000 (UTF-16 code-unit order) is not refused `noncanonical_wire`. | Test (TC-048) |
| FR-038-AC-80 | `quire-canonical` is a `branch = "main"` git dependency of this repository's manifests with its source in the `allow-git` list of `deny.toml`, and `make deny` passes; `CheckedSemanticId`, `CheckedSourceMapEntry`, `CheckedCapability` and `CheckedPackageLockV2`, and every type the three `Encode` types and these four hold that does not itself hold a `Value` (`CheckedOccurrence`, `CheckedDeclaration`, `NominalIdentityPreimage`, its four preimage structs, `NominalOwner`, `DimensionTerm`, `CheckedRational`, `CheckedDiagnosticStage`, `CheckedDiagnosticCode`, `CheckedDiagnosticCause`, the node identity, selection, domain-package, dependency-selection, source-region types, `CheckedArtifactRef` and `CheckedSourceRef`), derive `FixedShape`; `CheckedPackageIdentityPreimageV2`, `CheckedSemanticGraphV2` and `CheckedDiagnosticsV2` implement `quire_canonical::Encode`, and they and the types that hold a `Value` do not implement `FixedShape`; the crate's source holds no hand-written `impl FixedShape`, no `const DEPTH` and no wrapper type around a `quire-canonical` type; and no copy of `quire-canonical` source or of its published vectors is in the repository, each checked by a test that reads the manifests, `deny.toml` and crate source. | Test (TC-048) |
| FR-038-AC-81 | Observed at the operation check of a body-root application, an application whose `operation.identity` the operation catalog does not list refuses `invalid_package` with cause `unknown-operation` at `operation.identity`; `quire.op.integer.add`, a `binary` operation, under the operator `unary` refuses `invalid_package`/`operation-class-mismatch` at `operator`; and `quire.op.integer.add` under `binary` with two operands, no law, no mode and no member admits, so neither refusal is a refusal of every application. Their order against every other check of the entry is FR-038-AC-67's. An application whose `operation` is a bare string, not an object of the closed operation shape, refuses `invalid_semantic_graph` at that application's `operation`. | Test (TC-048) |
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
| FR-038-AC-93 | A selected model document holding the number `9007199254740993`, `-9007199254740993`, `9.007199254740993e15`, `1e20` or `18446744073709551617` at `/package/count` refuses `noncanonical_wire` at `/lock/model_selections/0/digest` with `document_pointer` equal to `/package/count`, whether the row selects the document's own digest or another digest, so the refusal precedes `byte-digest-mismatch`; a document holding such numbers at `/b` and then `/a/0` names `/b`, the first in document order (except that a number with no finite double is named when the reader reaches it, FR-038-AC-110); the same document holding `9007199254740992`, `-9007199254740992`, `9.007199254740992e15` or `0.5` at that pointer is not refused for it and is digested; an integer value type whose upper bound is `9007199254740993` in a document that is otherwise admitted refuses the same way, where today it is admitted (the reader change); and a package document refused `noncanonical_wire` for its own bytes (FR-038-AC-79) carries no pointer and no `document_pointer`. | Test (TC-048) |
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
| FR-038-AC-109 | A selected model document holding at `/package/ratio` the number `0.1000000000000000000001`, `9007199254740993.5`, `-0.1000000000000000000001`, `4.9e-324` or `1e-400` refuses `noncanonical_wire` at `/lock/model_selections/0/digest` with `document_pointer` equal to `/package/ratio` and cause `inexact-number` (quire-specification:FR-272), whether the row selects the document's own digest or another digest, so the refusal precedes `byte-digest-mismatch`; the same document holding `0.1`, `0.5`, `1.5`, `-0.25`, `5e-324`, `2.5e-10`, `1.0`, `-0` or `1e2` at that pointer (`1.0`, `-0` and `1e2` are spelled other than the text `quire-canonical` writes, `1`, `0` and `100`, and each is admitted because the value is the same), each a number whose exact decimal value equals that of its double's shortest round-trip text, is not refused for it and is digested; the text that decides is the one `quire-canonical` writes for the number, which where two shortest texts are equally close to the double is the one with the even last digit, so a document holding `1125899906842624.2`, `1500000000000000.2` or `2.9802322387695312e-8` at that pointer (each the even-digit text of a tie) is not refused and one holding `1125899906842624.3`, `1500000000000000.3` or `2.9802322387695313e-8` refuses `inexact-number`, and the two texts are compared by their digits and scale, never as doubles; two documents that differ only in `0.1` and `0.1000000000000000000001` at that pointer are not both admitted, the second refusing as above where today both digest alike; the decision is made on the number's text read by `quire-canonical`, through no `serde_json` value, so it is the same whatever features other crates in the build turn on. | Test (TC-048) |
| FR-038-AC-110 | The cause of each refusal FR-038-AC-93 names is `inexact-integer`, with the `document_pointer` AC-93 gives, for `9007199254740993`, `-9007199254740993`, `9.007199254740993e15`, `1e20`, `18446744073709551617`, `9007199254740993.0`, `1e400` and `-1e400`, and for an integer value type whose upper bound is `9007199254740993`; `9007199254740993.0` is `inexact-integer` and never `inexact-number` although its text also differs from its double's shortest round-trip text, and `1e400` and `-1e400` refuse `noncanonical_wire` at that `digest` with the `document_pointer` of the number; `9007199254740992`, `-9007199254740992` and `9.007199254740992e15` are unchanged and admitted, while `9007199254740992.5` (whose nearest double is `9007199254740992`) refuses `inexact-number`; a document holding an inexact number at `/b` and then a whole number past 2^53 at `/a/0` names `/b` with `inexact-number`, the first in document order, and the reverse order names the whole number with `inexact-integer`, except that a number with no finite double is named when the reader reaches it, and so can be named ahead of an earlier inexact number; when faults coexist the first fault `quire_canonical::read` returns decides (QSL FR-056), so `{"b":0.1000000000000000000001,"a":[1e400]}` and `{"b":9007199254740993,"a":[1e400]}` each refuse `noncanonical_wire`/`inexact-integer` with `document_pointer` `/a/0`, `{"a":1,"a":2,"n":1e400}` names `/n`, `{"n":1e400,"a":1,"a":2}` names `/n` and `[1e400` names `/0`, while `[1e400,"<0xFF>"]` (invalid UTF-8, which the read checks over the whole input first) refuses with no `document_pointer`, and, as this reader's reading, `[{"a":1,"a":2},1e400]`, `{"a":1,"a":2,"n":1e-400}` and `[1e400,"<0xFF>"]` offered under a digest that is not their raw digest refuse `stale_dependency`/`byte-digest-mismatch` at the row's `digest` because a reader fault other than an out-of-range number comes first; `1e-400` and `-1e-400` refuse `inexact-number`. | Test (TC-048) |
| FR-038-AC-111 | A package document holding the number `0.1000000000000000000001` or `9007199254740993.5` in a node body refuses `noncanonical_wire` with no pointer, no `document_pointer` and no cause, as it does today, each refused before any grammar, `package_id` or graph refusal the same document also earns; the same document holding `0.1` in that place is not refused `noncanonical_wire`; the package document is parsed through `serde_json`, so the manifest of the crate that holds the reader declares `serde_json` with the feature `float_roundtrip`, which makes the parse of a shortest round-trip text exact, so that a package document holding `1.2793061557049685`, `1.2106592671318679` or `1.3567384036451073` in a node body (each its double's shortest round-trip text, which `serde_json` without that feature reads as a neighbouring double and so would re-encode to different bytes) is not refused `noncanonical_wire`, whatever other crates in the build turn on. | Test (TC-048) |
| FR-038-AC-112 | `make conformance-qspec` reads `proposals/checked-package-v2/fixtures/adverse.json` from the checkout named by `QUIRE_SPECIFICATION_DIR` (never copied into this repository) and applies every mutation of its `structural_mutations` and `body_grammar_mutations` lists, each at its `pointer` with its `replacement`, to a fresh copy of the `positive-all-families.json` package, reading the result with the production reader; each mutation refuses with exactly the code its `outcome` names (`refused:<code>`, and the cause after a `/` where it gives one). The harness refreshes no identity after a mutation: `node_id`, the references to it, `identity_preimage.identity_projection` and `package_id` stay as the fixture holds them, as QSpec's TC-427 BG-02 applies the same mutations, because merged FR-322 has the reader validate the wire, body grammar included, ahead of every identity check, so a mutation under test is the first check that can refuse; a mutation the reader refuses at an identity check (`stale-node-key`, `stale_dependency`, `invalid_semantic_graph`) instead of its recorded code fails. For each `body_grammar_mutations` entry the harness also applies its `flattened` replacement at the same `pointer` in a fresh copy as a positive control, and the reader does not refuse that package `malformed_wire`; it may refuse it at a later identity check, the package being unrefreshed. The run fails, never skips, when the variable is unset or empty, when the file is missing or not JSON, when a list is absent or empty, when a `body_grammar_mutations` entry has no `flattened` member, or when a mutation does not refuse as recorded. A mutation the reader does not yet refuse as recorded is named in the harness's expected-failure list, each entry holding the mutation `id`, the refusal the reader gives it today, which differs from the recorded `outcome`, and the open ticket that owns the missing refusal; the run passes only while every listed id's refusal equals its listed one, and fails when a listed id is absent from `adverse.json`, when a listed id's refusal equals the recorded `outcome` (a stale entry, until it is deleted), when it equals neither, and when an unlisted mutation does not refuse as recorded. | Test (TC-048) |
| FR-038-AC-113 | `make conformance-qspec` reads `proposals/checked-package-v2/dependency-selection-vectors.json` from the checkout named by `QUIRE_SPECIFICATION_DIR` (never copied into this repository) and replaces `dependency_selections` with the file's version-free `{identity, package_id}` entries in both the `lock` and the `identity_preimage` of its `base` fixture, as QSpec's README states, then derives the `package_id` by calling this repository's own derivation, the reader's `CheckedPackageIdentityPreimageV2` encoded through `quire-canonical` (FR-038-AC-89), so a harness that hashes the JSON itself cannot pass; the result equals the file's recorded `package_id`, and the `base` fixture unchanged derives its own different `package_id`, so an entry carrying a `version` member, which is refused `unknown_member` (FR-038-AC-62 through FR-038-AC-64), is never the input of the recomputed identity. The run fails, never skips, when the variable is unset or empty, when the file is missing or not JSON, or when the recomputed identity differs from the file's. | Test (TC-048) |
| FR-038-AC-114 | A node body in which an application of any operator class other than `case` stands as an element of another application's `arguments`, as a member of an `aggregate` or as the value of a `binding` refuses `malformed_wire` at that nested application, for `quire.op.function.call`, `quire.op.state.clause` and an application of each of the `temporal`, `temporal_formula` and `temporal_fairness` classes, in each of the three positions, at a pointer such as `/semantic_graph/nodes/{n}/body/arguments/0`; an `aggregate` inside a Group's members, a `binding` whose value is an `aggregate` inside an `aggregate`'s members where the stratum admits only a Leaf value, a `binding` whose value is a `binding`, a `binding` or a Tuple inside a Tuple's members (a Tuple's members are each a Leaf or a Group) and a `binding` as a body root refuse `malformed_wire` at that value; and the same meaning with each composite subterm as its own node reached by `reference` is not refused `malformed_wire`. | Test (TC-048) |
| FR-038-AC-115 | A `case` application nested inside another term refuses `ill_typed`/`operator-ineligible` at the nested application's `operator`; a non-`case` application at the body root of a node its class does not place it in refuses `ill_typed`/`operator-ineligible` at that node and not `malformed_wire`; and where one body or `details` term holds several offending constructs the refusal is at the first in document pre-order, outermost first: a `details` term that is a `temporal_formula` application with a nested `case` argument refuses at `/diagnostics/entries/{e}/details/{d}/operator`, a `details` aggregate of a `temporal_formula` application and then a `case` application refuses at `/diagnostics/entries/{e}/details/{d}/members/0/operator`, a node body whose `arguments/0` is a nested `case` application and whose `arguments/1` is a nested `quire.op.function.call` application refuses `ill_typed`/`operator-ineligible` at `.../body/arguments/0/operator` and, with the two swapped, `malformed_wire` at `.../body/arguments/0` (an IR reading: merged FR-322 gives the pre-order rule, merged FR-440 decides a nested `case` at the operation step, and the two are unreconciled), and an application of a class other than `temporal_formula`, `temporal_fairness` and `case` in a `details` term refuses `malformed_wire` at that application. | Test (TC-048) |
| FR-038-AC-116 | A package whose `node_id`s, `identity_preimage` and `package_id` are all stale and whose one node body holds a nested non-`case` application refuses `malformed_wire` at the nested application, and not `stale-node-key`, `stale_dependency` or `invalid_semantic_graph`, and one whose body holds a nested `case` application refuses `ill_typed`/`operator-ineligible` at its `operator` and not at an identity check (an IR reading, as in FR-038-AC-115); the same package with that body flattened and its identities left stale refuses at an identity check and not `malformed_wire`. | Test (TC-048) |
| FR-038-AC-117 | `CheckedPackageReadLimits` has no depth member and `CheckedPackageLimit` has no `Depth` variant, and no source file under `crates/quire-contract-model/src/checked_package/` holds `MAXIMUM_DEPTH`, another `MAX_*DEPTH` constant, `stacker`, `serde_stacker` or `on_stack_for` (the crate's v1 modules, their limits and its manifest are out of this scan: FR-019 and FR-023 keep them); a package holding a chain of 100000 nodes, each referencing the previous (a 100000-deep expression, as merged QSpec FR-322-AC-41 and TC-427 BG-03 build it), read under byte, node, edge, occurrence, diagnostic and work limits all sized so that none decides the outcome, on a thread whose stack is 256 KiB, is admitted and lowered from its last node with no outcome naming a depth and no stack overflow, and has the same JSON nesting depth as a package of one level; and an otherwise canonical document whose node body nests a term 300 levels deep, past the strict parse's recursion limit of 128, read on a thread whose stack is 256 KiB, refuses `malformed_wire` with no pointer at the strict parse, ahead of the canonical-bytes check and the body grammar, and is never `incomplete`, while the same document nested 20 levels deep, and one nested within the strict parse's limit but far past the grammar (a body of 61 aggregates, 126 JSON levels), each refuse `malformed_wire` at the first value outside the body grammar (FR-038-AC-114), on a 256 KiB thread in a debug build with no stack overflow. | Test (TC-048) |
| FR-038-AC-118 | Each of the five `body_grammar_mutations` of QSpec's `adverse.json` (`application-in-application-arguments`, `application-in-aggregate-members`, `application-in-binding-value`, `aggregate-in-group-members` and `binding-as-body-root`) refuses `malformed_wire` as its recorded `outcome` names, through the harness of FR-038-AC-112, whose expected-failure list holds none of them. | Test (TC-048) |
| FR-038-AC-119 | On each of the eight interval operators under a clause whose `temporal_profile` law names `quire.temporal.timed/v1`, the timed form `{lower, upper, lower_end, upper_end}` admits with bounds `{numerator: "0", denominator: "1"}` and `{numerator: "3", denominator: "1"}` for each of the four end pairs `closed`/`closed`, `closed`/`open`, `open`/`closed` and `open`/`open`; it admits with `lower` `{numerator: "1", denominator: "2"}`, with `lower` and `upper` both `{numerator: "3", denominator: "1"}` and both ends `closed` (a punctual `[3, 3]`), and with `lower` `{numerator: "0", denominator: "1"}`; and a `null` interval on the same operators admits (merged FR-370-AC-3 and FR-370-AC-8). | Test (TC-048) |
| FR-038-AC-120 | Each of these timed-form intervals refuses `invalid_package`/`invalid-value` at the bound, `/semantic_graph/nodes/{n}/body/operation/member/interval/lower` (or `.../upper` where it is the `upper` that fails), in strict wire validation, before the `package_id` recomputation, any node-key check and any temporal step (so also in a package whose `package_id` and node ids are all stale), with the bound of the first failing member in member order (`lower` then `upper`) and the same refusal under `quire.temporal.timed/v1`, `quire.temporal.infinite-trace/v1` and each of the three bounded profiles: `lower` with `numerator` `"-1"` (merged FR-370-AC-9's negative numerator), `lower` with `numerator` `"01"`, `upper` with `denominator` `"0"`, `upper` with `denominator` `"-2"`, `lower` a JSON integer `0`, `lower` the string `"3"`, and `upper` `null` with the four members present (an IR reading from the member-set rule: merged FR-370 states the upper bound is always present); a timed form with `lower` numerator `"-1"` and `upper` denominator `"0"` refuses at `lower`; the pointer is an IR reading, the published schema naming `.../lower/numerator` and the crate's rational check of a nominal preimage `.../numerator`; and a package whose first node holds a non-reduced bound and whose later node holds any of these pattern failures refuses `invalid-value` at the later node's bound. | Test (TC-048) |
| FR-038-AC-121 | Under `quire.temporal.timed/v1`, `lower` `{numerator: "2", denominator: "4"}` refuses `invalid_semantic_graph` at `.../interval/lower` (the code is merged FR-370 and FR-322-AC-10's; the stage after strict wire validation and the identity recomputation, before the model-selection owners step, and the locus are an IR reading), before the temporal step, so a package holding it beside a profile-fit defect in a lower-digest node refuses `invalid_semantic_graph`; a non-reduced `lower` `{numerator: "2", denominator: "4"}` with `upper` numerator `"-1"` refuses `invalid_package`/`invalid-value` at `.../interval/upper`, a negative `lower` numerator `"-1"` with a non-reduced `upper` refuses `invalid-value` at `.../interval/lower`, and a non-reduced `upper` alone refuses `invalid_semantic_graph` at `.../interval/upper`; `(3, 3]`, `[3, 3)` and `(3, 3)` with both bounds `{numerator: "3", denominator: "1"}`, and `lower` `{numerator: "5", denominator: "2"}` over `upper` `{numerator: "2", denominator: "1"}`, refuse `invalid_package`/`invalid-value` at `/semantic_graph/nodes/{n}/body` (merged FR-370 and FR-370-AC-8), while `[3, 3]` admits; `lower` `{numerator: "1", denominator: "2"}` with `upper` `{numerator: "2", denominator: "3"}` admits and the two swapped refuses; and, both ends `closed`, `lower` `{numerator: "18446744073709551617", denominator: "3"}` with `upper` `{numerator: "18446744073709551616", denominator: "3"}` refuses `invalid-value` at the body, which a float comparison, rounding both numerators to 2^64, gets wrong by admitting it, and the two bounds swapped admit, which a checked parse into 64 bits gets wrong; that beyond-2^64 package, `lower` `{numerator: "18446744073709551617", denominator: "3"}` and `upper` `{numerator: "18446744073709551616", denominator: "3"}`, read with a work limit that the GCD or cross-multiplication of its bounds takes past returns `incomplete` naming the `work` limit (FR-038-AC-3) and no refusal of the interval. | Test (TC-048) |
| FR-038-AC-122 | The timed form `{lower, upper, lower_end, upper_end}` with bounds `{numerator: "0", denominator: "1"}` and `{numerator: "3", denominator: "1"}` and both ends `closed` refuses `invalid_package`/`operation-member-mismatch` at `/semantic_graph/nodes/{n}/body` under `quire.temporal.infinite-trace/v1` and under each of the three bounded profiles (merged FR-370-AC-8), and admits under `quire.temporal.timed/v1`, whose refusal of an integer-form `{lower, upper}` and of `{lower, upper: null}` is FR-038-AC-104's; four-member intervals with valid rational bounds and `lower_end` `"half"`, or `upper_end` `"half"`, each refuse `invalid_package`/`operation-member-mismatch` at `operation.member`, as do `{lower: "0", upper: "3", lower_end: "closed"}` (a missing end, integer-string bounds), `{lower: "0", upper: "3", lower_end: "closed", upper_end: "closed", extra: "x"}` (a fifth member, integer-string bounds), each an IR reading that diverges from the published schema, whose closed interval `oneOf` and end `enum` fail them at strict wire validation, and each reported at the operation step after the temporal step, so a profile-fit defect in a higher-digest clause is reported first; and the same missing-end and fifth-member intervals with the rational-object bounds of the timed form refuse `invalid-value` at `.../interval/lower` instead, an interval of another member set having no form, its bounds judged against the integer pattern (an IR reading). | Test (TC-048) |
| FR-038-AC-123 | Tamper regression (IR-627; implemented for the member-read, scalar-operand, state-field body-target and unreferenced rows). Over a package whose model-owned field read names a `bounded_domain`/`integer_range` node keyed as `Int[0, 1000]` (the field declared `Int[0, 1000]` in the selected domain document), the unmutated package admits; the same package with that node's `max` binding changed to `10`, and separately to `5000`, the `node_id` kept, `identity_projection` patched and `package_id` recomputed through `quire-canonical` in the test, each refuses `invalid_package`/`stale-node-key` at that node's `node_id` and returns no package. Mutation rows: `max` changed to `10`; `max` changed to `5000`; `min` changed to `1`; `min` and `max` swapped. The same four rows over an `Int[0, 1000]` node that no member read names (a scalar operand's bound and a state field's body target) refuse identically, so a reader that only compared the node a member read names admits those and fails. | Test (TC-226) |
| FR-038-AC-124 | Closed body of each derived shape (IR-627; implemented; every refused body is keyed again by its own derivation in the test, so only the closed-body rule can refuse it. A derived-shape node in a `recursion_group` also has no derivable key, the IR reading of IR-627-Q4). An `integer_range` or `collection_bounds` node whose body is an `aggregate` of exactly the bindings `min` then `max`, each an `integer` literal typed at the `Integer`-keyed node with a string of its form's grammar (`^(0|-?[1-9][0-9]*)$` for `integer_range`, `^(0|[1-9][0-9]*)$` for `collection_bounds`), and whose key is the derived one, admits; an `integer_range` `min` of `"-5"` admits. A body with `max` absent, a third binding, the order `max`, `min`, a `max` of `"01"`, `"+5"`, `"-0"`, `"-01"` or `""` on either form, a `collection_bounds` `min` of `"-1"`, a `max` literal typed at the node keyed as `Boolean` or at any node other than the `Integer`-keyed one, or a `max` that is a `text` literal, each refuses `invalid_package`/`stale-node-key` at that node's `node_id`. A `scalar_type`/`integer` node with a non-empty body, a `reference`, `option` or collection node with no member, two members or a member that is not a `reference`, each refuses identically. | Test (TC-226) |
| FR-038-AC-125 | Self-typing and `semantic_type` (IR-627; implemented; an `integer_range` node whose `semantic_type` is not the `Integer`-keyed node also has no derivable key, even when its key is derived over that `semantic_type`). A `scalar_type` or `composite_type` node of a derived shape whose `semantic_type` is another node refuses `invalid_package`/`stale-node-key` at its `node_id`, although its key is unchanged. A `collection_bounds` node whose `semantic_type` is re-pointed at a collection of another element range, an `integer_range` node whose `semantic_type` is re-pointed at a node other than the `Integer`-keyed one, and a `collection_bounds` node whose `min` and `max` literal `type` is re-pointed at a genuinely keyed node of another type, each keep the stored `node_id` and each refuse the same way, because `semantic_type` and the literal `type` are in the preimage. A rekeyed `integer_range` typed at Boolean and a rekeyed `collection_bounds` typed at Boolean likewise refuse `invalid_package`/`stale-node-key` at the fresh `node_id`: a bounded form has no derivable key when its semantic type is not respectively the Integer node or a collection (`set`, `bag`, `sequence`, `ordered_set`) node. | Test (TC-226) |
| FR-038-AC-126 | Indirect redirection (IR-627; implemented over the in-repo fixture and selected-document field and operation contexts). Over fields and operations declared `Option<Int[0, 1000]>`, `Set<Int[0, 1000]>`, `Sequence<Int[0, 1000]>`, `Reference<O>` and an operation parameter typed `Int[0, 1000]`, a package in which the `option`, a collection or `reference` node keeps its stored key while its body `reference` is re-pointed at a genuinely keyed `Int[0, 10]` node, and one in which the parameter's type node holds the body `max` `10`, each refuses `invalid_package`/`stale-node-key` at the re-pointed or tampered node's `node_id`, and never `ill_typed`. A reader that re-derived only `integer_range` nodes admits the re-pointed `option`, collection and `reference` rows and fails. | Test (TC-226) |
| FR-038-AC-127 | Stage and order (IR-627; implemented). The derived-shape re-derivation runs after the graph-shape stage and the application key stage and before the nominal key stage and every declaration, frame, state, temporal, abstraction and operation step. A package holding a graph-shape defect and a tampered node reports the graph-shape defect; one holding a stale application key and a tampered node reports the stale application key; one holding a tampered node and an `ill_typed` defect in an application that reads it reports the tampered node; one holding two tampered nodes reports the one whose own `node_id` digest is lower. Each report is `invalid_package`/`stale-node-key` at that node's `node_id` and never at the reading application. | Test (TC-226) |
| FR-038-AC-128 | Exact bounds at the extremes (IR-627; implemented over declared fields and in-repo fixture nodes). Over fields declared `Int[0, 0]` and `Int[-170141183460469231731687303715884105728, 170141183460469231731687303715884105727]` (the `i128` extremes), a node holding the declared bounds, the `i128` minimum as `min` included, admits and a node whose `max` differs from the declared bound by one in either direction refuses as AC-123 does. A comparison through a lossy float conversion would admit `"170141183460469231731687303715884105726"` against the `i128` maximum and fails this row. | Test (TC-226) |
| FR-038-AC-129 | Trust root (IR-627; implemented). A package whose `model_selections` row is re-pointed at a document declaring `Int[0, 10]`, whose node is re-keyed to `Int[0, 10]` and whose `package_id` is recomputed refuses `missing_import`/`missing-selection` at the row's `digest` when the evidence holds only the original document (FR-038-AC-27). The same package admits when the caller's evidence also holds the re-pointed document, because the reader admits a package against the evidence it is given, and the test records that admission as the stated limit of the trust root. | Test (TC-226) |
| FR-038-AC-130 | One derivation (IR-627; implemented by the unit test of `v2/derived_keys.rs`). For each of the ten derived shapes the key the admission stage derives from a node's own body equals the key `MemberType::node_key` derives from the matching member type, and a node whose body is built from the derived key's own preimage admits. A test derives both for each shape over the bounds of AC-128 and compares them. | Test (TC-226) |
| FR-038-AC-131 | Structural forms (IR-627 owner-free ungrouped stage implemented; owner-bearing, declared and grouped re-derivation remains gated on IR-630). For an owner-free ungrouped node, a stored `node_id` that differs from the QSL FR-092 derived key refuses `invalid_package`/`stale-node-key` at that node's `node_id` for forms including `rational_range`, `decimal_range`, `float_rounding`, `text_bounds`, `model_population`, `compound_unit`, `text`, `parameter`, `union`, `record`, `tuple`, the value forms. Declared (`declaration`-carrying), owner-bearing and grouped nodes are excluded from this owner-free stage and remain for IR-630. Mutation rows, one per included form: change one body value and keep the node id. A reader that derives only the ten decided shapes admits every row and fails. The generic stage is tested with one body mutation per included owner-free, ungrouped form, and owner-bearing, declared and grouped rows remain for IR-630. | Test (TC-226) |
| FR-038-AC-132 | No copy (IR-627; checkout-reference harness implemented, positive conformance pending QSpec #191 owner fixtures; IR-630 owner-bearing and grouped conformance remains gated). The vectors or types the reader is checked against for the gated forms come from the source IR-627-Q2 names, are not copied into this repository, and a test that reads them fails closed when the source is absent, as FR-038-AC-112 does. | Test (TC-226) |
| FR-038-AC-133 | Positive fixtures carry derived keys (IR-627; planned, ungated; supersedes FR-038-AC-107 for derived-shape nodes, which stands as built until then). The three fixtures AC-107 reads (`positive-all-families.json`, `positive-clause-operations.json`, `positive-union-nodes.json`) admit end to end with the derived key on every derived-shape node. The same fixture with the key of any one undeclared derived-shape node replaced by a placeholder, and every reference to it left in place, refuses; the nodes are `positive-all-families.json` nodes 24 (`integer`, `7f7f...`) and 26 (`reference`, `b2b2...`), `positive-clause-operations.json` nodes 0 (`boolean`, `aaaa...`), 7 (`integer`, `7f7f...`) and 12 (`reference`, `b2b2...`), and `positive-union-nodes.json` node 0 (`integer`, `a1a1...`), and `positive-all-families.json` nodes 0 (`boolean`, declared `Example::Flag`) and 2 (`integer_range`, declared `Example::Small`) are gated forms this row does not use. The refusal is `invalid_package`/`stale-node-key` at that node's `node_id`, and no placeholder key is tolerated. Owned by QSpec for those three fixtures and for the recorded `package_id` of `dependency-selection-vectors.json` (AC-113), which IR reads and whose base is `positive-all-families.json` (QSL-635, IR-627-Q5; QSL-635 also covers `positive-control-operations.json`, `positive-operation-identities.json` and `node-identity-vectors.json`, which IR does not read); `make conformance-qspec` fails on the three until they conform, so the IR code change merges after QSL-635's regenerated fixtures or a conformance lane is held citing QSL-635. QSpec also owns the conformance counterpart of the tamper regression row of TC-226: a negative fixture whose `integer_range` node keeps its id under a changed bound, patched `identity_projection` and recomputed `package_id`, which the reader refuses `stale-node-key` at that node (read from the checkout, never copied here). | Test (TC-226) |
| FR-038-AC-134 | In-repo fixtures carry derived keys (IR-627; implemented). Every in-repo fixture package (`tests/it/support/checked_package.rs`) carries on each undeclared node of the ten shapes the closed body of its form, its own key as `semantic_type` for a `scalar_type` or `composite_type`, and the derived key, and admits. The migration changes more than keys: the `aaaa` boolean's `literal` body becomes `aggregate{[]}`, and the `bbbb` option, typed at `aaaa` with an empty aggregate body, becomes self-typed with a body of one `reference`. The same package with the `aaaa` placeholder key restored on its boolean node refuses `invalid_package`/`stale-node-key` at that node's `node_id`, and with the `bbbb` body or `semantic_type` left unmigrated refuses the same way. Owned by this repository's code change. | Test (TC-226) |
| FR-038-AC-135 | Adverse mutations still reach their stage (IR-627; planned, ungated; supersedes AC-112's base package when the code lands). With `positive-all-families.json` carrying derived keys, each `structural_mutations` and `body_grammar_mutations` entry of `adverse.json` refuses with exactly its recorded `outcome`, among them `wrong-node-family-form` as `invalid_semantic_graph`, `negative-temporal-interval-bound` as `invalid_package` and the five `malformed_wire` body-grammar entries (eleven entries in all: six `structural_mutations`, five `body_grammar_mutations`), and none is refused `stale-node-key`; `dependency-selection-vectors.json` over the regenerated base carries its regenerated `package_id`. The harness fails, as AC-112 says, on a mutation refused at an identity check instead of its recorded code. | Test (TC-226) |
| FR-038-AC-136 | Fields as values (IR-628; implemented, ungated). Over an admitted package whose `model`/`object_type` node is a model declaration node with the empty body (the shape QSL emits), selected from a domain document declaring `balance: Int[0, 1000]` and `audit: Int[0, 1000]`, with a read of `balance` only, `model_object_fields` returns exactly two fields, `audit` then `balance`, each with `member_type` `Some(IntRange { lower: 0, upper: 1000 })` as `i128` values; `field("audit")` is `Some` although no read names it, and `field("missing")` is `None`. Mutation rows: a reader that takes the fields from the node body returns none; one that takes them from reads' `result_type` omits `audit`; one that returns the order of the document's declaration or of a hash map breaks the ascending-name row (a document declaring `zeta` before `alpha`). | Test (TC-227) |
| FR-038-AC-137 | Effective set is the one admission resolves (IR-628; implemented, ungated). Over a document with a supertype chain, an inherited field, a field redefined by a subtype with a narrower `Int[lo, hi]`, a most-derived redefiner two owners deep and an own field, the accessor returns each exposed name once, with the most-derived redefiner's type, and no hidden field; for every returned field, a package whose read of it is keyed by the independent recomputation of the returned type's key admits, and the same read keyed by the hidden base field's type refuses `ill_typed`/`operator-ineligible` (step 4). Mutation rows: inherited fields omitted; a redefined base field returned beside its redefiner; the base type returned for a redefined field; a less derived redefiner returned. | Test (TC-227) |
| FR-038-AC-138 | Every member kind (IR-628; implemented, ungated). Fields declared `Boolean`, `Integer`, `Int[0, 0]`, `Int[-5, 5]`, `Int[0, 9007199254740993]`, the `i128` extremes as `Int[lo, hi]`, a `Reference` to another object type, an optional `Int[0, 1000]`, a `Set`, `Bag`, `Sequence` and `OrderedSet` of `Int[0, 1000]`, a bounded collection `Set<Int[0, 1000]>[1, 4]` and an optional `Set` return `Boolean`, `Integer`, `IntRange` with the exact `i128` values, `Reference` carrying the key of the referenced model declaration node, `Option`, `Collection` with the matching `CheckedCollectionKind`, element and `Some((1, 4))`. Fields declared `Text`, `Rational`, `Decimal`, a systems interface type, a value type not bound as `Int[lo, hi]`, `Int[0, 170141183460469231731687303715884105728]` and `Int[-170141183460469231731687303715884105729, 0]` (one past each `i128` extreme), and a collection of lower bound 1 with no upper bound are present with `member_type` `None`, and the call returns the other fields (`Ok`, never a refusal). Mutation rows: bounds narrowed through `i64` (the `i128` extremes differ) or `f64` (2^53 + 1 differs); a bound past `i128` saturated, wrapped or truncated into an `IntRange`; `Option` dropped; a `Collection` kind swapped; a `None` field omitted; a `None` field refusing the call. | Test (TC-227) |
| FR-038-AC-139 | Absence and refusal semantics (IR-628; implemented, ungated). `UnknownNode` for an id whose digest is no node of the graph; `NotModelObjectType` for a `scalar_type` node, a `systems_interface` declaration node, a `relationship` declaration node, a `model`/`object_type` node with a `declaration`, one whose key matches no selected declaration, and one whose key matches but whose body is not `aggregate{[]}`; `AmbiguousField` carrying `x` for an object type whose document gives two unhidden effective fields named `x` (own and inherited, no redefinition), in package A, which no read of `x` names and which admits, the call returning the error for the whole call even though the object also declares a valid `Int[0, 1000]` field `y`; package B, the same document with a read of `x`, is refused at admission `ambiguous_declaration`/`ambiguous-name` and yields no package to call; `field("absent")` is `None` and the call is `Ok`. The `NotModelObjectType` rows over a node with an unread, non-empty body hold by the rule of item 17, so the two admitted packages that differ only in that node's body return `Ok` and `NotModelObjectType`. Mutation rows: `UnknownNode` and `NotModelObjectType` collapsed into one; ambiguity resolved by returning the first or last field, or the other fields; an absent field returned as an error; the non-empty-body node returning the fields. | Test (TC-227) |
| FR-038-AC-140 | Independent of any node body (IR-628; implemented, ungated; holds before and after IR-627's stage lands). A crate-internal test over an admitted package (the module's own tests can build a `CheckedPackageV2` from an admitted one) replaces, after admission, the body bounds of the `Int[0, 1000]` node that a read of the field names with `0` and `10`, and separately `0` and `5000`, and the accessor returns `IntRange { lower: 0, upper: 1000 }` for the field, equal to its result before the replacement. Mutation row: an accessor that takes the bounds from the node a read names, or from any node body, returns `10` or `5000`. This row stays catchable once IR-627's stage refuses such a package at admission, because the replacement happens after admission. | Test (TC-227) |
| FR-038-AC-141 | One derivation with admission (IR-628; implemented, ungated). For each field of AC-138 whose type is a range, `Reference`, `Option` or collection, a read keyed by the key the test recomputes through `quire-canonical` from the accessor's returned type admits, and the same read keyed from the perturbed type refuses `ill_typed`/`operator-ineligible` at the read. The perturbation is: for an `IntRange`, `upper - 1` when `upper > lower`, `upper + 1` when `lower == upper < i128::MAX`, and `lower - 1` when `lower == upper == i128::MAX`; for a collection's bounds, `upper - 1` when `upper > lower`, `upper + 1` when `lower == upper < u64::MAX`, and `lower - 1` when `lower == upper == u64::MAX`; so every perturbed type node stays well-formed (`min <= max`) and is refused at the read as `ill_typed`, never for its own shape; for a `Reference`, the key of another object type's model declaration node; for `Boolean` and `Integer`, the other scalar; for an `Option` or a collection, the same wrapper over the perturbed element. Mutation rows: an accessor that derives types by its own copy of `field_type` which differs in the optional wrapper, a collection's bounds, or a `Reference` target. | Test (TC-227) |
| FR-038-AC-142 | Public shape and stability (IR-628; implemented, ungated). A compile-time API fixture shows `model_object_fields` takes `&self` and one `&CheckedNodeId` and no evidence; an exhaustive match over `CheckedMemberType`, `CheckedCollectionKind` and `CheckedModelFieldsError` with no wildcard arm compiles; code constructing `CheckedModelObjectFields` or `CheckedModelField` by struct literal fails to compile; the criterion is the compile fixtures: the exhaustive matches compile from an external crate (`tests/it`) only while the enums are closed. Mutation rows: `#[non_exhaustive]` added to an enum (the wildcard-free match fails to compile); a public field on either struct (the struct-literal fixture compiles); a variant added to an enum (the exhaustive match fails to compile). | Test (TC-227) |
| FR-038-AC-143 | Pure, total, bounded (IR-628; implemented, ungated). Two calls on one package return equal results; a clone of the package returns an equal result; two admissions of the same bytes under different byte limits (both admitting) are equal as packages and return equal results; a package admitted (under a `work` limit set high enough) from a document with an inheritance chain of 200 object types and 100 redefinitions of one field has a call that returns the fields without panic or stack overflow, and the call does no resolution (a counter on the table-building function of items 2 and 7 reads the same value before and after the call); a node of AC-139's refusal cases returns its error and never panics. Equality of two packages admitted from the same bytes is unchanged by the retained tables. Mutation rows: the accessor panics on an unwrap of a missing owner; it recurses over a 20,000-type chain on a 256 KiB thread stack; it resolves per call; equality of packages gains the retained tables (compared with a crate-internal package holding the same admitted content and cleared tables). | Test (TC-227) |
| FR-038-AC-144 | Resolution is charged once, at admission (IR-628; implemented, ungated). Cost model of items 8 and 9, with the arithmetic. For a chain of N object types, the first declaring one field and each later one declaring one field and extending the previous, the charge is 1 for the first type and k + 1 for the type at depth k >= 2 (one own field, one supertype edge, k - 1 copied entries), in all N(N + 1)/2 + N - 1: 13 for N = 4 (1 + 3 + 4 + 5) and 43 for N = 8. A crate-internal test reads the table-building counter and asserts 13 and 43. For the same chains, the smallest `work` limit that admits is M, and the limit M - 1 returns `incomplete` with the pointer `/lock/model_selections/0`. Under `CheckedPackageReadLimits::bounded()` (work 1,000,000), a chain of N = 1000 (charge 1,000 x 1,001 / 2 + 999 = 501,499) admits and a chain of N = 1500 (1,500 x 1,501 / 2 + 1,499 = 1,127,249) returns `incomplete` at `/lock/model_selections/0`. Precedence (item 11), over one package each: a table charge that exhausts the limit together with a graph-stage refusal (a stale node key) returns `incomplete`, because tables precede `validate_graph`; a step 1 refusal at row 1 together with a table charge that exhausts the limit at row 0 returns the step 1 refusal, because every row's step 1 precedes any table charge. An object type with an ambiguous name (AC-139, package A) admits, and its table is charged as any other. Read charge (item 10): a read of a field of the N = 4 chain's deepest type charges 4 units (the four entries of its table, no hidden entries), a read of the root charges 1, and a read of an operation charges the unchanged step 3 walk; a frame entry naming a field and an abstraction relation's field entry on that deepest type each charge the same 4 units, a relationship edge read charges the entries of its table, and none runs the ancestor walk for a field (a counter on the walk stays at its pre-resolution value); a state clause resolves only operations and is unchanged. Cycles (items 4 to 6, 9), each pinned against what `resolve` returns today (the expected tables are recorded by running the pre-change `resolve` over each document before the code change and stored as the test's expected values, so they are not asserted from reading the code): a type `A extends A` declaring one field `f` admits, its table is `{f}` and the charge is 2; types `A extends B` with field `a` and `B extends A` with field `b` admit, the tables of `A` and `B` are the one shared table `{a, b}` and the charge is 4; `A extends A` declaring `f` and `g` where `g` redefines `f` admits and its table equals what `resolve` returns for `A` today (by reading `resolve`, `g` hides `f` and, since `A` is its own ancestor, itself, so neither is exposed); a document with a third type `C extends A` (outside the cycle) has a table equal to `resolve`'s for `C`, charged with the entries copied from the cycle's table. Mutation rows: tables computed after admission or per call (the limit never exhausts); the charge omitted for an unread object (the one-below limit admits); tables built after the graph stage (the precedence row returns the graph refusal); a per-type ancestor re-walk (the counter exceeds 13 for N = 4); an ambiguous table refusing admission; a field resolution (read, frame entry, abstraction field entry) that still runs the walk or charges nothing; a cycle refused at admission, built without termination, or built per member (the charge exceeds 4 for the two-cycle); a cyclic table differing from `resolve`'s. | Test (TC-227) |
| FR-038-AC-145 | Real groups: skip, not refuse (IR-627; planned, ungated; supersedes the refusal reading of IR #295's Q4). A package of `record List { next?: List; }` (an `Option` self-reference) and `record Tree { kids: Sequence<Tree>[0, 3]; }` (a `Sequence` self-reference whose `collection_bounds` node names the `Sequence<Tree>` node at its `semantic_type`), each under one `recursion_group` with its keys as QSL FR-092 mints them, admits, and the decided stage reports no `stale-node-key` for any in-group node. The skipped set is exactly the nodes of the six skippable shapes (`option`, `set`, `bag`, `sequence`, `ordered_set`, `collection_bounds`) that carry a `recursion_group`, lie on a cycle of the names graph and sit in a strongly connected component of that graph every node of which carries that label (FR-038 "Derived-shape nodes inside a recursion group"); the declared record and function of a group are gated forms the stage never touches. An application node inside a group keeps FR-038-AC-88's key check and refuses `stale-node-key` when its `node_id` is stale. | Test (TC-226) |
| FR-038-AC-146 | Shapes that can never be in a group are always re-derived (IR-627; planned, ungated). For `scalar_type` `boolean` and `integer`, `bounded_domain` `integer_range` and `composite_type` `reference`, the decided stage re-derives the key and checks the closed body whatever `recursion_group` the node carries. Over the `Int[0, 1000]` node: with `recursion_group` `"x"` added (a label no other node carries), `max` changed to `10`, `node_id` kept, `identity_projection` patched and `package_id` recomputed through `quire-canonical` in the test, the package refuses `invalid_package`/`stale-node-key` at that node's `node_id`; the same with the label of the genuine `Tree` group added, refusing at the `Int[0, 1000]` node alone (the group's own nodes carry that label too and are skipped, so no other node is reported); the same over the acyclic `Set<Int[0, 1000]>[0, 3]` package of FR-038-AC-147 with a genuine cycle closed through its own nodes: the `Int[0, 1000]` node's `semantic_type` re-pointed at the `set` node (which already names the `Int[0, 1000]` node in its body `reference`), both labelled `"x"` (a names cycle of the two nodes whose members all carry the label), and `max` changed to `10`, the `node_id` kept, refusing `stale-node-key` at the `Int[0, 1000]` node's `node_id`, which is the only node reported because the `set` node is a skippable shape on that cycle and is skipped. An `integer_range` node that carries a label and holds its derived key admits, so the label alone is not a refusal. Mutation rows: a reader that skips on the label alone admits the first two rows and fails; a reader that skips any labelled node on a names cycle, whatever its shape, skips the `Int[0, 1000]` node of the third row, which is on the cycle and labelled, and admits the tampered `max`, and fails; a reader that re-derives only `integer_range` admits the same rows over a `reference`, `boolean` and `integer` node and fails. | Test (TC-226) |
| FR-038-AC-147 | A label or a dependency does not make a cycle (IR-627; planned, ungated). Over a package with an acyclic bounded collection (a `Set<Int[0, 1000]>[0, 3]` field, so a `collection_bounds` node naming its `set` node and no cycle): the `collection_bounds` node with the label `"x"` added and `max` changed to `5` refuses `stale-node-key` at its `node_id` (it is on no names-graph cycle, so it is verified as ungrouped); the same node with its own `node_id` added to its `dependencies` (a self-dependency, outside the names graph) and the label `"x"` added, `max` changed to `5`, refuses identically, and without the label refuses `invalid_semantic_graph` at the graph-shape stage as FR-038-AC-18 does, before this stage. Over the `Tree` package, with one more node carrying the `Tree` group's label that is off the group's cycle (a `Set<Int[0, 1000]>` node, with its key left as its ungrouped key), every member of the group is still skipped and the extra node, verified as ungrouped, admits; with its body `reference` re-pointed at a genuinely keyed `Int[0, 10]` node and its `node_id` kept it refuses `stale-node-key` at its `node_id`. The existing admission of a lone label on an acyclic node, `tc_048_package_id_covers_exactly_the_identity_preimage`, stands. Mutation rows: a skip keyed on the label alone; a skip keyed on a cycle of IR's own graph, which counts `dependencies` (the self-dependency row admits and fails); a skip that requires the component to be exactly the labelled set (the extra-node row refuses a genuine member and fails). | Test (TC-226) |
| FR-038-AC-148 | The stated limit is recorded as tests (IR-627; planned, ungated). Over the `Tree` package, each single mutation below, with the `node_id` kept and the identity members recomputed, is not refused `stale-node-key` by the decided stage, and none is refused for carrying a `recursion_group`: `max` of the in-group `collection_bounds` node changed to `5`; its `min` changed to `1`; the `Sequence<Tree>` node's body `reference` re-pointed from the record at the `collection_bounds` node, which leaves a names cycle of those two nodes (the labelled record is off the cycle and is a gated form the stage never touches, and both remaining nodes are skipped). A forged group is recorded the same way: a package of an acyclic `Set<Int[0, 1000]>[0, 3]` field whose `set` node's body `reference` is re-pointed at its own `collection_bounds` node (a names cycle of two nodes through `semantic_type` and the body reference), both nodes labelled `"x"`, with the bounds `max` changed to `5`, admits, which is the limit; and so does the retag of FR-038 "Stated soundness limit": the node keyed as `Int[0, 1000]` retagged `collection_bounds` and labelled `"x"`, its `max` changed, its `semantic_type` re-pointed at a `set` node labelled `"x"` whose body references it, which a consumer reading by wire form finds to hold no `integer_range` and a consumer reading the node by the key of the field type finds to hold a body that is not that type. The test fails if any of these refuses `stale-node-key` at a skippable-shape node, so that the day FR-038-AC-150 lands the rows move to refusals in the same change. | Test (TC-226) |
| FR-038-AC-149 | A skipped node is charged (IR-627; planned, ungated). The stage charges one work unit for every node of the ten shapes it visits, skipped or not. Over the `Tree` package, in which the test counts K nodes of the ten shapes (the skipped `Sequence<Tree>` and `collection_bounds` nodes included), a crate-internal counter on the stage reads K after the read, K being the constant the test counts from the package it builds and not a value read from the stage. Mutation row: a stage that does not charge a skipped node reads K minus the number of skipped nodes and fails. | Test (TC-226) |
| FR-038-AC-150 | In-group re-derivation (IR-627; planned and GATED on IR-630, which waits on QSL-638's wire `owner`; IR-627-Q1 and Q4). Once the reader derives the preimage of each declared member of a group, including its `SourceOwner`, a package of the `List` and `Tree` groups of FR-038-AC-145 whose in-group keys are those QSL FR-092 mints (the group digest over the group-local preimages in ordinal order; `recursion` `{size, ordinal, group}` on every structural member; `{term: "group_reference", ordinal}` at every position that names a member of its own group, `semantic_type` included) admits. Each of these refuses `invalid_package`/`stale-node-key` at the node's `node_id`: `max` of the in-group `collection_bounds` node changed to `5`; the `Sequence<Tree>` node's body `reference` re-pointed at another member of the group (a different ordinal); the `semantic_type` of the `collection_bounds` node re-pointed at the record (a different ordinal at `semantic_type`); the group's members written in another wire order with every `node_id` kept; the forged group of FR-038-AC-148. A group in which no member carries a `declaration`, such as QSL FR-092's vector G1 (an `option` over itself), needs no owner and is not gated on Q4, because every member's preimage is derivable from the wire: it is re-derived once the reader computes the group digest. The preimage's `recursion` and `group_reference` members are derived, not wire members, so each mutation reaches them through the wire member that produces them. The stated limit, recorded as the admission of the same group with its members written in another wire order and every key recomputed for that order: the reader reads each ordinal from graph order, as QSL FR-092 states for FR-322, and does not recompute the content order (IR-627-Q4). Until the gate lifts this criterion has no test and no implementation. | Test (TC-226) |
| FR-038-AC-151 | Implemented by IR-644 and verified by TC-048. Over the QSL-shaped recursive `List` record with an integer field and `next` encoded as `binding(next, aggregate([binding(optional, reference Option<List>)]))`, `structural.eq` over two `List` values admits with `leaves` empty, and the leaf derivation terminates through the option's `inner` edge at the record reentry. Replacing that integer field with a text field whose profile is selected admits exactly its text leaf followed by `["field:next", "inner", "recursion:0"]`; omission of that recursion leaf refuses `invalid_package`/`operation-law-missing` at `operation.leaves`. Adding a healthy text sibling after `next` retains its own leaf after the recursion leaf in declaration order, proving that the wrapper consumes exactly one field edge. A direct field `reference` to `Option<List>` also admits with the same `field:next`, `inner` path. | Test (TC-048) |
| FR-038-AC-152 | Implemented by IR-644 and verified by TC-048. With the rest of the `List` comparison well formed and its identity members recomputed after each mutation, replace only the value of `next` by an `aggregate` with no member, two `optional` members, a member named otherwise, a non-binding member, or an `optional` binding whose value is not a reference to an option type. Each mutation is admitted by the flat body grammar; after identity re-derivation, each reaches the operation check and refuses `ill_typed`/`operator-ineligible` at `operation.leaves`, rather than silently omitting `next` or deriving a leaf from a different member. | Test (TC-048) |
| FR-038-AC-153 | A declared record, tuple and function each carries its source owner in the node and equal identity projection; QSpec's two-owner packages for `Point` and recursive `List` admit with distinct `Point` ids, distinct `List` group labels and member ids, and equal builtin `Integer` and application-keyed `three` ids under the two owners. An undeclared model declaration and clause function each carries its model owner, while an anonymous type, `source_locus` node and application-keyed node carry no owner. | Test (TC-228) |
| FR-038-AC-154 | Omitting a required node or projection owner, inserting `null`, using the wrong owner kind, adding `version` to `ModelOwner`, or placing an owner on an owner-free node refuses `malformed_wire` at the node or projection object that lacks a required owner and at the present `owner` otherwise, before identity validation. A well-shaped projection owner differing from its node's owner refuses `invalid_package`/`invalid-value` at `/identity_preimage/identity_projection/{i}/owner`, before the owner join and even with the package id recomputed over that projection. QSpec's `projection-source-owner-differs-from-node` and `projection-model-owner-differs-from-node` mutations each exercise this refusal. The reader does not reconstruct an omitted owner from an occurrence, source map or lock. | Test (TC-228) |
| FR-038-AC-155 | Before structural key re-derivation, a `SourceOwner` absent from `lock.sources` refuses `missing_declaration`/`missing-selection` at its node's `node_id`; a source-map region of its `declaration` occurrence naming another source pair refuses `invalid_package`/`invalid-value` at that source-map entry even when both pairs are lock-selected. A `ModelOwner` whose `identity` is unselected, whose `node` names no declaration, or whose `node` names the wrong kind refuses `missing_declaration`/`missing-selection` at its node's `node_id`, even if nothing reaches that node: a relationship cannot back `model`/`object_type`, an object type with `interfaceFeatures` cannot back `model`/`object_type`, an object type without `interfaceFeatures` cannot back `model`/`systems_interface`, and a field cannot back an operation clause. An operation member's clause and an object type's invariant pass. Two owner defects select the lowest retained node-id digest, ahead of a stale key on a lower-key node. Each selected-document lookup charges one validation visit. | Test (TC-228) |
| FR-038-AC-159 | For admitted `quire.op.integer.add`, `quire.op.integer.sub`, `quire.op.integer.mul` and `quire.op.integer.negate` applications whose every operand has finite bounds representable in `i128`, `scalar_application_operands` returns one entry per argument, in wire argument order, with ordinals `0..n-1`, typed identities and exact inclusive `i128` ranges; the unary operation returns one entry and the binary operations return two. Swapping two arguments swaps their entries and ordinals without sorting by child id. | Test (TC-048) |
| FR-038-AC-160 | A reference to a `value`/`parameter` returns `GraphChild` carrying that parameter's node id and its admitted integer-range bounds; a reference to an application subterm returns `GraphChild` carrying that subterm's own node id and result range. Two distinct children of the same integer type remain distinct identities. An admitted graph literal reference carries the literal node id and singleton `(value, value)`, independent of a wider declared type bound. | Test (TC-048) |
| FR-038-AC-161 | For an admitted inline integer literal argument of value `7`, the result carries `InlineLiteral` naming the application node id, the supplied occurrence key and that argument's ordinal, with range `(7, 7)`; another position or occurrence yields a distinct identity. No `CheckedNodeId` is fabricated for the inline term. | Test (TC-048) |
| FR-038-AC-162 | An unknown graph node, a known non-application node, an absent occurrence key, a dangling referenced child and a child without a readable integer range produce the distinct typed errors `UnknownNode`, `NotApplication`, `MissingOccurrence`, `MissingChild` and `MissingRange`, respectively. An admitted reference to unbounded `Integer` returns `UnboundedRange`; an admitted integer-range endpoint, graph literal or inline literal outside `i128` returns `RangeOutOfI128`. Each refusal returns no partial list, narrowed value or panic. The dangling-child row may use a crate-internal mutation after admission because the reader ordinarily rejects it. | Test (TC-048) |
| FR-038-AC-163 | A crate-internal mutation of an admitted application to an operation identity absent from the catalog returns `UnknownOperator`; an admitted application with a catalogued identity outside the four eligible integer operations returns `IneligibleOperator`. The accessor does not interpret a non-integer operation as an eligible scalar parity claim. | Test (TC-048) |
| FR-038-AC-164 | An external Rust API fixture calls the accessor with `&CheckedNodeId` and `&CheckedOccurrence`, exhaustively matches the child-identity and error enums, and reads each ordinal, node id or inline selector and range through typed fields; it does not read `graph().nodes[*].body` or deserialize JSON. Repeated calls and a cloned admitted package return equal results, and neither call changes package equality. | Test (TC-048) |
| FR-038-AC-165 | PLANNED / UNRUN (IR-661). A selected, independently authored FCD-shaped document with a global `relationship` slot identity admits, while an owner-nested relationship, a foreign-package identity and a wrong-slot identity each refuse `invalid_model_binding`/`malformed-declaration`. Owner-nested fields, operations and operation parameters still admit; moving a field or operation into the global relationship slot refuses `invalid_model_binding`/`malformed-declaration`. | Test |
| FR-038-AC-166 | PLANNED / UNRUN (IR-661). Changing only an authored role makes the old name refuse `missing_declaration`/`missing-name` and the new name resolve on the same selected relationship. Destination-type and multiplicity changes are observed through the admitted navigation result nodes of AC-170, not through an unspecified retention accessor. Omitting only `targetEnd.role` admits the document with only its forward named end. Removing `sourceEnd.role`, or setting either present role to null, empty or a non-string, refuses `invalid_model_binding`/`malformed-declaration`; no role is synthesized or checked against an inverse registry. | Test |
| FR-038-AC-167 | PLANNED / UNRUN (IR-661). An unresolved `sourceEnd.type` or `targetEnd.type` refuses `missing_declaration`/`missing-name`; a type naming a relationship or another declaration of the wrong meaning refuses `invalid_model_binding`/`malformed-declaration`, independent of declaration order. A source end naming another owning type, a malformed end or multiplicity refuses `invalid_model_binding`/`malformed-declaration`; `lower > upper` alone refuses `invalid_model_binding`/`unpreserved-model-meaning`. Each declaration refusal retains its typed code, cause and selection-row pointer. The planned public refusal also retains the authentic identity of the actual relationship declaration and its valid origin verbatim under the declaration-refusal retention contract; source coordinates remain supplied, generated origin gains no span, and missing/malformed origin remains absent. Selection-row path and existing graph-node locus meaning stay separate; no metadata is fabricated. All added retention checks remain PLANNED / UNRUN.  | Test |
| FR-038-AC-168 | PLANNED / UNRUN (IR-661). Repeating a relationship identity, including under two different owners, refuses `invalid_model_binding`/`conflicting-binding` at the selection row. Two missing relationship identities are malformed declarations, not conflicting identities. A malformed role beats an unresolved end type and reversed multiplicity on that relationship; an unresolved end type beats reversed multiplicity. Referencing a node refused for identity or kind keeps its own refusal rather than reporting a missing end type. With other limits sufficient, the measured successful selection-stage work admits that stage; one below its cumulative charge threshold returns `incomplete` at `/lock/model_selections/<i>` with no admitted package. This is separate from AC-171's total-reader work limit. | Test |
| FR-038-AC-169 | PLANNED / UNRUN (IR-661). `relationship_end.declaration` names the selected relationship's `relation`/`relationship` graph node and resolves through its content-only `ModelOwner`, with an empty body; a valid source or target object node used as its declaration refuses `ill_typed`/`operator-ineligible` at `member.declaration` after owner recovery and before role lookup, without invalidating that object owner. Source role lookup yields the forward end and a present target role yields the inverse end, by exact role bytes rather than the terminal identity segment. An unknown role or absent inverse refuses `missing_declaration`/`missing-name` at the member's `name` before receiver, direction or result checks; two matching roles refuse `ambiguous_declaration`/`ambiguous-name`. A wrong-kind or missing relationship owner keeps the existing owner-join refusal, with no role fallback. | Test |
| FR-038-AC-170 | PLANNED / UNRUN (IR-661). Forward and inverse navigation choose opposite receiver/destination endpoints and admit the canonical result nodes specified by QSpec FR-152 Navigation. The independently authored finite cases in TC-048 exercise Reference, Option, Set and Bag results; changing only destination type or bounds changes which result node admits. An inherited end admits on a subtype receiver with the same destination type; an unrelated object receiver refuses `ill_typed`/`operator-ineligible`. Wrong result or direction refuses `ill_typed`/`operator-ineligible` at the declared IR path. An eligible direction with an unbounded or ordered destination refuses `unsupported_construct`/`expression-form` at `member.name`, using the relationship-navigation-only variants named in AC-101. A named inverse on a source-to-target relationship refuses direction before unsupported destination multiplicity. | Test |
| FR-038-AC-171 | PLANNED / UNRUN (IR-661). For `quire.op.model.navigate`, the new relationship-end checks follow the specified operation-step order, explicit calling-node locus and RFC 6901 paths: owner recovery and declaration-kind eligibility before roles; roles before receiver, direction, multiplicity and result comparison. A missing role combined with a wrong receiver, disallowed direction and wrong result reports `missing_declaration`/`missing-name` at `.../operation/member/name`; a valid role with wrong receiver reports at `.../body/arguments/0`; a mismatched result reports at `.../body/result_type`. A successful complete read admits at its measured exact total work limit. One less returns `incomplete` at the first unpayable charge in the existing stage order, without a partial package: operation validation precedes later graph-body reference-edge charges, so the pointer may be `.../body/arguments/0/target`, not the selection row. Selection-stage-minus-one has the separate selection-row oracle of AC-168. | Test |
| FR-038-AC-172 | PLANNED / UNRUN (IR-661). A homogeneous self-relationship with a resolved forward or inverse end and an eligible Reference or Option destination admits `quire.op.model.reaches` with two references conforming to its static endpoint owner and Boolean result, including subtype operands. An unrelated operand, heterogeneous destination or otherwise ineligible Set/Bag edge refuses `ill_typed`/`operator-ineligible` at its operand or member-name path. Missing role and object-node declaration produce AC-169's respective typed outcomes; a wrong result node refuses at `body.result_type`. No operand is treated as Reference to the relationship graph node. | Test |
| FR-038-AC-173 | PLANNED / UNRUN (IR-661). Removing direction, category, composite or origin, setting each to null or a wrong type, using an unsupported direction/category value, or supplying a malformed common-schema origin branch refuses `invalid_model_binding`/`malformed-declaration` at the selection row before any application resolution, retaining code, cause and selection-row pointer under AC-167's existing refusal contract. The planned refusal retains any authentic declaration identity; a missing or malformed origin retains no origin, including both-branch and partial-branch cases. Schema-valid source and generated origins admit without a default direction; a later declaration refusal retains their valid origin exactly, with no span invented for generated origin. Retention remains PLANNED / UNRUN and does not change the existing code, cause, selection-row pointer or check order.  | Test |
| FR-038-AC-174 | PLANNED / UNRUN (refusal origin retention). Every located model-declaration refusal retains its authentic IR node identity and valid full typed Source or Generated origin through selected-document release and the public CheckedPackageRefusal boundary. A nested relationship retains its own metadata, not its owner's. Source coordinates and independently optional end members remain exact, including the existing admitted 2^53 boundary; generated input identity order/multiplicity and version text remain exact with no invented span. Missing or malformed origin retains none, never a salvaged branch. Existing admission/refusal codes, causes, row/member order and bounded accounting remain unchanged. | Test |
| FR-038-AC-175 | PLANNED / UNRUN (refusal propagation). Every refusal constructor and public reader/dispatch/typed handoff consumer explicitly preserves available declaration_identity/declaration_origin or authentic absence. A located declaration identity may not become an empty/default/inferred URI, a graph digest or an enclosing-owner identity. Pre-node byte/admission failures retain absence and existing code/path/cause/locus/contract_version/document_pointer semantics. Dropping either field, normalizing an origin, inventing a generated span or reclassifying an earlier failure must fail an independent constructor/consumer oracle; no compatibility layer or diagnostic-prose parsing supplies retention. | Test, Inspection |
| FR-038-AC-176 | PLANNED / UNRUN (IR-654 code). With `QUIRE_SPECIFICATION_DIR` naming the authoritative QSpec checkout, `make conformance-qspec` requires all nine `positive-*.json` packages under `proposals/checked-package-v2/fixtures/`: `positive-all-families.json`, `positive-clause-operations.json`, `positive-control-operations.json`, `positive-nominal-identities.json`, `positive-operation-identities.json`, `positive-recursive-records.json`, `positive-two-owners-a.json`, `positive-two-owners-b.json` and `positive-union-nodes.json`. Each admits through the production reader with its published `package_id`. For a package selecting `acme/orders`, the harness supplies `proposals/checked-package-v2/domain-package-acme-orders.json` from the same checkout under its selected `sha256-jcs` digest and the reader checks the bytes against that selection; this extends AC-107's existing positive read for packages with `model_selections`, not a second skipped read. A missing or malformed document, absent required positive fixture, unknown selected model identity, missing selection digest or non-admitted package fails the run. For the two-owner fixtures, read each node id and owner back through the admitted package's graph accessor and compare with the published graph and identity-projection entries: the corresponding declared structural nodes have different ids across source owners, as AC-153 requires. This checks published wire/admission, not source-owner key re-derivation, which remains IR-630. For every `model_declaration_nodes[*].preimage` in `proposals/checked-package-v2/model-member-type-vectors.json`, compare its `sha256` with the digest from the reader's production `declaration_key` path (`StructuralPreimage`/`structural_key` in `model_members.rs`), or admit its published wire node through that same production derivation; a harness that only hashes the vector's JSON preimage cannot pass. `node-identity-vectors.json` in that proposal remains the separate nominal/application-vector source; `model-effective-declaration-vectors.json` describes another model artifact and is not the structural-key oracle. No fixture document, source pin or digest catalog is copied into this repository. Live QSL emission followed by IR reading belongs to integration work outside this criterion. | Test (TC-048) |
| FR-038-AC-177 | PLANNED / UNRUN. An admitted structural.eq over supported parameter references returns the actual application and supplied occurrence with two typed operands in argument order, ordinals 0 and 1. Swapping arguments swaps entries; repeating a parameter retains two entries. A second authentic occurrence is retained exactly; an absent occurrence refuses MissingOccurrence with the supplied application and occurrence. | Test |
| FR-038-AC-178 | PLANNED / UNRUN. A supported closed graph composite literal retains its own GraphChild id and Literal disposition with no introduced positions even under a wider type. A value subtree reading a parameter refuses UnsupportedOperand with enclosing ordinal and reason NonliteralGraphValue; an application subterm refuses reason ApplicationSubterm. Defensive inline integer and noninteger term cases refuse reasons InlineInteger and InlineNonInteger, without fabricated identity or value. No publicly admitted structural.eq integer-inline success is claimed: the owning catalog excludes Integer from structural_kind; future success requires an owning eligibility/value specification. | Test |
| FR-038-AC-179 | PLANNED / UNRUN. A record containing bounded Sequence of integer-range elements returns its exact field/collection/element paths, Node keys rooted at the operand, and both collection endpoints. Changing only one path's source type changes that descriptor. Bounds beyond i128/u64 retain canonical decimal text; unbounded integer/collection have explicit unbounded descriptors without guessed maxima. | Test |
| FR-038-AC-180 | PLANNED / UNRUN. Tuple, direct Option and optional-presence record edges preserve the specified ordinals without wrapper segments; List with optional tail has Depth key [] and reentry [1,0]. Tree with two optional self-fields retains both [0,0] and [1,0] at Depth key []; shared nonrecursive siblings retain distinct paths without Depth. Boolean introduces no position. Enum retains ordered flag and exact preimage member order, including unordered Blue, Green, Red. Each of the eleven scalar and seven bounded-domain forms produces its specified descriptor or absence; text_bounds produces one Whole at the wrapper, and population-typed leaves produce Node-keyed Whole without Population key. The complete returned introduced-key set equals an independent enumeration. | Test |
| FR-038-AC-181 | PLANNED / UNRUN. Every one of the eleven specified error variants exposes exactly its tabled payload and no partial result or panic. Unsupported union, malformed optional wrapper and unanchored cycle carry the actual enclosing ordinal/type locus. Zero and one below measured successful work refuse WorkLimit; that exact budget succeeds. Checked-index failure returns PositionOutOfRange with authentic available ordinal/type metadata. Equivalent package node order and lookup strategy leave logical consumed_work unchanged; an earlier operand refusal or exhausted charge beats a later defect. | Test |
| FR-038-AC-182 | PLANNED / UNRUN. An external consumer calls the named public accessor and exhaustively matches all named output, descriptor, edge, key and error types using the specified fields without body JSON or a QSL model dependency. Repeated/clone calls yield equal content and unchanged package equality. API inspection finds no harness-bound inputs, coverage verdict or obligation encoder; structural.ne remains a declared consumer gap rather than a silently enabled operation. | Test, Inspection |
| FR-038-AC-183 | PLANNED / UNRUN (IR-680 code). For an unreferenced owner-free, ungrouped node in each of the `derived_key` closed-shape and `UngroupedPreimage` branches, a retained node id that differs from the production-derived key refuses `invalid_package`/`stale-node-key` at the original `node_id`, keeps that original typed id in `locus`, and returns the distinct derived typed key through `expected_node_id()`. A fresh package whose unreferenced node is rekeyed to that value and whose package identity is refreshed admits; one changed preimage member changes the expected digest. For each of the eight self-typed forms, a changed `node_id` can yield a key only when its `semantic_type` changes with it; a stale `semantic_type`, a non-self-typed shape with inconsistent semantic type, a `collection_bounds` over a noncollection type, and a malformed closed shape yield `None`. AC-185 governs shared retention and absence. | Test (TC-048, TC-226) |
| FR-038-AC-184 | PLANNED / UNRUN (IR-680 code). An unreferenced application node whose retained key differs from the digest already computed by `validate_application_keys` refuses `invalid_package`/`stale-node-key` at its original `node_id`; `locus` remains that original typed id and `expected_node_id()` returns the distinct computed typed key. A fresh package rekeyed to that value, with package identity refreshed and no referrer to cascade stale keys, admits; changing one application preimage member changes the expected digest. The value comes from the existing application computation, not a second hash or a diagnostic string. AC-185 governs shared retention and absence. | Test (TC-048) |
| FR-038-AC-185 | PLANNED / UNRUN (IR-680 code). The expected key is a private, bounded `Option<CheckedNodeId>` exposed only by `expected_node_id() -> Option<&CheckedNodeId>`; it participates in derived `Clone`/`Eq`/`PartialEq`, may appear in derived `Debug`, and is absent from Display, document pointer, locus text and serialized forms. A fixed-member model declaration shape failure, a canonical encoding failure and a pre-key-stage refusal return `None`; every IR refusal constructor and public V2/dispatch forwarder preserves an authentic value or absence, and IR's independent test-authored expected values check the actual reader field by field. Existing code, cause, path, locus, first-refusal order, byte/work charges and public key-derivation surface remain unchanged; no public synthetic-refusal constructor or derive/rekey API is added. The key is meaningful only with the reader-returned original `locus`, since callers can mutate public fields on a cloned refusal. | Test, Inspection (TC-048) |

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
QSpec FR-340 `modifies` entry shape, the FR-341 state clause and FR-342
operation anchor bodies, and the `model` form set.
