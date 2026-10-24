---
id: SR-16525
title: "IR-9 PR 536 incremental gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@ba543f3f2000eeef22a6b60e0deb1f616486eea0; FR-038-AC-160/162/163; FR-349-AC-2; FR-056-AC-8; TC-048; scalar_operands.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: references }
  - { target: "ix://agent-ix/quire-contract-ir/FR-349", type: references }
  - { target: "ix://agent-ix/quire-contract-ir/FR-056", type: references }
---
# SR-16525: IR-9 PR 536 incremental gap analysis

## Summary

Planless, diff-scoped check of the changed scalar range behavior against the computed matrix, tagged tests, source code, and the unresolved scanner criterion. This is an incremental review, not a repository-wide gap clearance.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- Reconciliation: `quire matrix --scope /home/peter/dev/worktrees/ir9-pr536-independent --format json` at the exact head. FR-038-AC-160/162/163 and FR-349-AC-2 are tagged. FR-056-AC-8 is untagged and remains open; this PR does not claim its acceptance.
- FR-038-AC-160/162 and TC-048: existing admitted-package tests assert graph-child identity, finite type bounds, unbounded Integer, out-of-`i128` range, and distinct refusals. New private tests assert both retained-kind/raw-spelling mismatches through `scalar_application_operands`, with expected ranges and error codes. The tests-first commits precede the production fix; the old raw branch would fail each inverse mismatch assertion by inspection.
- FR-038-AC-163: existing operator refusal and eligibility tests remain unchanged; this diff does not alter catalog dispatch.
- FR-349-AC-2: context only for retained typed interpretation; the new type decision no longer reads the raw node tag or semantic form. No new body decoder or admission rule was added.
- FR-056-AC-8: source diff removes the two raw tag/form tests within `type_range` (four reported scanner expressions according to the author's ticket evidence), and adds no string decision or exemption. The installed external scanner was absent on this review host, so the reported 245-to-241 count was not independently rerun or credited as a clean scan. Unrelated residual reports keep AC-8 open.
- Reverse gap/stub check: the new code changes an existing typed scalar accessor, with no new externally visible behavior without an owning FR and no stub. The three tests execute the real accessor and can fail on the prior branch.
- Plan completion: not assessed.

## Verdict

PASS for the incremental PR slice. FR-056-AC-8 remains unfulfilled and is not accepted by this review.
