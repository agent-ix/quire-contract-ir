---
id: SR-15654
title: "Base spec review — IR-276 independent V2 record lowering PR 517"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/TC-050-checked-package-v2-lowering.md, spec/checked_package/matrix/tests.md, spec/model/functional/FR-019-rust-library-interface.md; FR-038-AC-309, FR-019-AC-5"
review_set: subset
---

## Summary

Reviewed the new independent-record behavior and AC, TC-048/TC-050 procedures, matrix note, and FR-019 public-surface row in PR #517. Ticket: IR-276. The changed requirements are specific and consistent with the existing one-key lowering and whole-call package semantics.

## Verdict

**PASS** — the new criterion names the input, output, byte boundary, order, and sharing properties, and the TC procedures name discriminating assertions. No new hash, copied artifact, or pin was introduced.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope and evidence

- Examined FR-038-AC-309 and its new Behavior paragraph: `lower_independent_records` returns separate one-key-equivalent records in input order while sharing indexes/closure work; aggregate over-ceiling failure cannot erase a fitting individual result.
- Examined TC-048's new one-key versus union-ceiling procedure and TC-050's new shared-closure procedure.
- Examined FR-019's new `CheckedPackageV2::lower_independent_records` public-item row and FR-019-AC-5 as context. The table's exception for separately owned inherent methods makes the row consistent with its surrounding inventory rule.
- The changed spec documents validate with Quire. Existing unrelated full-repository spec status is outside this PR review.
