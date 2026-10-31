---
id: SR-2026103037
title: "FR-035 source-frame integrity review for IR-1186 PR 736"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md; spec/checked_package/test-cases/TC-047-complete-v1-contract-package-output.md"
review_set: subset
---

## Summary

Reviewed FR-035-AC-31..35, retired AC-12 and TC-047 against the merged QSpec source-frame and producer-order requirements. The replaced criterion retains its former obligations and the new criteria have explicit verification methods.

## Verdict

**PASS** — no independent integrity defect found. The missing AC-35 raw-collision executable control is recorded in gap-analysis SR-2026103034.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined units

- FR-035-AC-12: retirement preserves all omissions and the extra `identity` refusal in AC-31.
- FR-035-AC-31: exact six-member V1 shape and adverse controls.
- FR-035-AC-32: absent derived source key, operand closure and frame separation.
- FR-035-AC-33: output indexes, source indexes and references have defined check order and typed refusals.
- FR-035-AC-34: cross-frame joins use rebuilt structural keys and tolerate frame permutations.
- FR-035-AC-35: typed reached-closure queries use source-frame structural identity and preserve results across output permutations.
- TC-047: test method for each changed AC; the executable adverse gap is tracked separately.
