---
id: SR-610037
title: "EARS review — IR-1034 drain-bearing refinement catalog"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-061-admit-refinement-nodes.md; PR #594 at fc2afe9420821292a6eacf42f499892010593052"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-061
    type: references
---

## Summary

EARS review of IR-1034 PR #594 at fc2afe9420821292a6eacf42f499892010593052. The diff changes only FR-061. QSpec #447 at c5cfb1da5be1f29730ac90d2b8862251388d0100 supplies the authoritative FR-375/TC-310 shape. No applicable AssuranceProfile was found under spec/.

## Verdict

**PASS** — no distinct finding under this lens. The integrity finding remains blocking for the PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

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

## Method scope

Changed SHALL statements name the reader, writer, catalog, or typed class and state observable responses. No EARS-specific finding beyond the cross-criterion conflict recorded in integrity.
