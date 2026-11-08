---
id: SR-202611070951309553
title: PR995 bare population fault gap-analysis
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; IR-1404; model_members.rs; v2/mod.rs; model_members/tests/budgeted_charge.rs
review_set: subset
---

## Summary

Computed full Quire0.38.1 matrix inspected: method-without-symbol=26, tagged=1027, tagged-by-ignored-test=8, untagged=92. Existing repository-wide gaps remain open; this subset review grants no repository-wide assurance PASS. The new test binds TC-229, AC304 and AC511 and exercises genuine malformed selected-model records through the real admission/projection path. These are truthful partial semantic and request-accounting bindings, not whole-criterion or whole-corpus completion. No new unknown criterion tag, stub, mock provider, skipped case, weakened original assertion, lowered threshold or unowned product seam was found. Located document-pointer, missing-supplied-document, InexactNumber and other general From routes remain outside this slice and unqualified. AC511 whole-corpus, AC512 qualification and AC513 remain UNPROVEN. No spec or plan changed, so spec-review methods do not apply; optional broad semantic audit was not requested. Ordinary changed-source/test/spec agreement was examined.

## Verdict

PASS for this bare population carrier and public-projection slice: no introduced findings. Existing global gaps and whole-corpus AC511/AC512/AC513 qualification remain open.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Retained selected Rust1.98.1 workspace native test PASS1 (compile22.31s/test<0.01s): actual validate_domain_packages includes admission and public projection for missing identity, forbidden node, unresolved construct, wrong meaning, duplicate population, invalid member list and invalid member element. It compares every allocation request with admitted read storage, checks semantic code/cause/path and absence of private metadata, and covers ample/exact/repeat, private-box-short, public-record-short and zero allowances, pending identity, ModelDocument locus, unchanged work and no premature Paid. The unchanged nonignored tc_229_population_with_node_member_refuses_at_selection PASS1 (0.09s) retains physical and semantic assertions. Workspace/all-target Clippy conformance-cli,test-support -D warnings PASS16.92s; scoped formatting and diff checks pass. Logs are /tmp/ir1404-model-selection-fault-{native,original,clippy,fmt-check}.log. No Cargo, Kani, make ci, whole-binary, deferred-case or aggregate run was started by this reviewer; retained focused evidence was inspected under the user's post-merge fix-forward rule.

Ticket: IR-1404. Independently reviewed frozen PR995 at 9aebda16 against main161d2516. The selected-model population reader now pays the private SelectionRefusal box through its existing Budget before allocation at six logical bare-refusal conversion sites. The meaning check remains before the second retained-key payment; duplicate insertion and all other validation and work order remain unchanged. On private allocation denial the existing actual reader budget receives the final pending refusal code; this private phase never sets the public paid marker. The public bare/no-document-pointer projection ends the model Budget borrow, retains private metadata disposal, and uses the existing borrowed owner/member Path and stage-storage Refusal to reserve the complete public record, single location and pointer before construction. Only that successful public reservation sets Paid. Inspected Budget.request/reader_budget, stage-storage reservation and diagnostic sizing/building, all conversion sites, zero/short denial propagation, and unaffected located/InexactNumber branches. General From conversions outside this strict population slice remain unchanged and explicitly unqualified. No public API, trait, budget, pointer algorithm, dependency, fixture copy, ignored test or gate change is introduced. No applicable AssuranceProfile was found under spec/.

Plan completion: not assessed

Full reviewed scope:

```yaml
scope:
- id: SelectionFailure::bare_charged
  path: crates/quire-contract-model/src/checked_package/v2/model_members.rs
  role: examined
  excerpt: 'fn bare_charged(refusal: ModelRefusal, budget: &mut Budget<''_, ''_>)
    -> Self'
- id: read_semantic_ir population conversions
  path: crates/quire-contract-model/src/checked_package/v2/model_members.rs
  role: examined
  excerpt: "let node = text(population, \"identity\")\n            .map_err(|refusal|\
    \ SelectionFailure::bare_charged(refusal, budget))?;"
- id: validate_domain_packages
  path: crates/quire-contract-model/src/checked_package/v2/mod.rs
  role: examined
  excerpt: "fn validate_domain_packages(\n    roots: &[(&str, usize)],\n    checked_package:\
    \ &str,\n    evidence: &CheckedPackageEvidence,\n    meter: &mut WorkMeter,\n\
    \    bytes: u64,\n    unit_refs: &UnitRefs,\n) -> Result<(Vec<DomainModel>, Vec<Arc<quire_canonical::Document>>),\
    \ ValidationFailure>"
- id: bare_population_faults_prepay_the_private_carrier_and_public_diagnostic
  path: crates/quire-contract-model/src/checked_package/v2/model_members/tests/budgeted_charge.rs
  role: examined
  excerpt: fn bare_population_faults_prepay_the_private_carrier_and_public_diagnostic()
- id: run_population_fault
  path: crates/quire-contract-model/src/checked_package/v2/model_members/tests/budgeted_charge.rs
  role: examined
  excerpt: 'fn run_population_fault(document: &Value, allowance: u64) -> PopulationFaultOutcome'
- id: stage_storage::refuse
  path: crates/quire-contract-model/src/checked_package/v2/stage_storage.rs
  role: context_only
  excerpt: 'The refusal, its owned diagnostic charged to the budget before any of
    it is

    /// built.'
- id: tc_229_population_with_node_member_refuses_at_selection
  path: tests/it/checked_package_v2_owners.rs
  role: examined
  excerpt: fn tc_229_population_with_node_member_refuses_at_selection()
- id: FR-038-AC-304
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'After FR-154 admission, the reader joins a population reference''s `owner.node`
    to the raw `identity` of a `populations` record in the document selected by `owner.identity`.
    A record without an `identity` string, with a `node` member, with an unresolved
    `kind` construct, or with a construct whose meaning is not population refuses
    `invalid_model_binding`/`malformed-declaration` at the selection before the join.
    The QSpec `counters` and `archive` owners admit; an unknown node, an object-type
    node claimed as a population, and a population node claimed as an object type
    or relationship each refuse '
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
