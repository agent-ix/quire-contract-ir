---
id: SR-202611010306
title: 'IR-1252 spec-review/dependency: authenticated schedulability carrier'
type: SpecReview
analysis: dependency
scope: agent-ix/quire-contract-ir; IR-1252; five changed spec files
review_set: subset
---

## Summary

Ticket: IR-1252. Frozen PR #805 specification diff reviewed against the independently selected QSpec main contract. No defects found in this scoped method.

## Verdict

**PASS** — The added structured TC-048 reference resolves to QSpec TC-976. QSpec FR-322/FR-482 define the upstream carrier/body; IR FR-038 admits and exposes them, FR-019 exposes that model API, and FR-051 preserves supplied optional carrier rows. These are logical prerequisite edges, not a runtime cycle: QSpec → FR-038 → FR-019; FR-038 raw-row contract → FR-051; FR-038/FR-051 → TC-048/TC-451 evidence. QSpec contract and model interfaces are enablement; strict admission and complete encoding are features. No QSL implementation dependency is introduced.

No applicable AssuranceProfile was found under spec/. No source/Cargo/build/Kani work was run. Object review is not separately selected: this diff extends an existing checked-package interface and its language-local views, not a new cross-context domain-object payload. Evidence/risk/criterion-strength/architecture/security lenses are not selected for this narrow carrier/interface extension.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined scope

```yaml
scope:
- id: spec/checked_package/functional/FR-038-consume-checked-package-v2.md:addition-1
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: '### Authenticated schedulability claim carrier (planned until IR-1252
    lands)'
- id: spec/checked_package/functional/FR-038-consume-checked-package-v2.md:addition-2
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "The strict reader SHALL admit the optional top-level `schedulability_claims`\n\
    index defined by [QSpec FR-322](ix://agent-ix/quire-specification/FR-322),\nAC-300\
    \ and \u201CSchedulability claim carrier\u201D, and validate every\n`claim`/`schedulability`\
    \ body under\n[QSpec FR-482](ix://agent-ix/quire-specification/FR-482). The closed\n\
    `ClaimForm` vocabulary gains `Schedulability`. The index is absent when there\n\
    is no schedulability claim occurrence; otherwise it contains exactly one\nclosed\
    \ `{declaration, occurrence}` row for each such occurrence. The row's\n`declaration`\
    \ is a local structural `CheckedNodeId`; it"
- id: spec/checked_package/functional/FR-038-consume-checked-package-v2.md:addition-3
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'After aggregate-order validation and before the FR-482 body step, the
    reader

    SHALL visit index rows in array order. Before inspecting a row it charges one

    validation visit against the caller''s existing finite reader budget. It

    checks the local declaration join before the owned `claim` occurrence join,

    then repeated keys before canonical order; total occurrence coverage is last.

    Canonical row order is structural declaration identity followed by occurrence

    ordinal. Exhaustion returns the existing incomplete/resource-exhaustion

    reader result at the first uncharged row, without an admitted package'
- id: spec/checked_package/functional/FR-038-consume-checked-package-v2.md:addition-4
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'A missing index or missing occurrence row refuses

    `invalid_package`/`invalid-value` at `/schedulability_claims`. An absent,

    foreign, wrong-kind or wrong-form declaration refuses the same code/cause at

    `/schedulability_claims/{i}/declaration`; an unowned occurrence or an

    occurrence of a role other than `claim` refuses there at `/occurrence`.

    A repeated pair refuses `invalid_package`/`conflicting-definition` at the

    later row, before the order check. A descending distinct pair refuses

    `invalid_package`/`invalid-value` at the later row. Missing or extra row

    members and negative or non-integer occu'
- id: spec/checked_package/functional/FR-038-consume-checked-package-v2.md:addition-5
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The reader SHALL apply the complete FR-482 body checks in its published

    order: shape, literals, form fit, task order, unit, then task-set premises.

    This is the last body step, after the protocol step, visiting schedulability

    nodes in ascending structural node-id order. The body has the ordered five

    claim bindings and eleven bindings per task, retains exact rational time

    values in one package time unit, and has Boolean semantic type, no declaration

    or owner and empty dependencies. Admission checks the `schedulable` and

    `response` forms; fixed-priority, EDF and AMC policy premises; task ordering'
- id: spec/checked_package/functional/FR-038-consume-checked-package-v2.md:addition-6
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The raw wire row `CheckedSchedulabilityClaimRow` has public

    `declaration: CheckedNodeId` and `occurrence: u64` fields. It is an untrusted

    producer value, not an authenticated view. The admitted package''s

    `schedulability_claim_rows(&self) -> Option<&[CheckedSchedulabilityClaimRow]>`

    borrows its retained index for complete re-encoding under

    [FR-051](./FR-051-encode-checked-package-v2-envelope.md). Complete read/encode/read

    preserves every row and body and the absent member; the lossless reader check

    includes this member rather than silently dropping it.'
- id: spec/checked_package/functional/FR-038-consume-checked-package-v2.md:addition-7
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'After full admission, `CheckedPackageV2::schedulability_claims(&self)`
    SHALL

    return `impl ExactSizeIterator<Item = CheckedSchedulabilityClaimRef<''_>> + ''_`

    in canonical index order. Each borrowed view has private fields and no public

    constructor; `declaration(&self) -> &CheckedNodeId`,

    `occurrence(&self) -> &CheckedOccurrence` and `body(&self) -> &CheckedTerm`

    borrow the exact admitted graph identity, owned claim occurrence and FR-482

    body. Thus policy, task fields, units and response bound are available through

    the existing typed term tree without copying the body or parsing raw JSON.

    An admi'
- id: spec/checked_package/functional/FR-038-consume-checked-package-v2.md:addition-8
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The borrowed claim view SHALL also expose `kind() -> CheckedSchedulabilityKind`

    (closed variants `Schedulable`, `Response`),

    `policy() -> CheckedSchedulingPolicy` (`FixedPriority`, `Edf`, `Amc`),

    `response_task() -> Option<&str>`, `response_bound() -> Option<CheckedLiteralRead<''_>>`

    and `tasks() -> impl ExactSizeIterator<Item = CheckedSchedulabilityTaskRef<''_>>
    + ''_`.

    The two response accessors are present exactly for `Response`. Each task view

    has `name() -> &str`, `arrival() -> CheckedTaskArrival` (`Periodic`, `Sporadic`),

    `criticality() -> CheckedTaskCriticality` (`Lo`, `Hi`), and immutable'
- id: spec/checked_package/functional/FR-038-consume-checked-package-v2.md:addition-9
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The IR read ends at that authenticated claim. The QSL FR-418 checker owns

    joining it to a selected timed model, resolving source release bindings and

    recomputing source/scheduler successors. `TaskTransitionTemplate` and

    `TaskProductCertificate` remain backend-provider certificate values; exact

    `TaskTransitionIdentity` remains concrete replay/search evidence. This carrier

    adds none of them to the checked-package wire or the cycle-free model API.'
- id: FR-038-AC-396
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-1252 lands. The public strict reader admits complete packages
    constructed around QSpec TC-976 SCC-01 and SCC-02, retains one row for occurrence
    0 and two distinct rows for the shared node's occurrences 0 and 1 respectively,
    and exposes their exact borrowed declaration, claim occurrence and FR-482 body
    in canonical order. A package with no schedulability occurrence omits the index
    and returns an empty typed iterator. Equal display names or source regions do
    not select or merge claims.
- id: FR-038-AC-397
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-1252 lands. SCC-03/04/05/12 refuse `malformed_wire` at
    their published row/member loci. SCC-06/07/08 refuse `invalid_package`/`invalid-value`
    at the published declaration/occurrence loci; additional absent-local, wrong-kind
    and wrong-role controls have the same corresponding refusal. SCC-09 refuses `invalid_package`/`conflicting-definition`
    at the later row; SCC-11 refuses `invalid_package`/`invalid-value` there. SCC-10
    and an omitted required index refuse that code/cause at `/schedulability_claims`.
    Each refusal compares the exact typed primary locus and exposes no admitted p
- id: FR-038-AC-398
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-1252 lands. Wire and graph/aggregate validation precede
    carrier joins; declaration precedes occurrence, repeated key precedes order, coverage
    follows all rows, and carrier validation precedes FR-482 body validation. Combined
    defects distinguish each adjacent check. Each visited row charges one existing
    validation visit before inspection; an exact measured budget admits the positive
    package, and one below the measured charge refuses with the existing incomplete/resource-exhaustion
    result at the first uncharged row and no partial typed claim view.
- id: FR-038-AC-399
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'planned until IR-1252 lands. The reader admits the full FR-482 schedulable/response
    bodies and validates its shape, literal vocabulary, form fit, strict task order,
    common time unit and policy-specific task-set premises in that order after the
    protocol body step. QSpec FR-482-AC-3''s adverse bodies, including negative/excess
    BCET, wrong time unit, duplicate fixed-priority priorities and the EDF equal-priority
    positive control, have their exact published codes, causes and loci. Existing
    graph refusals for nonempty dependencies, owner, semantic type or identity precede
    the dependent body checks. '
- id: FR-038-AC-400
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'planned until IR-1252 lands. TC-048 reads QSpec''s `proposals/checked-package-v2/schedulability-claim-vectors.json`,
    TC-976 and FR-482/TC-967 cases by reference from a selected QSpec checkout, constructs
    complete IR packages around their resolved-node context, applies every carrier
    mutation and compares admission or exact typed code, cause and primary locus.
    The evidence states the selected checkout revision; no upstream schema, fixture
    or vector is copied into IR and the row vectors are not treated as complete packages.
    Complete public read/encode/read retains every identity row and the body, '
- id: spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md:addition-15
  path: spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md
  role: examined
  excerpt: '### Schedulability carrier preservation (planned until IR-1252 lands)'
- id: spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md:addition-16
  path: spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md
  role: examined
  excerpt: '`CheckedPackageSidecarsRef<''a>` SHALL gain

    `schedulability_claims: Option<&''a [CheckedSchedulabilityClaimRow]>` using

    [FR-038](./FR-038-consume-checked-package-v2.md)''s raw identity row. The

    complete encoder SHALL omit the top-level index for `None` and encode exactly

    the supplied rows for `Some`, retaining declaration identities, numeric

    occurrence ordinals and row order. This includes a supplied empty array;

    encoding it does not assert admission. The same optional field SHALL be

    retained by owned read, raw-term intake, lossless comparison and the borrowed

    canonical envelope. The encoder borr'
- id: spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md:addition-17
  path: spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md
  role: examined
  excerpt: 'An admitted package''s `schedulability_claim_rows` read supplies the borrowed

    rows for complete re-encoding. The exact-length and one-byte-short limits

    apply to the entire envelope including the index. Re-reading valid encoded

    bytes SHALL preserve the exact admitted claim body, identity and occurrence;

    invalid supplied rows remain subject to FR-038''s independent strict refusal.

    This extends the existing complete writer only and introduces no new envelope

    builder, evaluator or certificate member.'
- id: FR-051-AC-7
  path: spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md
  role: examined
  excerpt: planned until IR-1252 lands. An external producer encodes the complete
    SCC-01 and SCC-02 packages with `CheckedPackageSidecarsRef::schedulability_claims`;
    strict read and borrowed re-encode/read preserve the complete FR-482 body and
    every exact declaration-occurrence row, including both shared-node occurrences.
    The bytes equal the private envelope's canonical bytes at an exact measured byte
    limit; one byte less returns only the typed canonical limit error. `None` omits
    the index and `Some(empty)` encodes an empty array, with no writer normalization;
    malformed joins, missing coverage or reverse
- id: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:addition-19
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: "- target: ix://agent-ix/quire-specification/TC-976\n    type: references"
- id: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:addition-20
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: '## Authenticated schedulability claim carrier (FR-038-AC-396 through FR-038-AC-400)'
- id: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:addition-21
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: 'planned until IR-1252 lands. Read QSpec''s

    `proposals/checked-package-v2/schedulability-claim-vectors.json`, TC-976 and

    FR-482/TC-967 body cases directly from the selected checkout; state that

    checkout''s revision in execution evidence. Build complete IR-owned packages

    with the resolved graph, type/unit and claim-occurrence context those cases

    require, rather than submitting the identity-row vector as a package. Run

    SCC-01/02 through the public strict reader and inspect the exact borrowed

    identity, occurrence and typed body. Run every SCC schema and reader refusal

    against the corresponding compl'
- id: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:addition-22
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: 'Exercise the FR-482 positive schedulable/response forms, all ordered body

    checks and the published adverse bodies, including policy-specific task-set

    premises and the EDF equal-priority positive control. Combine defects to prove

    wire/graph/aggregate, carrier declaration, occurrence, duplicate, order,

    coverage and body precedence. Measure the existing validation budget charge;

    the exact positive budget admits, one less refuses at the first uncharged row

    without any admitted or partial claim view. The empty no-claim case omits the

    index and exposes an empty iterator. Confirm that the public read'
- id: spec/checked_package/matrix/TC-451-typed-v2-envelope-encoding.md:addition-23
  path: spec/checked_package/matrix/TC-451-typed-v2-envelope-encoding.md
  role: examined
  excerpt: '## Schedulability carrier round trip (FR-051-AC-7)'
- id: spec/checked_package/matrix/TC-451-typed-v2-envelope-encoding.md:addition-24
  path: spec/checked_package/matrix/TC-451-typed-v2-envelope-encoding.md
  role: examined
  excerpt: 'planned until IR-1252 lands. Read QSpec TC-976 and SCC-01/02 identities
    by

    reference and construct complete IR packages with valid FR-482 bodies. Encode

    using the optional borrowed schedulability rows, admit with the public strict

    reader, borrow the retained rows and re-encode/read. Compare all rows and exact

    body fields, including a shared node''s two occurrences. Assert exact-length

    success and one-byte-short typed canonical error without bytes. `None` omits

    the member and `Some(empty)` emits it empty; reversed or missing-coverage rows

    are not repaired by encoding and remain subject to strict'
- id: spec/model/functional/FR-019-rust-library-interface.md:addition-25
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: 'The authenticated schedulability carrier of FR-038-AC-396 through

    FR-038-AC-400 is planned until IR-1252 lands. The planned borrowed

    `CheckedSchedulabilityClaimRef<''a>` and the associated

    `CheckedPackageV2::schedulability_claims` read expose only admitted graph

    identity, claim occurrence, closed typed claim/task field reads and the

    existing typed `CheckedTerm` body, using

    the exact signatures FR-038 specifies. Fields remain private and no public

    constructor can mint the view. The closed `ClaimForm::Schedulability` variant

    joins that surface with the carrier implementation. These planned export'
- id: FR-019-AC-19
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: 'planned until IR-1252 lands. A default-feature external consumer imports
    `CheckedSchedulabilityClaimRef`, matches `ClaimForm::Schedulability`, and iterates
    an admitted package through `CheckedPackageV2::schedulability_claims`. Each view
    borrows `&CheckedNodeId`, `&CheckedOccurrence` with role `Claim`, and `&CheckedTerm`
    through the specified immutable accessors; the iterator reports the exact row
    count and preserves canonical order. The consumer exhaustively matches the specified
    closed kind/policy/arrival/criticality enums and reads task fields, optional response
    and HI-work values, optional '
```
