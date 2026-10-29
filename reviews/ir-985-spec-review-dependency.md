---
id: SR-16268
title: "Dependency review of IR-985 PR 527"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-contract-ir; FR-038-AC-311..317, TC-048 hybrid procedure, FR-019 HybridForm inventory; QSpec FR-481/TC-966"
review_set: subset
---
# SR-16268: Dependency review of IR-985 PR 527

## Summary

Ticket: IR-985. Reviewed IR against QSpec #415. The added FR-481 and TC-966 edges resolve at QSpec #415 head and allocate wire authority and fixture evidence to QSpec.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

- FR-038-AC-311 through FR-038-AC-317, examined: all seven new reader admission, identity, order and fixture criteria.
- TC-048 hybrid procedure, examined: published Braking, Braking2, Tank and HY vector assertions.
- FR-019 HybridForm inventory, examined: planned public enum under checked_package vocabulary.
- QSpec FR-481 and TC-966 at #415 head, context only: authoritative node forms, reader order and vectors.

## Verdict

Clean for this method on the examined diff.

## Upstream dependency

Verified after the initial review at QSpec #415: FR-481 specifies `ill_typed`/`non-boolean-root` for non-Boolean hybrid invariant, guard and initial condition, but the published `proposals/checked-package-v2/schema.json` Diagnostic cause-tag enum and cause pairing omit `non-boolean-root`. QSpec must reconcile its schema with its FR-481 refusal contract before the affected diagnostic vectors can be schema-valid. No IR spec finding is added because this PR quotes the QSpec normative contract correctly.
