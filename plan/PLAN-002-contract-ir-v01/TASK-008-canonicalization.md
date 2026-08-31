---
id: TASK-008
title: "Implement canonicalization, versions, and orphan coverage"
type: Task
status: done
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/9
    type: references
  - target: ix://agent-ix/quire-contract-ir/TASK-007
    type: depends_on
---
# TASK-008: Implement canonicalization, versions, and orphan coverage

Implement FR-016 and FR-017 after TASK-007 is Done. Freeze cross-platform
golden bytes and digests before schema/corpus publication.

## Plan Delta

- Define one source-free canonical JSON projection for packages, requirements,
  clauses, declarations, value types, and typed expressions, with exact object,
  string, number, sequence, and semantic-set ordering rules.
- Domain-separate SHA-256 identities by profile and semantic-object kind so
  equal payload bytes from different kinds cannot collide by construction.
- Accept schema 1.0 and current 1.1 only, expose the single registered
  1.0-to-1.1 reference-body migration, and retain a source digest in its
  migration receipt.
- Reject unknown majors and unregistered minor paths before decoding semantic
  package content.
- Provide a deterministic reservation-failure harness that proves canonical
  allocation failure returns `canonicalization_resource_exhausted` without
  partial public bytes or a digest.
- Classify deterministic artifact traces as shallow, deep, uncovered, or
  orphaned; require a matching current requirement digest for deep coverage,
  and keep missing, stale, cross-package, duplicate, and digest-mismatched
  artifacts from contributing coverage.
- Add TC-017 golden bytes/digests, permutation properties, digest-propagation
  checks, migration negatives, and orphan-coverage fixtures.
