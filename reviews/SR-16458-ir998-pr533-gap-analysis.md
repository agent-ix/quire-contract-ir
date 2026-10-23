---
id: SR-16458
title: "IR-998 PR 533 incremental gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@e76875c2226178ef5d214b04d8b352491b3e51cf; FR-038-AC-313..316; FR-056-AC-3/7/8; four changed files"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: references }
  - { target: "ix://agent-ix/quire-contract-ir/FR-056", type: references }
---
# SR-16458: IR-998 PR 533 incremental gap analysis

## Summary

Planless, diff-scoped trace and reverse-gap review of the hybrid/flat-wire typed-consumption slice at e76875c2226178ef5d214b04d8b352491b3e51cf. The code is owned by FR-038's hybrid reader checks and FR-056's post-intake scan/marker criteria.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-038-AC-313/314/315/316: existing `#[trace]` QSpec conformance tests bind each criterion in `tests/conformance_qspec/hybrid.rs`. New exact rate test binds AC-314 and checks public read results after rekeying. No new untraced reader behavior or hollow assertion found in the diff.
- Reconciliation: `quire matrix --scope . --format json` on this exact head returned 559 tagged, 87 untagged, 16 method-without-symbol and one tagged-by-ignored-test across the repository. The four examined FR-038 criteria are tagged by 4/5/4/8 tests respectively. FR-056-AC-3 and AC-8 are untagged, and AC-7 is method-without-symbol. This incremental PASS does not claim a repository-wide matrix PASS.
- FR-056-AC-3/7: the one added `#[string_edge]` marks strict-wire `hybrid::integer`, which converts a raw wire spelling under an exact integer grammar. The 18 semantic/flat-wire post-intake decisions are converted to retained typed values/kinds rather than exempted. No branch rule, broad marker, or local scanner copy was added.
- FR-056-AC-8: the author-reported same-tool scan changes 268 to 249 rows; I inspected the diff for 19 targeted removals but did not independently certify the scan count. The repository-wide scan remains red, so this PR makes no AC-8 completion claim.
- Plan completion: not assessed. This is an incremental subset review. The final reviewed-head gate remains with the author.

## Verdict

PASS for this incremental slice; FR-056-AC-8 remains open for other owners.
