---
id: SR-14720
title: "IR-937 raw population record code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-937 raw PopulationRecord hotfix; crates/quire-contract-model/src/checked_package/v2/model_members.rs; tests/it/checked_package_v2_owners.rs; FR-038-AC-303/304; TC-229"
review_set: subset
---

## Summary

The hotfix switches the owner join from a fictitious node field to the raw record identity, but admits a non-population record as a population and silently skips an invalid record with no identity.

## Verdict

**FAIL** — The hotfix switches the owner join from a fictitious node field to the raw record identity, but admits a non-population record as a population and silently skips an invalid record with no identity.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A raw record with a matching identity enters model.populations without checking that its kind resolves to the population meaning; the new default fixture has no population construct and still expects admission. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:2236; tests/it/checked_package_v2_owners.rs:136 |
| FND-002 | high | A selected population record lacking required identity is silently skipped, so an otherwise unreferenced invalid domain document admits and a referenced one reports missing-selection instead of rejecting the malformed selected record. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:2236; tests/it/checked_package_v2_owners.rs:230 |

## Coverage

Exact scope: origin/main through raw-record (two changed files). Reviewed FR-038-AC-303/304 and TC-229. The actual agent-ix-semantic-ir PopulationRecord schema requires identity, kind, members, extent and origin; QSpec FR-154-AC-7 requires a population kind to resolve to a Quire population meaning. The IR selected-document reader authenticates digest and package identity but does not run that schema validator.

Rust review: no new unsafe, async/lock path, wire cast or production panic in the changed lines. Cargo gates were not rerun in this reviewer worktree because the assigned review excludes the shared heavy lane.

## Normative basis

QSpec FR-322 “Model-owned members” step 1 requires this reader to admit each selected document under FR-154 before graph admission and to report the first FR-154 refusal. QSpec FR-154-AC-7 requires a population whose kind names no `constructs` entry, or resolves outside FR-208's meaning set, to refuse `invalid_model_binding`/`malformed-declaration`. IR FR-038's own “Model-owned members” step 1 (lines 810–833) states the same reader boundary. The locked `agent-ix-semantic-ir` PopulationRecord schema (`crates/semantic-ir/src/schema.rs:699–727` at filament-core-data rev ) requires `identity` and `kind`. `admit_document` authenticates and parses bytes but does not run that schema validator; `read_semantic_ir` is the only changed population-declaration check on the selected-document path.

## Dispositions

Round 1, exact hotfix:

| FND | Outcome | Evidence |
| --- | --- | --- |
| FND-001 | fixed | `read_semantic_ir` now calls `classify` on each raw population, whose `kind` must resolve through `constructs`, then requires `meaning::POPULATION`; the default TC-229 document declares the population construct and has separate unresolved and wrong-meaning negative tests. |
| FND-002 | fixed | Required `text(population, "identity")?` rejects the missing string at the model selection; TC-229 asserts `InvalidModelBinding`/`MalformedDeclaration` and `/lock/model_selections/0`. |

## New findings, round 1

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | high | A raw population entry containing a valid `identity` and an extra `node` is admitted because `read_semantic_ir` checks the identity and kind but never rejects `node`. Merged QSpec FR-322 requires any selected population entry with a `node` member to refuse `invalid_model_binding`/`malformed-declaration` at the selection row before graph checks; its new `with-node-member` adverse case isolates this input. | `crates/quire-contract-model/src/checked_package/v2/model_members.rs:2233`; QSpec `FR-322-AC-82` at merged #321; QSpec `adverse.json:90` |

## Dispositions, round 2

Exact hotfix:

| FND | Outcome | Evidence |
| --- | --- | --- |
| FND-003 | fixed | `read_semantic_ir` now rejects any raw population record with a `node` member after requiring `identity`, before owner recovery; the direct TC-229 mutation keeps valid `identity`, adds `node`, and asserts `InvalidModelBinding`/`MalformedDeclaration` at `/lock/model_selections/0`. |

## Current verdict

**PASS** — The open findings for this method are fixed at the exact reviewed head.

## Final-head inspection

The only change since the prior PASS head is `tests/it/checked_package_v2_abstraction_relation.rs`. Its shared selected-document fixture now declares a population construct and emits `identity`, `displayName`, `kind`, `members`, `extent` and `origin` for each population. Both identities match the existing `ModelOwner.node` values; no `node` member appears in the raw records. The production reader and FR-038 text are unchanged from the fixed head. No new finding.
