---
id: SR-10809
title: "IR-754/755 PR 382 FR-028 status review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR 382 spec/model/functional/FR-028-separate-cycle-free-contract-model.md Status, FR-028-AC-1 through FR-028-AC-5"
review_set: base
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-028
    type: reviews
---

## Summary

Ticket: IR-754 and IR-755. Reviewed only the changed FR-028 Status text against the new TC-041 controls. The acceptance criterion wording itself is unchanged.

## Verdict

FAIL. The blanket implemented/backed status is premature because the AC-1 and AC-5 controls have documented false negative cases. This review does not contradict the author-reported build run; that run was not independently verified here.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Status says AC-1 through AC-5 are implemented and backed by TC-041, but AC-1's host-filtered/name-limited graph control and AC-5's alias-blind source control can pass under concrete violations. Correct the controls or narrow this claim. | spec/model/functional/FR-028-separate-cycle-free-contract-model.md:75-82 | correct-requirement-no-evidence |

## Coverage

- Examined the changed Status claim and FR-028-AC-1 through FR-028-AC-5; the criteria themselves were context for assessing that claim.
- No requirement ID, relationship, or verification method changed in this diff. No applicable AssuranceProfile was found under `spec/`.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Head e66c9d3 removes AC-1's host filter, adds target-gated owner-edge evidence, and scans public callback aliases with adverse controls for AC-5. The erroneous name-limited path-owner example from the first pass was withdrawn. The unchanged Status now matches the two repaired controls reviewed in this round. |

## Disposition verdict

Round 1 at e66c9d3: the previously reported Status overclaim is resolved for the two reviewed control failures. No new spec finding in this fix diff.
