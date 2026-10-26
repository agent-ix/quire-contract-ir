---
id: SR-600004
title: "IR-874 PR 586 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #586 at 4a322bb13c354c04ef2c79b35c5067a693a0b310; crates/quire-contract-model/src/checked_package/v2/refinement.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: references }
  - { target: "ix://agent-ix/quire-contract-ir/FR-056", type: references }
---

## Summary

Ticket: IR-874. Reviewed the frozen PR #586 diff, its caller and direct test, QSpec FR-375, IR FR-038 and FR-056, and hosted gate output. This is an early catalog-presence slice; complete class reconstruction and memory-class qualification remain Planned under FR-038-AC-325.

## Verdict

**FAIL** — the exact PR head fails the required string-edge lint, and the added scan has unmetered quadratic work.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Post-intake `"over"` string branch fails the authoritative string-edge lint; classify through a reviewed typed intake or postpone this decision until admitted protocol layout exists, without a broad marker or allow-list exception. | crates/quire-contract-model/src/checked_package/v2/refinement.rs:1327 |
| FND-002 | medium | Each refinement scans every graph node while charging only protocol-clause visits, so many model-only refinements can force quadratic unmetered work under the finite read limit. | crates/quire-contract-model/src/checked_package/v2/refinement.rs:1312 |

## Scope

- **FR-038-AC-325 (examined)**: The protocol step compares every present refinement `class_catalog` against complete concrete and abstract classes, with protocol defects before the graph join and row/catalog defects before either. This PR implements only the early concrete-protocol presence case; no full AC qualification is claimed.
- **FR-038-AC-9 (examined)**: Every read meter is enforced at its true measured boundary; one below exact work returns `incomplete` for `work`.
- **FR-056-AC-7 (examined)**: A marked function must be an actual intake conversion; no broad marker or branch-dispatch suppression may conceal a post-intake string decision.
- **QSpec FR-375 checked class catalog (context_only)**: Present catalog on a model-only refinement and absent catalog when a protocol or memory class participates refuse at `/body/class_catalog` after the dedicated key and before row joins.

## Evidence

- Hosted Rust Checks at `4a322bb13c354c04ef2c79b35c5067a693a0b310` failed `make lint`: `qsl-string-edge` reported one unmarked, unlisted branch-gating occurrence at `refinement.rs:1327` in `concrete_protocol_presence`. The allow-list is empty; adding this post-intake function to it or marking the broad validator would violate FR-056.
- `concrete_protocol_presence` loops over `graph.index.values()` for each refinement. It charges only after finding `ProtocolClause`; nonprotocol entries incur no per-entry charge. The default allows 10,000 nodes and 1,000,000 work units.
- No CI workflow, rules file, dependency, or gate configuration changed in the PR diff. The hosted supply-chain and language-boundary jobs passed. Local Cargo and Make gates were not run because the shared build lock was occupied.
