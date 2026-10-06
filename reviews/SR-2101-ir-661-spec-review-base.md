---
id: SR-2101
title: "Spec review (base checklist) \u2014 IR-661 FR-038/TC-048 relationship reader amendment"
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir@705cef3f7a07370f111f19e8e7d86e1e07fd31d7; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; base 1540b3b6c0e4d167fe1ed9116c296e45e7dff258; Ticket IR-661
review_set: subset
---

## Summary

Base checklist over the IR-661 diff: ID format and sequence (AC-165..171 continue after AC-164; gaps 156-158 and retired AC-66 kept), verification cells (method only, no TC id), cross-references and the hash/pin rule. Two high findings. `quire properties` run on FR-038 at base and at head extracts 158 criteria both times and none of AC-165..171, because the new rows sit outside the criteria table. The amendment also adds six commit-pinned links.

Ticket: IR-661. Method: `spec-review/base`. Reviewer model `claude-opus-5-5`, run `236a8098-415e-43ce-aa4b-9a892aa7e251`.

## Verdict

**FAIL**: two `high` findings. Clean, as examined: the new ids are sequential and unique; every new Verification cell names only `Test`; each new criterion and TC-048's new section say PLANNED / UNRUN; the TC procedures address AC-165..171 one-to-one or better; the focused `quire validate` of the two files exits 0 (it does not detect the table split).

## Scope Examined

- `FR-038#identity-slot` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:975-981
- `FR-038#relationship-ends` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:992-1000
- `FR-038#relationship-end-mapping` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1010-1016
- `FR-038#object-node-declaration` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1021-1027
- `FR-038#role-lookup` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1029-1037
- `FR-038#receiver-endpoint` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1039-1047
- `FR-038#navigation-derivation` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1046-1054
- `FR-038#other-operations` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1058-1063
- `FR-038#declaration-order-accounting` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1065-1072
- `FR-038#operation-order-paths` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1078-1086
- `FR-038-AC-165` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3387
- `FR-038-AC-166` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3388
- `FR-038-AC-167` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3389
- `FR-038-AC-168` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3390
- `FR-038-AC-169` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3392
- `FR-038-AC-170` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3393
- `FR-038-AC-171` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3394
- `FR-038#ac-block` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3386-3388
- `FR-038#pinned-links` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:977-983
- `TC-048#fcd-relationships-status` (examined) spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:937-944
- `TC-048#fcd-step-1` (examined) spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:946-953
- `TC-048#fcd-step-2` (examined) spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:963-970
- `TC-048#fcd-step-3` (examined) spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:970-977
- `TC-048#fcd-step-4` (examined) spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:981-988
- `TC-048#fcd-step-5` (examined) spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:992-1000
- `TC-048#fcd-step-6` (examined) spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1014-1022
- `TC-048#fcd-step-7` (examined) spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1031-1039
- `TC-048#fcd-step-8` (examined) spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1043-1050

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-165..171 sit after a blank line outside the criteria table; quire extracts none | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3386-3388 |
| FND-002 | high | Six commit-SHA-pinned blob links introduced into normative FR-038 prose | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:977-983 |

## Finding Detail

- **FND-001** (high, confidence high, check `trace`, unit `FR-038#ac-block`): The seven new rows are separated from the `| ID | Criteria | Verification |` table (header at line 3226) by a blank line at 3386, and AC-169..171 by a second blank line at 3391. They are therefore not table rows. `quire properties --scope . FR-038` reports 158 criteria at base 1540b3b and at head 705cef3, and its JSON lists no FR-038-AC-165..171. The new criteria are invisible to `quire matrix --strict`: they can never be reported untagged, and no trace tag can bind them.
- **FND-002** (high, confidence medium, check `other`, unit `FR-038#pinned-links`): The amendment adds six `github.com/.../blob/<40-hex sha>/...` links (lines 977, 990, 991, 1016, 1018 and 1038). The base spec tree had none. The repository CLAUDE.md forbids introducing SHAs or pins that track files or versions, and the base checklist records any such hit as high. The normative content is already carried by the quoted text and the file and section names. The pinned revisions freeze FR-038 to superseded upstream text when FR-152, FR-322, QSL FR-094 or FCD FR-094/FR-095 change. The author's receipt says the citations were requested. That is ticket or peer prose, and this review does not grant an exception.

## Dispositions

Round 1, reviewed at `agent-ix/quire-contract-ir@c50050da5f20d4af41ab2dd6ea573d5e4b9abacc` (prior review `705cef3f7a07370f111f19e8e7d86e1e07fd31d7`), reviewer model `claude-opus-5-5`, run `ae1a4470-06bc-44c7-90df-81c70be53842`. Every outcome was verified against the fix commit's own text, not the author's receipt. All new criteria and procedures remain PLANNED / UNRUN; no implementation, CI gate or procedure coverage is claimed.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed c50050da5f20d4af41ab2dd6ea573d5e4b9abacc | Both blank lines are gone. `quire properties --scope . --json FR-038` at c50050d reports 167 criteria (158 at base), each of FR-038-AC-165..173 exactly once, with no duplicates. |
| FND-002 | fixed c50050da5f20d4af41ab2dd6ea573d5e4b9abacc | All six commit-SHA blob links are replaced by `blob/main` file/section links. `git grep` finds no 40-hex blob link in the amendment, and the only 40-hex strings left in FR-038 (lines 393, 442, 786) predate this change. |
