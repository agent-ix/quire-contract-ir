---
id: SR-202610310113
title: "IR-1005 PR 749 dependency analysis"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-contract-ir; PR #749; FR-350, TC-461, TM-010 placement correction"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-350
    type: references
---

## Summary

Ticket IR-1005. Checked ownership: `quire-integration` consumes QSL classification and TL owner readers; Contract IR model/root remain free of QSL dependencies under FR-028-AC-3/TC-041. IR-358 is the downstream implementation/test owner, not a new Contract IR Cargo dependency. No cycle or copied owner surface is introduced.

## Verdict

PASS for dependency placement; base review FND-001 remains open.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - | correct-requirement-no-evidence |

## Coverage

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-350 | examined | `agent-ix/quire-integration` (IR-358) owns the QSL-typed `project_tl` bridge, |
| FR-028-AC-3 | context_only | The root package's own manifest names no dependency in any kind on a crate from the agent-ix/quire-spec-language repository, by name or by git source. |
| TC-461 | examined | Verify the `agent-ix/quire-integration` temporal bridge (IR-358) against QSL |
