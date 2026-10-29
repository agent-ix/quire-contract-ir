---
id: SR-8361
title: "Gap analysis of IR-763 structural candidates"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; spec/model/functional/FR-048-derive-structural-and-group-candidates.md; tests/it/checked_package_v2_structural_keys.rs, checked_package_v2_model_members.rs, public_interface.rs; crates/quire-contract-model/src/checked_package/v2/structural_candidates.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-048
    type: reviews
---

## Summary

Checked FR-048's five criteria against the tagged tests and changed source. The grouped tests use already valid reader keys; they do not exercise a grouped draft whose in-group references use provisional selectors.

## Verdict

**CONDITIONAL** — A regression test is needed for FND-001; the code defect is recorded in SR-8360.

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
| FND-001 | medium | AC-3 has no test that rekeys a group from provisional selectors and submits the assembled wire to the reader | tests/it/checked_package_v2_structural_keys.rs:1594; tests/it/checked_package_v2_structural_keys.rs:1924 |

## Coverage

FR-048-AC-1 through FR-048-AC-5 had tagged tests in the changed integration files. At the original reviewed head, the AC-3 test checked an already admitted graph and a permutation of digest-form IDs. The deep-stack test used a provisional selector but only asserted candidate success, not reader admission. Plan completion: not assessed.

The computed Test Matrix was unavailable: quire matrix --scope. --format json exited without JSON after duplicate-module diagnostics. The tag-to-criterion check was a declared grep and source-inspection fallback.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | — tc_048_provisional_mixed_group_candidates_assemble_and_admit derives from non-digest group selectors, installs final IDs and label, and passes the real reader. |

## Recovery provenance

Reconstructed after the temporary reviewer scratchpad was removed. The original findings and fixed dispositions were posted privately to Linear IR-763 and read back byte-for-byte during the review. Source comment IDs: code finding 6ea32d2a-32d4-4f54-9740-8e303ca13b97, gap finding 7b993e43-7625-42e5-8640-ae792519e429, code disposition e1fee837-8f48-4986-a8c4-490b6b26fe67, gap disposition e848e562-9240-49b2-a91b-73065e75a760. The reviewed source revisions are and fix. Prose outside the finding and disposition rows is reconstructed, not asserted to be byte-identical to the deleted scratchpad files.
