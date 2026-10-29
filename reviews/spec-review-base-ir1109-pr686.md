---
id: SR-2026102753
title: "IR-1109 PR 686 spec-review base"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; IR-1109 PR #686 at 72334867; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5583-5588"
review_set: subset
---

## Summary

Reviewed the six FR-038 AC-357..362 status/evidence prefixes added in PR #686 at the reviewed PR head. The normative criterion text and Test verification cells remain verbatim. The matching test controls use the public V2 reader and selected QSpec vectors. No applicable AssuranceProfile or enforced spec-review policy was found in this module.

## Verdict

**PASS** — The IMPLEMENTED labels identify actual TC-048 controls for each of AC-357..362. AC-358 now has a simultaneous two-unlisted-node digest-order control; AC-357 has positive closed-form and exact round-trip assertions; AC-362 checks null cause for schema refusals. The final aggregate gate remains with the lead. EARS, integrity, dependency and other semantic lenses were not rerun because no obligation, dependency, verification method or boundary text changed. Targeted Quire validation passed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-038-AC-357: examined; closed forms, equality comparison admission and six strict-wire refusals.
- FR-038-AC-358: examined; two-unlisted-node digest first refusal and published join/omission cases.
- FR-038-AC-359: examined; region-first and structural tie-order vector cases.
- FR-038-AC-360: examined; positive anchor occurrence/urgency retention and CL-02/03/25/54 refusals.
- FR-038-AC-361: examined; typed per-node join cases CL-07..15/21..24.
- FR-038-AC-362: examined; CL-01..54 exact reader outcomes, stage controls and selected fixture digest.
- TC-048: context only; the existing matrix entry and changed tests supply the referenced controls.
- FR-038-AC-336/340/342: context only; their PLANNED/UNRUN status did not change in this PR.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | A blank line between AC-342 and AC-343 split the Markdown table. Quire validated the document but did not mint AC-343 or AC-357..362, so the new status rows were inert and trace tags unresolved. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5581 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-002 | fixed | removes the single blank line. Targeted Quire validation passes; matrix mints AC-343 as untagged and AC-357..362 as tagged. AC-343 remains honestly unbacked. |

## Final structural review

The exact delta from to removes one blank line before AC-343; the exact delta from to changes only three test trace tags. `quire trace --scope . --id` resolves both named AC-357 tests, all four AC-358 tests and all three AC-359 tests. The six IMPLEMENTED status rows now belong to minted criteria; AC-343's untagged state is preserved. No normative criterion text changed.
