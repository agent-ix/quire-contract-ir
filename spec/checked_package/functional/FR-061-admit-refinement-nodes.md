---
id: FR-061
title: "Admit closed checked-package refinement nodes"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-375
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
---

# FR-061: Admit closed checked-package refinement nodes

## Description

When a V2 checked package carries a `relation`/`refinement` node, the
Contract IR reader SHALL admit the exact closed body, identity, graph joins
and occurrences published by QSpec FR-375, FR-322 and the checked-package
schemas, or refuse the whole package with a typed, located defect. The
canonical writer SHALL retain every admitted member. The reader SHALL keep
the family and body vocabularies closed; it does not infer missing rows or
interpret an unknown term as a refinement.

## Inputs

The untrusted `quire.checked-package/v2` bytes, directly selected dependency
packages, definition selections, model evidence and limits of
[FR-038](./FR-038-consume-checked-package-v2.md). The normative shapes are
`ix://agent-ix/quire-specification/FR-375` and
`ix://agent-ix/quire-specification/FR-322`, including the published
`proposals/checked-package-v2/schema.json` and
`schema.json` at the selected QSpec source.

## Outputs

An admitted package retaining the complete refinement node, its resolved
local and selected-dependency targets, identity and source occurrences; or a
typed refusal with its cause where defined and RFC 6901 pointer. No partial
package and no node silently omitted.

## Behavior

### Closed form and body

The reader SHALL add `refinement` to the closed `relation` form enum, and an
unrecognized `relation` form SHALL still refuse `invalid_semantic_graph` at
its `semantic_form`. A refinement body SHALL be the closed
`RefinementBody`: `term: "refinement"`, `abstract`, `profile`, and all six
arrays `populations`, `objects`, `steps`, `histories`, `assume`, `ensure`.
Every array is required, including when empty. `protocol` is present exactly
for a protocol abstract subject. `abstract_internal` is absent for an empty
list and otherwise a nonempty ascending array of alias-free class spellings,
permitted only with `protocol`. `class_catalog` is present exactly when either
selected subject contributes a protocol or memory step class, and is absent
for a model-only refinement with neither kind of class. No field outside
this body is admitted.

The row entries SHALL match QSpec's closed `RefinementPopulationEntry`,
`RefinementObjectEntry`, `RefinementStepEntry`,
`RefinementHistoryEntry`, `RefinementAssumeEntry` and
`RefinementEnsureEntry` shapes. A population pairs abstract and concrete
population keys. An object entry pairs abstract and concrete type keys and
sorted `{name, expression}` field rows. A step entry holds `source_class`,
`source_node` exactly when that class has a declaration node, and one of
four targets: `operation` with operation key, receiver and arguments;
`protocol_step` with class, optional node according to the class and
arguments; `stutter`; or `any`. The exact `{term: "open"}` value is admitted
only as a step argument for `_`, not as a general semantic term. A history
entry holds a concrete type, field name, field type, a literal or local
history-literal reference as `initial`, and ordered operation/expression
updates. `assume` uses a local operation key; `ensure` uses a refinement key;
both require `kind` `weak` or `strong` and `granularity` `each` or `whole`.
Composite expressions use their own node references; an application nested
beyond FR-322's body-term grammar is refused.

The public `RefinementMemoryClass` SHALL retain the closed source-default
memory class variant through typed read/write/read. A `memory drain` entry
has exactly `class: "memory drain"`, `memory_model` in `sc`, `tso`, `ra`,
and `drain: "store-buffer-flush"`. A `memory flush` entry has exactly
`class: "memory flush"` and `memory_model: "tso"`, with no `drain`.
Neither variant has `node` or `binder_types`.
The typed `drain` vocabulary SHALL contain only `store-buffer-flush`; the
reader and writer SHALL NOT omit, infer, or default it. For each source-default
`sc`, `tso`, or `ra` model selected by a source `parallel` in a subject, its
catalog SHALL include `memory drain`; a source-default `tso` also contributes
the separate `memory flush` class. A request-time memory override SHALL NOT
change either checked-package catalog or package name. The drain class
means that the thread's pending writes become visible according to the
selected model's visibility rule: `sc` is a no-op, `tso` flushes the store
buffer in FIFO order, and `ra` has no store-buffer entries to remove and
grants no additional visibility or ordering beyond QSpec FR-437. The
separate TSO flush class removes only the oldest buffered write for one
thread.

Each catalog array SHALL contain one entry per catalog key. A declared
class's key is its alias-free `class` spelling; a memory class's key is
(`class`, `memory_model`). The arrays SHALL ascend by the UTF-8 bytes of
`class`, then by `memory_model` bytes when the class spellings match. Thus
the three drain entries order `ra`, `sc`, `tso`, followed by TSO flush.
Two `parallel` controls selecting the same source-default model contribute
one memory entry for that model, not one entry per control.

### Keys, graph and identity

`RefinementKey` SHALL be a local `NodeRef` for a node in this graph, or the
closed `{package, node}` `PackageNodeKey` for a node of a directly selected
dependency package. The current package name SHALL NOT be embedded as a dependency target in its own
body. `abstract` names the selected abstract model's canonical `model_root`;
optional `protocol` names the
selected protocol clause; `profile` is the selected infinite-trace
`DefinitionRef`, never the source alias. Local row declaration and type keys
are `NodeRef`s; abstract-side keys can be `RefinementKey`s. The reader SHALL
resolve each target against the selected package and require its declared
kind and subject. An unresolved key SHALL refuse under FR-038's
`missing_declaration`/`missing-name` rules, and a wrong-kind target SHALL
refuse `invalid_model_binding`/`malformed-declaration`, at that key. A
schema-invalid key or row SHALL refuse before such a meaning join. No
unresolved row, key or expression is dropped.

The node's `semantic_type` SHALL be the unique canonical local `model_root`
`NodeRef` identity-bound to the selected concrete model package, regardless
of its object-type or state-machine declaration count. Its owner identity is
the refinement's concrete identity; no separate concrete-selection member
exists. Substituting an object-type or state-machine declaration for that
root SHALL refuse `invalid_model_binding`/`malformed-declaration` at
`semantic_type`. Replacing it with another selected model's canonical root
instead selects that
other concrete identity. After the earlier closed-wire, graph-shape, structural-id and
canonical-order stages, the reader SHALL first require a concrete catalog for
a protocol subject, refusing `invalid_semantic_graph` at
`/body/class_catalog` when absent; it SHALL then visit concrete row keys in
`populations`, `objects`, `steps`, `histories`, `assume` order and refuse the
first declaration or anchor owned by a different model as
`invalid_model_binding`/`malformed-declaration` at that key. The later
protocol graph step SHALL compare the concrete catalog with classes derived
from the new concrete identity in catalog-key order, refusing
`invalid_semantic_graph` at an extra class entry or at
`class_catalog/concrete` for an omitted class. If there is no concrete row
key, a distinct structural model subject with no catalog SHALL admit as a distinct node;
a protocol subject SHALL do so when its concrete catalog equals the classes
derived from the new identity, including two empty sets. No protocol-body
node is compared at the row-join stage. The `abstract` key
SHALL name the selected abstract model's canonical `model_root`, locally or
through its directly selected dependency package; `protocol`, when present,
SHALL name the exact clause of that abstract model. A model-root key SHALL
NOT be replaced by one of its package's object-type or state-machine
declarations. `dependencies` SHALL be exactly the unique, structurally ordered local
keys named by `abstract`, optional `protocol`, row declarations and types,
and expression reference targets and application member declarations and
populations, plus local `class_catalog` class nodes and binder type keys.
Local header keys remain dependencies when all arrays are empty. An external
`PackageNodeKey`, profile `DefinitionRef`, literal type,
application result type and standalone `semantic_type` are not dependencies
unless another listed body key names the same local node. A dependency
mismatch SHALL refuse `invalid_semantic_graph` at `/dependencies` in
FR-322's graph-shape stage, before refinement meaning joins.

The node SHALL carry its structural id under QSpec FR-322. The complete body and ordered rows SHALL be retained. A valid body edit can retain the same structural id; the reader checks catalog, dependency, type and row meaning independently of the id.

The `steps`
array SHALL retain the producer's source declaration order, including tied
protocol classes and memory steps. The reader and writer SHALL preserve the
received array order without
sorting it or refusing an otherwise valid permutation. A permutation that
changes the `steps` array changes the complete body used by the existing
structural body; a source change that leaves the emitted body unchanged leaves
those identities unchanged. The other five row arrays SHALL retain their
FR-375 order by first named declaration's node key and then member name.
Object field names and history update operation keys SHALL each ascend.

### Occurrences and reader order

Every refinement owner SHALL have a `declaration` occurrence at ordinal 0.
A proof-eligible owner—no hidden field, history field or `any` target—SHALL
also have one `generated` occurrence at each ordinal 0 through its number
of step rows, the initial obligation at 0 and step row `i` at `i + 1`.
An ineligible owner SHALL have no `generated` occurrence. Each generated
entry SHALL carry exactly the source region FR-322 assigns to its ordinal.
A duplicate key, extra role or ordinal, or wrong region SHALL refuse
`invalid_package`/`invalid-value` at the offending `source_map` entry under
FR-322's occurrence rules. After every present entry passes, a missing
required occurrence SHALL refuse that pair at `/source_map`. These occurrences stay
on the owner node; the reader SHALL NOT invent derived per-step nodes.

The reader SHALL apply FR-322's strict wire, graph-shape, structural-id, declaration
and source-map checks in their published order. The closed schema and body
shape SHALL be checked before refinement meaning joins. The refinement step
SHALL visit nodes in structural node-id order and, within each
node, follow FR-375's stages: retain the structural refinement id;
check canonical order in `populations`, `objects` (including `fields`),
`histories` (including `updates`), `assume`, `ensure`, then optional
`abstract_internal` and `class_catalog` (`concrete`, then `abstract`), with
catalog entries ordered by their catalog key (`class` UTF-8 bytes, then
`memory_model` UTF-8 bytes for equal memory-class spellings); join the header
in `semantic_type`, `abstract`, optional `protocol`, `profile` order; check
catalog presence and entries before row joins; then join rows in
`populations`, `objects` (including `fields`), `steps` in declaration order,
`histories` (including `updates`), `assume`, `ensure` order. Within an array or
nested array it SHALL visit ascending indices; within one row at one stage
it SHALL visit members in the published schema's declared order where
FR-375 fixes no further tie-break.
A missing `drain`, a `drain` on `memory flush`, a wrong `drain` value, a
wrong model for `memory flush`, or an extra class-entry member SHALL refuse
`malformed_wire` at the offending node/member in the earlier closed-wire
stage, before graph or catalog joins. A schema-valid but out-of-order
catalog entry SHALL refuse `invalid_semantic_graph` at the first offending
entry; a duplicate declared class spelling or duplicate (`class`,
`memory_model`) memory pair SHALL refuse at the later duplicate entry in the
refinement canonical-order stage. A schema-valid but
semantically absent or extra source-default memory class in a present catalog SHALL refuse
`invalid_semantic_graph` at its catalog array or entry in the later protocol
graph join of [FR-038](./FR-038-consume-checked-package-v2.md); a wrong
subject, node association or binder signature SHALL refuse
`invalid_model_binding`/`malformed-declaration` at the catalog entry. A
malformed closed row that passed schema validation refuses
`invalid_semantic_graph` at its body or offending entry (IR reading); a
malformed term already assigned `malformed_wire` by FR-038 keeps that
earlier refusal. A wrong body on another `relation` form
does not become a refinement, and a valid refinement body on an unknown form
still refuses at `semantic_form`.

### Catalog-free operation step join

For a model-only refinement with no `class_catalog`, the reader SHALL join
each step's local `source_node` to an admitted `state`/`operation_anchor` node.
The anchor's operation name SHALL equal the row's alias-free `source_class`.
Its `context` SHALL be the object type, process or state machine that declares
that operation in the selected concrete model subject, including a declaration
from which the selected type inherits the operation. The anchor remains at
the declaring node when the operation is inherited. The reader SHALL establish
subject membership through the declaring node's selected model ownership;
the anchor `context` is a declaration node and SHALL NOT be compared for
equality with the refinement `semantic_type`, which is the selected concrete
model's canonical `model_root`.

An absent `source_node` for an operation class SHALL refuse
`invalid_semantic_graph` at the offending step entry in the earlier closed-body
check. An unresolved `source_node` SHALL refuse
`missing_declaration`/`missing-name` at `source_node`. A present node of
another kind or an anchor declaring an operation of a foreign or wrong
selected model subject SHALL refuse
`invalid_model_binding`/`malformed-declaration` at `source_node`; a mismatch
between the anchor's operation name and `source_class` SHALL refuse that
same code and cause at `source_class`. These joins SHALL run in FR-375's
existing row order, after the earlier wire, identity, canonical-order,
header and catalog checks. The present-`class_catalog` row-to-catalog path
and its refusal order remain as specified above.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-061-AC-1 | A positive model-subject refinement with one population, object, operation-step, history and fairness row admits as one `relation`/`refinement` node with the exact nine required body members and its complete rows; empty arrays are retained, optional `protocol` and `abstract_internal` are absent, `class_catalog` is absent when neither subject has a protocol or memory class, `semantic_type` names the selected concrete package's canonical `model_root` rather than a declaration node, `abstract` names the selected abstract package's canonical `model_root`, and the public writer's output reads back to the same structural node id and package name. | Test |
| FR-061-AC-2 | For each of the six arrays, a missing array, extra body or entry member, wrong key shape, wrong row scalar or an out-of-grammar nested term refuses the whole package at the closed-schema/body stage; `open` is admitted only in `operation` or `protocol_step` argument arrays, while an `open` receiver, field expression, history expression or body root refuses. An unknown `relation` form still refuses `invalid_semantic_graph` at `semantic_form`. | Test |
| FR-061-AC-3 | With zero, one or multiple concrete object-type or state-machine declarations, `semantic_type` names the unique local `model_root` identity-bound to the selected concrete model package; an object-type or state-machine declaration substituted there refuses `invalid_model_binding`/`malformed-declaration` at `semantic_type`. With two selected models, swapping to the other's declared structural root treats that root as the concrete identity: an absent protocol-subject catalog refuses `invalid_semantic_graph` at `/body/class_catalog`; otherwise the first foreign concrete key in `populations`, `objects`, `steps`, `histories`, `assume` row order refuses `invalid_model_binding`/`malformed-declaration` at that key before catalog comparison; otherwise an extra catalog class refuses `invalid_semantic_graph` at its entry and an omitted class at `class_catalog/concrete`, in catalog-key order. With no concrete key, a distinct structural model subject without a catalog admits as a distinct node regardless of either model's classes; a protocol subject admits when its catalog equals the new identity's derived classes, including two empty sets. A local abstract model uses its canonical `model_root` `NodeRef` and a local protocol subject additionally uses its exact clause `NodeRef`; both appear in `dependencies` even with all rows empty. The same abstract root and clause from a directly selected dependency use `PackageNodeKey`, join through that selection and are absent from local dependencies. A `PackageNodeKey` for a local target, a bare `NodeRef` for a dependency target, a key for an unselected dependency, and the current package name as an imported target in its own body each refuse without admission or dropped content. | Test |
| FR-061-AC-4 | The reader admits all four closed step target kinds, requires `source_node` and protocol target `node` exactly for declaration-naming classes, omits them for memory classes, and preserves tied protocol-step and memory rows in their declaration order through read/write/read, including a valid permutation of those rows. A mismatched source class/node, wrong-kind operation or protocol target, or missing declared row target refuses at the named key or entry with its typed graph/model-binding result. | Test |
| FR-061-AC-5 | A history `initial` literal and a local history-literal reference both admit; a dependency-package history-literal reference, an application initial, duplicate history update, duplicate object field, invalid fairness kind/granularity, or unsorted `populations`, `objects`, `histories`, `assume`, `ensure`, object fields or history updates refuses, with no reordered or defaulted output. A valid permutation of `steps` admits without a sort refusal or reordered output. | Test |
| FR-061-AC-6 | The exact local dependency set includes body header keys and every direct row declaration/type and expression/member target, excludes external keys and annotations, and is structurally ordered. An omitted, extra or misordered dependency refuses `invalid_semantic_graph` at `/dependencies` before any row-meaning defect. | Test |
| FR-061-AC-7 | PLANNED/UNRUN (IR-1141). Two row-empty refinements over one abstract model but different selected concrete models retain distinct structural model-root references even when their bodies and declaration counts match. Row-empty subjects admit exactly when their concrete catalog matches the selected class set. A body-row edit changes the complete retained body; a valid source edit leaving that body unchanged leaves the retained body unchanged. The reader validates dependency and catalog joins independently of the structural id. | Test |
| FR-061-AC-8 | A proof-eligible refinement with `n` steps admits exactly its declaration occurrence 0 and generated occurrences 0 through `n`; generated ordinal `i + 1` retains the region of `steps[i]` in received declaration order, including after a valid row permutation. An owner with a hidden field, a history or an `any` step admits only declaration occurrence 0. Duplicate, extra-role, extra-ordinal or wrong-region source-map entries refuse `invalid_package`/`invalid-value` at the offending entry; a missing required occurrence refuses the same pair at `/source_map` after every present entry passes. No derived per-step node is synthesized. | Test |
| FR-061-AC-9 | The reader reports the earlier FR-322 strict-wire, graph-shape, key, declaration or source-map defect before a later refinement meaning defect. Among two refinement nodes with separate meaning defects, the lower node id wins. Within one node, a stale dedicated refinement key precedes canonical array-order defects; canonical defects precede header-join defects; header joins run in `semantic_type`, `abstract`, optional `protocol`, `profile` order and precede catalog and row joins; canonical checks visit `populations`, `objects`/`fields`, `histories`/`updates`, `assume`, `ensure`, optional `abstract_internal`, then optional `class_catalog` (`concrete`, `abstract`) in FR-375 order. `steps` has no canonical-order refusal; row checks retain their existing six-array order and visit `steps` in declaration order. Within each array checks visit ascending index and schema-declared entry-member order where FR-375 fixes no further tie-break. An invalid body of another `relation` form never enters the refinement body path. | Test |
| FR-061-AC-10 | An external consumer builds one typed refinement catalog over source `parallel` controls defaulting to `sc`, `tso` and `ra`. Its `concrete` array contains exactly three `{class: "memory drain", memory_model, drain: "store-buffer-flush"}` entries ordered by the (`class`, `memory_model`) key as `ra`, `sc`, `tso`, then the separate `{class: "memory flush", memory_model: "tso"}` entry. The public writer encodes each without extra members; the strict reader admits all four and retains the typed drain value and model through read/write/read with unchanged structural node id and package name. The typed drain variant is distinct from the TSO-only flush variant. | Test |
| FR-061-AC-11 | Starting from an otherwise valid package, deleting `drain` from `memory drain`, changing its value or type, adding it to `memory flush`, assigning `memory flush` model `sc` or `ra`, or adding `node`, `binder_types` or another unknown entry member refuses `malformed_wire` at the closed-wire stage, with the offending node/member location and no admitted package or defaulted field. With structural ids retained, reordering the three drains, repeating a (`memory drain`, `memory_model`) pair, or repeating a declared class spelling refuses `invalid_semantic_graph` at the first out-of-order or later duplicate catalog entry after structural graph checks. | Test |
| FR-061-AC-12 | A protocol refinement catalog includes one `memory drain` per distinct `sc`, `tso` or `ra` source-default model selected by a `parallel` in its subject and additionally one `memory flush` for a TSO source default. Two `parallel` controls selecting the same source-default model contribute one entry per model, not one per control. In a present catalog, an omitted required drain or TSO flush, an extra memory class, or a memory class from another selected subject refuses `invalid_semantic_graph` at the catalog array or entry after the protocol-body check; a wholly absent required catalog refuses earlier in the refinement step. A request-time memory override changes neither catalog nor package name. | Test |
| FR-061-AC-13 | PLANNED/UNRUN (IR-1141). Changing a valid drain entry memory_model changes the complete retained refinement body, with no key recomputation. A local catalog class node or binder type omitted from the exact dependencies refuses invalid_semantic_graph at /dependencies before catalog joins; no memory entry contributes a node or binder dependency. | Test |
| FR-061-AC-14 | In a model-only refinement without `class_catalog`, a step whose `source_class` names an operation and whose local `source_node` is that operation's anchor admits when the anchor `context` is the object type, process or state machine declaring the operation in the selected concrete model subject. The same holds when the selected type inherits the operation: the anchor stays at its declaring type, while the refinement `semantic_type` stays at the selected concrete model's canonical `model_root`. The admitted row and identities survive read/write/read. | Test |
| FR-061-AC-15 | For a catalog-free operation step, an absent `source_node` refuses `invalid_semantic_graph` at the step entry in the earlier closed-body check; an unresolved `source_node` refuses `missing_declaration`/`missing-name` at `source_node`. A present wrong-kind node or an anchor for a foreign or wrong selected model subject refuses `invalid_model_binding`/`malformed-declaration` at `source_node`; an anchor whose operation name differs from `source_class` refuses the same pair at `source_class`. None admits a partial package. | Test |
| FR-061-AC-16 | With a catalog-free step defect and an earlier wire, identity, canonical-order defect in another array, header or catalog defect, the earlier FR-375 stage wins; with two step-row defects, the first row in declaration order wins. A refinement with `class_catalog` continues to join a step to its concrete catalog entry, including the existing missing-class and wrong-node refusals, independently of the catalog-free anchor rule. | Test |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns the V2 reader,
  writer, shared refusal types, identity and selected-dependency stages.
- `ix://agent-ix/quire-specification/FR-375` owns the refinement's emitted
  node, row grammar and identity; `ix://agent-ix/quire-specification/FR-322`
  owns graph shape, occurrence admission and reader order.

## Status

Planned for the catalog-free operation step join and declaration-order `steps`
admission. These additions do not claim implementation evidence.
