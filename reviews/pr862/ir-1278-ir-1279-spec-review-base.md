---
id: SR-20261103150653
title: "IR-1278 IR-1279 PR 862 FR-019 spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #862; spec/model/functional/FR-019-rust-library-interface.md; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-019", type: references }
---

## Summary

Reviewed the FR-019 public inventory and AC-25 status edit, plus FR-038 temporal-binding and clause-name status edits, against the new public type, accessor, validator, QSpec vectors and default-feature external test in PR #862. Tickets: IR-1278 and IR-1279.

## Verdict

**PASS after dispositions** — the public inventory wording is corrected and all six new FR-038 criteria are in the declared Acceptance Criteria table.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-25 says no new root export appears, but the PR adds `TemporalClauseName` to the model crate root re-export. Resolve whether the type is intended to be root-exported and make the criterion describe the chosen API. | spec/model/functional/FR-019-rust-library-interface.md:382 |

## Coverage

- FR-019-AC-25, examined: implementation status, external typed producer and borrowed consumer, exact public inventory, and no-extra-API assertion.
- FR-019 public inventory row at line 290, examined: TemporalClauseName, borrowed accessor and encoder sidecar input match the implementation.
- FR-019-AC-22, context only: temporal-binding borrowed accessor continues to exist.
- FR-038-AC-442 through AC-447, context only: clause-name carrier acceptance criteria.
- FR-038-AC-412 through AC-416, examined: compensation row admission, absence, duplicate, coverage, clock-name and profile refusals, and QSpec CB vector replay.
- FR-038-AC-437 through AC-447, examined: closed occurrence-keyed temporal binding and clause-name carriers, ordered joins, coverage, work accounting, and published BO/CN vector replay.
- No applicable AssuranceProfile in this repo describes a spec-review policy for these narrow status edits.
- Targeted `quire validate --scope .` on both changed FR-019 and FR-038 documents: passed; one pre-existing FR-038 grammar warning at line 5620.

## Evidence

`crates/quire-contract-model/src/lib.rs:222` adds `TemporalClauseName` to the root `pub use` list. FR-019-AC-25, newly marked Implemented, ends with “no new node identity, root export or unchecked admission conversion appears.” The inventory row now records the new public type without a planned qualifier. No new requirement ID, dependency edge, digest, pin or copied file was introduced by this spec edit.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | FR-019-AC-25 now explicitly states that the model root exports the named row type, matching `lib.rs`. |
| FND-002 | still-open | At 85c6988e a blank line separated AC-442 through AC-447 from the ID-headed Acceptance Criteria table, leaving their trace targets unminted. |
| FND-002 | fixed | Commit 2b011fa2 removes the blank line between AC-441 and AC-442, placing AC-442 through AC-447 back in the ID-headed table. |

## Round 1 evidence

The FR-019 edit matches the implemented root re-export and external test. The FR-038 status edits mark AC-412, AC-414 through AC-416 and AC-437 through AC-447 implemented; inspection of the reviewed code and tagged QSpec case tests supports those claims. AC-413 now describes the mandatory occurrence field. The changed FR-019 and FR-038 documents pass targeted Quire validation; no new defect was found.

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | A blank line after FR-038-AC-441 ends the Acceptance Criteria table. AC-442 through AC-447 form a pipe block without the ID header, so Quire's declared table target does not mint these six active criteria; their test tags cannot back them. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5671 |

## Round 2 evidence

The `acceptance-criterion` trace target in the installed `spec-artifacts-process` manifest reads the `Acceptance Criteria` section's `ID` column. The table starting at FR-038 line 5240 ends at AC-441 on line 5670; line 5671 is blank and line 5672 starts with AC-442 instead of a new header. Quire validation accepts the Markdown document, but the coverage gate reported no minted target for six AC-442 through AC-447 tags. This structural trace gap escaped the earlier base review.

## Round 3 evidence

Commit 2b011fa2 removes exactly the blank line after FR-038-AC-441. The six following rows now remain contiguous with the `| ID | Criteria | Verification |` table and are minted by the declared acceptance-criterion target. The lead reports Quire properties extracted all 435 FR-038 criteria, ending with AC-442 through AC-447. The 84 unbacked rows in strict coverage were reproduced on clean origin/main and are unrelated baseline debt; this PR-specific trace regression is resolved.
