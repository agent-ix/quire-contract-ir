---
id: TASK-006
title: "Implement identities, anchors, clauses, and dependencies"
type: Task
status: done
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/6
    type: references
  - target: ix://agent-ix/quire-contract-ir/TASK-005
    type: depends_on
---
# TASK-006: Implement identities, anchors, clauses, and dependencies

Implement FR-011 and FR-012 after TASK-005 is Done. Retain round-trip,
revision-invalidation, dependency, and orphan-reference evidence.

## Plan Delta

- Add validated package, requirement-revision, clause, source, and anchor identities.
- Add source spans and closed executable/informational clause kinds.
- Derive ordered dependency sets by walking clause bodies rather than accepting caller-supplied sets.
- Resolve requirement and clause references against exact revisions, distinguishing malformed, stale, and orphaned references.
- Preserve an implementation-language-independent serialized representation and stable diagnostic codes.
