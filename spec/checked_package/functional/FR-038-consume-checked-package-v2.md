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
  - target: ix://agent-ix/quire-specification/FR-322-AC-51
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-52
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-66
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-73
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-74
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-120
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-121
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-122
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-260
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-261
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-262
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-263
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-264
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-265
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-266
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-270
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-272
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-273
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-278
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-279
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-280
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-281
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-282
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-288
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-297
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-299
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-301
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-302
    type: references
  - target: ix://agent-ix/quire-specification/FR-142-AC-27
    type: references
  - target: ix://agent-ix/quire-specification/FR-142-AC-28
    type: references
  - target: ix://agent-ix/quire-specification/FR-142-AC-29
    type: references
  - target: ix://agent-ix/quire-specification/FR-142-AC-30
    type: references
  - target: ix://agent-ix/quire-specification/FR-342-AC-11
    type: references
  - target: ix://agent-ix/quire-specification/FR-415-AC-22
    type: references
  - target: ix://agent-ix/quire-specification/FR-415-AC-23
    type: references
  - target: ix://agent-ix/quire-specification/FR-415-AC-24
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
`quire.checked-package/v2` package only after validating its required package
name, structural node identities, selected evidence and graph, and lower admitted
items independently. Any other
contract version is refused with a typed `unknown_contract_version` code.

## Inputs

Untrusted wire bytes; caller-selected read limits, for which the crate ships one
named finite default appropriate to a single local request — 1048576 bytes,
10000 nodes, 100000 edges, 100000 occurrences, 10000 diagnostics and 1000000
term-validation visits — as stable API, every member finite so that the default
admits no unbounded read, and no member a nesting depth, the reader having no
depth limit (see "Reading"); package evidence holding
each selected domain package's Semantic IR 2.0.0 document
supplied as bytes under its identity, each selected dependency's
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
  carries a typed code and an ordered, non-empty list of locations, so a caller
  distinguishes the condition and locates it without parsing prose. An
  `unknown_contract_version` refusal also carries the `contract_version`
  string the reader read. A frame-body refusal additionally carries the cause
  tag FR-322's `DiagnosticCausePairing` pairs with that code — `missing-name`
  with `missing_declaration`, `malformed-declaration` with
  `invalid_model_binding` — and the node key of the offending entry or, for a
  canonical-order defect, of the frame node itself, so a caller need not
  re-derive from the path alone why admission failed or at which node.
  The public model-intake refusal conversion retains its existing code, cause
  and root-owner pointer. Declaration identity/origin retention below is
  implemented at the private intake return and checked by crate-local tests,
  not part of this public output.
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
  `ContractPackage`, which records the admitted package's `name` as its
  source identity; see
  [FR-035](./FR-035-complete-v1-contract-package-lowering.md).
- A typed accessor on the admitted package, `model_object_fields`, returning a
  model object type's effective fields with their derived member types, integer
  bounds as values; see "Typed accessor for a model object type's fields"
  (FR-038-AC-136 through FR-038-AC-144, IR-628).
- A borrowed typed read of the optional `operation_effects` carrier. Absence
  remains distinguishable from an admitted row whose four effect arrays are
  empty; a row carries its selected document, declaring node, original
  operation member, four admitted frame-entry arrays and producer provenance
  without creating another source of operation effects (IR-1226).
- A typed optional `operation_effects` input to the existing complete V2
  encoder, through `CheckedPackageSidecarsRef`. A constructible input row is
  untrusted until the strict reader admits the resulting package.
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

The public `CheckedPackageRefusal` carries `loci`, an ordered, non-empty list
of location records. Each record has `path: Option<JsonPointer>` and
`source_region: Option<CheckedSourceRegion>`; the first record is the primary
location. A value refusal has its RFC 6901 pointer in `path`. A refusal about
the byte stream has one record with no pointer or source region. A location
has a source region only when the reader can attribute that exact value to one
admitted source region; it never guesses a region from a graph-node id. The
separate graph-node `locus`, `document_pointer`, code and cause retain their
existing meanings. The former single `path` and `related_path` fields are
absent; callers receive no parallel legacy location fields.

For a repeated `lock.refinement_internals` row or spelling, the reader reports
both entries as two locations, in their declaration order, at their exact
concrete pointers. A repeated dependency identity and a dependency diamond
conflict likewise report both lock selections as two locations in declaration
order. Every other refusal reports one location. The second location does not change the
reader's code, cause, first-defect order or selected graph-node locus.

Every pointer in a location the reader reports is an RFC 6901 JSON pointer into the
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
the wrong kind. A structural node id or owner with a missing, extra or wrongly typed member
refuses at that member or at the containing object when a member is absent.
The code follows the closed-wire or graph stage that owns that check.

A refusal located at a graph node points at the node's member the failed
check read. Examples are its `node_id`, its `dependencies` (or one entry of
them), its `declaration.qualified_name`, one frame entry, one `aggregate`
member, or an `operation` law, mode or member. A reference that resolves to no
node points at that reference. A cycle without one shared
`recursion_group` points at the lowest-positioned member at fault: at its
`recursion_group` when it carries a different one, else at the node, which
lacks the member. Where this specification names the location of a refusal
as a dotted member path, such as `lock.sources`, the returned
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
  node body, a structural id member, an `operation`, a graph edge's
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
reader after the strict parse (the body grammar, term walks, structural-id checks,
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
refuses before structural graph validation on that stack. Direct decoding shall retain the same strict syntax and
duplicate-member refusal, canonical-byte decision, closed-wire losslessness
and unknown-member refusal, first-refusal precedence and JSON pointers,
structural-id shape and uniqueness checks, source-map content and admitted typed public API. For the
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

The reader SHALL admit a package with exactly the required non-empty `name`,
`lock`, `semantic_graph`, `source_map`, `capability_report` and `diagnostics`
beside `contract_version`, and the optional `refinement_step_rows` when present.
The package identity is its `name`, compared as a string by UTF-8 bytes. Two
packages under one identity are the same package only when their complete JSON
values compare member by member and array element by array element in order.
No package id, identity preimage, projection, content-derived identifier or
staleness refusal exists. The reader SHALL refuse any old extra top-level
member as `unknown_member` at that member. The lock's definition, profile,
feature and dependency selections are read directly; no duplicate selection
object exists to mirror or authenticate them.

The reader SHALL validate every `CheckedNodeId` as the closed structural value
`{package, path, ordinal}`. `package` is the owning package's `name`; an
imported node retains the package name it came from. `path` is a non-empty
array: a declared node's declaration path is its `declaration.qualified_name`;
an ungrouped derived node's path is `[node_tag, semantic_form, ...operands]`,
where an operand is a string or the closed index segment `{"node": i}` naming an operand in this document's `semantic_graph.nodes`. A path never contains a nested node id. A group member with a declaration keeps its own declaration path, so
several declared members in one group can have distinct paths. An anonymous
derived member uses the sole declared member's path when there is exactly one;
with zero or multiple declared members, the producer chooses its path and the
reader admits it without inventing an anchor. No grouped member embeds
operands in its path (QSpec FR-322-AC-274, TC-946 K14). Its structural
`node_id.ordinal`, like every node's, is producer assigned and need not be
zero or contiguous with another node's ordinal; the reader does not derive
it from the body or graph position or require sibling ordering. A recursion group is identified only by its shared, non-empty
`recursion_group` string label. References between its members name exact
structural node IDs, as outside references do. The wire carries no separate
group position or group-reference ordinal, and the reader derives none. A
valid graph permutation rewrites every index segment to its target's new graph
position while retaining each resolved structural key and exact reference.
The structural `node_id.ordinal` is an integer
from zero through 9007199254740991. A malformed structural ordinal refuses
`malformed_wire` at its own pointer. Two equal node ids refuse
`invalid_semantic_graph` at the second node's `node_id`; a repeated model root
of the same identity refuses at its later `owner`. A declared path that differs
from its `qualified_name` refuses at its `node_id`. The reader SHALL not
recompute, hash, rekey or seal a node id.

The reader SHALL accept only string or closed `{"node": i}` path segments,
where `i` is an I-JSON integer from zero through 9007199254740991. It SHALL
reject a nested node id, a malformed index segment or an index in a
`dependency_reference.node` declaration path as `malformed_wire` at the
offending segment. After closed-schema validation and before every
`semantic_graph` check, it SHALL reject an index at or beyond the number of
nodes as `malformed_wire` without cause at the segment's `/node` pointer,
choosing the first such index in canonical byte order. It SHALL then reject
a self or longer cycle through node-own-path index segments at the first
segment ordered by holding node's graph index and path position. Forward
indexes without a cycle are admitted. Graph order stays producer chosen;
reordering nodes requires each index segment to be updated to its target's
new graph position and never changes an ordinal by itself.

Every index segment SHALL resolve against the graph of the checked-package
document that holds its node id, even when the id's `package` names an
imported node's owner. A `dependency_reference.node` is a declared node id
with an all-string path. A node id in an external request or report resolves
against the graph of the package the document concerns; an FR-375 external
`{package, node}` key resolves its node against the selected dependency named
by the key's own `package`. The reader SHALL rebuild keys as shared,
structurally compared nodes: each index path segment holds its resolved
operand key, and no expansion into nested wire node ids or recursive stack
walk follows the length of an operand chain.

Where graph validation needs a deterministic node visit order, node ids compare
componentwise: `package` in UTF-8 byte order, then `path` elements (strings
before index operands, whose resolved structural keys compare recursively; proper prefixes first), then numeric `ordinal`. The
reader SHALL preserve graph array order; this comparison chooses a first
refusal, never reorders the package. Graph shape, owner joins, body grammar,
source-map totality and capability checks retain their independent typed
refusals. Required features are checked against the reported available
capability and the reader-supported feature set. The closed header decoder
checks tag/form and wrong-form vectors before graph-body validation.

A domain package is selected by a `model_root` node's owner, not by a lock
member. The selected identities are its `ModelRootOwner` identities, one root
per identity. The reader admits documents in ascending identity order:
`missing_import`/`missing-selection` at the root's `owner/identity` when no
document is supplied, `malformed_wire`/`malformed-json` there when supplied
bytes do not read, `noncanonical_wire` there with `document_pointer` when a
number cannot be represented by its canonical encoding, and
`invalid_model_binding`/`wrong-model-selection` there when the document names
a different identity. FR-154's declaration refusals retain their own order.
A model-owned node's owner `{kind: "model", identity, node}` joins that selected
root and the selected document's declaration; its `node_id.package` is the
selected domain package identity. Source-owned nodes join a selected
`{authority, identity}` source row. The reader does not infer an owner or a
path from a body on failure.

Each required declaration, literal type, application operation and result type
is read from the graph. Missing or invalid members refuse under the existing
graph/body rule, without a derived identity check. All fifteen expression forms
remain in the same closed taxonomy. A consumer sees structural ids and complete
bodies, not a projection of them.

### Canonical encoding of the wire types

The V2 wire types encode to RFC 8785 canonical bytes through the first-party
`quire-canonical` crate. This repository holds no copy or second encoder. The
reader compares supplied bytes with canonical bytes before closed-schema and
graph interpretation. It computes no V2 identity digest or known-answer hash.
The V1 canonical bytes and the lowering byte ceiling retain their separate
contracts (FR-016, FR-035).

`#[derive(FixedShape)]` is used only when a type's JSON depth is fixed by its
type; a member whose depth follows input uses iterative `Encode`. In-memory
`serde_json::Value` bodies and diagnostic details are delegated to
`quire-canonical`'s iterative encoder and nonrecursive drop helper. There is no
local recursive `Value` walker, extra nesting cap or second full-document
canonical buffer. An arbitrary-depth in-memory value encodes or returns the
encoder's typed limit/error on a 256 KiB stack without native recursion. The
encoder neither derives nor authenticates a node or package id.

| Type | Canonical path | Reason |
| --- | --- | --- |
| `CheckedSourceMapEntry` | `Encode` | Its structural node id has fixed-depth string or index path segments; source regions remain fixed shape. |
| `CheckedCapability` | `FixedShape` | A string and closed disposition. |
| `CheckedPackageLockV2` | `FixedShape` | Closed selections and strings, with no recursive member. |
| `CheckedSemanticGraphV2` | `Encode` | Node bodies contain variable-depth `Value` terms; node-id index segments have fixed wire depth. |
| `CheckedDiagnosticsV2` | `Encode` | Detail terms contain `Value`. |
| Complete V2 envelope | `Encode` | It delegates structural ids and graph bodies to their iterative encoders. |

The canonical byte output equals an independently assembled canonical JSON
value of the same complete envelope, including non-ASCII and astral strings.
The reader retains the exact complete bytes and typed fields; it never excludes
source occurrences or owner members from a hidden complete package value. A body
nested beyond the closed grammar refuses under the body grammar, even if it is
individually encodable in memory. Canonical encoder failures during lowering
remain typed `failed` records with the retained byte limit, not panics or
partial output.

### Canonical encoding and independent lowering

The reader uses `quire-canonical` for canonical wire encoding and the strict
canonical-byte decision. It computes no package or node id. Lowering keeps
the byte ceiling retained from the admitted package and reports encoder
failures as typed `failed` records with no partial package. A ceiling equal to
the complete canonical length succeeds; one byte lower fails. No hidden byte
ceiling or replacement identity is introduced.

`CheckedPackageV2::lower_independent_records` returns records without an
aggregate `ContractPackage`. For each requested key, its record shall equal
the record from a separate one-key `lower` call under the same profile and
retained byte ceiling. The operation shares graph indexing and component
closure work across its requests, but an aggregate package's byte-ceiling
failure does not replace a one-key result that fits. Each request still bears
its full work charge and reports the same terminal outcome as its one-key
call (FR-038-AC-309).

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
closed-schema decode, the header check and every
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
bytes; closed-schema decode and header; body grammar; then graph shape, owner,
selection, source-map and body joins in the order QSpec FR-322 defines. A body
outside the grammar refuses `malformed_wire` before any graph join. There is no
package-id recomputation or node-key authentication step.

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
pre-order, after the closed-schema decode and before structural graph validation
(the order of checks above).

Under QSpec FR-322 "Nominal nodes" and FR-322-AC-269, a source-owned
`enum`, `dimension` or `unit` node is an exception to the empty-aggregate
nominal body: its closed body holds the source values. An `enum` body is
`{term: "enum", ordered, members}` with boolean `ordered` and a non-empty
declared-order list of unique identifier members; `ordered: true` denotes
`ordered_enum`. A `dimension` body is `{term: "dimension", terms}` with a
list of `{dimension_node_id, exponent}` entries, where each exponent is an
integer string; a base dimension has an empty list. A `unit` body is
`{term: "unit", dimension_node_id, target_unit_node_id, scale, offset}`:
the first id names its dimension, the target is a node id or `null` for a
canonical root, and each rational has integer-string `numerator` and
positive-integer-string `denominator`. A model-owned nominal still has
`{term: "aggregate", members: []}` and takes its values from the selected
document. A source nominal with an empty aggregate, a missing or extra body
member, or a member of the wrong wire type refuses `malformed_wire` at the
closed-schema step before graph and semantic joins. A schema-valid body with
an incorrect owner, declaration, semantic type, dependency, case or unit join
refuses under the applicable semantic rule, not as a wire-shape defect.

The model crate SHALL expose an opaque `NominalBody` for producing these four
source-authored body forms. Its separate fallible `try_enum`, `try_dimension`,
`try_unit` and `try_dimension_equivalence` constructors SHALL accept typed
members and return a typed refusal with the offending body-relative RFC 6901
pointer when a member does not have the closed shape above. In particular,
they SHALL check non-empty unique identifier enum members, canonical integer
exponents, structural node-id shape, and the integer numerator and positive
denominator of every rational; the dimension-equivalence `root_scale`
numerator must also be positive. The constructors validate body shape only:
owner, declaration, graph references and semantic joins remain the V2 reader's
responsibility. No public raw variant, mutable field, unchecked constructor,
`From` conversion or deserializer may produce a `NominalBody`. A constructed
body SHALL serialize as its exact closed V2 body and be usable as a
`CheckedTerm::NominalBody`; ordinary package admission SHALL still perform all
reader checks after serialization.
An external consumer SHALL be able to read the ordered flag and borrowed
authored member names of an enum `NominalBody` through a read-only typed view.
The view SHALL return no enum details for dimension, unit or dimension-equivalence
bodies. It SHALL work on the body retained by an admitted V2 node, preserve
member order and spelling, and grant no mutation or package-admission path.

For a `scalar_type`/`dimension_equivalence` body, `dimension_node_ids` SHALL
contain two typed operands in ascending resolved structural node-id order. A
local operand SHALL be a `NodeRef` of a `scalar_type`/`dimension` node in the
importing graph. A package operand SHALL be the closed
`PackageNodeRef {package, node}` form, where `node` is a `DeclaredNodeId` of a
`scalar_type`/`dimension` node in the named directly selected dependency's
admitted package. The reader SHALL resolve a package operand by lookup in that
dependency only; it SHALL NOT mint a foreign node into the importing graph or
put that operand in the equivalence node's local `dependencies`. The writer and
`NominalBody::try_dimension_equivalence` SHALL preserve the two operand forms
as distinct typed values. The reader SHALL refuse a package operand naming an
absent, closure-only or self dependency as `missing_declaration`/
`missing-selection` at the operand's `/package`, and an absent declared node
as `missing_declaration`/`missing-name` at its `/node`. A wrong-kind operand or
a local `NodeRef` with a foreign package SHALL refuse `invalid_semantic_graph`
at the operand itself. These joins precede equivalence pair validation; they
do not relax the existing distinctness and ascending-order checks.

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
  depends on a chain of nodes' length, and structural-id and group-reference checks use explicit heap stacks.
  An in-group `reference` resolves by its exact structural target node ID.
  FR-038-AC-117.
- The V2 reader has no depth limit and nothing in it caps a caller's value by
  depth; the v1 limits of FR-019 and FR-023 are unchanged. FR-038-AC-117.
- The refusal of a nested `case` is decided in the same document pre-order walk
  as the `malformed_wire` refusals, ahead of structural graph checks, so that FR-322's
  first-in-pre-order rule holds across both. Merged FR-440 "Reader joins" lists the
  same refusal among the operation step's joins, and the merged texts do not
  reconcile the two stages; this stage is this reader's reading, which keeps one
  pre-order across the body. FR-038-AC-115 and FR-038-AC-116.
- Each of the five `body_grammar_mutations` of QSpec's `adverse.json` refuses
  `malformed_wire`, and the expected-failure list of FR-038-AC-112 holds none of
  them. FR-038-AC-118.

### A model document is a value inside a supplied document

A selected domain package's Semantic IR 2.0.0 document is evidence the caller
supplies under the package's identity (FR-322 step 1). The document is read
once with `quire_canonical::read`, which keeps each number's text, under the
reader's byte limit. It is not parsed a second time into a `serde_json::Value`
for the number decision, which is made on the number's text before any rounding
to a double. Two refusals about it are different in kind:

- A refusal about the checked package's own byte stream (malformed JSON,
  non-canonical bytes: FR-038-AC-79) is about the document the reader was handed
  as the package. It carries no pointer, because no value of that document is
  the fault: the bytes as a whole are.
- A refusal about a value located inside a supplied domain document is about
  one number of that document, which is not the package. It carries the RFC 6901
  pointer of that number into the model document, as `document_pointer`.

The reader refuses a model document that holds, at any depth, a number whose
text denotes a value of magnitude greater than 2^53 (9007199254740992), however
it is spelled: `9007199254740993`, `9.007199254740993e15`, `1e20` and an integer
spelling past the 64-bit range are all refused, and `9007199254740992`,
`-9007199254740992` and `9.007199254740992e15` are admitted. The decision is on
the text, so a spelling a double would round to 2^53 is still refused. The
refusal is `noncanonical_wire`, located at the root's owner identity
(`/semantic_graph/nodes/<i>/owner/identity`) and carrying a `document_pointer`,
the pointer of the first such number in document order, except that a number
with no finite double is named when the reader reaches it (`/package/count` for
a member `count` of the object `package` at the document's root). The document
is read under the reader's byte limit: bytes that exceed it return `incomplete`
for `bytes` with that limit and the document's length as consumed and no
pointer, as every byte-limit outcome is (FR-038-AC-26); the `work` limit charged
for the same document is located at the root's `owner` (FR-038-AC-30), and the
two are different limits with different locations.

The same check refuses a number whose exact value its RFC 8785 encoding loses,
by the requirements of the QSpec native `noncanonical_wire` diagnostic
(quire-specification:FR-271) and its two causes (quire-specification:FR-272):
`inexact-integer` for a number whose text denotes a whole value past 2^53,
however spelled, and `inexact-number` for any other number that FR-272 defines
as inexact; a number that matches both reports `inexact-integer`. The text a
number is compared with is the one `quire-canonical` writes for it, whose
tie-breaking is the even last digit. `quire_canonical::read` refuses a number
past the double range with `ReadError::NumberOutOfRange`, which carries the
number's RFC 6901 pointer and source text, and `admit_document` maps it to that
refusal, with the cause `inexact-integer` when the text denotes a whole value
past 2^53 and `inexact-number` otherwise. Every other read failure but the byte
limit is bytes that are not strict JSON (a truncation, a repeated member name,
a lone surrogate escape, invalid UTF-8): `malformed_wire`/`malformed-json` at
the root's `owner/identity`, with no `document_pointer`.

When one document carries several faults, the first fault `quire_canonical::read`
returns decides. The read checks, in this order: the byte limit; that the whole
input is UTF-8; then one pass in document order, in which a number past the
double range is raised when it is read, a repeated member name only when its
object closes, and any other malformation at its offset. It stops at the first
fault. So invalid UTF-8 anywhere in the bytes is the first fault even after an
out-of-range number (`[1e400,"<0xFF>"]` is `malformed-json` with no pointer),
whereas a lone surrogate escape after the number is read after it
(`[1e400,"\ud800"]` names `/0`). An out-of-range number is named ahead of every
earlier inexact number, which is decided only after a read that finds no fault.
A repeated name in an object that closes before the out-of-range number decides
first (`[{"a":1,"a":2},1e400]` is `malformed-json`); a repeated name in the
object that still holds the number open is not yet detected, so the number is
named (`{"a":1,"a":2,"n":1e400}` names `/n`); a truncation after the number is
read after it (`[1e400` names `/0`). A number below the double range (`1e-400`)
reads as zero with its text kept, so it is an `inexact-number`. The "first in
document order" of FR-038-AC-93 and FR-038-AC-110 is the first number, among the
numbers this reader decides on their text after a read that succeeds, in
document order. The package document is a different read: it is parsed through
`serde_json` and its bytes compared with the canonical bytes of the value read,
so the crate's own manifest declares `serde_json` with the feature
`float_roundtrip` (FR-038-AC-111), which makes that parse of a shortest
round-trip text exact whatever other crates in the build turn on. The package's
own byte stream already refuses such a number, because its text is not the
canonical bytes of the value read, and carries neither cause nor pointer
(FR-038-AC-79).

### Artifact references

A definition is named by its identity alone. QSpec's `DefinitionRef` is the
closed shape `{authority, identity}` (FR-322 `lock`; QSpec's Wave B Q9 change dropped a
definition's revision and digest, and STD-150 dropped a source's revision), and this reader's
`CheckedArtifactRef` is exactly that shape: `authority` and `identity`, each a
nonempty string, and no other member. A definition reference carries no
`revision`, no `digest_domain`, no `digest` and no `export`: the package identity is its required `name`, not a property of the references it holds. The reader reads a definition reference at
each place the wire has one:

- `lock.edition.definition` and each `lock.profile_selections[].definition`,
  the `definition` of a `Selection`;
- each entry of `lock.definition_selections`;
- `diagnostics.catalog`;
- each `operation.laws[].definition` of an `application` term;
- each `law_roles` entry of the operation catalog (the catalog's owner,
  `quire-verification-contracts`, publishes them as `{authority, identity}`;
  this reader holds no copy of it).

A source reference is the closed shape `{authority, identity}`, with both
members nonempty strings. This reader's source reference type is
`CheckedSourceRef`. It is the member type of
`lock.sources` and the `source` of every `source_map` region and every
`diagnostics.entries[].loci[]` region. Source references identify the producer's
source and the source ranges still identify locations within it; the reader
does not authenticate the source's bytes. A source reference has no `revision`,
`digest_domain` or `digest`. A changed source-map reference changes the complete canonical bytes of a lowered `ContractPackage` under FR-035; it changes no content-derived identifier.

Both shapes are closed under `deny_unknown_fields`, so there is one refusal
for a reference of the earlier shape and no reader for it. This is a refusal
control, not a compatibility layer: the reader never relabels, drops or
ignores the extra member to fit the current contract. The refusal code depends
on where the reference sits, because the reader decodes two places differently:

- The `lock`, `diagnostics` and `source_map` members are
  decoded with the typed wire decode. A definition reference carrying
  `revision`, `digest_domain`, `digest` or `export`, or a source reference
  carrying any of these members, refuses as `unknown_member` at the first such
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
refuses as `malformed_wire` at that member. A `lock.sources` row with an empty
`authority` or `identity` refuses as `malformed_wire` at that member. A
`source_map` region or
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
  only when it equals a `lock.sources` row in both members; any other
  refuses `invalid_source_map` at that `source`.

The reader reads the catalog's `law_roles` entries as `{authority, identity}`
and matches by those two members. The production catalog is a build-time input
and not a caller's bytes, so an entry of another shape is never a package
refusal. The catalog read is a function over supplied bytes that returns an
error naming the unreadable entry, so that a test can supply bytes in which an
entry carries `revision`; the production catalog is read through it once, and
only that one-time read of the build's own bytes may stop the process
(FR-038-AC-58).

### Selections bind by identity

A `DependencySelection` is exactly `{identity}`. Its identity is the
non-empty library name imported by the unit; no version, package id, digest,
hash or other content seal is recorded. The host supplies one admitted package
per selected identity. A package binds to the entry when its required `name`
equals that identity. A `model_root` owner binds the selected document under
its identity when the document's own `package.identity` equals it; a model
owner joins the root with the same identity. Source and definition references
are exactly `{authority, identity}`.

A second model root of one identity refuses `invalid_semantic_graph` at its
later owner. A repeated dependency identity refuses
`invalid_package`/`conflicting-definition` with both entries as loci. A
selected identity with no supplied package refuses
`missing_import`/`missing-selection` at its entry. No comparison against a
recorded content value occurs. The reader SHALL refuse an extra old `version`,
`package_id`, `digest` or `identity_preimage` member under the closed wire
rather than silently drop it.

### Model-owned members

A `field` or `operation` member whose declaring node is a model declaration
node resolves through the domain package the lock selects, because that node's
body is `aggregate{[]}` and names no member (QSpec FR-322 "Model-owned
members" and "Reference conformance"; FR-154 for the declaration read). The
reader runs FR-322's four steps:

1. **Selection evidence**, before graph admission, for each `model_root` owner in
   ascending identity order: the document supplied under the identity, the check
   that the document holds no number whose text denotes a magnitude past 2^53
   and its own identity, in that order. The reader then reads the
   document's object types, record value types, systems interfaces, integer
   value types and relationships, node by node in ascending IR node identity,
   and refuses with the first declaration refusal at the root's `owner`
   (`/semantic_graph/nodes/<i>/owner`).
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
package's root `owner` with its typed code, cause and RFC 6901 pointer. This public
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

A V2 `relationship_end` member SHALL have the closed member shape
`{kind: "relationship_end", declaration, name, destination_type}` (and the
separately optional `population` link for `reaches`), with `declaration` naming
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

Under QSpec FR-322 "Destination type of a relationship end" and
FR-322-AC-270/271, `destination_type` is a required producer-supplied
structural `NodeId` `{package, path, ordinal}`. The strict reader SHALL
shape-check it at the V2 schema step; an absent value, a string value, an
extra member or a string `ordinal` refuses `malformed_wire` there. The reader
SHALL retain and return the supplied id verbatim, without deriving it from
the relationship or selected document and without requiring it to name a
node in the graph. It is not an application dependency, including when its
node is present. This field does not replace the relationship owner, role,
direction, endpoint type, multiplicity or subtype checks below.

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
unused relationship. Their outer pointer is the root's `owner`, `/semantic_graph/nodes/<i>/owner`.
The reader SHALL retain the selected declaration's authentic
identity and valid origin in the private intake refusal under the retention contract below. A nested relationship
refusal shall retain that relationship's context, not its enclosing object type.
The current public conversion preserves code, cause and the selection-row pointer;
it does not expose declaration identity or origin. A selected-document declaration
identity is not a checked graph-node id: `locus` keeps its existing separate
meaning. Private metadata retention is PLANNED / UNRUN until implemented and tested.
They keep FR-154's table order and
member-path order within a row: malformed declaration before unresolved end type, duplicate identity
before reversed multiplicity, and unresolved end type before reversed
multiplicity. A reference to a declaration refused for its identity or kind
keeps that declaration's own refusal, independent of node order; it SHALL NOT
be recast as a missing end type. Selected-document parsing and each
relationship node and member read are charged through the existing work meter
at the root's `owner`. Exhaustion of that selection-stage charge returns
`incomplete` there and no partial package. The complete read also charges
operation validation and later graph-body reference edges in its existing
order. A total-reader work limit one below the measured successful total
reports the first unpayable charge at its own stage pointer, which may be a
graph argument target; it does not necessarily report the root's `owner`.

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
an artifact title, member lookup, selection-row pointer, graph node id or enclosing
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

When byte, package-shape or graph admission fails before identifying a
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
of that return to `ValidationFailure` carries code, the primary location's
path and cause only. The
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
node and member) are charged to the `work` limit at the root of the package
they belong to, so an exhausted limit is `incomplete` with the pointer
`/semantic_graph/nodes/<i>/owner`. Each model declaration node key computed in step
2 is one validation visit at its root.

### Dependency selections

`lock.dependency_selections` is an array of closed `{identity}` entries in
strictly ascending UTF-8 byte order of identity. An empty identity refuses
`malformed_wire` at that member. A repeated identity refuses
`invalid_package`/`conflicting-definition` with the first and repeated paths;
a nonascending distinct identity refuses `invalid_package`/`invalid-value` at
its entry. Missing `identity` refuses `malformed_wire` at the entry; any extra
member refuses `unknown_member` at that member. These shape checks run before
dependency supply and graph checks.

For each entry in lock order the reader requires a supplied, admitted package
whose `name` equals `identity`. Absence refuses
`missing_import`/`missing-selection` at the entry. A supplied package with a
nonmatching name does not bind that entry. The host supplies exactly one
package per identity. The reader derives each dependency's transitive closure
from its own lock, without a closure list in the importing lock. If two paths
reach the same identity with structurally unequal complete JSON packages,
they are conflicting definitions and the reader refuses
`invalid_package`/`conflicting-definition` with both complete dependency paths.
Structural equality compares object members by name and arrays by element in
order; no hash is computed to decide it. If the packages are structurally
equal, the diamond is admitted. The reader reports the first failing entry in
lock order and no later one. Every direct entry is supplied before any
`dependency_reference` check.

The reader reads no source bytes or replay `byte_provision`. A missing domain
document still refuses at its model-root owner. There is no
`stale_dependency`, `content-mismatch`, `digest_domain_mismatch` or
`byte-digest-mismatch` refusal in this V2 read surface.

### Local protocol references

The V2 reader SHALL admit `node_tag: "protocol"` with
`semantic_form: "protocol_reference"` as `ProtocolForm::ProtocolReference`.
This carrier has one or more occurrences, all of role `expression`, no
declaration or owner, and a closed body
`{term: "reference", target: NodeRef}`. The carrier MAY carry a
`recursion_group` only when it is a member of a cyclic component; an acyclic
carrier with a group label fails `malformed_wire` during graph validation. When present on a cycle member, it SHALL satisfy FR-322's shared-label and cycle rules; its structural `node_id.ordinal` remains producer assigned and every reference names an exact node ID. A cyclic component containing
protocol-reference carriers SHALL carry one shared group label for every
component member, under FR-322's cycle rule. For a predicate-join function
that references its own protocol, the complete component, including
non-protocol nodes, follows QSpec FR-464-AC-15. The body target is an exact
typed structural node id in this package's own
`semantic_graph.nodes`. The carrier's `dependencies` SHALL contain exactly
that target, and its `semantic_type` SHALL equal the target protocol node's
`semantic_type`. The body and target participate in the carrier's structural path and complete package value. The carrier has no
separate collection or package selector; a selected dependency cannot satisfy
its local target. A `dependency_reference` body or an extra body member fails
the closed V2 schema.

Every source reference to the same target SHALL share the one carrier node,
with a distinct occurrence ordinal and its own `source_map` row keyed by
(`node_id`, `expression`, `ordinal`) naming that reference's exact source
regions. Occurrences and source-map rows are retained in the complete package value, but do not change the carrier's structural node id. Adding a second source reference and its source correspondence SHALL preserve the carrier `node_id` and package name; it SHALL NOT introduce a discriminator or another carrier node with the same id.

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
`invalid_semantic_graph` at the carrier's `/dependencies`. After graph-shape checks, the declaration stage SHALL join carriers in
ascending structural node-id order, before frame and operation checks. A target
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
hybrid node's `dependencies` with the unique structurally ascending set of its
body's reference targets and refuse a mismatch as `invalid_semantic_graph`
at `/dependencies`. Each hybrid node SHALL use its retained structural id, be its own `semantic_type`, and pass its declaration, owner and complete body checks (null declaration and absent owner for a transition). Changing a mode, rate, guard, reset, variable class or initial entry changes the complete package value; it does not trigger content-key authentication.

The hybrid body step SHALL run after the step-relation step and before the
protocol step, as QSpec FR-322 "Body steps" orders them. It visits models
in ascending structural node-id order. For each model, it SHALL stop at the first
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
`invalid_semantic_graph` at that node in ascending structural node-id order.
Then hybrid claims are visited in ascending structural node-id order. The
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
`hybrid`-classed literal-one rate, and match the strict-wire and graph-shape
negative vectors by code, cause and locus. It SHALL neither copy those
artifacts into this repository nor claim the process-provider or result-trace
checks owned by QSpec FR-481 as IR reader evidence.

### Timed-model nodes and reader step

The V2 reader SHALL admit the `timed_model` node family and exactly the
`timed_model`, `time_source`, `clock_place`, `clock_reset`, `clock_guard`,
`time_invariant`, `urgency` and `clock_constraint` forms owned by QSpec
FR-322-AC-120. The published QSpec schema fixes each form's closed wire
shape and occurrence roles; each timed node's outer `semantic_type` is its
own structural id, as FR-322-AC-122 states. The IR reader SHALL
decode the family and form once with the other node kinds. Strict wire
validation SHALL refuse an unknown form or malformed member before graph
or body validation. The canonical writer SHALL emit each complete timed node
once in `semantic_graph.nodes`, without a second body or identity projection.

For the published body layout, a `timed_model` root carries the ordered
bindings `model`, `time_dimension`, `source`, `clock_places`, `resets`,
`guards`, `invariants`, `urgencies`, and `constraints`. A tick `time_source`
carries `kind`, `unit`, `model`, `operation`, and `period`; its explicit
`model` reference equals the root model, and its operation references a
`state`/`operation_anchor` whose context is that model. A `clock_place` carries
`model`, `path`, and `resolved_path`; a `guards` binding on the place SHALL
refuse at strict wire validation. The timed root retains its `guards` aggregate
as the sole list of `clock_guard` nodes. The resolved-path table has one
ordered row per path step and retains the declaring type, member identity,
member type, collection kind and bounds; the terminal member type is the
native `Clock` marker, possibly under an `Option` wrapper that contributes no
path step. These members are checked package data and are validated
against their path steps before semantic graph admission. For a source-owned
model node, the path's root name comes from its declaration's qualified name;
for a selected model-owned node, it comes from the explicit `owner.node`
identity. The reader compares this identity directly with each resolved row's
declaring type and member identity; it does not infer a type from a display
name or consult source text.

A `clock_place` is one path template for all its state instances. For each
`slot` step, the reader SHALL require a symbolic selector that references the
selector-domain type node: the package's `Integer` type node for `Sequence`,
`Set`, `Bag` and `OrderedSet`, or the key type node for `Map`. Each collection
level contributes its own slot step. The reader SHALL resolve collection and
`Option` field types from the selected model declaration when checking every
`resolved_path` row. An `Option` wrapper adds no path step, and a terminal
`Option<Clock>` row resolves to `Clock`. A slot selector that is a literal,
an expression node, or a type node other than the selector domain SHALL refuse
`invalid_package`/`malformed-clock-place` at the slot path step. A scalar map
key type is admitted only when the exact key type node is selected; unresolved
or unsupported key domains refuse the place rather than defaulting to Text.
The reader SHALL admit an empty `guards` aggregate on a timed root when the
model has no guard. It SHALL refuse any timed node absent from its root's
corresponding aggregate as `invalid_package`/`timed-node-reference` at the
unlisted node's own `body`, visiting unlisted nodes in structural node-id order.
For two distinct place nodes of one model and path, the `resolved_path` rows
are determined by the selected declaration. If their aggregate order is valid,
the second node must carry a forged row and SHALL refuse `invalid_package`/
`malformed-clock-place` at its first differing `resolved_path` row,
`/semantic_graph/nodes/{i}/body/members/2/value/members/{j}`. Row joins run
after aggregate order and before the repeated-path check. The
`duplicate-clock-place` cause remains the refusal for a repeated path whose
rows all join; no conforming package can reach it, because two such nodes have
one body and key, while listing one node twice fails aggregate order first.

For each timed root's `clock_places`, `resets`, `guards`, `invariants`,
`urgencies` and `constraints` aggregate, each guard part's constraints and
each invariant's bounds, the reader SHALL require strictly ascending order
by the referenced node's minimum (`source.identity`, `start`, `end`) triple
across all of its `source_map` regions, followed, on a tie, by QSpec
FR-322-AC-265's structural member key. Source identities compare as UTF-8
bytes and positions as integers. A place's structural key is its path steps;
a constraint, reset, invariant, guard or urgency uses its specified form,
operation and parameter names, and structural terms, with comparison rank
`lt`, `le`, `eq`, `ge`, `gt` and numeric rational bounds. Terms SHALL be keyed
by form and body shape with references replaced by target structure, never
by a shared term's region, type reference, structural node-id or package name.
For a place path, `resolved_path` and the slot selector do not enter the key.
Equal keys or a member below its predecessor SHALL refuse
`invalid_semantic_graph` with no cause at that member's exact aggregate
reference locus. An empty or one-member aggregate is in order. This check
SHALL run after strict wire and graph-shape checks and before
unlisted-node, per-node join and duplicate-clock-place checks. A repeated
clock path in two distinct nodes remains subject to aggregate order before
its row join; equal order keys refuse there before the forged row is read.

The timed reader SHALL validate the closed clause layouts of QSpec
FR-322-AC-261: `clock_constraint` of kind `place` has `kind`, `place`,
`owner`, `slots`, `comparison`, `bound`; kind `parameter` has `kind`,
`operation`, `parameter`, `comparison`, `bound`. `clock_reset` of kind
`place` has `kind`, `operation`, `place`, `owner`, `slots`, `binders`,
`condition`, `value`; kind `parameter` has `kind`, `operation`,
`parameter`, `value`. `clock_guard` has `operation`, nonempty `parts`,
each part a Boolean `predicate`, an optional `binders` aggregate immediately
after it, then at least one constraint reference. `time_invariant` has a
Boolean `predicate`, an optional `binders` aggregate after it, and nonempty
`bounds`. Each binder is a reference to a slot `parameter` node. The five
comparison words include `eq`; invariant bounds admit
only `lt` and `le`. Empty parts or bounds, an empty part constraint list,
an unknown comparison, a parameter reset with `condition`, and a place
with `guards` SHALL refuse at strict wire validation before any timed join.

After aggregate order the reader SHALL find all unlisted timed nodes before
joining any timed node. It SHALL then join timed nodes by ascending structural node id and apply model-wide checks last. An operation binding of a guard,
reset or parameter constraint SHALL name an `operation_anchor` of the root's
model with no execution point; a frame, a foreign-model anchor or an anchor
with a point SHALL refuse `invalid_package`/`timed-node-reference` at the
offending binding. A `parameter` SHALL name a declared `Clock` parameter of
that operation. A place clause's owner and ordered slots SHALL fit its place
template; a reset's binders SHALL be slot terms in path order, its condition
Boolean and its operation frame SHALL modify the place's first field. A
guard part SHALL use a constraint of the same operation, and an invariant
SHALL use only place constraints. A second guard for one operation and a
listed constraint named by no guard part or invariant SHALL refuse at QSpec
FR-322-AC-263's loci. For a guard part or invariant with `binders`, each
binder SHALL resolve to a `parameter` node used as a slot term of the
corresponding place constraint, occur once, and follow the place's slot
order. The first non-slot, duplicate or out-of-order binder SHALL refuse
`invalid_package`/`timed-node-reference` at that binder reference's `target`
path, with the guard or invariant node as locus. Guard-part binders scope
that part's predicate and constraints; invariant binders scope its predicate
and bounds. An unquantified part or invariant retains its existing layout
without `binders`; the reader SHALL not turn a quantified clause into an
unquantified approximation. QSpec's
`proposals/checked-package-v2/timed-clause-node-vectors.json` cases
`CL-57` through `CL-60` and `CL-62` are the conformance inputs, read from
QSpec rather than copied into this repository. `CL-55` and `CL-61` are
schema-invalid reset and operation-anchor controls outside this binder join;
the strict wire stage continues to refuse them before timed validation.

The existing one-operand `quire.op.option.present` application over an
Option-typed term SHALL retain its meaning and key. The reader SHALL also
admit its `presence_read` place form with Boolean result and the ordered
arguments: a `clock_place` template reference, an owner term of the place's
owner object type, and one slot term per slot step in path order, each typed by
that step's selector domain. The place identity is the owner, place node and
slot terms together; neither a presence bit nor an `Option<Boolean>` is added
to any place, parameter, model or type node. A quantified clock-collection
binder SHALL have the selector-domain type, rather than the element value
type, when it supplies a slot term; `present(e)` and other reads of that
element place SHALL use the bound selector as the slot term. The reader SHALL
reject a wrong argument count with `ill_typed`/`operator-ineligible` at
`/body/arguments`, a non-place first argument in the place form with that pair
at `/body/arguments/0`, and a wrong-typed owner or slot term with
`ill_typed`/`type-mismatch` at the offending argument.

Graph-shape validation SHALL compare each timed node's dependency set with its body references before the timed-model body step. A mismatch SHALL refuse at the graph-shape stage. Each retained structural id, owner, declaration and complete body is checked under its independent rule; reordering source occurrences does not alter a node's structural id.

The timed-model body step SHALL run after the hybrid step and before the
protocol step. It SHALL visit timed nodes in ascending retained node-id
structural id order and reconstruct each selected model from its one root, source,
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
operators. It SHALL distinguish ordered lists from sets sorted by structural node id, sorted case and branch records from ordered captures, optional Groups
of zero or one member from required Groups, and enumerated text and interval
terms from unrestricted terms. A member kind other than `profile_operator`,
or an operator outside the selected profile, refuses `invalid_package`/
`operation-member-mismatch`. A wrong binding name, shape, position or
enumerated spelling refuses `malformed_wire`; a repeated or misordered set or sorted
record refuses `invalid_semantic_graph`. Application nodes SHALL retain their structural ids, with declared paths and reference targets checked by their independent graph and body rules. Source order and occurrences do not compute ids.

Before any protocol body step, the reader SHALL rebuild each protocol-family
node's complete `dependencies` list from its body as written, under QSpec
FR-322's rules and FR-464 through FR-466's form-specific lists. This includes
`protocol_clause`, `role`, `channel`, `queue`, `obligation`, `control`
and `compensation` nodes. An existing local body target omitted from that
list, or any extra target, SHALL refuse `invalid_semantic_graph` at the
referencing node's `/dependencies` during the graph-shape stage, before a
protocol-body refusal. A body target that names no node but is listed in
`dependencies` SHALL pass that list comparison and refuse
`missing_declaration`/`missing-name` at the target during the applicable
body join. Neither case SHALL expose a partial activation relation.

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
decision facts. The progress check SHALL follow QSpec FR-464's nested-repeat
rule. A nested `repeat` contributes a necessary step to an enclosing
`repeat` exactly when its guard has no false valuation over independent
atoms, its `maximum` is absent or at least 1, and its own body necessarily
steps. The `exhausted` control never contributes a step. Entering a nested
`repeat` from outside resets its counter, including on each iteration of
the enclosing control. A sequence needs one stepping child, a choice needs
every case to step, and `parallel` and `await` supply their own steps.

The reader SHALL build the nested guard's atoms from every `visible` entry
in set order, including unread entries, then from every other maximal
subterm that is neither an inline or referenced Boolean `value`/`literal`
constant nor an application of `quire.op.boolean.not`, `.and`, `.or` or
`.implies`. It discovers those subterms by a depth-first walk that visits
the last connective operand first. A listed expression stays one atom; its
operands are not descended into. Equal node ids denote one atom, while
reversed operands of `quire.op.boolean.eq` remain distinct atoms. The
reader treats the atoms as independent, enumerates valuations from all
false as a little-endian counter with the first atom least significant,
and stops at the first false guard valuation. It adds no wire member and
compares neither guards nor guard and body.

After the causal-cycle check and before any progress refusal, the reader
SHALL run this valuation check on every `repeat` inside another `repeat`'s
`body`, however deep and even with `maximum: 0`. A `repeat` in `exhausted`
or outside every other `repeat` body incurs no valuation charge. Atom
collection and comparison charge nothing; each valuation charges one
`CheckedPackageReadLimits.work` unit before evaluation. A denied charge
returns incomplete `stage_limit_exceeded`/`work-budget-exceeded` at the
nested control, with no progress refusal. When valuation completes, the
reader reports the first nonprogressing `repeat` in tree order as
`unsupported_construct`/`declaration-form`, naming it and the selected
protocol profile. A causal cycle wins over both valuation and progress.

After every protocol node passes these checks, it SHALL
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
structural node-id order: the protocol clause key, `origin` or `each` with its
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
ordered first by protocol clause structural node-id and then by required temporal
clause structural node-id. `CheckedProtocolActivationRow` SHALL have exactly the
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
term vocabulary, decoded once with the other term tags. `package` is the selected dependency package name and `node` a structural node id; a term that
is not exactly that shape refuses `invalid_semantic_graph` at `package`, at
`node`, or, for a missing or extra member, at the term.

The term is a callee only: argument 0 of a `quire.op.function.call`
application. It is retained as an exact body reference; a change to `package` or `node` changes the referenced target but does not re-derive the enclosing node id, and it contributes
no entry to the node's `dependencies`; listing its target there refuses
`invalid_semantic_graph`. The `function` operand family of a call is met by a
`dependency_reference` callee.

The reader checks each term at its place in its node's pre-order term walk,
wherever it stands (an application argument, a binding value, an aggregate
member or the body root), in step 7 of the operation stage, after the
application's own structural-id shape, law, mode and member checks and before its operand
checks, by these three checks in this order:

1. `package` equal to no `dependency_selections` entry's `identity` refuses
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
`relation` node, and a node with a declaration owned by a domain package,
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
  and the decode and before graph and body
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
     graph and body checks). Merged FR-370 refuses a bound
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
     `.../lower/numerator`, and the rational value checker names `.../numerator` today).
  2. **Lowest terms, a graph check.** A bound that matches the pattern and is not in
     lowest terms (`{numerator: "2", denominator: "4"}`) refuses
     `invalid_semantic_graph`, as every non-normalized rational does (merged FR-370
     and FR-322-AC-10). It runs after strict wire validation of every node and after the
     graph and body checks, as the graph checks do, as the last of
     the graph checks of the nodes that run before the frame step, after the application
     dependency join and every other `invalid_semantic_graph` check of the nodes
     those checks make, and ahead of the frame and state-clause step and the
     temporal step; it runs before the model-selection owners step (the derivation
     of each selected model declaration's node key, which can refuse
     `invalid_semantic_graph` at the root's `/semantic_graph/nodes/{i}/owner`), an IR reading, so a
     package with a non-reduced bound and such a selection defect refuses at the bound, at the first such bound in ascending
     structural `node_id` order and
     then member order, located at the bound (the same pointer as stage 1). So a
     package with a non-reduced bound in one node and another defect of those earlier
     graph checks in another node refuses at the other defect, while the frame
     step's own `invalid_semantic_graph` refusals (a frame array out of canonical
     order, at the frame body, FR-038-AC-14) are later and lose to the bound. So a
     pattern failure in any node, in any bound, is refused `invalid-value` ahead of a
     non-reduced bound in any node; the stage, the position among the graph checks and
     the locus are an IR reading, merged text stating only the code. The GCD work is charged to the work limit at the node's
     body, as other reduced-rational checks charge their
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
  target}` with `fairness_kind` `weak` or `strong` and `granularity`
  `whole` or `each`. The closed STD-616 target is exactly one of
  `{kind: operation, declaration, name}`, `{kind: protocol_root, node}`,
  `{kind: attempt_node, node}`, `{kind: parallel_branch, parallel, branch}`,
  `{kind: compensation_template, node}`, or
  `{kind: hybrid_transition, declaration, name}`. Each identity contributes to
  the application's exact dependency join. The canonical typed operation read
  preserves the target without a flat-form compatibility path. A
  missing member, a member of another kind, an unknown member name or a value
  outside those words refuses `operation-member-mismatch` at `operation.member`.
  A `fairness` member on any other entry refuses the same way.
  Protocol-root, attempt and parallel-branch targets resolve through the
  protocol clause's retained `requirements` relation and the same membership
  used by protocol controls. A compensation-template target in an admitted
  external temporal clause's fairness argument may instead resolve through
  the template's unique owning protocol clause, even when that clause has no
  `requirements` back-edge to the temporal clause. A fairness node with no
  admitted temporal clause reference remains unowned by a fairness argument;
  a valid compensation target alone does not admit it. Package identity and
  same-clause ownership precede kind/operator checks and branch-label
  membership. An absent or foreign target, or a target of another clause when
  the temporal claim is requirement-linked, refuses
  `missing_declaration`/`missing-name` at `target.node` or `target.parallel`;
  a local owned target of the wrong kind or operator refuses
  `invalid_model_binding`/`malformed-declaration` there. An absent branch label
  refuses `missing_declaration`/`missing-name` at `target.branch`. A root names
  the owning clause, an attempt names an `attempt` control, a parallel branch
  names a `parallel` control, and compensation names a compensation template.
  Hybrid targets retain the FR-481 transition join under `target`.
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
no arm. An `expression`/`case` node and the other forms carry the structural `node_id` every node carries.

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
(1) is checked over the whole graph first, in ascending structural `node_id` order;
the clause checks (2) then run clause by clause, in ascending structural `node_id`
order of the clause node, and each clause is taken through its four checks (`over`,
fairness resolution, profile fit, interval bounds) in that order before the next
clause is read, as QSpec FR-370 "Reader order" nests them, so a earlier-structural-id
clause's profile-fit defect is reported ahead of a later-structural-id clause's `over`
defect:

1. **Placement of every temporal application**, in ascending structural `node_id`
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
     operation target's `name` by FR-322 "Model-owned members" steps 2 and 3 among the
     effective operation members of its `declaration` node, the set QSpec FR-362
     resolves a fairness constraint against (own and inherited, redefinitions
     applied), and state, for each fairness member reached through the clause's
     fairness argument, in this order: (1) the `declaration` target is a `model` node
     of `semantic_form` `object_type`, else `invalid_model_binding`/
     `malformed-declaration` located at the target, with path
     `/semantic_graph/nodes/{fairness node}/body/operation/member/target/declaration` and the
     `declaration` target's key as locus ("Path and locus" below); (2) the owner is
     recovered, else FR-322 step 2's own refusal, `missing_declaration`/
     `missing-selection` for a node no selected owner matches and
     `invalid_semantic_graph` for a duplicated structural node id; (3) exactly one
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
     every clause has been read, in ascending structural `node_id` order (an IR reading;
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
| fairness `declaration` not a `model`/`object_type` node, including a target that names no node (`malformed-declaration`, FR-370 "Fairness resolution" step 1, "located at the target") | `/semantic_graph/nodes/{fairness node}/body/operation/member/target/declaration` | the `declaration` target's key as named (`member.target.declaration`) |
| fairness `name` matches no effective operation (`missing-name`) or two or more (`ambiguous-name`) | `/semantic_graph/nodes/{fairness node}/body/operation/member/target/name` | the `declaration` target's key |
| fairness declaring node's owner not recovered | `/semantic_graph/nodes/{fairness node}/body/operation/member/target/declaration` | the fairness node's key (the calling node, as FR-038 states for a model-owner refusal) |
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
| a timed bound not in lowest terms (`invalid_semantic_graph`, graph check after strict wire and structural graph validation; an IR reading of stage and locus) | `/semantic_graph/nodes/{formula node}/body/operation/member/interval/lower` or `.../upper` | the formula node's key |
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
nested application, at strict wire validation and ahead of structural graph checks; only
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
structural `node_id` order, each node at its place in that order, and this reader follows
it: case placement is NOT part of the temporal step's placement pass, which covers
the three temporal classes only. The order consequence: a defect of the temporal step
(placement, `details`, clause checks) is reported before any operation-step defect,
whatever the structural id order, so it is reported before a `case` placement defect; among
operation-step defects the lowest structural `node_id` is reported, and case placement is
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
walk ahead of structural graph checks ("The flat wire"):
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
names both forms; the reader admits them in the exact QSpec FR-322 shapes and
checks the compound unit's derived shape independently of graph dependencies.

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
(whose declaration is an independent node), binding "exponent" = an `integer`
literal typed at a `scalar_type`/`integer` node whose value is a canonical
nonzero decimal string]}`, strictly ascending by unit node key. Its
`dependencies` is exactly those unit keys in that order. The empty body is
the dimensionless unit.

A node whose body is an `application` term has
`dependencies` exactly the unique, structurally ascending reference targets and
operation member declarations of its body (FR-322); a `result_type` or literal `type` is not a
dependency. An application stands only at a body root ("The flat wire"), so the
join applies to the nodes whose body root is one. A member `declaration` that is not
a node key is left to the operation stage's refusal.

A parameter body or a graph dependency mismatch refuses `invalid_semantic_graph`
at the member that breaks that graph rule, located at the offending node; a
`declaration` on either form follows the declaration rule above. A compound
unit's term order, term kind, root-unit identity, nonzero integer exponent or
semantic dimension mismatch instead refuses `invalid_package` with cause
`invalid-derived-shape` at that compound unit's `node_id`, under QSpec FR-322
"Derived type forms and forwarding" (including `RM-CU-ORDER`). The dependency
join remains an independent earlier graph-shape check; a compound unit with
both defects refuses at `/dependencies` first. The reader does not re-derive
either form's structural node id from its body.

### Wire owner on checked nodes

FR-038-AC-153 through FR-038-AC-155 own the owner-wire boundary.
IR-1141 replaces content-key authentication with structural node identities;
owner joining remains an independent admission step.

The optional `owner` member of `CheckedSemanticNodeV2` has QSpec FR-322's closed `NodeOwner` shape:
`SourceOwner` `{kind: "source", authority, identity}` or `ModelOwner`
`{kind: "model", identity, node}`. The owner is part of the node's declared meaning; changing it can change
its structural package or path and always requires the owner join to be checked.
A node is structural for this presence rule when it is not an application. Every declared structural node SHALL carry the declaring
unit's `SourceOwner`. An undeclared `model`/`object_type`,
`model`/`systems_interface`, `relation`/`relationship`,
`relation`/`population` or `function` node
SHALL carry its `ModelOwner`. Every other node SHALL omit `owner`: this
includes anonymous types, undeclared values and parameters, nominal nodes with their own closed owner rule and application nodes. A
`DefinitionOwner` is not a node owner. The reader SHALL derive no missing
owner from occurrences, the source map, the lock, or node position.

The strict wire/schema step SHALL refuse a missing required `owner` as
`malformed_wire` at the node object that lacks it. It SHALL refuse an owner on
a node that must omit one, a wrong owner kind, `null`, a wrong member type or
an unknown owner member as `malformed_wire` at `owner`. An added identity projection refuses as an unknown top-level member. There is no admission route for an owner-less
declared or model-owned structural node.

The owner-join step SHALL visit owner-bearing nodes in ascending retained
structural node-id order, before derived-shape semantic checks. A `SourceOwner` SHALL
match a `lock.sources` row by (`authority`, `identity`). Every source-map
region of that node's `declaration` occurrence SHALL name that same source
pair; a mismatch SHALL refuse `invalid_package`/`invalid-value` at the
occurrence's source-map entry, even when the other source is lock-selected.
A `ModelOwner` SHALL match a `model_root` owner by `identity`, and
its `node` SHALL resolve in that package's admitted document to the kind
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

An owner that passes this join proceeds to body and member validation.
`correspondence`/`abstraction_relation` retains FR-346's body check.
`source_locus` remains a structural form, and applications remain owner-free.

### Declared population references

QSpec FR-322 "Population references" and FR-322-AC-82/83 own this reader
boundary. A declared population is one `relation`/`population` structural
node with a required `ModelOwner`, no declaration, its
own `node_id` as `semantic_type`, an empty aggregate body and no dependencies.
The reader validates its structural id and the same fixed members as every other model declaration node. The
owner's `identity` selects a domain document, and its `node` must equal the
`identity` of a raw population record in that document's `populations` list;
otherwise the owner join refuses `missing_declaration`/`missing-selection` at
the node id.
A selected `populations` entry that lacks an `identity` string or carries a
`node` member refuses `invalid_model_binding`/`malformed-declaration` at the
model selection before the owner join.
The declared population id uses the owning package name and its FR-322 declaration path; the owner still joins by `owner.identity` and `owner.node`. The node carries no
member type, extent or maximum. An absent owner refuses `malformed_wire`
before the owner join.

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

### Structural node ids and closed bodies

Every V2 node id is the closed structural `{package, path, ordinal}` of QSpec
FR-322. The reader SHALL validate its shape, uniqueness, declaration-path
agreement, owner and reference joins; it SHALL NOT authenticate an id by a
preimage, digest or content seal. A node body remains independently subject
to its closed grammar and semantic checks. A changed body under an unchanged
structural id can be valid when the new body is well typed and all joins pass;
invalid content refuses under its body or graph rule, not a stale-key rule.

A declared source-owned node has `path` equal to its
`declaration.qualified_name` and `package` equal to the containing compilation
unit's name. A model-owned declaration has the owning domain package identity
as `package`. A derived ungrouped node has the QSpec path
`[node_tag, semantic_form, ...operands]`; each operand is a string or a closed
`{"node": i}` index into this document's graph. A declared recursion-group member keeps its own declaration path; an
anonymous derived member uses the sole declared member's path when exactly
one exists, or a producer-chosen path when zero or multiple exist. No grouped
member embeds operands in its path (QSpec FR-322-AC-274; TC-946 K14).
Its structural `node_id.ordinal` remains producer assigned. The shared
non-empty `recursion_group` label identifies membership; references to members
use exact structural node IDs. No group-position carrier is written or derived.
Each ungrouped node likewise retains its producer-assigned ordinal;
it may be nonzero even when no other node shares its package and path. The
reader checks shape and uniqueness among nodes sharing that package and path,
without inferring a missing ordinal, contiguity or sibling order. An imported node
keeps the dependency package's name. The reader's deterministic visit order
compares `package`, then `path` elements, then numeric `ordinal` as FR-322
specifies; graph-array order is retained.

The closed bodies remain: `scalar_type` `aggregate{[]}`; `reference`, `option`
and the four collections an aggregate of exactly one `reference`;
`integer_range` and `collection_bounds` an aggregate of exactly the bindings
`min` then `max`, each an `integer` literal typed at the `scalar_type`/`integer`
node. An `integer_range` decimal value matches `^(0|-?[1-9][0-9]*)$`; a
collection bound matches `^(0|[1-9][0-9]*)$`. The reader SHALL validate every
node of these forms, including unreferenced, declared, owned and grouped
nodes; a malformed body refuses `invalid_semantic_graph` at its offending
member. It SHALL not infer a bound from a node id or let an unrelated
referrer's type stand in for the node's own body. Bounds remain exact decimal
strings without narrowing to a host integer. A `scalar_type` or
`composite_type` of these forms is self typed; a `bounded_domain`'s semantic
type is the declared base type. Its semantic and dependency joins are checked
separately from identity.

A `bounded_domain`/`text_bounds` node SHALL have a `scalar_type`/`text`
semantic base. An integer-range base and body do not become text bounds when
only the form label changes. A wrong base refuses `invalid_package` with no
cause at the node's `body`, located at that node; its authored identifier
does not bypass this join. A valid text base retains the node's own body
without consulting another bound node.

A labelled recursion group contains every node carrying its shared non-empty
string label, including application and off-cycle members. Both in-group and
outside references carry exact structural target IDs. A repeated structural
ID, malformed structural ordinal, missing referenced member, unresolved
reference or invalid cycle label refuses under its applicable graph or wire
rule at the exact member. The former group-position, size and ordinal
permutation/refusal clauses are withdrawn in place: the checked wire carries
no such member and the reader derives none. Graph and body checks remain
subject to the caller's work and byte limits and run without input-dependent
call-stack recursion.

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
   admitted selected domain package document, a field table holding that
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
   root of its package, so that an exhausted limit is `incomplete` with the
   pointer `/semantic_graph/nodes/<i>/owner`.
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
    stage: after step 1 has admitted every selected package, in ascending
    identity order, and before the graph stage (`validate_graph`).
12. The reader shall retain the field tables, and an owned index from each
    selected declaration's model declaration node key to its package, form
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
   an object type's model declaration node when its structural `node_id` is the
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
20. The accessor shall return the fields in the retained table's
    `ordered_exposed` order: inherited fields in authored supertype order,
    then own declaration order, with a redefiner occupying the earliest slot
    of the field it hides and a diamond field appearing once. This is the
    declaration order the retained table already uses for binder types, not
    ascending field-name order; each exposed identity appears once.
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
    error. The lookup shall find the same item as `fields()` independently of
    the slice's declaration order; it shall not assume name sorting.
31. `CheckedModelField` shall expose `name()`, `member_type()`
    (`Option<&CheckedMemberType>`) and `declaring_node_id()` returning the
    authentic `&CheckedNodeId` of the model object declaration that owns that
    exposed field, including an inherited or redefining field. It shall never
    return the requested subtype's id merely because that subtype exposes the
    field.
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
    including each selected document's complete admitted value.
38. At admission, the reader shall retain an index from each selected
    model declaration identity to its zero, one or multiple genuine graph
    `model`/`object_type` declaration node IDs. For each exposed field, the
    accessor shall look up its selected-document declaring owner in that
    retained index and return the exact matched structural `CheckedNodeId`. No field declaration node is synthesized: a
    field is a member of its declaring object node, not an independent graph
    node (FR-040). QSpec FR-322-AC-30 permits a selected subtype's inherited
    field to resolve from the selected document even when the declaring
    supertype has no graph node. The accessor shall use the already retained field entries
    and selected declaration index, with no copied domain schema, alternate
    member-resolution walk, name-order fallback or compatibility accessor.
39. After `UnknownNode`, `NotModelObjectType` and `AmbiguousField` have been
    decided in that order, the accessor shall inspect effective fields in
    item 20's order. If an exposed field's actual declaring object type has
    no matching node in the admitted graph, it shall return the distinct typed
    `CheckedModelFieldsError::MissingDeclaringNode { field_identity: Box<str> }`
    with that field's selected IR identity. If two or more admitted object
    declaration nodes match that owner identity, it shall return the distinct
    typed `CheckedModelFieldsError::AmbiguousDeclaringNode { field_identity: Box<str> }`.
    The first such field decides the call. Neither error returns a partial
    field set, fabricates an id, chooses an ordinal or hides the field.
    Package admission remains governed by its existing owner and graph rules.
    CG's whole-object subject path can use only calls that return every field
    and its real declaring node.

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
`/semantic_graph/nodes/<i>/owner`, where it admits today; a chain of N = 1000 charges
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
caller's evidence supplied under the selected identity, whose name and declarations the reader
checked (FR-038-AC-27), so a package whose evidence holds a document declaring a
narrower range returns that document's bounds (FR-038-AC-129); which document is
authoritative is the caller's decision. For a field no read names, admission
checked only the selected document's identity and declarations (step 1),
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
   at admission, while the consumer already has the exact typed value it needs.
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
multiplicity, and the direction. It shall also retain the producer-supplied
`destination_type` from each admitted application's `relationship_end` member.
The accessor shall not derive that id from the selected document or resolve it
against graph nodes. The reader shall keep the relationship record outside
package equality, shall charge no work beyond what admission already charges,
and shall not change its refusal order. The accessor shall read the record and
the admitted member without a second derivation.

**Result.** `application` is the supplied id and `relationship` is the
`operation.member.declaration` node id the application names, so a consumer can
name the edge. `role` is the destination end's authored role, `None` when that
end carries none (a target role may be omitted, FR-038-AC-166); it is the role
the data holds and is never synthesised. `destination_type` is exactly the
structural `NodeId` the producer supplied in the admitted application's
`relationship_end` member. A subtype operand does not replace that member's
value. The named node may be absent from the graph, and the accessor neither
looks it up nor derives another id. `lower`, `upper` and `unique` are the destination
end's declared multiplicity and `ordered` its declared ordering, with no change
of value or type: `upper` is `None` when the declaration is unbounded. The
result carries no member type, charge, population, bound, evaluation or source
text, and no `serde_json::Value`.

**What an admitted application yields.** The reaches branch refuses an
unbounded or an ordered destination end (FR-038-AC-255), a destination end of
another type than the receiver endpoint (AC-256) and a traversal against the
declared direction (AC-254), so no node that survives admission has such an
end. For every admitted reaches application the accessor therefore returns
`ordered` false, `upper` `Some` and the member's producer-supplied
`destination_type`, regardless of whether that id names a graph node. The accessor does not itself re-check those
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
`select_population` returns `Finite` for a selected bounded `model_population`
and `AdmittedUnbounded` for a selected admitted `model_population` with no `max`,
retaining the selected node and `Type` occurrence in either case (QSpec
FR-322-AC-74). Nothing in the
admitted package tells a consumer which such node bounds the population a
reachability walks, and the IR must not choose one by object type: a
`Population<E>[N]` parameter type of the same `E` and `N` is the same
anonymous node (FR-053-AC-2), and a different `N` for the same `E` is another.
The producing language therefore names the node explicitly, and the reader
checks the name.

**The link.** A `quire.op.model.reaches` application MAY carry one additional
key in `operation.member`, `population`, whose value is the node id of one
`bounded_domain` / `model_population` node. The producer may name an admitted unbounded population with the key;
absence of the key means this accessor has no selected binding. The producing
language owns when to omit the key; an omitted key shall not be inferred from
an absent `max` on a selected node (QSpec FR-322-AC-74). The V2
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
  `reaches_population` shall return `Ok(None)`: the application names no
  population binding for this accessor to select. `reaches_population` shall not
  infer a population or a bound from an object type, a count of population
  nodes or the absence of a node. A domain-package declaration-only selection
  is a separate FR-053 input; this accessor does not construct it.
- Otherwise `reaches_population` shall return `Ok(Some(selector))` with
  `package` the package's own name, `binding` the `population` node id
  exactly as the member holds it, and `occurrence` the `Type` occurrence of that
  node with the least ordinal. The selector is exactly what FR-053's `Admitted`
  source accepts. After the reader validates the node's derived shape,
  `select_population` over the selector returns `Finite` with exact canonical
  nonnegative decimal `maximum` text and `minimum` zero when `max` is present,
  or `AdmittedUnbounded` with the same selected binding, occurrence, object
  type and authentic source regions when `max` is absent. The maximum has no
  `u64` cap (QSpec FR-322-AC-74). `reaches_population` shall itself read no `max`,
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
`check_relationship_member` returns, follow it. The structural dependency join (AC-276) runs before every operation-stage check.
For `reaches` and `navigate` the catalog gives no mode and no leaves, so a mode
or a leaf on such an application is refused by the strict body stage before
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
- The graph dependency join and owner/declaration checks precede derived-shape
  checking. A `model_population` body with no `max` is the admitted unbounded
  form. One canonical nonnegative integer `max` binding is the bounded form;
  `0` and exact decimal text above `u64::MAX` admit without narrowing. A
  negative or non-integer `max`, a second member, or a semantic type other
  than `set<reference<T>>` refuses `invalid_package` with cause
  `invalid-derived-shape` at the population node's `node_id`, under QSpec
  FR-322 "Derived type forms and forwarding", FR-322-AC-74 and `RM-POP-MAX`.
  A `max` term that is not a binding remains subject to the earlier graph
  term-shape refusal. The population link check does not impose a finite
  maximum or a `u64` projection limit. These corrected derived-shape checks
  are PLANNED/UNRUN until the IR reader implements them.
- When the node's object type is not exactly the declared type of the end
  named by the member's `name` in the selected relationship, the reader SHALL
  refuse `ill_typed`/`operator-ineligible` at `member.population`. The reader
  SHALL resolve that named role end in the selected relationship and use its
  declared type as the population comparand. It SHALL derive the population
  object type through FR-053's `population_object_type` once. The
  producer-supplied `destination_type` is retained verbatim under
  FR-322-AC-270 and SHALL NOT be resolved, derived or compared with the
  population object type; a `destination_type` naming `Shipment` does not
  change an `Order` role end's required `Set<Reference<Order>>` population.
  A subtype of the role end's declared type does not conform.

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
application's unique, structurally ascending dependency list when the application is a
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
| `scalar_type/enum` | `Enum` at this declaration node, retaining the selected enum declaration's `ordered` and `members` values; unordered members are byte-ascending, while ordered members retain semantic order |
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

### Application node identity

An application is a derived node with the structural id of QSpec FR-322:
`package` is the owning package name, `path` is
`[node_tag, semantic_form, ...operands]`, and `ordinal` is producer assigned
under the structural-id shape and uniqueness rules above; a recursion-group
member separately uses its graph-order member position. The body retains its exact operator, operation,
arguments and result type; the reader validates these under the application
body and dependency rules below. It SHALL not hash an application preimage,
refuse a stale key or expose a pre-read candidate-key helper. A changed
operand identity changes the structural path where that operand participates;
a changed body member that does not participate in the path is judged by the
body and graph rules. Source occurrences do not enter the structural id.

### Operation identity, laws, mode, member and operands

For a successfully admitted `CheckedApplicationOperation`, the public typed
operation views SHALL project only facts retained by the reader's validated
operation decode. `projection_field()` returns a pair of the field's
`CheckedNodeId` declaration and its exact name only when the member is a
catalogued `field`; `member_declaration()` returns the admitted
`CheckedNodeId` wherever the member has a declaration, independently of its
kind. Both return admitted `None` when their respective member fact is absent or
inapplicable. In particular, a non-field member cannot yield a field pair,
and a member with no declaration cannot yield a fabricated node id.

`mode()` returns an admitted `Option<CheckedOperationModeView<'_>>`, with the closed
`OperationModeKind` and the exact catalogued value borrowed from the admitted
mode. `None` means no mode; an operation with a different mode kind remains
distinguishable through the view's kind. The value is a text value because the
catalog owns its vocabulary; the public return is the typed mode view, not a
raw JSON value or an unconstrained mode getter. `law_definitions()` returns
an admitted view of the law definitions as typed `CheckedArtifactRef` values in their
admitted order; an operation with no laws returns an empty view. A consumer
matches both authority and identity when identity alone is insufficient to
identify a definition. These views do not re-parse the operation's JSON,
expose `record`, `get` or `pointer`, or alter admission and refusal order.
Each of `state_clause_kind()`, `projection_field()`, `member_declaration()`,
`mode()` and `law_definitions()` SHALL return a typed `NotAdmitted` refusal when
the operation has no validated operation views. The admitted value is distinct
from that refusal: an absent optional member returns admitted `None`, and no
laws return an admitted empty view. Direct deserialization of an operation
term is not operation admission. Neither it nor a term reconstructed from a
wire body without authenticated operation views may report admitted absence.
An operation rejected by the V2 reader has no public admitted view.

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
The original required-presence correction is implemented; the IR-1141 structural-id control is PLANNED/UNRUN through
`CheckedPackageV2::read` by
`operation_mode_and_member_are_required_even_when_null_is_admitted`. The revised test
retains valid structural ids for each omission, checks
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
  wire and structural graph checks, with the published validation order and resource
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
are structural identities, and one `Option` of `Text` serves every optional text
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
text leaf. A recursion leaf is required, and the complete `operation.leaves` value is validated independently of the retained structural id. A list written without a required recursion leaf, or with one where none is derived, is refused as below.

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
naming a malformed or unresolved structural node id refuses under the applicable graph-shape or reference-resolution rule at that entry. `deletes_scoped` entries are never node
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

The declaration stage ends with two name checks. A node carrying a
`declaration` (an enum declaration, a dimension or a declared unit) shall use its
declared structural path as `qualified_declaration`, else the reader refuses as
`invalid_package` with cause `declaration-nominal-mismatch`. Names are unique
within the types, dimensions and units namespaces: a unit, dimension and type
may share a qualified name. Two nodes in one namespace declaring the same
`qualified_name` refuse as `ambiguous_declaration` with cause
`ambiguous-name`, located at the later node in node-id order. QSpec FR-322
orders both before any frame
or operation refusal but not against each other; this reader checks the
nominal join first, each at the first offending node in ascending node-id
structural id order.

Once every node's identity is known, the reader shall evaluate frame-body
semantics as its own stage, in reader order graph-shape, declaration, frame, operation — immediately after declaration checks and
before the graph's dependency and body-reference edges are resolved, so a
frame's own `dependencies` entry naming no real node is this stage's own
refusal rather than the graph's generic unresolved-reference refusal. A
`creates` or `deletes` entry names a declared dependency of `model` form
`object_type` or `process`; a `modifies` entry's eligibility and field-name
resolution are [FR-040](./FR-040-admit-frame-entries-and-state-clauses.md)'s. This is the closed eligibility table over the family/form pairs
the contract's node taxonomy admits; a member and an entry's family/form pair
outside it is never admitted, regardless of the entry's own grammar validity.
An entry naming a structural node id that is not among the frame node's own
`dependencies` — including one declared but resolving to no node anywhere in
the graph — refuses as `missing_declaration` with cause `missing-name`,
located at that entry's own key. An entry that does resolve against a
declared dependency, but to a family/form pair the entry's member does not
admit, refuses as `invalid_model_binding` with cause `malformed-declaration`,
located at that entry's own key. Within one member array, entries shall
appear in strictly ascending order of their order key (for a `creates` or
`deletes` entry its structural node id, for a `modifies` entry FR-040's order key); an array not in that order refuses
as `invalid_semantic_graph` at the frame body path, located at the frame node
itself, carrying no cause.

A frame node carrying more than one defect refuses for exactly one of them,
selected by one precedence: any meaning-join defect (`missing_declaration` or
`invalid_model_binding`) outranks a canonical-order defect outright, so a
frame whose member order is wrong and whose entries are also ineligible
refuses for the meaning-join defect, never the order defect. Among
meaning-join defects, the member the defect's entry sits in decides first —
`modifies` before `creates` before `deletes` — and, within one member,
ascending structural entry id breaks the tie, except that a `conflicting-binding`
defect of two scoped `modifies` entries ranks after every other meaning-join
defect in any member (FR-040-AC-25); the selected defect therefore never
depends on where in its array an entry sits or on which member an author
wrote a defect into. A package carrying more than one defective frame node
refuses at the frame the reader reaches first in ascending structural `node_id`
order, reporting only that frame's own selected defect; a later frame's
defect, however it would otherwise rank, is never reached or compared
against an earlier frame's.

When a caller lowers requested items, the lowerer shall charge work per request and, for each visited reachable node, per node, per body term and per successor edge, return
`invalid_input` for an absent node key, `unsupported` when any reachable node's
tag is outside the profile, `requires_bound` when the profile requires bounds
and a reachable type is unbounded at a position or is recursive (both defined
below), and `failed` at the work limit. A lowered record
carries the node, its exact source-map entries, semantic type, reachable
dependency keys, bounding domain keys, reachable claim keys and no content-derived seal. A non-lowered record carries no node.
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
`dependencies`. The admitted lowered record retains all three lists as written; the reader compares the complete typed record and never computes an `ir_id` from them.

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

### Carried quantity type and conversion

The V2 reader SHALL admit the anonymous `scalar_type`/`carried_quantity`
form as a self-typed member of the `quantity` family. Its body is an aggregate
of exactly three ordered bindings: `unit` is a reference to a
`scalar_type`/`unit` node; `carrier` is a reference to a bounded
`bounded_domain`/`integer_range` or `/decimal_range` node; and `rounding` is
a `text` literal naming a rounding mode. The node has no declaration, owner
or recursion group. Its structural path is
`["scalar_type", "carried_quantity", U, C, "rounding", T, mode]`, with the
unit node id `U`, carrier node id `C`, text type node id `T` and literal mode.
The reader SHALL retain these exact bindings and distinguish otherwise equal
carried quantities whose unit, carrier or mode differs. The only required
`dependencies` are `U` and `C` in ascending node-id order; the text type of
the rounding literal is not an additional dependency.

For a decimal-range carrier, the carried node's rounding SHALL equal the
carrier's own rounding binding. An integer-range carrier permits any admitted
rounding mode, with `exact` for the mode-unspecified source form. A non-unit
target, unbounded or non-integer/non-decimal carrier, invalid mode, mismatched
decimal-carrier mode, extra or reordered body binding, non-self semantic type,
or path inconsistent with the body SHALL refuse
`invalid_package`/`invalid-derived-shape` at the carried node's `node_id`.
This is the QSpec derived-shape check, after the earlier strict-wire and graph
checks. It SHALL not reinterpret an unbounded scalar as a bounded carrier.

`quire.op.quantity.convert` SHALL admit a unit-typed or carried-quantity-typed
quantity operand and a carried-quantity result only when their units have the
same dimension node id and the operation's `rounding` mode equals the result
node's rounding literal. A missing or differing mode SHALL refuse
`invalid_package`/`operation-mode-type-mismatch` at the application before
the dimension check. A different dimension SHALL refuse
`ill_typed`/`operator-ineligible` there. Conversion to a unit node continues
to pin `exact`. A carried quantity used as an operand of another quantity
operation, including arithmetic, comparison or `collection.sum.quantity`,
SHALL refuse `ill_typed`/`operator-ineligible`; conversion from one carried
quantity into another remains eligible. The reader SHALL not decide value
conversion, rounding loss, carrier membership or evaluation resource outcomes.

The public `ScalarTypeForm` vocabulary SHALL expose `CarriedQuantity` for
the admitted form. An admitted node's existing typed body and node-kind reads
SHALL expose its exact unit, carrier and rounding bindings without making a
refused node available as an admitted value. No evaluation result type is
added to the IR reader by this requirement.

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

### Present checked-term data and probabilistic claim admission

The optional QSpec FR-322 checked-term-data sidecar is a closed, present V2
object with four required arrays: `equality_decisions`, `protocol_locations`,
`dispatch_tables` and `probabilistic_claims`. Each array may be empty. An absent
sidecar and a present sidecar with four empty arrays remain distinct on read
and write. A present object missing any array, or containing an unknown member,
is malformed wire at the missing object or offending member. The public typed
`CheckedTermDataV2` and private owned, raw-intake and borrowed-encode wire paths
use this same four-array shape; a nonempty `probabilistic_claims` array is not
refused merely for being nonempty.

The closed `ClaimForm` vocabulary includes `ProbabilisticClaim` for the
`claim`/`probabilistic_claim` semantic form. Its body is an ordinary retained
typed term under FR-322's claim-node body grammar; this requirement does not
add a new body production or infer a formula from the sidecar. The reader
applies the existing graph shape, structural identity, exact dependencies,
declaration, owner, occurrence and source-map checks before joining a sidecar
row. The writer preserves that node's complete body and structural identity.
The sidecar row carries only claim identity and occurrence.

Each probabilistic-claim row has exactly the required `declaration` and
`occurrence` members. `declaration` is the exact graph-local structural
`NodeRef` of one `claim`/`probabilistic_claim` declaration; `occurrence` is one
of that declaration node's own occurrences with role `claim`. The reader
resolves the declaration and occurrence without using display name, source
span, graph-array position or declaration order as identity. Every
probabilistic-claim declaration has exactly one row in a present sidecar.
Rows are in ascending structural declaration identity, then occurrence order.
The reader retains admitted row identities in the typed sidecar view and
charges each row to its sidecar work limit before inspection.

For an absent-local, foreign, wrong-kind or non-probabilistic declaration, the
reader refuses `invalid_package`/`invalid-value` at that row's `declaration`.
An occurrence absent from the named declaration, or one with a role other
than `claim`, refuses the same code and cause at that row's `occurrence`.
A second row for one declaration, including a second valid occurrence or the
same `(declaration, occurrence)` key, refuses
`invalid_package`/`conflicting-definition` at the later row. Reversed
otherwise-valid rows refuse `invalid_package`/`invalid-semantic-graph` at the
second row. A missing `occurrence` is a closed-schema `malformed_wire` refusal
at the missing member's published locus. Refusal loci follow the existing RFC
6901 location contract.

### Selected operation-effect carrier (planned until IR-1226 reader lands)

The strict reader SHALL admit QSpec STD-640's optional top-level
`operation_effects` member as a non-empty closed array of
`{document, declaring, operation, effect, provenance}` rows. If absent, the
admitted package SHALL retain effects as unavailable, and a request for an
operation effect SHALL return a typed unavailable-witness outcome under
[FR-354](./FR-354-project-admitted-runtime-context.md). If present, the
carrier SHALL contain exactly one row for each own operation of every selected
`model`/`object_type`, `model`/`process` and `model`/`state_machine` declaration;
an empty declared frame SHALL retain a row with all four arrays empty.
`model`/`persistence_interface` operations have no row. The row key is the
selected document identity, graph-local declaring `CheckedNodeId` and original
member name in that order. The reader SHALL expose admitted rows through a
borrowed typed read in carrier order, with the four exact `modifies`,
`creates`, `deletes` and `deletes_scoped` collections and provenance member.
An explicit `null`, empty array, or non-array value for `operation_effects`
SHALL refuse as `malformed_wire` at `/operation_effects`; neither is the
absent-member case.
The existing V2 encoder SHALL accept an optional borrowed collection of
constructible typed carrier rows through `CheckedPackageSidecarsRef` and
write the same closed top-level member in canonical order when supplied;
`None` omits it. An input row cannot mint an admitted effect view: only a
strictly admitted package may return the borrowed validated row. The
producer supplies all four entry collections and the selected document,
declaring node, original member and provenance member; the fixed provenance
origin is `model_document_operation_frame`.
It SHALL NOT derive an operation effect from a state/frame node, name, path or
default empty value, and SHALL NOT create an independently authoritative IR
effect mirror.

The reader SHALL apply QSpec STD-640's closed schema and check order to each
row after earlier wire and graph stages: selected document and local declaring
node; eligible declaration and exact own member; provenance origin/member;
FR-340 entry joins and canonical array order; equality with the selected
document's operation frame; repeated key before descending key; then complete
coverage. The first defect SHALL return QSpec TC-977's existing typed code,
cause and RFC 6901 locus without an admitted package. Each row and effect
entry SHALL charge the caller's finite validation-visit budget before
inspection; exhaustion SHALL return the existing incomplete outcome at the
first uncharged locus without a partial admitted effect view. Repeated reads
of one admitted package and its clone SHALL agree. TC-048 SHALL read
`proposals/checked-package-v2/operation-effect-vectors.json` from the selected
QSpec checkout by reference, checking OEC-01..07, OES-01..14 and OER-01..26
against their published admission or exact refusal oracle; IR SHALL store no
copy of that vector or schema.

### Authenticated schedulability claim carrier (planned until IR-1252 lands)

The strict reader SHALL admit the optional top-level `schedulability_claims`
index defined by [QSpec FR-322](ix://agent-ix/quire-specification/FR-322),
AC-300 and “Schedulability claim carrier”, and validate every
`claim`/`schedulability` body under
[QSpec FR-482](ix://agent-ix/quire-specification/FR-482). The closed
`ClaimForm` vocabulary gains `Schedulability`. The index is absent when there
is no schedulability claim occurrence; otherwise it contains exactly one
closed `{declaration, occurrence}` row for each such occurrence. The row's
`declaration` is a local structural `CheckedNodeId`; its wire `occurrence`
is the non-negative integer ordinal of that node's `claim` occurrence.
A shared claim node with two occurrences has two rows. The index contains
identity references only; the graph retains the complete claim body once.

After aggregate-order validation and before the FR-482 body step, the reader
SHALL visit index rows in array order. Before inspecting a row it charges one
validation visit against the caller's existing finite reader budget. It
checks the local declaration join before the owned `claim` occurrence join,
then repeated keys before canonical order; total occurrence coverage is last.
Canonical row order is structural declaration identity followed by occurrence
ordinal. Exhaustion returns the existing incomplete/resource-exhaustion
reader result at the first uncharged row, without an admitted package or
partial typed claim view. No new cap, budget or alternate admission path is
introduced. Wire validation and existing graph/aggregate stages retain their
earlier precedence; a combined-defect control makes each adjacent carrier
check's precedence observable.

A missing index or missing occurrence row refuses
`invalid_package`/`invalid-value` at `/schedulability_claims`. An absent,
foreign, wrong-kind or wrong-form declaration refuses the same code/cause at
`/schedulability_claims/{i}/declaration`; an unowned occurrence or an
occurrence of a role other than `claim` refuses there at `/occurrence`.
A repeated pair refuses `invalid_package`/`conflicting-definition` at the
later row, before the order check. A descending distinct pair refuses
`invalid_package`/`invalid-value` at the later row. Missing or extra row
members and negative or non-integer occurrence ordinals are `malformed_wire`
at QSpec TC-976's exact row/member locus. Every refusal uses the existing
ordered typed diagnostic `loci`, with the exact RFC 6901 primary pointer and
no admitted view.

The reader SHALL apply the complete FR-482 body checks in its published
order: shape, literals, form fit, task order, unit, then task-set premises.
This is the last body step, after the protocol step, visiting schedulability
nodes in ascending structural node-id order. The body has the ordered five
claim bindings and eleven bindings per task, retains exact rational time
values in one package time unit, and has Boolean semantic type, no declaration
or owner and empty dependencies. Admission checks the `schedulable` and
`response` forms; fixed-priority, EDF and AMC policy premises; task ordering;
non-negative BCET, jitter and blocking; positive period, deadline and WCET;
HI/LO work bounds; policy-specific priorities and EDF/AMC restrictions.
FR-482 owns the exact binding/literal/aggregate loci and typed codes and
causes for these checks; ordinary graph and term checks still precede them.
The reader preserves producer-assigned structural identities and does not
compute a body-derived node id.

The raw wire row `CheckedSchedulabilityClaimRow` has public
`declaration: CheckedNodeId` and `occurrence: u64` fields. It is an untrusted
producer value, not an authenticated view. The admitted package's
`schedulability_claim_rows(&self) -> Option<&[CheckedSchedulabilityClaimRow]>`
borrows its retained index for complete re-encoding under
[FR-051](./FR-051-encode-checked-package-v2-envelope.md). Complete read/encode/read
preserves every row and body and the absent member; the lossless reader check
includes this member rather than silently dropping it.

After full admission, `CheckedPackageV2::schedulability_claims(&self)` SHALL
return `impl ExactSizeIterator<Item = CheckedSchedulabilityClaimRef<'_>> + '_`
in canonical index order. Each borrowed view has private fields and no public
constructor; `declaration(&self) -> &CheckedNodeId`,
`occurrence(&self) -> &CheckedOccurrence` and `body(&self) -> &CheckedTerm`
borrow the exact admitted graph identity, owned claim occurrence and FR-482
body. Thus policy, task fields, units and response bound are available through
the existing typed term tree without copying the body or parsing raw JSON.
An admitted package without this index yields an empty iterator. The view
cannot outlive its package or be minted from a caller-asserted row. Repeated
reads are deterministic and perform no second validation or evaluation.

The borrowed claim view SHALL also expose `kind() -> CheckedSchedulabilityKind`
(closed variants `Schedulable`, `Response`),
`policy() -> CheckedSchedulingPolicy` (`FixedPriority`, `Edf`, `Amc`),
`response_task() -> Option<&str>`, `response_bound() -> Option<CheckedLiteralRead<'_>>`
and `tasks() -> impl ExactSizeIterator<Item = CheckedSchedulabilityTaskRef<'_>> + '_`.
The two response accessors are present exactly for `Response`. Each task view
has `name() -> &str`, `arrival() -> CheckedTaskArrival` (`Periodic`, `Sporadic`),
`criticality() -> CheckedTaskCriticality` (`Lo`, `Hi`), and immutable
`period`, `deadline`, `bcet`, `wcet`, `jitter` and `blocking` accessors returning
`CheckedLiteralRead<'_>`. Its `wcet_hi` accessor returns
`Option<CheckedLiteralRead<'_>>`, present exactly for `Hi`; `priority` returns
`Option<CheckedLiteralRead<'_>>`, present exactly when a priority was declared.
All time reads, including response bound, have `LiteralKind::Quantity`, exact
`CheckedLiteralValue::Rational` magnitude and the admitted common unit's
`type_id`; a present priority has `LiteralKind::Integer` and exact
`CheckedLiteralValue::Integer`. No float conversion, default priority or
second unit normalization occurs. Tasks iterate in admitted task-name order.
The task view, like the claim view, has private fields and no public
constructor. These accessors rely on admission: downstream consumers need not
repeat binding-shape or policy validation to obtain the fields. Existing
literal reads may decode exact numeric values; the body remains borrowed and
no entire task or body copy is needed.

The IR read ends at that authenticated claim. The QSL FR-418 checker owns
joining it to a selected timed model, resolving source release bindings and
recomputing source/scheduler successors. `TaskTransitionTemplate` and
`TaskProductCertificate` remain backend-provider certificate values; exact
`TaskTransitionIdentity` remains concrete replay/search evidence. This carrier
adds none of them to the checked-package wire or the cycle-free model API.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-038-AC-1 | Each positive V2 package fixture this repository builds from its own public vocabulary admits; an unknown, empty, absent or malformed `contract_version` refuses as `unknown_contract_version` or `malformed_wire` before any version-specific decoding; and the strict parse (duplicate-member, noncanonical) refusals occur before a version is selected. | Test (TC-048) |
| FR-038-AC-2 | PLANNED/UNRUN (IR-1141). The reader refuses malformed, duplicate-member, unknown-member, noncanonical, missing dependency, conflicting definition, unknown required capability, unsupported node tag, invalid graph and invalid source-map inputs with their exact applicable codes and loci before exposing a package; every authored adverse structural mutation returns its declared outcome. No content digest or stale-node-key refusal exists. | Test (TC-048) |
| FR-038-AC-3 | Exact byte, node, edge, occurrence, diagnostic and work limits admit a package; each one-over limit returns `incomplete` with that limit kind, the limit and the consumed counter and no package; no limit kind is a depth (FR-038-AC-117). | Test (TC-048) |
| FR-038-AC-4 | A V2 package has a required non-empty `name`, and two admitted packages with that name are structurally equal only when every JSON member and ordered array element equals. Changing a node body or node order changes the complete package value without changing its name; no stale-content refusal or content-derived package identity is computed. | Test |
| FR-038-AC-5 | Each nominal node carries a structural `{package, path, ordinal}` id. A source `enum`, `dimension` or `unit` has the closed typed body stated in "The flat wire"; an empty aggregate on one, an empty enum member list, a missing unit scale, an extra unit body member or a non-integer dimension exponent refuses `malformed_wire` at the schema step (QSpec FR-322-AC-269, NB-01..NB-05). An enum with `ordered: true` admits as `ordered_enum` (NB-06). After valid wire shape, the reader validates owner, declaration, semantic type, dependencies, enum case and unit rules independently; a semantic mismatch refuses under its applicable `invalid_semantic_graph` or more specific typed rule. A model-owned nominal retains an empty aggregate body; a model owner admits when its identity names a selected domain package and refuses when it names none or carries an empty node. | Test (TC-048) |
| FR-038-AC-374 | PLANNED/UNRUN (IR-1141). An external default-feature consumer constructs each of the four source nominal bodies through its typed fallible constructor, wraps each in `CheckedTerm::NominalBody`, and observes exact closed-body serialization. The same body read from a conforming package equals the constructed typed body. Empty or duplicate enum members, a nonidentifier member, a noncanonical exponent, an invalid node-id index segment, a noninteger rational numerator, a zero or negative dimension-equivalence `root_scale` numerator, and a nonpositive rational denominator each return `malformed_wire` at the offending body-relative pointer with no body. Public compile-fail probes reject raw variant and field construction, mutation, unchecked conversion and deserialization of `NominalBody`. A body with valid shape but a wrong owner or unresolved graph reference remains constructible and is refused by ordinary package admission under its existing semantic rule. | Test (TC-048, TC-058) |
| FR-038-AC-375 | PLANNED/UNRUN (IR-1141). A default-feature external consumer reads a source enum `NominalBody` from an admitted V2 node through its public typed body and observes the exact ordered flag and borrowed member names in authored order, including two names whose alphabetical order differs. The same view of an unordered enum reports `false` and its own authored order. Dimension, unit and dimension-equivalence bodies each return no enum details; an admitted model-owned empty aggregate is not presented as an enum body. The view has no mutable output, setter, unchecked body constructor or admission shortcut, and a malformed or semantically mismatched enum still receives its existing reader refusal without an admitted node. | Test (TC-048, TC-058) |
| FR-038-AC-376 | Implemented; evidence: `node_index::permutation_rebuilds_the_same_structural_keys_and_stale_operands_do_not`, `node_index::two_semantic_defects_choose_structural_order_over_graph_order`, and `node_index::mixed_operation_defects_choose_resolved_operand_order` against QSpec FR-322-AC-278..282.  The reader compares rebuilt structural keys by UTF-8 package, path strings before resolved operand keys, proper prefixes first and numeric ordinal, never by graph index; graph order stays producer chosen. A two-defect control with graph order opposite structural order reports the structurally first defect. A valid permutation with rewritten index segments retains every resolved key and reference; stale indexes that retarget an operand cannot silently pass the same structural-key oracle. This replaces retired AC-365 without narrowing its ordering and exact-equality controls. | Test (TC-048) |
| FR-038-AC-377 | Implemented; evidence: `node_index::published_index_segment_refusals_precede_semantic_checks` and `checked_package_v2_canonical_encoding::tc_048_envelope_refuses_indexed_dependency_reference_node`.  Against the selected QSpec node-index vectors NI-01..NI-07, the reader refuses nested node ids, malformed index objects and a dependency-reference node path containing an index as `malformed_wire` at the patched segment. It accepts only string or closed `{"node": i}` segments with I-JSON integer `i` in range; a `DeclaredNodeId` path is all strings. The selected positive dependency-call fixture admits without an inline nested node id. | Test (TC-048) |
| FR-038-AC-378 | Implemented; evidence: `node_index::published_index_segment_refusals_precede_semantic_checks` and `checked_package_v2_temporal::tc_048_out_of_range_operand_precedes_interval_semantics`.  NI-08..NI-10 each refuse `malformed_wire` with no cause at the out-of-range segment's `/node` pointer before any semantic-graph defect. With two out-of-range segments, the first in canonical byte order wins. NI-11 and NI-12 each refuse `malformed_wire` with no cause at the first cycle-closing own-path segment ordered by holding graph index then path position, including a self cycle; a forward reference without a cycle admits. The index checks precede every semantic-graph check, including duplicate ids, and retain their exact RFC 6901 pointers. | Test (TC-048) |
| FR-038-AC-379 | Implemented; evidence: `node_index::imported_index_resolves_in_the_importing_graph` and `checked_package_v2_refinement::refinement_external_abstract_resolves_only_the_directly_selected_package_root`.  Every index segment in a package document resolves against that document's graph, even when the id's `package` names an imported node. NI-14's imported `Int[0, 9]` rebuilds against the importing graph and admits; resolving it against its owner's library graph would create a self-cycle and fails the oracle. An external FR-375 `{package, node}` key resolves against the selected dependency named by the key's own `package`; an unselected dependency or wrong document graph cannot supply the node. A dependency-reference node path remains string-only. | Test (TC-048) |
| FR-038-AC-380 | Implemented; evidence: `node_index::long_index_chain_admits_on_a_small_stack_with_shared_keys` and iterative `CheckedNodeId` comparison, validation, clone and drop.  NI-13's 100000-node left-nested chain, with reader byte, node, edge, occurrence, diagnostic and work bounds sized not to decide the result, reads on a 256 KiB thread stack under serde_json's default 128 recursion limit. Its wire node ids stay at the fixed JSON depth and at most 200 bytes each; 100001 rebuilt structural keys are distinct and each derived key holds its predecessor's key as its final operand through shared `Arc` nodes. Key comparison, validation, clone and drop do not recurse in proportion to the chain. The existing AC-117 stack and AC-214 memory obligations remain. | Test, Inspection (TC-048) |
| FR-038-AC-381 | Implemented; evidence: `node_index::long_index_chain_admits_on_a_small_stack_with_shared_keys`, `checked_package_v2_flat_wire::tc_048_a_syntax_or_duplicate_defect_is_refused_before_the_parse_limit`, and intake source-span inspection.  The intake source-raw pass and its per-pointer reparse accept the canonical NI-13 package under the default serde_json recursion limit without allocating a full-document JSON value or disabling the limit. A malformed node-id segment deep in the graph is reported at its own pointer, and syntax, duplicate-member and noncanonical-byte precedence from AC-24 and AC-214 remains unchanged. An actually nested body past the closed grammar still receives its existing typed refusal. | Test, Inspection (TC-048) |
| FR-038-AC-382 | A public-reader mutation that relabels an integer-range node as `text_bounds` while retaining its Integer semantic base and body refuses `invalid_package`, no cause, at `/semantic_graph/nodes/<n>/body`, with that node as locus. A `text_bounds` node over Text retains its own minimum, maximum and text profile unchanged. | Test (TC-226) |
| FR-038-AC-383 | An external `temporal` clause with a fairness argument targeting a compensation template owned by protocol clause `Pay` admits when `Pay.compensations` lists that template exactly once and `Pay.requirements` is empty. The selected protocol and temporal profiles, all ordinary graph dependencies and every other body member are valid; the admitted fairness target retains the template's exact structural node id. Admission does not require a `Pay.requirements` back-edge to the external temporal clause. | Test (TC-048) |
| FR-038-AC-384 | For the external temporal clause of AC-383, a compensation target not listed by any protocol clause, named in another package or naming a node of the wrong protocol form retains AC-103's missing-name or malformed-declaration refusal at the fairness target's `node` field, with the fairness node as locus. A fairness node omitted from every admitted temporal clause's fairness argument also refuses missing-name there even when its compensation target has a unique valid owner. A temporal clause listed in one protocol clause's `requirements` still refuses a compensation target owned only by another clause; the ownership join does not change the requirement association of protocol-root, attempt or parallel-branch fairness targets. | Test (TC-048) |
| FR-038-AC-385 | The public V2 reader admits QSpec `timed-clause-node-vectors.json` cases `CL-57` and `CL-59`: a guard part and a time invariant each retain an ordered `binders` aggregate of typed slot-parameter references in the specified position. Existing unquantified `clock_guard` and `time_invariant` positive controls still admit with their original body and identity; no binder is dropped or interpreted as an unquantified approximation. | Test |
| FR-038-AC-386 | The reader refuses QSpec cases `CL-58` (non-slot guard binder), `CL-60` (repeated invariant binder) and `CL-62` (reversed nested slot binders) with exactly `invalid_package`/`timed-node-reference` at each vector's first bad binder `target` RFC 6901 path and the containing timed node as locus. The test reads QSpec's proposal fixture and vectors by reference and compares the exact code, cause and path; no QSpec artifact is copied. | Test |
| FR-038-AC-387 | The public reader admits QSpec `XP-01` and `XP-10`, with each local dimension operand as a local `NodeRef` and each foreign dimension operand as a closed `PackageNodeRef {package, node}` of a directly selected dependency. It resolves the foreign declared dimension in that dependency's admitted package, preserves the authored operand form in the typed nominal body and writer, compares the two resolved structural node ids for distinctness and ascending order, and retains only local operands in the equivalence node's `dependencies`. No foreign dimension node is minted into the importing graph. | Test (TC-280) |
| FR-038-AC-388 | For QSpec `XP-11`, `XP-12` and `XP-15`, a package operand naming only a closure dependency, no selected dependency or the importing package refuses `missing_declaration`/`missing-selection` at `/semantic_graph/nodes/{i}/body/dimension_node_ids/{k}/package`. `XP-13` names no node in its directly selected dependency and refuses `missing_declaration`/`missing-name` at the corresponding `/node`. The reader checks these operands in listed order before pair rules; neither package display names nor an imported graph node substitute for the selected dependency lookup. | Test (TC-280) |
| FR-038-AC-389 | QSpec `XP-14` supplies a self-minted foreign-package `NodeRef` and refuses `invalid_semantic_graph` at `/semantic_graph/nodes/{i}/body/dimension_node_ids/{k}`. A local operand naming no graph node, or a local or package operand resolving to a node other than `scalar_type`/`dimension`, refuses the same code at that operand before pair validation; `XP-05` names a unit node and refuses there at its wrong-kind operand. QSpec `XP-02` through `XP-04` and `XP-06` through `XP-09` keep their published `invalid_semantic_graph` pair outcomes after operand resolution; a valid mixed-package equivalence still admits. Reversed or duplicate resolved structural dimensions refuse `invalid_semantic_graph` at the later equivalence node in structural node-id order. | Test (TC-280) |
| FR-038-AC-390 | A present `CheckedTermDataV2` has exactly four required arrays, `equality_decisions`, `protocol_locations`, `dispatch_tables` and `probabilistic_claims`, including when all are empty. `None` omits the sidecar, while `Some(empty)` retains it through typed write and strict read. Removing any required array or adding an unknown member refuses `malformed_wire` at the missing object or unknown member; the public and private wire paths agree. | Test (TC-048, TC-451) |
| FR-038-AC-391 | The strict reader admits an IR-constructed complete V2 package containing QSpec `checked-term-data-vectors.json` positive `PC-01` and `PC-02` identities as two nonempty probabilistic-claim rows. It retains each exact local `claim`/`probabilistic_claim` declaration `NodeRef` and its own declared `claim` occurrence. Their equal display names and source spans do not merge them or determine identity; a probabilistic-claim declaration without its one required row is refused under QSpec FR-322-AC-297. | Test (TC-048, TC-233) |
| FR-038-AC-392 | The reader checks QSpec `PC-S1` missing occurrence and `PC-S2` extra member as closed-schema `malformed_wire` refusals. For `PC-R1` foreign, `PC-R2` wrong-kind, `PC-R3` absent-local and a non-probabilistic claim declaration, it returns `invalid_package`/`invalid-value` at `/checked_term_data/probabilistic_claims/{i}/declaration`; for `PC-R6` mis-mapped occurrence or wrong occurrence role, it returns that code/cause at `/checked_term_data/probabilistic_claims/{i}/occurrence`. Each refusal has the exact published primary RFC 6901 locus and no admitted package. | Test (TC-048, TC-233) |
| FR-038-AC-393 | QSpec `PC-R4`'s second row for one declaration with a different valid occurrence and `PC-R5`'s repeated declaration-occurrence key both refuse `invalid_package`/`conflicting-definition` at the later row. `PC-R7`'s reversal of two valid rows refuses `invalid_package`/`invalid-semantic-graph` at the second row. Structural declaration identity and occurrence determine order; display name, source span and graph-array position do not. | Test (TC-048, TC-233) |
| FR-038-AC-394 | IR tests read QSpec's published `checked-term-data-vectors.json` by reference and construct a complete valid V2 package in the test from its `PC-01` and `PC-02` row identities and resolved-node context. They run the positive and probabilistic-claim adverse mutations against the public strict reader and compare each published typed code, cause and exact RFC 6901 locus. No upstream artifact or vector content is copied into this repository, and the vector file is not treated as a complete checked-package fixture. An exact measured sidecar-work limit admits the IR-constructed positive; one below its measured charge returns `resource_exhausted` at its final probabilistic-claim row without a partial typed sidecar view. | Test (TC-048, TC-233) |
| FR-038-AC-395 | A package with a `claim`/`probabilistic_claim` graph node passes the closed `ClaimForm::ProbabilisticClaim` vocabulary and ordinary typed term-body checks, then graph identity, exact dependencies, declaration, owner, occurrence and source-map joins before its probabilistic sidecar row is checked. The node and complete body survive admitted read/write without a fabricated formula or identity. An unknown claim form, malformed term body or broken graph join retains its existing earlier-stage typed refusal rather than being accepted by a matching sidecar row. | Test (TC-048, TC-058) |
| FR-038-AC-396 | planned until IR-1252 lands. The public strict reader admits complete packages constructed around QSpec TC-976 SCC-01 and SCC-02, retains one row for occurrence 0 and two distinct rows for the shared node's occurrences 0 and 1 respectively, and exposes their exact borrowed declaration, claim occurrence and FR-482 body in canonical order. A package with no schedulability occurrence omits the index and returns an empty typed iterator. Equal display names or source regions do not select or merge claims. | Test |
| FR-038-AC-397 | planned until IR-1252 lands. SCC-03/04/05/12 refuse `malformed_wire` at their published row/member loci. SCC-06/07/08 refuse `invalid_package`/`invalid-value` at the published declaration/occurrence loci; additional absent-local, wrong-kind and wrong-role controls have the same corresponding refusal. SCC-09 refuses `invalid_package`/`conflicting-definition` at the later row; SCC-11 refuses `invalid_package`/`invalid-value` there. SCC-10 and an omitted required index refuse that code/cause at `/schedulability_claims`. Each refusal compares the exact typed primary locus and exposes no admitted package. | Test |
| FR-038-AC-398 | planned until IR-1252 lands. Wire and graph/aggregate validation precede carrier joins; declaration precedes occurrence, repeated key precedes order, coverage follows all rows, and carrier validation precedes FR-482 body validation. Combined defects distinguish each adjacent check. Each visited row charges one existing validation visit before inspection; an exact measured budget admits the positive package, and one below the measured charge refuses with the existing incomplete/resource-exhaustion result at the first uncharged row and no partial typed claim view. | Test |
| FR-038-AC-399 | planned until IR-1252 lands. The reader admits the full FR-482 schedulable/response bodies and validates its shape, literal vocabulary, form fit, strict task order, common time unit and policy-specific task-set premises in that order after the protocol body step. QSpec FR-482-AC-3's adverse bodies, including negative/excess BCET, wrong time unit, duplicate fixed-priority priorities and the EDF equal-priority positive control, have their exact published codes, causes and loci. Existing graph refusals for nonempty dependencies, owner, semantic type or identity precede the dependent body checks. The typed claim/task accessors return the closed kind, policy, arrival and criticality variants, task names, optional response fields, optional HI work and priority, and exact quantity magnitudes with the common unit identity; absent priority stays absent. Consumers require no raw JSON or binding-shape validation. The typed read retains every policy, task and exact time-value member without interpreting model releases or successors. | Test |
| FR-038-AC-400 | planned until IR-1252 lands. TC-048 reads QSpec's `proposals/checked-package-v2/schedulability-claim-vectors.json`, TC-976 and FR-482/TC-967 cases by reference from a selected QSpec checkout, constructs complete IR packages around their resolved-node context, applies every carrier mutation and compares admission or exact typed code, cause and primary locus. The evidence states the selected checkout revision; no upstream schema, fixture or vector is copied into IR and the row vectors are not treated as complete packages. Complete public read/encode/read retains every identity row and the body, including both shared-node occurrences and the absent index; the private envelope, raw intake and borrowed encoder agree on the optional member. The default-feature public read requires no timed-model, release-binding or scheduler-engine input and adds no task-transition template wire member. | Test |
| FR-038-AC-6 | Every admitted node family lowers independently with exact source, type, dependency, bound and claim correspondence; missing, unsupported, unbounded and over-work requests return `invalid_input`, `unsupported`, `requires_bound` and `failed` without a node and without changing sibling records, except that a package over the byte ceiling fails every requested record (FR-038-AC-95). | Test (TC-050) |
| FR-038-AC-9 | The shipped default read-limit policy is exactly those six finite values, every member is strictly positive and finite, and each meter is enforced at its own true measured boundary against a real package: the package's exact measured consumption for that meter admits it, and one below that exact value refuses it as `incomplete`, naming that meter and reporting the true consumption. The shipped default is far larger than any fixture, so this boundary is proven against each meter's real measured cost rather than against the default value itself; no fixture approaches that scale, and none is fabricated to do so. | Test (TC-048) |
| FR-038-AC-7 | Lowering every node of every positive fixture under a profile supporting every tag yields no `invalid_body` and no `body_incomplete` record, and the seven-member record vocabulary is exhaustive: no eighth kind is reachable and each of the seven is named. | Test (TC-052) |
| FR-038-AC-8 | A closure holding both an out-of-profile tag and an unbounded type returns `unsupported`; a zero work limit returns `failed` for an absent key rather than `invalid_input`; each named offending key is the least in ascending order rather than the first visited; each of the eight unbounded forms raises `requires_bound` and each other declared form of those two families, carrying no `recursion_group`, does not; and a lowered record's `dependencies` contains every key in its `bounds` and `claims`. | Test (TC-052) |
| FR-038-AC-10 | Withdrawn: the lock holds no domain-package row and no digest, so there is no `lock.model_selections` array whose entries can repeat. | Test (TC-048) |
| FR-038-AC-11 | Withdrawn: the lock holds no domain-package row and no digest, so there is no `lock.model_selections` array to order by defect class. | Test (TC-048) |
| FR-038-AC-12 | A `creates` or `deletes` entry's eligibility is exactly `model`/`object_type` and `model`/`process`, and this eligibility predicate is checked against every `(member, tag, form)` triple the closed node taxonomy can produce, not sampled; the `modifies` entry shape and its eligibility are [FR-040](./FR-040-admit-frame-entries-and-state-clauses.md)'s. The refusal an ineligible triple drives — `invalid_model_binding` with cause `malformed-declaration`, located at the offending entry — is verified end-to-end through the reader for a representative ineligible triple in each of `creates` and `deletes` and every node family; the reader's mapping from an ineligible predicate result to that refusal depends only on which member the entry sits in, never on its tag or form, so this sample together with the exhaustive predicate check covers the full triple space without driving every one of it through the reader. Each of the two eligible triples the published all-families fixture's own frame body does not already exercise (`process` in `creates`, `object_type` in `deletes`) is independently shown admitted. | Test (TC-053) |
| FR-038-AC-13 | A frame entry naming a structural node id absent from the frame node’s own dependencies refuses missing_declaration/missing-name at that entry, whether the id names another graph node or no node at all; it never becomes an invalid_model_binding refusal. | Test (TC-053) |
| FR-038-AC-14 | A frame node carrying both a meaning-join defect and a canonical-order defect refuses for the meaning-join defect; among two meaning-join defects in different members, the earlier member (`modifies` before `creates` before `deletes`) is selected regardless of which defect's entry structural id is lower, except that a `conflicting-binding` defect ranks after every other meaning-join defect (FR-040-AC-25); among two meaning-join defects in the same member, the lower entry structural id is selected; a member array not in strictly ascending structural id order, with no meaning-join defect present, refuses as `invalid_semantic_graph` at the frame body path, located at the frame node itself and carrying no cause. | Test (TC-053) |
| FR-038-AC-15 | A package carrying two defective `state`/`frame` nodes refuses at the one with the earlier structural `node_id`, reporting only that frame's own defect, even when the other frame's defect would otherwise outrank it under FR-038-AC-14's precedence — the visit order is ascending structural node-id across frames, and the reader reports the first defective frame it reaches rather than comparing every frame's defect. | Test (TC-053) |
| FR-038-AC-17 | Each required declaration, literal type, application operation and result_type is read from the complete graph. Deleting one refuses invalid_semantic_graph at the existing node declaration or body pointer before admission; the package name and structural node ids do not substitute for a missing member. All fifteen expression forms remain admitted through the same closed taxonomy. | Test |
| FR-038-AC-18 | For a keyed input that reaches the graph-reference stage, a self-typed node's own body-root `literal.type` naming itself is exempt from the reference-cycle check and needs no declared `recursion_group`. The same self-reference nested in an `aggregate` member, `binding` value or `application` argument is not exempt and refuses as `invalid_semantic_graph` at `/semantic_graph/nodes/<n>` (the node object lacking `recursion_group`), as do a self-typed node's `reference` body and `application.result_type` naming itself. Public admission checks the structural id for shape and uniqueness before that stage. Public tests exercise the conditional cycle carve-out directly through the graph stage, with valid structural ids and complete bodies. | Test (TC-048) |
| FR-038-AC-19 | Withdrawn: the lock holds no domain-package row and no digest, so there is no `lock.model_selections` array whose entries can carry classed defects. | Test (TC-048) |
| FR-038-AC-20 | Two `model_root` nodes naming one `identity` refuse `invalid_semantic_graph` at the later root's `owner` (`/semantic_graph/nodes/<i>/owner`), whichever order the graph holds them in; two roots naming different identities admit, each other check passing. A root carries no `version` and no `digest`, so the identity is the whole locator (FR-038-AC-62). | Test (TC-048) |
| FR-038-AC-21 | A node whose declared `qualified_name` differs from its declared structural path refuses as `invalid_package` with cause `declaration-nominal-mismatch` at that node; two nodes in the same namespace declaring one name refuse as `ambiguous_declaration` with cause `ambiguous-name` at the later node in node-id order, while the same name across namespaces admits; a package carrying both defects refuses for the nominal mismatch; and a package also carrying a frame or an operation defect still refuses for its declaration defect. | Test (TC-048, TC-230) |
| FR-038-AC-22 | PLANNED/UNRUN (IR-1141 derived-shape correction). A QSL-shaped package holding a function whose body applies an operation to its `value`/`parameter` nodes and a valid `scalar_type`/`compound_unit` node admits and lowers. A parameter with a negative, non-canonical or missing level, an empty, non-identifier or wrongly typed name, or an extra binding refuses `invalid_semantic_graph` at its `body`; a parameter with a dependency, itself as its type or a `declaration` follows the graph or declaration refusal at the broken member. A compound unit with a zero or non-integer exponent, a non-unit or non-root unit, repeated or out-of-order unit terms (`RM-CU-ORDER`), or the wrong semantic dimension refuses `invalid_package`/`invalid-derived-shape` at that node's `node_id` under QSpec FR-322 "Derived type forms and forwarding"; an independently wrong `dependencies` list refuses `invalid_semantic_graph` at `/dependencies` before derived-shape checking. The empty dimensionless compound unit admits. | Test (TC-048) |
| FR-038-AC-23 | An application node whose `dependencies` omits a body reference target, lists them out of structural id order, repeats one, or adds its `result_type` refuses as `invalid_semantic_graph` at `semantic_graph.nodes.dependencies`, located at that node. | Test (TC-048) |
| FR-038-AC-24 | Every refusal about a value carries the RFC 6901 pointer of that value, built from the reader's own position: member names escaped (`~` as `~0`, `/` as `~1`), array elements by index, resolving in the document read — an unknown member (including one whose name holds `~` or `/`) at that member, a repeated member at that member, a missing member at the object lacking it, a wrongly typed value at that value, a malformed structural id at its exact member, and each graph, lock, source-map, capability and diagnostic refusal at the member its failed check read; malformed JSON and non-canonical bytes carry no pointer. The `unknown field` occurrence-role value of FR-038-AC-2 points exactly at `/semantic_graph/nodes/0/occurrences/0/role`; the genuine extra-member and wrong-kind controls each point at the member or value they change. No refusal code changes. | Test (TC-048) |
| FR-038-AC-25 | An `unknown_contract_version` refusal points at `/contract_version` and carries the exact `contract_version` string the reader read, including an empty one; no other refusal carries a version. | Test (TC-048) |
| FR-038-AC-26 | Each one-over limit other than the byte limit returns `incomplete` carrying the RFC 6901 pointer of the value whose charge failed, which resolves in the document read: nodes at the first node past it, edges at the dependency and occurrences at the source-map entry or region that took the count past it, diagnostics at the first entry past it, and work at the value whose validation took the meter past it; the byte limit carries none. | Test (TC-048) |
| FR-038-AC-27 | A domain package selection is admitted only by the document the evidence supplies under its identity: no document under the root's identity refuses `missing_import`/`missing-selection` at the root's `owner/identity` (`/semantic_graph/nodes/<i>/owner/identity`); a document holding a number whose text denotes a magnitude past 2^53 refuses `noncanonical_wire` at that pointer (FR-038-AC-93); bytes that do not read refuse `malformed_wire`/`malformed-json` there with no `document_pointer`; a document naming another identity refuses `invalid_model_binding`/`wrong-model-selection` there; and a matching document admits. The document's `package.version` is neither required nor read: a matching document with no `package.version`, or a non-string one, admits, and two documents of one package at different versions admit when each is selected in its own package under the same identity (FR-038-AC-64). | Test (TC-048) |
| FR-038-AC-28 | A domain package document's declarations refuse at the row, in FR-154's order: a node whose object id is invalid, whose `kind` names no construct, or that shares its identity, refuses for itself and any reference to it reports that refusal, never `missing_declaration`/`missing-name`, wherever the two sort; a node failing two rows reports the earlier (a dangling `typeRef` before a multiplicity with `lower > upper`, a malformed member before both); a `typeRef` naming a relationship refuses `invalid_model_binding`/`malformed-declaration` wherever its declaring node sorts; two nodes with no identity refuse `malformed-declaration`, never `conflicting-binding`. | Test (TC-048) |
| FR-038-AC-29 | A package whose lock selects a domain package document with declared types, and whose graph holds a `dispatch_call` on one of its operations, admits; the same call naming an operation the document does not declare refuses `ill_typed`/`operator-ineligible` at the member's `name`; an inherited field resolves on a subtype and a subtype conforms to its supertype in either operand order; a field typed at an `Int[lo, hi]` value type has the integer-range member type `[lo, hi]`. | Test (TC-048) |
| FR-038-AC-30 | Reading a selected domain package and resolving a model-owned member are charged to the `work` limit: a limit one below the work a read used returns `incomplete` for `work` with the pointer `/semantic_graph/nodes/<i>/owner` of the package's root, and the exact work admits. | Test (TC-048) |
| FR-038-AC-31 | A lock with two distinct `{identity}` dependency selections in ascending identity order admits when each corresponding admitted dependency package is supplied under a matching `name`; the reader retains those entries exactly. The importers and dependencies carry no `package_id` or identity preimage. | Test (TC-048) |
| FR-038-AC-32 | An empty dependency identity or a missing required identity refuses `malformed_wire` at the member or entry. An entry carrying `package_id`, `version`, `digest` or a `Selection`/`DefinitionRef` member outside its closed `{identity}` shape refuses `unknown_member` at the first extra member; no legacy shape is accepted. | Test (TC-048) |
| FR-038-AC-33 | A `dependency_selections` entry repeating an earlier entry's `identity`, adjacent or not, refuses `invalid_package`/`conflicting-definition` at the repeating entry; an entry not strictly after its predecessor in UTF-8 byte order refuses `invalid_package`/`invalid-value` at that entry; entries in UTF-8 byte order where UTF-16 code-unit order differs admit. | Test (TC-048) |
| FR-038-AC-35 | A function call with a `dependency_reference` callee to a declared function in the supplied dependency admits and carries the term verbatim without listing that remote target as a local dependency. Listing it locally refuses `invalid_semantic_graph`; changing the referenced package name or structural node id changes the referenced target and is checked by the normal dependency and declaration joins, without a rekey step. | Test (TC-048) |
| FR-038-AC-36 | A `dependency_reference` requires a non-empty package identity and a closed structural node id `{package,path,ordinal}`; malformed members refuse at their own pointer. A well-formed term outside argument 0 of `quire.op.function.call` refuses `ill_typed`/`operator-ineligible` at the term. | Test (TC-048) |
| FR-038-AC-37 | An unsupplied selected dependency refuses `missing_import`/`missing-selection` at the lock entry. A term whose package no entry names refuses `missing_declaration`/`missing-selection` at `package`; a node absent from that package or without a declaration refuses `missing_declaration`/`missing-name` at `node`; a declared non-function target refuses `ill_typed`/`operator-ineligible`, with the calling node as locus. | Test (TC-048) |
| FR-038-AC-293 | Given an otherwise valid package with direct selections for two admitted dependencies whose own direct selections reach the same transitive name with unequal complete JSON package values, the V2 reader refuses `invalid_package`/`conflicting-definition` before graph-reference validation. Its two ordered `loci` name the exact importer-rooted selection paths in declaration order, each following `/lock/dependency_selections/<i>` through the admitted package selected by the preceding entry; neither path is encoded in prose. The same refusal holds when the importer has one direct selection whose identity recurs in that admitted child's transitive closure with an unequal complete JSON package value. | Test (TC-048) |
| FR-038-AC-38 | A dependency function whose parameter is a declared record, whose result is a declared record, whose parameter is a `Set` of one, a tuple holding one, or a `Reference` to a `model` node refuses `ill_typed`/`operator-ineligible` at the callee; the same function over a `Set` of a bounded integer admits. | Test (TC-048) |
| FR-038-AC-39 | Under a bounds-required profile, `x + 1` over a parameter `x` typed at an `integer_range` domain over `integer` lowers although the parameter's name literal is annotated with the unbounded `text` type, and the annotation stays in the closure and in `dependencies`; a parameter typed at an unbounded `integer` or `rational` type refuses `requires_bound` naming that type. | Test (TC-050) |
| FR-038-AC-40 | Under a bounds-required profile a `bounded_domain` over a shared unbounded type does not bound another position naming it: `{n: Integer, k: Int[0,9]}`, `Sequence<Integer>[0,3]` beside `Int[0,9]`, and `(x + 1) + n` over `x: Int[0,9]` and an `n` typed at `integer` each return `requires_bound` naming `integer`, while `Sequence<Boolean>[0,3]`, a record of bounded collections and ranged fields, and `x + 1` over `x: Int[0,9]` lower. | Test (TC-050) |
| FR-038-AC-41 | Under a bounds-required profile a `scalar_type` or `composite_type` node declaring a `recursion_group` returns `requires_bound` naming the least offending node key, although every field of the recursive record is bounded. | Test (TC-050) |
| FR-038-AC-42 | An enum's operand family is `ordered_enum` when its admitted enum declaration is `ordered` and `enum` otherwise (QSpec FR-322): `quire.op.enum.lt`, `le`, `gt` and `ge` over two operands of an ordered enum, member literals or parameters, admit; the same operations over an unordered enum refuse `ill_typed`/`operator-ineligible` at the first argument (QSpec FR-141-AC-5), and over operands of two different enums at the second; `quire.op.enum.eq` and `ne` admit over either. | Test (TC-048) |
| FR-038-AC-43 | `structural.eq` over an all-integer record, and `collection.contains` over a set of integers, admit with `leaves` empty, because the compared type has no text leaf; the same operations over a record with a nested `text` field, or a set of `text`, refuse `invalid_package`/`operation-law-missing` at `operation.leaves` when `leaves` is empty, and one supplied leaf over two text fields refuses the same way; `collection.flatten` to a `sequence` of text admits with `leaves` empty, while a `set` of text result refuses; a compared type that names a missing node, or that reaches an option or collection again with no record or tuple between the two visits, refuses `ill_typed`/`operator-ineligible` at `operation.leaves`, while a compared type that reaches itself through a record or tuple admits (FR-038-AC-70 through FR-038-AC-72); and a chain of 12 record levels of 4 fields naming the next level is decided inside the work budget. | Test (TC-048) |
| FR-038-AC-44 | `operation.leaves` is exactly the derived leaves, the text leaves and, for a type that reaches itself, its recursion leaves (FR-038-AC-70, FR-038-AC-71): a record with `a`, `b` and `c` (an option of text) fields over a `text_bounds` text type that binds the `nfc` profile admits the leaves `["field:a"]`, `["field:b"]` and `["field:c", "inner"]`, a tuple the `position:<n>` segments, and `collection.contains` over a set of text the one empty path, each carrying exactly one catalogued `text_profile` law the lock selects and the mode `{kind: text_profile, value: nfc}`; a text leaf whose type binds no profile refuses `ill_typed`/`operator-ineligible` at `operation.leaves`, with the leaves supplied or not; a leaf with no mode refuses `operation-mode-mismatch` at its `mode`, one of another kind at `mode/kind`, one of an uncatalogued value at `mode/value`, and a catalogued value other than the pinned one `operation-mode-type-mismatch` at `mode/value`; `collection.flatten` to a `sequence` of text refuses one supplied leaf `operation-law-mismatch` at `operation.leaves/0` and admits none; an entry with a law the lock does not select and a supplied leaf but no leaf source refuses `operation-law-mismatch`, not `operation-law-unselected`; two leaves over an all-integer record, one leaf more than the text leaves, and an entry that names no leaf source with a leaf refuse `invalid_package`/`operation-law-mismatch` at the first extra leaf; three unrelated paths, a wrong or missing `inner` segment, two leaves out of order and a wrong segment kind refuse the same way at that leaf's `path`; a leaf with no law, two laws, a law of another role and a law outside the catalogued `text_profile` definitions refuse the same way at that leaf's `laws`; a leaf law the lock does not select refuses `operation-law-unselected` at its `definition`; and 16 levels of 10 fields all naming the next level over text refuse `operation-law-missing` without listing its 10^16 leaves, while the one leaf of a 16-level path with nine integer fields per level admits at its exact path and refuses at a wrong segment. | Test (TC-048) |
| FR-038-AC-45 | PLANNED/UNRUN (IR-1141). A model-owned node with owner {kind: model, identity, node} and no version resolves its identity against the selected model_root and its node against the selected declaration; a selected field or operation member resolves through that declaration. A foreign owner identity refuses missing_declaration/missing-selection, and malformed owner shape or empty members refuse at their corresponding closed-wire or graph checks. No model-owned node id is recomputed from owner content. | Test (TC-048) |
| FR-038-AC-46 | PLANNED/UNRUN (IR-1141). A package whose definition references are exactly {authority, identity} at every lock, diagnostics.catalog and operation-law reference site admits when each selection and body check passes. The package has no identity_preimage mirror. | Test (TC-048) |
| FR-038-AC-47 | A package whose source references are exactly `{authority, identity}`, at each `lock.sources` row, each `source_map` region `source` and each `diagnostics.entries[].loci[].source`, admits, each other check passing. | Test (TC-048) |
| FR-038-AC-48 | PLANNED/UNRUN (IR-1141). A definition reference at a lock, diagnostics.catalog or operation-law site carrying revision, digest_domain, digest or export refuses unknown_member at the first extra member in document order and returns no package; the earlier five-member shape also refuses at its first extra member. | Test (TC-048) |
| FR-038-AC-49 | An `operation.laws[].definition` that carries a member beyond `authority` and `identity`, lacks one or holds one of the wrong kind refuses as `invalid_semantic_graph` at that member, or at the `definition` when a member is absent, as every closed shape inside an `operation` does (FR-038-AC-36), never as `unknown_member` or `malformed_wire`. | Test (TC-048) |
| FR-038-AC-50 | A `lock.sources` row, `source_map` region `source` or `diagnostics.entries[].loci[].source` that carries `revision`, `digest_domain`, `digest` or `export` refuses as `unknown_member` at that member and returns no package. | Test (TC-048) |
| FR-038-AC-51 | PLANNED/UNRUN (IR-1141). A definition reference at a lock, diagnostics.catalog or operation-law site whose authority or identity is absent or not a string refuses malformed_wire at the value or reference, and an empty value refuses malformed_wire at that member. | Test (TC-048) |
| FR-038-AC-52 | A source reference at a `lock.sources` row, `source_map` region or `diagnostics.entries[].loci[]` whose `authority` or `identity` is absent or not a string refuses as `malformed_wire` at the value, or at the reference when the member is absent, before any equality check. | Test (TC-048) |
| FR-038-AC-53 | A `lock.sources` row carrying `digest_domain` or `digest` refuses as `unknown_member` at the first extra member in document order, even when an `authority` or `identity` value is empty; no source digest is checked or retained. | Test (TC-048) |
| FR-038-AC-54 | A `lock.sources` row with an empty `authority` or `identity` refuses as `malformed_wire` at that member. | Test (TC-048) |
| FR-038-AC-55 | A `source_map` region `source` or `diagnostics.entries[].loci[].source` that equals no `lock.sources` row in both `authority` and `identity` refuses as `invalid_source_map` at that `source`, whether its pair names another source or contains an empty member; the nonempty row check of FR-038-AC-54 applies to `lock.sources` only. | Test (TC-048) |
| FR-038-AC-56 | An `operation.laws[].definition` of a value role admits only when its `{authority, identity}` pair is catalogued for the role and is a `lock.definition_selections` row, compared by those two members alone: a pair the role does not catalogue, including an empty one, refuses `invalid_package` with cause `operation-law-mismatch`, and a catalogued pair the lock does not select refuses `operation-law-unselected`, each at that law's `definition`. | Test (TC-048) |
| FR-038-AC-57 | An `operation.laws[].definition` of a profile role (`temporal_profile` or `protocol_profile`) admits only when its `{authority, identity}` pair equals the pair of the lock's `profile_selections` row of that role; any other pair refuses `operation-law-unselected` at that law's `definition`. | Test (TC-048) |
| FR-038-AC-58 | The operation catalog read over supplied bytes whose `law_roles` entries are exactly `{authority, identity}` returns the catalog, and over bytes in which an entry carries `revision`, `digest_domain` or `digest` returns an error naming the unreadable entry instead of a catalog and without a panic; this is a catalog read failure and not a package refusal. | Test (TC-048) |
| FR-038-AC-59 | Withdrawn: no identity preimage mirrors definition selections and no stale-selection refusal exists. FR-038-AC-366 checks the single authoritative lock row. | Test (TC-048) |
| FR-038-AC-60 | Editing one lock definition identity produces a different package value under the same name. The reader joins that identity to the supplied definition; it admits a valid selection and refuses a missing or ineligible selection under the existing definition-join code, without comparing an old package identity. | Test (TC-048) |
| FR-038-AC-61 | A schema-valid `SourceOwner` whose `{authority, identity}` pair is absent from `lock.sources` refuses `missing_declaration`/`missing-selection` at its node's `node_id` in the owner join (QSpec FR-322-AC-52; FR-038-AC-155). A node carrying `DefinitionOwner` refuses `malformed_wire` at its `owner` during closed-schema validation, before any lock join, because `NodeOwner` admits only `SourceOwner`, `ModelOwner` and `ModelRootOwner` (QSpec FR-322-AC-51). Both adverse cases remain required; there is no definition-owner selection or compatibility variant. | Test (TC-048) |
| FR-038-AC-62 | A package whose lock holds exactly `sources`, `edition`, `profile_selections`, `definition_selections`, `required_features` and `dependency_selections` admits when all checks pass. A lock with `model_selections` or `identity_preimage`, or a model-root owner with `digest`, `version` or any extra member, refuses `unknown_member` at the first extra member. A model-root owner lacking a non-empty string identity refuses `malformed_wire` at the owner or value. | Test (TC-048) |
| FR-038-AC-63 | Each dependency selection is exactly `{identity}`. Adding `package_id`, `version` or any other member refuses `unknown_member` there; omitting identity refuses `malformed_wire` at the entry. The reader does not ignore, relabel or compare a removed member. | Test (TC-048) |
| FR-038-AC-64 | A model-root owner admits when the supplied document names its identity; a model owner joins the root of that identity and an owner naming another identity refuses `invalid_semantic_graph`. A dependency selection admits only a supplied package whose `name` equals its selected identity; a package supplied under another identity leaves the selection unsupplied and refuses `missing_import`/`missing-selection`. No version or content-derived id is compared. | Test (TC-048) |
| FR-038-AC-65 | Every application operator class, operation member kind and constraint kind the operation catalog declares decodes to a member of its closed vocabulary, so the production catalog, whose `law_roles` entries carry no `revision`, reads once FR-038-AC-46 through FR-038-AC-61 are also implemented; each of `case`, `temporal_formula`, `temporal_fairness`, `temporal_interval`, `fairness` and `union_arms` converts wire string to enum member and back to the same string; catalog bytes that name an operator class, member kind or constraint kind outside its vocabulary return the typed error of the catalog read of FR-038-AC-58, naming the word, instead of a catalog; and a package application whose `operator` is outside the closed operator vocabulary refuses `invalid_semantic_graph` at that term. | Test (TC-048) |
| FR-038-AC-67 | An unknown identity refuses `unknown-operation` and an `operator` that differs from the catalogued class refuses `operation-class-mismatch`, each ahead of every other check of the entry at the operation step, so `quire.op.control.case` under the operator `unary` refuses `operation-class-mismatch`; and of two defective body-root nodes the one with the earlier structural `node_id` is reported when both defects are operation refusals. A placement defect of the temporal step (FR-038-AC-102) is reported ahead of every operation refusal, whatever the structural id order. (Amended by IR-549: the earlier text ordered both refusals ahead of blanket temporal/case `unsupported_construct`, which IR-549 removed, and put the placement of these applications inside the operation step.) | Test (TC-048) |
| FR-038-AC-68 | At the operation check of a `quire.op.temporal.clause` application, observed on that node as a unit-level check of the node's operation step and, since IR-549, also as an admitted package whose formula node is a `temporal_formula` application (FR-038-AC-96), an application with the one selected `temporal_profile` law, no member and six arguments (a `reference` to a `value`/`parameter` node of any type, a `text` literal, three `aggregate` terms and a `reference` to a `temporal`/`formula` node) passes; five or seven arguments refuse `ill_typed`/`operator-ineligible` at `arguments`; a first argument that is not a `reference` term and a sixth that references a Boolean node each refuse the same way at that argument; a `reference` to a `temporal`/`formula` node fits the sixth operand and an `any_term` position and no `boolean` operand position; and a member of kind `profile_operator` or any other kind refuses `invalid_package`/`operation-member-mismatch` at `operation.member`. | Test (TC-048) |
| FR-038-AC-69 | A member of kind `temporal_interval` or `fairness` on an application whose catalogued entry has none of the operator classes `case`, `temporal_formula` and `temporal_fairness` and a member of another kind or none, such as `quire.op.boolean.not` or `quire.op.temporal.clause`, refuses `invalid_package`/`operation-member-mismatch` at the member, and, since IR-549, so does a `fairness` member on a `temporal_formula` identity such as `quire.op.temporal.holds`, which the earlier text refused `unsupported_construct` at `operator`; the operator refusal is gone and every member that disagrees with its entry refuses alike. | Test (TC-048) |
| FR-038-AC-70 | `structural.eq` over `record Node { label: Text[0, 8; nfc]; next?: Node; }` admits with the leaf `["field:label"]` carrying one catalogued `text_profile` law the lock selects and the mode `{kind: text_profile, value: nfc}` followed by the recursion leaf `["field:next", "inner", "recursion:0"]` with no laws and no mode, and the same comparison with the text leaf alone refuses `invalid_package`/`operation-law-missing` at `operation.leaves`, as does the recursion leaf alone; over `Option<Node>` it admits the leaves `["inner", "field:label"]` and `["inner", "field:next", "inner", "recursion:1"]`; over a mutually recursive pair `A { name: Text[0, 8; binary-utf8]; b?: B }` and `B { tag: Text[0, 4; nfc]; a?: A }` in one package, compared at `A` and compared at `B`, each leaf with the mode its own type pins, it admits `["field:name"]`, `["field:b", "inner", "field:tag"]` and the recursion leaf `["field:b", "inner", "field:a", "inner", "recursion:0"]` at `A`, and `["field:tag"]`, `["field:a", "inner", "field:name"]` and the recursion leaf `["field:a", "inner", "field:b", "inner", "recursion:0"]` at `B`; over `Two { x: Node; y: Node }` it admits exactly the text leaves `["field:x", "field:label"]` and `["field:y", "field:label"]` each followed by its own recursion leaf, `["field:x", "field:next", "inner", "recursion:1"]` and `["field:y", "field:next", "inner", "recursion:1"]`, and refuses the list lacking either recursion leaf `operation-law-missing`; over `Tree2 { label: Text[0, 8; binary-utf8]; kids: Sequence<Tree2>[0, 3]; }` it admits `["field:label"]` and the recursion leaf `["field:kids", "inner", "recursion:0"]`; over a declared tuple `Pair` of a `Text[0, 8; nfc]` and an `Option<Pair>` it admits `["position:0"]` and the recursion leaf `["position:1", "inner", "recursion:0"]`; over `X { t: Text[0, 8; nfc]; n?: Y }`, `Y { u: Text[0, 8; nfc]; x?: X }` and `Wrap { y: Y; x: X }` compared at `Wrap` it admits exactly the four text leaves `["field:y", "field:u"]`, `["field:y", "field:x", "inner", "field:t"]`, `["field:x", "field:t"]` and `["field:x", "field:n", "inner", "field:u"]` with the two recursion leaves its reentries derive in their places, `["field:y", "field:x", "inner", "field:n", "inner", "recursion:1"]` after the second and `["field:x", "field:n", "inner", "field:x", "inner", "recursion:1"]` after the fourth, so a count memoised for `Y` under `field:y` is not reused under `field:x`, where `X` is open, and a list of the four text leaves alone refuses `operation-law-missing` at `operation.leaves` while a list of all six and one further text leaf refuses `operation-law-mismatch` at the seventh entry; over a record `R { a?: Text[0, 8; nfc]; b?: Text[0, 8; nfc] }`, whose two optional fields name one option node, it admits `["field:a", "inner"]` and `["field:b", "inner"]`; over a recursive record that reaches no `text` type, such as a `List` of integers, `structural.eq` and `collection.contains` admit with `leaves` empty; and the `Node` comparison with a second text leaf supplied after `["field:label"]` and the recursion leaf refuses `operation-law-mismatch` at that extra entry, `operation.leaves/2`, so a cyclic type is no longer refused `ill_typed`/`operator-ineligible` at `operation.leaves` and no longer admits unchecked leaves. | Test (TC-048) |
| FR-038-AC-71 | Over `Node`, the recursion leaf placed before the text leaf, a recursion leaf at `["field:next", "recursion:0"]`, one `["field:next", "inner", "recursion:1"]` with the wrong `d`, and a second recursion leaf after the first each refuse `invalid_package`/`operation-law-mismatch` at that leaf's `path`, one that carries a law refuses the same way at its `laws`, and one that carries a mode refuses `operation-mode-mismatch` at its `mode`; a recursion leaf at a reentry of an integer `List`, which reaches no `text` type, refuses `operation-law-mismatch` at that leaf's `path`; a text leaf inside a recursive record whose type binds no `text_profile` still refuses `ill_typed`/`operator-ineligible` at `operation.leaves`, with the leaves supplied or not; `T` = `Option<T>`, a `Sequence` of itself, and a record that holds a field of such a type each refuse `ill_typed`/`operator-ineligible` at `operation.leaves` under a work limit of 1000, so the refusal is not `incomplete` for `work`, whereas `R { x: Option<R> }` admits; and a leaf law the lock does not select inside a recursive record refuses `operation-law-unselected` at its `definition`. | Test (TC-048) |
| FR-038-AC-72 | The leaf count and derivation over a cyclic compared type are iterative and decided by the work budget: a cycle of 20000 record nodes, each holding an integer field and naming the next, the last naming the first, with the last also holding one `text` field, admits with its one 20000-segment leaf and its recursion leaf under byte, node, edge and work limits raised to admit it (the default byte limit of 1 MiB is below the package's size), on a thread whose stack is 256 KiB; ten records `R0` to `R9` that each hold a text field and an optional field naming every other record, compared at `R0` with `leaves` empty, return `incomplete` for `work` at `operation.leaves` under the default read limits, rather than a listing of their leaves or any other refusal; and for a ring of 12 records, each holding one text field and an optional field naming the next, compared at the first with its 12 text leaves and its one recursion leaf (the last record's field reenters the first), a work limit one below the work a read used returns `incomplete` for `work` and the exact work admits. | Test (TC-048) |
| FR-038-AC-73 | Under a bounds-required profile a position typed at a quantity returns `requires_bound` naming the `unit` or `compound_unit` node: a record whose field is typed at a `unit` node, a parameter typed at a `compound_unit` node, `Sequence<Quantity>[0,3]` whose element type is a `unit` node, and a record whose field is typed at a reachable `bounded_domain` over a `unit` node, which names the `unit` node rather than the domain, each return it; while a `unit` declaration lowered as the requested node, a `compound_unit` node lowered as the requested node with its unit nodes in `dependencies`, a record with no quantity position whose closure reaches a unit only as a `literal.type` annotation, and an application whose `result_type` is a quantity over parameters typed at a bounded type each lower. | Test (TC-050) |
| FR-038-AC-74 | For every in-repo positive fixture, canonical encoding of the complete closed V2 envelope agrees with independent canonical JSON. The full graph, including non-ASCII and astral strings and supported integer bounds, is retained once in `semantic_graph`; an added identity preimage or projection is refused as an unknown member. | Test |
| FR-038-AC-75 | PLANNED/UNRUN (IR-1141). For every positive graph fixture, quire_canonical::to_vec(CheckedSemanticGraphV2) equals independently constructed canonical JSON bytes for the same graph, including declared and derived structural ids, declaration, recursion_group, every JSON body kind, non-ASCII and astral strings, and the supported integer bounds. | Test (TC-048) |
| FR-038-AC-76 | For every in-repo positive fixture, the bytes `quire_canonical::to_vec` returns for each of `CheckedPackageLockV2`, `CheckedSourceMapEntry`, `CheckedCapability`, `CheckedNodeId` and `CheckedDiagnosticsV2` equal the independently constructed canonical JSON bytes for the same value, one assertion per type; the `CheckedNodeId` oracle includes its exact structural `{package,path,ordinal}` members and an operand index path segment, and the diagnostics value carries entries whose `details` hold nested objects and arrays and whose `loci` are non-empty. This replaces only the removed `CheckedSemanticId` oracle under FR-038-AC-80; no identity preimage or digest is encoded. | Test (TC-048) |
| FR-038-AC-77 | A `CheckedSemanticGraphV2` node body and `CheckedDiagnosticsV2` detail nested 100000 levels deep each encode on a 256 KiB stack to independently written expected text without stack overflow. The canonical envelope encodes each supplied deep body once without retaining a second wire body. | Test |
| FR-038-AC-78 | The encodes of FR-038-AC-77, run under a byte ceiling of the encoded text's exact length, return the bytes; run under a ceiling one byte lower, return the byte-limit error with no bytes; a body nested 20000 levels deep encodes and is not refused for its depth; and a body holding the integer 9007199254740993 returns the encoder's refusal naming that value with no bytes. | Test (TC-048) |
| FR-038-AC-79 | A package document holding 9007199254740993, -9007199254740993 or 2.0 in a node body, or an object with UTF-8-out-of-order member names, refuses noncanonical_wire before graph and body checks. | Test (TC-048) |
| FR-038-AC-80 | PLANNED/UNRUN (IR-1141). `quire-canonical` remains a `branch = "main"` git dependency with its source in the `allow-git` list of `deny.toml`, and `make deny` passes. Public closed wire types use the owning `quire-canonical::Encode` and `FixedShape` contracts where their shapes permit; the package graph and diagnostics encode complete values. Source inspection proves that removed `CheckedSemanticId` and identity-preimage types, hand-written `FixedShape` implementations, wrapper types and copies of quire-canonical source or vectors are absent. | Test (TC-048) |
| FR-038-AC-81 | Observed at the operation check of a body-root application, an application whose `operation.identity` the operation catalog does not list refuses `invalid_package` with cause `unknown-operation` at `operation.identity`; `quire.op.integer.add`, a `binary` operation, under the operator `unary` refuses `invalid_package`/`operation-class-mismatch` at `operator`; and `quire.op.integer.add` under `binary` with two operands, no law, no mode and no member admits, so neither refusal is a refusal of every application. Their order against every other check of the entry is FR-038-AC-67's. An application whose `operation` is a bare string, not an object of the closed operation shape, refuses `invalid_semantic_graph` at that application's `operation`. Implemented required-presence correction: through `CheckedPackageV2::read`, independently omitting `operation.mode` or `operation.member` from an otherwise valid application refuses `invalid_semantic_graph`, with no cause, at `/semantic_graph/nodes/{i}/body/operation`, before catalog semantic checks; the otherwise identical application with both fields present as `null` admits when its catalog entry requires neither. Retain each structural node id after mutation and check the operation shape refusal directly; no content-derived identity can mask it. Explicit-null required-mode and required-member semantic refusals remain AC-83 and AC-84's. | Test (TC-048) |
| FR-038-AC-82 | Observed at the operation check, `quire.op.integer.div`, which catalogues the one law role `integer_division`, with no law refuses `invalid_package`/`operation-law-missing` at `operation.laws`; `quire.op.integer.add`, which catalogues no law, with one law refuses `invalid_package`/`operation-law-mismatch` at `operation.laws/0`; and `quire.op.integer.div` with one law whose `role` is the word `not_integer_division` and whose `definition` is a catalogued `integer_division` definition refuses `invalid_package`/`operation-law-mismatch` at that law's `role`, so the role is checked in its own right and not only through the definition. | Test (TC-048) |
| FR-038-AC-83 | Observed at the operation check, `quire.op.decimal.add`, which catalogues the mode kind `rounding`, with `mode` `null` refuses `invalid_package`/`operation-mode-mismatch` at `operation.mode`; with the mode `{kind: rounding, value: toward-zero}` over a first operand typed at a `bounded_domain` over a `decimal` whose body binds `rounding` to `nearest-even`, reached by a `reference` to the domain and, separately, by a `literal` whose `type` names it, it refuses `invalid_package`/`operation-mode-type-mismatch` at `operation.mode/value`. | Test (TC-048) |
| FR-038-AC-84 | Observed at the operation check, `quire.op.quantity.convert`, which catalogues the member kind `type_argument`, with its `rounding` mode supplied and `member` `null` refuses `invalid_package`/`operation-member-mismatch` at `operation.member`; `quire.op.record.project` with a `field` member whose `name` is a field the declaring record type does not declare refuses `ill_typed`/`operator-ineligible` at `operation.member.name`; and `quire.op.model.reaches_field` with a `field` member whose `declaration` names no node of the graph refuses `ill_typed`/`operator-ineligible` at `operation.member.declaration`. | Test (TC-048) |
| FR-038-AC-85 | Observed at the operation check, `quire.op.integer.add`, whose entry has two fixed operands and no rest operand, with three arguments refuses `ill_typed`/`operator-ineligible` at `arguments`; with two `literal`s typed at an `integer` node it admits, and with a first `literal` typed at a `boolean` node it refuses the same way at `arguments/0`; `quire.op.boolean.not` over an application of `quire.op.state.clause`, whose catalogued result is `clause`, refuses the same way at `arguments/0`. | Test (TC-048) |
| FR-038-AC-86 | Observed at the operation check, `quire.op.structural.eq`, whose `same_type` constraint covers both operands, admits two distinct `value`/`parameter` nodes typed at one record type, two `literal`s typed at one type, and refuses `ill_typed`/`operator-ineligible` at `arguments/1` for parameters typed at two record types and `literal`s typed at two types, so the constraint compares the types the operands resolve to and not the operand nodes. | Test (TC-048) |
| FR-038-AC-87 | The operand family of a node is exactly, by tag and form: `scalar_type` `boolean`, `integer`, `rational`, `decimal`, `float32`, `float64`, `text` and `enum`, and `composite_type` `option`, `sequence`, `set`, `bag`, `ordered_set`, `record`, `tuple` and `reference`, each its own name; `expression`/`reference`, `reference`; `function` `pure_function`, `predicate` and `recursive_function`, `function`; `model` `object_type` and `systems_interface`, `object`; and `relation`/`population`, `population`; and `temporal`/`formula`, `temporal` (FR-038-AC-68); and, since IR-549, `composite_type`/`union`, `union` (FR-038-AC-99); every other kind of the closed node taxonomy has none, and the pairs are compared whole, so a form moved to another family or added to the list fails it. A kind is type-shaped exactly when its tag is `scalar_type`, `composite_type`, `bounded_domain`, `relation` or `function`, or it is `expression`/`reference` or `temporal`/`formula`, checked for every kind of the taxonomy. The `temporal`/`formula` pair arrives with the IR-503 code change; until it lands the pairs the unit test compares do not include it. | Test (TC-048) |
| FR-038-AC-88 | A two-member recursion group with an application referencing one member and an outside node retains its exact body, shared non-empty label and structural `{package,path,ordinal}` IDs. In-group and outside references each resolve by their exact structural target ID. A malformed structural ordinal, duplicate ID or unresolved target refuses at its own graph or wire locus. The former group-position/group-reference-ordinal branch is withdrawn in place because no such checked-wire carrier exists. | Test (TC-048) |
| FR-038-AC-89 | The reader preserves published structural node ids, complete graph bodies and canonical package bytes. An independently constructed `{package,path,ordinal}` for each positive declared or derived node equals its wire id; a changed declaration path is checked against the qualified name, while owner or body edits are checked by their independent joins and body rules without re-deriving an id. The test reads QSpec artifacts directly from its authoritative checkout and carries no fixed digest or key golden. | Test (TC-048, TC-228) |
| FR-038-AC-90 | Withdrawn: `NominalIdentityPreimage::digest` and its byte ceiling do not exist. Canonical envelope byte-limit behavior is required by FR-051-AC-2. | Test (TC-048) |
| FR-038-AC-91 | The canonical bytes of a representative complete V2 envelope equal independently authored expected bytes under `quire-canonical`; one changed semantic member changes those bytes while a source-map edit changes only its source-map bytes. No node or package name is computed. The bound V1 result of FR-023 has no digest or digest preimage. | Test (TC-048) |
| FR-038-AC-92 | Withdrawn by the IR-1141 structural-identity decision under QSpec #517: the preimage types and their FixedShape proof are removed with content-derived identities. This id is retained and never reused. | Withdrawn |
| FR-038-AC-93 | A selected model document holding the number `9007199254740993`, `-9007199254740993`, `9.007199254740993e15`, `1e20` or `18446744073709551617` at `/package/count` refuses `noncanonical_wire` at `/semantic_graph/nodes/<i>/owner/identity` with `document_pointer` equal to `/package/count`; a document holding such numbers at `/b` and then `/a/0` names `/b`, the first in document order (except that a number with no finite double is named when the reader reaches it, FR-038-AC-110); the same document holding `9007199254740992`, `-9007199254740992`, `9.007199254740992e15` or `0.5` at that pointer is not refused for it ; an integer value type whose upper bound is `9007199254740993` in a document that is otherwise admitted refuses the same way, where today it is admitted (the reader change); and a package document refused `noncanonical_wire` for its own bytes (FR-038-AC-79) carries no pointer and no `document_pointer`. | Test (TC-048) |
| FR-038-AC-94 | A selected model document whose bytes are exactly `limits.bytes` long is read and admitted, with the package itself shorter than that limit; read with a `limits.bytes` one lower the same package returns `incomplete` for `bytes` with that limit and the document's length reported and no pointer, as FR-038-AC-26 states for the byte limit, while a `work` limit one below the work of the same read still returns the root pointer `/semantic_graph/nodes/<i>/owner` (FR-038-AC-30). | Test (TC-048) |
| FR-038-AC-95 | PLANNED/UNRUN (IR-1141). Bounded canonical encoding of a complete typed node and package succeeds at the exact retained byte ceiling and returns a typed bytes-limit failure one byte below, without a partial value or panic. In a multi-request lowering call, a node-local byte failure preserves successfully lowered independent sibling records; a whole-package byte failure returns no lowered node, dependency node or package bytes. Each failure reports the configured limit and consumed work consistently; a work-budget failure remains distinct. No identity preimage or hash is constructed. | Test (TC-048) |
| FR-038-AC-96 | Each of the fifteen `temporal_formula` identities (`holds`, `true`, `false`, `not`, `and`, `or`, `implies`, `eventually`, `always`, `once`, `historically`, `until`, `release`, `since`, `triggered`) admits as the body root of a `temporal`/`formula` node, with operator `temporal_formula`, no law, `mode` `null`, no leaves, its catalogued member (`temporal_interval` with a closed interval on the eight interval operators, `null` on the other seven, under a clause selecting a bounded `temporal_profile`) and its catalogued operands (`holds` over one `boolean` operand, `true` and `false` over none, `not` and the four unary interval operators over one `reference` to a `temporal`/`formula` node, `and`, `or`, `implies` and the four binary interval operators over two); each formula node is referenced from a clause's formula argument or a formula operand; `quire.op.temporal.fair` admits as the body root of a `temporal`/`fairness` node with a well-formed `fairness` member and no operand, referenced from the fairness argument of a clause selecting `quire.temporal.infinite-trace/v1` (a bounded profile admits no fairness argument); and a package holding a `quire.op.temporal.clause` node whose sixth argument references a `temporal`/`formula` node applying `quire.op.temporal.eventually` with the member `{kind: temporal_interval, interval: {lower: "0", upper: "3"}}` over a second formula node applying `quire.op.temporal.holds` admits with its exact complete bodies and structural node ids retained, and lowers under a profile that supports the `temporal` tag and returns `unsupported` naming `temporal` under one that does not. | Test (TC-048) |
| FR-038-AC-97 | On each of the eight interval operators the member `{kind: temporal_interval, interval}` admits, under `quire.temporal.infinite-trace/v1`, with `interval` `{lower: "0", upper: "3"}`, `{lower: "9", upper: "10"}`, `{lower: "2", upper: null}` and `null`; `{lower: "-1", upper: "3"}` refuses `invalid_package`/`invalid-value` at the bound (`/semantic_graph/nodes/{n}/body/operation/member/interval/lower`), `{lower: "0", upper: "-2"}` refuses the same way at `.../interval/upper`, and `{lower: "-5", upper: "-2"}` refuses at `.../interval/lower`, the first bound in member order, each in strict wire validation, before any identity check and any temporal step and under every profile (merged QSpec FR-370-AC-9 and the schema's non-negative bound pattern); every other bound outside the pattern (`"1.5"`, `"01"`, `"+1"`, `""`, `"3x"`, and the JSON integer `0`, which is no string) refuses `invalid_package`/`invalid-value` at that bound, `{lower: "1.5", upper: "0"}` at `.../interval/lower`, in the same early stage and first in member order (`lower`, then `upper`), never `operation-member-mismatch` (merged FR-370: "a bound outside its form's pattern ... refuses `invalid_package`/`invalid-value` at that bound's pointer during strict wire validation"); `{lower: "3", upper: "0"}` and `{lower: "10", upper: "9"}` refuse `invalid_package`/`invalid-value` at the application (`/semantic_graph/nodes/{n}/body`), also when the formula node is reached by no clause (the sweep after every clause, an IR reading) and `{lower: "18446744073709551617", upper: "18446744073709551616"}` refuses the same way, so the comparison is numeric and neither lexicographic nor fixed-width; a member of kind `fairness` and an interval holding a third member each refuse `invalid_package`/`operation-member-mismatch` at `operation.member` (an IR reading, merged text silent), and a `null` member on an interval-capable operator refuses `invalid_package`/`operation-member-mismatch` at the application (`/semantic_graph/nodes/{n}/body`, merged FR-370); and any member on `quire.op.temporal.holds`, `quire.op.temporal.not` or `quire.op.temporal.clause` refuses `operation-member-mismatch` at `operation.member`. | Test (TC-048) |
| FR-038-AC-98 | The `fairness` member of `quire.op.temporal.fair` admits as `{kind: fairness, fairness_kind: weak, granularity: whole, target: {kind: operation, declaration, name}}` and as `strong` with `each`, with a `declaration` and `name` that resolve (the resolution refusals are FR-038-AC-103's); a `fairness_kind` of `medium`, a `granularity` of `part`, an operation target lacking `name`, a member holding an extra member, a `null` member and a member of kind `temporal_interval` each refuse `invalid_package`/`operation-member-mismatch` at `operation.member`; the closed protocol-root, attempt, parallel-branch, compensation-template and hybrid-transition targets preserve their exact identity and selected target data in the canonical typed read, while a missing or unknown target kind, extra target member or malformed structural identity never admits; and a `fairness` member on `quire.op.boolean.not` refuses the same way. | Test (TC-048) |
| FR-038-AC-99 | A package holding a `composite_type`/`union` node `Shape` with the members `Circle(Integer)`, `Rect(Integer, Integer)` and `Empty`, a `value`/`union_value` node `Shape::Rect(2, 3)` and an `expression`/`case` node whose body root applies `quire.op.control.case` over the scrutinee and the arms `Circle`, `Rect` and `Empty` in member declaration order, the binders `r`, `w` and `h` as `value`/`parameter` nodes typed `Integer` and every arm body of the `result_type`, admits, the three forms decoded and the operand family of the `Shape` node `union`; the same `case` with its arms out of member order, with the `Empty` arm omitted, with the `Circle` arm repeated, with the `Rect` binder aggregate holding one reference, with the `Circle` binder typed `Text` and with one arm body of another type than the `result_type` each refuse `ill_typed`/`operator-ineligible` at the `case` node (`/semantic_graph/nodes/{n}`); a union with no member and a union value with two bindings each refuse `invalid_semantic_graph` at the node's `body`; a union type with `Circle` twice refuses `invalid_package`/`duplicate-member` and a payload reference to a node that is not a type refuses `ill_typed`/`operator-ineligible`, each at the union node; and a union value naming `Triangle`, a `Rect` with one payload term and a `Circle` over a `Text` payload each refuse `ill_typed`/`type-mismatch` at the node. | Test (TC-048) |
| FR-038-AC-100 | Placement in both directions: an application of operator class `temporal`, `temporal_formula` or `temporal_fairness`, naming any identity, refuses `ill_typed`/`operator-ineligible` at the node that holds it when it is the body root of a node of another form (a `temporal_formula` application in a `function` node, a `quire.op.temporal.fair` application in a `temporal`/`formula` node), while the same application as an element of another application's `arguments`, a `binding` value or inside an `aggregate` refuses `malformed_wire` at the nested application at strict wire validation, ahead of every step here (FR-038-AC-114); a `temporal`/`formula` node, a `temporal`/`fairness` node and a `temporal`/`temporal_clause` node whose body is an empty `aggregate`, a `literal`, or an application of another class refuses the same way at the node; a `reference` to a `temporal`/`formula` node from a `function` node's body, from a clause's fairness argument or from a `case` argument, and a `reference` to a `temporal`/`fairness` node from a clause's formula argument or from a `temporal_formula` operand, refuses the same way at the node that holds the reference; a `case` application refuses `ill_typed`/`operator-ineligible` at the node that holds it when it is the body root of a node that is not an `expression` node, at the operation step and so after every temporal-step defect whatever the structural id order (merged FR-440 join 1; not in the temporal placement pass), and at its own `operator` (`/semantic_graph/nodes/{n}/body/.../operator`) when it is nested in another term (merged FR-322 "Body grammar"); an `expression` node whose `semantic_form` contradicts its root application's operator class (an `expression`/`case` node whose body is not a `case` application, an `expression` node of another form whose body root is a `case` application) refuses `invalid_semantic_graph` at the node's `body` (merged QSpec FR-440 and FR-322); a `diagnostics.entries[].details[]` term that references a `temporal`/`formula`, `temporal`/`fairness` or `expression`/`case` node refuses `ill_typed`/`operator-ineligible` at that entry (`/diagnostics/entries/{e}/details/{d}`; the `case` node reference is merged FR-370-AC-12), a `details` reference to a `composite_type`/`union` or `value`/`union_value` node is an ordinary reference and admits, and an application of the `temporal_formula`, `temporal_fairness` or `case` class as the root of or nested in a `details` term refuses `ill_typed`/`operator-ineligible` at that application's `operator` (`/diagnostics/entries/{e}/details/{d}/operator`, and `.../members/0/operator` for the first member of a `details` aggregate; merged FR-370-AC-12 and FR-322 "Body grammar"), the first in document pre-order, outermost first, while an application of any other class there refuses `malformed_wire` at that application (FR-038-AC-115); and each application, node and reference at its own place admits (FR-038-AC-96, FR-038-AC-99), so none of the refusals is a refusal of every such term. The in-repo `v2_all_families` formula node, whose body is an empty `aggregate`, is refused as above and the code change replaces its body. | Test (TC-048) |
| FR-038-AC-101 | Admitted temporal/formula and expression/case applications are not blanket-refused `unsupported_construct`/`expression-form`. IR-661 adds `CheckedPackageRefusalCode::UnsupportedConstruct` and `CheckedPackageRefusalCause::ExpressionForm` only for FR-152's unsupported unbounded or ordered relationship-navigation destination (PLANNED / UNRUN; AC-170); this does not restore the original temporal/case refusal; the diagnostics wire vocabulary `CheckedDiagnosticCode` still carries `unsupported_construct`, so a `diagnostics.entries[]` entry whose `code` is `unsupported_construct` reads as QSpec's schema allows; no application is evaluated by the reader or the lowerer, so an admitted `temporal`/`formula` node and an admitted `expression`/`case` node lower, under a profile that supports their tags, to nodes retaining the exact admitted structural IDs and bodies, and a profile lacking the tag returns `unsupported` naming it (FR-038-AC-8). | Test (TC-048) |
| FR-038-AC-102 | The temporal step runs after the frame and state-clause step and before the operation step, placement first: a package holding one node with a placement defect (a `temporal_formula` application as the body root of a `function` node) and a second with a earlier structural `node_id` whose operation identity is unknown refuses for the placement defect, and so does the same placement defect beside a earlier-structural-id node whose `operator` differs from its entry's class (`quire.op.control.case` under `unary`), in both structural id orders each, the placement defect taken in the shape of a `temporal_formula` application in a `function` node; two placement defects are reported at the earlier structural `node_id`; a placement defect is reported ahead of any clause defect in either structural id order; within one clause an `over` defect is reported ahead of a fairness-resolution defect, ahead of a profile-fit defect, ahead of an interval-bounds defect, each adjacent pair built as two defects of one clause and in both orders of the two defects' positions; and across two clauses the earlier-structural-id clause's later-stage defect (a profile-fit defect) is reported ahead of the later-structural-id clause's earlier-stage defect (an `over` defect), in both structural id orders of the two clauses. The temporal step skips a member whose shape the operation step refuses (a wrong-kind member, an interval with a third member), and a `null` member on an interval operator under a bounded profile refuses `invalid_package`/`operation-member-mismatch` at the application. Every bound outside the schema pattern, negative or malformed (`{lower: "1.5", upper: "0"}` included), is a schema-pattern failure, refused `invalid_package`/`invalid-value` at the bound in strict wire validation, before placement and every temporal step, first in member order, under every profile (merged QSpec FR-370-AC-9; there is no asymmetry between a negative and a malformed bound): `{lower: "0", upper: "-2"}` refuses at `.../interval/upper`, `{lower: "-1", upper: "-3"}` at `.../interval/lower`, and `{lower: "-1", upper: null}` refuses at `.../interval/lower` under a bounded profile, under `quire.temporal.infinite-trace/v1` and under a clause that also holds a placement defect at a earlier-structural-id node, never `operation-member-mismatch`; `lower > upper` stays in the bounds step, after profile fit, so `{lower: "3", upper: "0"}` under a bounded profile in a clause whose profile fit also fails refuses the profile-fit defect first. | Test (TC-048) |
| FR-038-AC-103 | A clause whose `over` argument is a `reference` to a `value`/`parameter` node among its `dependencies` admits; one whose `over` references a `value`/`parameter` node that is not among its `dependencies` refuses `missing_declaration`/`missing-name`, and one that references a declared dependency that is a `scalar_type` node refuses `invalid_model_binding`/`malformed-declaration`, each with path `/semantic_graph/nodes/{clause}/body/arguments/0` and the locus of the "Path and locus" table; a fairness member whose `declaration` and `name` resolve to an operation of a `model`/`object_type` declaration node admits (FR-370 "Fairness resolution" steps 1 to 3, in that order), one whose `name` is no operation of that node refuses `missing_declaration`/`missing-name` with path `/semantic_graph/nodes/{fairness node}/body/operation/member/target/name` and the `declaration` target's key as locus, and one whose `declaration` names a node that is not a `model`/`object_type` declaration (a `scalar_type` node, a `model`/`value_type` node, or a key that names no node) refuses `invalid_model_binding`/`malformed-declaration` with path `/semantic_graph/nodes/{fairness node}/body/operation/member/target/declaration` and the `declaration` target's key as named as locus; the `over`-not-among-`dependencies` case is observed at the unit level of the temporal step, as AC-68 observes the clause check, because the application-node dependency join refuses a package that names a non-dependency reference first, while the package-level half (an `over` that names no node) is read through the reader; a `name` that matches two exposed operation members of the declaration node refuses `ambiguous_declaration`/`ambiguous-name`, one that the node only inherits admits and resolves to its most-derived redefinition, and a declaring node whose owner is not recovered refuses with that resolution's own refusal (`missing_declaration`/`missing-selection` or an independent graph refusal), the ambiguous name with path `.../member/target/name` and the `declaration` target's key as locus and the unrecovered owner with path `.../member/target/declaration` and the fairness node's key (merged QSpec FR-370-AC-11 and FR-370 "Fairness resolution" state the ambiguous, inherited-admitted, unrecovered-owner and malformed-declaration outcomes; the loci of the name and owner rows are an IR reading). Protocol fairness resolves through its owning clause: root, attempt, branch and compensation targets admit only with the matching clause membership and target kind; absent, foreign and other-clause identities refuse missing-name at target.node or target.parallel before kind checks, wrong local kind/operator refuses malformed-declaration there, and an absent branch label refuses missing-name at target.branch. | Test (TC-048) |
| FR-038-AC-104 | Under a clause whose `temporal_profile` law names `quire.temporal.event-position.false-extension/v1`, `quire.temporal.fixed-sample.false-extension/v1` or `quire.temporal.timestamped-event.finite-window/v1`, every interval operator of its formula tree with a closed interval admits, and one with a `null` interval or `{lower, upper: null}` refuses `invalid_package`/`operation-member-mismatch` at that operator's application; a non-empty fairness argument refuses `operation-member-mismatch` at the clause's application and an empty one admits; under `quire.temporal.infinite-trace/v1` the same `null` interval, `{lower, upper: null}` and a non-empty fairness argument admit; under `quire.temporal.timed/v1` a `null` interval admits and `{lower, upper: null}` and an integer-form closed `{lower, upper}` each refuse `operation-member-mismatch` at that operator's application (merged QSpec FR-370-AC-3: a timed interval is the timed form; under `timed/v1` only `null` and the timed form are admitted, confirmed by the QSL ruling relayed 2026-10-03), and a non-empty fairness argument admits under it as under infinite-trace (only the three bounded profiles refuse one, as above; merged QSpec FR-370-AC-4); the timed form itself, `{lower, upper, lower_end, upper_end}`, is FR-038-AC-119 through FR-038-AC-122's (merged FR-370-AC-8), and this criterion's fit rows are unchanged by it; and the profile fit reads only the formula nodes reachable from that clause. | Test (TC-048) |
| FR-038-AC-108 | Each of the five members of QSpec FR-250's Values table admits as a clause's `temporal_profile` law (the lock's `temporal_profile` row naming it). A clause whose `temporal_profile` law `definition` is not one of the five refuses `unknown_profile` at `/semantic_graph/nodes/{n}/body/operation/laws/0/definition`, ahead of the same clause's `over` defect and of any defect of a later-structural-id clause, and is not read as bounded (a `null` interval under it refuses `unknown_profile` and not `operation-member-mismatch`). The cause, merged QSpec FR-370 "Profile check" and FR-370-AC-10, is decided from the package alone: `wrong-selection-role` when the same `{authority, identity}` is a row of the package's own `lock.profile_selections` (or the `lock.edition` row) under a `role` other than `temporal_profile`, `unsupported-selection` otherwise. A known profile is matched by identity label alone, `authority` taking no part: a law naming a FR-250 member's identity under a different `authority` is a known profile and refuses `operation-law-unselected` at AC-57's join, not `unknown_profile`. So `quire.fixture.temporal-profile/v1` selected nowhere else, an empty identity and a differently spelled `quire.temporal.infinite-trace/v1` refuse `unsupported-selection`; `quire.package.composed/v1` named by the law and selected in the lock as a `profile_selections` row of role `binding_contract` refuses `wrong-selection-role`, and so does `quire.protocol.complete/v1` selected as a `protocol_profile` row; and the same two identities named by the law with no such row in the lock refuse `unsupported-selection`. A clause whose `laws` is not exactly one law of role `temporal_profile` (none, or two) skips the profile check and its profile fit, and its law defect refuses at the operation step (`operation-law-missing`, `operation-law-mismatch`; merged QSpec FR-370 "Profile check"). The refusal is the reader's `CheckedPackageRefusalCode::UnknownProfile` with the causes `unsupported-selection` and `wrong-selection-role` in `CheckedPackageRefusalCause`, a code and two causes the code change adds. | Test (TC-048) |
| FR-038-AC-105 | The temporal and case operands are checked at the operation step, after the temporal step: `quire.op.temporal.holds` over a `reference` to a `temporal`/`formula` node, and over a Boolean-family `reference`, admits only the latter, the former refusing `ill_typed`/`operator-ineligible` at the argument; `quire.op.temporal.until` with one argument, `quire.op.temporal.not` with two, `quire.op.temporal.true` with one, and `quire.op.temporal.and` over a Boolean-family `reference` each refuse `ill_typed`/`operator-ineligible` at `arguments` or at the argument; and `quire.op.temporal.not` over a `reference` to a `temporal`/`formula` node admits. | Test (TC-048) |
| FR-038-AC-106 | `quire.op.structural.eq` over two values of the `Shape` union admits, and over a `Shape` value and a value of another union refuses `ill_typed`/`operator-ineligible` at `arguments/1` by its `same_type` constraint (QSpec FR-440-AC-6); over a union whose payload types reach no `text` type, `leaves` empty admits; and over `union Label { Named(Text[0, 8; nfc]), Tagged(Integer, Text[0, 8; nfc]), Empty }`, a comparison admits with exactly the leaves `["member:Named", "position:0"]` and `["member:Tagged", "position:1"]` in member declaration order, and, following FR-038-AC-70's convention, a list holding only the `Named` leaf, and a list holding only the `Tagged` leaf, each refuse `invalid_package`/`operation-law-missing` at `operation.leaves`, while the list of both leaves followed by the `Named` leaf again, and the list of both followed by a leaf `["member:Empty"]`, each refuse `invalid_package`/`operation-law-mismatch` at `operation.leaves/2` (the `member:<Ident>` segment is merged QSpec FR-322-AC-45); and over the recursive unions `union IntList { Cons(Integer, IntList), Nil }` and `union TextList { Cons(Text[0, 8; nfc], TextList), Nil }`, `structural.eq` admits (a cycle through a union is never refused `ill_typed`/`operator-ineligible`, merged QSpec FR-322-AC-46 and FR-440-AC-8): over `IntList`, which reaches no `text`, with `leaves` empty; over `TextList` with exactly the leaves `["member:Cons", "position:0"]` and the recursion leaf `["member:Cons", "position:1", "recursion:0"]` (`d` is 0, the segments the path held when `TextList` was entered, as for a record's `recursion:<d>`), a list lacking the recursion leaf refusing `operation-law-missing` at `operation.leaves`, one whose recursion leaf reads `recursion:1` refusing `operation-law-mismatch` at that leaf's `path`, and a recursion leaf at a reentry of `IntList` refusing `operation-law-mismatch` at its `path`. | Test (TC-048) |
| FR-038-AC-107 | PLANNED/UNRUN (IR-1141). make conformance-qspec reads every QSpec positive V2 fixture directly from QUIRE_SPECIFICATION_DIR and requires each to admit through the production reader with its published name, structural graph ids and complete bodies. The target fails if the checkout or fixture is absent; no QSpec artifact is copied into IR. Self-built clause, temporal and case controls independently check the same admitted body features. | Test (TC-048) |
| FR-038-AC-109 | A selected model document holding at `/package/ratio` the number `0.1000000000000000000001`, `9007199254740993.5`, `-0.1000000000000000000001`, `4.9e-324` or `1e-400` refuses `noncanonical_wire` at `/semantic_graph/nodes/<i>/owner/identity` with `document_pointer` equal to `/package/ratio` and cause `inexact-number` (quire-specification:FR-272); the same document holding `0.1`, `0.5`, `1.5`, `-0.25`, `5e-324`, `2.5e-10`, `1.0`, `-0` or `1e2` at that pointer (`1.0`, `-0` and `1e2` are spelled other than the text `quire-canonical` writes, `1`, `0` and `100`, and each is admitted because the value is the same), each a number whose exact decimal value equals that of its double's shortest round-trip text, is not refused for it ; the text that decides is the one `quire-canonical` writes for the number, which where two shortest texts are equally close to the double is the one with the even last digit, so a document holding `1125899906842624.2`, `1500000000000000.2` or `2.9802322387695312e-8` at that pointer (each the even-digit text of a tie) is not refused and one holding `1125899906842624.3`, `1500000000000000.3` or `2.9802322387695313e-8` refuses `inexact-number`, and the two texts are compared by their digits and scale, never as doubles; two documents that differ only in `0.1` and `0.1000000000000000000001` at that pointer are not both admitted, the second refusing as above even if binary floating-point parsing would collapse their values; the decision is made on the number's text read by `quire-canonical`, through no `serde_json` value, so it is the same whatever features other crates in the build turn on. | Test (TC-048) |
| FR-038-AC-110 | The cause of each refusal FR-038-AC-93 names is `inexact-integer`, with the `document_pointer` AC-93 gives, for `9007199254740993`, `-9007199254740993`, `9.007199254740993e15`, `1e20`, `18446744073709551617`, `9007199254740993.0`, `1e400` and `-1e400`, and for an integer value type whose upper bound is `9007199254740993`; `9007199254740993.0` is `inexact-integer` and never `inexact-number` although its text also differs from its double's shortest round-trip text, and `1e400` and `-1e400` refuse `noncanonical_wire` at that `owner/identity` with the `document_pointer` of the number; `9007199254740992`, `-9007199254740992` and `9.007199254740992e15` are unchanged and admitted, while `9007199254740992.5` (whose nearest double is `9007199254740992`) refuses `inexact-number`; a document holding an inexact number at `/b` and then a whole number past 2^53 at `/a/0` names `/b` with `inexact-number`, the first in document order, and the reverse order names the whole number with `inexact-integer`, except that a number with no finite double is named when the reader reaches it, and so can be named ahead of an earlier inexact number; when faults coexist the first fault `quire_canonical::read` returns decides (QSL FR-056), so `{"b":0.1000000000000000000001,"a":[1e400]}` and `{"b":9007199254740993,"a":[1e400]}` each refuse `noncanonical_wire`/`inexact-integer` with `document_pointer` `/a/0`, `{"a":1,"a":2,"n":1e400}` names `/n`, `{"n":1e400,"a":1,"a":2}` names `/n` and `[1e400` names `/0`, while `[1e400,"<0xFF>"]` (invalid UTF-8, which the read checks over the whole input first) refuses with no `document_pointer`, and, as this reader's reading, `[{"a":1,"a":2},1e400]`, `{"a":1,"a":2,"n":1e-400}` and `[1e400,"<0xFF>"]` refuse `malformed_wire`/`malformed-json` at that `owner/identity` because a reader fault other than an out-of-range number comes first; `1e-400` and `-1e-400` refuse `inexact-number`. | Test (TC-048) |
| FR-038-AC-111 | A package document holding an inexact numeric JSON token in a node body refuses noncanonical_wire before graph or body checks; a canonical numeric token at the same position is not refused for that cause. | Test (TC-048) |
| FR-038-AC-112 | PLANNED/UNRUN (IR-1141). make conformance-qspec reads QSpec adverse.json directly from QUIRE_SPECIFICATION_DIR and applies every structural and body-grammar mutation to a fresh positive package; each production read must return the published code and cause. The harness does not refresh identities or construct a projection. Each flattened body control reaches the later applicable reader check. Missing checkout, absent lists, missing flattened controls, stale expected-failure entries and mismatched outcomes fail the run. | Test (TC-048) |
| FR-038-AC-113 | `make conformance-qspec` reads the current authoritative dependency-selection vectors from `QUIRE_SPECIFICATION_DIR`, never copying them into IR. It checks each closed `{identity}` selection, exact structural supply and diamond result and fails if the checkout, vector or expected refusal is missing. No package identity is derived or compared to a recorded value. | Test (TC-048) |
| FR-038-AC-114 | A node body in which an application of any operator class other than `case` stands as an element of another application's `arguments`, as a member of an `aggregate` or as the value of a `binding` refuses `malformed_wire` at that nested application, for `quire.op.function.call`, `quire.op.state.clause` and an application of each of the `temporal`, `temporal_formula` and `temporal_fairness` classes, in each of the three positions, at a pointer such as `/semantic_graph/nodes/{n}/body/arguments/0`; an `aggregate` inside a Group's members, a `binding` whose value is an `aggregate` inside an `aggregate`'s members where the stratum admits only a Leaf value, a `binding` whose value is a `binding`, a `binding` or a Tuple inside a Tuple's members (a Tuple's members are each a Leaf or a Group) and a `binding` as a body root refuse `malformed_wire` at that value; and the same meaning with each composite subterm as its own node reached by `reference` is not refused `malformed_wire`. | Test (TC-048) |
| FR-038-AC-115 | A `case` application nested inside another term refuses `ill_typed`/`operator-ineligible` at the nested application's `operator`; a non-`case` application at the body root of a node its class does not place it in refuses `ill_typed`/`operator-ineligible` at that node and not `malformed_wire`; and where one body or `details` term holds several offending constructs the refusal is at the first in document pre-order, outermost first: a `details` term that is a `temporal_formula` application with a nested `case` argument refuses at `/diagnostics/entries/{e}/details/{d}/operator`, a `details` aggregate of a `temporal_formula` application and then a `case` application refuses at `/diagnostics/entries/{e}/details/{d}/members/0/operator`, a node body whose `arguments/0` is a nested `case` application and whose `arguments/1` is a nested `quire.op.function.call` application refuses `ill_typed`/`operator-ineligible` at `.../body/arguments/0/operator` and, with the two swapped, `malformed_wire` at `.../body/arguments/0` (an IR reading: merged FR-322 gives the pre-order rule, merged FR-440 decides a nested `case` at the operation step, and the two are unreconciled), and an application of a class other than `temporal_formula`, `temporal_fairness` and `case` in a `details` term refuses `malformed_wire` at that application. | Test (TC-048) |
| FR-038-AC-116 | A package with a malformed nested non-`case` application refuses `malformed_wire` at that term before graph joins; a nested `case` application refuses `ill_typed`/`operator-ineligible` at its operator. Flattening the body removes that defect and proceeds to the next independent graph or owner check, with no stale-key or package-id stage. | Test (TC-048) |
| FR-038-AC-117 | `CheckedPackageReadLimits` has no depth member and `CheckedPackageLimit` has no `Depth` variant, and no source file under `crates/quire-contract-model/src/checked_package/` holds `MAXIMUM_DEPTH`, another `MAX_*DEPTH` constant, `stacker`, `serde_stacker` or `on_stack_for` (the crate's v1 modules, their limits and its manifest are out of this scan: FR-019 and FR-023 keep them); a package holding a chain of 100000 nodes, each referencing the previous (a 100000-deep expression, as merged QSpec FR-322-AC-41 and TC-427 BG-03 build it), read under byte, node, edge, occurrence, diagnostic and work limits all sized so that none decides the outcome, on a thread whose stack is 256 KiB, is admitted and lowered from its last node with no outcome naming a depth and no stack overflow, and has the same JSON nesting depth as a package of one level; and an otherwise canonical document whose node body nests a term 300 levels deep, past the strict parse's recursion limit of 128, read on a thread whose stack is 256 KiB, refuses `malformed_wire` with no pointer at the strict parse, ahead of the canonical-bytes check and the body grammar, and is never `incomplete`, while the same document nested 20 levels deep, and one nested within the strict parse's limit but far past the grammar (a body of 61 aggregates, 126 JSON levels), each refuse `malformed_wire` at the first value outside the body grammar (FR-038-AC-114), on a 256 KiB thread in a debug build with no stack overflow. | Test (TC-048) |
| FR-038-AC-118 | Each of the five `body_grammar_mutations` of QSpec's `adverse.json` (`application-in-application-arguments`, `application-in-aggregate-members`, `application-in-binding-value`, `aggregate-in-group-members` and `binding-as-body-root`) refuses `malformed_wire` as its recorded `outcome` names, through the harness of FR-038-AC-112, whose expected-failure list holds none of them. | Test (TC-048) |
| FR-038-AC-119 | On each of the eight interval operators under a clause whose `temporal_profile` law names `quire.temporal.timed/v1`, the timed form `{lower, upper, lower_end, upper_end}` admits with bounds `{numerator: "0", denominator: "1"}` and `{numerator: "3", denominator: "1"}` for each of the four end pairs `closed`/`closed`, `closed`/`open`, `open`/`closed` and `open`/`open`; it admits with `lower` `{numerator: "1", denominator: "2"}`, with `lower` and `upper` both `{numerator: "3", denominator: "1"}` and both ends `closed` (a punctual `[3, 3]`), and with `lower` `{numerator: "0", denominator: "1"}`; and a `null` interval on the same operators admits (merged FR-370-AC-3 and FR-370-AC-8). | Test (TC-048) |
| FR-038-AC-120 | Each invalid timed-form interval refuses invalid_package/invalid-value at its exact lower or upper bound during strict wire validation before graph and timed-body checks, including when another independent graph defect is present. | Test (TC-048) |
| FR-038-AC-121 | An unreduced timed rational bound refuses invalid_semantic_graph at its bound after strict wire validation and before the temporal step, including when another later timed defect is present. | Test (TC-048) |
| FR-038-AC-122 | The timed form `{lower, upper, lower_end, upper_end}` with bounds `{numerator: "0", denominator: "1"}` and `{numerator: "3", denominator: "1"}` and both ends `closed` refuses `invalid_package`/`operation-member-mismatch` at `/semantic_graph/nodes/{n}/body` under `quire.temporal.infinite-trace/v1` and under each of the three bounded profiles (merged FR-370-AC-8), and admits under `quire.temporal.timed/v1`, whose refusal of an integer-form `{lower, upper}` and of `{lower, upper: null}` is FR-038-AC-104's; four-member intervals with valid rational bounds and `lower_end` `"half"`, or `upper_end` `"half"`, each refuse `invalid_package`/`operation-member-mismatch` at `operation.member`, as do `{lower: "0", upper: "3", lower_end: "closed"}` (a missing end, integer-string bounds), `{lower: "0", upper: "3", lower_end: "closed", upper_end: "closed", extra: "x"}` (a fifth member, integer-string bounds), each an IR reading that diverges from the published schema, whose closed interval `oneOf` and end `enum` fail them at strict wire validation, and each reported at the operation step after the temporal step, so a profile-fit defect in a later-structural-id clause is reported first; and the same missing-end and fifth-member intervals with the rational-object bounds of the timed form refuse `invalid-value` at `.../interval/lower` instead, an interval of another member set having no form, its bounds judged against the integer pattern (an IR reading). | Test (TC-048) |
| FR-038-AC-123 | Withdrawn by the IR-1141 structural-identity decision: changing the body of a node under a retained derived path does not trigger a key or path recomputation. The old content-authentication tamper proof and its exact stale-key refusal no longer exist. The id is not reused. | Withdrawn |
| FR-038-AC-124 | PLANNED/UNRUN (IR-1141). Each `scalar_type`/`boolean` or `/integer` body is the empty aggregate; `reference`, `option`, `set`, `bag`, `sequence` and `ordered_set` have exactly one reference; `integer_range` and `collection_bounds` have ordered `min` and `max` integer-literal bindings of their specified decimal grammar. A missing, extra, reordered, wrongly typed or malformed member refuses `invalid_package`/`invalid-derived-shape` at that node without using a content-derived key. For `composite_type`/`reference`, a valid body target must be a model `object_type`, `process`, `state_machine` or `systems_interface` declaration, or a relation `relationship` declaration; a schema-valid reference to a scalar, function, value or other composite kind (`RM-REF-TARGET`) refuses that same code and cause at the reference node's `node_id` under QSpec FR-322 "Derived type forms and forwarding". Positive `integer_range` `min` `-5` and canonical extreme bounds admit. | Test (TC-226, TC-048) |
| FR-038-AC-125 | PLANNED/UNRUN (IR-1141). A `scalar_type` or `composite_type` in those ten shapes is self typed; an `integer_range` is typed at the exact Integer node and `collection_bounds` at a collection node. A wrong semantic type or a bound literal typed at Boolean refuses `invalid_package`/`invalid-derived-shape` at the offending node; a valid structural id alone does not bypass the body/type check. | Test (TC-226) |
| FR-038-AC-126 | Withdrawn by the IR-1141 structural-identity decision: the former preimage-key tamper control for indirect type redirection is not a structural-id check. Other declaration, type, body and graph validation remains under its own criteria; this id is not reused. | Withdrawn |
| FR-038-AC-127 | PLANNED/UNRUN (IR-1141). Graph shape and owner joins precede derived-shape semantic checks; an invalid derived shape precedes dependent declaration, frame, state, temporal, abstraction and operation checks. Two malformed structural nodes are visited in ascending componentwise structural id order. Each check reports its own exact typed refusal and pointer; no application, nominal, structural or group hash stage runs. | Test (TC-226) |
| FR-038-AC-128 | PLANNED/UNRUN (IR-1141). Fields declared at `Int[0, 0]` and at the signed 128-bit minimum and maximum admit with exact decimal strings; one-past signed-128 endpoints remain exact text and are rejected or retained according to the declared bound grammar, never rounded or narrowed. An unchanged structural id cannot cause an altered body bound to be read as the declared range without an exact member/type join. The latter control is an independent type and member join, not a derived-path comparison. | Test (TC-226) |
| FR-038-AC-129 | PLANNED/UNRUN (IR-1141). The reader admits against the selected document actually supplied under an identity, after checking that document names the identity and its declarations join the graph. It does not authenticate a historical document snapshot. Two valid documents supplied in separate reads under the same identity are checked independently; the package name is unchanged, while their differing complete JSON values remain distinguishable structurally. | Test (TC-226) |
| FR-038-AC-130 | PLANNED/UNRUN (IR-1141). The ten derived shapes use one QSpec structural `{package,path,ordinal}` rule at the producer and one reader validation of closed body and graph joins. The model-member type derived from a selected declaration equals the node body and semantic type returned by the reader. No `MemberType::node_key`, second preimage encoder or candidate hash path is exported. | Test (TC-226) |
| FR-038-AC-131 | Withdrawn by the IR-1141 structural-identity decision under QSpec #517: the reader does not re-derive structural ids from actual node members, so the former universal stale-key tamper control no longer applies. Other closed-body, owner and reference checks remain independently required. This id is not reused. | Withdrawn |
| FR-038-AC-132 | No copy (IR-627 checkout-reference harness implemented; IR-630 extended conformance PLANNED and UNRUN). Authoritative vectors for owner-bearing, declared and labelled-group forms remain at their owning QSL or QSpec checkout and are read there, not copied into this repository. A conformance read fails closed when its source is absent or malformed, as FR-038-AC-112 and AC-176 require. Test-side derivation controls do not replace published goldens. | Test (TC-226) |
| FR-038-AC-133 | PLANNED/UNRUN (IR-1141). The three authoritative QSpec positive fixtures of AC-107 admit end to end with structural ids on every node. Repeating an id, making its ordinal malformed, changing a declared path away from its qualified name or breaking a body/owner join refuses at the exact applicable stage. `make conformance-qspec` reads current QSpec fixtures and vectors directly from the selected checkout; no fixture or digest catalog is copied into IR. | Test (TC-226) |
| FR-038-AC-134 | PLANNED/UNRUN (IR-1141). Every in-repo V2 fixture in `tests/it/support/checked_package.rs` uses QSpec structural node ids and closed bodies for the ten derived shapes, and admits. A duplicate id, malformed derived body, wrong self type or broken reference refuses by the corresponding independent reader rule. No placeholder digest key is accepted. | Test (TC-226) |
| FR-038-AC-135 | PLANNED/UNRUN (IR-1141). Each current QSpec `adverse.json` structural and body-grammar mutation refuses with its recorded code, cause and pointer against the current structural-id positive fixture. The harness fails on a missing or unexpectedly admitted mutation, a stale expected-failure entry or a mutation whose actual refusal differs; it reads QSpec directly and does not refresh a hash after mutation. | Test (TC-226) |
| FR-038-AC-136 | Fields as values (IR-628 type behavior implemented; declaration order and declaring identity PLANNED/UNRUN, IR-1164/IR-1165). Over an admitted package whose `model`/`object_type` node is a model declaration node with the empty body (the shape QSL emits), selected from a domain document declaring `balance: Int[0, 1000]` and `audit: Int[0, 1000]`, with a read of `balance` only, `model_object_fields` returns exactly two fields, `balance` then `audit`, each with `member_type` `Some(IntRange { lower: 0, upper: 1000 })` as `i128` values; `field("audit")` is `Some` although no read names it, and `field("missing")` is `None`. Mutation rows: a reader that takes the fields from the node body returns none; one that takes them from reads' `result_type` omits `audit`; one that sorts by field name or iterates a hash map breaks the declaration-order row (a document declaring `zeta` before `alpha` must return `zeta` then `alpha`). | Test (TC-227) |
| FR-038-AC-137 | Effective set is the one admission resolves (IR-628; implemented, ungated). Over a document with a supertype chain, an inherited field, a field redefined by a subtype with a narrower `Int[lo, hi]`, a most-derived redefiner two owners deep and an own field, the accessor returns each exposed name once, with the most-derived redefiner's type, and no hidden field; for every returned field, a package whose read names the returned structural type id admits, and the same read naming the hidden base field's type refuses `ill_typed`/`operator-ineligible` (step 4). Mutation rows: inherited fields omitted; a redefined base field returned beside its redefiner; the base type returned for a redefined field; a less derived redefiner returned. | Test (TC-227) |
| FR-038-AC-138 | Every member kind (IR-628; implemented, ungated). Fields declared `Boolean`, `Integer`, `Int[0, 0]`, `Int[-5, 5]`, `Int[0, 9007199254740993]`, the `i128` extremes as `Int[lo, hi]`, a `Reference` to another object type, an optional `Int[0, 1000]`, a `Set`, `Bag`, `Sequence` and `OrderedSet` of `Int[0, 1000]`, a bounded collection `Set<Int[0, 1000]>[1, 4]` and an optional `Set` return `Boolean`, `Integer`, `IntRange` with the exact `i128` values, `Reference` carrying the key of the referenced model declaration node, `Option`, `Collection` with the matching `CheckedCollectionKind`, element and `Some((1, 4))`. Fields declared `Text`, `Rational`, `Decimal`, a systems interface type, a value type not bound as `Int[lo, hi]`, `Int[0, 170141183460469231731687303715884105728]` and `Int[-170141183460469231731687303715884105729, 0]` (one past each `i128` extreme), and a collection of lower bound 1 with no upper bound are present with `member_type` `None`, and the call returns the other fields (`Ok`, never a refusal). Mutation rows: bounds narrowed through `i64` (the `i128` extremes differ) or `f64` (2^53 + 1 differs); a bound past `i128` saturated, wrapped or truncated into an `IntRange`; `Option` dropped; a `Collection` kind swapped; a `None` field omitted; a `None` field refusing the call. | Test (TC-227) |
| FR-038-AC-139 | PLANNED/UNRUN (IR-1141). UnknownNode is returned for a structural id absent from the admitted graph; NotModelObjectType is returned for admitted scalar, systems-interface or relationship declarations. A selected object-type declaration with a malformed body refuses admission at its independent body rule. | Test (TC-227) |
| FR-038-AC-140 | Independent of any node body (IR-628; implemented, ungated; holds before and after IR-627's stage lands). A crate-internal test over an admitted package (the module's own tests can build a `CheckedPackageV2` from an admitted one) replaces, after admission, the body bounds of the `Int[0, 1000]` node that a read of the field names with `0` and `10`, and separately `0` and `5000`, and the accessor returns `IntRange { lower: 0, upper: 1000 }` for the field, equal to its result before the replacement. Mutation row: an accessor that takes the bounds from the node a read names, or from any node body, returns `10` or `5000`. This row stays catchable once IR-627's stage refuses such a package at admission, because the replacement happens after admission. | Test (TC-227) |
| FR-038-AC-141 | One derivation with admission (IR-628; implemented, ungated). For each field of AC-138 whose type is a range, `Reference`, `Option` or collection, a read naming the structural type node selected from the accessor's returned type admits, and the same read naming a distinct structural type node for the perturbed type refuses `ill_typed`/`operator-ineligible` at the read. The perturbation is: for an `IntRange`, `upper - 1` when `upper > lower`, `upper + 1` when `lower == upper < i128::MAX`, and `lower - 1` when `lower == upper == i128::MAX`; for a collection's bounds, `upper - 1` when `upper > lower`, `upper + 1` when `lower == upper < u64::MAX`, and `lower - 1` when `lower == upper == u64::MAX`; so every perturbed type node stays well-formed (`min <= max`) and is refused at the read as `ill_typed`, never for its own shape; for a `Reference`, the key of another object type's model declaration node; for `Boolean` and `Integer`, the other scalar; for an `Option` or a collection, the same wrapper over the perturbed element. Mutation rows: an accessor that derives types by its own copy of `field_type` which differs in the optional wrapper, a collection's bounds, or a `Reference` target. | Test (TC-227) |
| FR-038-AC-142 | Public shape and stability (IR-628 implemented; new typed errors and declaring-node getter PLANNED/UNRUN, IR-1164/IR-1165). A compile-time API fixture shows `model_object_fields` takes `&self` and one `&CheckedNodeId` and no evidence; an exhaustive match over `CheckedMemberType`, `CheckedCollectionKind` and `CheckedModelFieldsError` with no wildcard arm compiles; code constructing `CheckedModelObjectFields` or `CheckedModelField` by struct literal fails to compile; the criterion is the compile fixtures: the exhaustive matches compile from an external crate (`tests/it`) only while the enums are closed. Mutation rows: `#[non_exhaustive]` added to an enum (the wildcard-free match fails to compile); a public field on either struct (the struct-literal fixture compiles); a variant added to an enum (the exhaustive match fails to compile). | Test (TC-227) |
| FR-038-AC-143 | Pure, total, bounded (IR-628; implemented, ungated). Two calls on one package return equal results; a clone of the package returns an equal result; two admissions of the same bytes under different byte limits (both admitting) are equal as packages and return equal results; a package admitted (under a `work` limit set high enough) from a document with an inheritance chain of 200 object types and 100 redefinitions of one field has a call that returns the fields without panic or stack overflow, and the call does no resolution (a counter on the table-building function of items 2 and 7 reads the same value before and after the call); a node of AC-139's refusal cases returns its error and never panics. Equality of two packages admitted from the same bytes is unchanged by the retained tables. Mutation rows: the accessor panics on an unwrap of a missing owner; it recurses over a 20,000-type chain on a 256 KiB thread stack; it resolves per call; equality of packages gains the retained tables (compared with a crate-internal package holding the same admitted content and cleared tables). | Test (TC-227) |
| FR-038-AC-144 | Resolution is charged once, at admission (IR-628; implemented, ungated). Cost model of items 8 and 9, with the arithmetic. For a chain of N object types, the first declaring one field and each later one declaring one field and extending the previous, the charge is 1 for the first type and k + 1 for the type at depth k >= 2 (one own field, one supertype edge, k - 1 copied entries), in all N(N + 1)/2 + N - 1: 13 for N = 4 (1 + 3 + 4 + 5) and 43 for N = 8. A crate-internal test reads the table-building counter and asserts 13 and 43. For the same chains, the smallest whole-read `work` limit that admits is M; M - 1 returns `incomplete` at the final selected-owner charge (`/semantic_graph/nodes/<i>/owner`), which follows graph validation. Separately, the limit one below the selected chain's last table-building charge returns `incomplete` at `/semantic_graph/nodes/<i>/owner`, preserving the table-before-graph precedence; these are distinct charges at the same pointer. A duplicate-id chain control finds the first budget that reaches its graph-stage duplicate-id refusal and shows that one below it returns work-`incomplete` at that node's `/semantic_graph/nodes/<n>/node_id` graph charge. Under `CheckedPackageReadLimits::bounded()` (work 1,000,000), a chain of N = 1000 (charge 1,000 x 1,001 / 2 + 999 = 501,499) admits and a chain of N = 1500 (1,500 x 1,501 / 2 + 1,499 = 1,127,249) returns `incomplete` at `/semantic_graph/nodes/<i>/owner`. Precedence (item 11), over one package each: a table charge that exhausts the limit together with a graph-stage refusal (a duplicate node id) returns `incomplete`, because tables precede `validate_graph`; a step 1 refusal at row 1 together with a table charge that exhausts the limit at row 0 returns the step 1 refusal, because every row's step 1 precedes any table charge. An object type with an ambiguous name (AC-139, package A) admits, and its table is charged as any other. Read charge (item 10): a read of a field of the N = 4 chain's deepest type charges 4 units (the four entries of its table, no hidden entries), a read of the root charges 1, and a read of an operation charges the unchanged step 3 walk; a frame entry naming a field and an abstraction relation's field entry on that deepest type each charge the same 4 units, a relationship edge read charges the entries of its table, and none runs the ancestor walk for a field (a counter on the walk stays at its pre-resolution value); a state clause resolves only operations and is unchanged. Cycles (items 4 to 6, 9), each pinned against what `resolve` returns today (the expected tables are recorded by running the pre-change `resolve` over each document before the code change and stored as the test's expected values, so they are not asserted from reading the code): a type `A extends A` declaring one field `f` admits, its table is `{f}` and the charge is 2; types `A extends B` with field `a` and `B extends A` with field `b` admit, the tables of `A` and `B` are the one shared table `{a, b}` and the charge is 4; `A extends A` declaring `f` and `g` where `g` redefines `f` admits and its table equals what `resolve` returns for `A` today (by reading `resolve`, `g` hides `f` and, since `A` is its own ancestor, itself, so neither is exposed); a document with a third type `C extends A` (outside the cycle) has a table equal to `resolve`'s for `C`, charged with the entries copied from the cycle's table. Mutation rows: tables computed after admission or per call (the limit never exhausts); the charge omitted for an unread object (the one-below limit admits); tables built after the graph stage (the precedence row returns the graph refusal); a per-type ancestor re-walk (the counter exceeds 13 for N = 4); an ambiguous table refusing admission; a field resolution (read, frame entry, abstraction field entry) that still runs the walk or charges nothing; a cycle refused at admission, built without termination, or built per member (the charge exceeds 4 for the two-cycle); a cyclic table differing from `resolve`'s. | Test (TC-227) |
| FR-038-AC-145 | PLANNED/UNRUN (IR-1141). `List { next?: List; }`, `Tree { kids: Sequence<Tree>[0, 3]; }` and the owner-free self-option group admit with retained producer-assigned structural IDs, a shared non-empty group label and exact in-group references. In the `List` and `Tree` groups, each declared member keeps its own declaration path and anonymous derived members use the sole declared member's path. The owner-free self-option group has no declared member: its anonymous derived path is producer chosen and admits without an invented declaration path. Every member, including declared, option, sequence, bounded-domain and off-cycle members, is checked for closed body, owner and reference resolution. The QSpec TC-946 K14 cycle of declared `Flow`, `joinPred` and `Holder` admits under one label with three distinct declaration paths. A group does not bypass those checks; no group-position ordinal is read or derived. | Test (TC-226) |
| FR-038-AC-146 | PLANNED/UNRUN (IR-1141). Boolean, Integer, integer-range and reference nodes retain their closed-shape checks inside or outside a labelled group. For a node whose closed shape permits `recursion_group`, adding a lone non-empty label to an otherwise valid acyclic node admits: the label constrains actual cycles only (QSpec FR-322; TC-946 K10). A cyclic component whose members do not share one label, or an invalid reference, refuses at its offending member under its independent rule. No stale ordinary key or label hash is compared. | Test (TC-226) |
| FR-038-AC-147 | PLANNED/UNRUN (IR-1141). Every node carrying one shared non-empty group label participates in that group, including an off-cycle `Set<Int[0,1000]>` member. A valid off-cycle member admits with the producer-assigned path appropriate to its declared or anonymous status, independently of the shared label. A missing member named by an exact structural reference, a cyclic component whose members have unequal labels, malformed structural ordinal, duplicate ID, malformed body or unresolved reference refuses under its independent rule at the offending locus. A wrong exact target that names a type-incompatible member refuses under the body/type rule. A lone labelled acyclic node whose closed shape permits `recursion_group` admits because its label imposes no cycle constraint; a model declaration node remains subject to QSpec FR-322-AC-29. The former group-reference position and size refusals are withdrawn in place because no such wire carrier exists. | Test (TC-226) |
| FR-038-AC-148 | Withdrawn by the IR-1141 structural-identity decision under QSpec #517: valid body or bound edits under a retained derived path are not refused by key or group-label recomputation. Independent malformed-body, owner, reference and type checks remain required. This id is not reused. | Withdrawn |
| FR-038-AC-149 | PLANNED/UNRUN (IR-1141). Every group-member visit, reference resolution, body-term visit and owner join is charged before work under the reader limit, including owner-bearing and off-cycle members. An exact limit succeeds and one unit less returns `Incomplete` with work kind, limit, consumed and failing position. No member or group digest computation is charged. | Test (TC-226) |
| FR-038-AC-150 | PLANNED/UNRUN (IR-1141). The reader validates every structural ID for shape and uniqueness, then joins owners and graph references in QSpec FR-322 order. Permuting otherwise valid group members retains their producer-assigned structural IDs and individual paths, shared label and exact in-group and outside references; the complete graph-array value changes, while admission holds. A duplicate ID, invalid shared label or cycle, wrong exact target that is type-incompatible, or unresolved structural reference refuses under its independent graph, label, body/type or reference rule. The former group-position update/refusal branch is withdrawn in place because no group-position carrier exists; no structural-key or label authentication stage is added. | Test (TC-226) |
| FR-038-AC-151 | Implemented by IR-644 and verified by TC-048. Over the QSL-shaped recursive `List` record with an integer field and `next` encoded as `binding(next, aggregate([binding(optional, reference Option<List>)]))`, `structural.eq` over two `List` values admits with `leaves` empty, and the leaf derivation terminates through the option's `inner` edge at the record reentry. Replacing that integer field with a text field whose profile is selected admits exactly its text leaf followed by `["field:next", "inner", "recursion:0"]`; omission of that recursion leaf refuses `invalid_package`/`operation-law-missing` at `operation.leaves`. Adding a healthy text sibling after `next` retains its own leaf after the recursion leaf in declaration order, proving that the wrapper consumes exactly one field edge. A direct field `reference` to `Option<List>` also admits with the same `field:next`, `inner` path. | Test (TC-048) |
| FR-038-AC-152 | PLANNED/UNRUN (IR-1141). Starting from a well-formed List comparison, replace only next by each authored malformed or mistyped aggregate. Flat body grammar admits the mutation, and the operation check refuses ill_typed/operator-ineligible at operation.leaves, without silently omitting next or reading a different member. | Test (TC-048) |
| FR-038-AC-153 | A declared record, tuple and function each carries its source owner in the node and retains the complete package value; QSpec's two-owner packages for `Point` and recursive `List` admit with distinct `Point` ids, distinct `List` group labels and member ids, and equal builtin `Integer` and application-keyed `three` ids under the two owners. An undeclared model declaration and clause function each carries its model owner, while an anonymous type, `source_locus` node and application-keyed node carry no owner. | Test (TC-228) |
| FR-038-AC-154 | Omitting a required node owner, inserting null, using the wrong owner kind, adding version to ModelOwner or placing an owner on an owner-free node refuses malformed_wire at the node that lacks a required owner or at the present owner, before identity validation. No missing owner is reconstructed from occurrences, source map or lock. A shipped complete package value is refused by AC-310 before any obsolete projection-owner comparison. | Test |
| FR-038-AC-155 | PLANNED/UNRUN (IR-1141). After strict wire and graph-shape validation, a `SourceOwner` absent from `lock.sources` refuses `missing_declaration`/`missing-selection` at its node's `node_id`; a source-map region of its `declaration` occurrence naming another source pair refuses `invalid_package`/`invalid-value` at that source-map entry even when both pairs are lock-selected. A `ModelOwner` whose `identity` is unselected, whose `node` names no declaration, or whose `node` names the wrong kind refuses `missing_declaration`/`missing-selection` at its node's `node_id`, except a `model`/`tuple_value_type` or `model`/`variant_type` owner refusal, which is at `/owner`; the refusal occurs even if nothing reaches that node. A relationship cannot back `model`/`object_type`, an object type with `interfaceFeatures` cannot back `model`/`object_type`, an object type without `interfaceFeatures` cannot back `model`/`systems_interface`, and a field cannot back an operation clause. An operation member's clause and an object type's invariant pass. Two owner defects select the least structural node id. An owner-selection defect precedes a dependent dangling `semantic_type` or `dependencies` edge, while an independent graph-shape defect precedes the owner join. Each selected-document lookup charges one validation visit. No content-key stage participates in precedence. | Test (TC-228; FR-322-AC-169, FR-322-AC-170) |
| FR-038-AC-159 | For admitted `quire.op.integer.add`, `quire.op.integer.sub`, `quire.op.integer.mul` and `quire.op.integer.negate` applications whose every operand has finite bounds representable in `i128`, `scalar_application_operands` returns one entry per argument, in wire argument order, with ordinals `0..n-1`, typed identities and exact inclusive `i128` ranges; the unary operation returns one entry and the binary operations return two. Swapping two arguments swaps their entries and ordinals without sorting by child id. | Test (TC-048) |
| FR-038-AC-160 | A reference to a `value`/`parameter` returns `GraphChild` carrying that parameter's node id and its admitted integer-range bounds; a reference to an application subterm returns `GraphChild` carrying that subterm's own node id and result range. Two distinct children of the same integer type remain distinct identities. An admitted graph literal reference carries the literal node id and singleton `(value, value)`, independent of a wider declared type bound. | Test (TC-048) |
| FR-038-AC-161 | For an admitted inline integer literal argument of value `7`, the result carries `InlineLiteral` naming the application node id, the supplied occurrence key and that argument's ordinal, with range `(7, 7)`; another position or occurrence yields a distinct identity. No `CheckedNodeId` is fabricated for the inline term. | Test (TC-048) |
| FR-038-AC-162 | An unknown graph node, a known non-application node, an absent occurrence key, a dangling referenced child and a child without a readable integer range produce the distinct typed errors `UnknownNode`, `NotApplication`, `MissingOccurrence`, `MissingChild` and `MissingRange`, respectively. An admitted reference to unbounded `Integer` returns `UnboundedRange`; an admitted integer-range endpoint, graph literal or inline literal outside `i128` returns `RangeOutOfI128`. Each refusal returns no partial list, narrowed value or panic. The dangling-child row may use a crate-internal mutation after admission because the reader ordinarily rejects it. | Test (TC-048) |
| FR-038-AC-163 | A crate-internal mutation of an admitted application to an operation identity absent from the catalog returns `UnknownOperator`; an admitted application with a catalogued identity outside the four eligible integer operations returns `IneligibleOperator`. The accessor does not interpret a non-integer operation as an eligible scalar parity claim. | Test (TC-048) |
| FR-038-AC-164 | An external Rust API fixture calls the accessor with `&CheckedNodeId` and `&CheckedOccurrence`, exhaustively matches the child-identity and error enums, and reads each ordinal, node id or inline selector and range through typed fields; it does not read `graph().nodes[*].body` or deserialize JSON. Repeated calls and a cloned admitted package return equal results, and neither call changes package equality. | Test (TC-048) |
| FR-038-AC-165 | PLANNED / UNRUN (IR-661). A selected, independently authored FCD-shaped document with a global `relationship` slot identity admits, while an owner-nested relationship, a foreign-package identity and a wrong-slot identity each refuse `invalid_model_binding`/`malformed-declaration`. Owner-nested fields, operations and operation parameters still admit; moving a field or operation into the global relationship slot refuses `invalid_model_binding`/`malformed-declaration`. | Test (TC-048) |
| FR-038-AC-166 | PLANNED / UNRUN (IR-661). Changing only an authored role makes the old name refuse `missing_declaration`/`missing-name` and the new name resolve on the same selected relationship. Destination-type and multiplicity changes are observed through the admitted navigation result nodes of AC-170, not through an unspecified retention accessor. Omitting only `targetEnd.role` admits the document with only its forward named end. Removing `sourceEnd.role`, or setting either present role to null, empty or a non-string, refuses `invalid_model_binding`/`malformed-declaration`; no role is synthesized or checked against an inverse registry. | Test (TC-048) |
| FR-038-AC-167 | PLANNED / UNRUN (IR-661). An unresolved `sourceEnd.type` or `targetEnd.type` refuses `missing_declaration`/`missing-name`; a type naming a relationship or another declaration of the wrong meaning refuses `invalid_model_binding`/`malformed-declaration`, independent of declaration order. A source end naming another owning type, a malformed end or multiplicity refuses `invalid_model_binding`/`malformed-declaration`; `lower > upper` alone refuses `invalid_model_binding`/`unpreserved-model-meaning`. Each declaration refusal keeps its typed code, cause and root-owner pointer. This criterion verifies refusal semantics only; private metadata retention is verified separately by the dedicated criteria under the declaration-refusal retention contract. | Test (TC-048) |
| FR-038-AC-168 | PLANNED / UNRUN (IR-661). Repeating a relationship identity, including under two different owners, refuses `invalid_model_binding`/`conflicting-binding` at the root's `owner`. Two missing relationship identities are malformed declarations, not conflicting identities. A malformed role beats an unresolved end type and reversed multiplicity on that relationship; an unresolved end type beats reversed multiplicity. Referencing a node refused for identity or kind keeps its own refusal rather than reporting a missing end type. With other limits sufficient, the measured successful selection-stage work admits that stage; one below its cumulative charge threshold returns `incomplete` at `/semantic_graph/nodes/<i>/owner` with no admitted package. This is separate from AC-171's total-reader work limit. | Test (TC-048) |
| FR-038-AC-169 | PLANNED / UNRUN (IR-661). `relationship_end.declaration` names the selected relationship's `relation`/`relationship` graph node and resolves through its content-only `ModelOwner`, with an empty body; a valid source or target object node used as its declaration refuses `ill_typed`/`operator-ineligible` at `member.declaration` after owner recovery and before role lookup, without invalidating that object owner. Source role lookup yields the forward end and a present target role yields the inverse end, by exact role bytes rather than the terminal identity segment. An unknown role or absent inverse refuses `missing_declaration`/`missing-name` at the member's `name` before receiver, direction or result checks; two matching roles refuse `ambiguous_declaration`/`ambiguous-name`. A wrong-kind or missing relationship owner keeps the existing owner-join refusal, with no role fallback. | Test (TC-048) |
| FR-038-AC-170 | PLANNED / UNRUN (IR-661). Forward and inverse navigation choose opposite receiver/destination endpoints and admit the canonical result nodes specified by QSpec FR-152 Navigation. The independently authored finite cases in TC-048 exercise Reference, Option, Set and Bag results; changing only destination type or bounds changes which result node admits. An inherited end admits on a subtype receiver with the same destination type; an unrelated object receiver refuses `ill_typed`/`operator-ineligible`. Wrong result or direction refuses `ill_typed`/`operator-ineligible` at the declared IR path. An eligible direction with an unbounded or ordered destination refuses `unsupported_construct`/`expression-form` at `member.name`, using the relationship-navigation-only variants named in AC-101. A named inverse on a source-to-target relationship refuses direction before unsupported destination multiplicity. | Test (TC-048) |
| FR-038-AC-171 | PLANNED / UNRUN (IR-661). For `quire.op.model.navigate`, the new relationship-end checks follow the specified operation-step order, explicit calling-node locus and RFC 6901 paths: owner recovery and declaration-kind eligibility before roles; roles before receiver, direction, multiplicity and result comparison. A missing role combined with a wrong receiver, disallowed direction and wrong result reports `missing_declaration`/`missing-name` at `.../operation/member/name`; a valid role with wrong receiver reports at `.../body/arguments/0`; a mismatched result reports at `.../body/result_type`. A successful complete read admits at its measured exact total work limit. One less returns `incomplete` at the first unpayable charge in the existing stage order, without a partial package: operation validation precedes later graph-body reference-edge charges, so the pointer may be `.../body/arguments/0/target`, not the selection row. Selection-stage-minus-one has the separate selection-row oracle of AC-168. | Test (TC-048) |
| FR-038-AC-172 | PLANNED / UNRUN (IR-661). A homogeneous self-relationship with a resolved forward or inverse end and an eligible Reference or Option destination admits `quire.op.model.reaches` with two references conforming to its static endpoint owner and Boolean result, including subtype operands. An unrelated operand, heterogeneous destination or otherwise ineligible Set/Bag edge refuses `ill_typed`/`operator-ineligible` at its operand or member-name path. Missing role and object-node declaration produce AC-169's respective typed outcomes; a wrong result node refuses at `body.result_type`. No operand is treated as Reference to the relationship graph node. FR-038-AC-252 supersedes the otherwise-ineligible Set/Bag clause for `quire.op.model.reaches`. | Test (TC-048) |
| FR-038-AC-173 | PLANNED / UNRUN (IR-661). Removing direction, category, composite or origin, setting each to null or a wrong type, using an unsupported direction/category value, or supplying a malformed common-schema origin branch refuses `invalid_model_binding`/`malformed-declaration` at the selection row before application resolution. Code/cause/pointer and check order remain unchanged. Schema-valid source and generated origins admit without a default direction. This criterion verifies origin validation and admission/refusal semantics only; private metadata retention is verified separately by the dedicated criteria under the declaration-refusal retention contract. | Test (TC-048) |
| FR-038-AC-174 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 identity retention). When FR-154 intake selects an offending declaration with a supplied string identity, the reader retains that exact FCD identity in the private refusal returned by `admit_selection`, including refused object-id spelling. Missing or non-string identity yields None; no graph node id, inferred owner, repaired string or separate artifact id supplies it. | Test (TC-048) |
| FR-038-AC-175 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 private constructors). Each private `SelectionRefusal` constructor and declaration-to-`SelectionFailure::Refused` conversion preserves the selected retained metadata record through the private `admit_selection` return after document release. Independently dropping available identity/origin or substituting a different declaration's record fails the typed return assertion. Pre-declaration constructors preserve genuine metadata absence. | Test (TC-048) |
| FR-038-AC-176 | PLANNED/UNRUN (IR-1141). make conformance-qspec reads every positive-*.json package and its required selected domain documents directly from QUIRE_SPECIFICATION_DIR; each admits through the production reader under its published name and retains the exact structural node ids, owners, graph order and complete bodies. Missing or malformed source, unknown selected identity, unadmitted package or unequal retained body fails. No source fixture or vector is copied into IR. | Test (TC-048) |
| FR-038-AC-177 | Implemented by `composite_application_operands`; verified by the TC-048 composite operand tests. An admitted structural.eq over supported parameter references returns the actual application and supplied occurrence with two typed operands in argument order, ordinals 0 and 1. Swapping arguments swaps entries; repeating a parameter retains two entries. A second authentic occurrence is retained exactly; an absent occurrence refuses MissingOccurrence with the supplied application and occurrence. | Test (TC-048) |
| FR-038-AC-178 | Implemented by `composite_application_operands`; verified by the TC-048 composite operand tests. A supported closed graph composite literal retains its own GraphChild id and Literal disposition with no introduced positions even under a wider type. A value subtree reading a parameter refuses UnsupportedOperand with enclosing ordinal and reason NonliteralGraphValue; an application subterm refuses reason ApplicationSubterm. Defensive inline integer and noninteger term cases refuse reasons InlineInteger and InlineNonInteger, without fabricated identity or value. No publicly admitted structural.eq or structural.ne integer-inline success is claimed: both owning catalog entries exclude Integer from structural_kind; future success requires an owning eligibility/value specification. | Test (TC-048) |
| FR-038-AC-179 | Implemented by `composite_application_operands`; verified by the TC-048 composite operand tests. A record containing bounded Sequence of integer-range elements returns its exact field/collection/element paths, Node keys rooted at the operand, and both collection endpoints. Changing only one path's source type changes that descriptor. Bounds beyond i128/u64 retain canonical decimal text; unbounded integer/collection have explicit unbounded descriptors without guessed maxima. | Test (TC-048) |
| FR-038-AC-180 | Implementation exists in `composite_application_operands`; the corrected 18-form code evidence is PLANNED/UNRUN (IR-491), and the prior 18-form public-admission claim does not discharge it. Tuple, direct Option and optional-presence record edges preserve the specified ordinals without wrapper segments; List with optional tail has Depth key [] and reentry [1,0]. Tree with two optional self-fields retains both [0,0] and [1,0] at Depth key []; shared nonrecursive siblings retain distinct paths without Depth. Boolean introduces no position. Enum retains ordered flag and exact preimage member order, including unordered Blue, Green, Red. Each of the eleven scalar and seven bounded-domain forms has its specified type-walk descriptor or absence, including one Whole at a `text_bounds` wrapper and a Node-keyed Whole without Population key for population-typed leaves. A public reader-admitted comparison over each of the fifteen non-IEEE forms returns its independently enumerated introduced-key set; isolated private type-walk unit controls over `float32`, `float64` and `float_rounding` return their tabled descriptors. Public structural comparisons reaching any of those three forms instead refuse `ill_typed`/`operator-ineligible` at `operation.leaves` under AC-231; no public accessor result is claimed for them. | Test (TC-048) |
| FR-038-AC-181 | Implemented by `composite_application_operands`; verified by the TC-048 composite operand tests. The public IneligibleOperator assertion uses a reader-admitted `boolean.and` application outside eq/ne (AC-199). Every one of the twelve specified error variants exposes exactly its tabled payload and no partial result or panic. Unsupported union, malformed optional wrapper and unanchored cycle carry the actual enclosing ordinal/type locus. Zero and one below measured successful work refuse WorkLimit; that exact budget succeeds. Checked-index failure returns PositionOutOfRange with authentic available ordinal/type metadata. Equivalent package node order and lookup strategy leave logical consumed_work unchanged; an earlier operand refusal or exhausted charge beats a later defect. | Test (TC-048) |
| FR-038-AC-182 | Implemented by `composite_application_operands`; verified by the TC-048 composite operand tests. An external consumer calls the named public accessor and exhaustively matches all named output, descriptor, edge, key and error types using the specified fields without body JSON or a QSL model dependency. Repeated/clone calls yield equal content and unchanged package equality. API inspection finds no harness-bound inputs, coverage verdict or obligation encoder; structural.ne eligibility is verified separately by AC-197 through AC-201. | Test, Inspection (TC-048) |
| FR-038-AC-183 | Withdrawn by the IR-1141 structural-identity decision under QSpec #517: the reader computes no expected derived key for an unreferenced node and exposes no `expected_node_id()` result. Independent malformed-body, duplicate-id and reference refusals remain required. This id is not reused. | Withdrawn |
| FR-038-AC-184 | Withdrawn by the IR-1141 structural-identity decision under QSpec #517: the reader computes no expected application key and has no stale-key refusal for a valid body edit under a retained path. Independent operation, body and graph checks remain required. This id is not reused. | Withdrawn |
| FR-038-AC-185 | PLANNED/UNRUN (IR-1141). `CheckedPackageRefusal` retains its typed code, cause, pointer and authentic structural node locus where applicable, but exposes no `expected_node_id()` computation from a content preimage. Every refusal constructor and public forwarder preserves those fields. A malformed wire or pre-graph refusal invents no node locus; Display and serialized forms disclose no fabricated id. | Test, Inspection (TC-048) |
| FR-038-AC-186 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 source origin). Given a located refusal with a valid FCD Source origin, the reader returns its exact typed source member values and optional-member presence at the private intake boundary, including all four end-coordinate presence combinations and the existing admitted 2^53 boundary. Retention adds no paired-end or span-order constraint. | Test (TC-048) |
| FR-038-AC-187 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 generated origin). Given a located refusal with a valid FCD Generated origin, the reader returns exact generator identity/version text and input identity sequence at the private intake boundary. Input order/repetition remains unchanged and no source span is present. | Test (TC-048) |
| FR-038-AC-188 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 selected nested context). When an existing field declaration/type reference, operation declaration/return type, parameter declaration/type reference or relationship-end check refuses, the reader retains the actual offending field, operation, parameter or relationship's supplied identity and valid origin. Missing nested metadata remains None; an enclosing owner or unresolved target cannot substitute. | Test (TC-048) |
| FR-038-AC-189 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 origin absence). Given a located refusal with missing or malformed origin, the reader returns None for origin while independently retaining any available declaration identity. Both-branch, partial-branch, unknown-member and invalid-member cases cannot yield a salvaged branch or default origin. | Test (TC-048) |
| FR-038-AC-190 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 same-identity type-group refinement). For every same-identity type group as defined above, including malformed-declaration and conflicting-binding refusals, the reader retains its common supplied identity and retains origin only if every candidate has a valid origin and all typed values and optional-member presence agree. Any missing, malformed or different valid candidate origin yields None. Candidate permutation cannot select a winner. This IR-local ambiguity refinement does not apply type-group consensus to a sequential relationship refusal. | Test (TC-048) |
| FR-038-AC-191 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 refusal precedence). Retaining metadata does not change the existing first code, cause or selection-row member in node/table/member-path order. Combining declaration defects preserves that exact first refusal; distinct Limit and InexactNumber alternatives retain their current payloads and precedence. | Test (TC-048) |
| FR-038-AC-192 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 bounded retention). Retained metadata is acquired and released within the existing bounded document/read/work accounting, without a second parse, raised ceiling or unbounded diagnostic collection. Exact sufficient work succeeds; the first unpayable charge preserves the existing incomplete result and no partial package. | Test (TC-048) |
| FR-038-AC-193 | IMPLEMENTED; source Inspection completed (IR-663 public boundary exclusion). Inspection of the IR-owned model-intake conversion confirms that it projects code, primary location path and cause to ValidationFailure's code/path/cause and adds no declaration metadata field to CheckedPackageRefusal or the IR-owned dispatch result. Unrelated graph-node locus, contract-version and document-pointer behavior remains intact. | Inspection (TC-048) |
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
| FR-038-AC-210 | Implemented (IR-484). For a valid self-typed declared `unit` node, references to distinct values typed by that node resolve to `quantity`: all twelve catalogued `quire.op.quantity.*` entries resolve each required quantity position and reject a Boolean at the first one, independent of their later result, member and dimension obligations; `quantity.add` and `quantity.eq` with valid same-unit operands also admit end to end. Replacing a required quantity argument by Boolean refuses `ill_typed`/`operator-ineligible` at that argument; placing a declared-unit quantity in the first `integer.add` position also refuses at `arguments/0`, so a missing quantity family cannot turn the rejection into a skip. `quire.op.quantity.pow` refuses a quantity rather than an integer exponent at `arguments/1`. The operation identity and package name remain selected after every mutation, and the positive controls admit through the public reader. | Test (TC-048) |
| FR-038-AC-211 | Implemented (IR-484). `quire.op.control.if` over a Boolean condition and two values of the same declared-unit quantity type admits, while replacing the second branch with an Integer value refuses `ill_typed`/`operator-ineligible` at `arguments/2` under its `same_family` constraint. A valid `aggregate` in each of the three aggregate positions of `quire.op.temporal.clause` and in the first position of `quire.op.state.clause` fits `aggregate` at the operation step after earlier shape checks. Placing an aggregate in the first `boolean.and` argument refuses `ill_typed`/`operator-ineligible` at `arguments/0`, so a missing aggregate family cannot make the check pass. Replacing the first `state.clause` aggregate by a Boolean reference instead refuses `invalid_semantic_graph` at that node's body during the earlier structural stage, never as an operation-family refusal. Each control supplies the other catalogued arguments, law and member and has a freshly derived identity. | Test (TC-048) |
| FR-038-AC-212 | Implemented (IR-484). A body-root or argument literal whose `value_kind` is `integer` and whose `type` names an Integer node admits when the enclosing operation is otherwise valid; retaining that value kind while changing only its declared type to a Boolean node and re-deriving identities refuses `ill_typed`/`operator-ineligible` at the literal's `type`, even if the enclosing operand position accepts either family. An enum literal accepts either `enum` or `ordered_enum` type according to its nominal declaration, and a `none` literal accepts an option type; each mismatched kind/type pair, including an integer literal declared at a unit type, refuses at its own `type`. | Test (TC-048) |
| FR-038-AC-213 | Implemented (IR-484). `quire.op.quantity.convert` over a declared-unit quantity and a selected quantity `type_argument` admits mode `{kind: rounding, value: exact}`; `toward-zero` on otherwise identical operands and result refuses `invalid_package`/`operation-mode-type-mismatch` at `operation.mode/value`, even though the unit type has no explicit rounding binding. The result type alone pins this conversion to the declared-unit default `exact`. The same mode check applies to `quire.op.collection.sum.quantity` when its quantity binder and member are valid. | Test (TC-048) |
| FR-038-AC-222 | Implemented (IR-742). Through `CheckedPackageV2::read`, a `float32_bits` literal declared at a Float32 type and a `float64_bits` literal declared at a Float64 type each admit in otherwise valid packages. Retaining each value kind and changing only its declared `type` to the opposite IEEE width, while retaining the structural node id and package name, refuses `ill_typed`/`operator-ineligible` at that literal's `type`, including in an operand position whose catalogued family would otherwise accept the declared type. Neither mutation is admitted or refused merely at an enclosing operand. This extends FR-038-AC-212's general literal-family rule without changing its integer, enum or option cases. | Test (TC-048) |
| FR-038-AC-223 | Implemented (IR-742). Through `CheckedPackageV2::read`, valid `quire.op.ieee.to_float32` over Float64 and `quire.op.ieee.to_float64` over Float32 admit when each `type_argument` and `result_type` name the same correctly typed destination node. For each identity, changing `result_type` and the enclosing application's root-node `semantic_type` together to the source-width node while retaining the valid destination `type_argument`, and re-deriving all affected identities, refuses `ill_typed`/`operator-ineligible` at the application's `result_type` by the `member` result form; changing `type_argument`, `result_type` and that root-node `semantic_type` together to the source-width node likewise refuses there by the catalogued `member_family` constraint. In both cases the root `result_type` equals its node's `semantic_type`, so that separate root mismatch rule cannot decide the result. Each adverse package supplies an otherwise valid source operand and law, and is not allowed to reach a same-width descriptor through the public reader. | Test (TC-048) |
| FR-038-AC-247 | Implemented (IR-225). Through `CheckedPackageV2::read`, a catalogued `result: member`/`member: type_argument` application whose member has a valid `declaration` admits only when `result_type` equals that declaration, regardless of whether its family resolves or the entry has a `member_family` constraint. A mismatch refuses `ill_typed`/`operator-ineligible` at `/semantic_graph/nodes/{i}/body/result_type`, with the application node as locus. `collection.fold` and `collection.reduce` admit equal results without a family constraint and refuse unequal results. `model.lookup` and `model.all_instances` have other result forms and remain admitted. `quire.op.ieee.to_rational` over each of Float32 and Float64 admits with rational declaration and rational result; float/rational and rational/float declaration/result pairs refuse at that pointer, with root `semantic_type` matched to `result_type` and all affected identities re-derived. AC-250 also requires equality refusals for two distinct non-IEEE catalogue entries, `quire.op.numeric.narrow` and `quire.op.quantity.convert`. After earlier member-shape and operand checks, this result refusal precedes `result_inner`, state-clause result, model-member, mode-type and leaf checks; a simultaneous operand defect wins over a result defect, while a simultaneous genuine rounding-mode type defect on `quantity.convert` does not displace the `result_type` refusal. | Test (TC-048) |
| FR-038-AC-248 | Implemented (IR-225). For a catalogued `type_argument` member result with `member_family`, a declared type whose family the reader resolves under the FR-322 ruling must fit the catalogue family; a resolved mismatch refuses `ill_typed`/`operator-ineligible` at `/semantic_graph/nodes/{i}/body/result_type`, with the application node as locus. `quire.op.ieee.to_rational` over both Float32 and Float64 refuses a same-width float declaration and float result, even though equality holds. A valid `quire.op.quantity.convert` into a `compound_unit` with equal member declaration and result admits: its unresolved family does not cause a refusal under the FR-038 deferral of `compound_unit`, `model_population` and `alias` family rulings to FR-322. An otherwise valid `quire.op.collection.size` whose `type_argument` declaration is a `model_population` node and whose `result_type` equals that declaration admits: this result check skips the unresolved family even though the generic bounded-domain walk reaches the `Set<Reference<T>>` base and reports `set`, which does not fit the entry's `integer` member-family constraint. | Test (TC-048) |
| FR-038-AC-249 | Implemented (IR-225). A `type_argument` member missing `declaration`, carrying a malformed declaration, or carrying `type_argument` instead of `declaration` refuses `invalid_package`/`operation-member-mismatch` at `/semantic_graph/nodes/{i}/body/operation/member`, with the application node as locus. The code PR changes the eleven existing IR fixtures using `member.type_argument` to the closed `member.declaration` spelling; no alternate read is admitted. | Test (TC-048) |
| FR-038-AC-250 | Implemented (IR-225). Through the public reader, valid `quire.op.numeric.narrow` and `quire.op.quantity.convert` applications each admit with equal member declaration and `result_type`, and each refuses a mismatched `result_type` as `ill_typed`/`operator-ineligible` at `/semantic_graph/nodes/{i}/body/result_type` with the application node as locus, after re-deriving affected identities. These two non-IEEE catalogue entries demonstrate entry-driven selection beyond the IEEE conversion identities. The `field`, `position`, `element`, `relationship_end` and `operation` member-result kinds do not acquire the type-argument equality rule; in particular, valid `tuple.project` and `collection.index` packages with member declarations distinct from their `result_type` remain admitted. | Test (TC-048) |
| FR-038-AC-251 | Implemented (IR-225; amended from an expected-set shape guard because the reader selects from the current catalogue entry). Member-result checking is selected by the catalogue entry shape alone (`result: member` and `member: type_argument`). Every other catalogued operation, including `model.lookup`, `model.all_instances` and the other member kinds, keeps its pre-IR-225 admission outcome; `collection.fold` and `collection.reduce`, which have no `member_family`, are checked for result-type/declaration equality only. A catalogue-wide differential test reads every entry from the same published catalogue as the reader, requires admission for each constructible control, and names the exact unavailable fixture set. | Test (TC-048) |
| FR-038-AC-214 | CODE-GATED (IR-575; isolated 2026-10-10 measurement passed, final repo gates pending). The production `CheckedPackageV2::read` path directly decodes the closed wire into the typed package without ever constructing a full-document `serde_json::Value` tree or allocating a second full-document canonical byte buffer; `Value` terms inside the typed wire remain permitted. Source inspection and the real-reader adverse corpus confirm unchanged duplicate-member, noncanonical-byte, unknown-member and first-refusal code/pointer outcomes, unchanged identity and source-map content, and the same public typed result. In three fresh Linux x86_64 reader processes built with the repository's debug `cargo test --locked --test` profile and Rust's default System allocator, using this repository's canonical 100000-reference-chain fixture prebuilt by a separate generator process, `CheckedPackageV2::read` admits the full package under nonbinding limits and uses at most 1.50 GiB (1572864 KiB) additional peak resident memory during the call. Each reader loads input bytes and prepares evidence before recording baseline VmRSS and VmHWM. A run whose pre-read VmHWM differs from pre-read VmRSS is invalid setup and must be recorded and retried in a fresh process, never counted as passing. For each valid run, sample VmRSS during `read`, record VmHWM immediately after return while retaining the admitted package, and compute additional peak as `max(sampled peak VmRSS, post-read VmHWM) - pre-read VmRSS`. Fixture generation, lowering and runtime startup lie outside the interval. Every valid run records input size, admitted node count, baseline/peak/post RSS and HWM, elapsed read time, compiler version, allocator and platform; an over-ceiling run fails. The existing AC-117 admission and stack obligations remain. | Test, Inspection (TC-048) |
| FR-038-AC-215 | Withdrawn: a selected domain package document has no digest to recompute, compare or refuse as `content-mismatch`. | Test (TC-048) |
| FR-038-AC-216 | Withdrawn: a selected domain package document has no digest, so there is no `byte-digest-mismatch`; bytes that do not read refuse `malformed_wire`/`malformed-json` (FR-038-AC-27). | Test (TC-048) |
| FR-038-AC-217 | A selected dependency with no package supplied refuses `missing_import`/`missing-selection` at its lock entry. A supplied package whose `name` matches the selected identity admits if otherwise valid. Two unequal packages supplied under one identity through a diamond refuse `invalid_package`/`conflicting-definition` with both dependency paths; equal complete JSON packages admit. | Test (TC-048) |
| FR-038-AC-218 | `CheckedPackageRefusal` exposes no expected/actual digest pair. Each missing selection, conflicting definition, malformed wire, noncanonical wire, model binding and graph failure retains its typed code, cause and exact pointer; no `stale_dependency`, `content-mismatch` or `digest_domain_mismatch` code/cause is emitted. | Test (TC-048) |
| FR-038-AC-224 | PLANNED/UNRUN (IR-490). A private operation-stage meter probe over an alias or non-population bounded-domain chain of N distinct forwarding nodes charges exactly N structural-resolution units on its first successful lookup, zero on another lookup of the same head or an already resolved suffix within that application's operation validation, and N again for the same chain in a second application. The terminal type adds zero forwarding units; the first lookup's exact limit succeeds and a limit one below returns typed Work before the failed forwarding node's `semantic_type` is read. For a 300-field record naming different entries of one 300-alias chain, measure total operation work W under a nonbinding limit: W reaches the ordinary `operation-law-missing` decision and W-1 returns typed Work when the required leaves are missing; this case measures the count pass only. In a separate version with every derived leaf and its selected law supplied, both leaf passes execute and the application admits at its measured total work limit W; W-1 returns typed Work. Across those two passes each of the 300 structural forwarding nodes charges once, while leaf-visit and `text_profile` pin charges remain independent and repeated pin lookups retain their separate first-visit charge rule. For each of `union_arms`, `inner:0`, `result_inner`, and a leaf edge, an otherwise valid operation whose remaining work limit is exhausted by structural forwarding returns the reader's `incomplete`/`work` at that operation, with no `ill_typed`/`operator-ineligible`, skipped constraint, or unchecked leaf. | Test (TC-048) |
| FR-038-AC-225 | The production reader admits the authoritative QSpec `positive-union-nodes.json` with both payload-bearing and nullary union-value bodies. It preserves each structural `{package,path,ordinal}` id and exact aggregate body. Ordinary applications, application-free record values and tuple values remain distinct forms; a private helper result alone cannot pass this admission control. | Test (TC-048) |
| FR-038-AC-226 | For each authentic QSpec union value, a duplicate structural node id refuses `invalid_semantic_graph` at the later node, an unresolved body reference refuses at its reference, and an invalid union case or payload refuses at the union body. The untampered package admits. No stale-key or expected-key accessor is involved. | Test (TC-048) |
| FR-038-AC-227 | A reader-admissible recursion group containing a union value and a nonapplication node retains each member's producer-assigned structural ID and shared non-empty label, and resolves in-group and outside references by exact structural target IDs. A duplicate member ID, malformed structural ordinal, invalid cycle label or unresolved reference refuses under its typed graph or wire rule before union-body interpretation. The former group-position refusal branch is withdrawn in place. Reader work and whole-package byte limits remain in force. | Test (TC-048; QSpec TC-940, FR-322-AC-144, FR-322-AC-272 through FR-322-AC-274) |
| FR-038-AC-228 | PLANNED/UNRUN (IR-491). Through `CheckedPackageV2::read`, otherwise valid `structural.eq` and `structural.ne` packages whose first operand is, separately, a reference to a structural value, a reference directly to its type node, a reference to an expression node with an application at its body root whose `result_type` equals that node's `semantic_type`, or an option-typed `none` literal, each derive the leaves of the selected declared type. An all-integer record admits with `leaves: []`, and a record with one pinned text field admits with exactly that field's selected-law and mode leaf, while removing the required leaf refuses `invalid_package`/`operation-law-missing` at `operation.leaves`. Replacing the first operand's reference with an inline `application` refuses `malformed_wire` at `arguments/0` before leaf checking. Each package's operand and result identities are independently re-derived; changing a declared type does not silently retain the original comparison. | Test (TC-048) |
| FR-038-AC-229 | PLANNED/UNRUN (IR-491). For `structural.eq`, `structural.ne` and `collection.contains`, a first operand replaced with a `binding`, `aggregate`, `frame` or `abstraction_relation` never admits by skipping type resolution: after any earlier applicable grammar/reference check, it refuses `ill_typed`/`operator-ineligible` at `arguments/0` if it reaches the operand-family check. A `dependency_reference` in that position is refused by the earlier dependency-reference placement check. Separately, a first operand whose family fits but whose required compared or `inner:0` type remains unresolved after earlier checks refuses `ill_typed`/`operator-ineligible` at `operation.leaves`, with `leaves` empty or nonempty; exhausted structural-resolution work remains `incomplete`/`work`. | Test (TC-048) |
| FR-038-AC-230 | PLANNED/UNRUN (IR-491). For every catalogued `result_inner` identity (`collection.set`, `.bag`, `.ordered_set`, `.map`, `.flat_map`, `.flatten`, `.convert`), a result of `set`, `bag` or `ordered_set` over pinned text admits exactly the selected element-type leaf at the empty path and refuses an omitted one as `operation-law-missing` at `operation.leaves`; where the catalogued result form permits a valid `sequence` result over that text, it admits with `leaves: []` and refuses one supplied leaf as `operation-law-mismatch` at `operation.leaves/0`. A set-like result whose required element type remains unresolved after earlier checks refuses `ill_typed`/`operator-ineligible` at `operation.leaves` rather than admitting either leaf list; exhausted structural-resolution work remains typed Work. Each positive control admits through the public reader with current catalog and lock selections. | Test (TC-048) |
| FR-038-AC-231 | PLANNED/UNRUN (IR-491). An otherwise valid `structural.eq` or `structural.ne` over a record, tuple, option or sequence whose selected compared type reaches Float32 or Float64 at any depth, and `collection.contains` over a sequence whose element type does, each refuse `ill_typed`/`operator-ineligible` at `operation.leaves` with `leaves: []` and with a supplied text leaf; changing only that nested IEEE type to an integer type and re-deriving identities admits with its exact empty leaf list. A valid `collection.flatten` result of `sequence<Float32>` or `sequence<Float64>` admits with no leaves because `result_inner` selects no compared type for a sequence result. A `set`, `bag` or `ordered_set` of an IEEE-bearing element is not a positive control: its element type is ineligible under QSpec FR-144. Each adverse comparison is first independently shown to pass strict wire, graph and identity admission without the comparison, so an earlier malformed package cannot satisfy the refusal. | Test (TC-048) |
| FR-038-AC-232 | PLANNED/UNRUN (IR-691). A fresh QSL FR-093 checked and emitted `record Point { x: Int[0,9]; }` value `Point { x: 3 }` is admitted by the public V2 reader; its `record_value` binding references a `numeric.narrow` conversion whose sole argument references the canonical Integer literal `3`, and whose member declaration, result type and semantic type name the same `Int[0,9]` range. In an admitted structural.eq or structural.ne over that value, `composite_application_operands` returns the original record GraphChild and semantic type with Literal domain and no introduced positions. At the private post-admission seam, an equivalent graph with canonical integer literal and range endpoints beyond i128 has the same result; no copied or narrowed value is returned. | Test (TC-048) |
| FR-038-AC-233 | PLANNED/UNRUN (IR-691). In crate-internal admitted-then-mutated controls, altering only the narrow node's operator, laws, leaves, mode, member kind/declaration, result type, semantic type or argument count/term; the range's presence, Integer base, endpoint binding/literal shape, endpoint literal type, endpoint grammar/order; or the operand literal's value kind, declared/semantic type or canonical spelling returns `MalformedDomain` with the enclosing ordinal and first valid type locus specified above. For both positive and negative ranges beyond i128, a canonical literal one below min or one above max returns `MalformedDomain` at the actual range node, with no rounded, clamped or partial Literal result; min and max themselves succeed. The negative controls use explicitly authored signed decimal strings on both sides of each bound, so a positive-only or lexicographic comparator fails. | Test (TC-048) |
| FR-038-AC-234 | PLANNED/UNRUN (IR-691). A narrow whose argument reaches a parameter, another application including a second narrow, or a value cycle returns `UnsupportedOperand`/`NonliteralGraphValue` at the first offending node with the enclosing ordinal; an unrelated application remains refused. A direct parameter operand retains its Parameter projection. Existing closed tuple, Option, collection and enum-value descendants remain accepted by their value-form paths, and a narrow integer field inside a supported composite receives only the stated narrow exception. Reader admission and first-refusal outcomes for the unchanged packages are identical before and after the accessor change. | Test (TC-048) |
| FR-038-AC-235 | PLANNED/UNRUN (IR-691). Independently count the exact logical graph, term/member, examined-endpoint-byte and examined-literal-byte charges for a small admitted narrow-record fixture. Its measured successful work limit succeeds; one less and zero return `WorkLimit` with the first attempted cumulative charge and no partial operands. At the private seam, a range with long canonical endpoints charges every endpoint byte once per logical visit before comparison even when its first sign or digit decides the order: the independently counted exact limit succeeds, and one less returns `WorkLimit`. A budget exhausted immediately before a malformed wrapper, out-of-range literal or later operand defect returns `WorkLimit` first. Reordering admitted graph nodes leaves the result and consumed work equal. No public accessor type, scalar eligibility, reader admission rule, second domain owner or fixed-width decimal conversion is introduced. | Test (TC-048), Inspection |
| FR-038-AC-236 | IMPLEMENTED (IR-777). A selected Semantic IR model declaration of a nonempty variant type, with a self-typed declaration node and exact model owner, is admitted through `CheckedPackageV2::read`. Its declared type is accepted as a field, parameter and result type, including under an optional or bounded sequence field. Changing the owner to an absent or different declaration refuses at the declaration node's `/semantic_graph/nodes/<i>/owner` with `missing_declaration`/`missing-selection`. | Test (TC-048) |
| FR-038-AC-237 | IMPLEMENTED (IR-777). A value typed by the selected variant accepts an `enum_value` whose `case` is an exact selected UTF-8 case name, including a name with a space, digit prefix or non-ASCII character. Distinct case names or variant declarations have distinct authored structural value paths, and `enum.eq` and `enum.ne` admit otherwise valid operands of the same variant type. `enum.lt` refuses that unordered variant at the first ineligible argument; `structural.eq` over two such values refuses `ill_typed`/`operator-ineligible` at the application node (`/semantic_graph/nodes/36` in the published TW-24 vector). | Test (TC-048) |
| FR-038-AC-238 | An empty selected case declaration refuses `invalid_model_binding`/`malformed-declaration` at the selected model row; an `enum_value` naming an absent case refuses `invalid_semantic_graph` at the value node. A supplied document with a different case set is read and checked under the same selected identity; ordinary scalar enum values keep their identifier-only case rule. | Test (TC-048) |
| FR-038-AC-239 | IMPLEMENTED (IR-782; reverified IR-1025). A package with a `protocol`/`protocol_reference` carrier whose closed reference body targets an exact local `protocol` node, with one `expression` occurrence, the same semantic type and exactly that target as its dependency, admits through the public V2 reader as `ProtocolForm::ProtocolReference`. Both a `protocol_clause` target and a different protocol form (`role` or `control`) admit; a reader that accepts only `protocol_clause` fails the second control. For each admitted target, the retained structural id and complete package value are compared with the authored fixture; changing the target changes the complete body and is checked by the target join. The admitted package retains one graph node collection and its source correspondence. | Test (TC-048) |
| FR-038-AC-240 | An extra dependency or one unequal to a protocol-reference carrier body target refuses `invalid_semantic_graph` at its `dependencies`. With body target and sole dependency changed together, an absent local target refuses `missing_declaration`/`missing-name` at `/body/target`, even if a selected dependency has that id. A non-protocol target or unequal semantic type refuses `ill_typed`/`operator-ineligible` there. Carrier target defects precede frame and operation defects. | Test (TC-048) |
| FR-038-AC-241 | IMPLEMENTED (IR-1025; QSpec FR-464-AC-15). A carrier with a non-reference or `dependency_reference` body or an extra body member fails the closed V2 schema. A carrier with a declaration or owner refuses at its offending member before target resolution. A carrier with a `recursion_group` outside a cycle refuses `malformed_wire` during graph validation; a cyclic component containing a carrier uses one shared non-empty label and exact structural IDs/references for every member, including non-protocol nodes. The predicate-join function's reference to its own protocol includes any carrier and non-protocol nodes in that component. Multiple valid `expression` occurrences are admitted as specified by FR-038-AC-294; they are not a forbidden carrier member. An ordinary node holding a generic reference remains an ordinary node; no second protocol-reference form or separate collection is admitted. | Test (TC-048, TC-946) |
| FR-038-AC-294 | A public-reader package with one local protocol target and one carrier admits after adding second and third `expression` occurrences with distinct ordinals and exact source-map rows. The one carrier keeps its structural node id and body `{term: "reference", target}`; its occurrences and source regions remain distinct. No identity discriminator or fixed two-occurrence maximum is added. | Test |
| FR-038-AC-295 | From that admitting repeated-reference package, a duplicate occurrence ordinal or source-map key refuses `invalid_source_map` at the later occurrence or row. Empty carrier occurrences refuse at `/occurrences`; an invalid role refuses `invalid_semantic_graph` at `/role`; a second graph node with the same structural carrier id refuses at the later `/node_id`. Every mutation is independent. | Test |
| FR-038-AC-242 | IMPLEMENTED (IR-800; tagged reader tests passed). A selected Semantic IR document with fields, parameters and results declared at `ix://quire/native/String`, `ix://quire/native/UUID` and `ix://quire/native/Timestamp` admits through `CheckedPackageV2::read` with distinct self-typed `scalar_type`/`string`, `/uuid` and `/timestamp` nodes and matching member types; the same String declaration never resolves as `text` or requires a text profile. Each kind's correctly typed literal admits with the exact canonical payload retained in its keyed body. | Test (TC-048) |
| FR-038-AC-243 | IMPLEMENTED (IR-800; tagged reader tests passed). The production catalog and closed V2 vocabularies admit exactly `quire.op.string.eq`, `.ne`, `quire.op.uuid.eq`, `.ne`, and `quire.op.timestamp.eq`, `.ne` as native comparisons; each public-reader package with same-family operands, Boolean result and no law, mode, member or leaves admits, and equal/unequal canonical payload controls retain their separate operand identities. For each of the six identities, replacing argument 0 and then argument 1, separately, with a foreign-family operand (`text` for `string`, `string` for `uuid`, `uuid` for `timestamp`) refuses `ill_typed`/`operator-ineligible` at the changed argument. `quire.op.integer.lt` and `quire.op.integer.add` over each of `string`, `uuid` and `timestamp` operands likewise refuse at `arguments/0`; neither a one-sided family check nor an ordering/arithmetic special case for one native family passes. | Test (TC-048) |
| FR-038-AC-244 | IMPLEMENTED (IR-800; tagged reader tests passed). Adding a selected `text_profile` law to each native comparison refuses `invalid_package`/`operation-law-mismatch` at that law; adding a `text_profile` mode refuses `invalid_package`/`operation-mode-mismatch` at the mode; placing a `string` operand in `quire.op.text.eq` refuses `invalid_package`/`operation-mode-type-mismatch` at the mode value. An arity defect or another wrong-family operand in that text comparison refuses `ill_typed`/`operator-ineligible` at the arguments or first offending argument before the String-in-text-operation mode refusal. None admits by treating `string` as `text` or by ignoring a forbidden law or mode. | Test (TC-048) |
| FR-038-AC-245 | IMPLEMENTED (IR-800; tagged reader tests passed). For each native `value_kind`, a literal typed at another family refuses `ill_typed`/`operator-ineligible` at its `type` even if its `value` is also malformed. With the right family, a non-string String value, an uppercase, hyphenless or malformed UUID, and a Timestamp value of `-0`, `+1`, `01`, empty text, outside signed-128 range or another noncanonical spelling refuse `invalid_package`/`invalid-value` at that literal's `value`; exact String Unicode sequences, UUID nil/max and both Timestamp endpoints admit. Use otherwise valid structural ids so no independent graph refusal can stand in for these decisions. | Test (TC-048) |
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
| FR-038-AC-262 | Implemented (IR-851); V2 wire amendment planned (IR-1141). When `relationship_end` is called with the id of an admitted `quire.op.model.reaches` application over a bidirectional homogeneous relationship (the direction under which both a forward and an inverse application admit) whose source end is `{1,1}` and whose target end is `{2,3}` unique with role `peer`, it returns `Ok` with `application` equal to the supplied id, `relationship` equal to the relationship node the application names, `forward` true when the application names the source role, and `lower` 2, `upper` `Some(3)`, `unique` true, `ordered` false and `role` `Some("peer")`; for a second admitted application over the same relationship naming the target role it returns `forward` false and the source end's `lower` 1, `upper` `Some(1)`, `unique` false and its own `role`, so a swap of the two ends fails. `destination_type` equals the producer-supplied structural `NodeId` in that admitted application's `relationship_end` member, without reader derivation or resolution; that member is required and its malformed shapes refuse `malformed_wire` under QSpec FR-322-AC-271. | Test (TC-048) |
| FR-038-AC-263 | Implemented (IR-851); V2 wire amendment planned (IR-1141). For admitted reaches applications over a destination end of `{0,3}` unique, `{0,3}` not unique, `{2,3}`, `{0,1}` and `{1,1}`, `relationship_end` returns exactly that `lower`, `upper` and `unique` with `ordered` false and `upper` `Some`, and for a destination end that omits its role it returns `role` `None`; an `unbounded` or `ordered` destination end is refused at admission by FR-038-AC-255, so no application exists to be read. For an admitted application whose operand 1 is a subtype of the receiver endpoint type, `destination_type` remains the producer-supplied id from the member, not the subtype operand's id; the result is `Ok` even when the graph holds no node with that destination id. A present destination node is also admitted and the id returned verbatim (QSpec FR-322-AC-270). | Test (TC-048) |
| FR-038-AC-264 | Implemented (IR-851). `relationship_end` over an id absent from the graph refuses `UnknownNode`; over an admitted `quire.op.model.navigate` application with a `relationship_end` member, an application of `quire.op.integer.add`, the relationship declaration node, an object type's declaration node, a field-read node and a `value`/`literal` node it refuses `NotReachesApplication` for each. Every refusal returns only the typed error and no partial end. | Test (TC-048) |
| FR-038-AC-265 | Implemented (IR-851). When a crate-internal post-admission mutation of an admitted reaches application removes its `operation.member`, sets the member `kind` to a value other than `relationship_end`, sets its `declaration` to a non-node-id value, points it at a node that resolves to no retained relationship, sets its `name` to a non-text value, to a text equal to neither role or to a text equal to both roles, `relationship_end` refuses `UnresolvedEnd` for each and returns no partial end; the defects that the reader can see in a package given to it are refused at admission as before. | Test (TC-048) |
| FR-038-AC-266 | Implemented (IR-851). When `relationship_end` is called twice on an admitted reaches application and on a clone of the admitted package, the results are equal, the package equals its pre-call self, the accessor charges no work, and every package of FR-038-AC-252 through FR-038-AC-257 yields its pre-change admission or refusal outcome at the same pointer and the same refusal order. | Test (TC-048) |
| FR-038-AC-267 | Implemented (IR-851). An external Rust API fixture calls `relationship_end` with `&CheckedNodeId`, exhaustively matches `CheckedRelationshipEndError` without a wildcard arm and constructs `CheckedRelationshipEnd` field by field with exactly the nine fields of the table, so a tenth public field or a `serde_json::Value`, `CheckedMemberType` or population member fails to compile. The FR-019 Public items row for the two new types is added by the code change that exports them, not by this specification change. | Test (TC-048) |
| FR-038-AC-268 | Implemented (IR-851). `model_object_fields` over the object type named by a relationship returns the same fields as before this change and no relationship end, and no `selected_population` item is `pub use`d at the model crate root or re-exported from the `quire-contract-ir` root (FR-053-AC-5); `selected_population` stays reachable only by its module path. | Test (TC-048) |
| FR-038-AC-269 | Implemented bounded selector path (IR-852); the role-end comparand correction is PLANNED/UNRUN (IR-1249), and the corrected admitted-unbounded and exact-decimal projection are PLANNED/UNRUN (IR-1141). For an admitted `quire.op.model.reaches` application whose `operation.member.population` names a `bounded_domain`/`model_population` node over the selected relationship role end's declared type, with `max` 3 and two `Type` occurrences at ordinals 1 and 4, `reaches_population` returns `Ok(Some(selector))` with the package's exact name, the member's structural node ID and the `Type` occurrence at ordinal 1. `select_population(Admitted)` returns `Finite` with canonical decimal maximum "3", minimum zero, selected object type and authentic source regions. For an otherwise equivalent admitted node without `max`, the same selector carries its exact binding and occurrence and `select_population(Admitted)` returns `AdmittedUnbounded` with the same identity and provenance fields and no finite extent (QSpec FR-322-AC-74). All expected values derive from independently authored input, not from the accessor. | Test (TC-048) |
| FR-038-AC-270 | Implemented (IR-852). When `reaches_population` is called with an admitted reaches application whose `operation.member` has no `population` key it returns `Ok(None)`, also in a package that holds a `model_population` node of the receiver end's declared type; two reaches applications in one package naming the same population node return equal selectors, and two naming populations of different bounds return selectors that differ in `binding`. | Test (TC-048) |
| FR-038-AC-271 | Implemented (IR-852). `reaches_population` over an id absent from the graph refuses `UnknownNode`; over an admitted `quire.op.model.navigate` application, an application of `quire.op.integer.add`, the relationship declaration node, a `model_population` node and a `value`/`literal` node it refuses `NotReachesApplication` for each; and when a crate-internal post-admission mutation of an admitted reaches application sets `population` to a non-node-id value, to a node id absent from the graph, to the id of a node that is not a `model_population` node, or to the id of a `model_population` node whose `Type` occurrences are removed, it refuses `UnresolvedPopulation` for each. Every refusal returns only the typed error and no partial selector. | Test (TC-048) |
| FR-038-AC-272 | Implemented (IR-852). A reaches application whose `operation.member.population` is not a node id (a number, an object and `null`), or is a node id with no node in the graph and listed in `dependencies`, is refused by the reader with `ill_typed`/`operator-ineligible` at the member's `population` pointer, for each, and none is refused `missing_declaration`. | Test (TC-048) |
| FR-038-AC-273 | Implemented (IR-852). A reaches application whose `operation.member.population` names a node that is not a `bounded_domain` / `model_population` node, or a `model_population` node with no `Type` occurrence, is refused by the reader with `ill_typed`/`operator-ineligible` at the member's `population` pointer, for each of the two. | Test (TC-048) |
| FR-038-AC-274 | Existing wrong-type refusal implemented (IR-852); the role-end comparand correction is PLANNED/UNRUN (IR-1249). A reaches application whose `operation.member.population` names a `model_population` node over an object type other than the declared type of the selected relationship end role, including a subtype of that type, is refused by the reader with `ill_typed`/`operator-ineligible` at the member's `population` pointer, and one over exactly that role end type is admitted. When the selected role end type is `Order` and the producer-supplied `destination_type` is the structural id of `Shipment`, a `Set<Reference<Order>>` population admits; a population over another declared model type refuses at that pointer. Changing only `destination_type` between schema-valid structural ids does not change the population check or its result. | Test (TC-048) |
| FR-038-AC-275 | Implemented (IR-852). A `quire.op.model.navigate` application whose `operation.member` carries a `population` key that names a node id its `dependencies` list, whatever the node is (an absent node, an integer-range node or a valid population node), is refused by the reader with `ill_typed`/`operator-ineligible` at the member's `population` pointer, at the operation stage; the same application whose `dependencies` omit the id is refused earlier by the dependency join (`invalid_semantic_graph`, no cause). An application of an operation other than `navigate` or `reaches` keeps its present outcome for a `population` key: `type_argument`, `temporal_interval`, `fairness` and `state_clause` members refuse as today and `field`, `position` and `element` members admit it unread. | Test (TC-048) |
| FR-038-AC-276 | Implemented (IR-852). A reaches or navigate application whose `operation.member.population` is a node id is admitted by the dependency join only when its `dependencies` hold that id in the unique, structurally ascending list; a package that omits that id, and a package that lists a population id no reaches or navigate application names, are each refused at the structural stage with `invalid_semantic_graph` and no cause, and a reaches application without the key keeps the dependency list it has today. | Test (TC-048) |
| FR-038-AC-277 | Implemented (IR-852). When `reaches_population` is called twice on an admitted reaches application and on a clone of the admitted package, the results are equal, the package equals its pre-call self and no work is charged; every package of FR-038-AC-252 through FR-038-AC-257 and FR-038-AC-262 through FR-038-AC-268 that carries no `population` key yields its pre-change admission or refusal outcome at the same pointer. | Test (TC-048) |
| FR-038-AC-278 | Implemented (IR-852). An external Rust API fixture calls `reaches_population` with `&CheckedNodeId`, binds the result as `Result<Option<CheckedSelectedPopulationSelector>, CheckedReachesPopulationError>` and exhaustively matches `CheckedReachesPopulationError` without a wildcard arm, so a fourth variant fails to compile; no `selected_population` item is `pub use`d at the model crate root or re-exported from the `quire-contract-ir` root (FR-053-AC-5). The FR-019 Public items row for the new error type is added by the code change that exports it, not by this specification change. | Test (TC-048) |
| FR-038-AC-279 | PLANNED/UNRUN (IR-1141 derived-shape and projection correction and IR-1249 role-end comparand; IR-852's earlier bounded-path evidence remains historical). Under QSpec FR-322 "Derived type forms and forwarding" and FR-322-AC-74, a reaches application linked to a `model_population` over the selected relationship role end's declared type with empty aggregate body and no `max` admits; `reaches_population` returns its exact selector and `select_population(Admitted)` returns `AdmittedUnbounded` retaining package name, selected binding, `Type` occurrence, object type and authentic source regions, with no finite extent. A single integer-literal `max` of 0 or 3 admits and projects `Finite` with exact canonical decimal maximum text and minimum zero; a canonical decimal-string `max` above `u64::MAX` also admits and projects unchanged. A negative or non-integer `max`, a second body member, or a wrong semantic target refuses `invalid_package`/`invalid-derived-shape` at the population node's `node_id` (`RM-POP-MAX`); an invalid non-binding body term retains its earlier graph-term refusal, and a noncanonical JSON number retains `noncanonical_wire` before semantic checks. Neither the reader nor the public projection invents a `u64` maximum cap. | Test (TC-048) |
| FR-038-AC-280 | Implemented (IR-852). A reaches application with an invalid `population` (a node id absent from the graph, listed in its `dependencies`) and an earlier defect in `check_relationship_member` (an operand 1 that does not conform to the receiver end) is refused for the operand at its `arguments` pointer, not at `population`; a reaches application that carries a mode or a leaf is refused by the identity stage (`operation-mode-mismatch` for the mode, a law mismatch for the leaf) whatever its `population` when the `population` is a node id listed in the application's `dependencies`, because the catalog gives `quire.op.model.reaches` neither; an invalid `population` id missing from `dependencies` is refused earlier by the dependency join (`invalid_semantic_graph`, no cause), which runs before the identity stage. | Test (TC-048) |
| FR-038-AC-281 | A reader-admitted protocol member has a `quire.op.protocol.control` body root, exactly one `protocol_profile` law matching the selected lock row, `null` mode, no leaves, Boolean result and semantic types, and a `profile_operator` member. Under the published `quire.protocol.finite-global/v1` selection the admitted operators are exactly `protocol`, `role`, `channel`, `queue`, `finish`, `terminal`, `sequence`, `choice`, `parallel`, `repeat`, `await`, `check`, `send`, `receive`, `attempt`, `effect`, `event`, `commit`, `fence` and `compensate`; an unknown operator refuses `operation-member-mismatch`. A law selected as `quire.protocol.complete/v1` does not satisfy a `finite-global/v1` row, nor the reverse. A missing or mismatched law refuses with its catalogued law cause; a member whose kind is not `profile_operator` refuses `invalid_package`/`operation-member-mismatch` at the application, before any member-body check. An `await` or `compensate` without a selected `temporal_profile` refuses `missing_declaration`/`missing-selection` because it reads a clock role. | Test (TC-048) |
| FR-038-AC-282 | Each of the seven protocol member forms admits only the FR-464, FR-465 or FR-466 fixed argument names, order and term shapes for its selected operator: thirteen clause bindings, the role, channel, queue, finish and terminal layouts, thirteen control layouts and thirteen compensation bindings. A missing, extra, reordered, wrongly shaped or misspelled binding refuses `malformed_wire`; an optional Group admits zero or one member only, an interval has non-negative integer bounds with lower no greater than upper, and a set or sorted-record list repeated or out of its specified ascending order refuses `invalid_semantic_graph`. An ordered `sequence` or capture list retains its authored order. | Test (TC-048) |
| FR-038-AC-283 | PLANNED/UNRUN (IR-1141). A protocol node retains QSpec structural `{package,path,ordinal}` identity: declared paths follow the qualified name, and a valid body or source edit under an unchanged path leaves that id unchanged while changing the complete package value when semantic content changes. A second reference and source reformatting preserve the id and body; changed referenced target, literal, ordered capture or `sequence` child is checked by the independent graph and body rules. Role, channel, finish, terminal, control and compensation declarations use the fixed qualified-name segments of QSpec FR-464; queues have no declaration and generated occurrences, while other protocol members have declaration occurrences. A wrong declared path or occurrence role refuses at the node. | Test (TC-048) |
| FR-038-AC-284 | The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending structural node-id. A package with both a refinement-step and protocol defect reports the refinement defect; one with both a state-step and protocol defect reports the state defect; one with both a protocol and operation defect reports the protocol defect. A nested protocol application refuses `malformed_wire` at strict wire validation before this step. | Test (TC-048) |
| FR-038-AC-285 | Every role, channel, queue, obligation and compensation is listed by exactly one owning protocol clause, and every control is held once in its root control tree with the declaration path of its holder followed by its own name. A member listed twice or unattached, a control held twice or by nothing, a channel without exactly one queue, or a queue attached to another clause refuses `invalid_semantic_graph` at that member. An existing target omitted from its referencing node's `dependencies` refuses `invalid_semantic_graph` at that node's `/dependencies` before the protocol step; a target listed there but absent from the graph refuses `missing_declaration`/`missing-name` at the target. A target of the wrong tag or form refuses `invalid_model_binding`/`malformed-declaration`, except a relationship target of the wrong export refuses `wrong-export`. | Test (TC-048) |
| FR-038-AC-286 | A role admits the FR-465 static and replicated shapes only, including a model of an eligible form, replication maximum at least one, a reference-typed binder and an `until` term exactly for an `until` lifetime; a channel admits only same-clause role endpoints, an eligible message type, explicit delivery and ordering, and a FIFO key with equality-stable type; a queue admits one channel, literal or symbolic capacity, explicit overflow and a loss selector exactly for `loss`. The reader admits the positive variants and refuses the wrong form, missing premise, wrong binder type, non-Boolean lifetime, noncanonical capacity or key and ineligible equality with the FR-465 code, cause and locus. | Test (TC-048) |
| FR-038-AC-287 | A clause names exactly one `finish` obligation whose closing-record binder and Boolean constraint read only the binders available after the root control; its optional `terminal` obligation admits `when` with one Boolean condition or `any` with none. A missing or extra condition refuses `malformed_wire`, a non-Boolean condition or constraint refuses `ill_typed`/`non-boolean-root`, an unjoined branch binder read by finish refuses `missing_declaration`/`missing-name`, and a terminal target in the finish position or an unattached finish refuses by the FR-464 membership rules. | Test (TC-048) |
| FR-038-AC-288 | Each of the thirteen control operators admits its FR-466 ordered arguments and local reference, type, bounds, label and ordering checks: `sequence`, `choice`, `parallel`, `repeat`, `await`, `check`, `send`, `receive`, `attempt`, `effect`, `event`, `commit` and `fence`. A choice or parallel branch list with fewer than two entries, repeated labels, a receive naming a non-send, an effect naming a non-attempt, an await naming an ineligible event, a non-Boolean guard or constraint, an invalid join count or an ineligible memory or ordering spelling refuses with the FR-466 code, cause and locus, without changing the accepted meaning of another operator. | Test (TC-048) |
| FR-038-AC-289 | A `compensate` body admits the thirteen FR-466 bindings in order, including distinct registration and activation captures, a trigger, interval, at-least-one attempt, retry relation, optional commit and recovery predicate. `commit never` is an absent optional Group, not a fabricated commit. The reader refuses a forward target that is not an `effect`, a commit target that is not a `commit`, an inverted interval or zero attempts, a commit that dominates its forward effect, and a read outside any of the five compensation stages with the corresponding FR-466 code, cause and locus; a commit in one case of a `choice` or in a non-all parallel branch before the effect is admitted when it does not dominate that effect, and the package creates no runtime compensation instance. | Test (TC-048) |
| FR-038-AC-290 | For each clause the reader constructs the QSpec FR-464 derived causal relation from control containment, sequencing, receive/send, effect/attempt and await edges, not source order or timestamps. A receive before its send or an await anchored at a later control refuses `invalid_package`/`definition-cycle` naming the cycle; a same pair on independent parallel branches admits. A repeat whose body can complete with no progress on a path refuses `unsupported_construct`/`declaration-form`, while a repeat with a real step admits with or without a maximum and loop proof. | Test (TC-048) |
| FR-038-AC-344 | An outer `repeat` whose sole body control is a nested `repeat` around an `attempt` admits when the nested guard exit is infeasible and its maximum is absent or positive. | Test (TC-048; QSpec TC-946 R13/R14/R24)
| FR-038-AC-345 | The same outer `repeat` refuses `unsupported_construct`/`declaration-form` at the outer control when the nested guard exit is feasible, regardless of an absent or positive maximum. | Test (TC-048; QSpec TC-946 R16)
| FR-038-AC-346 | The same outer `repeat` refuses `unsupported_construct`/`declaration-form` at the outer control when the nested maximum is zero, even if its guard exit is infeasible. | Test (TC-048; QSpec TC-946 R15)
| FR-038-AC-347 | A nested `repeat` with a nonstepping body supplies no progress to its outer `repeat`, even when `exhausted` contains an `attempt`; the first progress refusal is at the outer control. | Test (TC-048; QSpec TC-946 R18)
| FR-038-AC-348 | An outer `repeat` containing a nested control with a feasible exit admits when a sequence sibling, parallel or await provides the necessary step; a choice admits only when all cases step. | Test (TC-048; QSpec TC-946 R17/R19)
| FR-038-AC-349 | A nested guard whose listed visible entry is the composite expression `E = a and b` treats `E` as one independent atom beside `a` and `b`; `E or not a or not b` therefore has a feasible exit and leaves its sole-step outer `repeat` refused. | Test (TC-048; QSpec TC-946 R22)
| FR-038-AC-350 | A nested guard `eq(a,b) or not eq(a,b)` has an infeasible exit, while `eq(a,b) or not eq(b,a)` has a feasible exit when the two distinct equality nodes are listed; the outer sole-step `repeat` admits only the first. | Test (TC-048; QSpec TC-946 R20)
| FR-038-AC-351 | An inline or referenced Boolean literal `true` is a constant rather than an atom, so the outer sole-step `repeat` admits when its nested guard is either `true` or `a or true`. | Test (TC-048; QSpec TC-946 R23)
| FR-038-AC-352 | A nested two-atom tautology charges four valuation work units, while a nested `a or b` charges only its first, all-false valuation; atom collection and comparison add no work units. | Test (TC-048; QSpec TC-946 R21)
| FR-038-AC-353 | A valuation charge denied by `CheckedPackageReadLimits.work` returns incomplete `stage_limit_exceeded`/`work-budget-exceeded` at the nested control before an enclosing progress refusal, including for a nested maximum of zero. | Test (TC-048; QSpec TC-946 R21)
| FR-038-AC-354 | A standalone `repeat` guard with 30 visible atoms consumes the same reader work as its `true` replacement, because no valuation runs outside another `repeat` body. | Test (TC-048; QSpec TC-946 R21)
| FR-038-AC-355 | A nested `repeat max 1 while true` around an `attempt` with an `exhausted` `check` supplies the outer body's step on every outer iteration because its counter resets on entry. | Test (TC-048; QSpec TC-946 R24)
| FR-038-AC-356 | A causal-cycle refusal precedes nested valuation and progress outcomes; when the cycle is removed, denied valuation precedes the first nonprogressing `repeat` in tree order. | Test (TC-048; QSpec FR-464-AC-12, TC-946 O5/O7/R18/R21)
| FR-038-AC-291 | Binder levels start at the introducing construct's entry count; protocol `over` is level 0, an `each` trigger follows, captures advance in their authored order, and a control binder is at the size of its readable input set. The reader admits reads within the FR-464/466 stage tables and refuses a wrong level, duplicate readable name, forward or foreign binder, or read after a branch, choice, repeat or await path where the binder is unavailable with the specified code, cause, stage and owner. In a binders-only slot, including each `choice` or `repeat` `visible` entry and guard, a direct read of `over` or model state refuses `ill_typed`/`operator-ineligible` at the read before binder-scope or decision-fact checks. | Test (TC-048) |
| FR-038-AC-292 | At a `choice` or `repeat` decision the reader admits only QSpec FR-466 visible facts owned by the deciding role and listed individually in `visible`; a field fact from another role, a readable but ineligible `send`, `effect` or `commit` binder or protocol activation binder, a fact reachable only inside a composite entry, or a readable fact omitted from `visible` refuses `wrong_snapshot`/`wrong-observation` with the specified `other-role`, `ineligible-kind` or `unlisted` reason and a retained binder and field. In each of the four `choice`/`repeat` `visible`/guard slots a direct read of protocol input `over` refuses `ill_typed`/`operator-ineligible` at the read, naming the slot, never `wrong_snapshot`. A bare read of any record binder, including the protocol activation binder, with no field projection refuses `ill_typed`/`operator-ineligible` at that bare read with the slot detail after scope and model-state checks and before decision-fact checks; it never fabricates a field or `wrong_snapshot` detail. A read of the finish binder or a compensation-template binder in any of those slots refuses `missing_declaration`/`missing-name` with `read` `stage` or `foreign` respectively, never `wrong_snapshot`. A slot holding both an `over` read and a binder read reports the `over` refusal first; a slot holding an out-of-scope binder read and an in-scope ineligible binder read reports `missing-name` first; a slot holding a bare record binder read and an in-scope ineligible field read reports the bare-read refusal first in either operand order. | Test (TC-048; QSpec FR-466-AC-14/16, TC-948 DF7g/DF7h) |
| FR-038-AC-324 | For an admitted protocol clause the reader derives one activation-relation row per distinct required temporal clause from the clause's `activation` and `requirements` alone, in ascending required-clause structural node-id order. `CheckedPackageV2::protocol_activation_relation` returns the rows for all local clauses in ascending clause structural node-id order, with exactly the public row fields and `Origin`/`Each` variants specified above, and with no separate wire or identity member; repeated calls and equal admitted packages return equal rows without changing the package or charging reader work. The row contains the protocol clause key, `origin` or `each` with the trigger key, and the required temporal clause key. No requirement gives no row; two source `requires temporal` declarations encoded as one canonical set entry give one row; two duplicate wire entries refuse `invalid_semantic_graph` under the clause's set rule and expose no relation; a second distinct requirement gives two rows; a temporal clause sharing the clock but not required gives no row. An existing `requirements` or `each` trigger target omitted from the clause's `dependencies` refuses `invalid_semantic_graph` at the clause's `/dependencies` before the protocol step; a target listed there but absent from the graph refuses `missing_declaration`/`missing-name` at that target. Neither exposes a partial relation. | Test (TC-048; QSpec TC-946) |
| FR-038-AC-325 | The protocol step compares every present refinement `class_catalog` against the complete concrete and abstract classes reconstructed from selected model declarations and the admitted local or directly selected dependency protocol graph, including distinct `fork` and `join` classes on one parallel node, checked binder types and source-default memory classes, even with no step row. The concrete refinement `semantic_type` resolves to a selected package `model_root`, while an owned clause's separate `owner_model` resolves to an object-type node of a model owned by that selected declaration; classes from that clause count even without a step row. Foreign, unselected and wrong-kind owners cannot complete the selected inventory. A source subject with `sc`, `tso` and `ra` defaults admits three distinct `memory drain` entries with `drain: store-buffer-flush`, keyed by (`class`, `memory_model`) and ordered `ra`, `sc`, `tso`, followed by `memory flush` for `tso`; repeated selection of one default adds no entry. An `on each (x: T)` clause admits `activate <protocol>` associated with that clause node and binder types `[T]` for non-record `T`, or the effective fields of record `T` in source supertype order then own declaration order, retaining a diamond field once and placing a redefining field where the hidden field stood; reversing source supertype order changes that binder order. An `on origin` clause admits no activate entry. An ambiguous same-name inherited field without redefinition refuses `ambiguous_declaration`/`ambiguous-name` at `/semantic_graph/nodes/{n}/body/class_catalog/{side}/{i}/binder_types`. A wrong subject, clause/node association, or wrong or reordered binder signature refuses `invalid_model_binding`/`malformed-declaration` at the catalog entry; a missing or extra class refuses `invalid_semantic_graph` at the catalog array or entry; duplicate or misordered catalog keys refuse at the earlier refinement canonical-order check, with memory keys compared by class then model. An absent required or forbidden present catalog refuses `invalid_semantic_graph` at its member. A protocol-body defect reports before this graph join; a refinement-step row/catalog defect reports before either. A later request memory override does not change checked-package catalog facts or package identity. | Test (TC-048; QSpec TC-310) |
| FR-038-AC-326 | For a protocol activated `on each`, a Boolean `terminal when` condition that reads the protocol's `over` model observation admits, as QSpec FR-464's `terminal` stage and TC-947 TM11 require. The same read admits under `on origin`. Under `on each`, a condition that instead reads a protocol capture or the activation binder refuses `missing_declaration`/`missing-name` at the terminal node with stage `terminal`; under `on origin`, a condition reading an initialized protocol capture admits. The reader applies the stage table to each read independently: allowing `over` under `each` does not make its captures or activation binder readable. | Test (TC-048; QSpec TC-947 TM10 through TM13) |
| FR-038-AC-327 | For each protocol-family node form with a local body target, omitting an existing target from `dependencies` refuses `invalid_semantic_graph` at `/semantic_graph/nodes/{i}/dependencies` during graph-shape validation, ahead of a simultaneous protocol-body defect; an extra dependency refuses there too. A body target naming no graph node but included in `dependencies` instead reaches its body join and refuses `missing_declaration`/`missing-name` at that target. In particular QSpec TC-946 C13/C13a, A8/A8a, A9/A9a and OM-09 keep these distinct outcomes, with no activation relation exposed. | Test (TC-048; QSpec TC-946) |
| FR-038-AC-328 | A protocol refinement whose selected abstract subject is a model with an operation and no selected protocol clause admits a complete abstract catalog containing that operation class, even when no step row names it; omitting the class refuses `invalid_semantic_graph` at the abstract catalog array, and adding a class from another selected model refuses that code at the extra entry. | Test (TC-048) |
| FR-038-AC-329 | For selected clauses whose package-local `declaration.qualified_name` paths have different depths, the catalog uses each full clause path and the member's authored name to reconstruct the FR-432 alias-free source class. FR-464's fixed member-kind segment (`finish`, `run`, `channel`, `role` or `compensate`) and intervening control ancestors identify the node but are absent from the class spelling: a QSL-authored `P` clause with member paths `[P, finish, Done]` and `[P, run, Main, Par]` contributes `P::Done`, `fork P::Par` and `join P::Par`, while `activate` contributes `activate P`. A class that omits a clause-path segment, substitutes another clause or member name, or includes a fixed kind or control ancestor refuses `invalid_semantic_graph` at the affected catalog array or extra entry. | Test (TC-048) |
| FR-038-AC-330 | An admitted compensation template contributes its FR-432 `cend` class associated with the template node to the complete concrete or abstract catalog even without a step row; an omitted, extra or wrong-node `cend` entry refuses under the existing missing-class, extra-class or wrong-association rules of FR-038-AC-325. | Test (TC-048) |
| FR-038-AC-331 | Implemented (IR-1082). Through the public V2 reader, an admitted field-member operation returns its exact `(CheckedNodeId, name)` declaration/name pair from `projection_field()`; an admitted member of another kind and an operation with no member return `None`. A malformed or unselected field declaration is refused by admission and cannot be returned. | Test (TC-048) |
| FR-038-AC-332 | Implemented (IR-1082). Through the public V2 reader, `member_declaration()` returns the exact admitted `CheckedNodeId` for a field member and a catalogued non-field member carrying `declaration`, including `quire.op.quantity.convert`; it returns `None` when the admitted member has no declaration or is absent. A malformed declaration is refused by admission and cannot be returned. | Test (TC-048) |
| FR-038-AC-333 | Implemented (IR-1082). Through the public V2 reader, an admitted rounding-mode operation returns a `CheckedOperationModeView` with `OperationModeKind::Rounding` and its exact admitted value. A different admitted mode kind returns that kind and its value; an operation with no mode returns `None`. A malformed or catalog-ineligible mode is refused by admission and cannot be returned. | Test (TC-048) |
| FR-038-AC-334 | Implemented (IR-1082). Through the public V2 reader, `law_definitions()` returns the exact admitted `CheckedArtifactRef` authority/identity pair for each law, in admitted order; no laws return an empty view. A law with an unselected or malformed definition is refused by admission and cannot be returned. The accessor exposes no raw operation record. | Test (TC-048) |
| FR-038-AC-341 | PLANNED (IR-1087). For a V2 reader-admitted operation with no mode, member declaration, field projection, state-clause kind or laws, the five typed accessors return admitted absence or an admitted empty law view; for an otherwise identical direct-deserialized operation without validated views, each returns a typed `NotAdmitted` refusal, never the admitted absence value. Populated V2 operations retain their admitted values under the same fallible accessors. | Test (TC-048) |
| FR-038-AC-246 | Implemented (IR-803). For a reader-admitted `quire.op.structural.eq` application, `composite_application_operands` returns `CheckedCompositeOperator::Eq` together with its operands; for an otherwise equivalent reader-admitted `quire.op.structural.ne` application it returns `CheckedCompositeOperator::Ne` together with its operands. Between those cases, only the returned operator and the selected application's identity and occurrence may differ; operands (including order, child identities, semantic types and domains), ordinals and `consumed_work` are equal. The AC-182 public-fields inspection in `tests/it/checked_package_v2_composite_operands.rs` (`inspect_public_fields`) must exhaustively match both `CheckedCompositeOperator` variants, without a wildcard arm or `#[non_exhaustive]`. The consumer needs no separate operation lookup. TC-048 composite operand tests verify both cases. | Test (TC-048) |
| FR-038-AC-303 | PLANNED/UNRUN (IR-1141). A self-typed relation/population node with no declaration or recursion group, an empty aggregate body, empty dependencies and a ModelOwner admits under its structural id. Its owner joins the selected model declaration. An absent owner refuses malformed_wire before joins, and a wrong fixed member refuses by the model-declaration body-shape rule; no nominal preimage or key recomputation occurs. | Test (TC-229) |
| FR-038-AC-304 | After FR-154 admission, the reader joins a population reference's `owner.node` to the raw `identity` of a `populations` record in the document selected by `owner.identity`. A record without an `identity` string, with a `node` member, with an unresolved `kind` construct, or with a construct whose meaning is not population refuses `invalid_model_binding`/`malformed-declaration` at the selection before the join. The QSpec `counters` and `archive` owners admit; an unknown node, an object-type node claimed as a population, and a population node claimed as an object type or relationship each refuse `missing_declaration`/`missing-selection` at the first offending node id. The reference node's digest never substitutes for the declaration key. | Test (TC-229) |
| FR-038-AC-305 | `quire.op.model.all_instances` and `quire.op.model.lookup` each admit a reference to a declared population as operand 0 and list it in dependencies; the same declared population is one reference node across both reads, while the two named populations yield distinct reference and application paths within the same named package. A `value`/`parameter` typed `Population<O>[N]` remains an admitted operand 0. A unit maximum is represented only in the application's result-type collection bounds, not in the declared population node. | Test (TC-229) |
| FR-038-AC-306 | A declared population reference used as a `collection_kind` operand, as the `population` link of a reaches `relationship_end`, or as the declaring node of a `field`, `operation` or `relationship_end` member refuses `ill_typed`/`operator-ineligible` at that operand or member. An otherwise identical binding typed by `bounded_domain`/`model_population` keeps its existing outcome. | Test (TC-229) |
| FR-038-AC-307 | The reader consumes QSpec's selected-checkout published population-reference vectors and operation fixture directly, without copying them into this repository: it admits the amended positive package and checks its package, application and operand identities; it refuses the retired ownerless relation/population operand as `malformed_wire`; and it matches the `population_reference_cases` adverse outcomes by code, cause and locus. | Test (TC-229) |
| FR-038-AC-299 | A `quantity` literal typed by a `scalar_type`/`unit` node admits with family `quantity` for the canonical magnitudes `3`, `0`, `-5`, `1/1000` and `-7/2`. Each of `03`, `+3`, `2/4`, `4/2`, `1/0`, `0/5`, `3/1`, `1/-2`, ` 3`, `2.5` and the empty value refuses `invalid_package`/`invalid-value` at `value`. A quantity literal typed by an integer, string or Boolean node, and an integer literal typed by a unit node, refuse `ill_typed`/`operator-ineligible` at `type`; the V2 schemas enumerate `quantity`. | Test (TC-230) |
| FR-038-AC-300 | A field, operation parameter and operation result `typeRef` `ix://quire/unit/ms` resolve to the checked package's existing `ms` unit node and retain `Quantity<ms>` under scalar, optional and set multiplicities. A missing `ix://quire/unit/ghost` refuses `missing_declaration`/`missing-name`; a `Quantity<s>` or integer argument to a parameter requiring `Quantity<ms>` refuses `ill_typed`/`operator-ineligible`. No unit type node is synthesized from the model document. | Test (TC-230) |
| FR-038-AC-301 | A unit, dimension and record with the same qualified declaration name admit as three namespaces; two units or two types with that name refuse `ambiguous_declaration`/`ambiguous-name` at the later node in node-id order before operation checks. | Test (TC-230) |
| FR-038-AC-302 | TC-230 consumes QSpec's selected-checkout QL-01..04, MU-01..06 and NS-01..03 cases directly or constructs their recorded wire shapes through the public reader, checks accepted and refused outcomes by code, cause and locus, and compares strict coverage with the main-branch baseline. | Test (TC-230) |
| FR-038-AC-308 | For an admitted graph with K distinct requested leaves over one shared R-node closure, at K=2,4,8 and R=4,8,16, the reader validates each body once and retains the resolved adjacency and body-term count; lowering performs no further body grammar validation, reuses closure reachability per strongly connected component, and does not scan the full closure separately to decide each candidate bound. Per-request node, term and edge charges, the consumed value at a work-limit stop, each record and the complete package canonical bytes equal the independently lowered baseline. Admission performs R+K body grammar validations for the R shared nodes and K distinct leaves, and lowering performs zero additional validations; a grouped recursive component and a stop within the shared closure preserve the same outcomes. | Test (TC-050) |
| FR-038-AC-309 | `CheckedPackageV2::lower_independent_records` returns one record per requested key in input order, each byte-for-byte equal to the record from `lower` called with only that key and the same profile and retained byte ceiling. For a shared R-node closure under K requests, graph/source indexes and component closures are built once and body validation is not repeated. When every one-key package fits but their union exceeds the byte ceiling, this method retains each one-key record while `lower` on the union reports its existing whole-call `failed` result. | Test (TC-048, TC-050) |
| FR-038-AC-310 | PLANNED/UNRUN (IR-1141). A top-level identity_preimage, package_id or identity_projection member refuses unknown_member at that member. Removing the extra member admits the positive package with its required name, lock and graph. | Test |
| FR-038-AC-311 | The typed V2 node-family/form vocabulary admits exactly `hybrid`/`hybrid_model`, `hybrid`/`hybrid_mode` and `hybrid`/`hybrid_transition` under the `hybrid` tag. QSpec TC-966's transition with a declaration, mode occurrence of role `claim`, reordered model bindings, class `continuous`, reference target index, mode with no rates, unknown `hybrid_flow` form and ownerless model each refuse `malformed_wire` at strict wire validation, before hybrid body checks; the valid `Braking` and `Tank` nodes reach those checks. | Test (TC-048) |
| FR-038-AC-312 | The reader admits TC-966 `Braking` model, modes, transition and `SlowEnough` claim with their structural ids, self semantic types and exact dependencies. A meaning-bearing body or owner edit is validated through graph and body checks; a mismatched dependency list refuses at graph shape before a hybrid body defect. No rekey or package identity refresh is required. | Test (TC-048) |
| FR-038-AC-313 | With dependencies and references updated while structural ids and package name remain valid after each edit, the reader runs QSpec FR-481 model checks 1 through 4 in order: repeated mode/default initial, variable type/level/name, mode declaration/rate count/term types/transition index and reset conflict, then initial mode and condition. TC-966's HY-03 through HY-08, HY-17's structural-id initial-entry cases and HY-18's structural-id reset-conflict case match their published code, cause and locus; when two model checks fail, the earlier check wins. HY-11's repeated declared mode name is refused by earlier name validation, and HY-18's invalid `eq` relation is refused by strict wire validation, both before these model checks. | Test (TC-048) |
| FR-038-AC-314 | Model term reads and called-function types are checked after initial entries; clock rates and the required hybrid variable are checked after reads. TC-966's foreign `w` read, nonliteral clock rate, all-clock `Braking` and all-clock `Timer` refuse with the published code, cause and locus. TC-966's `Tank`, whose rate and reset value reference another variable and whose `hybrid` variable has a literal-one rate, admits. An unreferenced hybrid transition refuses after all model checks and before claim checks. | Test (TC-048) |
| FR-038-AC-315 | The temporal step precedes the hybrid step: a hybrid claim's integer or timed interval is admitted only under its matching FR-370 profile, including timed `[q, *]` under `quire.temporal.timed/v1` and integer `[2, *]` under `quire.temporal.infinite-trace/v1`. A wrong interval form refuses `invalid_package`/`operation-member-mismatch` there before a fairness defect. The surviving `SlowEnough`, `Slows` and `Recent` claims are read from their published graph nodes with their respective clock, activation, capture, fairness and read levels. | Test (TC-048) |
| FR-038-AC-316 | After all models and orphan modes/transitions, hybrid claims are checked in structural node-id order for clock/activation/guard/capture form, fairness, then model-variable/activation/capture reads. TC-966's `wall` clock, mistyped `each` trigger, fairness member naming `Halted`, `holds` reading `h` and `holds` reading a parameter at the wrong level refuse with FR-481's code, cause and locus; a valid claim and a later protocol defect yield the protocol refusal, while a hybrid defect and a protocol defect yield the hybrid refusal. | Test (TC-048) |
| FR-038-AC-317 | PLANNED/UNRUN (IR-1141). TC-048 reads QSpec TC-966's published `hybrid-node-vectors.json` and `positive-all-families.json` from the selected QSpec checkout, without a local copy. Strict-wire and graph-shape cases precede hybrid body checks; each vector checks its exact code, cause and locus. The IR reader does not substitute its own process-provider or host trace checks for QSpec's HY-15 and HY-16. | Test (TC-048) |
| FR-038-AC-318 | The public reader admits exactly QSpec FR-322-AC-120's eight `timed_model` forms with the ordered root, tick-source and clock-place members stated above. Each admitted form retains its published semantic type, occurrence roles, owner, declaration and complete body; the root's time dimension matches the source unit's nominal dimension; a tick source model and operation-anchor context match the root model; each resolved-path row matches its path step and selected model declaration, including collection bounds and terminal `Clock` after removal of any `Option` wrapper. An unknown form or malformed outer/body member refuses at strict wire validation before any timed-model body check. The writer emits one complete body per node and no wire complete package value. | Test (TC-048; QSpec TM-06..TM-21) |
| FR-038-AC-319 | For a timed model with source, place, reset, guard, invariant, urgency and constraint nodes, graph shape checks exact dependencies and reference resolution before timed-model interpretation. A meaning-bearing body or owner edit is validated on its own terms; a stale dependency list refuses at graph shape. Reordering source occurrences does not alter structural ids. | Test (TC-048) |
| FR-038-AC-320 | The timed-model step reconstructs the selected model from exactly one root and its referenced nodes. A missing root, dangling or wrong-form target, noncanonical path, and invalid or non-time quantity each refuse with FR-322-AC-120's paired code and cause at the failing member, visiting timed nodes in ascending structural node-id order. For TM-12, a distinct second place of one path in valid aggregate order refuses `invalid_package`/`malformed-clock-place` at its forged `resolved_path` row; TM-30 lists one place twice and refuses `invalid_semantic_graph` with no cause at the repeated aggregate member. The retained `duplicate-clock-place` rejection follows row joins, but a conforming distinct second place cannot reach it. The valid QSpec TC-367 CK-25 model admits and retains its normalized constraints. | Test (TC-048) |
| FR-038-AC-321 | A package with both a timed-model defect and a protocol or operation defect reports the timed defect; one with a hybrid defect and a timed defect reports the hybrid defect. Strict-wire and graph-shape defects continue to precede either body step. | Test (TC-048) |
| FR-038-AC-322 | The public V2 writer round-trips an admitted timed model through the reader with the same complete node order, structural node ids, selected model and normalized quantities, and emits no `package_id`, `identity_preimage` or `identity_projection` member. | Test (TC-048) |
| FR-038-AC-323 | For each timed root, the `time_dimension` reference names the exact selected base `Time` dimension node, and the source unit's `dimension_node_id` equals that node by identity. A tick source that owns a clock place refuses `invalid_package`/`clock-without-dense-time` at the place's model reference target. A source unit whose dimension is base `Length`, a derived non-Time dimension or a nominal derived-Time dimension refuses `ill_typed`/`invalid-time-quantity` at the source unit reference target `/semantic_graph/nodes/{source}/body/members/1/value/target`, for both dense and tick sources; dimension equivalence, matching terms, names or declaration order do not replace identity equality. | Test (TC-367; TM-23..TM-29) |
| FR-038-AC-335 | One `clock_place` template admits `ts.0.x`, `ts.1.x` and `ts.2.x` over `Sequence<Timer>[0, 3]` with path steps `field(ts)`, `slot(sequence, Integer type node, 0, 3)`, `field(x)` and one node key; Set, Bag and OrderedSet paths admit with their own collection literal and key, two nested sequences admit two Integer-typed slot steps, and `Map<Text, Clock>` admits with the exact Text key type node. No place node stores a concrete index or state element count. A slot selector given as an integer literal, an expression node, the Text type node under Sequence, or the Integer type node under `Map<Text, _>` refuses `invalid_package`/`malformed-clock-place` at `/semantic_graph/nodes/{i}/body/members/1/value/members/{j}`. | Test |
| FR-038-AC-336 | A `timed_model` root with empty `guards` admits when its selected model has no guard. Adding one root-listed guard changes the complete guard and root bodies while leaving the same `clock_place` path template unchanged; a `guards` binding on the place refuses at strict wire validation. A second distinct place node of the same model and path in otherwise canonical order has a forged `resolved_path` row and refuses `invalid_package`/`malformed-clock-place` at the first differing row; row join runs after aggregate order and before the retained `duplicate-clock-place` check. A conforming second place with all rows joined cannot exist: it has the same body and key, and repeating one node reference fails aggregate order first. A `clock_guard` omitted from the root's guard aggregate refuses `invalid_package`/`timed-node-reference` at the unlisted guard's own `body`; omission is never interpreted as an absent guard. | Test |
| FR-038-AC-337 | The public reader admits the Boolean `presence_read` place form of `quire.op.option.present` for `present(other.v)` and `present(e)` in a quantifier over `q`, with arguments exactly the place template, owner, and each slot term in path order. The `q` slot term `e` is typed at the Integer selector domain, including the quantifier binder `e`, rather than at the element value type. Each application retains its structural id while every instance of one path names its place template id; no model, type, parameter or place node gains a presence member or `Option<Boolean>`. The indexed read `present(self.q[i])` is not a presence form in this version, following QSpec FR-322 and owner ruling #485 (`81078f14`). The existing one-operand Option form keeps its admission and structural id. | Test |
| FR-038-AC-338 | A place-form `presence_read` over `q` lacking its slot term refuses `ill_typed`/`operator-ineligible` at the application's `/body/arguments`; a first argument that is not a `clock_place` refuses that pair at `/body/arguments/0`; an Integer-typed owner instead of the place's object owner and a wrong-typed slot term each refuse `ill_typed`/`type-mismatch` at the offending argument. No refusal returns an admitted package. | Test |
| FR-038-AC-339 | For `v: Option<Clock>`, the place has only `field(v)` and one resolved-path row retaining `Option<Clock>`; the reader joins that row to the selected owner's declared `v`, optional presence and `Clock` inner type, then accepts terminal Clock after unwrapping Option without adding a path step. For `q: Sequence<Option<Clock>>`, it admits `field(q)` and a symbolically selected `slot` whose row retains Option type. A missing selected member, wrong declared presence or wrong inner type refuses the existing malformed-clock-place path join; the owner object-type key is unchanged by the field set. | Test |
| FR-038-AC-340 | TC-048 reads QSpec's published `timed-slot-selector-vectors.json` through the selected checkout, with no copy in IR, and checks SL-01 through SL-07, SR-01 through SR-04, GD-01 through GD-03 and PR-01 and PR-03 through PR-06 against exact admission or each vector's expected code and cause. For SR-01..04 and PR-04..06, it also checks the vector's RFC 6901 locus suffix. For GD-02, it checks `invalid_package`/`malformed-clock-place` at the twin place's forged `resolved_path` row fixed by QSpec FR-322-AC-120, after aggregate order passes. GD-03 checks `invalid_package`/`timed-node-reference` at `/semantic_graph/nodes/47/body` with suffix `body`, fixed by QSpec FR-322-AC-243. The test checks reader stage against FR-322's stated order, separately from the vector data. Structural graph and owner checks precede timed-body or operation refusals, and the existing Option-operand form remains an admitted control. | Test |
| FR-038-AC-342 | PLANNED/UNRUN (IR-1096; amended by IR-1109). The public V2 reader admits QSpec's selected `AO-01` timed-slot package and `AO-05` tie-on-start package: all six root reference aggregates ascend by each node's minimum source-region triple, followed on a tie by FR-322-AC-265's structural member key. It refuses otherwise-valid `AO-02`, `AO-03`, `AO-04`, `AO-06` and `AO-07` as `invalid_semantic_graph` with no cause at the first out-of-order member locus; structural id order cannot admit `AO-03`, tied starts use ends in `AO-06`, and `AO-07` uses the earlier region of the additional source-map entry. `AO-08` admits tied-region places in path order; `AO-09` and `AO-10` refuse reversed or equal structural place keys at the second member before a duplicate-path check. For each root `resets`, `guards`, `invariants` and `constraints` branch, an admitted two-distinct-node aggregate and its otherwise-valid reversed aggregate prove admission and `invalid_semantic_graph` at the exact second-member locus; two distinct nodes with a tied minimum region admit in structural-key order and refuse when reversed, while two references with equal full keys refuse at the second member. An ordered repeated place path reaches `malformed-clock-place` at its forged row after aggregate order; `duplicate-clock-place` remains the post-row-join rejection, unreachable for a conforming second place. A root missing a guard reaches the unlisted-guard refusal. For every `AO-02`, `AO-03`, `AO-04`, `AO-06` and `AO-07`, malformed wire, missing dependency and duplicate structural id each refuse at their exact earlier-stage locus while its order defect remains. | Test |
| FR-038-AC-343 | PLANNED/UNRUN (IR-1141). A public V2 read admits a positive package with declared and derived structural ids. A duplicate id, invalid declared path or malformed derived shape refuses at its structural graph or body stage. An owner selecting an absent or unselected declaration refuses `missing_declaration`/`missing-selection` at the owner; an independent dangling `reference` target refuses at that reference; a wrong compound-unit shape refuses `invalid_package`/`invalid-derived-shape`; and a frame whose `semantic_type` names no node refuses `missing_declaration`/`missing-name`. When paired, graph shape precedes owner and body checks, while owner selection precedes dependent dangling semantic-type or dependency edges. Each case checks exact code, cause and RFC 6901 locus through the public reader. No nominal-rekey vector, content-key recomputation, stale-key refusal or copied QSpec artifact is used. | Test (TC-048) |
| FR-038-AC-357 | IMPLEMENTED; focused TC-048 reader evidence: `timed_clause_admission_retains_one_anchor_occurrence_and_operation_urgency` round-trips every admitted clause body, and `timed_clause_published_vectors_match_reader` checks CL-01 and CL-17..20/27/28. The public V2 reader admits the selected QSpec `CL-01` clause package and verifies the exact closed bodies of `clock_constraint` kinds `place` and `parameter`, `clock_reset` kinds `place` and `parameter`, `clock_guard`, `time_invariant`, and three-binding `clock_place`. It admits `eq` as a constraint comparison outside invariants. `CL-17` through `CL-20`, `CL-27` and `CL-28` refuse at strict wire validation before semantic joins, covering empty guard parts, empty invariant bounds, unknown comparison, a guard part without a constraint, place `guards` and a conditional parameter reset. | Test |
| FR-038-AC-358 | PLANNED/UNRUN (IR-1141). The existing focused TC-048 reader tests cover the published unlisted-node causes and stage distinctions; the old two-node test orders by a content-derived key and cannot back structural-id order until its fixture is revised. After aggregate order, the reader refuses the first unlisted timed node by structural node-id at that node's own `/body`, before any per-node join: `CL-05` (constraint), `CL-26` (reset) and an omitted guard all return `invalid_package`/`timed-node-reference`. After per-node joins, a second guard for one anchor (`CL-04`) and a root-listed constraint named by no part or invariant (`CL-06`) refuse at QSpec's exact operation and root-constraint loci respectively. A listed clause and a referenced constraint admit in the positive control. | Test |
| FR-038-AC-359 | IMPLEMENTED; focused TC-048 reader evidence: `timed_clause_published_vectors_match_reader`, `timed_clause_equal_regions_use_structural_order_before_refusing_repetition` and `timed_equal_region_rational_order_charges_before_big_integer_work` cover ordered/adverse pairs and rational ties. Root aggregates, each guard part's constraint list and each invariant's bounds use the region-first then structural key of QSpec FR-322-AC-260/265. `CL-16`, `CL-30`, `CL-32` and each reversed member-key case `CL-34`, `CL-36`, `CL-38`, `CL-40`, `CL-42`, `CL-44`, `CL-46`, `CL-49`, `CL-51`, `CL-53` refuse `invalid_semantic_graph` with null cause at the first nonascending member; their paired ordered cases admit. Numeric rational bounds, comparison rank, owner and slot terms, operation, condition, value, binder and every invariant bound/predicate distinguish ties. `CL-47` admits when a shared term acquires an earlier region; no shared term's region or node id reorders its owner. | Test |
| FR-038-AC-360 | IMPLEMENTED; focused TC-048 reader evidence: `timed_clause_admission_retains_one_anchor_occurrence_and_operation_urgency`, `timed_clause_published_vectors_match_reader` and `timed_clause_order_unlisted_join_and_model_wide_stages_are_distinct` cover positive retention and CL-02/03/25/54 first refusals. A guard, reset or parameter constraint joins its `operation` to an `operation_anchor` of the root model with no execution point and retains the anchor's declaration occurrence only once. `CL-02` frame, `CL-03` pointed anchor and `CL-25` valid foreign-model anchor refuse `invalid_package`/`timed-node-reference` at the operation target. The `CL-25` anchor, its frame and its `semantic_type` all belong to `Gadget`, so the state step admits them and the timed-model join refuses. `CL-54` retains the earlier half-rebound anchor: its `context` and `semantic_type` name the foreign model but its frame remains on the first model. It refuses `invalid_model_binding`/`malformed-declaration` at `/semantic_graph/nodes/70` in the state step, before the timed-model join. `CL-03` is reader-level until STD-555 admits the point carrier in the V2 schema; it is not counted as a schema-valid vector. An urgency of kind `operation` retains its model-plus-name form. | Test |
| FR-038-AC-361 | IMPLEMENTED; focused TC-048 reader evidence: `timed_clause_published_vectors_match_reader` checks CL-01 and CL-07..15/21..24 against exact outcomes. Per-node joins enforce a place clause's owner type, slot count and selector-domain types; reset operation frame write, binder membership/order and Boolean condition; parameter declaration and `Clock` type; guard-part constraint operation; Boolean guard and invariant predicates; and invariant upper-bound comparisons `lt` or `le`. `CL-07` through `CL-15`, `CL-21` through `CL-24` match QSpec's exact typed code, cause and locus, including `ill_typed` owner/slot refusals and `invalid_package`/`timed-node-reference` joins; `CL-01` remains admitted. | Test |
| FR-038-AC-362 | PLANNED/UNRUN (IR-1141). TC-048 reads QSpec's selected `timed-clause-node-vectors.json` and its published fixture directly from the selected QSpec checkout, applies `CL-01` through `CL-54`, compares exact admission or code, cause and RFC 6901 locus, and checks the order stage independently of the vector label. It checks the dense model and slot fixtures also carry the same closed forms and structural ids. Malformed wire and graph shape precede timed order, then unlisted-node, per-node join and model-wide checks; `CL-54` proves that a state-step anchor/frame mismatch precedes the timed-model join reached by the state-valid `CL-25`; a combined-defect control makes each adjacent precedence observable. No QSpec file is copied into IR. | Test |
| FR-038-AC-363 | PLANNED/UNRUN (IR-1141). The production V2 reader requires a non-empty top-level `name` and rejects `package_id`, `identity_preimage`, `identity_projection` and any other extra top-level member at that member. It exposes the exact name after admission. Equal names with unequal complete JSON packages remain distinguishable by structural comparison, with object member names and ordered array elements preserved. No content-derived package or graph-node id is computed or exposed; admitted structural node ids remain public. | Test (TC-048) |
| FR-038-AC-364 | PLANNED/UNRUN (IR-1141; QSpec FR-322-AC-272/273/274). The reader admits declared, derived, model-owned and imported nodes with QSpec FR-322 structural IDs `{package,path,ordinal}`. Declared paths equal `declaration.qualified_name`; derived ungrouped paths have `[node_tag,semantic_form,...operands]`; each declared group member keeps its own declaration path, so the three declared members of TC-946 K14 share one label but have distinct paths. An anonymous derived member uses the sole declared member's path when exactly one exists, or a producer-chosen path when zero or multiple exist; no grouped member uses an operand-derived path. Each structural `node_id.ordinal` is producer assigned and unique only within package and path. A K14 mutation assigning one declared member another member's path with a distinct valid ordinal refuses `invalid_package`/`declaration-nominal-mismatch` at the changed `node_id` (FR-038-AC-21), not for duplicate identity; mismatching one cyclic member's label or replacing one exact in-cycle target with a nonexistent structural ID retains its independent refusal. The group has only a shared non-empty string label and exact structural references; no group-position carrier exists. An otherwise valid ungrouped node with sole `(package,path)` and producer-assigned ordinal 7 admits; two otherwise valid ungrouped nodes sharing `(package,path)` with distinct producer-assigned ordinals 7 and 23 admit without a contiguity or sibling-order check. The published NO-05 sibling at 7 and NO-06 siblings at 3 then 2 before ordinal 0 admit; NO-07's repeated 0 and NO-08's repeated 4 refuse `invalid_semantic_graph` at the later duplicate `node_id`. An ordinal outside 0..9007199254740991 or not an integer refuses `malformed_wire` at the exact ordinal; a repeated model root refuses at its later `owner`. The reader derives no node's structural ordinal from graph position or body and re-derives no node ID from a body. A valid group permutation rewrites index segments to the new graph positions while retaining resolved keys and references; the former group-position change clause is withdrawn in place. | Test (TC-048) |
| FR-038-AC-365 | Retired with replacement FR-038-AC-376: the old oracle used inline nested node IDs in paths, which QSpec FR-322-AC-278 no longer admits. Its first-refusal and structural-order obligations continue under AC-376; this id is not reused. | Withdrawn |
| FR-038-AC-366 | PLANNED/UNRUN (IR-1141). Lock dependency selections are exactly `{identity}` in ascending UTF-8 identity order. Missing supply refuses `missing_import`/`missing-selection`; an extra `package_id` or `version` refuses `unknown_member`. The host supplies a package with matching `name`; a diamond over one identity admits equal complete JSON values and refuses unequal values `invalid_package`/`conflicting-definition` with both complete lock paths. The reader neither records a transitive closure row nor compares content-derived ids. | Test (TC-048) |
| FR-038-AC-367 | PLANNED/UNRUN (IR-1141). A selected domain document is joined by its model-root identity and every selected source by its exact `{authority,identity}` pair. Changing a selected document's declaration or a source body's meaning is checked by the applicable model, owner, body and graph rules; there is no stale-content refusal. The reader exposes no `CheckedSemanticId`, expected/actual digest pair, candidate-key helper or stale-node-key refusal. A valid edited package retains its name and its declared structural ids where their paths are unchanged. | Test (TC-048; TC-058) |
| FR-038-AC-368 | PLANNED/UNRUN (IR-1141). Every public V2 refusal carries a non-empty ordered `loci` list; its first entry is primary, and no public `path` or `related_path` field remains. A value refusal has one exact RFC 6901 pointer entry; a malformed JSON or non-canonical byte refusal has one entry with no pointer or source region. A graph-node `locus`, cross-document `document_pointer`, code and cause retain their separate values. A source region appears on a location only when that exact value has one admitted region. | Test (TC-048; TC-058) |
| FR-038-AC-369 | PLANNED/UNRUN (IR-1141). A repeated `lock.refinement_internals` row, a repeated spelling and a repeated dependency identity each report both concrete entry pointers as two `loci` in declaration order, with the first primary. An unequal dependency diamond reports both conflicting lock-selection pointers in declaration order. Every other refusal has one location entry; these changes preserve each refusal's code, cause and first-defect precedence. | Test (TC-048) |
| FR-038-AC-370 | PLANNED/UNRUN (IR-1164/IR-1165). For an admitted selected object with fields declared as `zeta` then `alpha`, `model_object_fields().fields()` returns `zeta`, `alpha`, not name order; `field("alpha")` selects that same second item and `field("absent")` is `None`. In a chain `Base` declaring `b`, `Mid` declaring `m`, and `Leaf` declaring `l`, the Leaf result is `b`, `m`, `l` with authentic declaring structural node IDs Base, Mid, Leaf respectively, matching the admitted graph nodes rather than the requested Leaf ID. Each returned name and `member_type` remains the effective field that the same retained table supplies to a field read. A name-sort or a declaration-ID copied from the requested subtype fails these observations. | Test (TC-227) |
| FR-038-AC-371 | PLANNED/UNRUN (IR-1164/IR-1165). A subtype redefining an inherited field retains one exposed field at the hidden field’s earliest declaration-order slot, with the redefiner’s name, derived member type and actual redefining owner’s structural node ID; the hidden owner’s ID is not returned. A diamond that reaches one inherited field twice returns it once at the first inherited slot with its original declaring ID. Reversing two authored supertype edges reverses distinct inherited field order but does not change their declaring IDs. The object’s accessor result agrees with the retained `ordered_exposed` table and its binder order; a name-sort, duplicate diamond field, hidden base identity, or later-slot redefiner fails. | Test (TC-227) |
| FR-038-AC-372 | PLANNED/UNRUN (IR-1164/IR-1165). A selected document whose `Leaf` inherits `Base.b` admits with only the Leaf object declaration node in the graph, as QSpec FR-322-AC-30 permits; `model_object_fields(Leaf)` returns `MissingDeclaringNode` carrying the selected IR field identity of `Base.b` for the entire call, without inventing a Base ID or omitting `b`. Adding the authentic Base declaration node makes the call succeed with Base’s structural node ID. Two distinct admitted `model`/`object_type` nodes matching the same selected owner declaration identity but bearing distinct structural ordinals yield `AmbiguousDeclaringNode` for the first exposed field they own; neither node is chosen. With distinct defects, the first field in effective declaration order decides; `UnknownNode`, `NotModelObjectType` and existing `AmbiguousField` retain their earlier precedence. Malformed or unselected owners retain their independent admission refusal. | Test (TC-227) |
| FR-038-AC-373 | PLANNED/UNRUN (IR-1164/IR-1165). A public API fixture calls `declaring_node_id()` on every successful `CheckedModelField` and exhaustively matches the closed `CheckedModelFieldsError` variants, including `MissingDeclaringNode` and `AmbiguousDeclaringNode`; it cannot construct a `CheckedModelField` with a fabricated ID through public fields. The accessor returns equal results on equal calls and a package clone, charges no new work, performs no second model-resolution walk and does not read raw graph JSON or a copied domain schema. The revised declaration-order and typed-identity controls supersede only AC-136’s former ascending-name oracle; AC-136’s field set and type obligations remain. | Test (TC-227) |
| FR-038-AC-401 | The strict reader admits QSpec `CQ-W01` through `CQ-W04`: decimal and integer carried-quantity nodes are anonymous, self typed and of family `quantity`; the integer carrier with `exact` and with `nearest-even` produces distinct structural nodes. The public `ScalarTypeForm::CarriedQuantity` and admitted body expose the exact unit, carrier and rounding without evaluating a quantity. | Test (TC-048; QSpec TC-280) |
| FR-038-AC-402 | The strict reader refuses `CQ-W05` through `CQ-W08` as `invalid_package`/`invalid-derived-shape` at the carried node's `node_id`: a dimension in the unit slot (`RM-CQ-UNIT`), a rational or unbounded decimal carrier (`RM-CQ-CARRIER`), and a decimal carrier whose rounding differs (`RM-CQ-MODE`). Additional missing, extra or reordered bindings and a path/body mismatch receive the same typed refusal after applicable earlier wire or graph checks. No refused node yields an admitted public type. | Test (TC-048; QSpec TC-280) |
| FR-038-AC-403 | The strict reader admits `CQ-W09`, `CQ-W10` and `CQ-W15`: conversion from a unit-typed quantity or a carried quantity into a carried result of the same dimension with the exact result-pinned mode. It refuses `CQ-W11` and `CQ-W12` as `invalid_package`/`operation-mode-type-mismatch` at the application for a differing or null mode, `CQ-W13` as `ill_typed`/`operator-ineligible` there for a different dimension, and `CQ-W14` with that code and cause for a carried operand of `quantity.add`. Other quantity arithmetic, comparison and `collection.sum.quantity` also reject a carried operand. | Test (TC-048; QSpec TC-280) |
| FR-038-AC-404 | TC-048 reads `proposals/checked-package-v2/quantity-carrier-vectors.json` from the selected QSpec checkout by reference and applies `CQ-W01` through `CQ-W15` to complete IR package reads, checking admission or exact typed code, cause and primary node or application locus. The test stores no copy of the upstream vector or schema. | Test (TC-048) |
| FR-038-AC-405 | `CQ-E01` through `CQ-E20` are referenced as downstream evaluation controls only: the IR strict reader admits their valid carried type and conversion wire independently of the evaluator's completed, refused or incomplete outcome. The reader never emits `inexact_conversion`, `decimal_out_of_domain` or `integer_out_of_domain` as a package-read result for a valid conversion application. | Test (TC-048; QSpec TC-187) |
| FR-038-AC-406 | planned until the IR-1226 reader lands. The strict reader admits QSpec STD-640 positive vectors OEC-01..07 by reference, including each of the four effect kinds, an intentionally empty effect, multiple own operations and equal member names under distinct selected documents or declaring objects. A present row exposes the exact producer frame entries and provenance through a borrowed typed read; an absent optional carrier yields no effect rows and a typed unavailable-witness outcome for an operation-effect request, never an inferred empty effect. The canonical writer emits the same top-level member from optional borrowed typed input rows and omits it for `None`; a raw input row alone cannot mint an admitted view. | Test |
| FR-038-AC-407 | planned until the IR-1226 reader lands. The strict reader applies QSpec TC-977's OES-01..14 schema refusals and OER-01..26 semantic refusals to complete selected packages, checking each published code, cause and primary RFC 6901 locus. An explicitly `null`, empty or non-array `operation_effects` refuses `malformed_wire` at `/operation_effects`, unlike omission. It refuses missing/extra row or effect members, wrong entry kinds or provenance, foreign/wrong-kind declaring nodes, undeclared or inherited-only operations, dropped/invented effect entries, repeated or descending keys and missing coverage before admission, in the published precedence. | Test |
| FR-038-AC-408 | planned until the IR-1226 reader lands. Each visited carrier row and effect entry consumes one validation visit before inspection; a limit one below the required work returns incomplete at the first uncharged locus without a partial effect view. Equal admitted packages and their clones yield equal borrowed row data or equal typed outcomes. TC-048 reads the QSpec STD-640 vector file through the selected checkout and holds no local vector or schema copy. | Test |

FR-038-AC-66 is retired and its ID is not reused (ADR-0056). It required that every
application of operator class `case`, `temporal_formula` or `temporal_fairness` be
refused `unsupported_construct`/`expression-form`, which IR-549 lifts: those classes
are admitted at their node roots (FR-038-AC-96 through FR-038-AC-100) and the
reader does not blanket-refuse those temporal/case forms (FR-038-AC-101).
IR-661's planned relationship-navigation-only refusal does not reinstate AC-66.


The timed-slot criteria FR-038-AC-335 through FR-038-AC-340 consume QSpec [FR-322](https://github.com/agent-ix/quire-specification/blob/main/spec/objects/interfaces/FR-322-checked-package-artifact.md), AC-148..150 and AC-155..157. Owner [ruling #485](https://github.com/agent-ix/quire-specification/pull/485) (`81078f14`) withdrew indexed presence (PR-02); the empty-guard-root control is GD-01 rather than withdrawn SL-08. IR #775 supplies public-reader qualification of all 19 currently published SL/SR/PR/GD cases and their earlier-stage controls.

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
encoder that "Canonical encoding of the wire types" consumes (FR-038-AC-74 through
FR-038-AC-80, FR-038-AC-89 through FR-038-AC-95 and FR-038-AC-109 through FR-038-AC-111), including its
`Encode` for `serde_json::Value` (quire-canonical #7, merged).
[FR-040](./FR-040-admit-frame-entries-and-state-clauses.md) owns the
QSpec FR-340 `modifies` entry shape, scope and `deletes_scoped`, the FR-341 state clause and FR-342
operation anchor bodies, and the `model` form set.
