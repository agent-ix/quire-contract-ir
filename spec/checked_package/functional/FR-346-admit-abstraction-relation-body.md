---
id: FR-346
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
# FR-346: Admit the V2 abstraction relation body

## Description

When the `quire.checked-package/v2` reader ([FR-038](./FR-038-consume-checked-package-v2.md))
reads a `correspondence` node of `semantic_form` `abstraction_relation`, the
reader shall admit that node's body exactly as QSpec FR-451 defines it. The
reader shall refuse a body whose joins do not hold with the refusal FR-451
fixes for that join, in FR-451's reader order. QSpec FR-451 here is the
checked-package abstraction relation body,
`spec/objects/interfaces/FR-451-checked-package-abstraction-relation-body.md`;
the relation's meaning is QSpec FR-353 and its source spelling QSpec FR-450.
The QSpec text and the published `proposals/checked-package-v2/schema.json`
(`AbstractionRelationBody`, `CorrespondenceNode`) are normative. Where FR-451
fixes no code, cause or locus, this requirement states the reading Contract IR
takes and marks it "IR reading": FR-451 gives no code for a schema failure and
no locus for any refusal, so every code, cause and locus below that FR-451 does
not state is an IR reading, taken from FR-038 and FR-040. Contract IR reads the
relation from this node and from nothing else, and reads nothing from QSL.

## Inputs

Untrusted V2 package bytes and the evidence FR-038 already takes, including
each selected domain package document the model-owned member resolution
reads ([FR-038](./FR-038-consume-checked-package-v2.md) "Model-owned
members").

## Outputs

An admitted package whose abstraction relation nodes expose their object,
population and frame bindings, each with its key and value; or one typed
refusal with its code, cause, RFC 6901 path and locus. No partial package.

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
FR-451's and FR-450's ("Rust spellings"). A `RustPath` is an array of one or
more strings. Every member of every shape above is required, and each shape is
closed: a member outside it is refused.

The body is flat: it holds node keys, identifiers and Rust strings and no
application, aggregate, binding or literal term. The reader validates it
against this shape alone, as it validates a frame body (FR-040), and not
against the semantic-term grammar; FR-038 "Frame bodies" and "The flat wire"
are amended to say so, as merged QSpec FR-322 "Body grammar" lists the
`abstraction_relation` body among the body forms.

IR reading (FR-451 gives no code, cause or locus for a schema failure): a body,
entry or member of the wrong shape (a missing member, an unknown member, a
`RustPath` that is an empty array, a wrong member type, a `term` that is not
`abstraction_relation` and is not an application) refuses as
`invalid_semantic_graph` with no cause, as FR-040 maps its schema-level
cases, with the path of the offending entry (an array element of `objects`,
`fields`, `populations`, `frames` or `parameters`, for example
`/semantic_graph/nodes/{n}/body/objects/{i}`) or, for a body shape, the path
`/semantic_graph/nodes/{n}/body`. An occurrence whose role is not
`declaration` refuses the same way at that occurrence's `role`. An
`application` term standing as the body root of this node refuses
`ill_typed`/`operator-ineligible` at the node, as FR-038-AC-115 refuses an
application at the body root of a node its class does not place there. FR-038
decides body-root placement in several steps (the temporal classes in the
temporal step, `quire.op.state.clause` in the state step, the other classes in
the operation step), and the abstraction step runs before the operation step,
so this requirement states the stage: for any class, the root is refused in the
first of those steps that reaches it, and otherwise by the shape check that
opens the abstraction step, before its identity check, with the same code and
locus (IR reading). A non-`case` `application` standing as a member value inside the body
refuses `malformed_wire` at that application at strict wire validation, as
FR-038-AC-114 refuses a nested application in any node body, and a nested `case`
application refuses `ill_typed`/`operator-ineligible` at its `operator`, as
FR-038-AC-115 states. A node of any other `semantic_form` whose body
carries this shape refuses `invalid_semantic_graph` at its `body`, as FR-038
does for a frame body (IR reading, pending a QSpec ruling: merged FR-451 states
no refusal for it).

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

IR reading (pending a QSpec reconciliation of FR-322 and FR-451): merged
FR-322 refuses a node whose retained key differs from its preimage digest as
`invalid_package`/`stale-node-key` before any declaration, frame or operation
refusal, and FR-451 refuses a stale `node_id` of this node as
`invalid_semantic_graph` at the abstraction step. The reader follows FR-451:
the stale-key stage of FR-038 does not re-derive this node's key, so a stale
`node_id` refuses in step 1 below, after the frame and state steps.

### Canonical order

`objects` is sorted ascending by `type` digest, `populations` by `population`
digest, and `frames` by `context` digest and then `operation` as UTF-8 bytes,
each digest compared as unsigned bytes. `fields` and `parameters` are sorted
ascending by `name` as UTF-8 bytes. An array out of order refuses as
`invalid_semantic_graph` (FR-451), located, as an IR reading, at the array
(`/semantic_graph/nodes/{n}/body/objects`, `.../objects/{i}/fields`, and so
on). IR reading: order is non-strict, so two entries with an equal order key
are in order and the member and uniqueness checks below refuse them.

### Reader order

IR reading (FR-451 says "after the state step" only): the abstraction step runs
after the state step of FR-040 and after the temporal step of FR-038-AC-102
and before the operation step, so the steps run in the order frame, state,
temporal, abstraction, operation. It runs over every
`correspondence`/`abstraction_relation` node in ascending `node_id` digest
order. For one node the checks run in this order and the first failing check
is its defect:

1. **Identity.** `node_id` is the digest of its preimage; `semantic_type`
   equals `node_id`; `dependencies` equal the body's targets; each failure is
   `invalid_semantic_graph`, located (IR reading) at the node,
   `/semantic_graph/nodes/{n}`.
2. **Order.** The canonical order holds.
3. **Targets**, entry by entry in body order (`objects`, `populations`,
   `frames`), located at the target's node key: a target that is not a
   declared dependency refuses as `missing_declaration`/`missing-name`; a
   `type` or `context` target that is not a `model`/`object_type` node, or a
   `population` target that is not a `relation`/`population` node, refuses as
   `invalid_model_binding`/`malformed-declaration`; an owner the reader cannot
   recover refuses with the owner recovery's own refusal (FR-038 "Model-owned
   members", step 2: `missing_declaration`/`missing-selection` or
   `invalid_package`/`stale-node-key`). IR reading: step 1 requires
   `dependencies` to equal the body's targets, so after step 1 every target is
   a declared dependency and the missing-name branch has no input; FR-451-AC-3
   (`invalid_semantic_graph` for `dependencies` that omit a body target) and
   FR-451-AC-4 (`missing_declaration`/`missing-name` for a target missing from
   `dependencies`) name the two outcomes for one input, and this
   reader follows the reader order: the identity step refuses it.
4. **Members**, entry by entry in the same order. The entry of a refusal is
   the innermost array element holding the offending member (an `objects`,
   `fields`, `populations`, `frames` or `parameters` element), and its locus is
   the entry's path (IR reading).
   - Two `fields` entries of one object entry with equal `name` refuse as
     `invalid_model_binding`/`conflicting-binding` at the second `fields`
     entry.
   - Otherwise, a `fields` name that is not an effective field member of the
     object type refuses as `invalid_model_binding`/`malformed-declaration`
     (FR-451 step 4); IR reading: a name that two supertypes both expose
     refuses as `ambiguous_declaration`/`ambiguous-name`, as FR-322 step 3
     gives it.
   - A frame `operation` that does not resolve is refused with FR-342
     "Operation resolution"'s own refusals, as FR-451 states: an undeclared
     name, or the name of a field, as `missing_declaration`/`missing-name`; a
     name two or more exposed operations carry as
     `ambiguous_declaration`/`ambiguous-name`; an operation the `context` only
     inherits as `invalid_model_binding`/`malformed-declaration`.
   - A `parameters` list that is not exactly the operation's declared
     parameters, two `rust_parameter` values that are equal, a `rust_parameter`
     equal to the `receiver`, and a Rust string not of the syntax FR-450 gives
     it (a keyword segment such as `crate` included) each refuse as
     `invalid_model_binding`/`malformed-declaration`.
   - Names resolve only when the target is a model declaration node, as FR-040
     scopes field and operation resolution.

Once every node's own checks hold, the reader checks key uniqueness over all
`correspondence`/`abstraction_relation` nodes of the package, in ascending
`node_id` digest order and body order: the second entry to claim an object
key, a population key or an operation key refuses as
`invalid_model_binding`/`conflicting-binding` (FR-451). The two entries may
stand in one node or in two. IR reading: a refusal carries one path and one
locus (FR-038 "Locating refusals"), so the refusal's path is the second
entry's, its locus is the key's `type`, `population` or `context` node key,
and its message names the key; the first entry is not carried. FR-451 says the
refusal names both entries; carrying the first needs a refusal member this
repository does not have, and is a QSpec question. The same holds for the
field name of a duplicate `fields` entry.

### Lowering

FR-038-AC-6 and FR-035 require every admitted node family to lower with exact
source, type and dependency correspondence, so an admitted
`correspondence`/`abstraction_relation` node reaches lowering. What it lowers
to, and any refusal record for an unbound model element, are not stated by
merged QSpec FR-451 and are an open question; this requirement does not decide
them. The `correspondence` arms in `v2/lower.rs` and `v2/mod.rs` take the new
form under the exhaustive-match rule as IR-509's code, and IR-509 does not pick
a lowering outcome without a ruling. QSpec FR-353 (FR-353-AC-3) places the
unbound-element refusal at emission onto implementation code, which this
reader does not do.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-346-AC-1 | 🚧 A package holding one `correspondence`/`abstraction_relation` node over a domain package built here (`ConfigVersion` with the field `version`, the operation `attemptUpdate(next: Integer)` and the operation `rebase(from: Integer, to: Integer)`, and a subtype `ConfigVersionDraft` that only inherits `attemptUpdate`) admits, for the body `{term: "abstraction_relation", objects: [{type: ConfigVersion, rust_type: ["config_store", "ConfigVersion"], fields: [{name: "version", rust_field: "version"}]}], populations: [], frames: [{context: ConfigVersion, operation: "attemptUpdate", function: ["config_store", "attempt_update"], receiver: "self", parameters: [{name: "next", rust_parameter: "next"}]}]}`; the body with all three arrays empty admits; a tuple-field `rust_field` of `"0"` and a raw identifier `r#type` admit. An unrecognized `correspondence` form still refuses `invalid_semantic_graph` at `semantic_form`. | Test (TC-225) |
| FR-346-AC-2 | 🚧 IR reading: a body missing `frames` (path `/semantic_graph/nodes/{n}/body`), a body with a fourth member (same path), an `objects` entry missing `fields` (`.../body/objects/0`), a frame entry missing `receiver` (`.../body/frames/0`), a `function` that is an empty array (`.../body/frames/0`), a `type` that is not a node key (`.../body/objects/0`) and a `term` of `"aggregate"` each refuse as `invalid_semantic_graph` with no cause; an occurrence of role `anchor` refuses the same way at `/semantic_graph/nodes/{n}/occurrences/{o}/role`; a node of another `semantic_form` carrying this body refuses the same way at its `body`. An `application` body root of an ordinary class refuses `ill_typed`/`operator-ineligible` at the node `/semantic_graph/nodes/{n}` from the shape check that opens the abstraction step, a `temporal_formula` application as the root refuses the same way from the temporal step, and a `quire.op.state.clause` application as the root refuses the same way from the state step; a non-`case` `application` standing as a `type` value refuses `malformed_wire` at that application, and a `case` application standing there refuses `ill_typed`/`operator-ineligible` at its `operator`. | Test (TC-225) |
| FR-346-AC-3 | 🚧 A frame entry for (`ConfigVersion`, `attemptUpdate`) admits in a package that holds no `state`/`frame` node for that operation; in a package that holds one, the entry's pair equals that frame's `operation_anchor` pair. | Test (TC-225) |
| FR-346-AC-4 | 🚧 A node whose `node_id` is not the digest of `{version: "quire.abstraction-relation-node/v1", body}`, a node whose `semantic_type` is not its `node_id`, and a node whose `dependencies` omit a body target or hold a target the body does not each refuse as `invalid_semantic_graph` at the node `/semantic_graph/nodes/{n}` (IR reading); the dependency case is the identity step's, and no `missing_declaration`/`missing-name` reaches it (FR-451-AC-3 against FR-451-AC-4, IR reading). A stale `node_id` refuses in this step and not as `invalid_package`/`stale-node-key`. A node whose `objects` array is out of `type` digest order refuses as `invalid_semantic_graph` at `/semantic_graph/nodes/{n}/body/objects` (IR reading); so do `frames` holding two entries of one `context` in descending `operation` order, `fields` out of `name` order and `parameters` out of `name` order, at their arrays. | Test (TC-225) |
| FR-346-AC-5 | 🚧 A `type` target naming a `relation`/`population` node and a `population` target naming an object type each refuse as `invalid_model_binding`/`malformed-declaration` at the target's node key; a `context` naming a `state`/`frame` node refuses the same way; a `context` keyed under an unselected domain package refuses as `missing_declaration`/`missing-selection`. | Test (TC-225) |
| FR-346-AC-6 | 🚧 A `fields` name `ConfigVersion` does not expose, a `rust_field` of `"not a name"`, a `function` segment `"crate"` and a `rust_type` segment `"9lives"` each refuse as `invalid_model_binding`/`malformed-declaration` at the entry; two `fields` entries named `version`, bound to different Rust fields, refuse as `invalid_model_binding`/`conflicting-binding` at the second `fields` entry; a `fields` name two supertypes both expose refuses as `ambiguous_declaration`/`ambiguous-name` (IR reading). | Test (TC-225) |
| FR-346-AC-7 | 🚧 A frame entry for (`ConfigVersion`, `noSuchOperation`) and one for (`ConfigVersion`, `version`) refuse as `missing_declaration`/`missing-name`; an operation name two supertypes of the `context` both carry refuses as `ambiguous_declaration`/`ambiguous-name`; (`ConfigVersionDraft`, `attemptUpdate`), an operation the context only inherits, refuses as `invalid_model_binding`/`malformed-declaration`; a `parameters` list missing `next` for `attemptUpdate` refuses as `invalid_model_binding`/`malformed-declaration` at the frame entry. For `rebase(from, to)`, a `parameters` list that is exactly `from` and `to` with `rust_parameter` values `"a"` and `"a"` refuses `invalid_model_binding`/`malformed-declaration` at the second `parameters` entry, and for `attemptUpdate` a `receiver` of `"next"` with `rust_parameter` `"next"` refuses the same way, each case holding the declared parameters exactly. | Test (TC-225) |
| FR-346-AC-8 | 🚧 Two entries that bind the object key `ConfigVersion`, in one node or in two, refuse as `invalid_model_binding`/`conflicting-binding` at the second entry in ascending `node_id` digest order and body order, with the key's node key as locus and the first entry not carried (IR reading; FR-451-AC-5 says both entries); the same holds for two entries on one population key and two frame entries on one (`context`, `operation`) pair. | Test (TC-225) |
| FR-346-AC-9 | 🚧 A package carrying a state-step defect and an abstraction defect reports the state defect; one carrying an abstraction defect and an operation-step defect reports the abstraction defect (IR reading); one carrying a temporal placement defect and an abstraction defect reports the temporal defect, and one carrying an ordinary-class application as an abstraction node's body root and an operation-step defect reports the body-root refusal (IR reading); one carrying a stale abstraction `node_id` and a frame defect reports the frame defect; one node carrying an identity defect and a target defect reports the identity defect, a target defect and a member defect the target defect, and an order defect and a target defect the order defect; two defective nodes report the lower `node_id` node's own defect; nodes that each hold only a key collision with another report it only after every node's own checks held. | Test (TC-225) |
| FR-346-AC-10 | 🚧 Changing one `rust_field` in an admitted body and recomputing `node_id` gives another `node_id` and, through the `identity_projection`, another `package_id`; the same body gives the same `node_id` on every read; the same change without recomputing `node_id` refuses as in FR-346-AC-4. | Test (TC-225) |

## Dependencies

[FR-038](./FR-038-consume-checked-package-v2.md) owns the reader, its refusal
codes and pointers, the closed vocabularies, the flat wire and the model-owned
member resolution this requirement reuses. [FR-040](./FR-040-admit-frame-entries-and-state-clauses.md)
owns the state step this step follows and the operation resolution frame
entries share. QSpec FR-451 owns the normative body and refusal order, FR-353
the relation's meaning and FR-450 the source form and Rust spellings; FR-322
owns the graph, node keys and the `identity_projection`. A generator reads the
relation from this node alone, and no part of this reader depends on QSL.

Questions for QSpec: (1) FR-451-AC-3 against FR-451-AC-4 give one input
(`dependencies` omitting a body target) two refusals. (2) Whether the stale-key
stage of FR-322 or FR-451's identity step refuses a stale abstraction
`node_id`. (3) The codes and loci of schema failures and of the identity,
order and member refusals. (4) How a refusal names two entries. (5) The place
of the abstraction step against the operation step. (6) The code of an
ambiguous `fields` name. (7) What an application body root or a body of another
form carrying this shape refuses. (8) The lowering outcome and the refusal
record for an unbound element.
