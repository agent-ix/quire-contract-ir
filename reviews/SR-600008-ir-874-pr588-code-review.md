---
id: SR-600008
title: IR-874 PR 588 code and Rust review
type: SpecReview
analysis: code-review
scope: 'agent-ix/quire-contract-ir; PR #588;
  crates/quire-contract-model/src/checked_package/shared.rs, crates/quire-contract-model/src/checked_package/v2/protocol_body.rs,
  crates/quire-contract-model/src/lib.rs, tests/it/checked_package_v2_protocol_body.rs,
  spec/model/functional/FR-019-rust-library-interface.md'
review_set: subset
---

## Summary

Reviewed the frozen decision-fact source and public-reader tests against QSpec FR-466/464, IR FR-038-AC-292 and FR-019 API inventory. Hosted Rust Checks fail to compile, and a nested record projection can bypass the decision-fact walk.

## Verdict

**FAIL** — The frozen head fails compilation and has a decision-fact bypass.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | New WrongObservation cause is absent from cause_admitted exhaustive match; hosted make lint fails E0004 and tests do not run. | crates/quire-contract-model/src/checked_package/v2/contract_package.rs:1502 |
| FND-002 | high | A record.project whose receiver is another expression terminates the walk before its nested binder-field read; an otherwise readable but unlisted or ineligible inner fact can be admitted. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:3742 |

## Scope

- **FR-038-AC-292 (examined):** At a `choice` or `repeat` decision the reader admits only QSpec FR-466 visible facts owned by the deciding role and listed individually in `visible`; a fact from another role, an ineligible send/effect/commit/finish or protocol-input binder, a fact reachable only inside a composite entry, or a readable fact omitted from `visible` refuses `wrong_snapshot`/`wrong-observation` with the specified reason. A binder unavailable at that decision refuses `missing_declaration`/`missing-name` first.
- **FR-464-AC-12 (examined):** The protocol step runs after the state, temporal, abstraction and refinement steps and before the operation step and reports, in order, placement, operator, declaration, member, membership, clause and refinement class-catalog graph-join defects; within a clause the cycle and progress defects precede the slot checks, and within a slot the `over`/model-state refusal precedes the binder-scope refusal, which precedes the decision-fact refusal; a state-step defect and a protocol defect in one package report the state-step defect.
- **FR-466-AC-14 (examined):** Decision facts: a `choice` owned by the `to` role of a channel decides on its `receive` record's Boolean field listed individually in `visible`; the other role's receive record, a `send`, `effect`, `commit` or finish binder, the protocol input binder, an unlisted fact and a fact reachable only inside a composite entry each refuse `wrong_snapshot`/`wrong-observation` with `reason` `other-role`, `ineligible-kind` or `unlisted`; an attempt or event binder is eligible for the role it is `by`; a repeat's `visible` and guard follow the same rule; a binder not readable at the node refuses as an out-o
- **FR-019-AC-5 (examined):** The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists as implemented at the tested revision. Entries explicitly marked planned are excluded until their owning code lands. The public-item inventory fails on an added or missing implemented item; no implemented item in the table is reachable through a `quire_contract_ir` path.

## Coverage

Hosted Rust Checks failed make lint with E0004 in contract_package.rs:1502; Supply-chain, guard and CLA passed. No local Cargo or Make run under the shared build lock. The nested projection path follows the unconditional continue after the project identity branch; the operation catalog permits a record operand and record-valued fields.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed |, cause_admitted pairs WrongObservation only with WrongSnapshot; the exhaustive temporal test helpers spell both new enum values explicitly. The coder reports focused public-reader tests passing under the shared build lock; hosted full Rust Checks at this head remain in progress. |
| FND-002 | fixed |, a record.project only stops traversal when its receiver is a direct parameter binder. Otherwise the walk follows its dependencies; nested_projection_exposes_its_unlisted_inner_fact asserts wrong_snapshot with the inner field in typed detail. Focused public-reader tests passed; hosted full Rust Checks remain in progress. |

## Disposition verdict

**PASS for the two code findings ** — both source defects are fixed in the changed code and backed by focused local tests. Final hosted gate evidence remains pending. FR-038-AC-292 remains Planned, including the STD-496 owner decision.

## Final exact-head integration review

Hosted Rust Checks at exact code passed Format, Lint and Test; Supply-chain Checks, language-boundary guard and CLA passed at the same head. The final diff changes no CI workflow or safety lint, introduces no unsafe code, and keeps the typed WrongObservation/WrongSnapshot pair exhaustive in model and test code. The nested record projection walk reaches the inner field through validated dependencies and is metered; the public-reader adverse case passed in the hosted Test step. Formal Rust review finds no new defect after the round-1 fixes. **Final code-review verdict: PASS** for this PR's independent slice. FND-001 and FND-002 remain fixed. FR-038-AC-292 remains Planned.
