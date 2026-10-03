---
id: FR-038
title: "Consume the CheckedPackage V2 contract and refuse every other version"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/AD-003
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
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
  - target: ix://agent-ix/quire-contract-ir/FR-040
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

- The depth limit at the first value, in document order, nested one level
  past it.
- The node limit at the first node past it.
- The edge limit at the dependency that took the count past it.
- The occurrence limit at the source-map entry or region that did.
- The diagnostic limit at the first entry past it.
- The work limit at the value whose validation took the meter past it: a
  node body, a nominal preimage member, an `operation`, a graph edge's
  member, or a diagnostic detail.

### Reading

The reader shall measure raw bytes against the byte limit, read the document
as strict JSON (duplicate members refuse, nesting charged against the depth
limit, a document at the limit admitted), require canonical bytes, and read
`contract_version` exactly once. Depth counts each container as one level and a
scalar value as one level below its container, so `[]` is depth 1 and `[1]`
and `{"a":1}` are depth 2. A syntax or duplicate-member defect anywhere in the
document refuses before depth is charged, however deep the document is, and the
JSON parser's own nesting cap never decides the outcome: a document deeper than
the depth limit returns `incomplete` for `depth` with that limit and the
measured depth, and one within it is read. The limit is the caller's up to
16,384, the most nesting the reader reads; a caller limit above that reads as
16,384. This ceiling is a known deviation from FR-322 and FR-038-AC-3, which
charge the caller's limit as given, pending the upstream amendment tracked as
STD-125; it is not a settled rule. An admitted package's own clone,
comparison, `Debug` rendering and lowering, and dropping any of its nodes,
projections or diagnostics, are safe at any depth the reader admits. The graph,
lock and diagnostics its accessors return, and the records and package a
lowering returns, are ordinary data whose derived traits recurse on the
caller's stack, so a caller who raises the depth limit past the default of 128
owns the stack they need. It shall
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
   with a `document_pointer` when the document holds a number whose text denotes a magnitude past 2^53 (this
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
such a member, because a node `body` and a diagnostic `details` entry nest as
deep as their input. The target is that `quire-canonical` provides the iterative
`Encode` for `serde_json::Value`, behind a `serde_json` feature, as the one
owner for this repository, `quire-contract-codegen` and `quire-spec-language`,
and that this repository's `Encode` types delegate their `Value` members to it.
That upstream change is quire-canonical #7, open and pending merge, and the code
change that depends on it is blocked until it merges. The same change adds a
public non-recursive drop helper for a deep `serde_json::Value`, because
`serde_json`'s own drop recurses and this reader parses with unbounded depth;
the reader drops a deep `Value` through that helper and writes no drop of its own. Until then `Value` members encode through this
repository's own walker, `value_to_vec` in `checked_package/v2/encode.rs`, which
stays; this requirement does not specify removing it before the upstream
`Encode` exists. The assignment follows from the types:

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
a `Value` (this repository's walker until `quire-canonical`'s `Encode` for
`serde_json::Value` lands, then that `Encode`).

Once the upstream `Encode` lands, this repository holds no walker over a
`Value`: the iterative encode of a `Value`, its number rule (an integer past
2^53 is refused, any other number is its `f64`) and its member order belong to
`quire-canonical`. Until then the walker is the one encoder of a `Value` and
FR-038-AC-77 and FR-038-AC-78 are verified against it. Encoding
adds no depth limit of its own: a body nested deeper than the reader admits,
built in memory, still encodes, and the only refusals an encode returns are the
byte ceiling and a number `quire-canonical` has no encoding for. The byte
ceiling is the read's byte limit.

The canonical bytes are unchanged for every value that the reader admitted
before this requirement and admits after it. For every V2 type above the bytes
`quire-canonical` produces equal the bytes of the type's `serde_json` canonical
form (`to_value` then `to_vec`, members in sorted order) for every in-repo
fixture and for the crafted values of FR-038-AC-74 and FR-038-AC-75 (strings
with non-ASCII and astral characters, and the integers 9007199254740992 and
-9007199254740992), and `package_id` is the SHA-256 of exactly those bytes
with no domain label hashed in, so every `package_id` already recorded in a
fixture still recomputes. The same holds for every node key and for the lowered
package: each application, structural and nominal node key recorded in a fixture
still recomputes, and a lowered node's `ir_id` and the lowered package's bytes
and `package_id` are the ones the lowering produced before the move
(FR-038-AC-89). Lowering output and the QSL output that consumes these types are
unchanged. This statement is about V2 only: the v1 model's canonical objects
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

### Canonical bytes are quire-canonical's bytes

This requirement changes what the reader accepts as canonical bytes, and gives
each input it changes one code. A document is canonical when its bytes equal `quire-canonical`'s bytes for
the value read, and the reader makes that check through `quire-canonical`,
without recursing over the document, in the place it checks canonical bytes
today: after the strict syntax, duplicate-member and depth checks and before the
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

The order of checks is: strict syntax, duplicate member and depth; canonical
bytes; closed-schema decode and header; `package_id` recomputation; then the
grammar and graph checks. The `package_id` recomputation therefore runs on a
value every number of which `quire-canonical` encodes, and a more specific
grammar refusal is never decided by an encoder refusal. An encode refusal at the
recomputation is not reachable from a document that passed intake; a caller who
meets one (an in-memory value) receives `quire-canonical`'s error.

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
`document_pointer`, the pointer of the first such number in document order
(`/package/count` for a member `count` of the object `package` at the document's
root). It is decided at the place the digest comparison is: after the supplied
document is read and before any digest is computed, so it precedes the
`byte-digest-mismatch` and the identity refusals of the same row. The document is
read and encoded under the reader's byte limit: bytes that exceed it return
`incomplete` for `bytes` with that limit and the document's length as consumed
and no pointer, as every byte-limit outcome is (FR-038-AC-26); the `work` limit
charged for the same document is located at the row (FR-038-AC-30), and the two
are different limits with different locations.

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

Every ancestor edge followed, member visited and redefinition pair compared
in step 3, the conformance walk of `conforming_reference`, and the parse and
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
  lower-bounded operator) or `null` (an unbounded operator); a `null` member is
  not an interval and refuses as a member of the wrong kind. Each of `lower` and
  `upper` is an integer string, the schema's `IntegerString`, which matches
  `^(0|-?[1-9][0-9]*)$` exactly: `"0"`, `"3"`, `"10"` and `"-1"` match, `"1.5"`,
  `"01"`, `"+1"`, `""` and `"3x"` do not. QSpec FR-255 and FR-091 require integer
  bounds with `0 <= a`. A QSpec owner ruling relayed via the IR planner, not yet in
  merged QSpec text (so an IR reading until it merges), is that QSpec's schema
  tightens the bound pattern to a non-negative integer and that the reader refuses
  a negative `lower` or `upper` as `invalid_package` with cause `invalid-value`,
  the cause of `lower > upper`, in the term walk of the node body (the walk that
  validates each body term's closed shape, which runs after the strict parse, the
  canonical-bytes check and the decode and before the graph's identity checks, the
  temporal step and the operation step; this is the stage that is "schema
  validation" for this rule, and the term walk gains this one check on a
  `temporal_interval` member), under every profile and whatever the
  member's other defects of the temporal step, first in member order (`lower`, then
  `upper`), located at the bound
  (`/semantic_graph/nodes/{n}/body/operation/member/interval/lower` or `.../upper`).
  A negative bound is read as an integer, so it is not the shape refusal below. The
  asymmetry is deliberate: `"-1"` matches the integer-string grammar and is refused
  `invalid-value` first, in the term walk, while `"1.5"` does not match it and is
  refused `operation-member-mismatch` later, at the operation step. A
  member of another kind, a `null`
  member, an `interval` with another member set, or a `lower` or `upper` that is
  not an integer string refuses `invalid_package` with cause
  `operation-member-mismatch` at `operation.member`. A closed interval whose
  `lower` is greater than its `upper` refuses `invalid_package` with cause
  `invalid-value` (QSpec FR-370: at the application, here at the application's
  body root, `/semantic_graph/nodes/{n}/body`). The two bounds are compared as
  integers of unbounded size, by sign, then digit count, then digits, never as
  strings and never through a fixed-width integer, so `{9, 10}` admits, `{10, 9}`
  refuses and a pair beyond 2^64 compares as its value does. On the other seven
  `temporal_formula` entries and on `quire.op.temporal.clause`, any member
  refuses `operation-member-mismatch` at `operation.member`.
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
node body does (confirmed relayed ruling, pending QSpec merge). A union value node's
`semantic_type` is a union type node and its body is one `binding` naming a
member of that union over exactly the member's payload terms, each of the
member's payload type; a binding naming no member, a wrong payload count or a
payload of another type refuses `ill_typed` with cause `type-mismatch` at the
node (QSpec FR-440 reader joins 1 and 2). Neither `duplicate-member` nor
`type-mismatch` is a cause of the reader's refusal types today, and the code
change adds both with the pairings FR-440 states and no other. A QSpec owner
ruling, relayed via the IR planner and not yet in merged QSpec text, is that
QSpec's V2 schema gains `type-mismatch` in its `cause_tag` enumeration and in its
cause-to-code pairing with `ill_typed`; until that merges this is an IR reading of
FR-440's own text, which names the cause, and the reader states `type-mismatch`
as the cause. The operand family of a
`composite_type`/`union` node is `union`.
`quire.op.structural.eq` and `quire.op.structural.ne` over two values of one union
admit and over values of two different unions refuse `ill_typed`/
`operator-ineligible` by their `same_type` constraint (QSpec FR-440-AC-6). The
leaves of a comparison over a union follow a QSpec owner ruling relayed via the
IR planner and not yet in merged QSpec text, so an IR reading until a QSpec PR
lands: FR-322's leaf segments gain `member:<Ident>`, naming the union member; a
`structural.eq` or `structural.ne` leaf path through a union is `member:<Name>`
followed by `position:i`, `i` the index within that member's ordered payload even
for a single payload, and then the `field:`, `position:` and `inner` segments into
the payload type, with one leaf per member whose payload reaches a `text` type,
members in declaration order and none for a payload-less member. A comparison over a union whose payloads reach
no `text` type admits with `leaves` empty, as for any compared type that reaches
none. Until that text merges the leaf grammar is not cited by id. A union member key (QSpec FR-441) is
derived by a reader and never carried; this reader derives none, since it selects
no arm. An `expression`/`case` node is keyed by the application-node preimage
like every application node, and the other three forms carry the `node_id` every
node carries.

**The temporal step.** QSpec FR-370 "Reader order" puts a temporal step after
the state step (FR-040) and before any operation refusal, and this reader runs it
there, replacing the earlier placement of these applications in the operation
step and the term walk. It reports the first defect of this order. Placement
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
   an application of these classes as the body root of a node of another form or
   nested as an element of another application's `arguments`, in a `binding`
   value or in an `aggregate`. A `temporal`/`formula` node is referenced only from
   a clause's formula argument or as an operand of a `temporal_formula`
   application, and a `temporal`/`fairness` node only from a clause's fairness
   argument (QSpec FR-370 "Placement and profile fit"); a `reference` to one from
   any other term refuses. Each refuses `ill_typed` with cause
   `operator-ineligible`, located at the node that holds the misplaced
   application or, for a reference, at the referencing node. QSpec FR-370 states
   "at the node that holds them" of applications only; locating a misplaced
   reference at the referencing node is a QSpec owner ruling relayed via the IR
   planner, not yet in merged QSpec text, and an IR reading until it merges.
2. **Each clause** (the clause checks, in this order for one clause):
   - **Profile identity** (first, "Unknown profile" below).
   - **`over`.** Its `over` argument is a `reference` to a node that is among the
     clause node's declared `dependencies` and is a `value`/`parameter` node, else
     `missing_declaration`/`missing-name` for a target that is no declared
     dependency and `invalid_model_binding`/`malformed-declaration` for a declared
     dependency of another tag or form, the pairing a frame entry's reference has
     (FR-040).
   - **Fairness resolution.** For each fairness member reached through the
     clause's fairness argument, the `declaration` names a `model`/`object_type`
     declaration node, else `invalid_model_binding`/`malformed-declaration` with path
     `/semantic_graph/nodes/{fairness node}/body/operation/member/declaration` and
     the `declaration` target's key as locus ("Path and locus" below; as FR-342
     step 1 refuses an anchor's context; a QSpec owner ruling
     relayed via the IR planner, pending QSpec merge, replacing the earlier reading
     that any `model`-tag node would do), and the `name` resolves among that node's exposed operation members, own
     and inherited, with redefinitions applied, as an operation anchor's
     `operation` name does (FR-040 "Operation anchor body"), except that an
     operation the node only inherits is admitted. QSpec FR-370 states only
     `missing_declaration`/`missing-name`; the other outcomes are QSpec owner
     rulings relayed via the IR planner, not yet in merged QSpec text, and IR
     readings until they merge: a `declaration` that names no node, and a `name` that
     no effective operation matches, refuse
     `missing_declaration`/`missing-name`; two or more matches refuse
     `ambiguous_declaration`/`ambiguous-name` (`ambiguous-name` pairs with
     `ambiguous_declaration`); an inherited-only operation is a valid fairness member
     and resolves to the most-derived redefinition, since fairness follows QSpec
     FR-362's effective operation and FR-040's inherited-only refusal is specific to
     an operation anchor; and a declaring node whose owner is not recovered refuses
     with that resolution's own refusal (`missing_declaration`/`missing-selection`
     or `invalid_package`/`stale-node-key`), as an operation anchor's unrecovered
     owner does.
   - **Profile fit** (below).
   - **Interval bounds**, the `lower > upper` refusal of the member rule above,
     which runs after profile fit. The refusal is a property of the member, not of
     a clause, so a formula node no clause reaches is checked too, in a sweep after
     every clause has been read, in ascending `node_id` digest order (an IR reading;
     a formula's interval is an interval all the same). A negative bound is not part of this stage: it is
     a schema-pattern failure, refused in the term walk of the body before any
     temporal step, so `{lower: "-1", upper: null}` under a bounded profile refuses
     `invalid-value` at `lower` and never `operation-member-mismatch`, and
     `{lower: "0", upper: "-2"}` refuses at `upper` as a negative bound, not as
     `lower > upper` (a relayed ruling, pending the QSpec change that tightens the
     pattern).

   The temporal step reads only a well-formed member: an interval member whose
   shape the member rules above refuse (a `null` member, a member of another kind,
   a non-integer bound such as `{lower: "1.5", upper: "0"}`) and a fairness member
   that is not the closed object are skipped by profile fit and bounds, and the
   operation step then refuses them `invalid_package`/`operation-member-mismatch`
   at `operation.member`; so `{lower: "1.5", upper: "0"}` refuses
   `operation-member-mismatch` and never `invalid-value`, whatever the profile.
3. **Profile fit.** The clause's one `temporal_profile` law names the profile.
   The known profiles are the five members of QSpec FR-250's Values table
   (merged text): `quire.temporal.event-position.false-extension/v1`,
   `quire.temporal.fixed-sample.false-extension/v1`,
   `quire.temporal.timestamped-event.finite-window/v1`,
   `quire.temporal.infinite-trace/v1` and `quire.temporal.timed/v1`. The fit of the
   interval members per profile below is a QSpec owner ruling relayed via the IR
   planner, not yet in merged QSpec text, and an IR reading until it merges (QSpec
   FR-370-AC-3 states only the bounded rule: a `null` interval or a `null` `upper`
   under a bounded profile refuses `operation-member-mismatch`):
   - `quire.temporal.infinite-trace/v1` admits `interval: null`, `{lower, upper:
     null}` and `{lower, upper}`, and any number of fairness references.
   - `quire.temporal.timed/v1` admits `interval: null` (an unbounded operator, which
     QSpec FR-250-AC-6 admits under this profile) and `{lower, upper}`, and refuses
     `{lower, upper: null}` because its upper bound is finite (QSpec FR-255). Timed
     bounds are exact rationals with open or closed ends, which V2's
     integer-string `temporal_interval` cannot carry: a timed bound that is not an
     integer string, and an open end, are pending the QSpec change that adds that
     form, and this specification neither invents it nor pins an outcome for them.
     Any number of fairness references is admitted, as under the infinite-trace
     profile (relayed ruling, pending QSpec merge).
   - The three bounded profiles (`event-position.false-extension`,
     `fixed-sample.false-extension`, `timestamped-event.finite-window`) admit
     `{lower, upper}` only: a `null` interval or a `{lower, upper: null}` interval
     refuses `invalid_package` with cause `operation-member-mismatch` at the
     operator's application, and a non-empty fairness argument refuses the same
     way at the clause's application.

   **Unknown profile.** The clause's `temporal_profile` law, checked before `over`,
   must name one of the five FR-250 members. A law whose `definition` is not one
   refuses `unknown_profile`, with the pairings of merged QSpec FR-272, each located
   at the clause application's `/semantic_graph/nodes/{n}/body/operation/laws/0/definition`.
   The locus and the place of this check first among a clause's checks (the
   profile decides interval fit) are a QSpec owner ruling relayed via the IR
   planner, not yet in merged QSpec text.

   **Cause.** A QSpec owner ruling relayed via the IR planner, not yet in merged
   QSpec text: the cause is `wrong-selection-role` when that same `definition`
   (`{authority, identity}`) appears in the package's own `lock.profile_selections`
   under a `role` other than `temporal_profile` (the roles `language`, `edition`,
   `profile`, `binding_contract` and `protocol_profile`, the other values of a
   row's `role`, and the `lock.edition` row), and `unsupported-selection`
   otherwise. The rule is decidable from the package alone: this reader holds no
   catalog and no identity list beyond FR-250's five members, so an identity the
   package does not select under another role, such as the in-repo fixtures'
   `quire.fixture.temporal-profile/v1`, refuses `unsupported-selection`, and
   `quire.protocol.complete/v1` or `quire.package.composed/v1` refuses
   `wrong-selection-role` only when the package itself selects it in another role.
   A known profile is matched by its FR-250 identity label, an exact string
   (QSpec FR-250 "Identity and comparison"; the label is the key); the
   `authority` of the definition reference takes no part, so a law with a member's
   identity and a different `authority` is still a known profile and later refuses
   `operation-law-unselected` at FR-038-AC-57's join (a confirmed relayed ruling,
   pending QSpec merge). This check precedes FR-038-AC-57's join of the law to the lock's
   `temporal_profile` row, which still refuses `operation-law-unselected` for a
   known FR-250 member that is not the selected one. An unknown identity is never read as bounded or as unbounded.
   `unknown_profile` is a code of QSpec FR-271's diagnostic code vocabulary and not
   yet a variant of the reader's `CheckedPackageRefusalCode`, and neither
   `unsupported-selection` nor `wrong-selection-role` is a variant of its
   `CheckedPackageRefusalCause` today (measured): the code change adds the code
   `UnknownProfile` and both causes, with these pairings and no other. A formula tree is the set of `temporal`/`formula`
   nodes reachable from the clause's formula argument by operand references.

**Path and locus.** Every refusal of the temporal step carries both a path (an
RFC 6901 pointer) and a locus (a node key), as the frame-entry refusals do (FR-040:
the path is on the referencing member, the locus is the declaring node's key; FR-342
step 1: "the locus is the target"). "At the node" and "at the application" above name
a path (`/semantic_graph/nodes/{n}` and its `/body`); unless a row below says
otherwise, the locus is the key of the node that path is on.

| Refusal | Path | Locus |
| --- | --- | --- |
| placement, misplaced application | `/semantic_graph/nodes/{holder}` | the holder's key |
| placement, misplaced reference | `/semantic_graph/nodes/{referencing node}` | the referencing node's key |
| `unknown_profile` | `/semantic_graph/nodes/{clause}/body/operation/laws/0/definition` | the clause's key |
| `over` not a declared dependency, no node (`missing-name`) | `/semantic_graph/nodes/{clause}/body/arguments/0` | the clause's key when it names no node, otherwise the target's key |
| `over` not a `value`/`parameter` (`malformed-declaration`) | `/semantic_graph/nodes/{clause}/body/arguments/0` | the target's key |
| fairness `declaration` names no node (`missing-name`) | `/semantic_graph/nodes/{fairness node}/body/operation/member/declaration` | the fairness node's key |
| fairness `declaration` not a `model`/`object_type` node (`malformed-declaration`) | `/semantic_graph/nodes/{fairness node}/body/operation/member/declaration` | the `declaration` target's key (`member.declaration`) |
| fairness `name` matches no effective operation (`missing-name`) or two or more (`ambiguous-name`) | `/semantic_graph/nodes/{fairness node}/body/operation/member/name` | the `declaration` target's key |
| fairness declaring node's owner not recovered | `/semantic_graph/nodes/{fairness node}/body/operation/member/declaration` | the fairness node's key (the calling node, as FR-038 states for a model-owner refusal) |
| profile fit, interval | `/semantic_graph/nodes/{formula node}/body` | the formula node's key |
| profile fit, fairness argument | `/semantic_graph/nodes/{clause}/body` | the clause's key |
| `lower > upper` | `/semantic_graph/nodes/{formula node}/body` | the formula node's key |
| negative bound | `/semantic_graph/nodes/{formula node}/body/operation/member/interval/lower` or `.../upper` | the formula node's key |

The paths AC-97, AC-102, AC-103, AC-104 and AC-108 give, and the loci they name,
are the rows of this table. The table is an IR reading of how path and locus are
paired, pending QSpec (QSpec PR #181 is silent on loci).

Two rows are a deliberate departure from FR-040, which makes the locus the entry's
declaring node key (the key the entry names, as `frame.rs` carries it): an `over` that
names no node and a fairness `declaration` that names no node put the locus on the
holder (the clause, the fairness node) rather than on the named key, because a key
that names no node cannot locate a node in the graph. Every other row that names a
target uses the target's key, as FR-040 does.

**QSpec PR #181 (draft, not merged; head 454bb057c32c73b472bd83cb4d94b2f77bf8c8ce)**
carries drafts of several of the relayed rulings marked pending above: the
`type-mismatch` cause and its `ill_typed` pairing, the `member:<Ident>` leaf segment, a
non-negative integer bound pattern with a negative bound refused `invalid_package`/
`invalid-value` before any FR-370 step, a rational timed interval form with open and
closed ends, the `unsupported-selection` and `wrong-selection-role` causes with their
`unknown_profile` pairing, and the cause rulings of its description for profile fit,
fairness resolution and placement. IR's text keeps every pending mark until that PR
merges, and nothing here cites it as merged.

Operand family and count and `result_type` stay the operation step's
(FR-370-AC-7), so `holds` over a `reference` to a `temporal`/`formula` node
refuses `ill_typed`/`operator-ineligible` there, after the temporal step, and an
identity the catalog does not hold or an `operator` that differs from the
entry's class still refuses `unknown-operation` or `operation-class-mismatch`
there, never ahead of a placement defect. A `case` application is not temporal.
QSpec FR-440 requires the producer to encode every `case` expression as an
`expression`/`case` node and the schema binds that form's body to a `case`
application, and a QSpec owner ruling relayed via the IR planner, not yet in merged
QSpec text and an IR reading until it merges, is that a `case` application stands
only at that node root: a `case` application as the body root of a
node of another form, or nested in a term, refuses `ill_typed`/`operator-ineligible`
at the node that holds it, and an `expression`/`case` node whose body is not a
`case` application refuses the same way, all in the one placement pass of the
temporal step: one pass over the graph covers all four classes, `case` included, in
ascending `node_id` digest order (confirmed, relayed, pending QSpec merge). The nested case is also what this reader
could not otherwise check, since it does not operation-check a nested application
(the stated deviation of "Operation identity" above); that is a second reason for
the nested position only, not the basis of the body-root one. An application of any
of the four classes in a `diagnostics.entries[].details[]` term, as its root or
nested, is no node's body and refuses the same way at that application's `operator`
(`/diagnostics/entries/{i}/details/{j}/operator`), likewise a QSpec owner ruling
relayed via the IR planner and not yet in merged QSpec text.

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
here). The reader's refusal types, `CheckedPackageRefusalCode::UnsupportedConstruct`
and `CheckedPackageRefusalCause::ExpressionForm`, are removed, since nothing
raises them; the diagnostics wire vocabulary `CheckedDiagnosticCode::
UnsupportedConstruct` stays, because QSpec's V2 schema enumerates
`unsupported_construct` as a diagnostics `code`.

**A breaking change for consumers, shipped inside the held branch.** The four new
node forms, the removed refusal code `UnsupportedConstruct` and the new public
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

A node whose body contains an `application` term at any depth has
`dependencies` exactly the unique, digest-ascending reference targets and
operation member declarations of its body (FR-322
`application_node_preimage`); a `result_type` or literal `type` is not a
dependency. The join applies to every node whose body contains an
application, not only to a node whose body root is one: that is the set of
nodes QSL's emitter writes the join for. A member `declaration` that is not
a node key is left to the operation stage's refusal.

Any violation refuses as `invalid_semantic_graph` at the member that breaks
the rule (`semantic_graph.nodes.body`, `.dependencies` or `.semantic_type`),
located at the offending node, at the first such node in ascending node-id
digest order. A `declaration` on either form refuses as the declaration rule
above does. These are graph-shape refusals: they precede the stale-key
stage. The reader re-derives neither form's node key: QSL keys both by its
proposed `quire.structural-node/v1` preimage, which QSpec does not publish.

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
application term. QSpec FR-322 keys every node whose body contains an
application. The reader re-derives a key only where the body root is the
application, so a nested application's own node is not re-keyed: a stated
deviation from QSpec FR-322, not a settled rule. This section fixes what the
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
`operation-mode-type-mismatch`, so it reverses the last pair, and it checks an
application only where it is a node's body root, where QSpec checks each argument
application as well (a nested application is refused by the term walk only for
the four node-root operator classes, `temporal`, `temporal_formula`,
`temporal_fairness` and `case`, at their placement rule, "Catalog words:
admitted, read and catalogued, never evaluated"). Both are stated deviations from QSpec FR-322, not settled rules, and
no criterion below fixes the order of the reversed pair.

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
- **Operand families.** A `literal` argument resolves to its declared `type` and a
  nested `application` argument to its `result_type`, as a `reference` resolves
  to its target, and each takes part in the operand-family and mode-pin checks. An
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
anchors in the walk as a record's field or a tuple's position does (a QSpec owner
ruling relayed via the IR planner, not yet in merged QSpec text, against reading a
cycle through a union as `ill_typed`/`operator-ineligible`: recursive unions such
as `List = Cons(Integer, List) | Nil` are core, and a union cycle counts in the
reader's visited set as IR-506 holds). The path of an edge into a union member's
payload is `member:<Name>` then `position:i`, `i` the index within the member's
ordered payload even for a single payload, and a reentry into an open union is a
reentry like any other: a cycle through a union is never record-free and never
refused for it, and expects one recursion leaf where a `text` type is reachable,
`d` counted as for a record.

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
literal, an aggregate or a nested application with no `result_type`), and a
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
`state`/`frame` node's `body` against this shape alone and every other node's
`body` against the semantic-term grammar alone. A frame body that omits one of
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

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-038-AC-1 | Each positive V2 package fixture this repository builds from its own public vocabulary admits; an unknown, empty, absent or malformed `contract_version` refuses as `unknown_contract_version` or `malformed_wire` before any version-specific decoding; and the strict parse (duplicate-member, noncanonical) refusals occur before a version is selected. | Test (TC-048) |
| FR-038-AC-2 | The reader refuses malformed, duplicate-member, unknown-member, noncanonical, stale-dependency, cross-domain digest, unknown required capability, unsupported node tag, invalid graph and invalid source-map inputs with exactly those codes, and every authored adverse structural mutation returns its declared outcome, before exposing a package; a domain package selection or model owner carrying `authority`, `revision`, `export` or `version` refuses as `unknown_member`, a domain package selection outside `sha256-jcs` as `digest_domain_mismatch`, one with no supplied document as `missing_import` with cause `missing-selection`, and a `model_export` semantic form as `invalid_semantic_graph`. | Test (TC-048) |
| FR-038-AC-3 | Exact byte, depth, node, edge, occurrence, diagnostic and work limits admit a package; each one-over limit returns `incomplete` with that limit kind, the limit and the consumed counter and no package. | Test (TC-048) |
| FR-038-AC-4 | The recomputed package id equals each positive fixture's declared id; editing a source-map region, occurrence, raw source digest or capability disposition leaves it unchanged, while editing the edition, a selection, a required feature or a node projection changes it and refuses unless mirrored. | Test (TC-048) |
| FR-038-AC-5 | Every nominal preimage of the in-repo nominal fixture re-derives the node key it is keyed by and round-trips its own wire form; every authored invalid nominal mutation, an absent or wrong preimage, and each retained-preimage change of enum case, `semantic_type`, dependency or unit target refuses as `invalid_semantic_graph`; a model owner admits when its identity names a selected domain package and refuses as `invalid_semantic_graph` when it names none or carries an empty node. | Test (TC-048) |
| FR-038-AC-6 | Every admitted node family lowers independently with exact source, type, dependency, bound and claim correspondence; missing, unsupported, unbounded and over-work requests return `invalid_input`, `unsupported`, `requires_bound` and `failed` without a node and without changing sibling records. | Test (TC-050) |
| FR-038-AC-9 | The shipped default read-limit policy is exactly those seven finite values, every member is strictly positive and finite, and each meter is enforced at its own true measured boundary against a real package: the package's exact measured consumption for that meter admits it, and one below that exact value refuses it as `incomplete`, naming that meter and reporting the true consumption. The shipped default is far larger than any fixture, so this boundary is proven against each meter's real measured cost rather than against the default value itself; no fixture approaches that scale, and none is fabricated to do so. | Test (TC-048) |
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
| FR-038-AC-26 | Each one-over limit other than the byte limit returns `incomplete` carrying the RFC 6901 pointer of the value whose charge failed, which resolves in the document read: depth at the first value nested one level past it, nodes at the first node past it, edges at the dependency and occurrences at the source-map entry or region that took the count past it, diagnostics at the first entry past it, and work at the value whose validation took the meter past it; the byte limit carries none. | Test (TC-048) |
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
| FR-038-AC-67 | An unknown identity refuses `unknown-operation` and an `operator` that differs from the catalogued class refuses `operation-class-mismatch`, each ahead of every other check of the entry at the operation step, so `quire.op.control.case` under the operator `unary` refuses `operation-class-mismatch`; and of two defective body-root nodes the one with the lower `node_id` digest is reported when both defects are operation refusals. A placement defect of the temporal step (FR-038-AC-102) is reported ahead of every operation refusal, whatever the digest order. (Amended by IR-549: the earlier text ordered both refusals ahead of `unsupported_construct`, which the reader no longer has, and put the placement of these applications inside the operation step.) | Test (TC-048) |
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
| FR-038-AC-78 | The encodes of FR-038-AC-77, run under a byte ceiling of the encoded text's exact length, return the bytes; run under a ceiling one byte lower, return the byte-limit error with no bytes; a body nested 20000 levels deep, past both the reader's default depth limit of 128 and its 16,384 ceiling, encodes and is not refused for its depth; and a body holding the integer 9007199254740993 returns the encoder's refusal naming that value with no bytes. | Test (TC-048) |
| FR-038-AC-79 | A package document holding the integer 9007199254740993 in a node body refuses `noncanonical_wire` with no pointer, as does one holding -9007199254740993, one holding the float `2.0` in a body, and one whose body object lists a member named with U+E000 before one named with U+10000 (UTF-8 byte order), each refused before any grammar, `package_id` or graph refusal the same document also earns; the same document with 9007199254740992, with `2` in place of `2.0`, and with U+10000 before U+E000 (UTF-16 code-unit order) is not refused `noncanonical_wire`. | Test (TC-048) |
| FR-038-AC-80 | `quire-canonical` is a `branch = "main"` git dependency of this repository's manifests with its source in the `allow-git` list of `deny.toml`, and `make deny` passes; `CheckedSemanticId`, `CheckedSourceMapEntry`, `CheckedCapability` and `CheckedPackageLockV2`, and every type the three `Encode` types and these four hold that does not itself hold a `Value` (`CheckedOccurrence`, `CheckedDeclaration`, `NominalIdentityPreimage`, its four preimage structs, `NominalOwner`, `DimensionTerm`, `CheckedRational`, `CheckedDiagnosticStage`, `CheckedDiagnosticCode`, `CheckedDiagnosticCause`, the node identity, selection, domain-package, dependency-selection, source-region types, `CheckedArtifactRef` and `CheckedSourceRef`), derive `FixedShape`; `CheckedPackageIdentityPreimageV2`, `CheckedSemanticGraphV2` and `CheckedDiagnosticsV2` implement `quire_canonical::Encode`, and they and the types that hold a `Value` do not implement `FixedShape`; the crate's source holds no hand-written `impl FixedShape`, no `const DEPTH` and no wrapper type around a `quire-canonical` type; and no copy of `quire-canonical` source or of its published vectors is in the repository, each checked by a test that reads the manifests, `deny.toml` and crate source. | Test (TC-048) |
| FR-038-AC-81 | Observed at the operation check of a body-root application, an application whose `operation.identity` the operation catalog does not list refuses `invalid_package` with cause `unknown-operation` at `operation.identity`; `quire.op.integer.add`, a `binary` operation, under the operator `unary` refuses `invalid_package`/`operation-class-mismatch` at `operator`; and `quire.op.integer.add` under `binary` with two operands, no law, no mode and no member admits, so neither refusal is a refusal of every application. Their order against every other check of the entry is FR-038-AC-67's. An application whose `operation` is a bare string, not an object of the closed operation shape, refuses `invalid_semantic_graph` at that application's `operation`. | Test (TC-048) |
| FR-038-AC-82 | Observed at the operation check, `quire.op.integer.div`, which catalogues the one law role `integer_division`, with no law refuses `invalid_package`/`operation-law-missing` at `operation.laws`; `quire.op.integer.add`, which catalogues no law, with one law refuses `invalid_package`/`operation-law-mismatch` at `operation.laws/0`; and `quire.op.integer.div` with one law whose `role` is the word `not_integer_division` and whose `definition` is a catalogued `integer_division` definition refuses `invalid_package`/`operation-law-mismatch` at that law's `role`, so the role is checked in its own right and not only through the definition. | Test (TC-048) |
| FR-038-AC-83 | Observed at the operation check, `quire.op.decimal.add`, which catalogues the mode kind `rounding`, with `mode` `null` refuses `invalid_package`/`operation-mode-mismatch` at `operation.mode`; with the mode `{kind: rounding, value: toward-zero}` over a first operand typed at a `bounded_domain` over a `decimal` whose body binds `rounding` to `nearest-even`, reached by a `reference` to the domain and, separately, by a `literal` whose `type` names it, it refuses `invalid_package`/`operation-mode-type-mismatch` at `operation.mode/value`. | Test (TC-048) |
| FR-038-AC-84 | Observed at the operation check, `quire.op.quantity.convert`, which catalogues the member kind `type_argument`, with its `rounding` mode supplied and `member` `null` refuses `invalid_package`/`operation-member-mismatch` at `operation.member`; `quire.op.record.project` with a `field` member whose `name` is a field the declaring record type does not declare refuses `ill_typed`/`operator-ineligible` at `operation.member.name`; and `quire.op.model.reaches_field` with a `field` member whose `declaration` names no node of the graph refuses `ill_typed`/`operator-ineligible` at `operation.member.declaration`. | Test (TC-048) |
| FR-038-AC-85 | Observed at the operation check, `quire.op.integer.add`, whose entry has two fixed operands and no rest operand, with three arguments refuses `ill_typed`/`operator-ineligible` at `arguments`; with two `literal`s typed at an `integer` node it admits, and with a first `literal` typed at a `boolean` node it refuses the same way at `arguments/0`; `quire.op.boolean.not` over an application of `quire.op.state.clause`, whose catalogued result is `clause`, refuses the same way at `arguments/0`. | Test (TC-048) |
| FR-038-AC-86 | Observed at the operation check, `quire.op.structural.eq`, whose `same_type` constraint covers both operands, admits two distinct `value`/`parameter` nodes typed at one record type, two `literal`s typed at one type and two nested applications of one `result_type`, and refuses `ill_typed`/`operator-ineligible` at `arguments/1` for parameters typed at two record types, `literal`s typed at two types and nested applications of two result types, so the constraint compares the types the operands resolve to and not the operand nodes. | Test (TC-048) |
| FR-038-AC-87 | The operand family of a node is exactly, by tag and form: `scalar_type` `boolean`, `integer`, `rational`, `decimal`, `float32`, `float64`, `text` and `enum`, and `composite_type` `option`, `sequence`, `set`, `bag`, `ordered_set`, `record`, `tuple` and `reference`, each its own name; `expression`/`reference`, `reference`; `function` `pure_function`, `predicate` and `recursive_function`, `function`; `model` `object_type` and `systems_interface`, `object`; and `relation`/`population`, `population`; and `temporal`/`formula`, `temporal` (FR-038-AC-68); and, since IR-549, `composite_type`/`union`, `union` (FR-038-AC-99); every other kind of the closed node taxonomy has none, and the pairs are compared whole, so a form moved to another family or added to the list fails it. A kind is type-shaped exactly when its tag is `scalar_type`, `composite_type`, `bounded_domain`, `relation` or `function`, or it is `expression`/`reference` or `temporal`/`formula`, checked for every kind of the taxonomy. The `temporal`/`formula` pair arrives with the IR-503 code change; until it lands the pairs the unit test compares do not include it. | Test (TC-048) |
| FR-038-AC-88 | The preimage of an application node that is member 1 of a `recursion_group` of two, whose body is `quire.op.integer.add` over a `reference` to member 0 and a `reference` to a node outside the group, with `declaration` `{qualified_name: [pkg, total]}`, serializes, observed at the key check and with members in sorted order, to exactly this object: `body` (`arguments` the `group_reference` `{ordinal: 0, term: group_reference}` then the `reference` to the outside node, `operation` `{identity: quire.op.integer.add, laws: [], leaves: [], member: null, mode: null}`, `operator` `binary`, `result_type`, `term` `application`), `declaration` `{qualified_name: [pkg, total]}`, `node_tag` `function`, `recursion` `{ordinal: 1, size: 2}`, `semantic_form` `function`, `semantic_type` and `version` `quire.application-node/v1`; a `reference` to a group member becomes a `group_reference` in an aggregate member, a binding value and a nested application's arguments, a reference outside the group stays, and `recursion` is `{ordinal, size}` of the node in its group; an application node whose `node_id` is not the SHA-256 of that preimage refuses `invalid_package`/`stale-node-key` at its `node_id`, and one whose `node_id` is that digest admits. | Test (TC-048) |
| FR-038-AC-89 | Every identity digest recomputes to the value an in-repo fixture recorded before the move to `quire-canonical`: for every positive fixture each nominal, application and structural node key and the `package_id` equal the recorded digest; lowering every node of every positive fixture yields `ir_id` values, a lowered package `package_id` and lowered package canonical bytes equal to the ones recorded from the lowering before the move; and the canonical bytes of one preimage of each kind (a nominal preimage of each of its four versions, an application, a structural, a lowered node and a lowered package preimage) equal an expected byte string written out in the test from the preimage's JSON text, not computed by a call into the code under test, with the recorded digests as the second oracle. | Test (TC-048) |
| FR-038-AC-90 | `NominalIdentityPreimage::digest` takes a byte limit: called with the limit equal to the canonical byte length of a preimage of each of its four versions it returns the digest the node key holds, and one byte lower it returns the encoder's byte-limit refusal and no digest; the reader passes its configured `limits.bytes` to it, and no call in the crate source passes `u64::MAX` or a literal cap, each checked by a test that reads the crate source. | Test (TC-048) |
| FR-038-AC-91 | The files `canonical.rs`, `binding.rs`, `output_mapping.rs` and everything under `checked_package/` hold none of the symbols `CanonicalWriter`, `canonical_envelope_bytes`, `digest_json` and `serde_json_canonicalizer`, and no `serde_json::to_vec` or `serde_json::to_value` call whose result reaches a digest, a node key, a lowered preimage, the order of a dimension's terms, the selected-document digest, output-mapping identity material or the bound identity envelope, each checked by a test that reads those files and counts the symbols and calls, which finds none. Planned, and blocked on `quire-canonical`'s pending `Encode` for `serde_json::Value`: the same scan finds no `value_to_vec` and no `impl` of `Encode` for `serde_json::Value`, and the manifest enables `quire-canonical`'s `serde_json` feature; until that upstream change lands the scan excludes `value_to_vec`. | Test (TC-048) |
| FR-038-AC-92 | The application node preimage, `LoweredNodePreimage` and `ContractPackagePreimage` implement `quire_canonical::Encode` and none derives `FixedShape`; `NominalIdentityPreimage` with its four preimage structs, the structural node preimage and the bound identity envelope derive `FixedShape` and hold no `Value`; no `FixedShape` implementation is hand-written for any of them, each checked by a test that reads the crate source. | Test (TC-048) |
| FR-038-AC-93 | A selected model document holding the number `9007199254740993`, `-9007199254740993`, `9.007199254740993e15`, `1e20` or `18446744073709551617` at `/package/count` refuses `noncanonical_wire` at `/lock/model_selections/0/digest` with `document_pointer` equal to `/package/count`, whether the row selects the document's own digest or another digest, so the refusal precedes `byte-digest-mismatch`; a document holding such numbers at `/b` and then `/a/0` names `/b`, the first in document order; the same document holding `9007199254740992`, `-9007199254740992`, `9.007199254740992e15` or `0.5` at that pointer is not refused for it and is digested; an integer value type whose upper bound is `9007199254740993` in a document that is otherwise admitted refuses the same way, where today it is admitted (the reader change); and a package document refused `noncanonical_wire` for its own bytes (FR-038-AC-79) carries no pointer and no `document_pointer`. | Test (TC-048) |
| FR-038-AC-94 | A selected model document whose bytes are exactly `limits.bytes` long is read, digested and matches its row, with the package itself shorter than that limit; read with a `limits.bytes` one lower the same package returns `incomplete` for `bytes` with that limit and the document's length reported and no pointer, as FR-038-AC-26 states for the byte limit, while a `work` limit one below the work of the same read still returns the row pointer `/lock/model_selections/0` (FR-038-AC-30). | Test (TC-048) |
| FR-038-AC-95 | Lowering encodes under the byte limit the package was read under: lowering an admitted package whose retained limit equals the length of a lowered node's preimage lowers it, and with the retained limit one byte lower, through a unit seam that takes the ceiling as an argument as FR-034-AC-6's package step does, that node's record is `failed` and its sibling records are unchanged; a lowered package whose bytes exceed the ceiling makes every requested record `failed` with no package bytes or id and no panic; and the crate source holds no `expect` or `unwrap` on the package encode, checked by a test that reads `lower.rs`. | Test (TC-048) |
| FR-038-AC-96 | Each of the fifteen `temporal_formula` identities (`holds`, `true`, `false`, `not`, `and`, `or`, `implies`, `eventually`, `always`, `once`, `historically`, `until`, `release`, `since`, `triggered`) admits as the body root of a `temporal`/`formula` node, with operator `temporal_formula`, no law, `mode` `null`, no leaves, its catalogued member (`temporal_interval` with a closed interval on the eight interval operators, `null` on the other seven, under a clause selecting a bounded `temporal_profile`) and its catalogued operands (`holds` over one `boolean` operand, `true` and `false` over none, `not` and the four unary interval operators over one `reference` to a `temporal`/`formula` node, `and`, `or`, `implies` and the four binary interval operators over two); each formula node is referenced from a clause's formula argument or a formula operand; `quire.op.temporal.fair` admits as the body root of a `temporal`/`fairness` node with a well-formed `fairness` member and no operand, referenced from the fairness argument of a clause selecting `quire.temporal.infinite-trace/v1` (a bounded profile admits no fairness argument); and a package holding a `quire.op.temporal.clause` node whose sixth argument references a `temporal`/`formula` node applying `quire.op.temporal.eventually` with the member `{kind: temporal_interval, interval: {lower: "0", upper: "3"}}` over a second formula node applying `quire.op.temporal.holds` admits with every node key and its `package_id` re-derived, and lowers under a profile that supports the `temporal` tag and returns `unsupported` naming `temporal` under one that does not. | Test (TC-048) |
| FR-038-AC-97 | On each of the eight interval operators the member `{kind: temporal_interval, interval}` admits, under `quire.temporal.infinite-trace/v1`, with `interval` `{lower: "0", upper: "3"}`, `{lower: "9", upper: "10"}`, `{lower: "2", upper: null}` and `null`; `{lower: "-1", upper: "3"}` refuses `invalid_package`/`invalid-value` at the bound (`/semantic_graph/nodes/{n}/body/operation/member/interval/lower`), `{lower: "0", upper: "-2"}` refuses the same way at `.../interval/upper`, and `{lower: "-5", upper: "-2"}` refuses at `.../interval/lower`, the first bound in member order, each in the term walk of the body, before any temporal step and under every profile (the non-negative bound is a relayed QSpec owner ruling, pending the QSpec change that tightens the pattern); `{lower: "3", upper: "0"}` and `{lower: "10", upper: "9"}` refuse `invalid_package`/`invalid-value` at the application (`/semantic_graph/nodes/{n}/body`), also when the formula node is reached by no clause (the sweep after every clause, an IR reading) and `{lower: "18446744073709551617", upper: "18446744073709551616"}` refuses the same way, so the comparison is numeric and neither lexicographic nor fixed-width; a member of kind `fairness`, a `null` member, an interval holding a third member, an interval whose `lower` is the integer `0`, and a bound of `"1.5"`, `"01"`, `"+1"`, `""` or `"3x"` each refuse `invalid_package`/`operation-member-mismatch` at `operation.member`; and any member on `quire.op.temporal.holds`, `quire.op.temporal.not` or `quire.op.temporal.clause` refuses `operation-member-mismatch` at `operation.member`. | Test (TC-048) |
| FR-038-AC-98 | The `fairness` member of `quire.op.temporal.fair` admits as `{kind: fairness, fairness_kind: weak, granularity: whole, declaration, name}` and as `strong` with `each`, with a `declaration` and `name` that resolve (the resolution refusals are FR-038-AC-103's); a `fairness_kind` of `medium`, a `granularity` of `part`, a member lacking `name`, a member holding an extra member, a `null` member and a member of kind `temporal_interval` each refuse `invalid_package`/`operation-member-mismatch` at `operation.member`; and a `fairness` member on `quire.op.boolean.not` refuses the same way. | Test (TC-048) |
| FR-038-AC-99 | A package holding a `composite_type`/`union` node `Shape` with the members `Circle(Integer)`, `Rect(Integer, Integer)` and `Empty`, a `value`/`union_value` node `Shape::Rect(2, 3)` and an `expression`/`case` node whose body root applies `quire.op.control.case` over the scrutinee and the arms `Circle`, `Rect` and `Empty` in member declaration order, the binders `r`, `w` and `h` as `value`/`parameter` nodes typed `Integer` and every arm body of the `result_type`, admits, the three forms decoded and the operand family of the `Shape` node `union`; the same `case` with its arms out of member order, with the `Empty` arm omitted, with the `Circle` arm repeated, with the `Rect` binder aggregate holding one reference, with the `Circle` binder typed `Text` and with one arm body of another type than the `result_type` each refuse `ill_typed`/`operator-ineligible` at the `case` node (`/semantic_graph/nodes/{n}`); a union with no member and a union value with two bindings each refuse `invalid_semantic_graph` at the node's `body`; a union type with `Circle` twice refuses `invalid_package`/`duplicate-member` and a payload reference to a node that is not a type refuses `ill_typed`/`operator-ineligible`, each at the union node; and a union value naming `Triangle`, a `Rect` with one payload term and a `Circle` over a `Text` payload each refuse `ill_typed`/`type-mismatch` at the node. | Test (TC-048) |
| FR-038-AC-100 | Placement in both directions: an application of operator class `temporal`, `temporal_formula` or `temporal_fairness`, naming any identity, refuses `ill_typed`/`operator-ineligible` at the node that holds it when it is the body root of a node of another form (a `temporal_formula` application in a `function` node, a `quire.op.temporal.fair` application in a `temporal`/`formula` node), an element of another application's `arguments`, a `binding` value or inside an `aggregate`; a `temporal`/`formula` node, a `temporal`/`fairness` node and a `temporal`/`temporal_clause` node whose body is an empty `aggregate`, a `literal`, or an application of another class refuses the same way at the node; a `reference` to a `temporal`/`formula` node from a `function` node's body, from a clause's fairness argument or from a `case` argument, and a `reference` to a `temporal`/`fairness` node from a clause's formula argument or from a `temporal_formula` operand, refuses the same way at the node that holds the reference; a `case` application refuses `ill_typed`/`operator-ineligible` at the node that holds it when it is a body root of a node of another form or nested, and an `expression`/`case` node whose body is not a `case` application refuses the same way; any of the four classes as the root of or nested in a `diagnostics.entries[].details[]` term refuses at that application's `operator` (`/diagnostics/entries/{i}/details/{j}/operator`, and below it for a nested one); and each application, node and reference at its own place admits (FR-038-AC-96, FR-038-AC-99), so none of the refusals is a refusal of every such term. The in-repo `v2_all_families` formula node, whose body is an empty `aggregate`, is refused as above and the code change replaces its body. | Test (TC-048) |
| FR-038-AC-101 | The reader's refusal types, `CheckedPackageRefusalCode` and `CheckedPackageRefusalCause`, carry neither `unsupported_construct` nor `expression-form`, so no input is refused with either; the diagnostics wire vocabulary `CheckedDiagnosticCode` still carries `unsupported_construct`, so a `diagnostics.entries[]` entry whose `code` is `unsupported_construct` reads as QSpec's schema allows; no application is evaluated by the reader or the lowerer, so an admitted `temporal`/`formula` node and an admitted `expression`/`case` node lower, under a profile that supports their tags, to nodes whose body equals the admitted body and whose `ir_id` is derived from it as for every node, and a profile lacking the tag returns `unsupported` naming it (FR-038-AC-8). | Test (TC-048) |
| FR-038-AC-102 | The temporal step runs after the frame and state-clause step and before the operation step, placement first: a package holding one node with a placement defect (a `temporal_formula` application as the body root of a `function` node) and a second with a lower `node_id` digest whose operation identity is unknown refuses for the placement defect, and so does the same placement defect beside a lower-digest node whose `operator` differs from its entry's class (`quire.op.control.case` under `unary`), in both digest orders each, the placement defect taken in the shape of a `temporal_formula` application in a `function` node; two placement defects are reported at the lower `node_id` digest; a placement defect is reported ahead of any clause defect in either digest order; within one clause an `over` defect is reported ahead of a fairness-resolution defect, ahead of a profile-fit defect, ahead of an interval-bounds defect, each adjacent pair built as two defects of one clause and in both orders of the two defects' positions; and across two clauses the lower-digest clause's later-stage defect (a profile-fit defect) is reported ahead of the higher-digest clause's earlier-stage defect (an `over` defect), in both digest orders of the two clauses. The temporal step skips a member whose shape the operation step refuses, so `{lower: "1.5", upper: "0"}` refuses `invalid_package`/`operation-member-mismatch` at `operation.member` and never `invalid-value`, under a bounded and under the infinite-trace profile alike, and a wrong-kind or `null` member on an interval operator under a bounded profile refuses the same way. A negative bound is a schema-pattern failure, refused `invalid_package`/`invalid-value` at the bound in the term walk of the body, before placement and every temporal step, while `"1.5"` is refused `operation-member-mismatch` at the operation step (the asymmetry is deliberate) (a relayed ruling, pending the QSpec change that tightens the pattern): `{lower: "0", upper: "-2"}` refuses at `.../interval/upper`, `{lower: "-1", upper: "-3"}` at `.../interval/lower`, and `{lower: "-1", upper: null}` refuses at `.../interval/lower` under a bounded profile, under `quire.temporal.infinite-trace/v1` and under a clause that also holds a placement defect at a lower-digest node, never `operation-member-mismatch`; `lower > upper` stays in the bounds step, after profile fit, so `{lower: "3", upper: "0"}` under a bounded profile in a clause whose profile fit also fails refuses the profile-fit defect first. | Test (TC-048) |
| FR-038-AC-103 | A clause whose `over` argument is a `reference` to a `value`/`parameter` node among its `dependencies` admits; one whose `over` references a `value`/`parameter` node that is not among its `dependencies` refuses `missing_declaration`/`missing-name`, and one that references a declared dependency that is a `scalar_type` node refuses `invalid_model_binding`/`malformed-declaration`, each with path `/semantic_graph/nodes/{clause}/body/arguments/0` and the locus of the "Path and locus" table; a fairness member whose `declaration` and `name` resolve to an operation of a `model` declaration node admits, and one whose `name` is no operation of that node, or whose `declaration` names no node, refuses `missing_declaration`/`missing-name` (a `name` with path `/semantic_graph/nodes/{fairness node}/body/operation/member/name` and the `declaration` target's key as locus, a `declaration` with path `.../member/declaration` and the fairness node's key), and one whose `declaration` names a node that is not a `model`/`object_type` declaration (a `scalar_type` node, a `model`/`value_type` node) refuses `invalid_model_binding`/`malformed-declaration` with path `/semantic_graph/nodes/{fairness node}/body/operation/member/declaration` and the `declaration` target node's key as locus; the `over`-not-among-`dependencies` case is observed at the unit level of the temporal step, as AC-68 observes the clause check, because the application-node dependency join refuses a package that names a non-dependency reference first, while the package-level half (an `over` that names no node) is read through the reader; a `name` that matches two exposed operation members of the declaration node refuses `ambiguous_declaration`/`ambiguous-name`, one that the node only inherits admits and resolves to its most-derived redefinition, and a declaring node whose owner is not recovered refuses with that resolution's own refusal (`missing_declaration`/`missing-selection` or `invalid_package`/`stale-node-key`), the ambiguous name with path `.../member/name` and the `declaration` target's key as locus and the unrecovered owner with path `.../member/declaration` and the fairness node's key (the ambiguous, inherited and unrecovered-owner outcomes are QSpec owner rulings relayed via the IR planner and not yet in merged QSpec text). | Test (TC-048) |
| FR-038-AC-104 | Under a clause whose `temporal_profile` law names `quire.temporal.event-position.false-extension/v1`, `quire.temporal.fixed-sample.false-extension/v1` or `quire.temporal.timestamped-event.finite-window/v1`, every interval operator of its formula tree with a closed interval admits, and one with a `null` interval or `{lower, upper: null}` refuses `invalid_package`/`operation-member-mismatch` at that operator's application; a non-empty fairness argument refuses `operation-member-mismatch` at the clause's application and an empty one admits; under `quire.temporal.infinite-trace/v1` the same `null` interval, `{lower, upper: null}` and a non-empty fairness argument admit; under `quire.temporal.timed/v1` a `null` interval and a closed integer interval admit and `{lower, upper: null}` refuses `operation-member-mismatch` at that operator's application, and a non-empty fairness argument admits under it as under infinite-trace (only the three bounded profiles refuse one, as above; a relayed ruling pending QSpec merge); this criterion pins no outcome for a timed bound that is not an integer string or an open end (pending the QSpec change that carries timed bounds); and the profile fit reads only the formula nodes reachable from that clause. | Test (TC-048) |
| FR-038-AC-108 | Each of the five members of QSpec FR-250's Values table admits as a clause's `temporal_profile` law (the lock's `temporal_profile` row naming it). A clause whose `temporal_profile` law `definition` is not one of the five refuses `unknown_profile` at `/semantic_graph/nodes/{n}/body/operation/laws/0/definition`, ahead of the same clause's `over` defect and of any defect of a higher-digest clause, and is not read as bounded (a `null` interval under it refuses `unknown_profile` and not `operation-member-mismatch`). The cause, a relayed ruling pending QSpec merge, is decided from the package alone: `wrong-selection-role` when the same `{authority, identity}` is a row of the package's own `lock.profile_selections` (or the `lock.edition` row) under a `role` other than `temporal_profile`, `unsupported-selection` otherwise. A known profile is matched by identity label alone, `authority` taking no part: a law naming a FR-250 member's identity under a different `authority` is a known profile and refuses `operation-law-unselected` at AC-57's join, not `unknown_profile`. So `quire.fixture.temporal-profile/v1` selected nowhere else, an empty identity and a differently spelled `quire.temporal.infinite-trace/v1` refuse `unsupported-selection`; `quire.package.composed/v1` named by the law and selected in the lock as a `profile_selections` row of role `binding_contract` refuses `wrong-selection-role`, and so does `quire.protocol.complete/v1` selected as a `protocol_profile` row; and the same two identities named by the law with no such row in the lock refuse `unsupported-selection`. The refusal is the reader's `CheckedPackageRefusalCode::UnknownProfile` with the causes `unsupported-selection` and `wrong-selection-role` in `CheckedPackageRefusalCause`, a code and two causes the code change adds. | Test (TC-048) |
| FR-038-AC-105 | The temporal and case operands are checked at the operation step, after the temporal step: `quire.op.temporal.holds` over a `reference` to a `temporal`/`formula` node, and over a Boolean-family `reference`, admits only the latter, the former refusing `ill_typed`/`operator-ineligible` at the argument; `quire.op.temporal.until` with one argument, `quire.op.temporal.not` with two, `quire.op.temporal.true` with one, and `quire.op.temporal.and` over a Boolean-family `reference` each refuse `ill_typed`/`operator-ineligible` at `arguments` or at the argument; and `quire.op.temporal.not` over a `reference` to a `temporal`/`formula` node admits. | Test (TC-048) |
| FR-038-AC-106 | `quire.op.structural.eq` over two values of the `Shape` union admits, and over a `Shape` value and a value of another union refuses `ill_typed`/`operator-ineligible` at `arguments/1` by its `same_type` constraint (QSpec FR-440-AC-6); over a union whose payload types reach no `text` type, `leaves` empty admits; and over `union Label { Named(Text[0, 8; nfc]), Tagged(Integer, Text[0, 8; nfc]), Empty }`, a comparison admits with exactly the leaves `["member:Named", "position:0"]` and `["member:Tagged", "position:1"]` in member declaration order, and, following FR-038-AC-70's convention, a list holding only the `Named` leaf, and a list holding only the `Tagged` leaf, each refuse `invalid_package`/`operation-law-missing` at `operation.leaves`, while the list of both leaves followed by the `Named` leaf again, and the list of both followed by a leaf `["member:Empty"]`, each refuse `invalid_package`/`operation-law-mismatch` at `operation.leaves/2` (the `member:<Ident>` segment is a QSpec owner ruling relayed via the IR planner and not yet in merged QSpec text); and over the recursive unions `union IntList { Cons(Integer, IntList), Nil }` and `union TextList { Cons(Text[0, 8; nfc], TextList), Nil }`, `structural.eq` admits (a cycle through a union is never refused `ill_typed`/`operator-ineligible`, a relayed ruling pending QSpec merge): over `IntList`, which reaches no `text`, with `leaves` empty; over `TextList` with exactly the leaves `["member:Cons", "position:0"]` and the recursion leaf `["member:Cons", "position:1", "recursion:0"]` (`d` is 0, the segments the path held when `TextList` was entered, as for a record's `recursion:<d>`), a list lacking the recursion leaf refusing `operation-law-missing` at `operation.leaves`, one whose recursion leaf reads `recursion:1` refusing `operation-law-mismatch` at that leaf's `path`, and a recursion leaf at a reentry of `IntList` refusing `operation-law-mismatch` at its `path`. | Test (TC-048) |
| FR-038-AC-107 | Read from the authoritative `quire-specification` checkout, never copied into this repository, the QSpec positive fixtures `positive-all-families.json` (the clause, `quire.op.temporal.holds` and `quire.op.temporal.eventually` with the interval `{lower: "0", upper: "3"}`), `positive-clause-operations.json` and `positive-union-nodes.json` (`quire.op.control.case`) each admit end to end with their recorded `package_id`. The test is named `tc_048_qspec_positive_fixtures_admit`, lives in its own test target, outside the run of `make test`, which has no checkout; it is run by a dedicated target, `make conformance-qspec`, which reads the root of a `quire-specification` checkout from the environment variable `QUIRE_SPECIFICATION_DIR` and fails, not skips, when the variable is unset, when it names a path that does not hold `proposals/checked-package-v2/fixtures/`, or when a fixture does not admit. The criterion is verified by that target alone and never counted from `make test`; wiring the target into CI is a separate decision this specification does not make. Nothing is copied from QSpec into this repository. The self-built packages of FR-038-AC-96 and FR-038-AC-99 reproduce the fixtures' features: a clause with the `temporal_profile` law `quire.temporal.event-position.false-extension/v1`, formula nodes whose body root is the application, a `case` scrutinee that is a `value`/`parameter` node typed at the union, union value payloads that are references to value nodes, arm binders that are `value`/`parameter` nodes whose body is the `{name, level}` aggregate, and the `case` node's `dependencies` as the digest-ascending reference targets. | Test (TC-048) |

FR-038-AC-66 is retired and its ID is not reused (ADR-0056). It required that every
application of operator class `case`, `temporal_formula` or `temporal_fairness` be
refused `unsupported_construct`/`expression-form`, which IR-549 lifts: those classes
are admitted at their node roots (FR-038-AC-96 through FR-038-AC-100) and the
reader has no such code or cause (FR-038-AC-101).

## Dependencies

QSpec FR-370 (temporal clause body and temporal operation identities) and FR-440
(union, union value and case nodes) own the encodings that FR-038-AC-96 through
FR-038-AC-101 admit and check. QSpec FR-322 (AC-4, AC-8, AC-10, AC-28, AC-35 through AC-37), FR-201 (AC-2, AC-3) and FR-195 (AC-1 through
AC-5) own the normative V2 wire, identity-domain and lowering semantics;
quire-specification:TC-217 names this repository as their consumer evidence
owner. The first-party `agent-ix/quire-canonical` crate owns the RFC 8785
encoder that "Canonical encoding of the wire types" and "Every identity digest
is computed through quire-canonical" consume (FR-038-AC-74 through
FR-038-AC-80 and FR-038-AC-89 through FR-038-AC-95), including its pending
`Encode` for `serde_json::Value`. [FR-040](./FR-040-admit-frame-entries-and-state-clauses.md) owns the
QSpec FR-340 `modifies` entry shape, the FR-341 state clause and FR-342
operation anchor bodies, and the `model` form set.
