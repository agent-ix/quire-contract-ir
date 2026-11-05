---
id: SR-2026110414560020
title: "Gap analysis of IR-1326 reader control PR 900"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; tests/it/checked_package_v2_temporal.rs; FR-038-AC-104, FR-038-AC-122"
review_set: subset
---

## Summary

A manual criterion-to-test check compared the new trace tags and assertions with IR FR-038-AC-104/122 and the merged QSL FR-337-AC-24. The adverse packages are exercised, but one trace tag claims a different timed form.

## Verdict

**CONDITIONAL** — Remove the inaccurate FR-038-AC-122 trace tag from the new test, or add an assertion that directly exercises that criterion's rational four-field timed form. Its existing dedicated test remains present.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | New test tags AC-122 but exercises a distinct timed_interval member, not AC-122's four-field rational temporal_interval form | tests/it/checked_package_v2_temporal.rs:2194 |

## Coverage

Plan completion: not assessed. Scoped manual criterion-to-test review; no production code changed. FR-038-AC-104's finite-window open interval refusal is asserted at the application body. QSL FR-337-AC-24's finite-window timed_interval mutation is asserted at the operation member. FR-038-AC-122's actual rational timed form is tested in `tc_048_the_timed_form_fits_only_the_timed_profile`. The QSL prose's member-locus statement for the open interval conflicts with IR's application-locus rule and is an upstream cross-spec question, already routed to planning; it is not a defect in this IR test.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Commit 81c43a67 removes the FR-038-AC-122 tag; the test retains TC-048 and FR-038-AC-104, and its two refusal assertions are unchanged. |

## Current verdict

**PASS** — The only review finding is fixed. The retained FR-038-AC-104 trace matches the open-interval refusal, and the timed-form AC-122 remains covered by its dedicated test.
