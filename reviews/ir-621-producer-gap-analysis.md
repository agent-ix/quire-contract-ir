---
id: SR-14531
title: gap-analysis review of IR-621 producer
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; IR-621 outcome constructors, external tests, status
  paragraphs
review_set: subset
---

## Summary

Ticket: IR-621. PR: agent-ix/quire-contract-ir#491. Frozen. No applicable AssuranceProfile exists. Reviewer ran no Cargo, fmt, metadata, native build/test, Kani, or aggregate: supplied focused raw logs and verified cgroup JSON were inspected independently. Exact deployment model SKU is unavailable; model marker uses gpt-6. Excerpts longer than 600 characters are verbatim prefixes. Full changed contract and files were read.

## Verdict

FAIL — whole-repository matrix retains 65 untagged criteria and one ignored-only criterion, individually recorded below. Scoped FR-030-AC-8/9 are backed by real external integration bindings and paired doctests; no new gap is introduced. All remaining missing bindings are inherited: normative criteria are unchanged in this PR and it adds three real tagged tests without removing any tag. A repository FAIL is not reported as a full PASS merely because the change is narrow.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-035-AC-9 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:183 |
| FND-002 | medium | FR-038-AC-132 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2488 |
| FND-003 | medium | FR-038-AC-135 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2647 |
| FND-004 | medium | FR-038-AC-232 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4871 |
| FND-005 | medium | FR-038-AC-233 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4872 |
| FND-006 | medium | FR-038-AC-234 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4873 |
| FND-007 | medium | FR-038-AC-235 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4874 |
| FND-008 | medium | FR-038-AC-294 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4880 |
| FND-009 | medium | FR-038-AC-295 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4882 |
| FND-010 | medium | FR-038-AC-281 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4916 |
| FND-011 | medium | FR-038-AC-282 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4917 |
| FND-012 | medium | FR-038-AC-283 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4918 |
| FND-013 | medium | FR-038-AC-284 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4919 |
| FND-014 | medium | FR-038-AC-285 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4920 |
| FND-015 | medium | FR-038-AC-286 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4921 |
| FND-016 | medium | FR-038-AC-287 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4922 |
| FND-017 | medium | FR-038-AC-288 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4923 |
| FND-018 | medium | FR-038-AC-289 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4924 |
| FND-019 | medium | FR-038-AC-290 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4925 |
| FND-020 | medium | FR-038-AC-291 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4926 |
| FND-021 | medium | FR-038-AC-292 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4927 |
| FND-022 | medium | NFR-006-M-1 has no criterion-tagged test in the computed matrix. | spec/checked_package/non-functional/NFR-006-v2-producer-memory.md:1 |
| FND-023 | medium | NFR-006-M-2 has no criterion-tagged test in the computed matrix. | spec/checked_package/non-functional/NFR-006-v2-producer-memory.md:1 |
| FND-024 | medium | NFR-006-M-3 has no criterion-tagged test in the computed matrix. | spec/checked_package/non-functional/NFR-006-v2-producer-memory.md:1 |
| FND-025 | medium | FR-045-AC-1 has no criterion-tagged test in the computed matrix. | spec/core/functional/FR-045-derive-proof-coverage-baseline.md:72 |
| FND-026 | medium | FR-045-AC-2 has no criterion-tagged test in the computed matrix. | spec/core/functional/FR-045-derive-proof-coverage-baseline.md:73 |
| FND-027 | medium | FR-045-AC-3 has no criterion-tagged test in the computed matrix. | spec/core/functional/FR-045-derive-proof-coverage-baseline.md:74 |
| FND-028 | medium | FR-045-AC-4 has no criterion-tagged test in the computed matrix. | spec/core/functional/FR-045-derive-proof-coverage-baseline.md:75 |
| FND-029 | medium | FR-045-AC-5 has no criterion-tagged test in the computed matrix. | spec/core/functional/FR-045-derive-proof-coverage-baseline.md:76 |
| FND-030 | medium | FR-045-AC-6 has no criterion-tagged test in the computed matrix. | spec/core/functional/FR-045-derive-proof-coverage-baseline.md:77 |
| FND-031 | medium | NFR-001-AC-2 has no criterion-tagged test in the computed matrix. | spec/core/non-functional/NFR-001-determinism.md:36 |
| FND-032 | medium | NFR-001-AC-3 has no criterion-tagged test in the computed matrix. | spec/core/non-functional/NFR-001-determinism.md:37 |
| FND-033 | medium | NFR-001-M-1 has no criterion-tagged test in the computed matrix. | spec/core/non-functional/NFR-001-determinism.md:1 |
| FND-034 | medium | NFR-001-M-2 has no criterion-tagged test in the computed matrix. | spec/core/non-functional/NFR-001-determinism.md:1 |
| FND-035 | medium | NFR-001-M-3 has no criterion-tagged test in the computed matrix. | spec/core/non-functional/NFR-001-determinism.md:1 |
| FND-036 | medium | NFR-002-M-1 has no criterion-tagged test in the computed matrix. | spec/core/non-functional/NFR-002-portability-independence.md:1 |
| FND-037 | medium | NFR-002-M-2 has no criterion-tagged test in the computed matrix. | spec/core/non-functional/NFR-002-portability-independence.md:1 |
| FND-038 | medium | NFR-003-M-1 has no criterion-tagged test in the computed matrix. | spec/core/non-functional/NFR-003-diagnostic-integrity.md:1 |
| FND-039 | medium | NFR-003-M-2 has no criterion-tagged test in the computed matrix. | spec/core/non-functional/NFR-003-diagnostic-integrity.md:1 |
| FND-040 | medium | NFR-003-M-3 has no criterion-tagged test in the computed matrix. | spec/core/non-functional/NFR-003-diagnostic-integrity.md:1 |
| FND-041 | medium | NFR-003-M-4 has no criterion-tagged test in the computed matrix. | spec/core/non-functional/NFR-003-diagnostic-integrity.md:1 |
| FND-042 | medium | FR-030-AC-4 has no criterion-tagged test in the computed matrix. | spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md:157 |
| FND-043 | medium | FR-030-AC-5 has no criterion-tagged test in the computed matrix. | spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md:158 |
| FND-044 | medium | FR-030-AC-7 has no criterion-tagged test in the computed matrix. | spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md:160 |
| FND-045 | medium | FR-036-AC-1 has no criterion-tagged test in the computed matrix. | spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md:58 |
| FND-046 | medium | FR-036-AC-2 has no criterion-tagged test in the computed matrix. | spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md:59 |
| FND-047 | medium | FR-036-AC-3 has no criterion-tagged test in the computed matrix. | spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md:60 |
| FND-048 | medium | FR-036-AC-4 has no criterion-tagged test in the computed matrix. | spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md:61 |
| FND-049 | medium | FR-036-AC-5 has no criterion-tagged test in the computed matrix. | spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md:62 |
| FND-050 | medium | FR-037-AC-6 has no criterion-tagged test in the computed matrix. | spec/kani/functional/FR-037-canonical-backend-replay-and-qualification.md:66 |
| FND-051 | medium | FR-039-AC-1 has no criterion-tagged test in the computed matrix. | spec/kani/functional/FR-039-root-crate-public-interface.md:113 |
| FND-052 | medium | FR-039-AC-2 has no criterion-tagged test in the computed matrix. | spec/kani/functional/FR-039-root-crate-public-interface.md:114 |
| FND-053 | medium | FR-039-AC-3 has no criterion-tagged test in the computed matrix. | spec/kani/functional/FR-039-root-crate-public-interface.md:115 |
| FND-054 | medium | FR-039-AC-4 has no criterion-tagged test in the computed matrix. | spec/kani/functional/FR-039-root-crate-public-interface.md:116 |
| FND-055 | medium | FR-016-AC-1 has no criterion-tagged test in the computed matrix. | spec/model/functional/FR-016-canonicalization-digests.md:148 |
| FND-056 | medium | FR-016-AC-3 has no criterion-tagged test in the computed matrix. | spec/model/functional/FR-016-canonicalization-digests.md:150 |
| FND-057 | medium | FR-017-AC-2 has no criterion-tagged test in the computed matrix. | spec/model/functional/FR-017-version-orphan-coverage.md:97 |
| FND-058 | medium | FR-019-AC-4 has no criterion-tagged test in the computed matrix. | spec/model/functional/FR-019-rust-library-interface.md:219 |
| FND-059 | medium | FR-028-AC-4 is tagged only by an ignored test; ordinary execution coverage is absent. | spec/model/functional/FR-028-separate-cycle-free-contract-model.md:63 |
| FND-060 | medium | FR-052-AC-7 has no criterion-tagged test in the computed matrix. | spec/model/functional/FR-052-source-obligation-correspondence.md:238 |
| FND-061 | medium | FR-052-AC-9 has no criterion-tagged test in the computed matrix. | spec/model/functional/FR-052-source-obligation-correspondence.md:259 |
| FND-062 | medium | FR-052-AC-10 has no criterion-tagged test in the computed matrix. | spec/model/functional/FR-052-source-obligation-correspondence.md:473 |
| FND-063 | medium | FR-052-AC-11 has no criterion-tagged test in the computed matrix. | spec/model/functional/FR-052-source-obligation-correspondence.md:474 |
| FND-064 | medium | FR-052-AC-16 has no criterion-tagged test in the computed matrix. | spec/model/functional/FR-052-source-obligation-correspondence.md:235 |
| FND-065 | medium | FR-052-AC-26 has no criterion-tagged test in the computed matrix. | spec/model/functional/FR-052-source-obligation-correspondence.md:489 |
| FND-066 | medium | FR-052-AC-27 has no criterion-tagged test in the computed matrix. | spec/model/functional/FR-052-source-obligation-correspondence.md:490 |

## Coverage

Recorded evidence: fmt/check, locked workspace/all-target Clippy and separate default-feature model Clippy pass; 3 actual external native tests, 12 shared native controls, and 32 outcome doctests (18 new compile-fail + 9 passing + 5 inherited) pass. Actual scopes enforce memory.max 17179869184 and swap.max 0 before Cargo, with peak 2675638272 bytes and zero memory events. These are producer/native/signature results, not Kani proof or live cancellation evidence. Final aggregate remains the code lead’s gate.

Plan completion: not assessed

Computed census: 589 criteria, 512 tagged, 65 untagged, one tagged-by-ignored-test, 11 method-without-symbol (method records are not silently classified as missing tests). Existing general debt handoff IR-878 may own inherited gaps only after the code lead posts the concrete owner handoff and this same reviewer records dispositions. No deferral is presumed here. Separate optional semantic expansion was not selected.

Reverse inventory: 14 source behavior areas checked against owning requirements: binding/expression/wire, schema/limits, decimal, canonical identity, conformance CLI, coverage/output mapping, source obligations, selected population, checked-package admission/strict readers, V2 lowering, profile negotiation, finite ABI/dispatch, arithmetic/collection/graph lowering, and outcomes. These map to the existing model/core/checked-package/Kani requirement families FR-001..FR-052. All nine added public operations are owned explicitly by FR-030/FR-039/FR-044 and TC-457; zero new unowned behaviors, zero confirmed production or test stubs. Whole-tree TODO/unimplemented scans only locate intentional compile-fail documentation placeholders in model V2 API examples. Intentional model root reexports are the documented cycle-free substrate, not hollow implementations. This is a source/trace review, not an execution of all repository tests.

## Examined Scope

```yaml
scope:
- id: FR-030-AC-3
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: No invalid, refused, incomplete, unavailable, timed-out, exhausted, cancelled,
    or inconclusive outcome carries Boolean success; earliest failures emit no partial
    artifact.
- id: FR-030-AC-4
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: A `proved` outcome built from a run with three SUCCESS checks carries the
    count three; a run with zero SUCCESS checks yields `inconclusive` with cause `kani_vacuous_proof`,
    carries no count and no Boolean claim; and a request for a `proved` outcome with
    a count of zero returns `KaniOutcomeError` with code `kani_outcome_invalid` and
    no outcome.
- id: FR-030-AC-5
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: An `unavailable` outcome built for an absent checker carries cause `kani_checker_absent`
    and one built for an absent backend carries `kani_backend_absent`, each with no
    Boolean claim; a request for an `unavailable` outcome with any other cause code,
    an `inconclusive` outcome with a cause other than `kani_vacuous_proof`, or a `proved`
    or `counterexample` outcome through the non-success constructor returns `KaniOutcomeError`
    with code `kani_outcome_invalid` and no outcome; and a struct-literal `KaniOutcome`
    outside the `kani` module fails to compile.
- id: FR-030-AC-6
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: The `code` of the outcome built for a zero-check proof equals `Std001Code::KANI_VACUOUS_PROOF`
    and its serialization holds the member `"code":"kani_vacuous_proof"`; the `code`
    of the `KaniOutcomeError` for a `proved` outcome with a count of zero equals `Std001Code::KANI_OUTCOME_INVALID`;
    a call that passes a `&str` or a `String` as the code of the non-success constructor
    fails to compile; and a serialized `KaniOutcome` whose `code` is `"Bad-Code"`
    fails to deserialize.
- id: FR-030-AC-7
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: 'The IR outcome constructor accepts the observed caller-request or timer-owner
    deadline source as `Cancelled { source: Requested }` or `Cancelled { source: Deadline
    }`; each serializes and deserializes with that exact `kind` source and no Boolean
    claim. Construction without a source fails to compile. Deserialization refuses
    a unit `"cancelled"` kind, a missing or unknown source, and an origin on a non-cancelled
    kind. Every existing non-cancelled kind retains its former wire spelling. The
    future CG Kani producer reads the event that actually stopped the run; an execution
    timeout without that eve'
- id: FR-030-AC-8
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: An external consumer calls each of the nine public infallible methods with
    an explicit `KaniOutcome` return type. The six coded methods retain a consumer-owned
    unregistered `Std001Code`; `checker_unavailable`, `backend_unavailable` and `vacuous_proof`
    carry their exact fixed causes. Every outcome retains distinct supplied source/context
    strings, has the specified kind and no Boolean claim, and survives a serialization/deserialization
    round trip with exactly the same kind, code, source and context. Both cancellation
    sources remain distinct in the wire form.
- id: FR-030-AC-9
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: External compile-fail probes cannot pass `KaniOutcomeKind::Proved` or `Counterexample`
    as a kind to the infallible route, cannot supply a cause argument to its three
    fixed-cause methods, cannot omit the cancellation source, and cannot pass a `&str`
    or `String` as a coded method's cause. Each probe is paired with a passing public
    call of the intended signature. Existing fallible `non_success` requests for `Proved`
    and `Counterexample` still return `KaniOutcomeError` naming the exact requested
    kind and cause and build no outcome.
- id: FR-035-AC-9
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: 'The typed profile exposes `required_source_selections: Vec<CheckedSelection>`
    and the IR adapter conforms to QSpec FR-330-AC-10 and TC-935: input refusals use
    `InvalidInput { node_id }`/`invalid_input`, work stops use existing `Failed`/`Work`
    with unchanged key/limit/consumed counters, and successful checks continue the
    existing FR-038 meter without reset. The adapter preserves exact source provenance
    and emits only `profile.reference`, with no duplicated requirement wire field
    or new refusal payload.'
- id: FR-038-AC-132
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: No copy (IR-627 checkout-reference harness implemented; IR-630 extended
    conformance PLANNED and UNRUN). Authoritative vectors for owner-bearing, declared
    and labelled-group forms remain at their owning QSL or QSpec checkout and are
    read there, not copied into this repository. A conformance read fails closed when
    its source is absent or malformed, as FR-038-AC-112 and AC-176 require. Test-side
    derivation controls do not replace published goldens.
- id: FR-038-AC-135
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Adverse mutations still reach their stage (IR-627; planned, ungated; supersedes
    AC-112''s base package when the code lands). With `positive-all-families.json`
    carrying derived keys, each `structural_mutations` and `body_grammar_mutations`
    entry of `adverse.json` refuses with exactly its recorded `outcome`, among them
    `wrong-node-family-form` as `invalid_semantic_graph`, `negative-temporal-interval-bound`
    as `invalid_package` and the five `malformed_wire` body-grammar entries (eleven
    entries in all: six `structural_mutations`, five `body_grammar_mutations`), and
    none is refused `stale-node-key`;'
- id: FR-038-AC-232
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'PLANNED/UNRUN (IR-691). A fresh QSL FR-093 checked and emitted `record
    Point { x: Int[0,9]; }` value `Point { x: 3 }` is admitted by the public V2 reader;
    its `record_value` binding references a `numeric.narrow` conversion whose sole
    argument references the canonical Integer literal `3`, and whose member declaration,
    result type and semantic type name the same `Int[0,9]` range. In an admitted structural.eq
    or structural.ne over that value, `composite_application_operands` returns the
    original record GraphChild and semantic type with Literal domain and no introduced
    positions. At the private po'
- id: FR-038-AC-233
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-691). In crate-internal admitted-then-mutated controls,
    altering only the narrow node's operator, laws, leaves, mode, member kind/declaration,
    result type, semantic type or argument count/term; the range's presence, Integer
    base, endpoint binding/literal shape, endpoint literal type, endpoint grammar/order;
    or the operand literal's value kind, declared/semantic type or canonical spelling
    returns `MalformedDomain` with the enclosing ordinal and first valid type locus
    specified above. For both positive and negative ranges beyond i128, a canonical
    literal one below min or one ab
- id: FR-038-AC-234
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-691). A narrow whose argument reaches a parameter, another
    application including a second narrow, or a value cycle returns `UnsupportedOperand`/`NonliteralGraphValue`
    at the first offending node with the enclosing ordinal; an unrelated application
    remains refused. A direct parameter operand retains its Parameter projection.
    Existing closed tuple, Option, collection and enum-value descendants remain accepted
    by their value-form paths, and a narrow integer field inside a supported composite
    receives only the stated narrow exception. Reader admission and first-refusal
    outcomes f
- id: FR-038-AC-235
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'PLANNED/UNRUN (IR-691). Independently count the exact logical graph, term/member,
    examined-endpoint-byte and examined-literal-byte charges for a small admitted
    narrow-record fixture. Its measured successful work limit succeeds; one less and
    zero return `WorkLimit` with the first attempted cumulative charge and no partial
    operands. At the private seam, a range with long canonical endpoints charges every
    endpoint byte once per logical visit before comparison even when its first sign
    or digit decides the order: the independently counted exact limit succeeds, and
    one less returns `WorkLimit`. A bu'
- id: FR-038-AC-294
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-785). Starting with an otherwise admitted public-reader
    package containing one local protocol target and one carrier with `expression`
    ordinal 0, add a second source reference to that target as `expression` ordinal
    1 and its own exact source-map row. Both packages admit through `CheckedPackageV2::read`,
    retain exactly one carrier, and have equal independently recomputed carrier `node_id`
    and `package_id`; the admitted carrier retains both distinct ordinals and both
    source correspondences. A third occurrence with a further distinct ordinal and
    source-map row also admits with t
- id: FR-038-AC-295
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-785). From the admitting repeated-reference package,
    independently duplicate an occurrence ordinal or a source-map key while retaining
    all otherwise valid identities. The public reader refuses `invalid_source_map`
    respectively at the later `/semantic_graph/nodes/{n}/occurrences/{j}` or `/source_map/{i}`
    pointer. An empty carrier occurrence array refuses `invalid_source_map` at `/semantic_graph/nodes/{n}/occurrences`;
    changing either occurrence role away from `expression` refuses `invalid_semantic_graph`
    at that occurrence's `/role` pointer before target resolution. Adding a s
- id: FR-038-AC-281
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A reader-admitted protocol member has a `quire.op.protocol.control` body
    root, exactly one `protocol_profile` law matching the selected lock row, `null`
    mode, no leaves, Boolean result and semantic types, and a `profile_operator` member.
    Under the published `quire.protocol.finite-global/v1` selection the admitted operators
    are exactly `protocol`, `role`, `channel`, `queue`, `finish`, `terminal`, `sequence`,
    `choice`, `parallel`, `repeat`, `await`, `check`, `send`, `receive`, `attempt`,
    `effect`, `event`, `commit`, `fence` and `compensate`; an unknown operator refuses
    `operation-member-mismatch
- id: FR-038-AC-282
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Each of the seven protocol member forms admits only the FR-464, FR-465
    or FR-466 fixed argument names, order and term shapes for its selected operator:
    thirteen clause bindings, the role, channel, queue, finish and terminal layouts,
    thirteen control layouts and thirteen compensation bindings. A missing, extra,
    reordered, wrongly shaped or misspelled binding refuses `malformed_wire`; an optional
    Group admits zero or one member only, an interval has non-negative integer bounds
    with lower no greater than upper, and a set or sorted-record list repeated or
    out of its specified ascending order refus'
- id: FR-038-AC-283
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A protocol node derives its `quire.application-node/v1` key from its exact
    tag, form, Boolean type, declaration path, `null` recursion and body, with occurrences
    excluded; a second reference, source reformatting or declaring roles and other
    set-backed members in another source order preserves the key and package id, while
    a changed name, referenced target, literal, ordered capture or `sequence` child
    changes both. Role, channel, finish, terminal, control and compensation declarations
    use the fixed qualified-name segments of QSpec FR-464; queues have no declaration
    and generated occurrences, wh
- id: FR-038-AC-284
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The reader runs a protocol step after state, temporal and abstraction
    and before operations. That step reports the first defect by QSpec FR-464 order:
    placement of applications and member references, operator and law, declarations,
    member nodes in role/channel/queue/obligation/control/compensation order, membership,
    then clauses; nodes within a stage are visited by ascending node-id digest. A
    package with both a state-step and protocol defect reports the state defect; one
    with both a protocol and operation defect reports the protocol defect. A nested
    protocol application refuses `malformed_wir'
- id: FR-038-AC-285
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Every role, channel, queue, obligation and compensation is listed by exactly
    one owning protocol clause, and every control is held once in its root control
    tree with the declaration path of its holder followed by its own name. A member
    listed twice or unattached, a control held twice or by nothing, a channel without
    exactly one queue, or a queue attached to another clause refuses `invalid_semantic_graph`
    at that member. A target not listed in its referencing node's `dependencies` refuses
    `missing_declaration`/`missing-name`; a target of the wrong tag or form refuses
    `invalid_model_binding`/`ma
- id: FR-038-AC-286
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A role admits the FR-465 static and replicated shapes only, including a
    model of an eligible form, replication maximum at least one, a reference-typed
    binder and an `until` term exactly for an `until` lifetime; a channel admits only
    same-clause role endpoints, an eligible message type, explicit delivery and ordering,
    and a FIFO key with equality-stable type; a queue admits one channel, literal
    or symbolic capacity, explicit overflow and a loss selector exactly for `loss`.
    The reader admits the positive variants and refuses the wrong form, missing premise,
    wrong binder type, non-Boolean lifetim
- id: FR-038-AC-287
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A clause names exactly one `finish` obligation whose closing-record binder
    and Boolean constraint read only the binders available after the root control;
    its optional `terminal` obligation admits `when` with one Boolean condition or
    `any` with none. A missing or extra condition refuses `malformed_wire`, a non-Boolean
    condition or constraint refuses `ill_typed`/`non-boolean-root`, an unjoined branch
    binder read by finish refuses `missing_declaration`/`missing-name`, and a terminal
    target in the finish position or an unattached finish refuses by the FR-464 membership
    rules.
- id: FR-038-AC-288
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Each of the thirteen control operators admits its FR-466 ordered arguments
    and local reference, type, bounds, label and ordering checks: `sequence`, `choice`,
    `parallel`, `repeat`, `await`, `check`, `send`, `receive`, `attempt`, `effect`,
    `event`, `commit` and `fence`. A choice or parallel branch list with fewer than
    two entries, repeated labels, a receive naming a non-send, an effect naming a
    non-attempt, an await naming an ineligible event, a non-Boolean guard or constraint,
    an invalid join count or an ineligible memory or ordering spelling refuses with
    the FR-466 code, cause and locus, with'
- id: FR-038-AC-289
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `compensate` body admits the thirteen FR-466 bindings in order, including
    distinct registration and activation captures, a trigger, interval, at-least-one
    attempt, retry relation, optional commit and recovery predicate. `commit never`
    is an absent optional Group, not a fabricated commit. The reader refuses a forward
    target that is not an `effect`, a commit target that is not a `commit`, an inverted
    interval or zero attempts, a commit that dominates its forward effect, and a read
    outside any of the five compensation stages with the corresponding FR-466 code,
    cause and locus; a commit in one c
- id: FR-038-AC-290
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: For each clause the reader constructs the QSpec FR-464 derived causal relation
    from control containment, sequencing, receive/send, effect/attempt and await edges,
    not source order or timestamps. A receive before its send or an await anchored
    at a later control refuses `invalid_package`/`definition-cycle` naming the cycle;
    a same pair on independent parallel branches admits. A repeat whose body can complete
    with no progress on a path refuses `unsupported_construct`/`declaration-form`,
    while a repeat with a real step admits with or without a maximum and loop proof.
- id: FR-038-AC-291
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Binder levels start at the introducing construct's entry count; protocol
    `over` is level 0, an `each` trigger follows, captures advance in their authored
    order, and a control binder is at the size of its readable input set. The reader
    admits reads within the FR-464/466 stage tables and refuses a wrong level, duplicate
    readable name, forward or foreign binder, or read after a branch, choice, repeat
    or await path where the binder is unavailable with the specified code, cause,
    stage and owner. In a binders-only slot, a read of `over` or model state refuses
    `ill_typed`/`operator-ineligible` before
- id: FR-038-AC-292
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: At a `choice` or `repeat` decision the reader admits only QSpec FR-466
    visible facts owned by the deciding role and listed individually in `visible`;
    a fact from another role, an ineligible send/effect/commit/finish or protocol-input
    binder, a fact reachable only inside a composite entry, or a readable fact omitted
    from `visible` refuses `wrong_snapshot`/`wrong-observation` with the specified
    reason. A binder unavailable at that decision refuses `missing_declaration`/`missing-name`
    first.
- id: NFR-006-M-1
  path: spec/checked_package/non-functional/NFR-006-v2-producer-memory.md
  role: examined
  excerpt: Peak incremental resident memory in the TC-455 emission window
- id: NFR-006-M-2
  path: spec/checked_package/non-functional/NFR-006-v2-producer-memory.md
  role: examined
  excerpt: Concurrent extra owned copies of each projected graph-node body on the
    successful producer path
- id: NFR-006-M-3
  path: spec/checked_package/non-functional/NFR-006-v2-producer-memory.md
  role: examined
  excerpt: Canonical package bytes and `package_id` for the same checked input
- id: FR-045-AC-1
  path: spec/core/functional/FR-045-derive-proof-coverage-baseline.md
  role: examined
  excerpt: From a fixture whose inventories contain two criteria, three obligations
    and two production functions in one module, the baseline emits every source-semantic
    candidate and explicit mappings or unmapped reasons before joining results; removing
    a result or adding/changing a method, proof subject or backend/profile attempt
    changes evidence or gaps but not any eligible denominator.
- id: FR-045-AC-2
  path: spec/core/functional/FR-045-derive-proof-coverage-baseline.md
  role: examined
  excerpt: A proved production Kani result credits only its exactly matched claim
    and declared finite domain when a load-bearing check and production reachability
    are evidenced; a zero-check result, tag-only record, LLVM probe-only record, duplicate
    result, or mismatched source/profile/subject/domain receives no proof credit and
    has a stated reason.
- id: FR-045-AC-3
  path: spec/core/functional/FR-045-derive-proof-coverage-baseline.md
  role: examined
  excerpt: An effective bound narrower than declared is reported only for that narrower
    domain; a shadow proof without refinement is a production-proof gap, and a shadow
    proof with separately executed sampled native refinement is labeled conditional
    and sampled, never unqualified production-proof credit.
- id: FR-045-AC-4
  path: spec/core/functional/FR-045-derive-proof-coverage-baseline.md
  role: examined
  excerpt: Requirement, module, function and obligation reports have separately derived
    eligible denominators and qualified numerators; an eligible entity with zero eligible
    mapped obligations remains a visible mapping gap, one with one of two eligible
    mapped obligations proved remains a partial gap, and no cross-level or cross-method
    blended percentage is emitted.
- id: FR-045-AC-5
  path: spec/core/functional/FR-045-derive-proof-coverage-baseline.md
  role: examined
  excerpt: An FR-029 `unsupported` negotiation disposition remains visible with its
    capability and reason and requires no Kani outcome or run; pre-execution and Kani
    refusals retain producer and stage. Missing, counterexample, invalid/incomplete
    input, unavailable, timed out, resource exhausted, cancelled, inconclusive and
    unfamiliar producer states remain visible with reasons; an ineligible candidate
    remains visible with its policy reason and is excluded only from its applicable
    denominator.
- id: FR-045-AC-6
  path: spec/core/functional/FR-045-derive-proof-coverage-baseline.md
  role: examined
  excerpt: Repeating a baseline on unchanged authoritative inventories, policy and
    results produces the same rows and totals; changing an inventory updates candidates
    without editing a separate list of file hashes, SHAs, tool pins or snapshots.
- id: NFR-001-AC-2
  path: spec/core/non-functional/NFR-001-determinism.md
  role: examined
  excerpt: For each fixed admitted corpus input and supported profile that yields
    canonical output, Linux, macOS, and Windows produce byte-identical canonical bytes
    when cross-platform CI is enabled; compare those bytes directly.
- id: NFR-001-AC-3
  path: spec/core/non-functional/NFR-001-determinism.md
  role: examined
  excerpt: For each fixed rejected corpus input and profile, repeated runs and supported
    operating systems produce exactly the same complete ordered sequence of diagnostic
    `(code, path)` tuples.
- id: NFR-001-M-1
  path: spec/core/non-functional/NFR-001-determinism.md
  role: examined
  excerpt: Cross-run byte equality
- id: NFR-001-M-2
  path: spec/core/non-functional/NFR-001-determinism.md
  role: examined
  excerpt: Cross-platform canonical byte equality
- id: NFR-001-M-3
  path: spec/core/non-functional/NFR-001-determinism.md
  role: examined
  excerpt: Ordered diagnostic code/path tuple equality
- id: NFR-002-M-1
  path: spec/core/non-functional/NFR-002-portability-independence.md
  role: examined
  excerpt: Forbidden public vocabulary
- id: NFR-002-M-2
  path: spec/core/non-functional/NFR-002-portability-independence.md
  role: examined
  excerpt: Target-dependent canonical fields
- id: NFR-003-M-1
  path: spec/core/non-functional/NFR-003-diagnostic-integrity.md
  role: examined
  excerpt: Public panic paths for untrusted input
- id: NFR-003-M-2
  path: spec/core/non-functional/NFR-003-diagnostic-integrity.md
  role: examined
  excerpt: Failure classes with stable code
- id: NFR-003-M-3
  path: spec/core/non-functional/NFR-003-diagnostic-integrity.md
  role: examined
  excerpt: Coverage diagnostic locus accuracy
- id: NFR-003-M-4
  path: spec/core/non-functional/NFR-003-diagnostic-integrity.md
  role: examined
  excerpt: Orphans counted as covered
- id: FR-036-AC-1
  path: spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
  role: examined
  excerpt: Generated symbolic ranges equal model-derived inclusive domains, including
    boundary values.
- id: FR-036-AC-2
  path: spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
  role: examined
  excerpt: Missing bounds, absent exact encodings, and unsupported meaning produce
    a per-item non-success and no artifact.
- id: FR-036-AC-3
  path: spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
  role: examined
  excerpt: Every generated item keeps its precondition, postcondition, invariant and
    frame clauses separate.
- id: FR-036-AC-4
  path: spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
  role: examined
  excerpt: A provider completes accounting for every requested item before exposing
    generated bytes or a result.
- id: FR-036-AC-5
  path: spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
  role: examined
  excerpt: A requested claim for which the provider manifest advertises no matching
    FR-290 capability settles `unsupported` and its record carries a warning naming
    the required capability.
- id: FR-037-AC-6
  path: spec/kani/functional/FR-037-canonical-backend-replay-and-qualification.md
  role: examined
  excerpt: Contract IR's public API names no replay envelope, replay request, replay
    result, witness, parity verdict or minimization type other than QSL's, the root
    crate has no `replay` or `witness` module, and no Contract IR source calls a replay
    executor.
- id: FR-039-AC-1
  path: spec/kani/functional/FR-039-root-crate-public-interface.md
  role: examined
  excerpt: The root crate's public items are exactly those the Public items table
    lists, checked by a public-signature inventory of `src/` that fails on an added
    or missing item; `src/lib.rs` re-exports no `quire_contract_model` item, and code
    naming a model item through `quire_contract_ir` fails to compile.
- id: FR-039-AC-2
  path: spec/kani/functional/FR-039-root-crate-public-interface.md
  role: examined
  excerpt: No public item names a `quire_spec_language::runtime` type and no source
    file under `src/` names `quire_spec_language::runtime`.
- id: FR-039-AC-3
  path: spec/kani/functional/FR-039-root-crate-public-interface.md
  role: examined
  excerpt: Code naming any item the "Items QSL owns" or "Items codegen owns" section
    lists through `quire_contract_ir` fails to compile, and no public item of the
    root crate names `qsl_replay::TerminalValue` or maps a `KaniOutcome` to one.
- id: FR-039-AC-4
  path: spec/kani/functional/FR-039-root-crate-public-interface.md
  role: examined
  excerpt: Every public `kani` function over untrusted input returns a typed error
    with no public panic path under the negative corpora of TC-041 and TC-042.
- id: FR-016-AC-1
  path: spec/model/functional/FR-016-canonicalization-digests.md
  role: examined
  excerpt: Exact golden fixtures, regenerated from `quire-canonical`'s own output,
    pin the profile envelope, escaping, the string spelling of the eight integer members
    (FR-016-AC-5) and the number spelling of the others, normalized rationals, semantic-set
    ordering, sequence preservation, source exclusion, and SHA-256 digest for every
    closed object kind; equivalent supported permutations are byte/digest identical.
- id: FR-016-AC-3
  path: spec/model/functional/FR-016-canonicalization-digests.md
  role: examined
  excerpt: A deterministic reservation-failure harness forces canonical byte allocation
    failure and verifies `canonicalization_resource_exhausted`, no partial public
    bytes, and no digest.
- id: FR-017-AC-2
  path: spec/model/functional/FR-017-version-orphan-coverage.md
  role: examined
  excerpt: Shallow, deep, uncovered, and each closed orphan reason have positive/negative
    fixtures; stale revision, missing, cross-package, duplicate, and over-limit inputs
    retain distinct diagnostics and cannot make a current requirement appear covered.
    Deep traces name the current requirement revision and carry no digest.
- id: FR-019-AC-4
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: 'A default-feature build of `quire-contract-model` exports neither `MappingAllocationPoint`
    nor `MappingExecutionControl::fail_allocation_at`: code naming either fails to
    compile without the `fault-injection` feature.'
- id: FR-028-AC-4
  path: spec/model/functional/FR-028-separate-cycle-free-contract-model.md
  role: examined
  excerpt: Default, all-feature and minimum-version builds prove that no optional,
    dev or historical dependency leaks into the production graph.
- id: FR-052-AC-7
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Complete Quire scope covers every authored acceptance criterion in the
    bound selected corpus and declared obligation sources, including unbound criteria;
    complete IR V2 scope covers every semantic node and source occurrence in the selected
    V2 graph; complete QSL scope covers every checked `RequirementRecord` and native
    pending obligation for the selected request, identified by its collision-free
    native occurrence key or site/kind tuple rather than an IR requirement reference
    or semantic digest; complete `quire-code-rs` scope covers every selected production
    module and every function fact, incl
- id: FR-052-AC-9
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: For each of the four owners, the native enumerator or typed adapter issues
    a `Complete` outcome only after the selected named scope reaches its normal end;
    the outcome retains the selected source, typed owner and scope, every owner-emitted
    row and the observed count. For Quire this requires PLAT-1165's selected-scope
    producer, not an IR traversal of `obligation::derive` output. A complete zero-row
    scope and a fully parsed module with zero functions remain complete, while an
    absent owner report remains absent. Deleting one row after the owner's full output
    is observed makes the owner-output com
- id: FR-052-AC-10
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: For each owner, stopping before normal exhaustion issues `Incomplete` with
    retained partial rows and the typed cause the owner actually reports. Quire discovery/load/scan/skipped-row
    failures retain their distinct owner-native causes once PLAT-1165 supplies them;
    current `obligation::derive` cannot issue `Complete`. QSL type-work exhaustion
    retains dimension, limit, prior, requested charge and site; QSL `TypeDisposition::Unfinished`
    retains its declaration handle and is distinct. An IR V2 reader limit retains
    the `CheckedPackageIncomplete` fields. For an owner path that reports `quire_exact::C
- id: FR-052-AC-11
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: The neutral model retains an owner-issued outcome only when its source,
    owner, scope and observed count match the retained rows; a mismatch is typed and
    refuses the outcome. Owner adapter tests show that no adapter completion branch
    accepts an arbitrary caller-supplied row vector/count/completion flag in place
    of enumerating the selected native owner output. The model does not claim independent
    authentication of external owner exhaustion or a completed four-owner census.
- id: FR-052-AC-16
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: With PLAT-1165's real Quire producer, normal exhaustion of the validated
    selected scope issues `Complete` with every authored criterion and its provenance;
    a valid empty scope completes with zero rows. Changing or deleting one authored
    criterion in the selected Quire corpus changes the owner-issued inventory before
    generation or proof results, including when that criterion has zero mappings.
    An unreadable selected document, skipped criterion row, failed discovery, missing
    declared table/ID column, and early stop each report typed `Incomplete` with emitted
    partial rows, and deletion of a row fr
- id: FR-052-AC-26
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Given a selected QSL checked request, the QSL-owned adapter enumerates
    every `CheckedGraph::requirements()` key and every native pending obligation,
    preserving each requirement's wire node bytes, occurrence role and ordinal and
    each obligation's declaration, unit, expression, byte span, kind and predicate
    target. Its `Complete` outcome matches an independent full-output traversal only
    after normal exhaustion, including a valid empty scope; an omitted row fails comparison
    even with an adjusted count. Native type-work exhaustion instead issues `Incomplete`
    with dimension, limit, prior, requested
- id: FR-052-AC-27
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Given a `quire-code-rs` selected production module, the owner adapter retains
    that module and every function fact from its complete native parse, including
    a parsed module with zero functions, and issues `Complete` only after the selected
    module's normal terminal state. Its source, owner, scope, rows and count equal
    independent full output; deleting one emitted row fails comparison even with an
    adjusted count. A missing selected module or failed parse instead issues `Incomplete`
    with the native module or parse cause and retained partial rows; no completion
    path accepts caller-authored rows, co
bindings:
- test_id: coded_constructors_retain_consumer_causes_and_exact_context
  ac_id: FR-030-AC-8
  trace: correct
- test_id: fixed_cause_constructors_retain_the_registered_cause_and_context
  ac_id: FR-030-AC-8
  trace: correct
- test_id: broad_fallible_constructor_still_rejects_both_boolean_kinds
  ac_id: FR-030-AC-9
  trace: correct
```

## Dispositions

Round 1: all 66 finding identities and statuses were matched against the actual IR-878 comment read-back and its exact external source. Repository gap verdict remains FAIL. No new local defect was found.

Baseline qualification: inheritance was inferred from the normative/spec and trace diffs, not from executing a parent matrix or strict-coverage command. The candidate matrix was executed; a separate actual parent strict-coverage comparison remains the lead’s verification when needed.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-002 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-003 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-004 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-005 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-006 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-007 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-008 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-009 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-010 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-011 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-012 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-013 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-014 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-015 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-016 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-017 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-018 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-019 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-020 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-021 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-022 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-023 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-024 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-025 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-026 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-027 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-028 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-029 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-030 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-031 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-032 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-033 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-034 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-035 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-036 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-037 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-038 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-039 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-040 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-041 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-042 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-043 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-044 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-045 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-046 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-047 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-048 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-049 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-050 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-051 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-052 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-053 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-054 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-055 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-056 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-057 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-058 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-059 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-060 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-061 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-062 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-063 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-064 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-065 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
| FND-066 | deferred | Deferred to existing IR-878 under explicit owner handoff deeae13f-debf-4f9c-b37c-10c8af97fa3a: luna:IR:plan handles non-Kani triage; Kani rows stay in titan:kani:plan assigned queue. Inherited coverage remains unresolved; no test or proof execution is implied. |
