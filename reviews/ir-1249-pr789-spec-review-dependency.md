---
id: SR-202611010107
title: "IR-1249 PR 789 dependency review"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-contract-ir; PR #789; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
---

## Summary

Checked the new QSpec references and implementation-state dependencies across the three edited FR-038 criteria. No applicable AssuranceProfile was found under `spec/`.

## Verdict

**CONDITIONAL** — the QSpec reference edges are valid and do not introduce a cycle, but AC-279's planned-work label omits the new comparand prerequisite.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-279 now requires the IR-1249 role-end comparand but its PLANNED/UNRUN label names only IR-1141; add IR-1249 as a prerequisite, as AC-269 does. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4835 |

## Coverage

- FR-038 population comparison paragraph, role: examined. The role-end lookup is a reader prerequisite to population admission.
- FR-038-AC-269, role: examined. It explicitly marks IR-1249 before IR-1141 projection correction.
- FR-038-AC-274, role: examined. It explicitly marks the role-end comparand correction as IR-1249.
- FR-038-AC-279, role: examined. It now requires admission over the selected role-end type while the planned-work label still names only IR-1141.
- QSpec FR-322-AC-73, AC-74, and AC-270, role: context_only. Their new frontmatter references are already merged and are reference edges, not copied content or new implementation cycles.

## Evidence

AC-269 and AC-274 both name IR-1249 for the same role-end comparison. AC-279's positive unbounded admission also depends on that comparison, before its IR-1141 projection outcome can be exercised.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Head ed3b7d8b adds IR-1249 role-end comparand to AC-279's PLANNED/UNRUN label alongside IR-1141; the criterion's behavior and verification cell remain intact. |

## Disposition verdict

**PASS** — the sole dependency finding is fixed on the reviewed PR head.
