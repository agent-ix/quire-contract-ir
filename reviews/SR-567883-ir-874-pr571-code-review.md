---
id: SR-567883
title: "IR-874 PR 571 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; shared.rs, contract_package.rs, mod.rs, protocol_body.rs, checked_package_v2_protocol_body.rs, checked_package_v2_temporal.rs; PR #571 "
review_set: subset
---

## Summary

Reviewed the check, fence, send and receive local checks, channel payload type, related relationship selection and endpoint conformance, refusal details, and changed public-reader tests. The full FR-038-AC-282/284/288 criteria remain Planned while the remaining controls and later protocol stages are implemented.

## Verdict

**PASS** — No code or Rust idiom defect found in this scoped change. Test evidence gaps are recorded separately in SR-567884.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

- **FR-038-AC-282 (examined)**: Each of the seven protocol member forms admits only the FR-464, FR-465 or FR-466 fixed argument names, order and term shapes for its selected operator: thirteen clause bindings, the role, channel, queue, finish and terminal layouts, thirteen control layouts and thirteen compensation bindings. A missing, extra, reordered, wrongly shaped or misspelled binding refuses `malformed_wire`; an optional Group admits zero or one member only, an interval has non-negative integer bounds with lower no greater than upper, and a set or sorted-record list repeated or out of its specified ascending order refus
- **FR-038-AC-284 (examined)**: The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending node-id digest. A package with both a refinement-step and protocol defect reports the refinement defect; one with both a
- **FR-038-AC-288 (examined)**: Each of the thirteen control operators admits its FR-466 ordered arguments and local reference, type, bounds, label and ordering checks: `sequence`, `choice`, `parallel`, `repeat`, `await`, `check`, `send`, `receive`, `attempt`, `effect`, `event`, `commit` and `fence`. A choice or parallel branch list with fewer than two entries, repeated labels, a receive naming a non-send, an effect naming a non-attempt, an await naming an ineligible event, a non-Boolean guard or constraint, an invalid join count or an ineligible memory or ordering spelling refuses with the FR-466 code, cause and locus, with
- **FR-466-AC-1 (examined)**: Each of the thirteen control operators is admitted with exactly its layout; the arguments reordered, a missing or surplus binding, an unknown operator spelling and an enumerated value outside its set (`memory`, `outstanding`, `policy`, `ordering`, `kind`) refuse `malformed_wire` or refuse `operation-member-mismatch`.
- **FR-466-AC-7 (examined)**: A `send` and a `receive` are admitted over one channel with payload binders typed as the channel's `carries`; a binder of another type, a `receive` of a `send` on another channel and a `receive` whose `of` is not a `send` refuse as the reader order states; the pair in the wrong causal order refuses `invalid_package`/`definition-cycle`.
- **FR-466-AC-9 (examined)**: An `event` is admitted with and without a compensation; a `commit` and a `fence` are admitted; `related` entries name a relationship the clause lists with end terms of the endpoint types, and an unlisted relationship refuses `missing_declaration`/`missing-name` and an ill-typed end `ill_typed`/`type-mismatch`.
- **FR-464-AC-12 (context_only)**: The protocol step runs after the state, temporal and abstraction steps and before the operation step and reports, in order, placement, operator, declaration, member, membership and clause defects; within a clause the cycle and progress defects precede the slot checks, and within a slot the `over`/model-state refusal precedes the binder-scope refusal, which precedes the decision-fact refusal; a state-step defect and a protocol defect in one package report the state-step defect.

- **FR-038-AC-90 (examined)**: `NominalIdentityPreimage::digest` takes a byte limit: called with the limit equal to the canonical byte length of a preimage of each of its four versions it returns the digest the node key holds, and one byte lower it returns the encoder's byte-limit refusal and no digest; the reader passes its configured `limits.bytes` to it, and no call in the crate source passes `u64::MAX` or a literal cap, each checked by a test that reads the crate source.

## Coverage

The diff changes no CI workflow or dependency declarations. Rust style and repository rules were checked. The coder's final-head focused evidence was model tests 10/10, public-reader tests 48/48, formatting and string-edge checks passing. `git diff --check` passed in review. Aggregate gates belong to the code lead after the review. No spec or plan file changed, so spec-review methods do not apply. No separate Rust-review artifact is emitted because Rust review is the code-review lane.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | Hosted Rust Checks at failed two FR-038-AC-90 identity tests: the new related-record sort encoded through `serde_json::to_value` and passed `u64::MAX` to `Limits::new`, violating the repository's checked-package encoding boundary. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:435 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed |, related sorting calls `quire_canonical::to_vec` directly on the typed `CheckedTerm` with the reader's `limits.bytes`; neither forbidden encoder path remains. |

## Disposition verdict

**PASS ** — The newly discovered hosted gate failure was fixed in the same PR. Its final full hosted run remains the code lead's gate.
