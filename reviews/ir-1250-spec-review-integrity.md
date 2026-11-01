---
id: SR-202611010105
title: "IR-1250 PR 796 certificate vocabulary integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; IR-1250; PR #796; spec/model/functional/FR-070-certificate-member-vocabulary.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Checked the new requirement and planned inventory row for internal consistency, atomicity and testable acceptance criteria. The schema set rule, duplicate failure, and separate ordered API choice are mutually consistent; the base review records the source pin and the scope-boundary review records the cross-repo AC issue.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-070 Description, Inputs, Outputs and Behavior, role: examined. Checked vocabulary closure, failure responses and distinct set and order obligations.
- FR-070-AC-1 through FR-070-AC-4, role: examined. Checked verification methods and requirement-to-criterion coverage.
- FR-019 planned inventory row, role: examined. Checked the public API obligation is explicitly planned.

## Evidence

QSpec FR-331-AC-46 and TC-431 BP-42 define equality of two 51-name sets and explicitly leave ordering non-normative. FR-070 labels ordered equality an IR public API decision, so a schema reorder has a distinct IR test failure without misrepresenting QSpec.
