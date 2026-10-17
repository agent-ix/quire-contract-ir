---
id: SR-12121
title: "IR-840 spec integrity review of IR PR #440"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@e1e0eddf3a0672be7648f3bcaf1804d944f548d9 (PR #440); git diff origin/main...HEAD: spec/checked_package/matrix/tests.md:32; consistency against spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Acceptance Criteria Verification column), spec/checked_package/matrix/tests.md:21 (FR-038 coverage row) and :40-92 (per-criterion rows), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

# IR-840 spec integrity review of IR PR #440

## Summary

Ticket: IR-840. This review checks that the edited TC-048 index agrees with the
other places that bind FR-038 criteria to test cases. The id checks are clean:
all 188 ids exist, the cell has no duplicates, and the ids are in ascending
order. The only id with a second claim, AC-183, is shared with TC-226 and is
consistent with FR-038's own `Test (TC-048, TC-226)`. No id is double-claimed
in a way that contradicts. Adding AC-202..209 would have created a
contradiction with FR-038's `Test (TC-445)`, and the PR correctly does not add
them.

This change makes three pre-existing integrity gaps visible. They sit next to
the edit rather than in it, so they are recorded here as low findings and do
not block the PR. The missing ids that do block it are in SR-12120.

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| TC-048 Traces To cell | examined | spec/checked_package/matrix/tests.md:32 | 188 FR-038-AC ids, ascending, no duplicates, none nonexistent |
| FR-038-AC-183 shared claim | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | Test (TC-048, TC-226) |
| FR-038-AC-202..209 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4568-4575 | Test (TC-445) |
| FR-038 Verification column, AC-177..182, 186..201, 214, 246 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4543-4567, 4587, 4631 | Test / Test, Inspection / Inspection (no TC id) |
| FR-038-AC-236..238 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4604-4606 | IMPLEMENTED (IR-777) ... Test |
| FR-038 coverage row | examined | spec/checked_package/matrix/tests.md:21 | ... FR-038-AC-215 through FR-038-AC-218, FR-038-AC-239 through FR-038-AC-241, FR-038-AC-246, FR-038-AC-252 ... |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The TC-048 cell now claims AC-177..182, AC-186..201, AC-214 and AC-246. FR-038's own Verification cells for these criteria name no test case: they read `Test`, `Test, Inspection` or `Inspection`. Other FR-038 rows name their TC, for example `Test (TC-048)`. The matrix index and the per-criterion Verification column therefore disagree about which TC owns these criteria. Pre-existing. Fix in a follow-up: add `(TC-048)` to those Verification cells. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4543-4567, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4587, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4631 |
| FND-002 | low | No test case owns FR-038-AC-236..238 (IR-777, IMPLEMENTED). Their Verification cell is `Test`. No TC row, TC document or tests.md per-criterion row names them. The tagged tests (`checked_package_v2_model_members.rs:543`, `:476`, `:668`, `checked_package_v2_enum_order.rs:284`) carry only `/// Trace: FR-038-AC-23x`. This PR is right to leave them out of TC-048, because no procedure there covers them. Pre-existing. Fix in a follow-up: author an owning TC procedure (most likely a TC-048 section), then add the ids. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4604-4606, tests/it/checked_package_v2_model_members.rs:543 |
| FND-003 | low | The FR-038 coverage row (tests.md:21) does not list AC-214, AC-222..238, AC-242..245 or AC-247..251. FR-038 criteria rows exist for all of them, and per-criterion tests.md rows exist for AC-222..224 and AC-247..251. The aggregate row is stale, so it cannot serve as the source the ticket says to derive this cell from. Pre-existing. Fix in a follow-up: extend the row's ranges with a status note. | spec/checked_package/matrix/tests.md:21 |

## Verdict

PASS WITH FINDINGS for integrity. The edited cell is internally consistent and
claims nothing that conflicts with another TC's claim. The three low findings
are pre-existing and should be deferred to a follow-up ticket. The cell is
still incomplete (SR-12120 FND-001..003), and that keeps the PR from merging.
