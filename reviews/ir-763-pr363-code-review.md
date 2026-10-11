---
id: SR-8360
title: "Code review of IR-763 structural candidates"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@4e7dfd4af083e3cc716c6aeda59e55c2687d134f; crates/quire-contract-model/src/checked_package/v2/structural_candidates.rs, encode.rs, structural.rs, application_identity.rs; tests/it/checked_package_v2_structural_keys.rs, checked_package_v2_model_members.rs, public_interface.rs; FR-048-AC-1..5"
review_set: subset
---

## Summary

Reviewed the public candidate API, its shared reader encoders, and the changed tests at PR #363's exact head. Grouped drafts with provisional selectors can produce a label and key that the reader will reject after assembly.

## Verdict

**FAIL** — FND-001 violates FR-048's provisional-selector and grouped-candidate contract.

## Examined scope

| ID | Role | Excerpt |
| --- | --- | --- |
| FR-048 | context_only | When a caller assembles a CheckedPackage V2 graph before reader admission, quire-contract-model SHALL derive typed candidate structural node identities and recursion-group labels through the reader's canonical rules. |
| FR-048-AC-1 | examined | An external default-feature model-crate consumer can derive typed candidates from ordered drafts and inspect candidate IDs, labels and typed errors; it cannot construct a candidate with unchecked fields or treat one as an admitted package. The root bridge crate re-exports none of these items. |
| FR-048-AC-2 | examined | Reader-admissible owner-free `reference`, `option`, collection and bounded-domain drafts, and source-owned and model-owned structural drafts including a model relationship, produce exactly the identities authenticated by the reader. Independently changing owner, declaration, semantic type, body or an outside reference changes the applicable candidate; retaining the old key in assembled wire yields the reader's stale-key refusal. |
| FR-048-AC-3 | examined | A reader-admissible group with structural and application members, including an off-cycle member, produces member candidates and one label agreeing with reader admission in graph order. Permuting members requires new ordinals, keys and label. A body/member change changes its local label candidate; retaining an old member key or a corrupt retained label yields the reader's typed stale-key refusal at the existing pass. |
| FR-048-AC-4 | examined | Repeated selectors, an unknown tag/form, malformed closed shapes (including bounded-domain type, literal type and decimal grammar), an invalid model-owned fixed shape, and a group member outside this candidate API's local-digest scope each return a distinct typed error and no candidate result. The helper does not normalize the invalid draft or widen reader admission. |
| FR-048-AC-5 | examined | Exact canonical byte ceiling succeeds; one less and zero retain the typed canonical error. The first unpaid work charge returns typed exhaustion with position and count. A deeply nested in-memory draft reaches a typed result or refusal on a 256 KiB stack without recursive traversal or cleanup. Inspection finds one structural preimage/encoder and unchanged reader charge and refusal precedence. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Grouped body references to provisional selectors are not rewritten to ordinals, so candidate keys and labels disagree with final reader admission | crates/quire-contract-model/src/checked_package/v2/encode.rs:121; crates/quire-contract-model/src/checked_package/v2/structural.rs:660; crates/quire-contract-model/src/checked_package/v2/application_identity.rs:293 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | b24215c04b2bb92eda91eebec4c1f1e1ecd231fa — grouped structural and application encoding now accept typed provisional selectors; the reader's final-digest check remains in reference_target. |

## Recovery provenance

Reconstructed after the temporary reviewer scratchpad was removed. The original findings and fixed dispositions were posted privately to Linear IR-763 and read back byte-for-byte during the review. Source comment IDs: code finding 6ea32d2a-32d4-4f54-9740-8e303ca13b97, gap finding 7b993e43-7625-42e5-8640-ae792519e429, code disposition e1fee837-8f48-4986-a8c4-490b6b26fe67, gap disposition e848e562-9240-49b2-a91b-73065e75a760. The reviewed source revisions are 4e7dfd4af083e3cc716c6aeda59e55c2687d134f and fix b24215c04b2bb92eda91eebec4c1f1e1ecd231fa. Prose outside the finding and disposition rows is reconstructed, not asserted to be byte-identical to the deleted scratchpad files.
