---
id: SR-202611070951309542
title: PR989 root case diagnostic code-review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; IR-1404; operations.rs
review_set: subset
---

## Summary

Ticket: IR-1404. Independently reviewed frozen PR989 at 814e58d9. The existing caller WorkMeter now reaches the private first-pass case-placement factory. Its borrowed Application path and existing stage-storage Refusal builder reserve the pointer, boxed refusal, location and located node copy before construction. Inspected all three fault predicates and both no-fault outcomes, caller early-return behavior for budget stops, unchanged structural traversal and temporal-before-operations ordering, exact pointer/cause/node semantics, denial paid/pending metadata and zero-work behavior. No public API, new adapter, private budget, copied pointer algorithm, fixture copy, gate change or spec change is introduced. No applicable AssuranceProfile was found under spec/.

## Verdict

PASS for this one-file paid case-placement slice: no introduced findings. This is not repository-wide assurance completion; whole-corpus AC511 and AC513 remain UNPROVEN.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Inspected retained selected Rust 1.98.1 workspace native test PASS (one actual test; compile 21.89s), covering all three fault arms and both no-fault arms with actual allocator bytes, ample/exact/one-short/zero/repeated budgets, final code/cause/path/node, paid/pending outcomes and unchanged work. Workspace/all-target Clippy with conformance-cli,test-support and -D warnings passed in 27.45s; scoped formatting and diff checks passed. The unchanged original root-placement test remains RED: its first case driver completed the physical and semantic checks (one retained sample row, sequence 0), then the next temporal_formula read failed with 522 unpaid bytes from the separate temporal ineligible factory before operations. This review does not call the whole original test passing, and no ignore/assertion was weakened. The user mandates bounded selections, no whole-binary include-ignored run, no make ci/Kani and post-merge fixes forward; no Cargo or aggregate was run by this reviewer.

The selected Quire 0.38.1 computed full matrix was inspected: 1028 tagged, 92 untagged, 8 tagged-by-ignored-test and 26 method-without-symbol criteria. These repository-wide existing gaps remain open and are not claimed resolved by this PR. The changed source is owned by FR-038 case-placement and budgeted-read contracts. The native trace bindings to AC100, AC510 and AC511 are truthful partial evidence for placement behavior, denied diagnostic identity and allocation accounting respectively; they do not prove whole-read coverage or every top-level mapped outcome. No stubs, fake providers, new ignored tests, threshold reduction, unknown criterion tags or extra obligation are introduced. Optional full semantic-audit expansion was not requested; ordinary scoped code/spec faithfulness was checked. No spec/plan changed, so spec-review methods do not apply.

Plan completion: not assessed

Full reviewed scope:

```yaml
scope:
- id: operations::validate_operations
  path: crates/quire-contract-model/src/checked_package/v2/operations.rs
  role: examined
  excerpt: if let Some(failure) = case_placement_defect(position, &graph, meter)
- id: operations::case_placement_defect
  path: crates/quire-contract-model/src/checked_package/v2/operations.rs
  role: examined
  excerpt: let mut refusal = super::stage_storage::Refusal::new(code, path).at_node(&node.node_id);
- id: operations::tests::case_placement_faults_prepay_paths_and_node_copies_before_allocation
  path: crates/quire-contract-model/src/checked_package/v2/operations.rs
  role: examined
  excerpt: fn case_placement_faults_prepay_paths_and_node_copies_before_allocation()
- id: FR-038-AC-100
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: a `case` application refuses `ill_typed`/`operator-ineligible` at the node
    that holds it when it is the body root of a node that is not an `expression` node,
    at the operation step and so after every temporal-step defect whatever the structural
    id order (merged FR-440 join 1; not in the temporal placement pass), and at its
    own `operator` (`/semantic_graph/nodes/{n}/body/.../operator`) when it is nested
    in another term (merged FR-322 "Body grammar"); an `expression` node whose `semantic_form`
    contradicts its root application's operator class (an `expression`/`case` node
    whose body is not a `case
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
  excerpt: 'Whole-read accounting, including each read''s owned catalog. A test binary
    whose global allocator counts, on the reading thread during the call, the size
    of each allocation request and the new size of each reallocation request reads,
    under an unlimited budget, an exact budget and each one-below budget of AC-497
    through AC-507: every positive TC-048 fixture, the QSpec checked-package conformance
    packages this repository reads, and the adverse shapes — the 100000-node reference
    chain, a 10000-node flat graph, a node with 10000 dependencies, a 1 MiB string
    member, 10000 source-map entries, 10000 d'
- id: FR-038-AC-67
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: 'An unknown identity refuses `unknown-operation` and an `operator` that
    differs from the catalogued class refuses `operation-class-mismatch`, each ahead
    of every other check of the entry at the operation step, so `quire.op.control.case`
    under the operator `unary` refuses `operation-class-mismatch`; and of two defective
    body-root nodes the one with the earlier structural `node_id` is reported when
    both defects are operation refusals. A placement defect of the temporal step (FR-038-AC-102)
    is reported ahead of every operation refusal, whatever the structural id order.
    (Amended by IR-549: the earl'
```
