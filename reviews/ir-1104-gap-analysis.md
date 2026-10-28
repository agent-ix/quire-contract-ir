---
id: SR-2026102737
title: "IR-1104 PR 676 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #676 head 31bdc5099b27d87592b3f86af3905f1788d82de0; crates/quire-contract-model/src/checked_package/v2/contract_package.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs, crates/quire-contract-model/src/lib.rs, tests/it/checked_package_v2_temporal.rs, tests/it/complete_v1_contract_package.rs"
review_set: subset
---

## Summary

Ticket: IR-1104. Plan completion: not assessed. The new test is tagged to all four criteria and exercises the main path; two mandated edge cases lack assertions in this PR. Existing AC-26 handoff tests cover related checks on represented nodes, but do not exercise the new unsupported-root path.

## Verdict

**CONDITIONAL** pending acceptance evidence for FR-035-AC-28 and FR-035-AC-29.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-29's represented-through-another-root case has no test. The new test uses a refused clause root and directly requested fairness/formula roots; none is represented in V1 through an independent lowered request, so a plain reader that guessed a represented root kind would pass. | tests/it/complete_v1_contract_package.rs:1227 |
| FND-002 | medium | AC-28's unsupported-root handoff does not test a mismatched lock or V1 limit outcome. The new test checks source identity and a missing requested root, while earlier AC-26 tests use a package without an unsupported record; the new path's lock and limit precedence remain unverified. | tests/it/complete_v1_contract_package.rs:1352 |

## Coverage

The computed trace binding is present for FR-035-AC-27 through AC-30 in the new test, which contains real behavioral assertions. The producer's clause-versus-fairness distinction, index alignment, plain NotAdmitted, source identity, missing root, and wire/identity equality are exercised. Reverse ownership is FR-035; no source stub or copied upstream artifact was found in the diff. Plan completion: not assessed. Optional repository-wide semantic review was not requested.

## Scope

- **FR-035-AC-27 (examined):** PLANNED (IR-1104). For producer-admitted mixed accounting, the requested-root kind at each `unsupported` accounting index equals the exact `CheckedNodeKind` of that record's `node_id` in the admitted V2 source, including an unrepresented `Temporal(TemporalClause)` root whose first unsupported reachable node is a distinct `Temporal(Fairness)` node referenced in its fairness argument, and separately direct `Temporal(Fairness)` and `Temporal(Formula)` requests. Interposing a healthy sibling or another request does not shift the index, change the sibling disposition, or substitute the first unsupported node's kind.
- **FR-035-AC-28 (examined):** PLANNED (IR-1104). `read_with_source` of the producer's canonical V1 bytes and exact reader-admitted V2 source returns the same requested-root kind for every `unsupported` accounting index as the producer. A mismatched source identity or lock, or a source missing an `unsupported` requested root, returns a typed handoff error and no authenticated package; ordinary V1 reader refusal or limit outcomes retain their existing precedence. No requested-root kind is inferred from `node_tag`, `unsupported_node_id`, or a same-tag represented node.
- **FR-035-AC-29 (examined):** PLANNED (IR-1104). For a canonical V1 package admitted by plain `read`, querying an `unsupported` accounting index returns typed `NotAdmitted` for the requested-root kind, including when the requested key is represented through another root. A non-`unsupported` or out-of-range index returns typed absence, distinct from `NotAdmitted`. Neither query changes the existing `unsupported` record's first-reachable `unsupported_node_id`/`node_tag` or an independent sibling's result.
- **FR-035-AC-30 (examined):** PLANNED (IR-1104). Producer and authenticated reader requested-root-kind queries change none of the six FR-330 wire members, the closed `unsupported` payload, canonical bytes or identity. Fairness, formula and temporal-clause requests with the same `temporal` tag remain distinguishable through the typed view, while the V1 wire's first-reachable-node tag remains unchanged.
- **TC-047 IR-1104 paragraph (examined):** lower one admitted V2 source with separate requested `temporal/fairness`, `temporal/formula` and `temporal/temporal_clause` roots, a healthy sibling, and a temporal-clause root whose admitted fairness argument references a distinct fairness node with the least typed key among temporal nodes in the clause's reached closure, making that node the first unsupported reachable node under a profile that does not support the temporal family. Include a refused root absent from the represented V1 graph. The fairness node's application has zero operands and only its admitted declaration dependency; it does not reference the temporal clause. At each `feature_accounting` index, compare the producer's requested-root `CheckedNodeKind` with the independently located V2 source root and compare the unchanged `unsupported_node_id` and `node_tag` with the first unsupported reachable node. Interpose a healthy request and verify the accounting indices and sibling results. Strict-read the producer's canonical V1 bytes both with the exact reader-admitted V2 source through `read_with_source` and without a source through plain `read`. The authenticated result must match the producer's requested-root kinds; the plain result must report typed `NotAdmitted` for each `unsupported` index, including a requested root also represented through a different lowered root. Query a healthy record and an out-of-range index and require typed absence, distinct from `NotAdmitted`. Supply a mismatched source identity and lock separately, and a source missing the requested root through a controlled test-only mutation; require a typed handoff error with no partially authenticated package. Also assert existing V1 reader refusal and limit precedence over source handoff. Compare all six wire members, canonical bytes and identity before and after the typed queries.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At ebd596ad3c2cb8c1d39bb546434b77696f3091f7, the test constructs an admitted canonical V1 package whose unsupported request key is the separately lowered, represented healthy node; it verifies node presence and plain-read typed NotAdmitted. |
| FND-002 | fixed | The same unsupported-record test now verifies a rehashed wrong source lock returns SourceLock, a one-below byte limit returns Incomplete before handoff, and a stale V1 identity returns Refused before source identity matching. |

## Disposition verdict

**PASS at ebd596ad3c2cb8c1d39bb546434b77696f3091f7.** Both review findings are fixed in the original PR.
