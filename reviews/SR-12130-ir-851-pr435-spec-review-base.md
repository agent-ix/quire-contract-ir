---
id: SR-12130
title: "IR-851 spec review (base)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@dcd84d128d6ce891d555032cb9767a4301d55fbe (PR #435); spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Typed relationship end of a reaches claim, AC-262..267), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

# IR-851 spec review (base)

## Summary

Ticket: IR-851. Spec-only PR #435 adds the FR-038 section "Typed relationship end of a reaches claim", AC-262..267 (PLANNED), six tests.md rows and a TC-048 procedure. Each statement was checked against origin/main code (`model_members.rs` RelationshipEnd/RelationshipDecl/navigation_type :218-282, `operations.rs` check_relationship_member :2132-2286, `model_fields.rs` RetainedModels :407-457, `mod.rs` validate :916-969 and equality :559-563, `parameter_declaration.rs` as the sibling pattern). The core claims hold: an admitted package keeps no relationship end today; retention after `validate` returns is outside equality, uncharged and does not move any refusal; the direction rule and the destination rule match `navigation_type`; and admission guarantees `ordered` false, `upper` Some and a homogeneous destination type for every admitted reaches claim (navigation_type is called on the reaches path at :2261 and the homogeneity check is at :2265).

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-038 measured-gap paragraph | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | RetainedModels keeps only the declaration index, the object types' field tables and the operation parameter types, so an admitted CheckedPackageV2 holds no relationship end. |
| FR-038 API statement | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | CheckedPackageV2::relationship_end(&self, claim: &CheckedNodeId) -> Result<CheckedRelationshipEnd, CheckedRelationshipEndError> shall return the destination end the reader resolved for that claim |
| FR-038 Selection paragraph | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | which the reader therefore retains for each selected relationship declaration (both ends' role, type reference and multiplicity, and the direction) |
| FR-038 Result paragraph | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | destination_type is the identity of the destination object type's (or systems interface's) model declaration node, the node whose Reference is the claim's operand type. |
| FR-038 admitted-claim paragraph | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | For every admitted reaches claim the accessor therefore returns ordered false, upper Some and a destination_type that names the receiver endpoint's own object type. |
| FR-038 refusal paragraph | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | A claim whose declaration does not resolve to a retained relationship, whose name matches no role or both roles of it, or whose destination type has no model declaration node in the graph shall refuse UnresolvedEnd. |
| FR-038 FR-053 split paragraph | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | A relationship end is not an edge field. FR-053 and its AC-5 name model_object_fields as the sole authority for model edge-FIELD multiplicity |
| FR-038-AC-262 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | homogeneous relationship whose source end is {1,1} and whose target end is {2,3} unique with role peer ... so a swap of the two ends fails |
| FR-038-AC-263 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | destination end of {0,3} unique, {0,3} not unique, {2,3}, {0,1} and {1,1} ... omits its role it returns role None |
| FR-038-AC-264 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | refuses UnknownNode; ... navigate application with a relationship_end member ... refuses NotRelationshipEndClaim for each |
| FR-038-AC-265 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | four post-admission mutations ... refuses UnresolvedEnd for each |
| FR-038-AC-266 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | results are equal, the package equals its pre-call self, the accessor charges no work |
| FR-038-AC-267 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | exhaustively matches CheckedRelationshipEndError ... constructs CheckedRelationshipEnd field by field |
| TC-048 relationship-end section | examined | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md | Build admitted packages from the IR-824 reaches fixtures |
| FR-038-AC-252..257 | context_only | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | reaches admission criteria the new section relies on |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The retention list (both ends' role, type reference, multiplicity, direction) is not enough to produce `destination_type`. Admission derives the destination key from the selected document's identity, the destination's object-type vs systems-interface form and the byte limit (`declaration_key(&owner.package.identity, form, &destination.type_ref, bytes)`). RetainedModels/DeclaredNode keep no per-selection document identity and no interface flag. So the code PR must either retain more than the spec names, or re-derive the key, which breaks "adds no second derivation". Fix: state that the reader retains the derived destination declaration key (or the identity and form) with each relationship record. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3225-3233, crates/quire-contract-model/src/checked_package/v2/operations.rs:2245-2260, crates/quire-contract-model/src/checked_package/v2/model_fields.rs:394-417 |
| FND-002 | low | The Result paragraph defines `destination_type` as "the node whose `Reference` is the claim's operand type". That is false for a subtype operand, which FR-038-AC-252/253 admit: the operand type is `Reference<Subtype>`, and the destination is the declared end type (`conforms_to` via ancestors). AC-263 already says the accessor makes no statement about a subtype operand. Fix: say it is the declared destination end type (equal to the receiver end's), to which every operand conforms. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3237-3239, crates/quire-contract-model/src/checked_package/v2/model_members.rs:901-907 |
| FND-003 | low | UnresolvedEnd lists four causes. Other post-admission defects of `operation.member` get no variant and so have no specified outcome: member absent, `declaration` not a node id, `name` not text, `kind` other than `relationship_end`. The sibling `MalformedBody` (IR-832) is a catch-all for "body no longer the admitted shape". Fix: make UnresolvedEnd cover any `operation.member` that no longer resolves as admission resolved it. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3262-3267, crates/quire-contract-model/src/checked_package/v2/parameter_declaration.rs:30-32 |
| FND-004 | low | AC-262 and the TC-048 procedure do not state the relationship direction. The forward claim and the inverse claim over one relationship both admit only when the direction is `bidirectional` or `undirected` (`RelationshipDirection::admits`; AC-254 refuses the other direction). Fix: name the direction in the fixture. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4609, crates/quire-contract-model/src/checked_package/v2/model_members.rs:242-247 |
| FND-005 | low | Naming. `claim` / `NotRelationshipEndClaim` refer to the reaches application node. In the consumer (CG FR-015-AC-98) a "graph claim" is the `state_clause` node plus an argument path. Also, a `navigate` application over a `relationship_end` member is a relationship-end application, yet it refuses `NotRelationshipEndClaim`. Two implementers could read the parameter as the clause id. Fix: rename to `application` / `NotReachesApplication`, and say the id is the reaches application node, not its enclosing clause. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3202-3215, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3257-3261 |

## Verdict

PASS WITH FINDINGS. The measured gap, the direction and destination rule, the admitted-claim guarantees (ordered false, upper Some, homogeneous destination type), the equality and work statements, and the FR-053 field/end split were each verified against origin/main. FND-001 is a hidden requirement that the code PR would hit first. The rest are wording.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | The TC-048 fixture text spells the one-way directions `source_to_target` and `target_to_source`. The reader's wire values are `source-to-target` and `target-to-source` (`model_members.rs:2505`), so a fixture author copying the TC text gets a malformed-declaration refusal, not the AC-254 direction refusal. Fix: use the hyphenated spellings. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:2298, crates/quire-contract-model/src/checked_package/v2/model_members.rs:2505 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 (29f86c7): the new Retention paragraph retains "the derived model declaration node id of each end's type" and says "The accessor shall not recompute that id". The ids are already in the admission declaration index (ModelOwners::index), so this can be implemented without a second derivation. |
| FND-002 | fixed | Round 1 (29f86c7): `destination_type` is "the declared end type to which every operand conforms, not the type of a particular operand, so a subtype operand does not change it". |
| FND-003 | fixed | Round 1 (29f86c7): UnresolvedEnd is now a catch-all, "a reaches application whose operation.member no longer resolves as admission resolved it", listing absent member, wrong kind, non-node-id or unresolved declaration, and non-text, no-role or both-role name. AC-265 covers all seven. |
| FND-004 | fixed | Round 1 (29f86c7): AC-262 and TC-048 name a bidirectional relationship. |
| FND-005 | fixed | Round 1 (29f86c7): the parameter and field are renamed `application`, the error is `NotReachesApplication`, and the spec states the id is the reaches application, not the enclosing state_clause. A `relationship: CheckedNodeId` field was added (nine fields). |
