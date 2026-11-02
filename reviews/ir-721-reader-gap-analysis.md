---
id: SR-1288
title: IR-721 reader gap-analysis
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/model/functional/FR-019-rust-library-interface.md
review_set: subset
---

## Summary

Ticket: IR-721. No introduced reader coverage gap found. Whole-repository static traceability remains FAIL from 130 untagged and one ignored-only baseline obligation; this is not a whole-repository PASS.

## Verdict

**FAIL** — No introduced reader coverage gap found. Whole-repository static traceability remains FAIL from 130 untagged and one ignored-only baseline obligation; this is not a whole-repository PASS.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Reconciliation: quire matrix (static axis); no run evidence store read. Whole repository: 957 obligations, 804 tagged, 130 untagged, 1 tagged-by-ignored-test, 22 method-without-symbol. All eleven activated monitor/interface ACs are tagged. The baseline-gap-check.json records each inherited gap: all statements exist on the base, including twelve engine-generated metric IDs whose rows do not spell their IDs literally. Diff inspection removes no existing tests or trace attributes. These baseline gaps are context under the lead's explicit scope ruling, not introduced same-PR findings. The full list is reproduced below.

Coverage diagnostics include absent inspection/interface/suite archetypes, a missing NFR criterion section, existing nonbinding tags and uncatalogued methods. Coverage reports 116 untracked symbols; none belongs to the new monitor files. No new owning-requirement gap or stub was found in the changed Rust surface, which maps to FR-038-AC-421..430 and FR-019-AC-23. Reverse-gap/source semantic inspection is limited to the changed reader surface; no end-to-end clean architecture claim is made about unrelated production code.

Plan completion: not assessed

Optional whole-repository semantic gap analysis: skipped; no separate opt-in was requested. Code-review still checks changed code/test intent. Producer/lowering and runtime differential qualification remain open, as specified.

The immutable candidate was verified against local HEAD and GitHub PR head. Source review is read-only; artifacts are outside the branch. No applicable AssuranceProfile exists in spec/. QSpec FR-487/FR-322 define the wire; IR implements reader admission only. Root and all four entry structural keys bind the full program content; downstream consumers must independently bind their selected package/clause/subject/target identity. No PlanId, execution, lowering or producer qualification credit is claimed.

Recorded focused evidence was inspected rather than rerunning Cargo, as dispatched: twelve monitor groups exercise all 60 reader recipes plus MP-32; one real default-feature consumer reads and encodes/re-admits the package; three default reader controls and 26 owner controls pass; nine negative doctests fail for E0515/E0451/E0277 as intended. fmt and workspace/all-targets plus default-model Clippy pass. These are focused records, not an aggregate gate. Two population Text NFC failures reproduce on the supplied base with the same selected QSpec input; they remain explicit baseline qualification failures. Final aggregate/hosted gates remain the parent's work.

Examined requirement excerpts and roles:

```yaml
scope:
- id: FR-038-AC-421
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. The strict reader admits QSpec FR-487 MP-01
    and MP-02 as distinct temporal monitor-plan roots with their exact seventeen ordered
    bindings, State, Inputs, Scope, Results, four ordinary pure-function signatures,
    six-case outcome and exact integer/rational/text/unit premises. The anonymous
    ownerless root has generated source occurrences and semantic_type equal to State;
    generic owner/source-map checks retain precedence. A capacity edit remains admitted;
    the entries' expression bodies are checked without execution and flatten/sequence
    extension MP-32 adds no operation
- id: FR-038-AC-422
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. Missing, reordered and extra bindings, wrong
    premise literal kinds and wrong reference kinds (MP-03..MP-09, MP-28, MP-35, MP-37,
    MP-39..MP-42) refuse invalid_semantic_graph at the first breaking binding. Missing
    bindings use the expected index and an extra uses the index after the seventeenth.
    Each refusal retains the root node locus and exact primary pointer; no admitted
    view escapes.
- id: FR-038-AC-423
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. Entry arity, parameter order/type and result-type
    defects, and missing/extra outcome cases or wrong Failed payload (MP-10..MP-17,
    MP-29), refuse ill_typed/type-mismatch at the applicable entry binding or results
    binding. Ordinary function body typing remains enforced before any admitted entry
    view is returned.
- id: FR-038-AC-424
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. Out-of-range width, gap, event rate and capacity
    (MP-18..MP-22), negative uncertainty, empty counter, negative rate and noncanonical
    rational spellings (MP-34, MP-36, MP-38, MP-46, MP-47) refuse invalid_package/invalid-value
    at the literal's value; descending/repeated/unreached buffer subformulas and missing,
    mistyped, extra or misordered state fields (MP-23..MP-27, MP-30, MP-31) refuse
    invalid_semantic_graph at buffers or state. State is exactly integer now, integer
    history_start, one buffer_<n> sequence per ordered buffer, then activations with
    the specified ord
- id: FR-038-AC-425
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. Paired defects select shape before reference
    kind, signatures/outcome before premise range, and range before buffers/state;
    a wrong dependencies set selects the earlier graph-shape invalid_semantic_graph
    at /dependencies. Two defective roots stored opposite structural order choose
    the structurally first root at the post-temporal plan step. The exact code, optional
    cause, primary pointer and root node locus are asserted for each control.
- id: FR-038-AC-426
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. A default-feature consumer iterates admitted
    plans and reads the specified typed root/type/clause keys, resolved root and entry
    structural content keys through structural_key, exact entry signatures and bodies,
    width/gap/counter/tick-unit/activation-capacity and exact rational event-rate/uncertainty
    plus ordered buffers/capacities through private-field borrowed views. Two plans
    sharing labels but differing selected keys remain distinct; empty plans yield
    an empty iterator. An entry or root content change is observed through the corresponding
    structural keys withou
- id: FR-038-AC-427
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. TC-048 reads QSpec proposals/checked-package-v2/monitor-plan-vectors.json
    MP-01..MP-73 by reference from the selected QSpec checkout, constructs complete
    selected packages around the MP-01..MP-31 and MP-33..MP-53 and MP-58..MP-65 root
    recipes and separately exercises MP-32's flatten/sequence expression, and checks
    exact admission or refusal code, cause, primary RFC 6901 pointer and structural
    locus. No upstream vector, schema or fixture is copied into IR. MP-54..MP-57 and
    MP-66..MP-73 remain by-reference QSL/runtime execution controls under AC-429/AC-430.
    These co
- id: FR-038-AC-428
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. MP-33 and MP-43..MP-45 admit zero uncertainty,
    changed counter, fractional rational rate and changed tick unit with distinct
    resolved root content keys; a capacity edit MP-02 also changes that key. MP-40
    and MP-41 refuse invalid_semantic_graph at tick_unit for a wrong node kind or
    non-Time unit by the exact base-Time identity test, never invalid-time-quantity.
    Exact rate and uncertainty are borrowed CheckedRational values with no floating
    conversion or narrowing. Tick intervals remain derivable from the selected clause
    and clock/tick-unit scales with no extra bind
- id: FR-038-AC-429
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. By-reference MP-48 and MP-49 refuse a missing
    on_gap or wrong reference kind as invalid_semantic_graph with no cause at /semantic_graph/nodes/{n}/body/members/15;
    MP-50..MP-52 refuse reordered or noninteger gap parameters and a wrong return
    type as ill_typed/type-mismatch at the same binding, retaining the root structural
    locus. MP-53 admits a changed on_gap node with a distinct resolved root content
    key. The mandatory borrowed on_gap entry view exposes exactly (State, integer
    gap) and (State, Results), its whole checked body and resolved entry key; no Option/defa
- id: FR-038-AC-430
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. By-reference MP-58 admits activation_capacity
    0 with a distinct root content key and the borrowed canonical nonnegative decimal
    getter returns "0" without narrowing. MP-59 refuses a negative value as invalid_package/invalid-value
    at /semantic_graph/nodes/{n}/body/members/16/value; MP-60 and MP-61 refuse absent
    or noninteger activation capacity as invalid_semantic_graph with no cause at member
    16. MP-62..MP-65 refuse missing history_start or activations, missing origin in
    an activation record and a noninteger history_start as invalid_semantic_graph
    with no cause at
- id: FR-019-AC-23
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: 'Implemented; evidence: `default_feature_consumer_reads_an_actual_admitted_monitor_plan`
    and the monitor-view compile-fail doctests. A default-feature external consumer
    imports CheckedMonitorPlanRef, CheckedMonitorEntryRef and CheckedMonitorBufferRef
    and uses CheckedPackageV2::monitor_plans and the exact typed accessor signatures
    of FR-038, including the nonnegative decimal activation_capacity getter. Compile
    controls show no public view constructor, mutable field, unchecked raw-to-admitted
    conversion or borrow escaping the selected package lifetime. The implemented inventory
    row participates i'
```

## Baseline traceability context

- FR-035-AC-5 (untagged): spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
- FR-035-AC-12 (untagged): spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
- FR-035-AC-32 (untagged): spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
- FR-035-AC-34 (untagged): spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
- FR-038-AC-10 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-11 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-19 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-20 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-59 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-88 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-90 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-92 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-113 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-123 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-126 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-129 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-130 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-131 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-132 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-135 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-148 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-183 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-184 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-215 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-216 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-232 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-233 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-234 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-235 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-319 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-320 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-321 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-342 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-412 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-413 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-414 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-415 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-416 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-038-AC-420 (untagged): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- FR-040-AC-26 (untagged): spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
- FR-058-AC-1 (untagged): spec/checked_package/functional/FR-058-admitted-clause-execution-point.md
- FR-058-AC-2 (untagged): spec/checked_package/functional/FR-058-admitted-clause-execution-point.md
- FR-058-AC-6 (untagged): spec/checked_package/functional/FR-058-admitted-clause-execution-point.md
- FR-058-AC-7 (untagged): spec/checked_package/functional/FR-058-admitted-clause-execution-point.md
- FR-061-AC-6 (untagged): spec/checked_package/functional/FR-061-admit-refinement-nodes.md
- FR-063-AC-2 (untagged): spec/checked_package/functional/FR-063-dimension-equivalence-node.md
- FR-066-AC-2 (untagged): spec/checked_package/functional/FR-066-retain-identity-bound-execution-points.md
- FR-067-AC-5 (untagged): spec/checked_package/functional/FR-067-admit-boolean-state-transition.md
- FR-069-AC-1 (untagged): spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md
- FR-069-AC-2 (untagged): spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md
- FR-069-AC-3 (untagged): spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md
- FR-069-AC-4 (untagged): spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md
- FR-069-AC-5 (untagged): spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md
- FR-069-AC-6 (untagged): spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md
- FR-069-AC-7 (untagged): spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md
- FR-069-AC-8 (untagged): spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md
- FR-346-AC-10 (untagged): spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md
- FR-348-AC-2 (untagged): spec/checked_package/functional/FR-348-admit-extension-form-nodes.md
- FR-352-AC-1 (untagged): spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md
- FR-352-AC-2 (untagged): spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md
- FR-352-AC-3 (untagged): spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md
- FR-352-AC-4 (untagged): spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md
- FR-352-AC-6 (untagged): spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md
- FR-352-AC-7 (untagged): spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md
- FR-352-AC-8 (untagged): spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md
- FR-352-AC-9 (untagged): spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md
- NFR-006-M-1 (untagged): spec/checked_package/non-functional/NFR-006-v2-producer-memory.md
- NFR-006-M-2 (untagged): spec/checked_package/non-functional/NFR-006-v2-producer-memory.md
- NFR-006-M-3 (untagged): spec/checked_package/non-functional/NFR-006-v2-producer-memory.md
- FR-057-AC-2 (untagged): spec/conformance/functional/FR-057-bind-cross-repository-conformance.md
- FR-057-AC-3 (untagged): spec/conformance/functional/FR-057-bind-cross-repository-conformance.md
- FR-057-AC-4 (untagged): spec/conformance/functional/FR-057-bind-cross-repository-conformance.md
- FR-057-AC-5 (untagged): spec/conformance/functional/FR-057-bind-cross-repository-conformance.md
- FR-057-AC-6 (untagged): spec/conformance/functional/FR-057-bind-cross-repository-conformance.md
- FR-057-AC-7 (untagged): spec/conformance/functional/FR-057-bind-cross-repository-conformance.md
- FR-057-AC-8 (untagged): spec/conformance/functional/FR-057-bind-cross-repository-conformance.md
- FR-057-AC-10 (untagged): spec/conformance/functional/FR-057-bind-cross-repository-conformance.md
- FR-045-AC-1 (untagged): spec/core/functional/FR-045-derive-proof-coverage-baseline.md
- FR-045-AC-2 (untagged): spec/core/functional/FR-045-derive-proof-coverage-baseline.md
- FR-045-AC-3 (untagged): spec/core/functional/FR-045-derive-proof-coverage-baseline.md
- FR-045-AC-4 (untagged): spec/core/functional/FR-045-derive-proof-coverage-baseline.md
- FR-045-AC-5 (untagged): spec/core/functional/FR-045-derive-proof-coverage-baseline.md
- FR-045-AC-6 (untagged): spec/core/functional/FR-045-derive-proof-coverage-baseline.md
- FR-055-AC-1 (untagged): spec/core/functional/FR-055-owned-lock-coherence-gate.md
- FR-055-AC-2 (untagged): spec/core/functional/FR-055-owned-lock-coherence-gate.md
- FR-055-AC-3 (untagged): spec/core/functional/FR-055-owned-lock-coherence-gate.md
- FR-055-AC-5 (untagged): spec/core/functional/FR-055-owned-lock-coherence-gate.md
- NFR-001-AC-2 (untagged): spec/core/non-functional/NFR-001-determinism.md
- NFR-001-AC-3 (untagged): spec/core/non-functional/NFR-001-determinism.md
- NFR-001-M-1 (untagged): spec/core/non-functional/NFR-001-determinism.md
- NFR-001-M-2 (untagged): spec/core/non-functional/NFR-001-determinism.md
- NFR-001-M-3 (untagged): spec/core/non-functional/NFR-001-determinism.md
- NFR-002-M-1 (untagged): spec/core/non-functional/NFR-002-portability-independence.md
- NFR-002-M-2 (untagged): spec/core/non-functional/NFR-002-portability-independence.md
- NFR-003-M-1 (untagged): spec/core/non-functional/NFR-003-diagnostic-integrity.md
- NFR-003-M-2 (untagged): spec/core/non-functional/NFR-003-diagnostic-integrity.md
- NFR-003-M-3 (untagged): spec/core/non-functional/NFR-003-diagnostic-integrity.md
- NFR-003-M-4 (untagged): spec/core/non-functional/NFR-003-diagnostic-integrity.md
- FR-030-AC-7 (untagged): spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
- FR-036-AC-1 (untagged): spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
- FR-036-AC-2 (untagged): spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
- FR-036-AC-3 (untagged): spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
- FR-036-AC-4 (untagged): spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
- FR-036-AC-5 (untagged): spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
- FR-016-AC-1 (untagged): spec/model/functional/FR-016-canonicalization-digests.md
- FR-016-AC-3 (untagged): spec/model/functional/FR-016-canonicalization-digests.md
- FR-017-AC-2 (untagged): spec/model/functional/FR-017-version-orphan-coverage.md
- FR-019-AC-4 (untagged): spec/model/functional/FR-019-rust-library-interface.md
- FR-019-AC-22 (untagged): spec/model/functional/FR-019-rust-library-interface.md
- FR-028-AC-4 (tagged-by-ignored-test): spec/model/functional/FR-028-separate-cycle-free-contract-model.md
- FR-046-AC-2 (untagged): spec/model/functional/FR-046-derive-candidate-application-node-identity.md
- FR-046-AC-6 (untagged): spec/model/functional/FR-046-derive-candidate-application-node-identity.md
- FR-052-AC-7 (untagged): spec/model/functional/FR-052-source-obligation-correspondence.md
- FR-052-AC-9 (untagged): spec/model/functional/FR-052-source-obligation-correspondence.md
- FR-052-AC-10 (untagged): spec/model/functional/FR-052-source-obligation-correspondence.md
- FR-052-AC-11 (untagged): spec/model/functional/FR-052-source-obligation-correspondence.md
- FR-052-AC-16 (untagged): spec/model/functional/FR-052-source-obligation-correspondence.md
- FR-052-AC-26 (untagged): spec/model/functional/FR-052-source-obligation-correspondence.md
- FR-052-AC-27 (untagged): spec/model/functional/FR-052-source-obligation-correspondence.md
- FR-070-AC-1 (untagged): spec/model/functional/FR-070-certificate-member-vocabulary.md
- FR-070-AC-2 (untagged): spec/model/functional/FR-070-certificate-member-vocabulary.md
- FR-070-AC-3 (untagged): spec/model/functional/FR-070-certificate-member-vocabulary.md
- FR-070-AC-4 (untagged): spec/model/functional/FR-070-certificate-member-vocabulary.md
- FR-350-AC-1 (untagged): spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
- FR-350-AC-2 (untagged): spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
- FR-350-AC-3 (untagged): spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
- FR-350-AC-4 (untagged): spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
- FR-350-AC-5 (untagged): spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
- FR-350-AC-6 (untagged): spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
- FR-350-AC-7 (untagged): spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
- FR-350-AC-8 (untagged): spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md

## Review carry — pass 1

The review carries to frozen PR head `12ddb85e860e8804fc804f73919c8b565b163700`. Local HEAD and GitHub PR head agree. The merge combines review-artifact parent `d1281f487348c38a60e97ab1695c79df2c1774a5` with main `dcfe1b25da9dbe8e7d84b51ee57198a23843a39f`. No Rust, Cargo manifest, lockfile or Makefile difference exists from the original reviewed candidate, so the source/test analysis and its focused evidence remain applicable.

The single FR-019 resolution preserves implemented AC-23 verbatim from the reader branch and planned AC-24 verbatim from main. The operation-effect inventory row and complete TC-058 document also match main verbatim. AC-24 describes IR-1267's separate planned raw operation-effect-row read and does not assert that reader PR #824 implements it. Scoped Quire validation and merge diff whitespace checks pass. No new finding was found; the original verdict carries, including the whole-repository gap limitation.

The prior aggregate strict-coverage failure contains 98 unbacked rows and zero contradicted statuses. A structural git-show comparison confirms all 98 exact table rows exist unchanged on base `2aba7d2f4f04e624f57cfbf2c38948b8ddaff628`; the reader diff removes no evidence declarations and changes no module configuration. This is structural evidence of inheritance, not a baseline coverage-engine rerun. No coverage-engine or aggregate result is claimed for this merged head. Parent-owned final aggregate/hosted qualification remains required. Actual QSL producer, downstream lowering/execution and overall IR-721 completion remain open.

Additional context examined:

```yaml
scope:
- id: FR-019-AC-24
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: context_only
  excerpt: planned until IR-1267 lands. A default-feature external consumer calls
    `CheckedPackageV2::operation_effect_rows(&self) -> Option<&[CheckedOperationEffectRow]>`
    only after strict package admission. It obtains `None` when the optional carrier
    was omitted and a borrowed, nonempty slice of the exact admitted producer rows
    when present, retaining row order, selected document and declaring identities,
    original operation members, all four ordered effect collections and provenance;
    an explicitly empty effect remains a present row. It passes that slice directly
    as `CheckedPackageSidecarsRef::operation_
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: context_only
  excerpt: Existing public-interface test case; the complete merged document matches
    incoming main verbatim.
```

## Normative amendment review — pass 2

The new untagged planned obligations are explicit coding prerequisites, not silently classified as inherited gaps or as completed. Plan completion: not assessed.

Frozen normative review at PR #824 head `32926d915c4356aa0e79f8a00cca4735f39ba382`, verified against local and GitHub heads. The author-owned normative commit changes only FR-038 relative to parent `c889608dc02bb22d74d3932b726fec64a4e2418c`; incoming main's source/TC-058 work is distinct upstream material and receives no new source-review credit here. This is specification-first acceptance before reader implementation changes, not a final implementation carry or merge approval.

The exact State is now, history_start, ordered buffer_<n>, activations, then integer event_position. MP-74..76 select invalid_semantic_graph, no cause, at root member 1 with the root structural locus. There remain exactly seventeen root bindings and no new getter. Status stays opaque. MP-77's event increments (baseline included), MP-57's gap behavior and MP-69's poll behavior remain QSL/runtime program controls; the reader only checks shape/type and executes no entry. Updated QSpec contains 63 reader recipes; the new execution control is not a 64th reader recipe.

FR-322-AC-309 gives the producer's declaration-less entry derivation: parameter count, ordered parameter indices, body root and optional measure. EK-01..06 concern supplied-key equality/change; EK-07 is the existing duplicate package/path/ordinal refusal at the second node_id; EK-08 preserves a model accessor's owner-selected label. The reader resolves supplied keys, never recomputes the producer derivation or adds a content-path mismatch refusal. Root plus four entry keys remain the identity, with independent downstream context selection still required and no PlanId.

AC-431 and AC-432 are explicitly planned. Existing implementation claims distinguish those pending obligations. The source and prior 60-recipe logs do not qualify this amended contract: current matrix has 960 obligations, 805 tagged, 132 untagged, 1 ignored-only and 22 method-without-symbol. Both new criteria are untagged until the coding phase adds real controls. The gap-analysis overall verdict remains FAIL; the two planned additions must be implemented and tested before final qualification. Historical 98-row strict-coverage structural proof remains limited to its prior reviewed head; no new strict coverage, baseline engine replay, Cargo or aggregate run is claimed.

Scoped Quire validation passes, with one inherited AC grammar warning outside the changed text. quoin advise reports both new criteria authored Test with no mismatch, inconclusive or uncatalogued result. Reviewer judgment: complete-package reader tests and by-reference identity controls fit these admission obligations; runtime recommendations do not assign execution semantics to IR. No introduced normative defect was found. Actual producer/lowering/runtime differential qualification and overall IR-721 remain open.

```yaml
scope:
- id: FR-038-AC-424
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. Out-of-range width, gap, event rate and capacity
    (MP-18..MP-22), negative uncertainty, empty counter, negative rate and noncanonical
    rational spellings (MP-34, MP-36, MP-38, MP-46, MP-47) refuse invalid_package/invalid-value
    at the literal's value; descending/repeated/unreached buffer subformulas and missing,
    mistyped, extra or misordered state fields (MP-23..MP-27, MP-30, MP-31) refuse
    invalid_semantic_graph at buffers or state. State is exactly integer now, integer
    history_start, one buffer_<n> sequence per ordered buffer, then activations with
    the specified ord
- id: FR-038-AC-427
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. TC-048 reads QSpec proposals/checked-package-v2/monitor-plan-vectors.json
    MP-01..MP-77 by reference from the selected QSpec checkout, constructs complete
    selected packages around the MP-01..MP-31 and MP-33..MP-53 and MP-58..MP-65 root
    recipes (MP-74..MP-76 planned under AC-431) and separately exercises MP-32's flatten/sequence
    expression, and checks exact admission or refusal code, cause, primary RFC 6901
    pointer and structural locus. No upstream vector, schema or fixture is copied
    into IR. MP-54..MP-57, MP-66..MP-73 and MP-77 remain by-reference QSL/runtime
    execu
- id: FR-038-AC-431
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-721 lands. The exact monitor State is now, history_start,
    the ordered buffer_<n> fields, activations, then integer event_position. By-reference
    QSpec MP-74..MP-76 refuse a missing event_position, text-typed event_position
    or placement before activations as invalid_semantic_graph with no cause at /semantic_graph/nodes/{n}/body/members/1,
    retaining the root structural locus. Updated positive State contexts admit with
    no added root binding or public getter. MP-77, MP-57 and MP-69 remain by-reference
    QSL/runtime controls for init zero, one increment per admitted event including
    ba
- id: FR-038-AC-432
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'planned until IR-721 lands. TC-048 reads QSpec entry-key-vectors.json
    EK-01..EK-08 by reference: the published declaration-less function path has parameter
    count, ordered parameter indices, body-root index and optional measure index;
    equal content gives equal paths and a changed body, reordered parameter or measure
    gives a different path. Complete package controls resolve those supplied keys,
    refuse EK-07''s duplicate package/path/ordinal as invalid_semantic_graph at the
    second node_id, and admit EK-08''s owner-chosen model accessor label unchanged.
    The reader recomputes no producer path and add'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The state SHALL have exactly integer `now`, integer `history_start`, one

    sequence field `buffer_<n>` per buffer in order, then `activations`, a sequence

    of records with exactly ordered fields `subformula` (integer), `origin`

    (integer), and `status`; finally integer `event_position` (this final field is

    planned until IR-721 lands). The published plan contract leaves the type of

    `status` unspecified; the reader SHALL preserve its ordinary admitted field

    type without inventing an additional plan-specific type constraint.

    Planned until IR-721 lands: the exact closed State SHALL end with

    `event_pos'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Planned until IR-721 lands: the exact closed State SHALL end with

    `event_position`, an ordinary integer field after `activations`; it SHALL

    refuse absence, text typing or earlier placement as `invalid_semantic_graph`

    with no cause at the root `state` binding. The seventeen root bindings remain

    unchanged. QSpec FR-487-AC-11 owns its meaning as the unwrapped monotone count

    of admitted events: init 0, plus one for each admitted event including the

    baseline on_event(0), unchanged by watermark or gap, with polling calling no

    entry. These are QSL/runtime program responsibilities; reader admission ch'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The reader SHALL NOT recompute this derivation or refuse a path merely
    for

    differing from it; it resolves the supplied identities and refuses duplicate

    package/path/ordinal identities under the existing graph rule. The new

    by-reference EK-01..EK-08 qualification is planned under AC-432, not a claim

    that the reader implements a new content-path checker.


    #### Borrowed monitor-plan reads


    `CheckedPackageV2::monitor_plans(&self)` SHALL return an

    `impl ExactSizeIterator<Item = CheckedMonitorPlanRef<''_>>` over admitted roots

    in ascending structural node-id order. No roots yields an empty iterator.

    '
- id: FR-487-AC-11
  path: /home/peter/dev/qualification-inputs/IR-721-qspec/spec/objects/interfaces/FR-487-checked-package-monitor-plan-body.md
  role: context_only
  excerpt: The state ends with `event_position`, an `integer` that `init` sets to
    0, `on_event` increases by 1 per admitted event (the baseline reading included),
    and `on_watermark`, `on_gap` and a clock poll leave unchanged (`MP-77`, `MP-57`,
    `MP-69`); a state lacking `event_position`, with it typed `text` or placed before
    `activations` refuses `invalid_semantic_graph` at the `state` binding (`MP-74`
    to `MP-76`).
- id: FR-322-AC-309
  path: /home/peter/dev/qualification-inputs/IR-721-qspec/spec/objects/interfaces/FR-322-checked-package-artifact.md
  role: context_only
  excerpt: 'A declaration-less `function` node has the derived path `["function",
    <semantic_form>, <decimal parameter count>, <parameter index segments in order>,
    <body root index segment>, <measure index segment if any>]`: equal parameters,
    body root and measure give an equal path, and a changed body root, a changed or
    reordered parameter or an added measure gives a different one; two nodes with
    equal `package`, `path` and `ordinal` are refused as a duplicate `node_id`, and
    a model-owned accessor with an owner-chosen label is admitted unchanged (`proposals/checked-package-v2/entry-key-vectors.json`,
    `EK-'
```

## Normative optional-measure follow-up

The optional-measure check identified FND-002 in SR-1290 (integrity). The earlier clean wording in this normative-pass narrative is superseded for the overall amendment: specification acceptance and the coding handoff await its fix. Other methods report no separate defect; no implementation or merge approval is given.

## Normative fix verification — pass 3

Frozen fix head `958c1f44ba1dd43c75591bc25e950f61d47509df` matches local and GitHub PR heads; the fix changes only FR-038. The owning paragraph and AC-432 now explicitly require exact ordered parameters/body/optional-final-measure admission, including a genuine complete EK-06 package with typed references. Malformed measure, duplicate, extra, reordered and clause-only controls retain existing ordinary validation precedence without measure execution, semantic recheck or path recomputation. This resolves SR-1290 FND-002; its original finding text is unchanged. No new normative defect was found. Scoped Quire and whitespace validation pass, with only the inherited unrelated AC grammar advisory.

**Normative verdict: PASS.** The coder may implement AC-431/432 against this reviewed contract. This is not a final code/aggregate pass: current code and tests still require the event_position/optional-measure changes and fresh by-reference qualification. Previous whole-repository gaps and real-producer/lowering/runtime limitations remain.

## Final amended reader review — pass 4

Frozen final reader head `09b843ee4e3a482395846990eacebe0d782db2fa` matches both local and GitHub PR head; the tree is clean. The amended implementation is reviewed against QSpec `75508820b5003a342b2af8782dc400bdc9080403`, consumed by reference. No introduced defect was found. SR-1290 FND-002 remains fixed; no finding row or prior disposition was rewritten.

The closed State requires exactly buffers.len()+4 fields: integer now, integer history_start, the ordered buffer_<n> sequences, activations with its existing opaque-status shape, and final integer event_position. Old State, wrong type, order, duplicate/extra field and renamed final field controls refuse at root member 1. There is no compatibility/default path, new root binding, getter, execution counter or fixed-width narrowing. The State rule changes admission only.

The anonymous-owner shape gate and monitor-entry parser both accept exactly parameters/body/[optional final measure]. Parameter references still target parameter nodes and body references still target expressions. The optional measure must be an ordinary resolved reference; no measure evaluator or new semantic check is added. Outside monitor selection malformed literal/unknown-member measure references, duplicate/extra/reordered/clause-only bindings and wrong parameter/body reference kinds retain ordinary owner-form refusal. Selected monitor entries retain root binding/signature refusal and ordinary malformed-wire precedence. Independent ordinary owner/model joins still validate the full graph. The optional-measure exception does not enable other function forms.

The test fixture producer now constructs the formal parameter-count/ordered-parameter/body-root/optional-measure path. It is fixture production, not a reader derivation. EK-01..06 are materialized in complete admitted packages and compared to independent published paths by reference, including a genuine measured function without a monitor root. EK-07 duplicate identity refuses at the second node_id; distinct ordinal admission remains legal. EK-08 uses a real selected model accessor label. An additional assigned-path test holds the function body constant and verifies admission/resolved identity without path recomputation. Selected measured entry and whole root keys change together. No independent PlanId or copied upstream vectors/schema are added.

Fresh focused logs inspected: monitor-amended 15 groups (all 63 reader recipes, MP-32 and adjacent controls); entry-keys 5 groups (four new entry-key tests and one overlapping existing identity test); public-amended one actual default-feature consumer with measure/State reads and canonical encode/read preservation; default-it-amended three controls; owner-controls-amended 26 controls; public-doc-amended nine negative probes with intended E0515/E0451/E0277; amended-lint-fmt passes fmt, string-edge, workspace/all-targets and default-model Clippy. Recorded red runs demonstrate the old reader admitted MP-74's invalid old State and rejected the genuine optional-measure function, with both now passing. No Cargo command was run by this reviewer.

MP-77, MP-57, MP-69 and the other execution controls remain by-reference runtime/QSL context, never reader verdicts. No entry or measure body is executed by the reader. Real producer closure, RT/driver lowering, executor/runtime differential qualification and overall IR-721 completion remain open. Parent-owned aggregate/hosted gates are still required after review; focused logs are not an aggregate pass.

Fresh static matrix: 960 obligations, 807 tagged, 130 untagged, 1 tagged-by-ignored-test, 22 method-without-symbol. AC-431 and AC-432 are now tagged by real tests; all thirteen inspected monitor/interface ACs are tagged. Whole-repository gap verdict remains FAIL for inherited gaps, while the changed reader surface has no new gap. Fresh coverage JSON still has exactly the same 98 normalized unbacked rows and zero contradicted statuses as the earlier reviewed coverage. Each final-head table row also exists byte-for-byte on the supplied base via git show. This is structural inheritance evidence, not a baseline coverage-engine rerun. No new monitor/entry-key untracked symbols or diagnostics appear; 116 unrelated untracked symbols remain.

Plan completion: not assessed

Final scoped FR-038 validation and diff whitespace checks pass. The new EARS statements and previously reviewed component allocation carry unchanged through these implementation commits. No applicable AssuranceProfile was introduced.

**Method result: FAIL for inherited repository gaps; no introduced reader gap.**

```yaml
scope:
- id: FR-038-AC-421
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. The strict reader admits QSpec FR-487 MP-01
    and MP-02 as distinct temporal monitor-plan roots with their exact seventeen ordered
    bindings, State, Inputs, Scope, Results, four ordinary pure-function signatures,
    six-case outcome and exact integer/rational/text/unit premises. The anonymous
    ownerless root has generated source occurrences and semantic_type equal to State;
    generic owner/source-map checks retain precedence. A capacity edit remains admitted;
    the entries' expression bodies are checked without execution and flatten/sequence
    extension MP-32 adds no operation
- id: FR-038-AC-422
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. Missing, reordered and extra bindings, wrong
    premise literal kinds and wrong reference kinds (MP-03..MP-09, MP-28, MP-35, MP-37,
    MP-39..MP-42) refuse invalid_semantic_graph at the first breaking binding. Missing
    bindings use the expected index and an extra uses the index after the seventeenth.
    Each refusal retains the root node locus and exact primary pointer; no admitted
    view escapes.
- id: FR-038-AC-423
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. Entry arity, parameter order/type and result-type
    defects, and missing/extra outcome cases or wrong Failed payload (MP-10..MP-17,
    MP-29), refuse ill_typed/type-mismatch at the applicable entry binding or results
    binding. Ordinary function body typing remains enforced before any admitted entry
    view is returned.
- id: FR-038-AC-424
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. Out-of-range width, gap, event rate and capacity
    (MP-18..MP-22), negative uncertainty, empty counter, negative rate and noncanonical
    rational spellings (MP-34, MP-36, MP-38, MP-46, MP-47) refuse invalid_package/invalid-value
    at the literal's value; descending/repeated/unreached buffer subformulas and missing,
    mistyped, extra or misordered state fields (MP-23..MP-27, MP-30, MP-31) refuse
    invalid_semantic_graph at buffers or state. State is exactly integer now, integer
    history_start, one buffer_<n> sequence per ordered buffer, then activations with
    the specified ord
- id: FR-038-AC-425
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. Paired defects select shape before reference
    kind, signatures/outcome before premise range, and range before buffers/state;
    a wrong dependencies set selects the earlier graph-shape invalid_semantic_graph
    at /dependencies. Two defective roots stored opposite structural order choose
    the structurally first root at the post-temporal plan step. The exact code, optional
    cause, primary pointer and root node locus are asserted for each control.
- id: FR-038-AC-426
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. A default-feature consumer iterates admitted
    plans and reads the specified typed root/type/clause keys, resolved root and entry
    structural content keys through structural_key, exact entry signatures and bodies,
    width/gap/counter/tick-unit/activation-capacity and exact rational event-rate/uncertainty
    plus ordered buffers/capacities through private-field borrowed views. Two plans
    sharing labels but differing selected keys remain distinct; empty plans yield
    an empty iterator. An entry or root content change is observed through the corresponding
    structural keys withou
- id: FR-038-AC-427
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. TC-048 reads QSpec proposals/checked-package-v2/monitor-plan-vectors.json
    MP-01..MP-77 by reference from the selected QSpec checkout, constructs complete
    selected packages around the MP-01..MP-31 and MP-33..MP-53 and MP-58..MP-65 root
    recipes (MP-74..MP-76 planned under AC-431) and separately exercises MP-32's flatten/sequence
    expression, and checks exact admission or refusal code, cause, primary RFC 6901
    pointer and structural locus. No upstream vector, schema or fixture is copied
    into IR. MP-54..MP-57, MP-66..MP-73 and MP-77 remain by-reference QSL/runtime
    execu
- id: FR-038-AC-428
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. MP-33 and MP-43..MP-45 admit zero uncertainty,
    changed counter, fractional rational rate and changed tick unit with distinct
    resolved root content keys; a capacity edit MP-02 also changes that key. MP-40
    and MP-41 refuse invalid_semantic_graph at tick_unit for a wrong node kind or
    non-Time unit by the exact base-Time identity test, never invalid-time-quantity.
    Exact rate and uncertainty are borrowed CheckedRational values with no floating
    conversion or narrowing. Tick intervals remain derivable from the selected clause
    and clock/tick-unit scales with no extra bind
- id: FR-038-AC-429
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. By-reference MP-48 and MP-49 refuse a missing
    on_gap or wrong reference kind as invalid_semantic_graph with no cause at /semantic_graph/nodes/{n}/body/members/15;
    MP-50..MP-52 refuse reordered or noninteger gap parameters and a wrong return
    type as ill_typed/type-mismatch at the same binding, retaining the root structural
    locus. MP-53 admits a changed on_gap node with a distinct resolved root content
    key. The mandatory borrowed on_gap entry view exposes exactly (State, integer
    gap) and (State, Results), its whole checked body and resolved entry key; no Option/defa
- id: FR-038-AC-430
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. By-reference MP-58 admits activation_capacity
    0 with a distinct root content key and the borrowed canonical nonnegative decimal
    getter returns "0" without narrowing. MP-59 refuses a negative value as invalid_package/invalid-value
    at /semantic_graph/nodes/{n}/body/members/16/value; MP-60 and MP-61 refuse absent
    or noninteger activation capacity as invalid_semantic_graph with no cause at member
    16. MP-62..MP-65 refuse missing history_start or activations, missing origin in
    an activation record and a noninteger history_start as invalid_semantic_graph
    with no cause at
- id: FR-038-AC-431
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-721 lands. The exact monitor State is now, history_start,
    the ordered buffer_<n> fields, activations, then integer event_position. By-reference
    QSpec MP-74..MP-76 refuse a missing event_position, text-typed event_position
    or placement before activations as invalid_semantic_graph with no cause at /semantic_graph/nodes/{n}/body/members/1,
    retaining the root structural locus. Updated positive State contexts admit with
    no added root binding or public getter. MP-77, MP-57 and MP-69 remain by-reference
    QSL/runtime controls for init zero, one increment per admitted event including
    ba
- id: FR-038-AC-432
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'planned until IR-721 lands. TC-048 reads QSpec entry-key-vectors.json
    EK-01..EK-08 by reference: the published declaration-less function path has parameter
    count, ordered parameter indices, body-root index and optional measure index;
    equal content gives equal paths and a changed body, reordered parameter or measure
    gives a different path. Complete package controls admit genuine declaration-less
    pure_function bodies with exactly ordered parameters/body and an optional final
    measure, including EK-06 with typed parameter references, body-root reference
    and measure reference. Negative controls rej'
- id: FR-019-AC-23
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: 'Implemented; evidence: `default_feature_consumer_reads_an_actual_admitted_monitor_plan`
    and the monitor-view compile-fail doctests. A default-feature external consumer
    imports CheckedMonitorPlanRef, CheckedMonitorEntryRef and CheckedMonitorBufferRef
    and uses CheckedPackageV2::monitor_plans and the exact typed accessor signatures
    of FR-038, including the nonnegative decimal activation_capacity getter. Compile
    controls show no public view constructor, mutable field, unchecked raw-to-admitted
    conversion or borrow escaping the selected package lifetime. The implemented inventory
    row participates i'
bindings:
- test_id: monitor_plan_published_reader_recipes_have_exact_outcomes
  ac_id: FR-038-AC-421
  trace: correct
- test_id: monitor_plan_mp32_uses_existing_flatten_and_sequence_operations
  ac_id: FR-038-AC-421
  trace: correct
- test_id: monitor_plan_published_reader_recipes_have_exact_outcomes
  ac_id: FR-038-AC-422
  trace: correct
- test_id: monitor_plan_adjacent_stages_and_premises_keep_first_refusal
  ac_id: FR-038-AC-422
  trace: correct
- test_id: monitor_plan_owner_source_map_and_wire_defects_keep_precedence
  ac_id: FR-038-AC-422
  trace: correct
- test_id: monitor_plan_empty_body_refuses_at_first_required_binding
  ac_id: FR-038-AC-422
  trace: correct
- test_id: monitor_plan_published_reader_recipes_have_exact_outcomes
  ac_id: FR-038-AC-423
  trace: correct
- test_id: monitor_plan_adjacent_stages_and_premises_keep_first_refusal
  ac_id: FR-038-AC-423
  trace: correct
- test_id: monitor_plan_entry_owner_exception_cannot_admit_a_model_descriptor
  ac_id: FR-038-AC-423
  trace: correct
- test_id: monitor_plan_published_reader_recipes_have_exact_outcomes
  ac_id: FR-038-AC-424
  trace: correct
- test_id: monitor_plan_adjacent_stages_and_premises_keep_first_refusal
  ac_id: FR-038-AC-424
  trace: correct
- test_id: monitor_plan_state_layout_rejects_order_extra_and_wrong_activation_types
  ac_id: FR-038-AC-424
  trace: correct
- test_id: monitor_plan_published_reader_recipes_have_exact_outcomes
  ac_id: FR-038-AC-425
  trace: correct
- test_id: monitor_plan_adjacent_stages_and_premises_keep_first_refusal
  ac_id: FR-038-AC-425
  trace: correct
- test_id: monitor_plan_exact_work_limit_admits_and_one_less_has_no_partial_package
  ac_id: FR-038-AC-425
  trace: correct
- test_id: monitor_plan_root_order_and_content_changes_use_selected_keys
  ac_id: FR-038-AC-425
  trace: correct
- test_id: monitor_plan_rational_work_denial_stays_incomplete
  ac_id: FR-038-AC-425
  trace: correct
- test_id: monitor_plan_owner_source_map_and_wire_defects_keep_precedence
  ac_id: FR-038-AC-425
  trace: correct
- test_id: monitor_plan_entry_owner_exception_cannot_admit_a_model_descriptor
  ac_id: FR-038-AC-425
  trace: correct
- test_id: anonymous_entry_support_keeps_model_and_source_owner_requirements
  ac_id: FR-038-AC-425
  trace: correct
- test_id: monitor_plan_published_reader_recipes_have_exact_outcomes
  ac_id: FR-038-AC-426
  trace: correct
- test_id: monitor_plan_exact_reads_allow_unbounded_capacities_and_opaque_status
  ac_id: FR-038-AC-426
  trace: correct
- test_id: monitor_plan_resolved_root_and_four_entry_keys_survive_graph_permutation
  ac_id: FR-038-AC-426
  trace: correct
- test_id: monitor_plan_exact_work_limit_admits_and_one_less_has_no_partial_package
  ac_id: FR-038-AC-426
  trace: correct
- test_id: monitor_plan_root_order_and_content_changes_use_selected_keys
  ac_id: FR-038-AC-426
  trace: correct
- test_id: monitor_plan_absence_is_an_empty_exact_size_iterator
  ac_id: FR-038-AC-426
  trace: correct
- test_id: monitor_plan_published_reader_recipes_have_exact_outcomes
  ac_id: FR-038-AC-427
  trace: correct
- test_id: monitor_plan_mp32_uses_existing_flatten_and_sequence_operations
  ac_id: FR-038-AC-427
  trace: correct
- test_id: monitor_plan_adjacent_stages_and_premises_keep_first_refusal
  ac_id: FR-038-AC-428
  trace: correct
- test_id: monitor_plan_exact_reads_allow_unbounded_capacities_and_opaque_status
  ac_id: FR-038-AC-428
  trace: correct
- test_id: monitor_plan_resolved_root_and_four_entry_keys_survive_graph_permutation
  ac_id: FR-038-AC-428
  trace: correct
- test_id: monitor_plan_root_order_and_content_changes_use_selected_keys
  ac_id: FR-038-AC-428
  trace: correct
- test_id: monitor_plan_rational_work_denial_stays_incomplete
  ac_id: FR-038-AC-428
  trace: correct
- test_id: monitor_plan_tick_unit_uses_the_admitted_model_dimension_identity
  ac_id: FR-038-AC-428
  trace: correct
- test_id: monitor_plan_published_reader_recipes_have_exact_outcomes
  ac_id: FR-038-AC-429
  trace: correct
- test_id: monitor_plan_exact_reads_allow_unbounded_capacities_and_opaque_status
  ac_id: FR-038-AC-429
  trace: correct
- test_id: monitor_plan_resolved_root_and_four_entry_keys_survive_graph_permutation
  ac_id: FR-038-AC-429
  trace: correct
- test_id: monitor_plan_root_order_and_content_changes_use_selected_keys
  ac_id: FR-038-AC-429
  trace: correct
- test_id: monitor_plan_adjacent_stages_and_premises_keep_first_refusal
  ac_id: FR-038-AC-430
  trace: correct
- test_id: monitor_plan_exact_reads_allow_unbounded_capacities_and_opaque_status
  ac_id: FR-038-AC-430
  trace: correct
- test_id: monitor_plan_state_layout_rejects_order_extra_and_wrong_activation_types
  ac_id: FR-038-AC-430
  trace: correct
- test_id: monitor_plan_root_order_and_content_changes_use_selected_keys
  ac_id: FR-038-AC-430
  trace: correct
- test_id: monitor_plan_published_reader_recipes_have_exact_outcomes
  ac_id: FR-038-AC-431
  trace: correct
- test_id: monitor_plan_event_position_old_shape_is_refused
  ac_id: FR-038-AC-431
  trace: correct
- test_id: monitor_plan_genuine_measure_entry_keeps_signature_and_closed_body_checks
  ac_id: FR-038-AC-431
  trace: correct
- test_id: monitor_plan_event_position_is_the_exact_final_integer_field
  ac_id: FR-038-AC-431
  trace: correct
- test_id: entry_keys_published_paths_and_admission_cover_optional_measure
  ac_id: FR-038-AC-432
  trace: correct
- test_id: entry_keys_measure_function_is_admitted_without_a_monitor_root
  ac_id: FR-038-AC-432
  trace: correct
- test_id: entry_keys_assigned_label_is_resolved_without_recomputation
  ac_id: FR-038-AC-432
  trace: correct
- test_id: entry_keys_malformed_measure_and_descriptor_bindings_keep_owner_refusals
  ac_id: FR-038-AC-432
  trace: correct
- test_id: monitor_plan_genuine_measure_entry_keeps_signature_and_closed_body_checks
  ac_id: FR-038-AC-432
  trace: correct
- test_id: monitor_plan_absence_is_an_empty_exact_size_iterator
  ac_id: FR-019-AC-23
  trace: correct
```

## Final main-merge carry — pass 5

Frozen main-merge carry head `47bdf036717575ebcf9d132c5155dffb36e777da` matches local and GitHub PR heads; tree clean. Parents are the reviewed-source/review-record head `ef9909be2e6db4a35bbc28feea5d9291768eab95` (reader source from `09b843ee4e3a482395846990eacebe0d782db2fa`) and main `6c5e430c58e802f5fb84a25c967c7520797d7269`. No new merge defect was found; reader review carries and FND-002 remains fixed.

Historical IDs in all earlier sections and finding/disposition records remain unchanged. The exact live mapping is:

| Historical reader criterion | Live reader criterion | Meaning |
| --- | --- | --- |
| FR-038-AC-431 | FR-038-AC-433 | Final integer event_position State admission; MP-74..76, with runtime MP-77 context |
| FR-038-AC-432 | FR-038-AC-434 | Producer entry keys and genuine optional-measure admission; EK-01..08; SR-1290 FND-002 |

The current FR-038-AC-431 and AC-432 are main's separate planned FMA obligations from IR-1265, not the historical reader criteria meant by earlier sections of these artifacts. Neither these reader tests nor the historical FND-002 disposition claim FMA coverage. Main owns those existing live IDs; only the previously unmerged reader criteria were renumbered. The reader AC texts are byte-identical after ID substitution, and all three affected source/test files differ only by those trace-ID substitutions. Owning prose references follow the same mapping.

The monitor-plan implementation, including exact final State shape, anonymous optional-measure helper and entry parser/index, is byte-identical to the prior reviewed source. The owner.rs merge is exactly the previous reader owner logic plus main's Event additions: model event_type requires the correct model owner; selected exact event declaration identities populate DomainModel.event_types and participate in owner/invariant joins. The model_members.rs result matches main byte-for-byte. Both main's Event owner logic and the reader's bounded anonymous-function exception survive, with no additional semantic change. Historical SR files are preserved.

Fresh supplied logs inspected at this merge: 15 monitor groups, 5 entry-key groups (one identity overlap), 28 owner controls, one actual default consumer/read/roundtrip, nine intended-error negative probes, three default reader controls, and fmt/string-edge/workspace/default-model Clippy all pass. The earlier two known population NFD failures remain separately reported, not hidden by these focused owner controls. Parent's prior full native run had five scanner-PATH environment failures; no source defect or pass is inferred from them. The environment-corrected full native run and final compile/corpus/spec/hosted gates remain parent-owned and pending after this carry. Draft HOLD stays in effect; Luna alone owns merge.

Fresh static matrix at the merged head: 962 obligations, 808 tagged, 131 untagged, 1 ignored-only, 22 method-without-symbol. All thirteen live reader/interface criteria examined here are tagged, including AC-433/434. Main's planned FMA AC-431/432 are untagged and are distinct from the carried reader scope. Whole-repository gap verdict remains FAIL; no clean-repository claim is made. Prior strict-coverage 98-unbacked/zero-contradicted results and structural inheritance comparison apply only to their recorded earlier head. No merged-head strict-coverage count or baseline engine replay is claimed.

Reader qualification still means 63 admission recipes plus existing MP-32 and by-reference EK controls. MP-77 and the other runtime controls remain unexecuted reader context. Actual producer closure, RT/driver lowering, runtime differential qualification and overall IR-721 completion remain open. No Cargo was run by this reviewer. Scoped renumber Quire validation and merge whitespace checks pass. Plan completion: not assessed.

```yaml
criterion_id_mapping:
- historical_id: FR-038-AC-431
  live_id: FR-038-AC-433
  meaning: monitor State final integer event_position; MP-74..76 admission controls
    and runtime-owned MP-77 context
- historical_id: FR-038-AC-432
  live_id: FR-038-AC-434
  meaning: formal producer entry keys, genuine optional-measure function admission,
    EK-01..08; includes SR-1290 FND-002
scope:
- id: FR-038-AC-420
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: 'A structural `model`/`event_type` node without `declaration` requires
    `ModelOwner`: the selected event type in QSpec TC-281 EV-01 passes the owner join,
    while EV-02''s owners naming a record value type, object type or absent declaration
    refuse `missing_declaration`/`missing-selection` at the node''s `node_id`. A structural
    `model`/`event_type` node with `declaration` retains its required `SourceOwner`.
    An undeclared event node with `SourceOwner` or an owner of another kind refuses
    `malformed_wire` at `/semantic_graph/nodes/{N}/owner`; an absent required owner
    refuses `malformed_wire` at `/semant'
- id: FR-038-AC-421
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. The strict reader admits QSpec FR-487 MP-01
    and MP-02 as distinct temporal monitor-plan roots with their exact seventeen ordered
    bindings, State, Inputs, Scope, Results, four ordinary pure-function signatures,
    six-case outcome and exact integer/rational/text/unit premises. The anonymous
    ownerless root has generated source occurrences and semantic_type equal to State;
    generic owner/source-map checks retain precedence. A capacity edit remains admitted;
    the entries' expression bodies are checked without execution and flatten/sequence
    extension MP-32 adds no operation
- id: FR-038-AC-422
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. Missing, reordered and extra bindings, wrong
    premise literal kinds and wrong reference kinds (MP-03..MP-09, MP-28, MP-35, MP-37,
    MP-39..MP-42) refuse invalid_semantic_graph at the first breaking binding. Missing
    bindings use the expected index and an extra uses the index after the seventeenth.
    Each refusal retains the root node locus and exact primary pointer; no admitted
    view escapes.
- id: FR-038-AC-423
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. Entry arity, parameter order/type and result-type
    defects, and missing/extra outcome cases or wrong Failed payload (MP-10..MP-17,
    MP-29), refuse ill_typed/type-mismatch at the applicable entry binding or results
    binding. Ordinary function body typing remains enforced before any admitted entry
    view is returned.
- id: FR-038-AC-424
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. Out-of-range width, gap, event rate and capacity
    (MP-18..MP-22), negative uncertainty, empty counter, negative rate and noncanonical
    rational spellings (MP-34, MP-36, MP-38, MP-46, MP-47) refuse invalid_package/invalid-value
    at the literal's value; descending/repeated/unreached buffer subformulas and missing,
    mistyped, extra or misordered state fields (MP-23..MP-27, MP-30, MP-31) refuse
    invalid_semantic_graph at buffers or state. State is exactly integer now, integer
    history_start, one buffer_<n> sequence per ordered buffer, then activations with
    the specified ord
- id: FR-038-AC-425
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. Paired defects select shape before reference
    kind, signatures/outcome before premise range, and range before buffers/state;
    a wrong dependencies set selects the earlier graph-shape invalid_semantic_graph
    at /dependencies. Two defective roots stored opposite structural order choose
    the structurally first root at the post-temporal plan step. The exact code, optional
    cause, primary pointer and root node locus are asserted for each control.
- id: FR-038-AC-426
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. A default-feature consumer iterates admitted
    plans and reads the specified typed root/type/clause keys, resolved root and entry
    structural content keys through structural_key, exact entry signatures and bodies,
    width/gap/counter/tick-unit/activation-capacity and exact rational event-rate/uncertainty
    plus ordered buffers/capacities through private-field borrowed views. Two plans
    sharing labels but differing selected keys remain distinct; empty plans yield
    an empty iterator. An entry or root content change is observed through the corresponding
    structural keys withou
- id: FR-038-AC-427
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. TC-048 reads QSpec proposals/checked-package-v2/monitor-plan-vectors.json
    MP-01..MP-77 by reference from the selected QSpec checkout, constructs complete
    selected packages around the MP-01..MP-31 and MP-33..MP-53 and MP-58..MP-65 root
    recipes (MP-74..MP-76 planned under AC-433) and separately exercises MP-32's flatten/sequence
    expression, and checks exact admission or refusal code, cause, primary RFC 6901
    pointer and structural locus. No upstream vector, schema or fixture is copied
    into IR. MP-54..MP-57, MP-66..MP-73 and MP-77 remain by-reference QSL/runtime
    execu
- id: FR-038-AC-428
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. MP-33 and MP-43..MP-45 admit zero uncertainty,
    changed counter, fractional rational rate and changed tick unit with distinct
    resolved root content keys; a capacity edit MP-02 also changes that key. MP-40
    and MP-41 refuse invalid_semantic_graph at tick_unit for a wrong node kind or
    non-Time unit by the exact base-Time identity test, never invalid-time-quantity.
    Exact rate and uncertainty are borrowed CheckedRational values with no floating
    conversion or narrowing. Tick intervals remain derivable from the selected clause
    and clock/tick-unit scales with no extra bind
- id: FR-038-AC-429
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. By-reference MP-48 and MP-49 refuse a missing
    on_gap or wrong reference kind as invalid_semantic_graph with no cause at /semantic_graph/nodes/{n}/body/members/15;
    MP-50..MP-52 refuse reordered or noninteger gap parameters and a wrong return
    type as ill_typed/type-mismatch at the same binding, retaining the root structural
    locus. MP-53 admits a changed on_gap node with a distinct resolved root content
    key. The mandatory borrowed on_gap entry view exposes exactly (State, integer
    gap) and (State, Results), its whole checked body and resolved entry key; no Option/defa
- id: FR-038-AC-430
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. By-reference MP-58 admits activation_capacity
    0 with a distinct root content key and the borrowed canonical nonnegative decimal
    getter returns "0" without narrowing. MP-59 refuses a negative value as invalid_package/invalid-value
    at /semantic_graph/nodes/{n}/body/members/16/value; MP-60 and MP-61 refuse absent
    or noninteger activation capacity as invalid_semantic_graph with no cause at member
    16. MP-62..MP-65 refuse missing history_start or activations, missing origin in
    an activation record and a noninteger history_start as invalid_semantic_graph
    with no cause at
- id: FR-038-AC-431
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: planned until IR-1265 lands. The public strict reader admits QSpec `FM-01`
    and `FM-02` as generic `application` terms whose operator is `call`, identity
    is `quire.op.ieee.float64.fma` or `quire.op.ieee.float32.fma`, and three `reference`
    arguments name the same-width operands in a, b, c order. The result has that width;
    the operation carries one selected `ieee_profile` law and a `rounding` mode, with
    no member or leaves. No dedicated FMA node or callee argument is required.
- id: FR-038-AC-432
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: 'planned until IR-1265 lands. The public strict reader checks QSpec `FM-03`
    through `FM-12`: wrong arity, wrong or mixed operand width, an integer operand,
    or wrong-width result refuse `ill_typed`/`operator-ineligible`; absent rounding
    mode refuses `invalid_package`/`operation-mode-mismatch`; absent `ieee_profile`
    law refuses `invalid_package`/`operation-law-missing`. TC-048 reads the exact
    `FM-01` through `FM-12` case set from QSpec `proposals/checked-package-v2/fma-application-vectors.json`
    by reference and compares admission or code and cause on complete IR-owned packages,
    without storing a '
- id: FR-038-AC-433
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-721 lands. The exact monitor State is now, history_start,
    the ordered buffer_<n> fields, activations, then integer event_position. By-reference
    QSpec MP-74..MP-76 refuse a missing event_position, text-typed event_position
    or placement before activations as invalid_semantic_graph with no cause at /semantic_graph/nodes/{n}/body/members/1,
    retaining the root structural locus. Updated positive State contexts admit with
    no added root binding or public getter. MP-77, MP-57 and MP-69 remain by-reference
    QSL/runtime controls for init zero, one increment per admitted event including
    ba
- id: FR-038-AC-434
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'planned until IR-721 lands. TC-048 reads QSpec entry-key-vectors.json
    EK-01..EK-08 by reference: the published declaration-less function path has parameter
    count, ordered parameter indices, body-root index and optional measure index;
    equal content gives equal paths and a changed body, reordered parameter or measure
    gives a different path. Complete package controls admit genuine declaration-less
    pure_function bodies with exactly ordered parameters/body and an optional final
    measure, including EK-06 with typed parameter references, body-root reference
    and measure reference. Negative controls rej'
- id: FR-019-AC-23
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: 'Implemented; evidence: `default_feature_consumer_reads_an_actual_admitted_monitor_plan`
    and the monitor-view compile-fail doctests. A default-feature external consumer
    imports CheckedMonitorPlanRef, CheckedMonitorEntryRef and CheckedMonitorBufferRef
    and uses CheckedPackageV2::monitor_plans and the exact typed accessor signatures
    of FR-038, including the nonnegative decimal activation_capacity getter. Compile
    controls show no public view constructor, mutable field, unchecked raw-to-admitted
    conversion or borrow escaping the selected package lifetime. The implemented inventory
    row participates i'
bindings:
- test_id: monitor_plan_published_reader_recipes_have_exact_outcomes
  ac_id: FR-038-AC-421
  trace: correct
- test_id: monitor_plan_mp32_uses_existing_flatten_and_sequence_operations
  ac_id: FR-038-AC-421
  trace: correct
- test_id: monitor_plan_published_reader_recipes_have_exact_outcomes
  ac_id: FR-038-AC-422
  trace: correct
- test_id: monitor_plan_adjacent_stages_and_premises_keep_first_refusal
  ac_id: FR-038-AC-422
  trace: correct
- test_id: monitor_plan_owner_source_map_and_wire_defects_keep_precedence
  ac_id: FR-038-AC-422
  trace: correct
- test_id: monitor_plan_empty_body_refuses_at_first_required_binding
  ac_id: FR-038-AC-422
  trace: correct
- test_id: monitor_plan_published_reader_recipes_have_exact_outcomes
  ac_id: FR-038-AC-423
  trace: correct
- test_id: monitor_plan_adjacent_stages_and_premises_keep_first_refusal
  ac_id: FR-038-AC-423
  trace: correct
- test_id: monitor_plan_entry_owner_exception_cannot_admit_a_model_descriptor
  ac_id: FR-038-AC-423
  trace: correct
- test_id: monitor_plan_published_reader_recipes_have_exact_outcomes
  ac_id: FR-038-AC-424
  trace: correct
- test_id: monitor_plan_adjacent_stages_and_premises_keep_first_refusal
  ac_id: FR-038-AC-424
  trace: correct
- test_id: monitor_plan_state_layout_rejects_order_extra_and_wrong_activation_types
  ac_id: FR-038-AC-424
  trace: correct
- test_id: monitor_plan_published_reader_recipes_have_exact_outcomes
  ac_id: FR-038-AC-425
  trace: correct
- test_id: monitor_plan_adjacent_stages_and_premises_keep_first_refusal
  ac_id: FR-038-AC-425
  trace: correct
- test_id: monitor_plan_exact_work_limit_admits_and_one_less_has_no_partial_package
  ac_id: FR-038-AC-425
  trace: correct
- test_id: monitor_plan_root_order_and_content_changes_use_selected_keys
  ac_id: FR-038-AC-425
  trace: correct
- test_id: monitor_plan_rational_work_denial_stays_incomplete
  ac_id: FR-038-AC-425
  trace: correct
- test_id: monitor_plan_owner_source_map_and_wire_defects_keep_precedence
  ac_id: FR-038-AC-425
  trace: correct
- test_id: monitor_plan_entry_owner_exception_cannot_admit_a_model_descriptor
  ac_id: FR-038-AC-425
  trace: correct
- test_id: anonymous_entry_support_keeps_model_and_source_owner_requirements
  ac_id: FR-038-AC-425
  trace: correct
- test_id: monitor_plan_published_reader_recipes_have_exact_outcomes
  ac_id: FR-038-AC-426
  trace: correct
- test_id: monitor_plan_exact_reads_allow_unbounded_capacities_and_opaque_status
  ac_id: FR-038-AC-426
  trace: correct
- test_id: monitor_plan_resolved_root_and_four_entry_keys_survive_graph_permutation
  ac_id: FR-038-AC-426
  trace: correct
- test_id: monitor_plan_exact_work_limit_admits_and_one_less_has_no_partial_package
  ac_id: FR-038-AC-426
  trace: correct
- test_id: monitor_plan_root_order_and_content_changes_use_selected_keys
  ac_id: FR-038-AC-426
  trace: correct
- test_id: monitor_plan_absence_is_an_empty_exact_size_iterator
  ac_id: FR-038-AC-426
  trace: correct
- test_id: monitor_plan_published_reader_recipes_have_exact_outcomes
  ac_id: FR-038-AC-427
  trace: correct
- test_id: monitor_plan_mp32_uses_existing_flatten_and_sequence_operations
  ac_id: FR-038-AC-427
  trace: correct
- test_id: monitor_plan_adjacent_stages_and_premises_keep_first_refusal
  ac_id: FR-038-AC-428
  trace: correct
- test_id: monitor_plan_exact_reads_allow_unbounded_capacities_and_opaque_status
  ac_id: FR-038-AC-428
  trace: correct
- test_id: monitor_plan_resolved_root_and_four_entry_keys_survive_graph_permutation
  ac_id: FR-038-AC-428
  trace: correct
- test_id: monitor_plan_root_order_and_content_changes_use_selected_keys
  ac_id: FR-038-AC-428
  trace: correct
- test_id: monitor_plan_rational_work_denial_stays_incomplete
  ac_id: FR-038-AC-428
  trace: correct
- test_id: monitor_plan_tick_unit_uses_the_admitted_model_dimension_identity
  ac_id: FR-038-AC-428
  trace: correct
- test_id: monitor_plan_published_reader_recipes_have_exact_outcomes
  ac_id: FR-038-AC-429
  trace: correct
- test_id: monitor_plan_exact_reads_allow_unbounded_capacities_and_opaque_status
  ac_id: FR-038-AC-429
  trace: correct
- test_id: monitor_plan_resolved_root_and_four_entry_keys_survive_graph_permutation
  ac_id: FR-038-AC-429
  trace: correct
- test_id: monitor_plan_root_order_and_content_changes_use_selected_keys
  ac_id: FR-038-AC-429
  trace: correct
- test_id: monitor_plan_adjacent_stages_and_premises_keep_first_refusal
  ac_id: FR-038-AC-430
  trace: correct
- test_id: monitor_plan_exact_reads_allow_unbounded_capacities_and_opaque_status
  ac_id: FR-038-AC-430
  trace: correct
- test_id: monitor_plan_state_layout_rejects_order_extra_and_wrong_activation_types
  ac_id: FR-038-AC-430
  trace: correct
- test_id: monitor_plan_root_order_and_content_changes_use_selected_keys
  ac_id: FR-038-AC-430
  trace: correct
- test_id: monitor_plan_published_reader_recipes_have_exact_outcomes
  ac_id: FR-038-AC-433
  trace: correct
- test_id: monitor_plan_event_position_old_shape_is_refused
  ac_id: FR-038-AC-433
  trace: correct
- test_id: monitor_plan_genuine_measure_entry_keeps_signature_and_closed_body_checks
  ac_id: FR-038-AC-433
  trace: correct
- test_id: monitor_plan_event_position_is_the_exact_final_integer_field
  ac_id: FR-038-AC-433
  trace: correct
- test_id: entry_keys_published_paths_and_admission_cover_optional_measure
  ac_id: FR-038-AC-434
  trace: correct
- test_id: entry_keys_measure_function_is_admitted_without_a_monitor_root
  ac_id: FR-038-AC-434
  trace: correct
- test_id: entry_keys_assigned_label_is_resolved_without_recomputation
  ac_id: FR-038-AC-434
  trace: correct
- test_id: entry_keys_malformed_measure_and_descriptor_bindings_keep_owner_refusals
  ac_id: FR-038-AC-434
  trace: correct
- test_id: monitor_plan_genuine_measure_entry_keeps_signature_and_closed_body_checks
  ac_id: FR-038-AC-434
  trace: correct
- test_id: monitor_plan_absence_is_an_empty_exact_size_iterator
  ac_id: FR-019-AC-23
  trace: correct
```
