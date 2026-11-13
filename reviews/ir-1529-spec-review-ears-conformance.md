---
id: SR-202611081512300013
title: "spec-review (ears-conformance) of IR-1529 source-declared object reader requirements"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; PR #1019; spec/checked_package/functional/FR-362..FR-368"
review_set: subset
---

## Summary

Ticket: IR-1529. EARS conformance pass over the SHALL statements of FR-362 to
FR-368, seeded by `quire validate`'s grammar warnings (quire 0.44.0) and checked
by reading. FR-363, FR-365, FR-366 and FR-368 statements name the reader or IR
as subject and parse as ubiquitous or event statements. The warnings are minor
statement-form defects with no behavioural effect.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-362's two Description/Behavior statements are wrapped so the validator sees `SHALL` with no subject (lines 41 and 59). "Every source-object rule ... SHALL apply" also names a rule, not the reader, as its actor. Name the reader as the subject. | FR-362 |
| FND-002 | low | FR-364 "SHALL be admitted in the positions ..." is agentless passive (line 44), and line 48's statement has no subject on the line. State "The reader SHALL admit ... and SHALL refuse ...". | FR-364 |
| FND-003 | low | FR-367 "A row SHALL be classified in this order and the first failure SHALL refuse" is agentless passive and carries two SHALLs (line 54); line 52 has no subject. Split it into one reader statement for classification and one for the refusal (see the base review's FND-006 for the order itself). | FR-367 |

## Verdict

Acceptable once reworded. These are low findings and do not block on their own.
