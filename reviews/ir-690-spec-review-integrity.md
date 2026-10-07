---
id: SR-3141
title: "Integrity review of the IR-690 structural inequality composite operand extension"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir branch spec/ir690-structural-ne-domains (IR-690, specification only); spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; spec/checked_package/matrix/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

## Summary

Ticket: IR-690. Checks that the amended composite-accessor text, the five new
criteria, the TC-048 procedure and the matrix rows agree with each other and
with the IR-651 criteria they extend (FR-038-AC-177 through AC-182). The new
statement, the extension paragraphs and AC-197 through AC-201 agree with one
another, and every TC-048 step traces to exactly one new criterion. The
problem is the IR-651 text the change did not amend. One IR-651 criterion
still says the opposite of the new ones, and the error table is now normative
for the extended behavior while the tagged test and the matrix status still
describe the eq-only behavior.

## Examined units

- FR-038 composite section opening statement (examined)
- FR-038 composite error table, `IneligibleOperator` row (examined)
- FR-038 structural.ne extension paragraphs (examined)
- FR-038 defensive inline-integer paragraph (examined)
- FR-038-AC-178, FR-038-AC-181, FR-038-AC-182 (examined)
- FR-038-AC-197 through FR-038-AC-201 (examined)
- TC-048 AC181 and AC182 paragraphs, and structural inequality extension steps 1 to 5 (examined)
- tests.md FR-038 row and TC-048 row (examined)
- tests/it/checked_package_v2_composite_operands.rs external error consumer test (context_only)
- checked-operation catalog structural.eq and structural.ne entries (context_only)

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038-AC-182 is unchanged and still ends "structural.ne remains a declared consumer gap rather than a silently enabled operation". The amended opening statement ("shall admit only these two operation identities"), the IneligibleOperator row and AC-197/AC-199 make structural.ne eligible. Both cannot hold. The TC-048 AC182 paragraph was edited to drop the same clause, but the criterion was not. Amend AC-182 so the clause names AC-197 through AC-201 as the only path to structural.ne eligibility, or remove it. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3869; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2910-2915; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3057; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:611-616 |
| FND-002 | medium | The IneligibleOperator row ("Known identity other than structural.eq or structural.ne") and the TC-048 AC181 procedure now describe the extended behavior with no PLANNED qualifier. Only the later extension paragraph says the current accessor returns IneligibleOperator for structural.ne. The tagged AC-181/AC-182 test asserts that structural.ne returns IneligibleOperator ("no ne allocation"), and tests.md still reports AC-177 through AC-182 as implemented and verified without noting that this assertion now contradicts the amended table. Qualify the row as the IR-690 target, or note in the FR-038/TC-048 matrix rows that the IR-690 code step retargets that assertion to a catalogued non-eq/ne identity (AC-199). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3057; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3109-3111; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:595-598; spec/checked_package/matrix/tests.md:16; tests/it/checked_package_v2_composite_operands.rs:1351-1386 |
| FND-003 | low | The defensive inline-integer rationale and AC-178 justify InlineInteger by "reader admission already refuses an integer operand of `structural.eq`" and "No publicly admitted structural.eq integer-inline success". Nothing extends this premise to structural.ne, although AC-199 and TC-048 step 3 rely on it for ne. It holds, because the catalog gives ne the same structural_kind families, but the specification should say so rather than leave it implicit. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2968-2972; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3865; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3886 |
| FND-004 | low | tests.md formatting: the FR-038 criteria cell reads "FR-038-AC-186 through FR-038-AC-196 , FR-038-AC-197 through FR-038-AC-201" (a space before the comma), and the TC-048 status cell has a double space before "IR-690 structural.ne extension". | spec/checked_package/matrix/tests.md:16; spec/checked_package/matrix/tests.md:24 |

## Verdict

CONDITIONAL. The new criteria are consistent among themselves and trace
one-to-one to TC-048 steps 1 to 5. FND-001 must be fixed before merge: as
written, FR-038 states contradictory normative behavior for structural.ne.
FND-002 should be fixed in the same round. FND-003 and FND-004 are
clarifications.
