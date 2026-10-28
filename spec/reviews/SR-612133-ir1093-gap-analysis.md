---
id: SR-612133
title: IR-1093 PR 659 protocol catalog gap analysis
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 659; FR-038-AC-329, crates/quire-contract-model/src/checked_package/v2/refinement.rs, tests/it/checked_package_v2_refinement.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Ticket: IR-1093. Planless PR-scoped requirement, tagged-test, and code audit. The changed tests cover complete clause path, authored member name, and fixed member-kind exclusion on nonoverlapping clause names, while leaving an inner control-ancestor spelling adverse case unasserted.

## Verdict

CONDITIONAL: the computed matrix tag for FR-038-AC-329 does not prove its control-ancestor refusal or nested selected-clause ownership clauses.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The AC-329 tests mutate `fork Example::OrderFlow::Holder` to include `run`, but never to include the intervening `Main` control ancestor. The criterion and TC-048 item 16 require refusal for both. Add a rekeyed `fork Example::OrderFlow::Main::Holder` adverse case with exact `invalid_semantic_graph` locus; include a prefix-related pair of selected clauses to catch the code-review finding. | tests/it/checked_package_v2_refinement.rs:4303 |

## Coverage

- FR-038-AC-329, examined: full selected clause path, authored member name, fixed kind exclusion, control ancestor exclusion, and wrong-clause refusal.
- FR-038-AC-325, context: complete inventory of selected model's admitted clauses.
- TC-048 item 16, examined: rekeyed adverse class spellings and exact refusal loci.
- Changed production code is owned by FR-038-AC-329; no new unowned feature or source stub found.
- Plan completion: not assessed.
