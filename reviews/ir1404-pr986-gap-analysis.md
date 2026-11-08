---
id: SR-202611070951309539
title: PR986 flat-wire diagnostic gap-analysis
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; IR-1404; flat_wire.rs; read_charge.rs; stage_storage.rs
review_set: subset
---

## Summary

Ticket: IR-1404. Scoped production-to-spec and computed-matrix analysis of this PR. The full selected Quire 0.38.1 matrix was inspected: 1028 tagged criteria, 92 untagged, 26 method-without-symbol and 8 tagged-by-ignored-test. These global unresolved rows are not declared fixed or qualified by this slice. The new actual native test is bound to FR-038-AC-510/511 and TC-472; the unchanged unknown-operator case binds to AC114. Reverse inspection finds the modified diagnostic behavior owned by FR-038 budgeted-read and grammar criteria. There are no new stubs, ignored tests, fake providers, threshold changes or trace tags purporting to close whole-read coverage. No spec or plan changed, so spec-review methods do not apply. Optional full semantic-audit expansion was not requested; ordinary code/spec faithfulness within the changed paths was checked.

## Verdict

PASS for the changed fault-family slice: no introduced findings. This is not repository-wide assurance completion; AC511 whole-corpus and AC513 remain UNPROVEN.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Inspected retained selected 1.98.1 workspace native test PASS (compile 21.64s; test <0.01s), three unchanged nonignored original catalog-word cases PASS (unknown operator 0.11s, placed class 1.29s, diagnostic detail 0.98s), workspace all-target Clippy with conformance-cli,test-support and -D warnings PASS (34.84s), scoped formatting and diff checks. The native test uses the real walk, native budget and allocator counter through both Meter and SinkCell; four actual grammar failures exercise complete code/cause/path/locus, exact bytes, zero/short denial, final paid/pending classes, and cumulative repeated requests. No Cargo or aggregate was run by this reviewer; the user requires bounded checks and fixes forward after merge. No public API, dependency, workflow, spec or gate changed. No applicable AssuranceProfile was found under spec/.

Scoped production-to-spec and computed-matrix analysis of this PR. The full selected Quire 0.38.1 matrix was inspected: 1028 tagged criteria, 92 untagged, 26 method-without-symbol and 8 tagged-by-ignored-test. These global unresolved rows are not declared fixed or qualified by this slice. The new actual native test is bound to FR-038-AC-510/511 and TC-472; the unchanged unknown-operator case binds to AC114. Reverse inspection finds the modified diagnostic behavior owned by FR-038 budgeted-read and grammar criteria. There are no new stubs, ignored tests, fake providers, threshold changes or trace tags purporting to close whole-read coverage. No spec or plan changed, so spec-review methods do not apply. Optional full semantic-audit expansion was not requested; ordinary code/spec faithfulness within the changed paths was checked.

Plan completion: not assessed

Full reviewed scope:

```yaml
scope:
- id: flat_wire::nested_application_refusal
  path: crates/quire-contract-model/src/checked_package/v2/flat_wire.rs
  role: examined
  excerpt: pub(super) fn nested_application_refusal
- id: flat_wire::Flat::outside_grammar
  path: crates/quire-contract-model/src/checked_package/v2/flat_wire.rs
  role: examined
  excerpt: fn outside_grammar(&self) -> ValidationFailure
- id: ChargeAt::reserve_diagnostic
  path: crates/quire-contract-model/src/checked_package/v2/read_charge.rs
  role: examined
  excerpt: Pays an owned outcome through the same caller-owned diagnostic sink.
- id: Refusal::build
  path: crates/quire-contract-model/src/checked_package/v2/stage_storage.rs
  role: examined
  excerpt: Builds only after the caller has admitted the complete diagnostic.
- id: stage_storage::refuse_charged
  path: crates/quire-contract-model/src/checked_package/v2/stage_storage.rs
  role: examined
  excerpt: A walk's refusal, paid through its existing caller-owned diagnostic sink
- id: tests::flat_faults_prepay_the_complete_final_diagnostic
  path: crates/quire-contract-model/src/checked_package/v2/flat_wire.rs
  role: examined
  excerpt: fn flat_faults_prepay_the_complete_final_diagnostic()
- id: FR-038-AC-114
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A node body in which an application of any operator class other than `case`
    stands as an element of another application's `arguments`, as a member of an `aggregate`
    or as the value of a `binding` refuses `malformed_wire` at that nested application,
    for `quire.op.function.call`, `quire.op.state.clause` and an application of each
    of the `temporal`, `temporal_formula` and `temporal_fairness` classes, in each
    of the three positions, at a pointer such as `/semantic_graph/nodes/{n}/body/arguments/0`;
    an `aggregate` inside a Group's members, a `binding` whose value is an `aggregate`
    inside an `aggreg
- id: FR-038-AC-510
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: On a budgeted entry, a work denial whose diagnostic is affordable returns
    `BudgetIncomplete` with `denied` `Work` and its amount, the budget's configured
    work limit, cumulative work plus the denied amount, and the value's pointer; a
    read-storage denial likewise returns `BudgetIncomplete` with `denied` `ReadStorage`.
    When the diagnostic's own request is denied, or the denied request was a refusal
    diagnostic, the read returns `Exhausted` holding the original denied resource
    and amount, its configured bound (including zero), its `PackageRead` locus and
    the pending outcome class (refusal code or s
- id: FR-038-AC-511
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "Whole-read accounting, including each read's owned catalog. A test binary\
    \ whose global allocator counts, on the reading thread during the call, the size\
    \ of each allocation request and the new size of each reallocation request reads,\
    \ under an unlimited budget, an exact budget and each one-below budget of AC-497\
    \ through AC-507: every positive TC-048 fixture, the QSpec checked-package conformance\
    \ packages this repository reads, and the adverse shapes \u2014 the 100000-node\
    \ reference chain, a 10000-node flat graph, a node with 10000 dependencies, a\
    \ 1 MiB string member, 10000 source-map entries, 10000 d"
```
