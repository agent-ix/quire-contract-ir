---
id: FR-026
title: "Bind native temporal projections to exact TL correspondence"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-012
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-023
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/issues/63
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/issues/52
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/57
    type: references
  - target: ix://agent-ix/quire-contract-ir/ADR-0053
    type: references
---
# FR-026: Bind native temporal projections to exact TL correspondence

## Description

The IR shall define a versioned, fail-closed projection from a validated native
Quire temporal clause to an exact TL correspondence. Native Quire remains the
sole authored formal-clause language; a TL formula is derived output and never
an alternative authoring surface.

The projection shall preserve every semantic and provenance dimension needed to
decide whether the native temporal clause and the selected TL evaluator profile
denote the same obligation. Unsupported dimensions shall produce an explicit
refusal instead of approximation, omission, or an unqualified formula.

The selected bridge profile is
`quire.contract.native-temporal-tl-correspondence/v1`. It replaces
ADR-0053's unaccepted FRETish-source candidate with a native-source
correspondence; FRETish remains a downstream output mapping under issue #57.

## Inputs

The projection consumes:

- the native source-profile identifier, revision, feature set, and digest;
- the source-document identity, revision, content digest, clause span, and
  temporal-expression span;
- the FR-012 ClauseRef, anchor, and source span carried by the FR-023 BoundClause;
- the model-closure identity and the issue #63 total-Boolean predicate/signal
  projection;
- the native temporal profile, operator, interval, clock, observation model,
  capture bindings, history policy, progress policy, and closure policy; and
- the bridge-profile identifier and revision, TL formula, TL semantic-profile
  identifier, proposition map, and evaluator identity and revision.

All inputs are validated, immutable values. FR-023 supplies the aggregate
semantic node, depth, collection, and byte limits. Issue #63 owns validation of
its predicate projection and proposition-map identities. The bridge introduces
no ambient registry lookup, state read, callback, plugin, network operation, or
evaluator invocation.

## Behavior

For valid inputs, the bridge shall return exactly one of two tagged decisions:

- a supported projection containing a TL formula and the complete source,
  clause, model, bridge, TL-profile, proposition-map, and evaluator identities;
  or
- an unsupported projection containing no TL formula and an ordered, nonempty
  set of unmatched semantic dimensions.

The decision record has `format` equal to
`quire.contract.native-temporal-tl-correspondence/v1`, `kind` equal to
`supported` or `unsupported`, and a `correspondence_ref`. The reference is a
structural tuple of the source-profile, source-document, ClauseRef,
BoundClause, model-closure, predicate-projection, native-temporal-profile,
bridge-profile, clock, observation, capture, history, progress, closure,
TL-profile, evaluator, and proposition-map identities and revisions.

A supported decision additionally contains the TL formula and its formula
identity, and the correspondence reference includes that formula identity. An
unsupported decision contains no formula or formula identity and
contains an ordered, duplicate-free `unmatched_dimensions` list. That list uses
this closed order: `source_profile`, `source_document`, `clause`,
`bound_clause`, `model_closure`, `predicate_projection`, `temporal_profile`,
`operator`, `interval`, `clock`, `anchor`, `capture`, `history`, `progress`,
`closure`, `tl_profile`, `evaluator`, `proposition_map`, and `formula`.
Repeating a decision over structurally equal validated inputs shall return a
structurally equal record. An evaluation result is attributable to the
correspondence only when it cites the complete `correspondence_ref`, its
evaluated trace identity, and its own result-profile identity.

Invalid structural inputs return existing FR-011 through FR-023 or issue #63
diagnostics before a decision is constructed; no partial decision is returned.
An unknown or semantically unequal but well-formed profile is an unsupported
decision, not a successful approximation and not a newly invented diagnostic.

The correspondence decision shall compare source profile, temporal profile,
operator, interval convention, clock, predicate typing, execution anchor,
capture environment, history policy, progress policy, closure policy, TL
semantic profile, evaluator identity, and proposition mapping. A change in any
of these dimensions shall invalidate a previously derived projection even when
the formula bytes or observed Boolean outcome remain equal.

| Native profile and clock | Observation | TL profile | Decision |
|---|---|---|---|
| `quire.temporal.event-position.false-extension/v1` | closed and complete | `mltl.closed-trace/v1` | supported for the admitted bounded-future operators |
| `quire.temporal.event-position.false-extension/v1` | open prefix | `mltl.online-prefix/v1` | supported with a non-final prefix state |
| `quire.temporal.fixed-sample.false-extension/v1` | exact epoch, period, unit, and sample map | matching closed-trace or online-prefix profile | supported for the admitted bounded-future operators |
| `quire.temporal.timestamped-event.finite-window/v1` | any | current TL profiles | unsupported: clock and closure semantics differ |
| any native past-time profile | any | current TL profiles | unsupported: no reviewed TL past/history profile |

A closed but incomplete observation shall return an unsupported decision naming
`closure` or `history` as applicable, even when a TL closed-trace evaluator
could manufacture a Boolean through false extension. A complete closed
observation is required before the closed-trace row is supported.

The bridge shall support bounded future `eventually`, `always`, `until`, and
`release` over an event-position clock with false extension after closure when
the TL evaluator uses `mltl.closed-trace/v1`. Before closure, the same native
profile shall correspond only to `mltl.online-prefix/v1`; the result remains
incomplete where a future continuation can change the answer.

The bridge shall preserve inclusive integer intervals `0 <= a <= b` without
unit conversion, endpoint contraction, or horizon truncation.

The bridge shall support a fixed-sample clock under those same TL profiles only
when sample epoch, period, unit, and position mapping agree exactly. It shall
refuse timestamped-event and finite-window closure profiles because a closed TL
trace uses false extension outside the trace. It shall refuse every past-time
operator until a separately reviewed TL past/history profile exists.

For a one-position closed trace in which `p` is true, native
finite-window `always[0,1] p` evaluates true while TL false-extension
`always[0,1] p` evaluates false. The bridge shall use this discriminator to
prevent those profiles from being declared equivalent.

For TL `p until[a,b] q`, offsets before `a` do not require `p`; `p` is required
only from offset `a` through the offset immediately before the witness for `q`.
The bridge shall pin this lower-bound convention. The bridge shall refuse a
native profile with a different convention.

Only the issue #63 total-Boolean predicate/signal projection may populate TL
propositions. The bridge shall refuse partial, unknown, error-valued, or
implicitly coerced predicates.

Until issue #63 supplies that reviewed projection, every atom-bearing temporal
clause shall return an unsupported decision naming `predicate_projection`.
The bridge shall not use a hardcoded proposition, a Boolean default, or a
formula-text name as an interim fallback.

The bridge shall resolve capture bindings and the execution anchor before
temporal evaluation and include them in the projection identity.
The bridge shall return an unsupported projection rather than a Boolean result
when capture data or required history is missing or the anchor is incompatible.

An open observation shall expose current progress without claiming a final
closed-trace verdict. A closed observation shall bind its exact closure
position. Silence shall advance a deadline only when the native progress policy
and the clock provide the required position or sample. The bridge shall not
synthesize an event position from wall-clock passage alone. Late data that
changes a previously reported open-prefix state shall supersede that state
under a new observation identity. Late data shall not rewrite a closed
observation.

## Outputs

The bridge emits a supported or unsupported correspondence decision, or
existing structured diagnostics for an invalid input. The structural
`correspondence_ref` is the bridge/result join key; it is not a new FR-016
canonical object. The bridge does not emit a proof, execution verdict, evidence
record, accreditation statement, or release decision.

## Dependencies

- Issue #63 must supply a reviewed total-Boolean executable projection.
- FR-023 supplies the BoundClause and complete executable clause population.
- FR-012 supplies clause, anchor, and source-span identity.
- A reviewed native Quire temporal profile is the semantic source authority.
- A versioned TL semantic profile and evaluator revision are correspondence
  inputs, not source-language authorities.
- Issue #52 owns the native-source integration boundary. Issue #57 consumes
  this bridge for output-only FRETish mapping and is not a prerequisite.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-026-AC-1 | Supported closed and open event-position projections preserve every required identity and match the selected TL evaluator on a differential corpus for bounded `eventually`, `always`, `until`, and `release`. | Test (TC-039) |
| FR-026-AC-2 | A fixed-sample projection succeeds only when epoch, period, unit, and position mapping match exactly; a mutation to each field is refused. | Test (TC-039) |
| FR-026-AC-3 | The one-position `always[0,1] p` discriminator refuses finite-window semantics and distinguishes it from TL false extension. | Test (TC-039) |
| FR-026-AC-4 | Timestamped-event semantics and every past-time operator return an unsupported projection with no TL formula until corresponding reviewed TL profiles exist. | Test (TC-039) |
| FR-026-AC-5 | Partial, unknown, error-valued, or coerced predicates are refused; only issue #63 total-Boolean projections may populate propositions. | Test (TC-039) |
| FR-026-AC-6 | Open, closed, incomplete, missing-history, capture, silence, late-data, and supersession cases follow the declared clock, progress, history, anchor, and closure policies without synthesizing positions. | Test (TC-039) |
| FR-026-AC-7 | Mutating any correspondence dimension invalidates the projection; each refusal has no formula and names a deterministic, nonempty set of unmatched dimensions. | Test (TC-039) |
| FR-026-AC-8 | Supported and unsupported records conform to the exact v1 tagged shape; repeated equal inputs preserve structural identity, invalid inputs return no decision, and attributable evaluator results cite the complete correspondence, trace, and result-profile identities. | Test (TC-039) |
