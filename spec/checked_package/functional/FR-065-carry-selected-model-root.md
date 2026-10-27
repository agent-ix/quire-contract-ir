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
A missing root, repeated root, malformed owner or region, mismatched
selection, stale key or wrong root shape SHALL refuse the whole package at
the applicable FR-322 stage with its typed code, cause and pointer. A
malformed earlier stage SHALL win over a later root or refinement defect.
For a root whose closed owner is well formed but whose identity/digest pair
does not join a selected row, the owner-join refusal SHALL be
`missing_declaration`/`missing-selection` at that node's `owner`.

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
| FR-065-AC-4 | For zero, one and multiple object types or state machines in a selected model document, the package has exactly one root for that selection. Zero selected model packages produce no root. A missing root, two roots for one selection, or a root for an unselected pair refuses the whole package under FR-322; the reader never chooses a representative declaration. | Test |
| FR-065-AC-5 | An absent or extra `ModelRootOwner` member, a `ModelOwner` on a root, or a `ModelRootOwner` on another form refuses at the closed wire/owner-presence stage. A closed root owner whose identity or digest does not match one selected row refuses `missing_declaration`/`missing-selection` at its `owner` in the owner-join stage. An earlier independent wire, selection-evidence, graph or source-map defect wins over the later owner-join, key or refinement check according to FR-322's order. | Test |
| FR-065-AC-6 | A model-subject refinement uses the selected concrete root for `semantic_type` and the selected abstract root for its local `abstract` key, including an otherwise row-empty refinement. A protocol-subject refinement uses the abstract model root plus the exact protocol clause key. A root in a directly selected dependency resolves only through its package key. A header naming an object type, state machine, alias-derived surrogate, unselected dependency or wrong selected root refuses under FR-375's header joins after earlier FR-322 checks; the reader does not infer a substitute. | Test |
| FR-065-AC-7 | The public typed writer and reader carry `model_root` and `ModelRootOwner` in the existing V2 graph and envelope, with exact read/write/read retention. No parallel untyped writer, fallback owner inference, compatibility reader, copied QSpec schema or fixture is introduced. | Inspection |

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
include `model_root`, so no reader or writer acceptance is claimed.
