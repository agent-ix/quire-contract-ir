---
id: PLAN-001
title: "Implement PGM-01 governance gate"
type: Plan
relationships:
  - target: ix://agent-ix/quire-contract-ir/TASK-001
    type: contains
  - target: ix://agent-ix/quire-contract-ir/TASK-002
    type: contains
  - target: ix://agent-ix/quire-contract-ir/TASK-003
    type: contains
  - target: ix://agent-ix/quire-contract-ir/TASK-004
    type: contains
---
# PGM-01 implementation plan and delta

## Dependency DAG

```text
TASK-001 requirements, matrix, and composite review
  -> TASK-002 policy and Draft 7 schema/corpus
  -> TASK-003 requirement-tagged tests and local gates
  -> TASK-004 immutable evidence and gap analysis
  -> CODEOWNER review
  -> project Done + issue #3 closure
```

## Plan Delta

- Replace issue #3's undefined `PGM-E` dependency with umbrella issue #1.
- Retain neutral contribution policy while naming `@kreneskyp` as the human
  authority enforced by CODEOWNERS and protected `main`.
- Express the envelope as one authoritative Draft 7 schema, positive and
  negative fixtures, deterministic error taxonomy, and mutation probes.
- Declare Python locally and in the disabled workflow without dispatching it.
- Keep evidence verification outside ordinary development tests and mint one
  immutable record per exact subject revision.
- Do not implement semantic IR from #5/#6/#8/#9/#10.

## Exit Criteria

Every task is represented by merged source on the branch, all Quire artifacts
validate, matrix coverage is fully backed, local gates pass, evidence is
revision-scoped, and the PR is ready for human CODEOWNER re-review.
