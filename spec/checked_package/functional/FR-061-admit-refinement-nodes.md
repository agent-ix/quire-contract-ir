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
`node-identity-preimage.schema.json` at the selected QSpec source.

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
permitted only with `protocol`. No field outside this body is admitted.

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

### Keys, graph and identity

`RefinementKey` SHALL be a local `NodeRef` for a node in this graph, or the
closed `{package, node}` `PackageNodeKey` for a node of a directly selected
dependency package. The current package id SHALL NOT be embedded in its own
body. `abstract` names the abstract model; optional `protocol` names the
selected protocol clause; `profile` is the selected infinite-trace
`DefinitionRef`, never the source alias. Local row declaration and type keys
are `NodeRef`s; abstract-side keys can be `RefinementKey`s. The reader SHALL
resolve each target against the selected package and require its declared
kind and subject. An unresolved key SHALL refuse under FR-038's
`missing_declaration`/`missing-name` rules, and a wrong-kind target SHALL
refuse `invalid_model_binding`/`malformed-declaration`, at that key. A
schema-invalid key or row SHALL refuse before such a meaning join. No
unresolved row, key or expression is dropped.

The node's `semantic_type` SHALL be the concrete model declaration's local
`NodeRef`. `dependencies` SHALL be exactly the unique, digest-ascending local
keys named by `abstract`, optional `protocol`, row declarations and types,
and expression reference targets and application member declarations and
populations. Local header keys remain dependencies when all arrays are
empty. An external `PackageNodeKey`, profile `DefinitionRef`, literal type,
application result type and standalone `semantic_type` are not dependencies
unless another listed body key names the same local node. A dependency
mismatch SHALL refuse `invalid_semantic_graph` at `/dependencies` in
FR-322's graph-shape stage, before refinement meaning joins.

The node id SHALL be the JCS SHA-256 of the closed
`{version: "quire.refinement-node/v1", semantic_type, body}` preimage. The
same complete body enters FR-322's package identity projection. A changed
row, profile, abstract or protocol target, internal list, or concrete model
SHALL change the node id and package id when identities are recomputed; a
stale retained key SHALL refuse at FR-322's earlier key stage. A source row
reorder that yields the same canonical arrays SHALL change neither identity.
Every six-array row order SHALL follow FR-375: first named declaration's
node-key digest, then member name bytes; tied protocol step rows additionally
use alias-free class bytes, with memory rows after declaration-naming rows.
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

The reader SHALL apply FR-322's strict wire, graph-shape, key, declaration
and source-map checks in their published order. Within refinement body
validation, it SHALL check the closed shape before resolving targets, and
shall report the first defect by ascending node id, then this fixed body
member order: `term`, `abstract`, `profile`, `populations`, `objects`,
`steps`, `histories`, `assume`, `ensure`, optional `protocol`, optional
`abstract_internal`. Within an array it SHALL visit ascending indices;
within an entry it SHALL visit members in the order the published schema
declares them (IR reading where FR-375 fixes no tie-break). A malformed
closed body or row refuses `invalid_semantic_graph` at its body or offending
entry (IR reading); a malformed term already assigned `malformed_wire` by
FR-038 keeps that earlier refusal. A wrong body on another `relation` form
does not become a refinement, and a valid refinement body on an unknown form
still refuses at `semantic_form`.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-061-AC-1 | A positive model-subject refinement with one population, object, operation-step, history and fairness row admits as one `relation`/`refinement` node with the exact nine required body members and its complete rows; empty arrays are retained, optional `protocol` and `abstract_internal` are absent, `semantic_type` names the concrete model, and the public writer's output reads back to the same node and package identities. | Test |
| FR-061-AC-2 | For each of the six arrays, a missing array, extra body or entry member, wrong key shape, wrong row scalar or an out-of-grammar nested term refuses the whole package at the closed-schema/body stage; `open` is admitted only in `operation` or `protocol_step` argument arrays, while an `open` receiver, field expression, history expression or body root refuses. An unknown `relation` form still refuses `invalid_semantic_graph` at `semantic_form`. | Test |
| FR-061-AC-3 | A local abstract model or protocol clause uses `NodeRef` and appears in `dependencies` even with all rows empty; the same clause from a directly selected dependency uses `PackageNodeKey`, joins through that selection and is absent from local dependencies. A `PackageNodeKey` for a local target, a bare `NodeRef` for a dependency target, a key for an unselected dependency, and the current package id in its own body each refuse without admission or dropped content. | Test |
| FR-061-AC-4 | The reader admits all four closed step target kinds, requires `source_node` and protocol target `node` exactly for declaration-naming classes, omits them for memory classes, and preserves canonical tied protocol-step and memory ordering through read/write/read. A mismatched source class/node, wrong-kind operation or protocol target, or missing declared row target refuses at the named key or entry with its typed graph/model-binding result. | Test |
| FR-061-AC-5 | A history `initial` literal and a local history-literal reference both admit; a dependency-package history-literal reference, an application initial, duplicate history update, duplicate object field, invalid fairness kind/granularity, or unsorted arrays/fields/updates refuses, with no reordered or defaulted output. | Test |
| FR-061-AC-6 | The exact local dependency set includes body header keys and every direct row declaration/type and expression/member target, excludes external keys and annotations, and is digest-ascending. An omitted, extra or misordered dependency refuses `invalid_semantic_graph` at `/dependencies` before any row-meaning defect. | Test |
| FR-061-AC-7 | Two row-empty refinements over one abstract model and profile but distinct concrete models have equal bodies, distinct `semantic_type` keys and distinct `quire.refinement-node/v1` node ids. Mutating any meaning-bearing body row or optional internal list with recomputed identities changes node and package ids; retaining a stale key refuses at the earlier key stage; source row permutation that canonicalizes to the same arrays changes neither id. | Test |
| FR-061-AC-8 | A proof-eligible refinement with `n` steps admits exactly its declaration occurrence 0 and generated occurrences 0 through `n`; an owner with a hidden field, a history or an `any` step admits only declaration occurrence 0. Duplicate, extra-role, extra-ordinal or wrong-region source-map entries refuse `invalid_package`/`invalid-value` at the offending entry; a missing required occurrence refuses the same pair at `/source_map` after every present entry passes. No derived per-step node is synthesized. | Test |
| FR-061-AC-9 | The reader reports the earlier FR-322 strict-wire, graph-shape, key, declaration or source-map defect before a later refinement meaning defect. Among two refinement nodes with separate meaning defects, the lower node id wins; within one node, defects are selected in the fixed `term`, `abstract`, `profile`, `populations`, `objects`, `steps`, `histories`, `assume`, `ensure`, `protocol`, `abstract_internal` order, then ascending array index and schema-declared entry-member order. An invalid body of another `relation` form never enters the refinement body path. | Test |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns the V2 reader,
  writer, shared refusal types, identity and selected-dependency stages.
- `ix://agent-ix/quire-specification/FR-375` owns the refinement's emitted
  node, row grammar and identity; `ix://agent-ix/quire-specification/FR-322`
  owns graph shape, occurrence admission and reader order.

## Status

Planned. The current IR reader rejects `relation`/`refinement` at the closed
semantic-form gate. The merged QSpec wire now defines its admission shape;
this FR does not claim implementation evidence.
