---
id: SR-16628
title: "Spec integrity — IR-645 FR-030 status"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; IR-645; spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md, spec/kani/matrix/TC-223-kani-outcome-fr331-result-map.md"
review_set: subset
---

## Summary

Consistency review of the changed FR-030 and TC-223 implementation status against adjacent normative/context text. One unchanged nearby paragraph now contradicts the PR's new implemented status.

## Verdict

**CONDITIONAL** — remove or update the stale planned-work statement in the same spec amendment.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-030 still calls AC-4/5 constructor and field validation planned, contradicting the new implemented status | spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md:142 |

## Evidence

Lines 141–144 say this API addition does not claim planned AC-4/AC-5 work is implemented and that invalid broad cause requests do not refuse. The amended Status at lines 179–181 says AC-4/5 are exercised and the broad constructor rejects invalid fixed causes. These can be read as historical chronology, but the first paragraph is present-tense specification text and gives a current reader conflicting implementation claims. TC-223's amended Status also says implemented.

## Scope

FR-030-AC-4 and AC-5 (examined), FR-030 Status (examined), TC-223 Status (examined), FR-030 public infallible constructor paragraph (examined).

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At aa3cdd1, the paragraph now states that AC-4/AC-5 rules make invalid broad non-success cause requests refuse, matching the amended Status and implementation. |
