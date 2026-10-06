---
id: SR-1731
title: "Integrity review of IR-644 optional record leaf walk"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@55c22b8de810ff3b1fcf9e19eb470705a6aeb5ca; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

Ticket: IR-644. Checked the new leaf-walk rule and AC-151/152 with TC-048 for one interpretation, observable oracles, malformed-shape coverage, and precedence against QSpec FR-322's body grammar. The wrapped field's `field:next`, `inner`, `recursion:0` path follows the pre-existing reentry definition. Two parts of the proposed tests leave the stated rule under-verified.

## Scope examined

| Unit | Role | Evidence |
| --- | --- | --- |
| FR-038 operation leaves | examined | `spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2643` |
| FR-038-AC-151 | examined | `spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3126` |
| FR-038-AC-152 | examined | `spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3127` |
| TC-048 optional record fields | examined | `spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:573` |
| QSpec FR-322 body grammar | context_only | `quire-specification/spec/objects/interfaces/FR-322-checked-package-artifact.md:380` |

## Verdict

**CONDITIONAL** — The direct-reference distinction and four malformed-variant precedence checks need explicit, falsifiable oracles.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-151 says a direct field reference to `Option<List>` is never classified as an optional-presence wrapper, but TC-048 checks only its `field:next`, `inner` path. Both forms are specified to yield that same path, so a reader that wrongly classifies a direct reference as a wrapper can pass the control. State an observable distinction or remove the internal classification claim from the criterion. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3126; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:585 |
| FND-002 | medium | AC-152 and TC-048 allow four malformed variants to pass by asserting an unspecified earlier typed refusal and pointer whenever an earlier stage rejects them. That conditional oracle can accept a premature grammar refusal for a shape that should reach the operation check. Specify the expected stage, code and pointer for each variant, or require operation-stage refusal for every variant that the published wire/body grammar admits. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3127; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:592 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ccd719ef20580cad745a6e3191bad9c1e8a64659 — AC-151 and TC-048 now require observable admission with the same field and inner path, without the unobservable classification claim. |
| FND-002 | fixed | ccd719ef20580cad745a6e3191bad9c1e8a64659 — AC-152 and TC-048 now require every listed grammar-valid mutant to reach the operation check and refuse `ill_typed`/`operator-ineligible` at `operation.leaves`; an earlier refusal fails. |

## Round 1 verdict

**PASS** — Both integrity findings are fixed at ccd719ef20580cad745a6e3191bad9c1e8a64659; no new defect was found in the fix delta.
