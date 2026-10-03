---
id: FR-345
title: "Admit the V2 abstraction relation body"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-040
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-451
    type: references
  - target: ix://agent-ix/quire-specification/FR-353
    type: references
  - target: ix://agent-ix/quire-specification/FR-450
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
  - target: ix://agent-ix/quire-specification/FR-341
    type: references
  - target: ix://agent-ix/quire-specification/FR-342
    type: references
---
# FR-345: Admit the V2 abstraction relation body

## Description

When the `quire.checked-package/v2` reader ([FR-038](./FR-038-consume-checked-package-v2.md))
reads a `correspondence` node of `semantic_form` `abstraction_relation`, the
reader shall admit that node's body exactly as QSpec FR-451 defines it. The
reader shall refuse every other body of that node with the refusal FR-451
fixes, in its reader order. QSpec FR-451 here is the checked-package abstraction
relation body,
`spec/objects/interfaces/FR-451-checked-package-abstraction-relation-body.md`;
the relation's meaning is QSpec FR-353 and its source spelling QSpec FR-450.
The QSpec text and the published `proposals/checked-package-v2/schema.json`
(`AbstractionRelationBody`, `CorrespondenceNode`) are normative; this
requirement states what the Contract IR reader does with them and adds no
shape of its own. Contract IR reads the relation from this node and from
nothing else, and reads nothing from QSL.

## Inputs

Untrusted V2 package bytes and the evidence FR-038 already takes, including
each selected domain package document the model-owned member resolution
reads ([FR-038](./FR-038-consume-checked-package-v2.md) "Model-owned
members").

## Outputs

An admitted package whose abstraction relation nodes expose their object,
population and frame bindings, each with its key and value; or one typed
refusal with its cause and locus. No partial package.

## Behavior

### Form gate

The `correspondence` family's closed form enum, decoded once at intake
([FR-038](./FR-038-consume-checked-package-v2.md) "Closed vocabularies are
decoded once"), holds `source_locus`, `model_correspondence`, `binding_role`
and `profile_correspondence` today. QSpec's `CorrespondenceNode` adds a fifth
member, `abstraction_relation`, so this requirement adds it: the reader shall
decode `abstraction_relation` as a `correspondence` form, matching it
exhaustively with no catch-all arm, and an unrecognized `correspondence` form
still refuses as `invalid_semantic_graph` at the node's `semantic_form`
(FR-344). The node tag `correspondence` is already one of the thirteen
admitted tags and needs no change.

### Body shape

A `correspondence`/`abstraction_relation` node's `body` is
`{"term": "abstraction_relation", "objects", "populations", "frames"}`, all
three members required, each an array that may be empty:

- an `objects` entry is `{type, rust_type, fields}`, `type` a node key,
  `rust_type` a `RustPath`, and `fields` an array of `{name, rust_field}`;
- a `populations` entry is `{population, collection}`, `population` a node
  key and `collection` a `RustPath`;
- a `frames` entry is `{context, operation, function, receiver, parameters}`,
  `context` a node key, `operation` an identifier, `function` a `RustPath`,
  `receiver` a `RustReceiver`, and `parameters` an array of
  `{name, rust_parameter}`.

`RustPath`, `RustField`, `RustReceiver` and the Rust spellings are QSpec
FR-451's and FR-450's. A `RustPath` is an array of one or more strings. Every
member of every shape above is required, and each shape is closed: a member
outside it is an unknown member. The body is flat: it holds node keys,
identifiers and Rust strings and no application, aggregate, binding or
literal term, so it has no nesting for the flat wire of
[FR-038](./FR-038-consume-checked-package-v2.md) "The flat wire" to bound, and
the reader validates it against this shape alone, as it validates a frame body
(FR-040), not against the semantic-term grammar.

A body, entry or member of the wrong shape (a missing member, an unknown
member, a `RustPath` that is an empty array, a wrong member type, a `term`
other than `abstraction_relation`) refuses as `invalid_semantic_graph` with no
cause, located at the offending entry, or at the node's `body` for a body
shape, as FR-040 locates the schema-level cases; so does an occurrence whose
role is not `declaration`, located at that occurrence's `role`. A node of any
other `semantic_form` whose body carries this shape refuses the same way at
its `body`, as FR-038 does for a frame body (an IR reading, pending a QSpec
ruling: merged FR-451 states no refusal for it).

### Keys

An object entry's key is its `type` target; a population entry's key is its
`population` target; a frame entry's key is its `context` target and its
`operation` identifier together, the pair a `state`/`operation_anchor` node
binds (FR-040). A frame entry carries no frame or anchor identity and binds
its operation whether or not the package holds the `state`/`frame` or
`state`/`operation_anchor` node for the pair.

### Identity

The node's `node_id` is the SHA-256 of the RFC 8785 canonical bytes of
`{"version": "quire.abstraction-relation-node/v1", "body": <body>}`, computed
through `quire-canonical` as every identity digest is (FR-038 "Every identity
digest is computed through quire-canonical"). The node's `semantic_type` is
its own `node_id`. Its `dependencies` are exactly the unique,
digest-ascending set of every `type`, `population` and `context` target of the
body. Every binding therefore enters the node key and, through the
`identity_projection`, the `package_id`.

### Canonical order

`objects` is sorted ascending by `type` digest, `populations` by `population`
digest, and `frames` by `context` digest and then `operation` as UTF-8 bytes,
each digest compared as unsigned bytes. `fields` and `parameters` are sorted
ascending by `name` as UTF-8 bytes. An array out of order refuses as
`invalid_semantic_graph`, located at the array. Order is non-strict: two
entries with an equal order key are in order, and the member step and the
uniqueness step below refuse them.

### Reader order

The abstraction step runs after the state step of FR-040, over every
`correspondence`/`abstraction_relation` node in ascending `node_id` digest
order. For one node the checks run in this order and the first failing check
is its defect:

1. **Identity.** `node_id` is the digest of its preimage; `semantic_type`
   equals `node_id`; `dependencies` equal the body's targets; each failure is
   `invalid_semantic_graph` located at the node.
2. **Order.** The canonical order holds.
3. **Targets**, entry by entry in body order (`objects`, `populations`,
   `frames`), located at the target's node key: a target that is not a
   declared dependency refuses as `missing_declaration`/`missing-name`; a
   `type` or `context` target that is not a `model`/`object_type` node, or a
   `population` target that is not a `relation`/`population` node, refuses as
   `invalid_model_binding`/`malformed-declaration`; an owner the reader cannot
   recover refuses with the owner recovery's own refusal (FR-038 "Model-owned
   members", step 2).
4. **Members**, entry by entry in the same order, located at the entry unless
   a resolution refusal fixes its own locus: two `fields` entries of one
   object entry with equal `name` refuse as `invalid_model_binding`/
   `conflicting-binding`, naming both entries and the name. Otherwise, a
   `fields` name that is not an effective field member of the object type, a
   frame `operation` that does not resolve as in FR-040's operation anchor
   resolution (an operation the context only inherits included), a
   `parameters` list that is not exactly the operation's declared parameters,
   two equal `rust_parameter` values, a `rust_parameter` equal to the
   `receiver`, and a Rust string not of the syntax FR-450 gives it each
   refuse as `invalid_model_binding`/`malformed-declaration`; an unrecovered
   owner or an ambiguous name keeps the code FR-040's resolution gives it.
   Names resolve only when the target is a model declaration node, as FR-040
   scopes field and operation resolution.

Once every node's own checks hold, the reader checks key uniqueness over all
`correspondence`/`abstraction_relation` nodes of the package, in ascending
`node_id` digest order and body order: the second entry to claim an object
key, a population key or an operation key refuses as
`invalid_model_binding`/`conflicting-binding`, naming both entries and the
key, located at the second entry. The two entries may stand in one node or in
two.

Lowering an admitted abstraction relation node, and any refusal record a
lowering gives an unbound model element, are not stated here: merged QSpec
FR-451 and FR-353 place the unbound-element refusal at emission onto
implementation code, which this reader does not do.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-345-AC-1 | 🚧 A package holding one `correspondence`/`abstraction_relation` node over a domain package built here (`ConfigVersion` with the field `version` and the operation `attemptUpdate(next: Integer)`) admits, for the body `{term: "abstraction_relation", objects: [{type: ConfigVersion, rust_type: ["crate", "ConfigVersion"], fields: [{name: "version", rust_field: "version"}]}], populations: [], frames: [{context: ConfigVersion, operation: "attemptUpdate", function: ["crate", "attempt_update"], receiver: "self", parameters: [{name: "next", rust_parameter: "next"}]}]}`; the body with all three arrays empty admits; a tuple-field `rust_field` of `"0"` admits; an unrecognized `correspondence` form still refuses `invalid_semantic_graph` at `semantic_form`. | Test (TC-224) |
| FR-345-AC-2 | 🚧 A body missing `frames`, a body with a fourth member, an `objects` entry missing `fields`, a frame entry missing `receiver`, a `function` that is an empty array, a `type` that is not a node key and a `term` other than `abstraction_relation` each refuse as `invalid_semantic_graph` with no cause, located at the offending entry or, for a body shape, at the node's `body`; an occurrence of role `anchor` refuses the same way at its `role`; a node of another `semantic_form` carrying this body refuses the same way at its `body`. | Test (TC-224) |
| FR-345-AC-3 | 🚧 A frame entry for (`ConfigVersion`, `attemptUpdate`) admits in a package that holds no `state`/`frame` node for that operation; in a package that holds one, the entry's pair equals that frame's `operation_anchor` pair. | Test (TC-224) |
| FR-345-AC-4 | 🚧 A node whose `node_id` is not the digest of `{version: "quire.abstraction-relation-node/v1", body}`, a node whose `semantic_type` is not its `node_id`, a node whose `dependencies` omit a body target or hold a target the body does not, and a node whose `objects` array is out of `type` digest order each refuse as `invalid_semantic_graph`, the first three at the node and the last at the array. A body whose `frames` hold two entries of one `context` in descending `operation` order, one whose `fields` are out of `name` order and one whose `parameters` are out of `name` order refuse the same way. | Test (TC-224) |
| FR-345-AC-5 | 🚧 A `type` target naming a `relation`/`population` node and a `population` target naming an object type each refuse as `invalid_model_binding`/`malformed-declaration` at the target's node key; a target missing from `dependencies` refuses as `missing_declaration`/`missing-name` at the target; a `context` naming a `state`/`frame` node refuses as `invalid_model_binding`/`malformed-declaration`; a target keyed under an unselected domain package refuses as `missing_declaration`/`missing-selection`. | Test (TC-224) |
| FR-345-AC-6 | 🚧 A `fields` name that `ConfigVersion` does not expose refuses as `missing_declaration`/`missing-name`; an object entry holding two `fields` entries named `version`, bound to different Rust fields, refuses as `invalid_model_binding`/`conflicting-binding` naming both entries and `version`; a frame `operation` that `ConfigVersion` only inherits refuses as `invalid_model_binding`/`malformed-declaration`; a `parameters` list missing `next`, two `rust_parameter` values that are equal, a `rust_parameter` equal to a `receiver` of `"next"`, and a `rust_field` of `"not a name"` each refuse as `invalid_model_binding`/`malformed-declaration` at the entry. | Test (TC-224) |
| FR-345-AC-7 | 🚧 Two entries that bind the object key `ConfigVersion`, in one node or in two, refuse as `invalid_model_binding`/`conflicting-binding` naming both entries and the key, located at the second entry in ascending `node_id` digest order and body order; the same holds for two entries on one population key and two frame entries on one (`context`, `operation`) pair. | Test (TC-224) |
| FR-345-AC-8 | 🚧 A package carrying a state-step defect and an abstraction defect reports the state defect; one node carrying an identity defect and a target defect reports the identity defect, a target defect and a member defect reports the target defect, and an order defect and a target defect reports the order defect; two defective nodes report the lower `node_id` node's own defect; a package whose nodes each hold only a key collision with another node reports it only after every node's own checks held. | Test (TC-224) |
| FR-345-AC-9 | 🚧 Changing one `rust_field` in an admitted body and recomputing `node_id` gives another `node_id` and, through the `identity_projection`, another `package_id`; the same body gives the same `node_id` on every read; the same change without recomputing `node_id` refuses as in FR-345-AC-4. | Test (TC-224) |

## Dependencies

[FR-038](./FR-038-consume-checked-package-v2.md) owns the reader, its refusal
codes and pointers, the closed vocabularies, the flat wire and the model-owned
member resolution this requirement reuses. [FR-040](./FR-040-admit-frame-entries-and-state-clauses.md)
owns the state step this step follows and the operation resolution frame
entries share. QSpec FR-451 owns the normative body and refusal order, FR-353
the relation's meaning and FR-450 the source form and Rust spellings; FR-322
owns the graph, node keys and the `identity_projection`. A generator reads the
relation from this node alone, and no part of this reader depends on QSL.
