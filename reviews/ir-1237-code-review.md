---
id: SR-202611010120
title: "IR-1237 PR 803 code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 803; crates/quire-contract-model/src/checked_package/v2/ and tests/conformance_qspec/probabilistic_claims.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Ticket: IR-1237. Reviewed frozen head 53670148b9953fc398561c8e103f44fdf1548da0 against merged FR-038-AC-390 through AC-395, FR-019-AC-18, QSpec FR-322-AC-288/297, writer and Rust conventions. No applicable AssuranceProfile was found. No vendored content or CI workflow change was introduced.

## Verdict

**FAIL** — the required string-edge lint fails on the new unknown-member path helper. The typed row, graph join, ordering, duplicate refusal and canonical writer logic otherwise match the inspected contract. Aggregate gates belong to the code lead.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The new `probabilistic_row_unknown_member_path` branches on a string path without the required `string_edge` marker, so `make string-edge` fails with one unmarked branch-gating occurrence. Mark the helper and rerun the gate. | crates/quire-contract-model/src/checked_package/v2/intake.rs:273 |

## Coverage

- FR-038-AC-390, role: examined. Excerpt: A present `CheckedTermDataV2` has exactly four required arrays, `equality_decisions`, `protocol_locations`, `dispatch_tables` and `probabilistic_claims`, including when all are empty. `None` omits the sidecar, while `Some(empty)` retains it through typed write and strict read. Removing any required array or adding an unknown member refuses `malformed_wire` at the missing object or unknown member; the public and private wire paths agree.
- FR-038-AC-391, role: examined. Excerpt: The strict reader admits an IR-constructed complete V2 package containing QSpec `checked-term-data-vectors.json` positive `PC-01` and `PC-02` identities as two nonempty probabilistic-claim rows. It retains each exact local `claim`/`probabilistic_claim` declaration `NodeRef` and its own declared `claim` occurrence. Their equal display names and source spans do not merge them or determine identity; a probabilistic-claim declaration without its one required row is refused under QSpec FR-322-AC-297.
- FR-038-AC-392, role: examined. Excerpt: The reader checks QSpec `PC-S1` missing occurrence and `PC-S2` extra member as closed-schema `malformed_wire` refusals. For `PC-R1` foreign, `PC-R2` wrong-kind, `PC-R3` absent-local and a non-probabilistic claim declaration, it returns `invalid_package`/`invalid-value` at `/checked_term_data/probabilistic_claims/{i}/declaration`; for `PC-R6` mis-mapped occurrence or wrong occurrence role, it returns that code/cause at `/checked_term_data/probabilistic_claims/{i}/occurrence`. Each refusal has the exact published primary RFC 6901 locus and no admitted package.
- FR-038-AC-393, role: examined. Excerpt: QSpec `PC-R4`'s second row for one declaration with a different valid occurrence and `PC-R5`'s repeated declaration-occurrence key both refuse `invalid_package`/`conflicting-definition` at the later row. `PC-R7`'s reversal of two valid rows refuses `invalid_package`/`invalid-semantic-graph` at the second row. Structural declaration identity and occurrence determine order; display name, source span and graph-array position do not.
- FR-038-AC-394, role: examined. Excerpt: IR tests read QSpec's published `checked-term-data-vectors.json` by reference and construct a complete valid V2 package in the test from its `PC-01` and `PC-02` row identities and resolved-node context. They run the positive and probabilistic-claim adverse mutations against the public strict reader and compare each published typed code, cause and exact RFC 6901 locus. No upstream artifact or vector content is copied into this repository, and the vector file is not treated as a complete checked-package fixture. An exact measured sidecar-work limit admits the IR-constructed positive; one below its measured charge returns `resource_exhausted` at its final probabilistic-claim row without a partial typed sidecar view.
- FR-038-AC-395, role: examined. Excerpt: A package with a `claim`/`probabilistic_claim` graph node passes the closed `ClaimForm::ProbabilisticClaim` vocabulary and ordinary typed term-body checks, then graph identity, exact dependencies, declaration, owner, occurrence and source-map joins before its probabilistic sidecar row is checked. The node and complete body survive admitted read/write without a fabricated formula or identity. An unknown claim form, malformed term body or broken graph join retains its existing earlier-stage typed refusal rather than being accepted by a matching sidecar row.
- FR-019-AC-18, role: examined. Excerpt: The default-feature model crate exports `ClaimForm::ProbabilisticClaim` and the public `CheckedProbabilisticClaim` row with exact typed `declaration: CheckedNodeId` and `occurrence: CheckedOccurrence` members, usable in the fourth required `CheckedTermDataV2::probabilistic_claims` array and borrowed `encode_checked_package_v2` input. A public consumer constructs nonempty rows without importing a private wire type or passing display name or source span as identity, and reads the same typed values from a strictly admitted package. The existing public-item inventory includes the row type when implementation lands.
- QSpec FR-322-AC-288 and FR-322-AC-297, role: examined against the owning checkout.
- `make string-edge`: failed; unmarked branch-gating occurrence at `intake.rs:273`.
