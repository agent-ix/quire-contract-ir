---
id: SR-567894
title: "IR-874 PR 578 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs; PR #578 frozen "
review_set: subset
---

## Summary

The computed Quire matrix tags FR-038-AC-282 and AC-286, and the new tests exercise one whole-model role at a time. QSpec FR-465-AC-9 additionally requires simultaneous roles from two selections and admission across clause `over` types.

## Verdict

**CONDITIONAL** — the changed feature lacks public-reader evidence for the cross-selection and clause-over cases.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Whole-model admission tests never put two selected-text roles in one clause or vary the clause `over` type, so the distinctive FR-465-AC-9 cases can regress unnoticed. | tests/it/checked_package_v2_protocol_body.rs:3785 |

## Scope

- **FR-038-AC-26 (examined)**: Each one-over limit other than the byte limit returns `incomplete` carrying the RFC 6901 pointer of the value whose charge failed, which resolves in the document read: nodes at the first node past it, edges at the dependency and occurrences at the source-map entry or region that took the count past it, diagnostics at the first entry past it, and work at the value whose validation took the meter past it; the byte limit carries none.
- **FR-038-AC-282 (examined)**: Each of the seven protocol member forms admits only the FR-464, FR-465 or FR-466 fixed argument names, order and term shapes for its selected operator: thirteen clause bindings, the role, channel, queue, finish and terminal layouts, thirteen control layouts and thirteen compensation bindings. A missing, extra, reordered, wrongly shaped or misspelled binding refuses `malformed_wire`; an optional Group admits zero or one member only, an interval has non-negative integer bounds with lower no greater than upper, and a set or sorted-record list repeated or out of its specified ascending order refus
- **FR-038-AC-286 (examined)**: A role admits the FR-465 static and replicated shapes only, including a model of an eligible form, replication maximum at least one, a reference-typed binder and an `until` term exactly for an `until` lifetime; a channel admits only same-clause role endpoints, an eligible message type, explicit delivery and ordering, and a FIFO key with equality-stable type; a queue admits one channel, literal or symbolic capacity, explicit overflow and a loss selector exactly for `loss`. The reader admits the positive variants and refuses the wrong form, missing premise, wrong binder type, non-Boolean lifetim
- **FR-465-AC-9 (examined, QSpec)**: A static role whose `model` is the text `identity` of a `model_selections` row of the lock is admitted as a role on a whole model, with no `model` dependency, whatever the clause's `over` type (a model type, a reference, a scalar, a record or a `Set<M::Cell>`); one clause holding `role w on M` and `role u on N` for two selections is admitted, and the two roles, and a role on a whole model and a role on a model type of the same selection, have different node keys. A text naming no `model_selections` row refuses `missing_declaration`/`missing-selection` at the role node, and so does such a text

## Coverage

Plan completion: not assessed. `quire matrix --scope. --format json` completed: FR-038-AC-26 is tagged by 2 tests; AC-282 and AC-286 are each tagged by 12 tests. Those tags do not establish the missing whole-model combinations. The changed source has an owning criterion and no stub. Optional repository-wide semantic review was not requested; this scoped review compared new test behavior against the QSpec criterion. Full FR-038-AC-282/286 remain Planned.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed: the new public-reader test places two static text-model roles from separate lock selections in one clause, checks distinct keys and empty model dependencies, admits both a model-type and scalar over type, then removes one selection and checks MissingDeclaration/MissingSelection at the second role. |

## Disposition verdict

**PASS ** — the focused finding is fixed on the original PR. Full FR-038-AC-282/286 remain Planned.
