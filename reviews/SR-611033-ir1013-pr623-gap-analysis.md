---
id: SR-611033
title: "IR-1013 PR 623 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #623; FR-058-AC-3/4/5/9/10 and TC-462"
review_set: subset
---

## Summary

Planless criterion-to-test audit of the accessor PR. The new TC-462 tests bind AC-3, AC-4, AC-5, AC-9 and AC-10. Their assertions exercise the public accessor, refusal precedence, clone stability, exact point variants, and same-valued claims on distinct node owners. One AC-9 tag claims more than its test proves.

## Verdict

**CONDITIONAL** — add an accessor assertion to the existing point identity and rekey controls, or expand TC-462 to prove AC-9's full identity and changed-name behavior through the accessor before merging.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The AC-9 tagged accessor test checks exact values and a nonidentifier name but does not assert distinct node and package identities or changed-name rekeying through the accessor | tests/it/checked_package_v2_frame_entries.rs:3728 |

## Coverage

- FR-058-AC-3 (examined): TC-462 asserts all three precedence levels and supplied error fields.
- FR-058-AC-4 (examined): TC-462 asserts missing point and two authentic claims of one pointed node.
- FR-058-AC-5 (examined): TC-462 calls the public model API from an integration test and checks repeat/clone stability.
- FR-058-AC-9 (examined): TC-462 exact value and unusual-name assertion is partial; TC-056 checks typed carrier identity separately but is not an accessor test and has no AC-9 tag.
- FR-058-AC-10 (examined): TC-462 checks wrong owner and nonclaim refusal; earlier wire admission controls under FR-040 are context only.
- FR-058-AC-1/6 (context only): fresh QSL package origin remains QSL-1183, so synthetic fixtures earn no producer credit.
- TC-462 (examined): three new tagged integration tests.
- Plan completion: not assessed. Optional repository-wide semantic review: not invoked; changed criterion bindings were inspected in this PR review. Computed matrix was unavailable within the review time budget because quire matrix did not complete promptly; static trace tags were inspected directly.

### FND-001

The test tagged FR-058-AC-9 constructs four independent packages and compares returned points. It does not compare node IDs or package IDs between initialization and handler, and it does not rekey after changing only the handler name and assert the new accessor result. TC-056 checks carrier identity for the first distinction, but its FR-040 tags do not establish the accessor half of AC-9. A wrong accessor that returned a point from another node after rekeying could leave the current AC-9 tagged test green.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | adds an AC-9 tagged test for invariant and constraint, checks shared nonpoint arguments, distinct node and package identities across initialization/handler and handler-name changes, and reads the exact point through the accessor on each admitted rekeyed package. |

## Disposition verdict

**PASS ** for the recorded finding. This is a focused disposition on the test fix, not a repeat review or aggregate gate result. Fresh QSL producer AC-1/6 evidence remains with QSL-1183.

## Post-disposition compile correction

The AC-9 test computes `positions` by borrowing `sources` through `std::array::from_fn` instead of consuming the array with `map`. All node/package identity inequalities and accessor value assertions remain intact. FND-001 stays fixed. Hosted CI and the focused local test were still pending at this inspection; this note makes no gate claim.
