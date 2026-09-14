---
id: TASK-021
title: "FR-027 manifest contract and admission"
type: Task
status: done
track: G
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-027
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-040
    type: verifies
---
# TASK-021: FR-027 manifest contract and admission

## Scope

Publish immutable manifest schema bytes/digest and admit the complete exact nine-repository campaign selection through one bounded strict reader.

## Subtasks

- [x] Define closed typed repository, component, object, interface, contract and evidence nodes plus typed edges.
- [x] Strict-read canonical bytes and require the exact campaign/version, nine repository identities and immutable lowercase merged revisions.
- [x] Enforce every independent byte/depth/string/population/work/allocation ceiling without partial checked state.

## Deliverables

- `src/ecosystem_model/manifest.rs`
- `schemas/temporal-ecosystem-manifest-v1.schema.json`
- TC-040 manifest fixtures and boundary cases

## Notes

- Manifest values are caller-supplied exact selections, never ambient Git/GitHub discovery.
- Unblocks TASK-022.
