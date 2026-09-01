---
id: PLAN-003
title: "Reconcile shared assurance governance"
type: Plan
status: active
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/38
    type: references
  - target: ix://agent-ix/quire-contract-ir/TASK-010
    type: contains
  - target: ix://agent-ix/quire-contract-ir/TASK-011
    type: contains
  - target: ix://agent-ix/quire-contract-ir/TASK-012
    type: contains
---

# PLAN-003: Reconcile shared assurance governance

## Dependency DAG

```text
TASK-010 specification and all-dimension review
  -> TASK-011 campaign documents, dispositions, and TC-023..TC-028
    -> TASK-012 code review, gap analysis, issue feedback, and merge
      -> Quire CLI #74 and Quoin CLI #322
```

## Plan Delta

- Replace the common-envelope/executor/retention architecture with the exact
  PGM-01 shared responsibility assignment.
- Preserve historical PGM-01 records, checksums, and corrections byte-for-byte
  through the accepted read-only compatibility mapping.
- Publish a repository-owned reconciliation/disposition document and update
  campaign issue #1, re-scope issue #7, and close issue #20 as superseded.
- Preserve prototype adversarial/domain cases as an inventory only; adopt none
  of its executor, profile, verdict, authority-index, adoption, or retention
  behavior.
- Add TC-023..TC-028 and keep hosted CI manual-dispatch only.
- Modify none of the eight migration repositories.

## Identifier Disposition

An unpushed, superseded `issue-20-shared-evidence` worktree also used
`PLAN-003`. Issue #20 is now closed and that branch is not a merge source. This
reviewed plan is the sole `PLAN-003` entering `main`; the separate user worktree
is preserved read-only rather than deleted or rewritten.

## Exit Criteria

All three tasks are done; every FR-008/FR-009/FR-021 criterion resolves to a
passing requirement-tagged test; Quire validation, traceability, Rust tests,
local CI, code review, and gap analysis pass; all issue and PR feedback is read;
and issues #1/#7/#20 carry the reviewed linked dispositions.
