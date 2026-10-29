---
id: SR-610035
title: "Integrity review — IR-1034 drain-bearing refinement catalog"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-061-admit-refinement-nodes.md; PR #594 "
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-061
    type: references
---

## Summary

Integrity review of IR-1034 PR #594. The diff changes only FR-061. QSpec #447 supplies the authoritative FR-375/TC-310 shape. No applicable AssuranceProfile was found under spec/.

## Verdict

**FAIL** — the multi-default catalog cannot satisfy both completeness and uniqueness.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-12 requires a memory drain entry for each SC, TSO and RA source default in one subject, but the same FR requires catalog classes to be unique by alias-free class spelling and AC-11 refuses duplicate entries. Two parallel controls with different defaults need two entries both named `memory drain`; neither a unique-class catalog nor the specified duplicate refusal can admit the valid subject. QSpec FR-375 and TC-310 CC-12 carry the same conflict. Resolve the catalog key/order and multiplicity in QSpec and IR together; preserve all three defaults. | spec/checked_package/functional/FR-061-admit-refinement-nodes.md:203 |

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-061-AC-1 | examined | A positive model-subject refinement with one population, object, operation-step, history and fairness row admits as one `relation`/`refinement` node with the exact nine required body members and its complete rows; empty arrays are retained, optional `protocol` and `abstract_internal` are absent, `semantic_type` names the concrete model, and the public writer's output reads back to the same node and package identities. |
| FR-061-AC-2 | examined | For each of the six arrays, a missing array, extra body or entry member, wrong key shape, wrong row scalar or an out-of-grammar nested term refuses the whole package at the closed-schema/body stage; `open` is admitted only in `operation` or `protocol_step` argument arrays, while an `open` receiver, field expression, history expression or body root refuses. An unknown `relation` form still refuses `invalid_semantic_graph` at `semantic_form`. |
| FR-061-AC-3 | examined | A local abstract model or protocol clause uses `NodeRef` and appears in `dependencies` even with all rows empty; the same clause from a directly selected dependency uses `PackageNodeKey`, joins through that selection and is absent from local dependencies. A `PackageNodeKey` for a local target, a bare `NodeRef` for a dependency target, a key for an unselected dependency, and the current package id in its own body each refuse without admission or dropped content. |
| FR-061-AC-4 | examined | The reader admits all four closed step target kinds, requires `source_node` and protocol target `node` exactly for declaration-naming classes, omits them for memory classes, and preserves canonical tied protocol-step and memory ordering through read/write/read. A mismatched source class/node, wrong-kind operation or protocol target, or missing declared row target refuses at the named key or entry with its typed graph/model-binding result. |
| FR-061-AC-5 | examined | A history `initial` literal and a local history-literal reference both admit; a dependency-package history-literal reference, an application initial, duplicate history update, duplicate object field, invalid fairness kind/granularity, or unsorted arrays/fields/updates refuses, with no reordered or defaulted output. |
| FR-061-AC-6 | examined | The exact local dependency set includes body header keys and every direct row declaration/type and expression/member target, excludes external keys and annotations, and is digest-ascending. An omitted, extra or misordered dependency refuses `invalid_semantic_graph` at `/dependencies` before any row-meaning defect. |
| FR-061-AC-7 | examined | Two row-empty refinements over one abstract model and profile but distinct concrete models have equal bodies, distinct `semantic_type` keys and distinct `quire.refinement-node/v1` node ids. Mutating any meaning-bearing body row or optional internal list with recomputed identities changes node and package ids; retaining a stale key refuses at the earlier key stage; source row permutation that canonicalizes to the same arrays changes neither id. |
| FR-061-AC-8 | examined | A proof-eligible refinement with `n` steps admits exactly its declaration occurrence 0 and generated occurrences 0 through `n`; an owner with a hidden field, a history or an `any` step admits only declaration occurrence 0. Duplicate, extra-role, extra-ordinal or wrong-region source-map entries refuse `invalid_package`/`invalid-value` at the offending entry; a missing required occurrence refuses the same pair at `/source_map` after every present entry passes. No derived per-step node is synthesized. |
| FR-061-AC-9 | examined | The reader reports the earlier FR-322 strict-wire, graph-shape, key, declaration or source-map defect before a later refinement meaning defect. Among two refinement nodes with separate meaning defects, the lower node id wins. Within one node, a stale dedicated refinement key precedes canonical array-order defects; canonical defects precede header-join defects; header joins run in `semantic_type`, `abstract`, optional `protocol`, `profile` order and precede catalog and row joins; canonical checks visit `populations`, `objects`/`fields`, `steps`, `histories`/`updates`, `assume`, `ensure`, optional `abstract_internal`, then optional `class_catalog` (`concrete`, `abstract`) in FR-375 order; row checks retain their existing six-array order. Within each array checks visit ascending index and schema-declared entry-member order where FR-375 fixes no further tie-break. An invalid body of another `relation` form never enters the refinement body path. |
| FR-061-AC-10 | examined | An external consumer builds a typed refinement catalog for each source-default model `sc`, `tso` and `ra` with the exact `{class: "memory drain", memory_model, drain: "store-buffer-flush"}` entry, and for `tso` with the separate `{class: "memory flush", memory_model: "tso"}` entry. The public writer encodes each without extra members; the strict reader admits each and retains the typed drain value and model through read/write/read with unchanged node and package identities. The typed drain variant is distinct from the TSO-only flush variant. |
| FR-061-AC-11 | examined | Starting from an otherwise valid package, deleting `drain` from `memory drain`, changing its value or type, adding it to `memory flush`, assigning `memory flush` model `sc` or `ra`, or adding `node`, `binder_types` or another unknown entry member refuses `malformed_wire` at the closed-wire stage, with the offending node/member location and no admitted package or defaulted field. A schema-valid duplicate or misordered memory catalog entry refuses `invalid_semantic_graph` at the first offending entry after the dedicated key check. |
| FR-061-AC-12 | examined | A protocol refinement catalog includes `memory drain` for every `sc`, `tso` or `ra` source-default `parallel` selected in its subject and additionally `memory flush` for a TSO source default. In a present catalog, an omitted required drain or TSO flush, an extra memory class, or a memory class from another selected subject refuses `invalid_semantic_graph` at the catalog array or entry after the protocol-body check; a wholly absent required catalog refuses earlier in the refinement step. A request-time memory override changes neither catalog nor package id. |
| FR-061-AC-13 | examined | With all identities recomputed, changing a schema-valid drain entry's `memory_model` changes the dedicated refinement node id and package id; retaining its old node id refuses `invalid_package`/`stale-node-key` at `node_id` before catalog joins. A local catalog class node or binder type omitted from the exact `dependencies` list refuses `invalid_semantic_graph` at `/dependencies` before the dedicated key check, while no memory entry contributes a node or binder dependency. |

## Evidence

- `quire validate --scope . spec/checked_package/functional/FR-061-admit-refinement-nodes.md --summary`: 1/1 grammar-clean.
- `quire lint` on the changed FR: exit 0. `quire properties --criteria --all`: 13 criteria classified, 8 extractable.
- Diff has no code, fixture, schema, or copied QSpec file. Existing AC-1 through AC-8 are unchanged; AC-9 extends ordering without removing the prior stages.
- QSpec PR #447 FR-375 states one catalog entry per alias-free class spelling and one `memory drain` per source default; TC-310 CC-12 asks one subject with SC, TSO and RA defaults.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | Round 1 changes only FR-061 model-root header and AC-3. AC-12 and the duplicate class-entry refusal are unchanged; STD-499 remains open for QSpec catalog multiplicity and order. |

| FND-001 | still-open | Round 2: rebased on merged IR main. AC-1, AC-3 and AC-7 are byte-identical to main; AC-9 through AC-13 are byte-identical to the originally reviewed memory-drain. The multi-default catalog contradiction is unchanged and remains owned by STD-499. |

| FND-001 | fixed | Round 3: QSpec #459 (merge ) defines declared-class key as `class` and memory-class key as (`class`, `memory_model`), ordered by class then model bytes. FR-061 now requires ra/sc/tso drains plus TSO flush, one entry per distinct source-default model, and rejects only equal catalog keys. The valid multi-default subject can now be admitted. |

## Current verdict

**FAIL**. The model-root correction matches QSpec origin/main FR-375 header joins, FR-375-AC-25 and FR-322 model-root owner rules. A selected package has one canonical root regardless of object-type or state-machine count; a local abstract root is a local dependency, while a directly selected external root joins through its package. The new AC-3 wrong-declaration and wrong-owner-root cases can fail and preserve the prior local/external-key refusal cases. No new finding. FND-001 remains open pending STD-499; PR #594 remains draft.

## Current verdict after round 2

**FAIL**. Rebase introduces no new finding or AC narrowing. FND-001 remains still-open solely because QSpec STD-499 has not settled memory catalog uniqueness, multiplicity and order. PR #594 remains draft.

## Current verdict after round 3

**PASS**. Original FND-001 is fixed by the QSpec #459 catalog-key ruling and aligned IR prose/AC-10 through AC-12. All three source-default drain entries remain required and the duplicate/reorder refusals use the complete catalog key. AC-1 through AC-8 are unchanged from IR main; AC-9 and AC-13 are unchanged from the prior reviewed memory head. No new finding or AC narrowing. Spec validation and lint pass.
