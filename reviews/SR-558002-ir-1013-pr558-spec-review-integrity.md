---
id: SR-558002
title: "Spec integrity review of IR-1013 PR #558"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@c89dc3704e74117f08e4001e46ea4a2982e3baaa; spec/checked_package/functional/FR-058-admitted-clause-execution-point.md, spec/checked_package/matrix/TC-462-admitted-clause-execution-point.md, spec/spec.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-058
    type: reviews
---
# SR-558002: Spec integrity review of IR-1013 PR #558

## Summary

Ticket: IR-1013. This review checks completeness, consistency and atomicity of
FR-058, TC-462 and the master-index paragraph at `c89dc370`.

- **Ids.** FR-058 and TC-462 are free on main and on the only other open PR (#553).
  The ids are sequential after FR-057 and TC-461.
- **AC to TC.** TC-462 step 1 covers AC-1, step 2 covers AC-2, step 3 covers AC-3
  and AC-4, and step 4 covers AC-5 and AC-6. Every refusal is observable as a
  named code with the supplied ids. Initialization and Handler are separated by
  a shared node id, kind and anchor with differing results. Determinism is
  observed over repeated calls and a clone, compared by value, equality and
  canonical identity.
- **Consistency.** Outputs, Behavior 1-4 and AC-3/AC-4 list the same four
  refusal codes in the same order. The ownership statements in Description and
  Dependencies agree with each other, and with the CG drafts.
- **Matrix.** `spec/checked_package/matrix/tests.md` (TM-002) is not updated. That
  is consistent with repo precedent: FR-054's row arrived with its
  implementation in #496, and spec-only #507 added FR-348/TC-459 without a row.
  It is not a finding.
- **Unchanged.** No existing FR or AC is changed or narrowed. The diff adds
  only.

## Verdict

**PASS with findings.** There are two low findings and nothing blocks the merge.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Behavior 5 and AC-2 frame the initialization/handler distinction only for an "invariant" node. V2 `state_clause` also admits `constraint`, which is checked as an invariant (QSpec FR-341-AC-13) and takes the same initialization/handler points. FR-058 neither includes constraint claims in AC-2 nor excludes them, so a test can pass with constraint claims never exercised | spec/checked_package/functional/FR-058-admitted-clause-execution-point.md:62-65; spec/checked_package/functional/FR-058-admitted-clause-execution-point.md:77 |
| FND-002 | low | The master-index paragraph is filed in the planned lock/conformance cluster after FR-057 (spec/spec.md:78), not in the checked-package cluster where the sibling admitted-V2 accessors FR-049, FR-051 and FR-054 are indexed (spec/spec.md:142-159). A reader of the checked-package section does not find FR-058 there | spec/spec.md:78-83 |

## Examined scope

- **FR-058** (examined): statement, Inputs, Outputs, Behavior 1-7, Dependencies.
- **FR-058-AC-1** (examined): exact four-variant producer-origin return.
- **FR-058-AC-2** (examined): shared invariant node, distinct initialization and handler points selected by claim.
- **FR-058-AC-3** (examined): ordered first-applicable selection refusals.
- **FR-058-AC-4** (examined): `MissingExecutionPoint` on absent or foreign point.
- **FR-058-AC-5** (examined): external consumer, exhaustive refusal match, determinism.
- **FR-058-AC-6** (examined): no-copy, no-compat, single-carrier inspection.
- **TC-462** (examined): procedure steps 1-4 and expected results.
- **spec/spec.md** (examined): new index paragraph and its placement.
- **spec/checked_package/matrix/tests.md** (context_only): TM-002 precedent for spec-only additions.

## Dispositions

Round 1, checked against the repo at `095a9bdc` (diff `c89dc370..095a9bdc`).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-2 now requires the startup/tick pair for both an invariant and a constraint node, with distinct node ids per QSpec FR-341-AC-13. Behavior 5 says "invariant or constraint", and TC-462 step 2 repeats the pair for a constraint node |
| FND-002 | fixed | The paragraph was removed from after FR-057 and placed directly after FR-054/TC-458 in the checked-package cluster (spec/spec.md:149-154) |
