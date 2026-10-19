---
id: "SR-13242"
title: "IR PR 480 integrity: FR-035 AC ids, TC-047 traces and matrix rows"
type: SpecReview
analysis: integrity
review_set: subset
scope: "agent-ix/quire-contract-ir PR #480; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md, spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md, spec/checked_package/matrix/tests.md (diff origin/main...HEAD)"
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: reviews
---

# SR-13242: IR PR 480 integrity

## Summary

Ticket: IR-889. PR: agent-ix/quire-contract-ir#480 at the reviewed PR head.

Renumbering note: after IR PR #479 (IR-893) merged with its own FR-035-AC-12 to AC-14, this
PR's criteria were renumbered. The AC-12 and AC-13 named in this review, including its findings
and dispositions, are now FR-035-AC-15 and FR-035-AC-16. This review records the ids as they
stood when it was written.

Checked:

- **Ids.** On the reviewed base, FR-035 ends at AC-11, so AC-12 and AC-13 are the next free
  ids. No other spec file references FR-035-AC-12 or AC-13.
- **TC-047.** The Description range now covers "AC-10 through AC-13". The tests.md TC-047
  "Traces To" list adds AC-12 and AC-13. The FR-035 row range becomes "AC-1 through FR-035-AC-13".
- **Status prose.** The tests.md FR-035 row, the TC-047 row and the TC-047 Status all say the
  same thing: AC-10 and the original AC-11 behaviour are backed by the two named tagged tests,
  and AC-11's amended clauses, AC-12 and AC-13 are PLANNED/UNRUN until the IR-889 code PR.
- **Consistency.** TC-047's procedure and Expected Results match the new criteria: a cumulative
  bound, the exact limit and one below, a limit that fits each pass but not their sum, a
  value-pass body pointer, and the occurrence or region pointer.
- **AC-11 amendment.** It changes only "consumed amount of the meter that stopped" to "consumed
  amount at the failed charge" and the occurrence path from `/source_map` to the failing value.
  It replaces "The work meter's scope is open under IR-889" with a pointer to AC-12 and AC-13.
  Nothing is weakened.
- **Validation.** `quire validate --scope . 'spec/**/*.md'` passes with module advisory warnings
  only. `quire coverage --scope . --strict` gives 40 unbacked rows at the base and 42 at the
  head, with 0 contradicted in both. The two new unbacked rows are AC-12 and AC-13. The strict
  command exits 1 at the base too, so that failure predates this PR.

Informational, not a finding: IR PR #479 (IR-893) adds a different FR-035-AC-12 and AC-13 (and
AC-14), so the ids collide. The lead merges #479 first and renumbers this PR. Renumbering must
update AC-11's "AC-12 and AC-13" reference, the TC-047 Description range, the tests.md FR-035
range and TC-047 Traces To, and both status texts.

## Verdict

CLEAN on integrity. The ids are next-free on main, the traces and ranges are consistent, the
statuses are planned, and the coverage delta adds only new planned rows with 0 contradicted.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
