---
id: FR-048
title: "Derive candidate structural node and recursion-group identities"
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
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-058
    type: references
---
# FR-048: Derive candidate structural node and recursion-group identities

## Description

When a caller assembles a CheckedPackage V2 graph before reader admission,
`quire-contract-model` SHALL derive typed candidate structural node identities
and recursion-group labels through the reader's canonical rules. The result is
assembly data, not an admission verdict.

## Inputs

A borrowed, complete, ordered slice of `CheckedSemanticNodeV2` drafts and
explicit `byte_limit: u64` and `work_limit: u64`. Draft `node_id` values are
provisional selectors. References to nodes outside a recursion group must name
their intended final identities when the candidate is derived.

## Outputs

`derive_structural_candidates` returns either `StructuralCandidates` or a
typed `StructuralCandidateError`. `StructuralCandidates` contains one
`CandidateStructuralNodeId` for each structural draft, associated with its
supplied graph position, and one `CandidateRecursionGroupLabel` for each
retained group. Their fields are private; accessors expose the candidate node
ID or label and the originating position or retained label. No output converts
to an admitted package or installs keys into caller input.

## Behavior

### Public boundary and graph selection

The model crate SHALL export `derive_structural_candidates`,
`StructuralCandidates`, `CandidateStructuralNodeId`,
`CandidateRecursionGroupLabel` and `StructuralCandidateError` by name. The
function SHALL borrow its graph, preserve graph-array order and leave drafts
unchanged. It SHALL reject repeated provisional node keys with their first
and repeated positions. A retained `recursion_group` label selects every node
carrying that label, including application members and off-cycle members, in
wire order. A group does not arise from a names cycle or from a caller-supplied
member list. Position and count conversions SHALL be checked.

The function SHALL derive candidates for the same structural nodes that the
reader's first structural-key pass authenticates: nodes without a nominal
preimage or application key, except the separately owned
`correspondence`/`abstraction_relation` identity. It SHALL accept a declared
source owner or an undeclared model owner as a typed `CheckedNodeOwner` and
include the owner actually carried by the draft; it SHALL NOT invent or infer
one. A draft's tag/form, declaration and owner remain subject to the reader's
later joins and admission rules.

### One structural rule and group label

The helper and reader SHALL call the same `StructuralPreimage` construction,
`structural_key` encoder and grouped-body reference mapping described by
[FR-038](../../checked_package/functional/FR-038-consume-checked-package-v2.md#structural-keys-inside-and-outside-labelled-groups-ir-630).
The preimage SHALL include the actual typed owner, declaration, semantic type,
body and recursion place under that rule. The candidate for a grouped
structural node uses the newly derived candidate label in `recursion.group`
and graph-order `{size, ordinal}`; the retained label selects only the draft
members. In-group body and non-self semantic-type references use ordinals,
while outside references retain their final node IDs. An ungrouped candidate
uses null recursion. No public raw-preimage, arbitrary group-ordinal or
caller-selected hash rule is added.

For each group, the helper SHALL first derive every member-local digest in
graph order with `recursion.group` omitted, using the reader's
application-local digest for an application member and its structural-local
digest for a structural member. It SHALL hash that ordered digest array
through the same canonical encoder to return the candidate group label, then
derive member keys using that new label. This initial API SHALL refuse a
labelled group containing a nominal member or separately keyed correspondence
shape, outside its required candidate scope; the helper SHALL NOT synthesize
a local digest for either. Renaming the retained label
consistently across its members does not change the computed local label. A
wire assembled with a corrupt label remains subject to the reader's
stale-key refusal.

The ten closed derived shapes, including owner-free composite and bounded
domain forms, SHALL pass the reader's existing self-type, exact body,
Integer-literal-type and decimal-bound grammar checks before a candidate is
returned. The model-owned object, interface and relationship shapes without
a declaration SHALL pass the reader's existing fixed self-typed, empty-body,
ungrouped check. A malformed shape has no candidate. The helper SHALL not
repair a malformed body, semantic type, owner, group label or graph edge.

### Refusals and resources

`StructuralCandidateError` SHALL distinguish at least repeated key, invalid
tag/form, invalid closed shape, unsupported group member, counter overflow,
work exhaustion and canonical encoding failure through typed variants and
typed node/position context. Canonical failure SHALL retain the typed
`quire_canonical::Error`; work exhaustion SHALL retain the ceiling, consumed
work and failing draft position. No caller shall need to parse error text.
The helper SHALL charge every graph-key comparison, group-membership visit,
body-term visit and digest operation to its explicit work limit before that
operation and stop at the first unpaid charge. Its canonical
preimage and digest encodings SHALL use `Limits::new(byte_limit)`, including
success exactly at the byte ceiling and typed refusal when it is exceeded.
The limits do not claim to bound caller-owned graph memory. Term walks and
temporary cleanup SHALL be iterative, including deep in-memory draft bodies;
the helper SHALL not add a fixed depth cap, global cache, second preimage
implementation or new reader work charge.

The reader SHALL retain its own order: owner and graph checks, application
keys, the complete structural-key pass, then group labels. The helper's
validation SHALL NOT be used as a substitute for `CheckedPackageV2::read`.
After replacing provisional node keys and every matching reference, updating
labels, dependencies, projection and package identity, the caller SHALL submit
the wire to that reader. A stale structural key or group label then yields its
existing `invalid_package`/`stale-node-key` refusal, while invalid graph or
owner joins retain their existing typed refusals and precedence.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-048-AC-1 | An external default-feature model-crate consumer can derive typed candidates from ordered drafts and inspect candidate IDs, labels and typed errors; it cannot construct a candidate with unchecked fields or treat one as an admitted package. The root bridge crate re-exports none of these items. | Test |
| FR-048-AC-2 | Reader-admissible owner-free `reference`, `option`, collection and bounded-domain drafts, and source-owned and model-owned structural drafts including a model relationship, produce exactly the identities authenticated by the reader. Independently changing owner, declaration, semantic type, body or an outside reference changes the applicable candidate; retaining the old key in assembled wire yields the reader's stale-key refusal. | Test |
| FR-048-AC-3 | A reader-admissible group with structural and application members, including an off-cycle member, produces member candidates and one label agreeing with reader admission in graph order. Permuting members requires new ordinals, keys and label. A body/member change changes its local label candidate; retaining an old member key or a corrupt retained label yields the reader's typed stale-key refusal at the existing pass. | Test |
| FR-048-AC-4 | Repeated selectors, an unknown tag/form, malformed closed shapes (including bounded-domain type, literal type and decimal grammar), an invalid model-owned fixed shape, and a group member outside this candidate API's local-digest scope each return a distinct typed error and no candidate result. The helper does not normalize the invalid draft or widen reader admission. | Test |
| FR-048-AC-5 | Exact canonical byte ceiling succeeds; one less and zero retain the typed canonical error. The first unpaid work charge returns typed exhaustion with position and count. A deeply nested in-memory draft reaches a typed result or refusal on a 256 KiB stack without recursive traversal or cleanup. Inspection finds one structural preimage/encoder and unchanged reader charge and refusal precedence. | Test |

## Dependencies

[FR-038](../../checked_package/functional/FR-038-consume-checked-package-v2.md)
owns reader admission, structural/group rules, and stale-key outcomes.
[FR-046](./FR-046-derive-candidate-application-node-identity.md) owns the
application preimage and its candidate API. [FR-019](./FR-019-rust-library-interface.md)
owns the model-crate public inventory. This requirement adds no new wire
version, compatibility path, copied preimage, or version/digest tracking.
The planned [TC-048](../../checked_package/matrix/TC-048-checked-package-v2-strict-reader.md)
procedures compare candidates with reader admission and refusals;
[TC-058](../matrix/TC-058-model-crate-public-interface.md) covers the
compiled public interface and inventory.
