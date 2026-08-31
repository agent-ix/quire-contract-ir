---
id: PLAN-002
title: "Build quire-contract-ir v0.1 semantic substrate"
type: Plan
status: active
relationships:
  - target: ix://agent-ix/quire-contract-ir/TASK-005
    type: contains
  - target: ix://agent-ix/quire-contract-ir/TASK-006
    type: contains
  - target: ix://agent-ix/quire-contract-ir/TASK-007
    type: contains
  - target: ix://agent-ix/quire-contract-ir/TASK-008
    type: contains
  - target: ix://agent-ix/quire-contract-ir/TASK-009
    type: contains
---
# PLAN-002: Build quire-contract-ir v0.1 semantic substrate

## Dependency DAG

```text
TASK-005 issue #5 specification + assurance foundation
  -> TASK-006 issue #6 identities, anchors, clauses, dependencies
    -> TASK-007 issue #8 types, expressions, definedness
      -> TASK-008 issue #9 canonicalization, digests, versions, orphans
        -> TASK-009 issue #10 schema, corpus, reusable runner
          -> epic #11 code review + gap analysis + human source decision
```

Each task advances through Specify, Spec Review, Implement, In review, and Done.
Only the dependency-ready task moves out of Backlog.

## Plan Delta

- Replace the placeholder crate with a closed v0.1 semantic model in dependency order.
- Keep public semantics language-neutral and downstream-engine independent.
- Separate wire parsing from validated semantic values and typed diagnostics.
- Freeze canonical JSON and digest behavior before publishing the corpus.
- Add exact requirement tags, tests, review, gap analysis, and immutable evidence per child.
- Keep Cargo registry publication disabled and Actions manual-dispatch only.

## Exit Criteria

All five tasks and native issues are Done; 100% of implemented criteria are
backed; the complete corpus, golden bytes, diagnostics, dependencies, and
digests pass; no blocking review or reverse gap remains; and `@kreneskyp`
records the v0.1 source decision for an exact candidate.
