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
  - target: ix://agent-ix/quire-specification/FR-375-AC-25
    type: references
---

# FR-065: Carry the identity-bound selected model root

## Description

When a checked-package V2 lock selects a domain package, the Contract IR
typed writer and strict reader SHALL carry exactly one `model`/`model_root`
node for that selected identity and digest, as defined by QSpec
`ix://agent-ix/quire-specification/FR-322-AC-132`. The root denotes the
selected package as a whole, including when it has no object type or state
machine declaration. IR SHALL preserve the root as the identity-bound target
used by QSpec `ix://agent-ix/quire-specification/FR-375-AC-25` for refinement
headers, without choosing a representative declaration or source alias.

## Inputs

A typed V2 graph and existing `model_selections` supplied to the encoder of
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
`ModelRootOwner` `{kind: "model_root", identity, digest}`. The public typed
node-kind vocabulary SHALL include `model`/`model_root`. The writer SHALL
encode those members through the existing V2 graph and envelope.
The reader SHALL retain them in the admitted graph and occurrence-free node
projection.
No new lock selection field or FCD declaration is required for the root.

For every selected domain-package identity and digest, the reader SHALL
require exactly one root whose `owner.identity` and `owner.digest` equal that
selection's values. The root SHALL be self-typed, carry no `declaration`,
`recursion_group` or nominal preimage, have the empty aggregate body
`{term: "aggregate", members: []}` and empty `dependencies`, and carry the
closed `ModelRootOwner` on no other node form. Its structural node id SHALL
equal the JCS SHA-256 of QSpec's closed `ModelDeclarationNode` preimage:
`{version: "quire.structural-node/v1", node_tag: "model",
semantic_form: "model_root", semantic_type: null, declaration: null,
recursion: null, owner: {kind: "model_root", identity, digest},
body: {term: "aggregate", members: []}}`. This key SHALL depend on the
selected identity and digest, not the source alias or any particular model
declaration. The package identity SHALL include the root in its ordinary
occurrence-free graph projection.

The root SHALL carry exactly one `generated` occurrence at ordinal zero,
whose source-map region is the existing source occurrence that selected that
model package. It SHALL carry no declaration occurrence. The reader SHALL
retain that correspondence without synthesizing a new source declaration,
source-map entry, selection or region.

The strict reader SHALL apply QSpec FR-322's established order: closed wire
and owner-presence checks, selected-document evidence, graph and occurrence
shape, owner join and structural-key checks, then later refinement meaning.
If an identical second root repeats the structural key for one selection,
the reader SHALL refuse `invalid_semantic_graph`, with no cause, at the later
`/semantic_graph/nodes/{j}/node_id` during graph shape, before structural-key
checks. If a second root has a distinct retained key but the same owner pair,
the reader SHALL first fail the structural-key check as `invalid_package`/
`stale-node-key` at that root's `/semantic_graph/nodes/{j}/node_id`, after
earlier graph defects. After all owner joins and node-key checks pass, the
reader SHALL check root completeness by selected-row order. A selected row
with no root SHALL refuse `invalid_semantic_graph`, with no cause, at
`/semantic_graph/nodes` before declaration and refinement meaning checks.
When several selected rows lack roots, the first selected row in lock order
determines that refusal. A malformed earlier stage SHALL win over the
root-completeness check; the root-completeness check SHALL win over a later
refinement-header defect. The reader SHALL admit no partial package.

A root whose closed `ModelRootOwner` identity/digest pair is absent from
`model_selections` SHALL refuse the whole package, with no inferred
selection or rewritten owner. QSpec FR-322-AC-132 assigns this case to the
V2 schema, while FR-322's detailed owner-join order assigns it to a reader
`missing_declaration`/`missing-selection` refusal at the node's `owner`.
STD-500 must reconcile the published schema and reader and fix the exact
stage, code, cause and RFC 6901 pointer before IR freezes this case's test
oracle or claims admission. Neither interpretation is selected here.

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
| FR-065-AC-1 | An external consumer constructs a complete typed V2 package selecting one domain package, with its self-typed `model`/`model_root` node and closed `ModelRootOwner`, calls `encode_checked_package_v2`, then reads the bytes through the public strict reader. The admitted graph, occurrence-free projection and canonical read/write/read retain the same owner, root node id and package id. A source producer derives the root from the existing selection without a new FCD declaration or lock field. | Test |
| FR-065-AC-2 | The root has absent `declaration`, `recursion_group` and nominal preimage, empty aggregate body and dependencies, and exactly one `generated` occurrence at ordinal zero whose source-map region is the source occurrence of the selected model package. A declaration occurrence, wrong role or ordinal, wrong or missing region, or extra occurrence refuses through FR-322's occurrence/source-map check; none is silently repaired. | Test |
| FR-065-AC-3 | The root's node id equals the JCS SHA-256 of the exact closed `ModelDeclarationNode` preimage containing `ModelRootOwner` and `semantic_type: null`, including QSpec's published model-root vector. Changing only a source alias while preserving the selected identity and digest leaves the root id unchanged. A different selected identity or changed selected-package content gives a different identity or digest and, after rebuilding, different root and package ids. A retained stale root key refuses under FR-322's structural-key check. | Test |
| FR-065-AC-4 | For zero, one and multiple object types or state machines in a selected model document, the package has exactly one root for that selection; zero selected packages produce no root. After earlier wire, selected-evidence, graph, occurrence, owner and key checks pass, a selection with no root refuses `invalid_semantic_graph` with no cause at `/semantic_graph/nodes`, in selected-row order before declaration and refinement meaning. Cloning a valid root as a second graph node refuses `invalid_semantic_graph` with no cause at the later `/semantic_graph/nodes/{j}/node_id` in graph shape, before key checking. Giving that second root a distinct retained id but the same owner pair refuses `invalid_package`/`stale-node-key` at its `/semantic_graph/nodes/{j}/node_id` in the structural-key stage; a prior graph defect wins. The reader never chooses a representative declaration or admits a partial package. | Test |
| FR-065-AC-5 | An absent or extra `ModelRootOwner` member, a `ModelOwner` on a root, or a `ModelRootOwner` on another form refuses at the closed wire/owner-presence stage. A well-formed root whose owner identity/digest pair is not selected also refuses, without inferred selection or repaired owner; an earlier independent defect wins according to FR-322's order. The exact stage, code, cause and RFC 6901 pointer for this unselected-pair case remain pending QSpec STD-500, which must reconcile FR-322-AC-132, the detailed owner-join rule, the schema and an authoritative negative control before this portion is claimed or its test oracle fixed. | Test |
| FR-065-AC-6 | A model-subject refinement uses the selected concrete root for `semantic_type` and the selected abstract root for its local `abstract` key, including an otherwise row-empty refinement. A protocol-subject refinement uses the abstract model root plus the exact protocol clause key. A root in a directly selected dependency resolves only through its package key. A header naming an object type, state machine, alias-derived surrogate, unselected dependency or wrong selected root refuses under FR-375's header joins after earlier FR-322 checks; the reader does not infer a substitute. | Test |
| FR-065-AC-7 | An external consumer uses the public typed writer and strict reader to carry `model_root` and `ModelRootOwner` through the existing V2 graph and envelope; read/write/read retains the exact root owner, node id, package id and generated source correspondence. | Test |
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
  `ix://agent-ix/quire-specification/FR-375-AC-25` own the authoritative root
  and refinement-header semantics.

## Status

Planned. The measured IR typed owner and model-form vocabularies do not yet
include `model_root`, so no reader or writer acceptance is claimed. The
unselected-pair refusal oracle in AC-5 remains blocked by QSpec STD-500;
the full refusal obligation remains in this requirement.
