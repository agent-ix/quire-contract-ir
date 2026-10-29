---
id: SR-2026102733
title: IR-1104 integrity review of unsupported requested-root kind
type: SpecReview
analysis: integrity
scope: 'agent-ix/quire-contract-ir; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md;
  spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md; PR #674
  @ '
review_set: subset
---

## Summary

Ticket: IR-1104. Frozen PR. The new requested-root kind stays outside FR-330 wire and distinguishes the requested root from the first unsupported descendant. The required fairness-to-clause fixture conflicts with QSpec FR-370.

## Verdict

**FAIL.** AC-27 and TC-047 require an unreachable admitted source shape.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-035-AC-27 requires an admitted fairness root to reach a temporal-clause node, but QSpec FR-370 gives fairness a zero-operand body whose only dependency is its object-type declaration. The required producer-admitted fixture in AC-27 and TC-047 is unreachable. Use a clause root reaching fairness as the distinct unsupported descendant, while retaining direct fairness, formula and clause root controls. | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:292; spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md:128; QSpec FR-370:74-78,100 |

## Coverage

- FR-035-AC-27 (examined): PLANNED (IR-1104). For producer-admitted mixed accounting, the requested-root kind at each `unsupported` accounting index equals the exact `CheckedNodeKind` of that record's `node_id` in the admitted V2 source, including an unrepresented `Temporal(Fairness)` root whose first unsupported reachable node is `Temporal(TemporalClause)`, and separately `Temporal(Formula)` and `Temporal(TemporalClause)` roots. Interposing a healthy sibling or another request does not shift the index, change the sibling disposition, or substitute the first unsupported node's kind.
- FR-035-AC-28 (examined): PLANNED (IR-1104). `read_with_source` of the producer's canonical V1 bytes and exact reader-admitted V2 source returns the same requested-root kind for every `unsupported` accounting index as the producer. A mismatched source identity or lock, or a source missing an `unsupported` requested root, returns a typed handoff error and no authenticated package; ordinary V1 reader refusal or limit outcomes retain their existing precedence. No requested-root kind is inferred from `node_tag`, `unsupported_node_id`, or a same-tag represented node.
- FR-035-AC-29 (examined): PLANNED (IR-1104). For a canonical V1 package admitted by plain `read`, querying an `unsupported` accounting index returns typed `NotAdmitted` for the requested-root kind, including when the requested key is represented through another root. A non-`unsupported` or out-of-range index returns typed absence, distinct from `NotAdmitted`. Neither query changes the existing `unsupported` record's first-reachable `unsupported_node_id`/`node_tag` or an independent sibling's result.
- FR-035-AC-30 (examined): PLANNED (IR-1104). Producer and authenticated reader requested-root-kind queries change none of the six FR-330 wire members, the closed `unsupported` payload, canonical bytes or identity. Fairness, formula and temporal-clause requests with the same `temporal` tag remain distinguishable through the typed view, while the V1 wire's first-reachable-node tag remains unchanged.
- TC-047-IR-1104 (examined): For FR-035-AC-27 through FR-035-AC-30, lower one admitted V2 source with separate requested `temporal/fairness`, `temporal/formula` and `temporal/temporal_clause` roots, a healthy sibling, and a fairness root whose first unsupported reachable node is a distinct temporal-clause node with a lower typed key. Include a refused root absent from the represented V1 graph. At each `feature_accounting` index, compare the producer's requested-root `CheckedNodeKind` with the independently located V2 source root and compare the unchanged `unsupported_node_id` and `node_tag` with the first unsupported reac

## Context

QSpec FR-330 keeps the unsupported wire payload closed. QSpec FR-370 fixes the temporal body shapes and dependencies. IR `read_with_source` currently authenticates source identity/lock and represented nodes only. No applicable AssuranceProfile was found under `spec/`; no Rust/build gate was run for this spec-only review.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At PR, AC-27 and TC-047 use a temporal-clause root whose fairness argument references a distinct fairness node. QSpec FR-370 explicitly permits that reference, and the fairness node remains a zero-operand application that does not reference the clause. Direct fairness and formula requests remain covered. |
