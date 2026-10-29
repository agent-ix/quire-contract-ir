---
id: SR-558003
title: "Spec EARS review of IR-1013 PR #558"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-058-admitted-clause-execution-point.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-058
    type: reviews
---
# SR-558003: Spec EARS review of IR-1013 PR #558

## Summary

Ticket: IR-1013. This review checks the EARS grammar of the FR-058 requirement
statements.

- The two Description statements are ubiquitous SHALL and SHALL NOT. Each has
  one subject and one response.
- Behavior 2-4 use the unwanted-behaviour form correctly: "If ..., then
  Contract IR SHALL refuse X".
- Behavior 5 is event-driven, plus one ubiquitous SHALL NOT.
- Behavior 6 and 7 are event-driven: "When resolving a point, Contract IR SHALL /
  SHALL NOT ...".

## Verdict

**PASS with one low finding.**

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Behavior 1 mixes an event trigger with a trailing condition in one sentence: "When the accessor receives a clause id, Contract IR SHALL resolve it ... and refuse `UnknownClause` if absent". The refusal is an unwanted-behaviour response and should read "If the clause id is absent from the admitted package, then Contract IR SHALL refuse `UnknownClause`", matching Behavior 2-4 | spec/checked_package/functional/FR-058-admitted-clause-execution-point.md:52-53 |

## Examined scope

- **FR-058** Description statements (examined): exact `ExecutionPoint` exposure; no inference from kind, anchor, source or caller fallback.
- **FR-058** Behavior 1 (examined): resolve clause id; `UnknownClause`.
- **FR-058** Behavior 2 (examined): `NotStateClause`.
- **FR-058** Behavior 3 (examined): `MissingClaimOccurrence`.
- **FR-058** Behavior 4 (examined): `MissingExecutionPoint`.
- **FR-058** Behavior 5 (examined): exact variant and `AnchorName`; no kind or anchor derivation.
- **FR-058** Behavior 6-7 (examined): read only the retained relation; no JSON decode, recheck, export, mutation or parallel authority.

## Dispositions

Round 1, checked against the repo (diff `..`).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Behavior 1 now reads "If the clause id is absent from the admitted package, then Contract IR SHALL refuse `UnknownClause`." (lines 75-76) |
