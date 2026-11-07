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
  - target: ix://agent-ix/quire-spec-language/FR-043
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
classification, the native `EvidenceRequest`, `ValuationEvidence`, exact
occurrence, assessed instance and observation reference, the admitted
observation revision and QObs views, and the native FR-052 report; and against
the TL owners' public producers and readers, including the quire-mltl mapped
result and its non-evaluating predecessor validation. The case executes in
`agent-ix/quire-integration`, which owns the QSL-typed bridge; neither Contract
IR package gains a QSL dependency. Every fixture value is produced by its
genuine owner: native reports by `evaluate_v2`, TL results by the quire-mltl
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
3. **Unsupported rows (AC-2).** Classify timestamped-event, mixed
   past-and-future, infinite-trace unbounded and finite-window equivalence
   subjects; require `tl-projection-unsupported` carrying the complete
   classification and no projection.
4. **Native input binding (AC-9).** Use a request whose opaque anchor is an
   observation identity spelled like a different numeric position; require
   the numeric TL anchor of the matching position. With `request.occurrence`
   absent on a single-occurrence subject require success, and on a
   two-occurrence subject require refusal. Independently supply a wrong
   binding, wrong node, role and ordinal, a control outside the subject's
   requirements, a mismatched instance anchor, an empty capture instance, a
   capture at another anchor, a missing, null, stale and wrong-type capture, a
   retained `Refused` fault, an evicted capture, and an absent, duplicate and
   foreign observation identity; require each typed error and no TL artifact.
   Add per-position request valuation maps that contradict `ValuationEvidence`
   and require an unchanged projection.
5. **Valuations (AC-6).** Supply complete rows with an explicit false value and
   require it in the TL input. Then independently supply a row for another
   occurrence, a position at `request.positions.len()`, another anchor,
   another capture instance, another revision and a non-leaf operand; require
   `tl-projection-valuation` with that `ValuationCause` and row index. Repeat a
   row with the same and with the opposite value and require the duplicate
   refusal naming both row indices. Omit one row and require
   `tl-projection-incomplete`. Project a formula with an aggregate atom and
   require `tl-projection-formula`.
6. **Fixed sample (AC-4).** Project a fixed-sample subject with matching epoch,
   period and unit. Change only the period with equal formula bytes and
   require refusal; reinterpret samples as event counts and require refusal;
   omit a sample and require `incomplete`. Select a unit the TL request owner
   cannot represent and require that owner's typed refusal, with no converted
   value.
7. **Record members (AC-3, AC-10, AC-11).** Compute the expected
   correspondence record from the inputs by FR-350's member table and encoding
   rules and require byte equality for a future, a past and a fixed-sample
   projection. For each member's source input, mutate only that input,
   require exactly that member to change, and require `read_tl_projection` of
   the original bytes against the mutated inputs to refuse. Reorder the
   selected model identities of an otherwise equal package and require a
   different `models` member. Require the request bytes in the record to equal
   the owner-read request, its `subject_identity` to be the native subject name
   and its `correspondence_identity` the profile name, and the request bytes to
   contain no record bytes. Supply a wrong subject name and a wrong profile name
   and require refusal.
8. **Strict reading (AC-6).** For projection and join bytes, insert an unknown
   member, duplicate a member, remove a member, reorder two members, add
   whitespace and append a trailing byte; require refusal with no partial
   value. Supply each owner reader refusal and require the typed owner error.
9. **Join rows (AC-5).** From genuine native and TL results, construct one pair
   reaching each of R1 to R8 and require the decision, comparison, cause list
   and Boolean of FR-350's table. For each pair of R1 to R6 rows build a pair
   satisfying both and require the first decision and both causes once in row
   order. Require the native and TL result bytes in every join record. Build
   the expected join record by FR-350's rules and require byte equality.
10. **Corrections (AC-5, AC-10).** Advance a real QObs progress fact to create
    native and TL corrections. Require R7 when `previous` holds both direct
    predecessors, and R6 for a one-sided correction, an original with
    `previous`, a correction without `previous`, a previous join whose native
    side was not admitted, a native full-predecessor mismatch, and TL
    `missing-predecessor`, `unexpected-predecessor` and `predecessor-mismatch`.
    Substitute a different complete predecessor with equal result identity on
    each side and require R6 and `read_tl_join` refusal. Require a native
    `PredecessorLimit` to return `tl-join-limit-reached` with no join.
11. **Limits (AC-8).** Require `TlLimits::default()` to equal FR-350's
    defaults. For each limit and each operation that charges it, run the
    operation with ample settings and read its usage `n` for that limit; rerun
    with setting `n` and require admission, then with `n - 1` and require
    `limit-reached{limit, value: n, setting: n - 1}`, an unvisited rejected
    unit and no partial value, all other settings ample. Set two limits so one
    action crosses both and require only the first in table order.
12. **No evaluation (AC-6).** After each join, require the retained TL source
    result to equal the supplied one with unchanged `usage()`, and the retained
    native report to equal the supplied report. Inspect the bridge's call graph
    and require no call path from any bridge operation to `evaluate_v2`,
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
  carries a Boolean; corrections agree only with both exact direct
  predecessors.
- Each limit admits at its measured count and refuses one over with the named
  cause, and no bridge operation invokes an evaluator.
