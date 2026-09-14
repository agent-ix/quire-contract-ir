---
id: TASK-023
title: "FR-027 deterministic model export and strict reading"
type: Task
status: done
track: G
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/TASK-022
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-027
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-040
    type: verifies
---
# TASK-023: FR-027 deterministic model export and strict reading

## Scope

Export one canonical identity-bound model from a checked manifest and strict-read it only by independent re-export and byte equality.

## Subtasks

- [x] Sort nodes/edges, retain exact selections/gaps/limits and add only derived adjacency/topological projections.
- [x] Compute the domain-separated model identity with the identity field omitted and publish immutable model schema bytes/digest.
- [x] Re-export during reading and reject every field, identity, digest, count, graph, campaign or selection mismatch.
- [x] Represent improvement proposals as source-model-bound descriptive values with no acceptance state or authority input conversion.

## Deliverables

- `src/ecosystem_model/document.rs`
- `src/ecosystem_model/reader.rs`
- `schemas/temporal-ecosystem-model-v1.schema.json`
- TC-040 model identity, strict-reader and non-authority cases

## Notes

- No model output is accepted by predicate, temporal, evaluator, protocol or assurance decision APIs.
- Unblocks TASK-024.
