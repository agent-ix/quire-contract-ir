---
id: SR-590008
title: "IR-874 PR 582 code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/shared.rs, crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs; PR #582 frozen head 2b1dbad0d248ee68efc77562da22ffbc88906f4f against 4b407abc"
review_set: subset
---

## Summary

Reviewed the derived causal graph, cycle reporting, repeat progress, public reader assertions, refusal order and work charging against IR FR-038 and QSpec FR-464. No CI, dependency, spec or plan file changed.

## Verdict

**CONDITIONAL** — the newly inserted causal check runs after clause binder and guard checks that FR-464 orders after cycle and progress.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The causal check runs after clause guard and capture slot checks. A package with both a causal cycle and an invalid activation guard or capture initializer reports the slot defect first, contrary to the required cycle-before-slots order. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:3697 |

## Scope

- **FR-038-AC-26 (examined)**: Each one-over limit other than the byte limit returns `incomplete` carrying the RFC 6901 pointer of the value whose charge failed.
- **FR-038-AC-284 (examined)**: The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order.
- **FR-038-AC-290 (examined)**: For each clause the reader constructs the QSpec FR-464 derived causal relation from control containment, sequencing, receive/send, effect/attempt and await edges, not source order or timestamps.
- **FR-038-AC-291 (context_only)**: Binder levels start at the introducing construct's entry count; protocol `over` is level 0, an `each` trigger follows, captures advance in their authored order.
- **FR-464-AC-9 (examined, QSpec)**: The derived causal relation is acyclic: a `receive` sequenced before the `send` it names, a send/receive pair in one sequence in the wrong order and an `await` anchored at a later control each refuse `invalid_package`/`definition-cycle` naming the controls on the cycle.
- **FR-464-AC-12 (examined, QSpec)**: The protocol step runs after the state, temporal, abstraction and refinement steps and before the operation step and reports, in order, placement, operator, declaration, member, membership, clause and refinement class-catalog graph-join defects; within a clause the cycle and progress defects precede the slot checks.

## Coverage

The graph has a port per control entry and exit; sequencing, choice, parallel join, repeat, await, send/receive and attempt/effect edges follow FR-464. Iterative traversal and edge/DFS charges bound work. The new cycle details retain controls in tree order. Public reader tests assert cycle code, cause and control identities for receive-before-send and self-anchored await, and admit independent parallel branches. Host backoff barred Cargo, Rustc, Make and test jobs during this review; no gates were started. No applicable AssuranceProfile was found under `spec/`.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At 8fabf73f, `check_clause` retains clause references, activation form and capture binder declarations, then `check_clause_causality` checks cycle and progress, and `check_clause_slots` checks the activation guard and capture initializers. The new public test combines a later-sibling await cycle with a conflicting guard and asserts the cycle refusal. |

## Disposition verdict

**PASS at 8fabf73f** for the recorded code-review finding. Host backoff left runtime gates to the final exact-head CI run.
