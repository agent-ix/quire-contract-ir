---
id: SR-731
title: "integrity review of PR 249 (IR-504 content-only ModelOwner identity)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@fdad364d27f9e77c4525d06f2653eb2e69eca569; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/tests.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; context: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md, spec/checked_package/matrix/TC-056-checked-package-v2-frame-entries-and-state-clauses.md, spec/kani/matrix/tests.md, spec/core/matrix/tests.md"
review_set: subset
---
# SR-731: integrity review of PR 249

## Summary

Ticket: IR-504. Checked consistency between the amended FR-038 and the rest of the spec,
and checked the matrix's planned and implemented marking against repo convention.

Clean: FR-038 line 97 refers to the preimage's own schema `version`, not the owner's. The
class 2 rationale (lines 215-231) is still sound, because the join is by identity, and it
now states the cross-package key equality as intended. Line 263 drops the nonempty
`version` owner requirement. The Dependencies list adds FR-322 AC-28. The FR-038 matrix row
range and the TC-048 AC list both include AC-45. AC-27 is about the lock row's `version`
checked by selection evidence, which still exists, so its text needs no change. AC-5's text
names no owner `version`, so it is consistent with the amendment.

`make spec`: validation passes at both shas. Strict coverage fails with 23 unbacked rows at
both, the known baseline. Rows backed go from 161/184 to 161/185, and FR-038 goes from 40/42
to 40/43. The new AC-45 row is unbacked, but the strict list does not name it. The PR body's
"23 before and after" is literally true.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-040 (not in this diff) still specifies refusals for a model declaration node "keyed under an unselected version" (FR-040 Behavior, FR-040-AC-3, FR-040-AC-13, TC-056 procedure). Under content-only `ModelOwner` identity (FR-322-AC-28, which also keys `ModelDeclarationNode`), a key under another version equals the key under the selected one, so these refusals cannot be constructed. When SR-730 FND-001 is fixed they contradict FR-038 directly. QSpec FR-322-AC-29 now says "keyed under an unselected domain package". | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:213,238,248; spec/checked_package/matrix/TC-056-checked-package-v2-frame-entries-and-state-clauses.md:45 |
| FND-002 | medium | The TC-048 summary row keeps status `✅` while declaring AC-45 planned with no test. Repo convention marks a TC row with any planned AC `🚧` (TC-042, TC-223, FR-029 rows). Both the TC-048 and FR-038 rows still claim AC-5 implemented, but AC-5's evidence admits a model owner carrying `version: "1.0.0"` (`tests/it/checked_package_v2_reader.rs:1195`). The amended Behavior now says that owner refuses `unknown_member`. The FR-038 row also says "AC-5's model-owner clause ... change[s]", but AC-5's text is already version-free. Only its tests change. | spec/checked_package/matrix/tests.md:15,23 |

## Verdict

Changes requested. The FR-038 text is internally consistent as far as it goes. The
inconsistency is in the dependent FR-040 version-keyed refusals, which the IR-505 handoff
does not mention, and in the matrix status marking for AC-5 and TC-048.

## New findings (disposition pass 1)

Reviewed at b091fe566b97f8ecb52c8a874f7f4ee05e8417cd. The branch is one commit behind
origin/main a91d5bd (#247, code, tests and reviews only). A trial merge is clean, and
`make spec` on the merge result gives 163/185 rows backed with 23 unbacked.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | The FR-040 matrix row still claims AC-3, AC-7 and AC-13 `✅` implemented. Their refusal clauses were reworded in b091fe5 to "keyed under an unselected domain package", but the TC-056 evidence for them keys the node under the selected identity at another version (`model_key(ORDER_NODE, "2.0.0")`, `tests/it/checked_package_v2_frame_entries.rs:1273-1279,1517,1576`). Amended FR-038 says that key equals the selected one, so the node must resolve. That is the same evidence-honesty gap FND-002 fixed for FR-038-AC-5, now on FR-040. The IR-505 scope comment covers `declaration_key` but not these fixtures. | spec/checked_package/matrix/tests.md:13,28; spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:238,242,248 |

## New findings (disposition pass 2)

Reviewed at 6ee99c7df7d6b977a4263cf768a8a241bb8a50d8, rebased onto origin/main a91d5bd, so
it is up to date. The spec delta since b091fe5 is only `spec/checked_package/matrix/tests.md`
(the FR-040 and TC-056 rows and the TC-056 coverage-design row). The four copied `reviews/`
files (SR-730, SR-732, SR-733, SR-734) are byte-identical to the reviewer's final versions.
`make spec`: validation passes, 163/185 rows backed, 23 unbacked (baseline).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | The FR-040 row says the planned clauses' evidence is `model_key` in `tests/it/checked_package_v2_frame_entries.rs`. That is true for AC-3 and AC-7 (lines 1517, 1576). AC-13's unselected case is instead `declaration_key_of(ORDER, OTHER_VERSION)` in `tests/it/checked_package_v2_model_members.rs:469,681`. The IR-505 addendum repeats the same attribution, so whoever implements IR-505 could miss the AC-13 fixture. The status marking itself is correct. | spec/checked_package/matrix/tests.md:13 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | b091fe5 |
| FND-002 | fixed | b091fe5 |
| FND-003 | fixed | b65385b |
| FND-004 | deferred | IR-505: the fixture re-key lands in IR-505's code change, and the 🚧 status is already right. Add `tests/it/checked_package_v2_model_members.rs:469,681` (AC-13) to the IR-505 addendum. No repo change is needed for this PR. |
