---
id: SR-202611070951309548
title: PR992 final derived-shape code-review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; IR-1404; structural.rs
review_set: subset
---

## Summary

Ticket: IR-1404. Independently reviewed frozen PR992 at dac7b474 against main05277256. The only production change replaces the final invalid-derived-shape branch's eager owned JsonPointer and node clone with the existing borrowed stage Path, Site and charged Refusal builder. Preserved the Map root path, bounded-quantity root or noncanonical endpoint path, and other node_id path. All shape predicates, iteration and earlier refusal order, canonical endpoint inspection and logical work are unchanged. The actual caller WorkMeter reserves the complete record, location, pointer and node-copy allocation before construction and records the final paid/pending refusal identity. Inspected the fixed-size borrowed path capacity (the longest new path is seven steps), complete diagnostic sizing/building, and denial propagation. Adjacent text-bound and selected-bound raw factories at the earlier source locations are unchanged and outside this introduced-fault review. No public API, trait, budget, pointer algorithm, numeric semantic rewrite, dependency, fixture copy, ignored test or gate change is introduced. No applicable AssuranceProfile was found under spec/.

## Verdict

PASS for this changed final derived-shape refusal family: no introduced findings. Whole-corpus AC511, AC512 qualification and AC513 remain UNPROVEN. Existing repository-wide gaps remain open.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Retained selected Rust1.98.1 workspace native test PASS1 (compile22.05s/test<0.01s): actual validate_derived_type_shapes for population node_id, map root, bounded-quantity root and bounded endpoint paths. It checks all measured allocation requests against reservations, ample/exact/one-short/zero/repeated limits, complete fault code/cause/path/node, paid and pending identity, Graph diagnostic locus and unchanged zero work. Zero/short direct diagnostic attempts allocate nothing. The unchanged nonignored tc_229_population_reference_requires_owner_and_selected_population PASS1 (0.60s) retains its physical-accounting and semantic assertions, including the formerly failing third wrong-dependencies read. Retained workspace/all-target Clippy conformance-cli,test-support -D warnings PASS16.38s; scoped formatting and diff checks pass. Actual per-component exit0 receipts are /tmp/ir1404-structural-fault-paid-receipts.json and logs /tmp/ir1404-structural-fault-paid-{native,original,clippy}.log. No Cargo, Kani, make ci, whole-binary or aggregate run was started by this reviewer; the user requires bounded selections and post-merge fixes forward.

Computed full Quire0.38.1 matrix inspected: tagged1028, untagged92, tagged-by-ignored-test8 and method-without-symbol26. Existing repository-wide gaps remain open and this scoped review does not claim repository-wide assurance PASS. Existing FR-038 derived-shape, population fixed-member and budgeted-read requirements own the changed behavior. The new AC511 tag is truthful partial request-accounting evidence; four local refusal variants plus one original fixture are not whole-read corpus closure, AC512 qualification or AC513 proof. No new unknown criterion tag, stub, fake provider, weakened assertion, ignored case, coverage threshold reduction or unowned seam was found. The original integration test is unchanged. No spec or plan changed, so spec-review methods do not apply. Optional broad semantic audit was not requested; ordinary changed-source/test/spec agreement was examined.

Plan completion: not assessed

Full reviewed scope:

```yaml
scope:
- id: validate_derived_type_shapes
  path: crates/quire-contract-model/src/checked_package/v2/structural.rs
  role: examined
  excerpt: if !valid { let node_path = Path::root().key("semantic_graph").key("nodes").index(position);
    ... stage_storage::refuse(meter, &Site::new(...), Refusal::new(InvalidPackage,
    at).because(InvalidDerivedShape).at_node(node_id)) }
- id: noncanonical_bounded_endpoint
  path: crates/quire-contract-model/src/checked_package/v2/structural.rs
  role: context_only
  excerpt: 'fn noncanonical_bounded_endpoint(term: Option<&CheckedTerm>, meter: &mut
    WorkMeter) -> Result<Option<usize>, ValidationFailure>'
- id: derived_shape_faults_prepay_the_complete_diagnostic
  path: crates/quire-contract-model/src/checked_package/v2/structural.rs
  role: examined
  excerpt: fn derived_shape_faults_prepay_the_complete_diagnostic()
- id: stage_storage::Refusal
  path: crates/quire-contract-model/src/checked_package/v2/stage_storage.rs
  role: context_only
  excerpt: 'The heap the refusal owns: the boxed record, its one location, the pointer
    and the located node''s clone.'
- id: tc_229_population_reference_requires_owner_and_selected_population
  path: tests/it/checked_package_v2_owners.rs
  role: examined
  excerpt: fn tc_229_population_reference_requires_owner_and_selected_population()
- id: FR-038-AC-124
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-1141). Each `scalar_type`/`boolean` or `/integer` body
    is the empty aggregate; `reference`, `option`, `set`, `bag`, `sequence` and `ordered_set`
    have exactly one reference; `integer_range` and `collection_bounds` have ordered
    `min` and `max` integer-literal bindings of their specified decimal grammar. A
    missing, extra, reordered, wrongly typed or malformed member refuses `invalid_package`/`invalid-derived-shape`
    at that node without using a content-derived key. For `composite_type`/`reference`,
    a valid body target must be a model `object_type`, `process`, `state_machine`
    or `
- id: FR-038-AC-125
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-1141). A `scalar_type` or `composite_type` in those ten
    shapes is self typed; an `integer_range` is typed at the exact Integer node and
    `collection_bounds` at a collection node. A wrong semantic type or a bound literal
    typed at Boolean refuses `invalid_package`/`invalid-derived-shape` at the offending
    node; a valid structural id alone does not bypass the body/type check.
- id: FR-038-AC-303
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-1141). A self-typed relation/population node with no
    declaration or recursion group, an empty aggregate body, empty dependencies and
    a ModelOwner admits under its structural id. Its owner joins the selected model
    declaration. An absent owner refuses malformed_wire before joins, and a wrong
    fixed member refuses by the model-declaration body-shape rule; no nominal preimage
    or key recomputation occurs.
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
