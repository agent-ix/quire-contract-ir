---
id: FR-048
title: "Assemble structural node and recursion-group identities"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-046
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-349
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-058
    type: references
---
# FR-048: Assemble structural node and recursion-group identities

## Description

When a producer assembles a CheckedPackage V2 graph, it SHALL use QSpec FR-322's structural node ids with producer-assigned ordinals and shared string recursion-group labels. The reader SHALL validate the complete graph. The former derived-key and group-label candidate helper is removed without a shim or alternate hash path.

## Inputs

A complete ordered graph of typed V2 node drafts, with each node's explicit `{package, path, ordinal}`, body, owner, declaration and references. The package name and selected dependency names are explicit; no byte or work ceiling is accepted by an identity-derivation operation.

For a bulk rewrite of one mutable `CheckedSemanticNodeV2` draft or
`CheckedTerm`, a caller supplies a `FnMut(&CheckedNodeId) ->
Option<CheckedNodeId>` callback. `None` means that logical slot retains its
original identity; `Some(id)` supplies its replacement.

## Outputs

The typed graph may be encoded and submitted to `CheckedPackageV2::read`. Only the reader returns an admitted package or typed refusal/incomplete result. No candidate key or group-label object is returned.

The public `CheckedSemanticNodeV2::map_node_ids` and
`CheckedTerm::map_node_ids` operations mutate only the supplied draft node or
term. They return no admitted package, computed identity or validation verdict.

## Behavior

A declared node's path equals its declaration path, including when several declared nodes with distinct paths share one recursion-group label; an ungrouped derived node's path is `[node_tag, semantic_form, ...operands]`, each operand a string or closed `{"node": i}` index segment into this document's graph. An anonymous derived group member uses the sole declared member's path when exactly one exists, or a producer-chosen path when zero or multiple exist. No grouped path embeds operands. The structural ordinal is producer assigned and unique only among nodes sharing package and path, independent of graph order. The reader checks all graph ids for shape and uniqueness, joins owners, resolves references and dependencies, checks group membership and body semantics, and retains caller limits on the encoder and reader. It rebuilds structural keys from the resolved index operands as FR-038 requires. It performs no structural preimage construction, local-member hash, label hash, content-derived candidate-key recomputation or stale-key refusal.

When a producer rewrites V2 identities before encoding, Contract IR SHALL
visit each logical `CheckedNodeId` slot in the supplied node or term exactly
once, passing the original value to the callback. A closed operation record
and its typed read view mirror some slots; the rewrite SHALL call the
callback once for each such logical slot and apply the same result to both
representations, preserving their agreement. It SHALL apply a returned
replacement at that occurrence without invoking the callback on the
replacement. A `None` result SHALL leave that occurrence unchanged. Callback
invocation order SHALL be deterministic: for a node, `node_id`,
`semantic_type`, dependencies in vector order, optional `owner_model`, then
body; within a term, depth-first preorder of typed fields and members in
their declared order. Equal original identities in independent logical slots
are separate visits, and a callback may map them differently.

Within an application term, Contract IR SHALL visit the operation member before
`result_type` and the argument terms in argument order. Its closed raw member
record's logical identity slots, in visit order, are `declaration` for
`Field`, `Position`, `Element`, `Operation` and `TypeArgument`;
`declaration`, `destination_type`, then optional `population` for
`RelationshipEnd`; and the sole fairness-target identity for `Fairness`:
`declaration` for an operation or hybrid-transition target, `node` for a
protocol-root, attempt-node or compensation-template target, or `parallel`
for a parallel-branch target. `ProfileOperator`, `StateClause` and
`TemporalInterval` have no member identity slot. A structural path containing
nested `node` segments remains part of its enclosing whole `CheckedNodeId`:
Contract IR SHALL pass that whole id to the callback once and apply its result
without a separate visit or inferred rewrite of its path segments.
Contract IR SHALL apply the same callback result to the raw member and any typed member,
projection-field or member-declaration read view mirroring that slot without
another callback invocation.

The rewrite SHALL cover every typed node-identity slot of the node and of
each closed term production, including nested bindings, application records
and their typed views, frames, abstraction bodies, extension forms, and
refinement bodies. It SHALL NOT rewrite source names, ordinary strings,
recursion-group labels, or literal data merely because they resemble an
identity; path text changes only as part of a whole callback-supplied id.
It SHALL traverse deeply nested terms iteratively and perform one walk regardless of the number of distinct
replacements. Neither operation SHALL synthesize a structural id, rederive
one from a body, or construct an admitted package. A callback-supplied id is
producer input: the rewrite does not certify its shape, uniqueness, package
ownership, or reference resolution. When a rewritten graph is submitted to
the reader, the reader SHALL reject invalid or conflicting identities at its
normal typed locus; the mapper SHALL NOT silently repair such a graph.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-048-AC-1 | An external model-crate consumer constructs ordered structural ids, graph nodes and a complete envelope, then reads them through `CheckedPackageV2::read`; the raw graph is not an admitted package. The model crate's public inventory contains no candidate-key or group-label helper, and the root crate exports none. | Test |
| FR-048-AC-2 | PLANNED/UNRUN (IR-1141). Owner-free reference, option, collection and bounded-domain nodes and source-owned or model-owned declarations admit with their QSpec structural ids and exact bodies. Changing an owner, declaration, semantic type, body or outside reference is checked by the applicable owner, path, body or graph rule; a changed path component changes the structural id. | Test |
| FR-048-AC-3 | A group with declared structural and application members, including TC-946 K14's three-declared-member cycle and an off-cycle member, retains each declared member's own path under one shared label; an anonymous derived member has the sole declared member's path or a producer-chosen path for zero or multiple declared members. Permuting the group rewrites index segments to target positions and retains every resolved structural key and exact reference while changing graph-array order; a body or reference change is checked by its own graph/body rule. A repeated id, invalid label, mismatched cyclic-component label or unresolved exact reference refuses at its independent locus. | Test |
| FR-048-AC-4 | Repeated ids, unknown tag/form, malformed closed body, invalid owner shape, invalid or mismatched group label or out-of-range structural ordinal return distinct typed reader refusals and no admitted package; no invalid draft is silently normalized. | Test |
| FR-048-AC-5 | Canonical envelope encoding succeeds at its exact byte ceiling and fails with a typed no-bytes result one byte below it. Reader work exhaustion names the limit and failing position. A 100000-node in-memory graph and deeply nested body are traversed iteratively without stack overflow; no second graph copy, identity hash or reader hash work charge exists. | Test |
| FR-048-AC-6 | An external model-crate consumer calls public `CheckedSemanticNodeV2::map_node_ids` and `CheckedTerm::map_node_ids` with `FnMut(&CheckedNodeId) -> Option<CheckedNodeId>`; `None` preserves a logical slot, while `Some(id)` substitutes only that slot and any mirrored read representation. This includes a whole structural id with nested `node` path segments in each named operation-member identity site above. A callback mapping A to B and B to C sees each original whole logical slot once and does not cascade A through C or visit its nested path segments separately; two independent slots with equal original ids may receive distinct results in the specified visit order. PLANNED/UNRUN (IR-1240): existing tagged tests do not exercise the newly named nested operation-member case; an external-consumer test must back it. | Test |
| FR-048-AC-7 | A fixture with every typed V2 node-id slot and nested closed term production maps each marked logical slot exactly once, including raw operation-member records and their typed views, frames, abstraction and refinement bodies. It exercises `declaration` for field, position, element, operation and type-argument members; `declaration`, `destination_type` and optional `population` for relationship-end members; and the single `declaration`, `node` or `parallel` fairness-target slot for each target variant, using whole ids with nested structural path segments. A stateful callback gives one operation-record identity a different result than the next callback invocation; the record and each mirrored typed read view still contain the same replacement, with no callback on a mirror. The callback observes each member slot in its listed order before the application `result_type` and argument terms, stably across repeated runs and a clone. Source names, recursion-group labels, unmapped identities and literal data shaped like an id remain byte-for-byte unchanged; a path text segment changes only within a substituted whole id. PLANNED/UNRUN (IR-1240): the operation-member slot census, nested structural case and mirrored-view checks await the external-consumer test. | Test |
| FR-048-AC-8 | A producer rewrites an in-memory 100000-node graph with nested terms using a position remap: callback calls equal the number of logical identity slots, traversal completes without stack growth or a full walk for each replacement, and the resulting valid graph admits with the intended resolved references. A callback that instead supplies a duplicate, unselected foreign, malformed or dangling id creates no admitted package through the mapper; when that graph is submitted to the reader, the reader returns its existing typed refusal at the applicable locus rather than silently normalizing it. | Test |
| FR-048-AC-9 | An external model-crate consumer imports the two methods through the existing `CheckedSemanticNodeV2` and `CheckedTerm` public types, with no new crate-root export. A source review confirms that the mapper is a mutable draft visitor with no identity derivation, graph clone, name-based replacement, computed key or reader bypass. | Inspection |

## Dependencies

FR-038 and QSpec FR-322 own the reader, structural identity and group rules. FR-349 owns typed term slots, FR-019 owns the public inventory, and FR-051 owns canonical envelope encoding. The QSpec #517 structural-id ruling treats derived paths as identity labels and requires no reader recomputation. Body and graph checks apply independently.
