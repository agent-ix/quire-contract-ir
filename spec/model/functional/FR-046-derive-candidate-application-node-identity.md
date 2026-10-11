---
id: FR-046
title: "Derive a candidate application node identity before reader admission"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: traces_to
---
# FR-046: Derive a candidate application node identity before reader admission

## Description

PLANNED/UNRUN (IR-738). The API and its acceptance controls below are required
upon implementation; this specification does not claim present exports or
executed runtime evidence.

When a caller supplies a draft application before checked-package reader
admission, `quire-contract-model` SHALL derive its candidate canonical node
identity through the reader's authoritative application preimage and encoder.

## Inputs

A borrowed draft `CheckedSemanticNodeV2`, an optional immutable context built
from the caller's complete ordered draft graph, and an explicit canonical
`byte_limit: u64`. A draft graph is pre-read input, not an admitted package.

## Outputs

`Result<CandidateApplicationNodeId, ApplicationIdentityError>`. A candidate
contains the canonical `CheckedNodeId` for assembling the package wire; it
does not certify admission, operation laws, reference resolution or group
validity of that package.

## Behavior

### Typed pre-read surface

The model crate SHALL export `ApplicationNodeDraft`,
`ApplicationIdentityContext`, `CandidateApplicationNodeId`,
`ApplicationIdentityError` and `derive_application_node_id` by name.
These items have private storage and no unchecked context constructor.

`ApplicationNodeDraft::new(&CheckedSemanticNodeV2)` borrows the supplied node.
Its `node_id` is a provisional self/reference selector, explicitly unverified:
the final canonical ID need not be known when the draft is created. The wrapper
does not validate or mutate that node. `ApplicationIdentityContext::try_new`
takes `&[CheckedSemanticNodeV2]` and the draft; it returns a context or the
typed error below. The context borrows this complete ordered snapshot and
retains private validated target/group indices, not an owned graph or an
independently mutable group list.

`derive_application_node_id(draft, context, byte_limit)` accepts a borrowed
draft, `Option<&ApplicationIdentityContext>` and `u64`, and returns the typed
result above. `CandidateApplicationNodeId::candidate_node_id()` returns an
immutable `&CheckedNodeId`; its name and documentation identify it as a
candidate. No helper result converts to `CheckedPackageV2` or asserts that a
provisional selector is an admitted semantic identity.

### Ordered context and refusals

When constructing context, the constructor SHALL find exactly one graph node
with the draft's supplied key and require that node's complete fields to equal
the borrowed draft. All supplied graph node keys are unique. The group is every
node carrying the target's recursion label, filtered from the complete graph
in its existing order, including nonapplication nodes. Its size and target
ordinal are derived from that order; an arbitrary caller ordinal or a list
containing only applications is not an accepted substitute. A context for an
ungrouped draft has no group. A grouped draft requires context; an ungrouped
draft accepts no context or its matching ungrouped context.

If a context or draft check fails, the helper SHALL return the corresponding
`ApplicationIdentityError` variant with no candidate:

| Variant | Condition and typed payload |
| --- | --- |
| `NonApplication` | The draft's body root is not `application`; carries its supplied `CheckedNodeId`. |
| `MissingContext` | A grouped draft has no context; carries its supplied `CheckedNodeId`. |
| `MissingTarget` | The complete snapshot has no node with the draft's supplied key; carries that key. |
| `RepeatedKey` | A graph key is repeated; carries the key and first/repeated graph positions as `u64`. |
| `ContextMismatch` | The snapshot target differs from the draft, or a derivation uses a context constructed for a different draft; carries the supplied draft key. |
| `ContextIndexOutOfRange` | A graph position, group size or ordinal cannot be represented as `u64`; carries the supplied draft key. |
| `CanonicalEncoding` | The canonical encoder refuses the preimage, including the byte ceiling; retains the typed `quire_canonical::Error`. |

Construction scans repeated keys in graph order and selects the earliest
repeated position, then the earliest equal predecessor; this refusal precedes
missing-target and target mismatch. Derivation checks application root, then
required/matching context, then canonical encoding. Group context never falls
back to an ungrouped hash on failure. The constructor verifies the supplied
snapshot, not its completeness relative to an unavailable future package:
supplying the complete intended graph remains the caller's obligation.

### One authoritative preimage

The helper and reader SHALL use one application preimage construction and one
canonical encoder for `{version: "quire.application-node/v1", node_tag,
semantic_form, semantic_type, declaration, recursion, body}`. Missing
declaration and ungrouped recursion encode as `null`. Grouped recursion encodes
`{size, ordinal}`. Body `reference` terms targeting group members encode as
`{term: "group_reference", ordinal}` through application arguments, aggregate
members and binding values. Outside-group references and dependency references
retain their supplied content. Semantic type and declaration retain their
original preimage rules; they are not rewritten as body references. No
provisional key is added as an extra preimage member.

The helper SHALL hash that preimage through `quire-canonical` with
`Limits::new(byte_limit)` and return domain `quire.checked-semantic-node/v1`.
Its byte ceiling is exactly the reader's application-hash ceiling, including
success when the canonical preimage byte count equals the limit and refusal
when it exceeds it. No alternate encoder, fallback hash or new ceiling exists.

### Resources and reader separation

The public context constructor SHALL borrow the graph without cloning nodes,
bodies or a group-member vector. With N supplied nodes it performs at most
N(N-1)/2 key comparisons for uniqueness and linear scans for target/group
selection, plus one target-node equality comparison. Auxiliary context storage
is constant in N: borrowed snapshot, target position, group size and ordinal.
Key comparison and node equality costs include the supplied key/field sizes.
Position/count conversions are checked. No new graph-size cap, work meter,
global cache or hidden owned graph is introduced.

The shared grouped-body rewrite currently needs one copied body, visits each
term once and searches the group for each reference. With B body storage,
T visited terms and G group nodes, its body copy is O(B) and reference selection
is O(TG); a borrowed group scan can inspect up to N nodes per reference.
Traversal and canonical encoding retain their existing heap-stack costs.
These costs are bounded by the actual supplied input, not by `byte_limit`:
that limit constrains canonical output and does not bound context scans,
the draft graph, body cloning or traversal allocations. This contract makes
no new recoverable-allocation guarantee for the existing body clone or encoder;
an allocator exhaustion outside their typed error surface is not a fabricated
`CanonicalEncoding` refusal. Context construction itself needs no heap allocation.

The reader SHALL retain its existing application ordering, authenticated graph
context, refusal translation and one reader work-unit charge before preimage
construction and hashing. Sharing the preimage/encoder does not call the public
context constructor in the reader or add its checks, refusals or work charges
to reader admission.

### Caller assembly and final admission

The caller SHALL replace every provisional key and matching reference in the
final wire with the corresponding candidate/final node key while preserving
the intended graph order and recursion labels. For group-internal body
references, this substitution preserves the preimage's ordinals, including
self references. Outside-group references must already name their intended
final targets when deriving the referencing candidate; changing one later
requires re-derivation. The same applies to semantic type and declaration
members that bind identities. The helper does not compute another node's key
or perform substitutions for the caller. The caller refreshes affected
dependencies, identity projection and package identity under their existing
contracts, then invokes `CheckedPackageV2::read` for final admission. A retained
candidate after a body change remains subject to the reader's stale-key refusal.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-046-AC-1 | An external model-crate consumer derives a candidate from a typed draft before any package read, obtains its candidate key and matches typed errors; no helper output is an admission verdict or admitted-package constructor. The named public items are exported only by the model crate. | Test, Inspection |
| FR-046-AC-2 | An ordinary application produces the independently constructed canonical application-preimage digest. Changing node tag, semantic form, semantic type, declaration presence/content or body independently changes the bound digest. Changing only its provisional selector does not change an ungrouped candidate. | Test |
| FR-046-AC-3 | A group containing applications and nonapplication nodes uses every group node in graph order. Self and internal references through application arguments, aggregate members and binding values produce the expected ordinals; provisional-to-final key substitution preserves the candidate. Outside-group and dependency references remain unchanged, and changing an outside target changes the candidate. | Test |
| FR-046-AC-4 | Missing grouped context, absent target, repeated graph key, mismatching target fields or context from another draft return their exact typed variants/payloads and no hash candidate. Repetition order is deterministic; nonapplication input refuses. Inspection confirms borrowed immutable constant-size context, checked counters and the stated scan bounds, without a new cap or allocator-dependent context failure. | Test, Inspection |
| FR-046-AC-5 | For an independently encoded preimage of C canonical bytes, byte limits C and C+1 produce the same candidate; C-1 and zero refuse CanonicalEncoding with the canonical byte-limit error. A canonical nonencodable-number control retains its typed encoder error. The byte ceiling is not asserted to bound graph/context/body-copy memory or work. | Test, Inspection |
| FR-046-AC-6 | After correct final-key/reference substitution, reader-admissible ordinary and grouped controls read successfully with the helper candidate. Changing the body while retaining that key produces the reader's stale-node-key outcome and distinct expected key. Inspection finds one authoritative preimage/encoder and unchanged reader charge-before-hash ordering; an exhausted existing work charge wins before an otherwise failing application hash. Public context validation adds no reader refusal or work charge. All runtime procedures remain planned/unrun until implementation. | Test, Inspection |

## Dependencies

[FR-019](./FR-019-rust-library-interface.md) owns the explicit model-crate
public inventory. [FR-028](./FR-028-separate-cycle-free-contract-model.md)
owns the cycle-free substrate boundary. [FR-038](../../checked_package/functional/FR-038-consume-checked-package-v2.md)
owns reader admission, application-key preimage semantics and stale-key refusal.
The existing [TC-048](../../checked_package/matrix/TC-048-checked-package-v2-strict-reader.md)
and [TC-058](../matrix/TC-058-model-crate-public-interface.md) are the intended
procedure artifacts; the new runtime controls are PLANNED/UNRUN. This slice
adds no compatibility layer, schema, downstream engine dependency, operation
law, package identity helper or reader gate.
