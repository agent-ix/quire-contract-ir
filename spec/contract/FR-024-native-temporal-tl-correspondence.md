---
id: FR-024
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
---
# FR-024: Bind native temporal projections to exact TL correspondence

## Description

The IR shall define a versioned, fail-closed projection from a validated native
Quire temporal clause to an exact TL correspondence. Native Quire remains the
sole authored formal-clause language; a TL formula is derived output and never
an alternative authoring surface.

The projection shall preserve every semantic and provenance dimension needed to
decide whether the native temporal clause and the selected TL evaluator profile
denote the same obligation. Unsupported dimensions shall produce an explicit
refusal instead of approximation, omission, or an unqualified formula.

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

## Behavior

The bridge shall return exactly one of two tagged results:

- a supported projection containing a TL formula and the complete source,
  clause, model, bridge, TL-profile, proposition-map, and evaluator identities;
  or
- an unsupported projection containing no TL formula and an ordered, nonempty
  set of unmatched semantic dimensions.

The correspondence decision shall compare source profile, temporal profile,
operator, interval convention, clock, predicate typing, execution anchor,
capture environment, history policy, progress policy, closure policy, TL
semantic profile, evaluator identity, and proposition mapping. A change in any
of these dimensions shall invalidate a previously derived projection even when
the formula bytes or observed Boolean outcome remain equal.

The bridge shall support bounded future `eventually`, `always`, `until`, and
`release` over an event-position clock with false extension after closure when
the TL evaluator uses `mltl.closed-trace/v1`. Before closure, the same native
profile shall correspond only to `mltl.online-prefix/v1`; the result remains
incomplete where a future continuation can change the answer.

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

The bridge emits a supported or unsupported correspondence record. It does not
emit a proof, execution result, evidence record, accreditation statement,
release decision, or new FR-016 canonical object.

## Dependencies

- Issue #63 must supply a reviewed total-Boolean executable projection.
- FR-023 supplies the BoundClause and complete executable clause population.
- FR-012 supplies clause, anchor, and source-span identity.
- A reviewed native Quire temporal profile is the semantic source authority.
- A versioned TL semantic profile and evaluator revision are correspondence
  inputs, not source-language authorities.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-024-AC-1 | Supported closed and open event-position projections preserve every required identity and match the selected TL evaluator on a differential corpus for bounded `eventually`, `always`, `until`, and `release`. | Test (TC-039) |
| FR-024-AC-2 | A fixed-sample projection succeeds only when epoch, period, unit, and position mapping match exactly; a mutation to each field is refused. | Test (TC-039) |
| FR-024-AC-3 | The one-position `always[0,1] p` discriminator refuses finite-window semantics and distinguishes it from TL false extension. | Test (TC-039) |
| FR-024-AC-4 | Timestamped-event semantics and every past-time operator return an unsupported projection with no TL formula until corresponding reviewed TL profiles exist. | Test (TC-039) |
| FR-024-AC-5 | Partial, unknown, error-valued, or coerced predicates are refused; only issue #63 total-Boolean projections may populate propositions. | Test (TC-039) |
| FR-024-AC-6 | Open, closed, incomplete, missing-history, capture, silence, late-data, and supersession cases follow the declared clock, progress, history, anchor, and closure policies without synthesizing positions. | Test (TC-039) |
| FR-024-AC-7 | Mutating any correspondence dimension invalidates the projection; each refusal has no formula and names a deterministic, nonempty set of unmatched dimensions. | Test (TC-039) |
