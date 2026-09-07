---
id: PLAN-005
title: "Bind conformance results to declared verification targets"
type: Plan
status: active
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/44
    type: references
  - target: ix://agent-ix/quire-contract-ir/TASK-016
    type: contains
---

# PLAN-005: Bind conformance results to declared verification targets

## Dependency DAG

```text
TASK-016 declare suite and producer trace ids
  -> released Quoin adapter preserves trace_ids
    -> exact-head record demonstrates nonzero bindings
```

## Plan Delta

- Declare the native corpus and shared static/intake suites in `spec/evidence/suites.md`.
- Add a non-empty, sorted, schema-validated `trace_ids` array to every manifest fixture and copy it
  unchanged into the corresponding structured result.
- Name TC-015 through TC-018, allowing Quire's existing obligation graph to resolve acceptance
  criteria without a second repository-local mapping.
- Require the Quoin `contract-conformance` adapter to preserve those ids. Do not wrap it, scrape
  output, or create a local evidence binder while that released adapter lacks the field.
- Dispatch no hosted workflow.

## Exit Criteria

The corpus regenerates byte-for-byte, 99/99 fixtures match, every row carries its manifest-declared
trace ids, Quire validates the suite declaration, a released Quoin adapter preserves those ids, and
`quoin evidence record` reports nonzero exact obligation bindings at one candidate revision.
