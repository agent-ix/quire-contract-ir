---
id: FR-026
title: "Establish native temporal and TL result correspondence"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-025
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-051
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-052
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-004
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-042
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-011
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-012
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-007
    type: depends_on
  - target: ix://agent-ix/tl-syntax/IF-005
    type: implements
  - target: ix://agent-ix/tl-syntax/VO-007
    type: implements
---
# FR-026: Establish native temporal and TL result correspondence

## Description

When a checked native temporal subject and its complete predicate valuations are available under selected owner contracts, the Contract IR temporal bridge SHALL construct and owner-read the exact TL artifacts for that profile and join native and TL results only through their exact selected validated mapping views.

The bridge profile is `quire.contract.native-temporal-correspondence/v1`.
Contract IR owns correspondence, construction and the agreement decision; it
does not own native/TL source, observation authority, either evaluator or either
result vocabulary.

## Temporal subsystem and API

The implementation SHALL be organized as
`temporal::{admission,formula,valuation,request,correspondence,join,decision,reader}`
over the shared contract/canonical/limit/identity/diagnostic foundations and the
FR-025 predicate subsystem. Public operations SHALL accept only
constructor-private owner views and return closed decisions:

```text
temporal::project(&ValidatedTemporalSubject,
  &ValidatedPredicateProjection, ObservationViews, TargetSelection, Limits)
  -> TemporalProjectionDecision
temporal::read_projection(bytes, ExpectedTemporalProjection, Limits)
  -> Result<ValidatedTemporalProjection, TemporalDecision>
temporal::join(&ValidatedTemporalProjection,
  Option<&NativeTemporalResultView>, Option<&TlMappedResultView>,
  Option<&ValidatedTemporalJoin>, Limits)
  -> TemporalJoinDecision
temporal::read_join(bytes, ExpectedTemporalJoin, Limits)
  -> Result<ValidatedTemporalJoin, TemporalDecision>
```

`NativeTemporalResultView` is the QSL FR-052 constructor-private result view,
not one Quire Protocol predicate-result mapping. The optional prior join is
required exactly when either owner result is a correction and binds the two
owner-specific predecessor identities without equating their identity domains.
No API accepts a parser, evaluator, callback, plugin, trust flag or raw mirrored
owner record, and Contract IR invokes neither evaluator.

## Owner admission and independent axes

Before construction, the bridge SHALL admit and cross-check:

- one QSL `quire.checked-temporal-subject/v1` view and all referenced FR-025
  checked predicate views;
- observation position-ledger, clock, capture, progress, closure, completeness
  and availability views under their exact FR-004 contracts; and
- immutable target formula, semantic, history/trace, request, evaluator-report
  and selected-result-mapping contract/schema revisions.

Decision-scope progress, decision-scope closure, surrounding-execution progress
and surrounding-execution closure are four distinct assertions. Each carries
only `open` or `closed`, its own owner reference/revision/digest, scope, authority,
clock, boundary and source population. Completeness is a fifth separate owner
assertion with `complete`, `incomplete` or `contradicted`. No combined
`closed-complete`/`closed-incomplete` value exists. Availability, activation,
truth and settlement are also independent.

The surrounding-execution closure assertion alone selects online-prefix versus
closed input. Completeness independently decides whether the required artifact
can be constructed. A closed but incomplete/contradicted population yields no
false-padded request. A decision scope may be closed while its surrounding
execution remains open.

## Formula and valuation construction

Construction traverses the owner-checked native tree in left-to-right postorder,
preserves node occurrences without deduplication, assigns contiguous
operand-before-consumer IDs, maps each `holds` leaf through its exact FR-025
proposition and preserves every inclusive `[a,b]` with checked
`0 <= a <= b <= u32::MAX`.

Future-only subjects emit `tl-syntax.formula/v1` under exactly
`mltl.closed-trace/v1` or `mltl.online-prefix/v1`. Pure bounded-past subjects
emit `tl-syntax.formula/v2` under exactly
`mltl.origin-complete-history/v1` and the
`tl-syntax.past-operators/v1` catalog. The v1 past profile refuses every future
or mixed graph, timestamp/dense clock, weak previous, unbounded interval and
unknown profile. Such out-of-profile shapes are `unsupported`; missing admitted
history/valuation/authority under an otherwise supported profile is
`incomplete` or `unavailable`, not unsupported.

For each required position and proposition, the bridge requires one explicit
FR-025 Boolean valuation bound injectively to the same position, anchor,
snapshot, invocation, capture, population and observation revision. False is an
explicit admitted cell even where the target trace/history encodes it by
omission. Missing, duplicate, swapped, replayed or foreign cells refuse.

Future projections construct and owner-read `tl-mltl.trace/v1` plus the selected
future request. Past projections construct and owner-read
`tl-mltl.position-history/v1`, `tl-mltl.history-requirement/v1` and the selected
past request/report contracts. An `execution-origin` boundary may authorize
pre-origin false extension; a `history-cutoff` never does, including after state
restoration. Event-position and exact fixed-sample clocks retain their exact
mapping. Silence, timestamp order, rounding, interpolation and implicit samples
create no position or history.

Every emitted artifact is admitted by its real public owner reader. Formula,
valuation, trace/history, request and correspondence identities each hash their
own canonical bytes and complete contract/input tuple; no one identity
substitutes for another. A source display-only mutation does not change semantic
artifact bytes, while every semantic or contract mutation changes the applicable
identity.

## Native and TL evaluation boundary

The projection constructs two sibling evaluator inputs from the same admitted
native subject, predicate correspondence, complete explicit valuation cells,
position ledger, clock, capture, four progress/closure assertions, completeness,
availability and limits. The native sibling selects QSL
`quire.native-temporal-request/v1`; the TL sibling selects the applicable
`tl-mltl` trace/history and request contracts. Both siblings retain the bridge
correspondence identity and the exact source-owner identities from which each
field was derived. Neither input is derived from the other input's bytes.

QSL strict-reads and evaluates only its native request and publishes
`quire.native-temporal-result/v1`. TL-MLTL strict-reads and evaluates only its
TL request and derives `tl-mltl.contract-ir-result-map/v1`. Contract IR accepts
only those constructor-private formula-wide views under exact selections. Quire
Protocol's `quire.protocol.contract-ir-result-map/v1` remains the independent
owner input to each FR-025 leaf valuation; it is not a native temporal result
and its leaf Boolean SHALL NOT be compared to a temporal formula Boolean.

Each formula-wide view SHALL bind the same bridge correspondence, native
subject, formula semantics, valuation population, positions, anchor and
applicable request. Each preserves assessment execution, a typed Boolean value
or non-value, all four progress/closure assertion refs/states, completeness
assertion/state/facts, activation, settlement, exact decision support and
correction/direct predecessor. Contract IR verifies each view against the
projection, translates support only through the projection's exact
position-to-observation bijection, and then compares normalized structures. It
does not compare owner label strings or require owners to share an identity or
wire vocabulary.

One native result and one TL result are formula-wide. The number of checked
predicate leaves is irrelevant to result arity: their distinct Quire Protocol
results have already supplied explicit valuation cells under FR-025. A missing,
duplicate, extra, reordered-to-a-different-position or foreign leaf valuation
therefore refuses projection before either evaluator input exists.

The closed join table is:

| Validated mapped state | Join decision |
| --- | --- |
| both formula values equal and all applicable premises/support equal | `agreement` / `equal-final`, that Boolean |
| both formula non-values are equal pending/unsettled under an open decision scope | `agreement` / `equal-pending`, no Boolean |
| required result or producer unavailable | `unavailable` / `not-compared` |
| either mapping reports unsupported | `unsupported` / `not-compared` |
| either reports resource/input incomplete | `incomplete` / `not-compared` |
| either reports failed | `failed` / `not-compared` |
| either refuses or has stale/wrong subject | `refused` / `not-compared` |
| unequal values/premises, contradicted deciding support, or identity collision | `conflict` / `mismatch` |

Every non-final decision contains no Boolean and a nonempty ordered typed cause
set. A final value on an open decision scope requires exact decisive support and
the selected profile's continuation-stability rule; otherwise it is pending.
A closed decision scope cannot be pending. Incomplete facts outside exact
complete decision support remain visible without erasing the final value.

Corrections retain immutable owner bytes and exact direct predecessors. An
original/original pair requires no prior join. A corrected/corrected pair
requires one strict-read prior join whose native and TL source-result identities
and digests equal the respective direct predecessors and whose correspondence
equals the corrected projection's declared predecessor correspondence. A mixed
original/corrected pair, missing prior join, self-edge, wrong-domain
substitution, predecessor mismatch or revision regression conflicts. Each owner
validates its own result graph; Contract IR validates only this paired direct
edge and never claims global lineage validation. A late/corrected input cannot
reopen or rewrite a prior result in place.

## Strict reading and resources

Projection and join readers reject unknown/duplicate/missing/out-of-order fields,
trailing/noncanonical bytes, invalid tagged shapes, unknown contracts/labels,
non-tree references, owner-reader rejection, identity/digest mismatch,
cross-wired positions/scopes/clocks/history/corrections and forbidden axis
combinations. Limits independently bound bytes, depth, strings, nodes,
positions, valuations, support, causes and total work. Work is charged before
allocation/traversal; exact limits pass and one-over/allocation failure exposes
no partial artifact or decision.

## Semantic discriminators

The future one-position `always[0,1] p` case distinguishes false-extension from
finite-window semantics. `p until[1,2] q` requires `p` only from offset 1 to the
witness. Past O/H/Y/S/T match MRS-003 including zero width, nonzero lower bound,
pre-origin false extension and S/T duality. These vectors are evaluated by the
independent owners; Contract IR proves only artifact and result correspondence.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-026-AC-1 | Postorder construction maps every supported future or pure-past occurrence, binds every holds leaf, constructs complete position-bound valuation plus sibling native/TL requests, rejects cell replay/swapping, and passes every selected public reader. | Test (TC-039) |
| FR-026-AC-2 | Each formula/valuation/trace-history/request/correspondence identity changes for exactly its semantic/contract inputs, is invariant to display-only or set-order changes where specified, and cannot substitute for another owner identity. | Test (TC-039) |
| FR-026-AC-3 | Future event-position/fixed-sample subjects select online versus closed solely from surrounding closure; pure past selects formula-v2/origin-complete history; closed incomplete, cutoff history, timestamp/dense, mixed/future-in-past and unknown profiles return the correct non-value with no substitute formula. | Test (TC-039) |
| FR-026-AC-4 | Every inclusive interval including zero width and u32::MAX is preserved; negative, fractional, inverted, unbounded, overflowing or larger bounds refuse before partial construction. | Test (TC-039) |
| FR-026-AC-5 | Execution, four independent open/closed progress/closure axes, completeness, formula value/non-value, settlement, decision premises, availability, activation and join kind remain independently encoded; no leaf result is treated as a formula result and no non-final join exposes a Boolean. | Test (TC-039) |
| FR-026-AC-6 | Both formula-result owner readers reject stale or cross-wired identities and impossible combinations; equal complete mapped premises agree, every disagreement conflicts or preserves its exact non-value, and paired direct corrections require the exact strict-read prior join while preserving prior bytes without claiming global graph validation. | Test (TC-039) |
| FR-026-AC-7 | Independent QSL-native and TL future/past evaluations detect finite-window, wrong-until, wrong-origin, wrong-S/T and Boolean-fallback mutations while Contract IR constructs requests and compares results but performs no evaluation. | Test (TC-039) |
| FR-026-AC-8 | Missing/duplicate/foreign artifacts or valuations, substituting a Protocol leaf result for a native temporal result, owner-reader rejection, resource/allocation failure and every contract/axis mismatch return one deterministic non-admitted decision with no partial artifact, parser/evaluator/callback/network invocation or alternate authored language. | Test (TC-039) |

## Dependencies

FR-025 supplies predicate correspondence and Quire Protocol-backed leaf
valuations. QSL FR-051 supplies the checked temporal subject and QSL FR-052
supplies the native request/result owner boundary. Quire Observation FR-004
supplies position, clock, capture and scope assertions. TL-MLTL FR-018 supplies
the independently validated TL result mapping. tl-syntax FR-011/012/014 owns
the selected temporal graphs and profiles.

## Status

Implemented for `quire-contract-ir#71` against immutable QSL
`f1700a9264d6d3bcdd07e0f77b70f3dae9ed4c07`, Quire Observation
`9ac80e93f4b68a2c7d5a337f9a448ad10de798fc`, Quire Protocol
`34d1752e6c5f789a52ccf115b0694eedd96cdd46`, tl-syntax
`842d82553f045eb69a7f38745756d968254fc25e`, and tl-mltl
`22862189ac4eb515ab84928faec25b2eac47d835` owner revisions. The implementation
constructs and strict-reads sibling QSL-native and TL requests, admits both
formula-wide result views, and joins them without parsing or evaluating either
owner language inside Contract IR.
