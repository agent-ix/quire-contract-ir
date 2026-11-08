---
id: TC-461
title: "Project native temporal subjects and join owner results through correspondence"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-350
    type: verifies
  - target: ix://agent-ix/quire-spec-language/FR-045
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-050
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-052
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-526
    type: references
  - target: ix://agent-ix/quire-specification/FR-095
    type: references
  - target: ix://agent-ix/quire-specification/FR-092
    type: references
  - target: ix://agent-ix/quire-mltl/FR-001
    type: references
  - target: ix://agent-ix/quire-mltl/FR-003
    type: references
---
# TC-461: Project native temporal subjects and join owner results through correspondence

## Description

Verify the `agent-ix/quire-integration` temporal bridge against the native
owner inputs of FR-350: an admitted checked package, QSL's checked subject and
classification, the original native `EvidenceRequest` and exact occurrence,
the admitted observation revision and QObs views, and the original whole
`Result<EvaluatedEvidence, CallFailure>` call outcome; and against
the TL owners' public producers and readers, including the quire-mltl mapped
result and its non-evaluating predecessor validation. The case executes in
`agent-ix/quire-integration`, which owns the QSL-typed bridge; neither Contract
IR package gains a QSL dependency. Every fixture value is produced by its
genuine owner: native opaque carriers by `EvidencePackage::evaluate_temporal_evidence`, TL results by the quire-mltl
producer and reader, QObs views by their owner readers. No test constructs a
fake owner result. Expected bytes are computed by the test from the inputs by
FR-350's encoding rules; no frozen byte vector or digest is used.

## Test Procedure

1. **Admission (AC-2).** Emit one source and admit its bytes twice. Require a
   local subject and classification to project; substitute the other
   admission's subject, then its classification, and require
   `tl-projection-binding`. A clone retaining the admission projects. Alter
   support, a retained field, the definition selection and the surrounding
   closure independently; each returns `tl-projection-binding` before a TL
   artifact.
2. **Supported rows (AC-1, AC-11).** Project a future-only event-position
   formula under complete and open surrounding closure and require the
   closed-trace and online-prefix targets. Under each false-extension profile
   project `once[0,2] p`, `historically[0,2] p`, `p since[0,2] q`,
   `p triggered[0,2] q` and `once[1,1] p` alone with an origin-complete
   history. Strict-read every generated formula, proposition map, trace or
   history and request with its owner reader and require equality with the
   projection's values. Repeat each `[0,2]` case with a clock range start of 1,
   and again with `authoritative_origin` `Some(false)` and `None`; require
   `tl-projection-incomplete` with missing premise `history`.
3. **Unsupported rows (AC-2).** Classify timestamped-event (finite-window),
   mixed past-and-future and infinite-trace subjects; require
   `tl-projection-unsupported` carrying the complete classification and no
   projection.
4. **Native input binding (AC-9).** Use a request whose opaque anchor is an
   observation identity spelled like a different numeric position; require
   the numeric TL anchor of the matching position. With `request.occurrence`
   absent on a single-occurrence subject require success, and on a
   two-occurrence subject require native-premise `occurrence`. For each
   failure of FR-350 binding steps 2 to 8, supply one input exhibiting it and
   require exactly the code and premise FR-350 assigns, with no TL artifact.
   This includes: a wrong binding (`tl-projection-native-clock`); a counter
   clock cover (`clock-cover`); a foreign correspondence reference
   (`correspondence-reference`); wrong node, role and ordinal (`occurrence`);
   a foreign control (`control`); an absent anchor and a trigger with no
   admitted delivery or origin (incomplete `anchor`, `instance`); a delivery anchor mismatching the request (`anchor`); an activation
   contradiction and a retained `Refused` fault (`activation`) and a retained
   `Missing` fault (incomplete `activation`); missing, null and stale captures
   and an evicted capture (incomplete `capture`); wrong-type and other-anchor
   captures (`capture`); a position observation authority or revision of another bundle
   (`observation-revision`); an absent and a duplicate observation identity
   (incomplete `observation`, native-premise `observation`); a position of
   another clock revision (`position`); a gap in stamps (incomplete
   `position`); a timestamped range (`clock`); differing decision closure,
   surrounding closure, progress and contradicted completeness
   (`decision-closure`, `surrounding-closure`, `progress`, `completeness`);
   and absent completeness (incomplete `complete-input`).
   Change the selected request valuation and require the projected cell to change;
   shared values cannot supply an instance-bound cell, and a foreign trigger
   map cannot supply the selected instance.
5. **Valuations (AC-6, AC-9).** Supply complete original request maps with
   explicit false and require false in the TL input. Select shared atoms from
   `valuations` and instance-bound atoms only from the selected trigger's
   `instance_valuations`, through `native_atom_operands`. An unknown atom in
   either a shared map or any instance map returns `tl-projection-valuation`
   with the original request position and atom index. Missing, undefined or
   absent selected values return incomplete `valuation{position, operand}`;
   a foreign trigger's valid map does not satisfy the selected cell. Maps admit
   one entry per atom and instance, so repeated/disagreeing rows have no request
   form. Require all provided map entries, including unused foreign-trigger
   maps, to be charged. Project an aggregate atom and require
   `tl-projection-formula`; an integer interval without an upper bound or above
   `u32::MAX` returns native-premise `interval`.
6. **Fixed sample (AC-4).** Project a fixed-sample subject with a nanosecond
   unit and matching epoch and period. Change only the period with equal
   formula bytes and require native-premise `clock`; reinterpret samples as
   event counts and require native-premise `clock`; omit a sample and require
   incomplete `position`. With a millisecond unit, require native-premise
   `clock-representation` for a future target and `tl-projection-owner`
   carrying quire-mltl `ExpectedMismatch("clockBinding")` for a past target,
   with no converted value.
7. **Record members (AC-3, AC-10, AC-11).** Compute the expected
   correspondence record from the inputs by FR-350's member table and encoding
   rules and require byte equality for a future, a past and a fixed-sample
   projection. For each member's source input, mutate only that input,
   require exactly that member to change, and require `read_tl_projection` of
   the original bytes against the mutated inputs to return
   `tl-projection-correspondence-mismatch`. Require the `models` member to
   equal `selected_model_packages()` in ascending order, and two packages
   selecting the same models in different source orders to yield an identical
   `models` member. Require the record to contain no JSON number. Require the request bytes in the record to equal
   the owner-read request, its `subject_identity` to be the native subject name
   and its `correspondence_identity` the profile name, and the request bytes to
   contain no record bytes. Supply a wrong subject name and a wrong profile name
   and require refusal.
8. **Strict reading (AC-6).** For projection and join bytes, insert an unknown
   member, duplicate a member, remove a member, reorder two members, add
   whitespace and append a trailing byte; require refusal with no partial
   value. Supply each owner reader refusal and require the typed owner error.
9. **Join rows (AC-5).** From genuine native and TL results, construct one pair
   reaching each of R1 to R8 except the defensive R4 and require the decision, comparison, cause list
   and Boolean of FR-350's table, including an equal-final pair whose
   settlement differs (`P` false, R6), a final/pending pair (R6), a TL
   `Failed` result (R2) and a native predecessor mismatch (R6). For each pair
   of R1 to R6 rows build a pair
   satisfying both and require the first decision and both causes once in row
   order. Require the native issued identity or typed original failure and complete TL
   bytes in every join record. Keep original admission refusal/incomplete and
   fault outcomes distinct from absence and from result `NonValue`. Build
   the expected join record by FR-350's rules and require byte equality.
10. **Corrections (AC-5, AC-10).** Advance a real QObs progress fact and
    change native request inputs, using the original evaluator-written ledger
    and issued predecessor identity. Require R7 with both direct predecessors;
    require R6 for one-sided correction, original with `previous`, correction
    without `previous`, a previous join whose native side was not admitted,
    another prior issued carrier, and TL `missing-predecessor`,
    `unexpected-predecessor` and `predecessor-mismatch`. A relation-only change
    is refused by QSL. Require native subject, occurrence, source-map and request
    mismatch to remain R1. Verify actual changed truth, support, settlement,
    latest ledger and lineage through immutable result borrows, with original
    carrier/ledger unchanged. A changed read snapshot cannot substitute for a
    carrier. TL equal-identity different-content predecessors remain refused.
    Exercise QSL's exact/one-short `lineage` limit, not retired native
    predecessor-byte budgets; the stopped result is incomplete with no Boolean.
11. **Limits (AC-8).** Require `TlLimits::default()` to equal FR-350's
    defaults. For each limit and each operation that charges it, compute the
    expected count `n` from the operation's inputs by FR-350's Limits table
    (for example `request.positions.len()`, all provided valuation-map entries, the owner-read
    formula's node count, the cause-list length, or the largest checked byte
    value), without reading the bridge's usage. Run with setting `n` and
    require admission and reported usage `n`; run with `n - 1` and require
    `limit-reached{limit, value: n, setting: n - 1}`, an unvisited rejected
    unit and no partial value, all other settings ample. Set two limits so one
    action crosses both and require only the first in table order. For
    `max_bytes`, compute `n` as the record length with the setting itself
    embedded and test the fixed point: the smallest setting `s` with
    `record_len(s) <= s` admits and `s - 1` refuses with `value` equal to
    `record_len(s - 1)`. The owner refusal at the
    lowered byte ceiling is defensive and not a tested clause.
12. **No evaluation (AC-6).** After each join, require the retained TL source
    result to equal the supplied one with unchanged `usage()`, and the retained
    native outcome to be the original supplied outcome, with immutable result
    borrows retaining truth, support, settlement, checked subject and ledger. Inspect the bridge's call graph
    and require no call path from any bridge operation to `evaluate_temporal_evidence`,
    `quire_mltl::report::evaluate` or `quire_mltl::report::read`.

## Expected Results

- Each supported row projects with owner-admitted values; each cutoff case is
  `incomplete` with `history`; each unsupported row returns the complete
  classification and no artifact.
- Each binding, valuation, capture, observation and fixed-sample defect
  returns its typed error before any TL artifact, and a missing value is never
  false.
- The bridge's correspondence and join records equal the bytes the test
  computes from FR-350's rules; each input mutation changes exactly its member
  and fails strict re-reading.
- Every decision row is reached with its decision, causes and Boolean; only R7
  carries a Boolean; corrections agree only with the admitted native issued identity/lineage and TL complete direct
  predecessor.
- Each limit admits at its measured count and refuses one over with the named
  cause, and no bridge operation invokes an evaluator.
