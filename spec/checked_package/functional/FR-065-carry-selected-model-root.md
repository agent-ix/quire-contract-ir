---
id: FR-065
title: "Carry the identity-bound selected model root"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-051
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-322-AC-132
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-135
    type: references
  - target: ix://agent-ix/quire-specification/FR-375-AC-25
    type: references
---

# FR-065: Carry the identity-bound selected model root

## Description

A checked-package V2 selects a domain package by carrying a `model`/`model_root`
node for its identity: the Contract IR typed writer and strict reader SHALL carry
exactly one such node per selected identity, as defined by QSpec
`ix://agent-ix/quire-specification/FR-322-AC-132`. The root denotes the
selected package as a whole, including when it has no object type or state
machine declaration. IR SHALL preserve the root as the identity-bound target
used by QSpec `ix://agent-ix/quire-specification/FR-375-AC-25` for refinement
headers, without choosing a representative declaration or source alias.

## Inputs

A typed V2 graph supplied to the encoder of
[FR-051](./FR-051-encode-checked-package-v2-envelope.md), or untrusted V2
bytes, selected model documents, source-map evidence and limits supplied to
the strict reader of [FR-038](./FR-038-consume-checked-package-v2.md). QSpec's
published schemas and model-root identity vector remain in its authoritative
repository.

## Outputs

Canonical V2 bytes carrying the complete root, an admitted typed package
retaining the root and its source correspondence, or one typed, located
refusal with no partial admitted package.

## Behavior

The public typed `CheckedNodeOwner` vocabulary SHALL include the closed
`ModelRootOwner` `{kind: "model_root", identity}`. The public typed
node-kind vocabulary SHALL include `model`/`model_root`. The writer SHALL
encode those members through the existing V2 graph and envelope.
The reader SHALL retain them in the admitted graph and occurrence-free node
projection.
The root is the selection: the lock holds no domain-package row, and no FCD
declaration is required for the root. The root's identity carries no digest and
no version.

The selected domain-package identities are the `owner.identity` values of the
roots. The reader SHALL require at most one root per identity. The root is self-typed, carry no `declaration`,
`recursion_group` or nominal preimage, have the empty aggregate body
`{term: "aggregate", members: []}` and empty `dependencies`, and carry the
closed `ModelRootOwner` on no other node form. Its structural node id uses the selected package name and declared path under QSpec FR-322. The reader checks its shape, uniqueness, owner and body independently and does not derive an expected id from the owner or body.

The root SHALL carry exactly one `generated` occurrence at ordinal zero,
whose source-map region is the existing source occurrence that selected that
model package. It SHALL carry no declaration occurrence. The reader SHALL
retain that correspondence without synthesizing a new source declaration,
source-map entry, selection or region.

The strict reader SHALL apply QSpec FR-322's established order: closed wire
and owner-presence checks, selected-document evidence, graph and occurrence
shape, owner join and structural-key checks, then later refinement meaning.
If a second root repeats the identity of an earlier root, whether or not it
repeats the structural key, the reader SHALL refuse `invalid_semantic_graph`,
with no cause, at the later `/semantic_graph/nodes/{j}/owner`, in the
selected-document evidence stage, before graph shape and structural-key
checks. A `ModelOwner` whose identity names no root SHALL refuse
`missing_declaration`/`missing-selection` in the owner join. The reader SHALL
admit no partial package.

The reader SHALL resolve a refinement's local concrete `semantic_type` and
local model-subject `abstract` key to the corresponding selected model roots.
An external `abstract` package key SHALL join only through its directly
selected dependency package's admitted root. The reader SHALL not substitute
an object type, state machine, alias, similarly named node or first
declaration for either root. [FR-061](./FR-061-admit-refinement-nodes.md)
continues to own the refinement body and its later header-join order.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-065-AC-1 | An external consumer constructs a complete typed V2 package selecting one domain package by identity, with its self-typed `model`/`model_root` node and closed `ModelRootOwner`, calls `encode_checked_package_v2`, then reads the bytes through the public strict reader. The admitted graph, occurrence-free projection and canonical read/write/read retain the same owner, root structural node id and package name. A source producer derives the root from the selected identity without a new FCD declaration or lock field. | Test |
| FR-065-AC-2 | The root has absent `declaration`, `recursion_group` and nominal preimage, empty aggregate body and dependencies, and exactly one `generated` occurrence at ordinal zero whose source-map region is the source occurrence of the selected model package. A declaration occurrence, wrong role or ordinal, wrong or missing region, or extra occurrence refuses through FR-322's occurrence/source-map check; none is silently repaired. | Test |
| FR-065-AC-3 | Withdrawn by the IR-1141 structural-identity decision under QSpec #517: model-root node ids are structural labels and are never recomputed from owner content. This id is retained and never reused. | Withdrawn |
| FR-065-AC-4 | For zero, one and multiple selected object types or state machines, the package has exactly one root for each model identity. Cloning a valid root as a second graph node refuses invalid_semantic_graph at the later owner in the selected-document evidence stage; a prior wire defect wins. The reader neither picks a representative nor admits a partial package. | Test |
| FR-065-AC-5 | An absent or extra `ModelRootOwner` member (including a `digest` or `version`), a `ModelOwner` on a root, or a `ModelRootOwner` on another form refuses at the closed wire/owner-presence stage. A `ModelOwner` whose identity names no root refuses `missing_declaration`/`missing-selection` in the reader's owner join at `/semantic_graph/nodes/{i}/owner`, without inferred selection or repaired owner; an earlier independent defect wins according to FR-322's order. A root with no supplied document refuses `missing_import`/`missing-selection` at `/semantic_graph/nodes/{i}/owner/identity`. | Test |
| FR-065-AC-6 | A model-subject refinement uses the selected concrete root for `semantic_type` and the selected abstract root for its local `abstract` key, including an otherwise row-empty refinement. A protocol-subject refinement uses the abstract model root plus the exact protocol clause key. A root in a directly selected dependency resolves only through its package key. A header naming an object type, state machine, alias-derived surrogate, unselected dependency or wrong selected root refuses under FR-375's header joins after earlier FR-322 checks; the reader does not infer a substitute. | Test |
| FR-065-AC-7 | An external consumer uses the public typed writer and strict reader to carry `model_root` and `ModelRootOwner` through the existing V2 graph and envelope; read/write/read retains the exact root owner, structural node id, package name and generated source correspondence. | Test |
| FR-065-AC-8 | Source inspection finds one typed V2 graph and envelope path for the root, with no parallel untyped writer, fallback owner inference, compatibility reader, copied QSpec schema or fixture. | Inspection |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns the strict reader,
  selected evidence, identity, occurrence validation and refusal precedence.
- [FR-051](./FR-051-encode-checked-package-v2-envelope.md) owns the public
  typed envelope writer.
- [FR-061](./FR-061-admit-refinement-nodes.md) owns refinement-body
  admission; this requirement supplies the selected model-root carrier that
  its header joins require.
- QSpec `ix://agent-ix/quire-specification/FR-322-AC-132` and
  `ix://agent-ix/quire-specification/FR-322-AC-135` own the selected root,
  schema boundary and unselected-owner refusal.
- QSpec `ix://agent-ix/quire-specification/FR-375-AC-25` owns the
  refinement-header semantics.

## Status

In progress. The typed root carrier and reader are implemented; AC-5's
unselected-owner refusal is fixed by merged QSpec STD-500. AC-2's source
correspondence remains open for reconciliation with QSpec FR-322-AC-136;
this requirement retains its full source-correspondence obligation.
