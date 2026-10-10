---
id: SR-5021
title: "IR-347 PR 331 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@4c749b2d2360f3ac05d3b712045e84ab6af3922b; FR-019, FR-028, FR-039, TC-041, TC-058, computed matrix and changed source"
review_set: subset
---
# SR-5021: IR-347 PR 331 gap analysis

## Summary

Ticket: IR-347. Plan completion: not assessed. The computed Quire matrix at this exact head reports 315 tagged, 44 untagged, 2 tagged by ignored tests, and 6 method-without-symbol criteria. The changed TC-058 tests provide FR-019-AC-5 evidence; the new FR-039-AC-1 tag is unsound as recorded in SR-5020 FND-001. No new implementation stub or copied owner contract was found in the diff. Optional whole-repository semantic review was not requested.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | TC-058 and its matrix rows still say Planned after this PR implements and tags the model public-interface test; update their status to the measured partial/completed state. | spec/model/matrix/TC-058-model-crate-public-interface.md:39-41; spec/model/matrix/tests.md:17,29; spec/tests.md:13 | wrong-requirement |
| FND-002 | low | FR-028's status paragraph and TC-041 matrix still describe the now-deleted bridge re-export test and assert that the root re-exports the model; update them while preserving AC-2 and AC-5 as unbacked. | spec/model/functional/FR-028-separate-cycle-free-contract-model.md:75-77; spec/model/matrix/tests.md:19,28 | wrong-requirement |

## Coverage

- FR-019-AC-5/TC-058, examined: named export inventory matches the merged FR-019 table; one legal and one compile-fail path are present. The matrix marks AC-5 tagged.
- FR-028-AC-2/AC-5/TC-041, examined: deleted test carried stale tags and asserted the wrong root re-export. Both are now untagged; no valid criterion evidence was lost.
- FR-039-AC-1/TC-055, examined: matrix marks AC-1 tagged via an incomplete source inspection. SR-5020 FND-001 owns the substantive trace correction; AC-2 through AC-4 remain untagged.
- Reverse gap, examined: changed code is attributable to FR-019-AC-5 and the FR-028/FR-039 one-path boundary; no new unspecified behavior or stub found.
- Plan completion: not assessed. Whole-repository semantic review: skipped, as optional and not requested.

## Verdict

CONDITIONAL. Correct the two stale status descriptions and the FR-039 trace issue in SR-5020. The repository-wide 44 untagged criteria are pre-existing backlog, not introduced by this PR.

## Dispositions

Reviewed the same PR at agent-ix/quire-contract-ir@794bf6ef3a6a0abaa5333d5bbc39e6372a26bd3c.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 794bf6e — TC-058 `## Status`, both model matrix rows, and the global model row now describe the three tests and two doctest paths as implemented. |
| FND-002 | fixed | 794bf6e — FR-028 `## Status`, TC-041 matrix rows, and case description no longer assert a root model re-export or cite the deleted bridge test; AC-2 and AC-5 are expressly unbacked. |
