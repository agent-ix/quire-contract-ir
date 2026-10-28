---
id: SR-2026102736
title: "IR-1104 PR 676 Rust code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #676 head 31bdc5099b27d87592b3f86af3905f1788d82de0; crates/quire-contract-model/src/checked_package/v2/contract_package.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs, crates/quire-contract-model/src/lib.rs, tests/it/checked_package_v2_temporal.rs, tests/it/complete_v1_contract_package.rs"
review_set: subset
---

## Summary

Ticket: IR-1104. Reviewed the frozen PR head and the requested-root kind path in producer, plain reader, and authenticated handoff. The new state is in the admitted value only. No applicable AssuranceProfile, copied schema, compatibility layer, unsafe operation, or CI workflow change was found.

## Verdict

**PASS** for the Rust implementation. The acceptance evidence gaps are recorded separately in SR-2026102737.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

The producer uses the admitted V2 source root key, and the authenticated read applies existing source identity, lock, and represented-node checks before resolving unsupported roots. A plain read returns typed NotAdmitted for unsupported indices. Non-unsupported and out-of-range indices return absence. The new data is not serialized; existing accounting cause, six wire fields, and identity remain unchanged. Existing focused fmt, 34 tests, and Clippy are reported on the PR; this reviewer did not rerun an aggregate gate. Formal final gate belongs to the lead after review fixes.

## Scope

- **FR-035-AC-27 (examined):** PLANNED (IR-1104). For producer-admitted mixed accounting, the requested-root kind at each `unsupported` accounting index equals the exact `CheckedNodeKind` of that record's `node_id` in the admitted V2 source, including an unrepresented `Temporal(TemporalClause)` root whose first unsupported reachable node is a distinct `Temporal(Fairness)` node referenced in its fairness argument, and separately direct `Temporal(Fairness)` and `Temporal(Formula)` requests. Interposing a healthy sibling or another request does not shift the index, change the sibling disposition, or substitute the first unsupported node's kind.
- **FR-035-AC-28 (examined):** PLANNED (IR-1104). `read_with_source` of the producer's canonical V1 bytes and exact reader-admitted V2 source returns the same requested-root kind for every `unsupported` accounting index as the producer. A mismatched source identity or lock, or a source missing an `unsupported` requested root, returns a typed handoff error and no authenticated package; ordinary V1 reader refusal or limit outcomes retain their existing precedence. No requested-root kind is inferred from `node_tag`, `unsupported_node_id`, or a same-tag represented node.
- **FR-035-AC-29 (examined):** PLANNED (IR-1104). For a canonical V1 package admitted by plain `read`, querying an `unsupported` accounting index returns typed `NotAdmitted` for the requested-root kind, including when the requested key is represented through another root. A non-`unsupported` or out-of-range index returns typed absence, distinct from `NotAdmitted`. Neither query changes the existing `unsupported` record's first-reachable `unsupported_node_id`/`node_tag` or an independent sibling's result.
- **FR-035-AC-30 (examined):** PLANNED (IR-1104). Producer and authenticated reader requested-root-kind queries change none of the six FR-330 wire members, the closed `unsupported` payload, canonical bytes or identity. Fairness, formula and temporal-clause requests with the same `temporal` tag remain distinguishable through the typed view, while the V1 wire's first-reachable-node tag remains unchanged.
- **TC-047 IR-1104 paragraph (examined):** lower one admitted V2 source with separate requested `temporal/fairness`, `temporal/formula` and `temporal/temporal_clause` roots, a healthy sibling, and a temporal-clause root whose admitted fairness argument references a distinct fairness node with the least typed key among temporal nodes in the clause's reached closure, making that node the first unsupported reachable node under a profile that does not support the temporal family. Include a refused root absent from the represented V1 graph. The fairness node's application has zero operands and only its admitted declaration dependency; it does not reference the temporal clause. At each `feature_accounting` index, compare the producer's requested-root `CheckedNodeKind` with the independently located V2 source root and compare the unchanged `unsupported_node_id` and `node_tag` with the first unsupported reachable node. Interpose a healthy request and verify the accounting indices and sibling results. Strict-read the producer's canonical V1 bytes both with the exact reader-admitted V2 source through `read_with_source` and without a source through plain `read`. The authenticated result must match the producer's requested-root kinds; the plain result must report typed `NotAdmitted` for each `unsupported` index, including a requested root also represented through a different lowered root. Query a healthy record and an out-of-range index and require typed absence, distinct from `NotAdmitted`. Supply a mismatched source identity and lock separately, and a source missing the requested root through a controlled test-only mutation; require a typed handoff error with no partially authenticated package. Also assert existing V1 reader refusal and limit precedence over source handoff. Compare all six wire members, canonical bytes and identity before and after the typed queries.

## Final public-inventory correction verification

**PASS at 382c163c71fb049fb6ed45b3c14710474f154170.** The sole delta after the final gate failure adds `ContractUnsupportedSourceKindError` to FR-019's `checked_package` ContractPackage v1 public-items row. The crate-root named re-export at `crates/quire-contract-model/src/lib.rs:199` already exposes that type; FR-019-AC-5 and TC-058 require exact inventory agreement. `ContractScalarOperandViewV1` was already present in both the inventory row and the crate-root export before this delta. No source or test file changed. The coder reports the focused TC-058 tests passing 6/6 and Quire validation passing; this reviewer checked the inventory delta and did not rerun the full gate.
