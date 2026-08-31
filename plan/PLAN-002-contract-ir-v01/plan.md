---
id: PLAN-002
title: "Build quire-contract-ir v0.1 semantic substrate"
type: Plan
status: done
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/11
    type: references
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
          -> epic #11 implementation review + gap analysis
            -> PGM-02 Wave 4 human source decision and tag
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

Wave 1 Agent A is complete when all five tasks and native issues are Done; all
implemented criteria resolve to executable completed tests; the complete
corpus, golden bytes, diagnostics, dependencies, and digests pass; retained
review and reverse-gap findings have dispositions; and the source-release claim
remains honestly bounded. The program handoff assigns the human source
decision, source tag, and checksums to PGM-02 in Wave 4 after all eight
repository epics complete. They are therefore not asserted by this plan.

## Completion

Issues #5, #6, #8, #9, and #10 are closed and their project items are Done.
Post-merge `main` at `5c49ebfd1c87415f74420ad047392bd03b1bd202`
passes the complete isolated local CI lane, Rust 1.75 check, specification and
matrix-status gates, and retained-evidence verification. REV-007 records the
epic gap analysis and the limitations carried into Wave 4. No hosted workflow,
source tag, registry publication, qualification, or accreditation was
performed.
