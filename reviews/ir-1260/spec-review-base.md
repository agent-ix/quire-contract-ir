---
id: SR-2026110101260
title: "IR-1260 text literal membership base spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; FR-038-AC-409 through FR-038-AC-411 and TC-048 text literal membership"
review_set: base
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: reviews
---

## Summary

Reviewed the IR-1260 specification diff against QSpec FR-322-AC-304 and the referenced text literal vectors. The final requirement and test case cover normalization, inclusive scalar or byte bounds, family-check precedence, exact refusal locus and by-reference vector use without copying the upstream fixture. No applicable `AssuranceProfile` document was found under this repository's `spec/` or `reviews/` trees; consequently no enforced review policy applies to this diff.

## Verdict

**PASS after fixes.** All three findings below were corrected in the final diff. Acceptance criteria AC-409 through AC-411 have unique IDs and direct TC-048 verification edges. The QSpec reference edge targets FR-322-AC-304; the test case also references QSpec TC-233. No wire or schema copy was added.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Fixed: AC-409 through AC-411 initially cited TC-048 without corresponding verification edges or a text literal test plan. TC-048 now links all three and specifies a public strict-reader exercise. | FR-038-AC-409..411; TC-048 frontmatter and Text literal membership |
| FND-002 | medium | Fixed: QSpec TX vectors alone omit NFKC, NFKD, nested literals, plain text and family-before-value precedence. TC-048 now requires self-built controls for these cases in addition to reading the vectors by reference. | FR-038-AC-409..410; TC-048 Text literal membership |
| FND-003 | high | Fixed: an interim TC-048 revision required recording the selected checkout HEAD in execution evidence, contrary to the base checklist's recorded-revision rule. The final text selects the checkout without requiring a revision record. | TC-048 Text literal membership |

## Coverage

- **FR-038-AC-409:** TC-048 covers root and nested literals, all four normalizing profiles, the two non-normalizing profiles, inclusive bounds, and plain unbounded text.
- **FR-038-AC-410:** TC-048 covers below-minimum, above-maximum and non-normalized refusals with code, cause, pointer and graph-node locus, plus family-check precedence.
- **FR-038-AC-411:** TC-048 reads TX-01 through TX-14 and TX-13s from the selected QSpec checkout, builds complete local packages, checks exact outcomes and retains no vector, schema or normalization-table copy.
