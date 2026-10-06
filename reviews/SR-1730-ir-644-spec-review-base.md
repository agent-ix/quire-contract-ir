---
id: SR-1730
title: "Base spec review of IR-644 optional record leaf walk"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@55c22b8de810ff3b1fcf9e19eb470705a6aeb5ca; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

Ticket: IR-644. Reviewed FR-038's new operation-leaf paragraph, FR-038-AC-151 and AC-152, the new TC-048 procedure, and the FR-038 coverage row in TM-002 against QSpec FR-322's body grammar and the existing recursive-leaf rules. Both new criteria and the procedure explicitly say PLANNED; no implementation claim was inferred. The local QSpec checkout at ece259a contains no `positive-recursive-records.json`, so exact fixture bytes were unavailable for this review.

## Scope examined

| Unit | Role | Evidence |
| --- | --- | --- |
| FR-038 operation leaves | examined | `spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2643` |
| FR-038-AC-151 | examined | `spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3126` |
| FR-038-AC-152 | examined | `spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3127` |
| TC-048 optional record fields | examined | `spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:573` |
| TM-002 FR-038 row | examined | `spec/checked_package/matrix/tests.md:16` |
| QSpec FR-322 body grammar | context_only | `quire-specification/spec/objects/interfaces/FR-322-checked-package-artifact.md:380` |

## Verdict

**CONDITIONAL** — The wrapped edge, path order, malformed-edge refusal, and planned status are specified, but the coverage row does not name the new criteria.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TM-002's FR-038 coverage row stops at AC-150 and omits planned AC-151 and AC-152, although both now name TC-048. The row therefore gives an incomplete criterion inventory and cannot report the new criteria's planned status. Add both IDs and their planned/no-test-yet state. | spec/checked_package/matrix/tests.md:16; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3126; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:573 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ccd719ef20580cad745a6e3191bad9c1e8a64659 — TM-002 FR-038 and TC-048 rows now name AC-151 and AC-152 and state that they are planned with no executable test. |

## Round 1 evidence correction

The first-pass summary's fixture-availability sentence was mistaken: the QSpec checkout's working-tree HEAD was stale. At QSpec `origin/main` ece259a038b5e3592ffa97b102fa9e3357ba1434, `proposals/checked-package-v2/fixtures/positive-recursive-records.json` exists and its `List.next` body is a `binding` whose `aggregate` contains a single `optional` binding to an option reference. The round 1 review read that Git object without copying it into IR.

## Round 1 verdict

**PASS** — The base finding is fixed at ccd719ef20580cad745a6e3191bad9c1e8a64659; no new defect was found in the fix delta.
