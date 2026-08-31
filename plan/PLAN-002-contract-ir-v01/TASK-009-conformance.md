---
id: TASK-009
title: "Publish schema, corpus, and conformance runner"
type: Task
status: in_progress
phase: implement
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/10
    type: references
  - target: ix://agent-ix/quire-contract-ir/TASK-008
    type: depends_on
---
# TASK-009: Publish schema, corpus, and conformance runner

Implement FR-018 through FR-020 after TASK-008 is Done. Complete corpus
coverage, downstream pin documentation, retained measurements, and epic review.

## Plan Delta

- Publish one Draft 7 package schema and one Draft 7 conformance schema whose
  named subschemas validate the manifest and every operation input and
  expectation, all with immutable profile identities and checksums.
- Define declarative package, expression-check, migration, and coverage fixtures;
  the runner must not dispatch hard-coded behavior by fixture ID.
- Maintain a closed public-construct and diagnostic inventory, and require the
  manifest's coverage tags to cover it exactly without unknown tags.
- Emit one deterministic JSON Lines result per fixture in manifest order, with
  stable mismatch kinds and exit 0/1/2 separation.
- Reject unsafe paths, duplicate fixture IDs, unsupported profiles, oversized
  inputs, schema drift, and malformed expectations before semantic execution.
- Preflight the fixed semantic node, recursion-depth, and collection limits and
  retain exact-at-limit and one-past-limit fixtures without public recursion.
- Add TC-018 end-to-end positive, negative, mutation, stdout/stderr, and
  process-exit fixtures; retain issue-scoped review and evidence before epic
  closure.
