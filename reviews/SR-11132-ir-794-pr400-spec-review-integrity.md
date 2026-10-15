---
id: SR-11132
title: "IR-794 spec-review/integrity review of neutral enumeration boundary"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/model/functional/FR-052-source-obligation-correspondence.md; spec/model/matrix/TC-452-source-obligation-correspondence.md; spec/model/matrix/tests.md"
review_set: subset
---

## Summary

Reviewed the PR #400 diff against the merged neutral contract and PR #399 synthetic TestEnumerator behavior. AC-9 through AC-11 retain owner-native Planned status.

## Verdict

**CONDITIONAL** — TC-452 omits a compile-fail or equivalent API-shape check for AC-13 private fields.

## Reviewed Units

| Unit | Role |
| --- | --- |
| FR-052-AC-9 | context_only |
| FR-052-AC-10 | context_only |
| FR-052-AC-11 | context_only |
| FR-052-AC-12 | examined |
| FR-052-AC-13 | examined |
| TC-452 synthetic procedure | examined |
| TC-452 matrix row | examined |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-452 omits a compile-fail or equivalent API-shape check for AC-13 private fields. | spec/model/functional/FR-052-source-obligation-correspondence.md:262; spec/model/matrix/TC-452-source-obligation-correspondence.md:84 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-13 and TC-452 now require positive accessor compilation plus separate direct-field and struct-literal compile-fail controls, each refusing specifically for field privacy. |
