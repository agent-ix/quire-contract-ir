---
id: SR-567889
title: "IR-874 PR 575 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs, tests/it/checked_package_v2_protocol_reference.rs; PR #575 frozen head decf37259513a31a003831c2ce04ec026932f1c9 against 5e0ea8d3"
review_set: subset
---

## Summary

Reviewed the frozen stage-5 membership diff and hosted Rust Checks at decf37259513a31a003831c2ce04ec026932f1c9.

## Verdict

**FAIL** — Membership scans charge every semantic node, including nonprotocol packages, changing exact work limits and refusal paths.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Membership scans charge every semantic node, including nonprotocol packages, changing exact work limits and refusal paths. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:2756 |

## Scope

- **FR-038-AC-26 (examined)**: Each one-over limit other than the byte limit returns `incomplete` carrying the RFC 6901 pointer of the value whose charge failed, which resolves in the document read: nodes at the first node past it, edges at the dependency and occurrences at the source-map entry or region that took the count past it, diagnostics at the first entry past it, and work at the value whose validation took the meter past it; the byte limit carries none.
- **FR-038-AC-284 (examined)**: The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending node-id digest. A package with both a refinement-step and protocol defect reports the refinement defect; one with both a
- **FR-038-AC-285 (examined)**: Every role, channel, queue, obligation and compensation is listed by exactly one owning protocol clause, and every control is held once in its root control tree with the declaration path of its holder followed by its own name. A member listed twice or unattached, a control held twice or by nothing, a channel without exactly one queue, or a queue attached to another clause refuses `invalid_semantic_graph` at that member. A target not listed in its referencing node's `dependencies` refuses `missing_declaration`/`missing-name`; a target of the wrong tag or form refuses `invalid_model_binding`/`ma

## Coverage

Rust style, bounded traversal, reader order, refusal types, and scoped public-reader membership tests were checked. Hosted Rust Checks failed 10 of 875 integration tests; seven failures expose incorrect global work charging, two expose invalid protocol-reference fixtures, and one structural-key work boundary also fails. No workflow or dependency file changed.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The clause scan now charges only after recognizing a protocol clause. The final member scan likewise skips nonprotocol nodes before charging. Protocol-only exact work remains unchanged, while protocol fixtures gain five ledgered units. |

## Disposition verdict

**PASS at a72241c2ab8b6c19ca11d8df9cda0becae0c45fd** — FND-001 fixed; full FR-038-AC-282/284/285 remain Planned.
