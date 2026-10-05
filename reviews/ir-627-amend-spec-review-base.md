---
id: SR-1582
title: "base review of quire-contract-ir PR #298 (IR-627 recursion-group amendment)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@a5e2e32ea87a3c649a827caf296ef294d243a583; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-226-checked-package-v2-anonymous-structural-node-bodies.md, spec/checked_package/matrix/tests.md, spec/tests.md"
review_set: subset
---

## Summary

Ticket: IR-627. Base checklist over the PR #298 diff. ID formats, link integrity,
and AC-to-TC coverage were checked: AC-136 through AC-138 are each mapped to
TC-226 in the matrix and in TC-226's procedure. `quire validate` is clean at the
head. AC-136 has catchable mutants on its main axes:
- skip-everything is caught by the out-of-group `max` tamper that still refuses;
- refuse-in-group is caught by the admission of `List` and `Tree`;
- skip-applications is caught by the stale in-group application key;
- dropping the graph-shape check is caught by the label-removed refusal.

AC-138 has one mutation row per clause. The digests in this diff are the subject
of the requirement: content-addressed node keys re-derived from bytes. They are
load-bearing content identity, not pins or tracking records, so the checklist's
hash antipattern does not apply. Criterion-strength analysis was not run: it
needs Jev, which is not installed on this machine. Its strength observations are
folded into this file and into SR-1586. The soundness gap (no lone-label row) is
SR-1586 FND-001, and the ID collision is SR-1583 FND-001.

## Findings

| ID      | Severity | Summary | Refs | Escape Cause |
| ------- | -------- | ------- | ---- | ------------ |
| FND-001 | low | AC-136 ends with "The rustdoc of the stage names the skip and its consumer consequence", but its verification method is "Test (TC-226)". No test in TC-226's procedure can fail on rustdoc wording. Move that clause to an Inspection criterion, or name the source-scan test that checks it. | FR-038-AC-136 | correct-requirement-no-evidence |

## Dispositions

Round 1, reviewed at 49b85f5d96d54b655660117c5fff54ba4973eade.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 7f70b5c: The rustdoc clause is gone: AC-145, which replaces AC-136, has no rustdoc clause. |
