---
id: SR-600009
title: IR-874 PR 588 gap analysis
type: SpecReview
analysis: gap-analysis
scope: 'agent-ix/quire-contract-ir; PR #588 cf151429fe0965b4a9e93326983cdca9aee003c2;
  crates/quire-contract-model/src/checked_package/shared.rs, crates/quire-contract-model/src/checked_package/v2/protocol_body.rs,
  crates/quire-contract-model/src/lib.rs, tests/it/checked_package_v2_protocol_body.rs,
  spec/model/functional/FR-019-rust-library-interface.md'
review_set: subset
---

## Summary

The changed implementation is traced to FR-038-AC-292 by five public-reader tests, but the test set omits several named negative cases and a nested-projection bypass. Plan completion: not assessed.

## Verdict

**CONDITIONAL** — Named acceptance paths remain untested.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The five new AC-292 tests do not exercise another-role receive, commit and finish binders, protocol-input binder, or nested record projection; the named adverse paths lack direct public-reader evidence. | tests/it/checked_package_v2_protocol_body.rs:318 |

## Scope

- **FR-038-AC-292 (examined):** At a `choice` or `repeat` decision the reader admits only QSpec FR-466 visible facts owned by the deciding role and listed individually in `visible`; a fact from another role, an ineligible send/effect/commit/finish or protocol-input binder, a fact reachable only inside a composite entry, or a readable fact omitted from `visible` refuses `wrong_snapshot`/`wrong-observation` with the specified reason. A binder unavailable at that decision refuses `missing_declaration`/`missing-name` first.
- **FR-466-AC-14 (examined):** Decision facts: a `choice` owned by the `to` role of a channel decides on its `receive` record's Boolean field listed individually in `visible`; the other role's receive record, a `send`, `effect`, `commit` or finish binder, the protocol input binder, an unlisted fact and a fact reachable only inside a composite entry each refuse `wrong_snapshot`/`wrong-observation` with `reason` `other-role`, `ineligible-kind` or `unlisted`; an attempt or event binder is eligible for the role it is `by`; a repeat's `visible` and guard follow the same rule; a binder not readable at the node refuses as an out-o
- **FR-019-AC-5 (examined):** The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists as implemented at the tested revision. Entries explicitly marked planned are excluded until their owning code lands. The public-item inventory fails on an added or missing implemented item; no implemented item in the table is reachable through a `quire_contract_ir` path.

## Coverage

Plan completion: not assessed. Trace tags name FR-038-AC-292, and the new tests assert real public-reader outcomes for attempt, event, receive, unlisted, composite, repeat and scope cases. The absent cases are explicit in FR-038-AC-292 and QSpec FR-466-AC-14. Quoin matrix at the reviewed head reported FR-038-AC-292 tagged by five new public-reader tests and FR-019-AC-5 tagged by three existing public-interface tests; its run-evidence status is no run evidence because hosted Rust Checks stopped at lint. No source stub or unowned new API was found.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | At da9239cb, new traced public-reader cases cover nested projection, another-role receive and ineligible commit; the earlier effect/send and finish-scope cases remain, and the finish case now asserts BinderScope detail. The protocol-input over path still has no resolved expected verdict: QSpec FR-466 prose and FR-464-AC-12 require ill_typed/operator-ineligible before decision facts, while QSpec FR-466-AC-14 and IR FR-038-AC-292 list it with wrong_snapshot cases. STD-496 is the owner decision; the criterion remains Planned with no narrowed claim. |
| FND-001 | deferred | STD-496 is filed as the owner decision for conflicting protocol-input over verdicts and blocks IR-874. The independent decision-fact paths are covered by public-reader tests. No new PR #588 test asserts the disputed over verdict; FR-038-AC-292 remains Planned and the deferred case is unclaimed in PR and matrix. Finish remains scope-first with BinderScope detail. |

## Disposition verdict

**CONDITIONAL at da9239cb** — the independent adverse test gaps are addressed, but FND-001 stays open on the protocol-input over conflict pending STD-496. Hosted full Rust Checks at this head remain in progress.

## Disposition verdict after owner routing

**PASS for the independent safe slice at da9239cb, subject to hosted Rust Checks** — all actionable code and public-reader gaps are fixed; the contradictory protocol-input over verdict is deferred to STD-496 with FR-038-AC-292 still Planned. This disposition does not declare the whole criterion satisfied.

## Final exact-head integration review

Hosted Rust Checks at exact code head da9239cbf5ab663037ad02b56e9764e2cf8af8a8 passed Format, Lint and Test; Supply-chain Checks, language-boundary guard and CLA passed at the same head. The final public-reader tests exercise admitted attempt, event and receive facts, owner refusal for attempt and receive, ineligible send/effect/commit, unlisted and composite facts, nested projection, repeat, and binder-scope precedence including finish. The latest disposition of FND-001 is deferred to STD-496 solely for the disputed protocol-input `over` verdict. PR #588 does not assert that verdict and does not claim full FR-038-AC-292 coverage; the criterion remains Planned. **Final gap-analysis verdict: PASS for the independent slice with the STD-496 deferral recorded.** Plan completion: not assessed.
