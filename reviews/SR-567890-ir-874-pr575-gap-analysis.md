---
id: SR-567890
title: "IR-874 PR 575 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs, tests/it/checked_package_v2_protocol_reference.rs; PR #575 frozen head decf37259513a31a003831c2ce04ec026932f1c9 against 5e0ea8d3"
review_set: subset
---

## Summary

Reviewed the frozen stage-5 membership diff and hosted Rust Checks at decf37259513a31a003831c2ce04ec026932f1c9.

## Verdict

**CONDITIONAL** — Protocol-reference tests append an unattached role, so two intended admissions now refuse under the new membership rule.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Protocol-reference tests append an unattached role, so two intended admissions now refuse under the new membership rule. | tests/it/checked_package_v2_protocol_reference.rs:64 |

## Scope

- **FR-038-AC-26 (examined)**: Each one-over limit other than the byte limit returns `incomplete` carrying the RFC 6901 pointer of the value whose charge failed, which resolves in the document read: nodes at the first node past it, edges at the dependency and occurrences at the source-map entry or region that took the count past it, diagnostics at the first entry past it, and work at the value whose validation took the meter past it; the byte limit carries none.
- **FR-038-AC-284 (examined)**: The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending node-id digest. A package with both a refinement-step and protocol defect reports the refinement defect; one with both a
- **FR-038-AC-285 (examined)**: Every role, channel, queue, obligation and compensation is listed by exactly one owning protocol clause, and every control is held once in its root control tree with the declaration path of its holder followed by its own name. A member listed twice or unattached, a control held twice or by nothing, a channel without exactly one queue, or a queue attached to another clause refuses `invalid_semantic_graph` at that member. A target not listed in its referencing node's `dependencies` refuses `missing_declaration`/`missing-name`; a target of the wrong tag or form refuses `invalid_model_binding`/`ma

## Coverage

Plan completion: not assessed. Quire matrix marks FR-038-AC-26/284/285 tagged; the tagged membership tests pass, but two existing protocol-reference admission fixtures fail. Optional repository-wide semantic review was not requested.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | role_target now lists the appended role in the protocol clause and calls settle, so the two protocol-reference admission tests exercise a valid owned role. |

## Disposition verdict

**PASS at a72241c2ab8b6c19ca11d8df9cda0becae0c45fd** — FND-001 fixed; full FR-038-AC-282/284/285 remain Planned.
