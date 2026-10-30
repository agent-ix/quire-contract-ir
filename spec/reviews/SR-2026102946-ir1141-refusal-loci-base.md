---
id: SR-2026102946
title: "Base spec review of IR-1141 PR #711 refusal loci"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #711 FR-038, FR-035 and FR-062 diff"
review_set: subset
---

## Summary

Reviewed exact head cb149b15 against QSpec FR-322-AC-77 and the IR public refusal type. QSpec requires two locations for repeated refinement entries. The new rule is testable and keeps graph-node locus separate, but it conflicts with the existing repeated-dependency requirement and other IR acceptance criteria that name the public fields being removed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | New Behavior and AC-369 say every other refusal has one locus, but existing FR-038 Behavior says a repeated dependency identity refuses with both entries as loci. Include this existing two-location case and its test; preserve code and precedence. | FR-038-AC-33; FR-038-AC-369 |
| FND-002 | medium | FR-038-AC-193 still specifies that public model-intake projection carries code/path/cause. Update its public-interface wording to primary locus while preserving the required code, cause, exact pointer and boundary inspection. | FR-038-AC-193; FR-038-AC-368 |
| FND-003 | high | CheckedPackageRefusal is shared. FR-035-AC-18 requires public path and related_path, and FR-062-AC-4 requires public path and absence of related_path. Reconcile these criteria with the replacement list without narrowing their exact pointer and terminal-duplicate controls. | FR-035-AC-18; FR-062-AC-4; FR-038-AC-368 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | At d7076090, Behavior and AC-369 include repeated dependency identity among two-location cases, preserving the existing both-entry rule. |
| FND-002 | fixed | At d7076090, AC-193 and Behavior describe projection of the primary location path to ValidationFailure code/path/cause, keeping the IR-663 boundary inspection. |
| FND-003 | fixed | At d7076090, FR-035-AC-18 requires two ordered exact importer paths; FR-062-AC-4 and its Behavior/Outputs require one primary pointer and retain typed terminal payload, code, cause and graph-node locus. |
