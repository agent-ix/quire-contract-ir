---
id: SR-1153
title: "AC-form review of PR 276 (FR-038-AC-114 to AC-118 and amended ACs)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@dd407ae54c8e813c9d6e8efb97d6d435d6947ec1; spec/checked_package/functional/FR-038-consume-checked-package-v2.md FR-038-AC-3, AC-26, AC-78, AC-86, AC-88, AC-100, AC-114 to AC-118; spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md FR-040-AC-10"
review_set: subset
---
# SR-1153: AC-form review of PR 276

## Summary

Ticket: IR-495. Checked the new and amended acceptance criteria against the
repo's convention. An AC is a direct, observable assertion with no "shall",
planned rows carry the planned marker, and AC rows name no tracker state. The FR
statement keeps its "The reader shall ..." EARS form, and the PR does not change
it. `quire validate` reports one grammar finding repo-wide (FR-014, baseline)
and none in the changed files.

- FR-038-AC-114 to AC-118: each is a direct assertion of code, pointer and
  outcome. Each carries the planned marker, and none uses "shall".
- FR-038-AC-3, AC-26, AC-78, AC-86, AC-88 and AC-100 (amended): direct
  assertions, and no planned marker is needed for the removed clauses. The
  testability and content defects of AC-100 and AC-117 are recorded in SR-1152.
- FR-040-AC-10 (amended): a direct assertion. The nested-clause clause is
  marked planned inline.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Clean on form. Content findings are in SR-1152.
