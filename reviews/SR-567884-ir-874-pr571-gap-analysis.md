---
id: SR-567884
title: "IR-874 PR 571 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; protocol_body.rs and checked_package_v2_protocol_body.rs; PR #571 "
review_set: subset
---

## Summary

Compared the new related-record order and endpoint checks with FR-038-AC-282/288 and QSpec FR-466-AC-9. The tagged tests exercise duplicates and one bad `from` end, but leave two newly implemented branches without direct public-reader evidence.

## Verdict

**CONDITIONAL** — Two medium test evidence gaps in the related-record checks need focused tests on this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The canonical related-record test repeats an identical tuple; it never checks that two distinct tuples in reverse relationship/JCS order refuse or that their ascending order admits. | tests/it/checked_package_v2_protocol_body.rs:1864 |
| FND-002 | medium | Related-end tests make `from` ill-typed and assert only that detail; an ill-typed `to` end and the exact `expected` endpoint type in the typed refusal remain unverified. | tests/it/checked_package_v2_protocol_body.rs:1943 |

## Scope

- **FR-038-AC-282 (examined)**: Each of the seven protocol member forms admits only the FR-464, FR-465 or FR-466 fixed argument names, order and term shapes for its selected operator: thirteen clause bindings, the role, channel, queue, finish and terminal layouts, thirteen control layouts and thirteen compensation bindings. A missing, extra, reordered, wrongly shaped or misspelled binding refuses `malformed_wire`; an optional Group admits zero or one member only, an interval has non-negative integer bounds with lower no greater than upper, and a set or sorted-record list repeated or out of its specified ascending order refus
- **FR-038-AC-288 (examined)**: Each of the thirteen control operators admits its FR-466 ordered arguments and local reference, type, bounds, label and ordering checks: `sequence`, `choice`, `parallel`, `repeat`, `await`, `check`, `send`, `receive`, `attempt`, `effect`, `event`, `commit` and `fence`. A choice or parallel branch list with fewer than two entries, repeated labels, a receive naming a non-send, an effect naming a non-attempt, an await naming an ineligible event, a non-Boolean guard or constraint, an invalid join count or an ineligible memory or ordering spelling refuses with the FR-466 code, cause and locus, with
- **FR-466-AC-7 (examined)**: A `send` and a `receive` are admitted over one channel with payload binders typed as the channel's `carries`; a binder of another type, a `receive` of a `send` on another channel and a `receive` whose `of` is not a `send` refuse as the reader order states; the pair in the wrong causal order refuses `invalid_package`/`definition-cycle`.
- **FR-466-AC-9 (examined)**: An `event` is admitted with and without a compensation; a `commit` and a `fence` are admitted; `related` entries name a relationship the clause lists with end terms of the endpoint types, and an unlisted relationship refuses `missing_declaration`/`missing-name` and an ill-typed end `ill_typed`/`type-mismatch`.
- **FR-038-AC-284 (context_only)**: The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending node-id digest. A package with both a refinement-step and protocol defect reports the refinement defect; one with both a

## Coverage

Plan completion: not assessed. The computed Quoin matrix has tagged FR-038-AC-282/284/288 criteria but no run evidence in its store. Reverse-gap and stub inspection found no new ownerless code or stub. Optional repository-wide semantic review was not requested; changed behavior was compared manually with FR-464/466 and IR FR-038. The complete protocol acceptance criteria remain Planned, and this artifact does not claim to discharge them.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | In the reviewed PR, the public-reader test admits two distinct related tuples in JCS order and refuses the reverse order as `invalid_semantic_graph`. |
| FND-002 | fixed | In the reviewed PR, tests assert the exact expected and actual source type, then refuse a malformed `to` end at its pointer with exact target type detail. |

## Disposition verdict

**PASS** — Both test evidence gaps were closed in the same PR. Remaining protocol stages and the full FR-038 criteria remain Planned.
