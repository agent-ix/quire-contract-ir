---
id: SR-5501
title: gap-analysis of IR-7 PR 334 temporal refusal ordering
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir@138df4fe52bc49856ca078a8c9a76aa688f124dd; crates/quire-contract-model/src/checked_package/v2/mod.rs;
  crates/quire-contract-model/src/checked_package/v2/temporal.rs; tests/it/checked_package_v2_temporal.rs;
  FR-038-AC-100 diagnostic-reference clause; FR-038-AC-102 temporal-placement ordering
  clause
review_set: subset
---

## Summary

Ticket: IR-7. Reviewed only the three-path frozen PR diff. No defect found in the changed diagnostic-reference ordering or retained refusal location. Other IR-7 producer and redefinition gaps are outside this diff.

## Verdict

**PASS (PR-diff scope only)** — no findings in the changed behavior. This is not whole-ticket acceptance, whole-repository assurance, or a green full gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

Reconciliation: quire matrix (quire 0.36.2, engine 0.50.2) — no run evidence store read. Whole-repository static context: 314 tagged, 45 untagged, 2 tagged-by-ignored-test, 6 method-without-symbol, 367 total. Those pre-existing whole-repository gaps are outside this diff. FR-038-AC-100 and FR-038-AC-102 are tagged, each with ten binders; the added test binds both, and the expanded placement test binds AC-102. Scope examines only the excerpts below, not the full acceptance criteria.

Changed behavior inventory: one existing diagnostic-reference placement rule moved into temporal validation; zero new public APIs, untraced changed behaviors, source stubs or test stubs. The old diagnostic rule is removed rather than duplicated. The scan uses the already decoded node kinds, the shared iterative term walker and existing typed refusal/pointer constructors. Byte intake bounds the package, and existing diagnostic work/limits remain in diagnostic validation. No unsafe, casts, async, locks, compatibility machinery, vendoring, lints or CI workflows are introduced or weakened. No applicable AssuranceProfile found under spec/.

Dedicated optional gap semantic fan-out: skipped; code-review independently checks the changed test oracles and their binding strength as requested. The existing aggregate and focused evidence are described below; no new full test or gate run was performed.

Tool diagnostics: static tools emitted semantic.inline-data-schema, DuplicateArchetype ADR/Plan/Review/SpecReview/Standard (first-wins) and DuplicateInverseEdge (first-wins). These are reported as tool context, not concealed or attributed to this three-file change. Coverage reports zero untracked symbols; its declarations also report three archetype-matches-nothing, six uncatalogued-verification-method and one catch-all-universal diagnostic. The reviewed two criteria are present and bind to the real tests; these broader declaration diagnostics limit any whole-repository claim.

## Test Oracle Evidence

Existing evidence read, with no builds or gates run by this reviewer: candidate make ci exits 2 at strict coverage. The authenticated continuation records candidate coverage exit 1, 43 baseline and candidate unbacked rows with the same full row set and zero contradictions, then deny, one-copy, audit and unsafe checks all exit 0. Candidate formatting, workspace lint, 443 integration tests, 191 model unit tests, 24 doc tests, corpus and 576/576 document validation passed; no ignored execution tests. This is not CI green. No Kani/replay run is claimed.

The separate focused source counterfactual at f2391523a879f910d4d28f047759c650eeebdf76 reports exit 101: removing the early diagnostic scan yields UnknownProfile/UnsupportedSelection at the clause definition with a node locus, instead of IllTyped/OperatorIneligible at /diagnostics/entries/0/details/0 with no locus. Restoration passes the same test, exit 0. I independently compared the three reviewed paths at that focused revision and this frozen revision. This demonstrates an adverse source change can fail the new oracle; it is separate from the candidate gate evidence.

## Examined Scope

```yaml
scope:
- id: FR-038-AC-100
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: a `diagnostics.entries[].details[]` term that references a `temporal`/`formula`,
    `temporal`/`fairness` or `expression`/`case` node refuses `ill_typed`/`operator-ineligible`
    at that entry (`/diagnostics/entries/{e}/details/{d}`; the `case` node reference
    is merged FR-370-AC-12)
- id: FR-038-AC-102
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The temporal step runs after the frame and state-clause step and before
    the operation step, placement first:'
- id: validate_graph::temporal-call
  path: crates/quire-contract-model/src/checked_package/v2/mod.rs
  role: examined
  excerpt: '&wire.diagnostics.entries,'
- id: validate_temporal::ordering
  path: crates/quire-contract-model/src/checked_package/v2/temporal.rs
  role: examined
  excerpt: '    validate_diagnostic_references(nodes, kinds, diagnostics)?;'
- id: validate_diagnostic_references::iteration
  path: crates/quire-contract-model/src/checked_package/v2/temporal.rs
  role: examined
  excerpt: "    for (entry_index, entry) in diagnostics.iter().enumerate() {\n   \
    \     for (detail_index, detail) in entry.details.iter().enumerate() {"
- id: forbidden_diagnostic_reference_precedes_unknown_temporal_profile
  path: tests/it/checked_package_v2_temporal.rs
  role: examined
  excerpt: '/// Forbidden diagnostic references precede the clause''s unknown profile,

    /// retaining the detail pointer and no node locus.'
- id: tc_048_a_placement_defect_is_reported_ahead_of_every_other_defect
  path: tests/it/checked_package_v2_temporal.rs
  role: examined
  excerpt: '    // Graph placement also precedes a forbidden diagnostic reference.'
bindings:
- test_id: forbidden_diagnostic_reference_precedes_unknown_temporal_profile
  ac_id: FR-038-AC-100
  trace: correct
- test_id: forbidden_diagnostic_reference_precedes_unknown_temporal_profile
  ac_id: FR-038-AC-102
  trace: correct
- test_id: tc_048_a_placement_defect_is_reported_ahead_of_every_other_defect
  ac_id: FR-038-AC-102
  trace: correct
```
