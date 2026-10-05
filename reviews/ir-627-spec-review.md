---
id: SR-1552
title: "Base spec review of quire-contract-ir PR #295 (IR-627 anonymous structural node bodies)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@984c283099ce117b5ab7cba2b8f03fe3d6e5e5bc; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (section 'Anonymous structural node bodies and keys', FR-038-AC-123..130), spec/checked_package/matrix/TC-226-checked-package-v2-anonymous-structural-node-bodies.md, spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

Ticket: IR-627. Base checklist over `git diff origin/main...HEAD` of PR #295 (three spec files, no
code, no CI edits; re-measured). IDs FR-038-AC-123..130 and TC-226 are new and unique in the repo;
`quire validate` passes; `quire coverage --strict` goes from 23 unbacked rows at the merge base to 32
at head, and the 9 new rows are exactly FR-038-AC-123..130 and TC-226 (all planned). The FR-038 matrix
row range, the TC-226 summary row and the TC-226 cases row agree with the criteria. AC-128..130 are
honestly marked planned and gated with no test. Two index documents outside the edited matrix were
not updated, and AC-129 names a verification method that its own text does not use.

Analyses run: base (this file), integrity (SR-1553), scope-boundary (SR-1554), ears-conformance
(SR-1555), failure-domain (SR-1556). criterion-strength was not run: the installed
`spec-criterion-strength-analysis` skill states it is blocked on its Jev client and must not be run
yet. Its strength observations are folded into FND-002 below and into SR-1556.

## Verdict

Changes requested overall (high findings in SR-1554 and SR-1556). This file's own findings are low.
Clean units: ID format and uniqueness, the coverage delta (exactly the new planned rows), the FR-038
matrix row and the TC-226 summary and cases rows, and that no CI or workflow file changed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | spec/tests.md Checked package row still ends "the other rows are implemented" and lists no AC-123..130 planned rows, and the StR-001 row of spec/core/matrix/tests.md lists TC-225 but not TC-226; both indexes now state something false about the matrix | spec/tests.md:15, spec/core/matrix/tests.md:12 |
| FND-002 | low | FR-038-AC-129 is verified `Test (TC-226)` but its evidence clause includes "an inspection that the source has one `quire.structural-node/v1` preimage type"; an inspection is not a test, so the method cell does not match how the criterion is discharged | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2453 |

## Dispositions

Round 1, reviewed at ab860cac3a7162c5eea073d99bb5bcf5433e1237.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ab860cac3a7162c5eea073d99bb5bcf5433e1237 |
| FND-002 | fixed | ab860cac3a7162c5eea073d99bb5bcf5433e1237 |
