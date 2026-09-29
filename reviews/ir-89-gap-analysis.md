---
id: SR-592
title: "PR #205 FR-040 and FR-038-AC-40 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@97bcea493dbc71b1785ba67d25ff632a2f8fd141; FR-040-AC-1..13, FR-038-AC-22, FR-038-AC-40; TC-056, TC-057; tests/it/checked_package_v2_frame_entries.rs, tests/it/checked_package_v2_node_identity_vectors.rs, crates/quire-contract-model/src/checked_package/v2/operations/model_member_vectors.rs, crates/quire-contract-model/src/checked_package/v2/frame.rs (tests)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-040
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-056
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-057
    type: reviews
---
# SR-592: PR #205 FR-040 and FR-038-AC-40 gap analysis

## Summary

Ticket: IR-89.

Each acceptance criterion was traced to the tests that back it. The oracle
strength of each test was checked against QSpec origin/main `e56756f`.

`quire coverage --scope . --strict` at `97bcea4` reports 20 unbacked rows
(origin/main reports 37). None of the 20 is an FR-040, TC-056, TC-057 or
FR-038-AC-40 row, and FR-040 is 13/13. The exit code is 1 because of the 20
pre-existing unbacked rows, which belong to FR-036, FR-037, FR-039 and FR-344.
`scripts/validate_matrix_status.py` exits 0.

`quire validate` exits 1 on MP-001 and MP-002. The PR does not touch either
file. They fail because the installed quire 0.33.0 schema has drifted from
those files.

Expected values come from QSpec's published data, not from the
implementation: TC-280 `frame_field_cases` (6), `anchor_cases` (7),
`clause_signature_cases` (9), `frame_mutations` (30) and the node-identity
arrays (17 `vectors`, 21 `operation_vectors`, 13 `invalid_mutations`, 24
`operation_mutations`). The `frame_mutations` replay asserts the code, the
cause and the locus digest. The authored TC-056 cases assert the code, the
cause, the RFC 6901 path and, where given, the locus. They would fail on a
wrong reader.

## Verdict

**CHANGES REQUESTED** (one medium gap).

AC-1, AC-2 and AC-4 through AC-13 are backed. The operation-resolution and
signature parts of AC-7 and AC-9 run only under `make qspec-vectors`.

The `record_value_type` clause of AC-3 has no test. That test would have
caught SR-591 FND-001.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-040-AC-3 says "one on a `record_value_type` node admit[s] with [its name] unresolved". No test covers it, either in-repo or through a TC-280 replay. The only "admits unresolved" case uses a source-declared object type. An undeclared `record_value_type` declaring node actually refuses `missing-selection` (SR-591 FND-001). | tests/it/checked_package_v2_frame_entries.rs:355-361 |
| FND-002 | low | The "replayed count equals published count" check cannot fail. TC-056 compares a loop counter with the length of the same array. TC-057 has no per-array count assertion; it only prints the counts. An empty or truncated published array passes. | tests/it/checked_package_v2_frame_entries.rs:1244-1292; tests/it/checked_package_v2_node_identity_vectors.rs:473-479 |
| FND-003 | low | For a non-stale-key operation mutation, the retained-key refusal is accepted when it is `stale-node-key` or any `invalid_semantic_graph`. A setup error that produces an unrelated graph-shape refusal passes this assertion. | tests/it/checked_package_v2_node_identity_vectors.rs:388-396 |
| FND-004 | low | The new `locate_in_preimage` remap (closed-schema `malformed_wire` inside a nominal preimage becomes `invalid_semantic_graph`) has no in-repo test. Only the QSPEC_DIR-gated TC-057 `invalid_mutations` exercise it, so a plain `make test` cannot detect a regression. | crates/quire-contract-model/src/checked_package/v2/mod.rs:445-455 |

## Dispositions

Round 1, reviewed at `1d89455fc04ddfb60cd2ac932886f1b223cd3688` (rebased on origin/main `a38f3db`).

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 1d89455 |
| FND-002 | fixed | 1d89455 |
| FND-003 | fixed | 1d89455 |
| FND-004 | fixed | 1d89455 |
| FND-005 | fixed | 24f0eae (round 3, reviewed at c877c64) |

## New findings (disposition pass 2)

Round 2, reviewed at `995bd4bec7fe526c7891f728044fec4c2e7d469e`. FND-001 and FND-004 stay fixed: the tests are kept and still pass. The test code behind FND-002 and FND-003 was deleted by owner order, so those two need no further action.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | medium | FR-040's Behavior section still specifies model-owned resolution in the frame and state steps: field names on a model declaration node, and anchor operation resolution with the own-versus-inherited `malformed-declaration` rule. It also specifies the clause signature's `result` and operation-parameter checks. Removing the TC-280 replays left no test that reaches `resolve_field`'s owner branch, `resolve_operation`, or the result and parameter comparison in `check_clause`, and no postcondition is tested at all. Because no AC names this behaviour, `quire coverage` cannot report it as unbacked. A regression in these reader paths would pass `make test`. | spec/contract/FR-040-admit-frame-entries-and-state-clauses.md:97-112; spec/contract/FR-040-admit-frame-entries-and-state-clauses.md:141-152; spec/contract/FR-040-admit-frame-entries-and-state-clauses.md:180-191; crates/quire-contract-model/src/checked_package/v2/state.rs |
