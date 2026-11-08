---
id: SR-202611070951309547
title: PR991 temporal placement gap-analysis
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; IR-1404; temporal.rs
review_set: subset
---

## Summary

Selected Quire0.38.1 computed full matrix inspected: tagged1028, untagged92, tagged-by-ignored-test8 and method-without-symbol26. Existing repository-wide gaps remain open and are not claimed resolved. Changed production behavior is owned by the existing FR-038 temporal-placement and budgeted-read contracts. The new native AC510/AC511 bindings are truthful partial evidence for diagnostic denial identity and request accounting; they do not demonstrate every top-level outcome mapping or whole-read corpus. No unknown criterion tag, fake provider, stub, weakened assertion, new ignored case, coverage-threshold change or unowned seam was found. The retained original read exercises the actual strict reader. The changed hyper call reuses the paid diagnostic builder but this PR does not claim whole-hyper allocation closure. No spec or plan changed, so spec-review methods do not apply. Optional broad semantic audit was not requested; ordinary scoped source/test/spec agreement was examined.

## Verdict

PASS for this narrow temporal placement slice: no introduced findings. Whole-corpus AC511, AC512 qualification and AC513 remain UNPROVEN.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Retained selected Rust1.98.1 workspace native test PASS1 (compile21.96s/test<0.01s): five genuine decoded-wire placement routes, with actual allocated request bytes equal to reservations, ample/exact/one-short/zero/repeated limits, complete fault and budget-state comparisons, pending/paid final code, original holder locus and unchanged logical work. The zero limit may stop a reference walk before its eventual placement fault is known; the native test correctly does not invent pending-refusal metadata for that earlier stop. The unchanged full catalog root-placement test PASS1 (0.54s) exercises case, temporal_formula and temporal_fairness with physical and semantic assertions; all three retained sample rows explicitly say NOT_QUALIFIED/ac_512_credit false. This is composed with merged operations fix989. Retained workspace/all-target Clippy conformance-cli,test-support -D warnings PASS16.42s; scoped formatting/diff checks pass. Logs are /tmp/ir1404-temporal-placement-{native,original,clippy}.log and the original three rows are /tmp/ir1404-temporal-placement-original-rows.jsonl. No Cargo, Kani, make ci, whole-binary or aggregate was run by this reviewer; the user requires bounded selections and fixes forward after merge.

Ticket: IR-1404. Independently reviewed frozen PR991 at 75e65f0d against main05277256. The one-file change routes the existing temporal placement/ineligible diagnostic through temporal_fault and the real caller WorkMeter. The private walk stop carries no diagnostic while the meter is borrowed; its caller constructs the same original-node fault only after the walk returns and the existing stage-storage builder has reserved the complete record, location, pointer and node copy. Root placement and the existing hyper call pass the same meter. Inspected first-fault predicates, monitor-plan exemption, structural order, reference-slot rules, walk-denial propagation, exact code/cause/path/node semantics and reservation paid/pending identity. Integer/Quantity arms and hyper semantics are unchanged. No new public API, trait, budget, pointer algorithm, dependency, schema, fixture copy, ignore or gate is introduced. No applicable AssuranceProfile was found under spec/.

Plan completion: not assessed

Full reviewed scope:

```yaml
scope:
- id: temporal::ineligible
  path: crates/quire-contract-model/src/checked_package/v2/temporal.rs
  role: examined
  excerpt: temporal_fault(meter, position, CheckedPackageRefusalCode::IllTyped, CheckedPackageRefusalCause::OperatorIneligible,
    borrowed node path, node id)
- id: temporal::placement_defect
  path: crates/quire-contract-model/src/checked_package/v2/temporal.rs
  role: examined
  excerpt: ControlFlow::Break(()) => Some(ineligible(graph, position, meter))
- id: temporal::Placement
  path: crates/quire-contract-model/src/checked_package/v2/temporal.rs
  role: examined
  excerpt: type Stop = ();
- id: temporal::check_hyper_shape
  path: crates/quire-contract-model/src/checked_package/v2/temporal.rs
  role: examined
  excerpt: return Err(ineligible(graph, position, meter));
- id: placement_faults_prepay_diagnostics_at_exact_short_zero_and_repeated_limits
  path: crates/quire-contract-model/src/checked_package/v2/temporal.rs
  role: examined
  excerpt: fn placement_faults_prepay_diagnostics_at_exact_short_zero_and_repeated_limits()
- id: WorkMeter::walk
  path: crates/quire-contract-model/src/checked_package/v2/graph_storage.rs
  role: context_only
  excerpt: 'pub(super) fn walk<W: quire_walk::Walk + ?Sized>'
- id: FR-038-AC-102
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The temporal step runs after the frame and state-clause step and before
    the operation step, placement first: a package holding one node with a placement
    defect (a `temporal_formula` application as the body root of a `function` node)
    and a second with a earlier structural `node_id` whose operation identity is unknown
    refuses for the placement defect, and so does the same placement defect beside
    a earlier-structural-id node whose `operator` differs from its entry''s class
    (`quire.op.control.case` under `unary`), in both structural id orders each, the
    placement defect taken in the shape of a `temp'
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
