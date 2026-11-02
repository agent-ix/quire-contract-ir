---
id: SR-1292
title: IR-721 reader spec-review/scope-boundary
type: SpecReview
analysis: scope-boundary
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/model/functional/FR-019-rust-library-interface.md
review_set: subset
---

## Summary

Ticket: IR-721. Reader implementation activation and the default model API inventory preserve a clear allocation to QSpec, IR, QSL and runtime consumers.

## Verdict

**PASS** — Reader implementation activation and the default model API inventory preserve a clear allocation to QSpec, IR, QSL and runtime consumers.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

IR strict reader (core): guarantees closed shape/kinds/signatures/premises, selected model joins, borrowed access and canonical package roundtrip. QSpec is an external normative dependency consumed by reference and checked via reader contract tests. QSL producer semantics/capacity derivation and real producer closure are external assumptions not qualified here. RT modular deltas/execution and driver independently selected context/identity binding are downstream responsibilities, not reader guarantees. RT ProgramParts -> Program::try_new -> RcProgram remains the runtime construction seam; no executor-facing replacement or copied execution semantics is added. FR-038 and FR-019 each retain one owning IR component and core responsibility. No new external actor or dependency is created by status/inventory edits.

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
