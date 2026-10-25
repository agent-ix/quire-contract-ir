---
id: SR-17190
title: "IR-1014 PR 559 spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir#559; FR-059, FR-060, TC-060, TC-061"
review_set: base
---

# SR-17190: IR-1014 PR 559 spec review

## Summary

Reviewed the four added artifacts in PR #559 against QSpec FR-322,
FR-475, FR-476, TC-960 and TC-961 and the IR V2 reader's closed form gate.
The requirements preserve the published bodies, refusal precedence, codes,
causes and pointers. One identity case stated in FR-060's behavior is missing
from its acceptance criteria. Refinement is absent from this PR pending IR-1016;
the closed reader continues to refuse unknown forms. No applicable
AssuranceProfile was found in this repository's spec catalog.

`quire validate --scope /home/peter/dev/worktrees/ir1014-spec 'spec/**/*.md'`
and `git diff --check origin/main...HEAD` passed. This is a spec-only review;
no Cargo or runtime gates were run. Planned acceptance criteria do not claim
implemented evidence yet.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-060 says changing the selected `model` changes the terminal node id and package identity, but AC-4 and TC-061 cover only mode, condition, written-name rename and terminal presence. Add a second selected-model vector that changes only the `model` reference and asserts both identities. | FR-060-AC-4; TC-061; QSpec FR-476 Identity |

## Verdict

CHANGES REQUESTED for FND-001. Each new FR otherwise has five verifiable
acceptance criteria and a corresponding IR-owned test case. The procedures
cover positive read/write/read, individual refusals and two-defect order
probes. QSpec's request-level obligation identities remain outside the IR
reader.

## Disposition

FND-001 **fixed** in the same PR. FR-060-AC-4 now requires two packages with
the same selected `M` and `N` nodes, changing only an `any` terminal's model
reference. TC-061 constructs both admitted packages, rebuilds dependencies and
identities, checks distinct terminal node ids and package ids, and preserves
each through read/write/read. Focused Quire validation of the two edited
documents passed. Final verdict: **PASS** for this spec review.
