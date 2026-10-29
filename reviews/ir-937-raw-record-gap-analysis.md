---
id: SR-14721
title: "IR-937 raw population record gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-937 raw PopulationRecord hotfix; crates/quire-contract-model/src/checked_package/v2/model_members.rs; tests/it/checked_package_v2_owners.rs; FR-038-AC-303/304; TC-229"
review_set: subset
---

## Summary

The computed matrix tags FR-038-AC-303/304, but the new tests do not gate raw-record meaning or the required-identity refusal at the selection boundary.

## Verdict

**CONDITIONAL** — The computed matrix tags FR-038-AC-303/304, but the new tests do not gate raw-record meaning or the required-identity refusal at the selection boundary.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new TC-229 positive fixture omits a population construct, while only one separately constructed case supplies it; no test mutates the raw record kind to a non-population meaning or asserts refusal before owner join. | tests/it/checked_package_v2_owners.rs:136; tests/it/checked_package_v2_owners.rs:196 |
| FND-002 | medium | The missing-identity test asserts a downstream MissingDeclaration/MissingSelection outcome for an invalid raw PopulationRecord, so it cannot catch silent acceptance of the malformed selected document when no owner references that record. | tests/it/checked_package_v2_owners.rs:230 |

## Coverage

Exact scope: origin/main through raw-record (two changed files). Reviewed FR-038-AC-303/304 and TC-229. The actual agent-ix-semantic-ir PopulationRecord schema requires identity, kind, members, extent and origin; QSpec FR-154-AC-7 requires a population kind to resolve to a Quire population meaning. The IR selected-document reader authenticates digest and package identity but does not run that schema validator.

Plan completion: not assessed. Optional whole-repository semantic review: not requested; this method checked changed criterion-to-test bindings.

## Normative basis

QSpec FR-322 “Model-owned members” step 1 requires this reader to admit each selected document under FR-154 before graph admission and to report the first FR-154 refusal. QSpec FR-154-AC-7 requires a population whose kind names no `constructs` entry, or resolves outside FR-208's meaning set, to refuse `invalid_model_binding`/`malformed-declaration`. IR FR-038's own “Model-owned members” step 1 (lines 810–833) states the same reader boundary. The locked `agent-ix-semantic-ir` PopulationRecord schema (`crates/semantic-ir/src/schema.rs:699–727` at filament-core-data rev ) requires `identity` and `kind`. `admit_document` authenticates and parses bytes but does not run that schema validator; `read_semantic_ir` is the only changed population-declaration check on the selected-document path.

## Dispositions

Round 1, exact hotfix:

| FND | Outcome | Evidence |
| --- | --- | --- |
| FND-001 | fixed | The default TC-229 fixture now declares the population construct. Separate traced tests mutate the kind to unresolved and wrong meaning and assert model refusal. |
| FND-002 | fixed | The missing-identity TC-229 test now expects `InvalidModelBinding`/`MalformedDeclaration` at `/lock/model_selections/0`, so it catches the former silent skip. |

## New findings, round 1

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | high | QSpec #321 merged with 15 population-reference adverse cases, including independent `with-node-member` and `without-identity` selected-document mutations. TC-229 still asserts 13 cases and handles only `node-only`; against current merged QSpec it fails at inventory before testing either new case, and its fallback document for the new cases remains valid. | `tests/it/checked_package_v2_owners.rs:721,739`; QSpec `adverse.json:90-91` at merged #321  |

## Dispositions, round 2

Exact hotfix:

| FND | Outcome | Evidence |
| --- | --- | --- |
| FND-003 | fixed | TC-229 now expects the 15 cases on merged QSpec main, handles `node-only`, `with-node-member`, and `without-identity` separately, and asserts selection-row locus for all three. The direct `with_node_member` case independently gates production refusal. |

## Current verdict

**PASS** — The open findings for this method are fixed at the exact reviewed head.

## Final-head inspection

, the TC-225 shared fixture replaces two synthetic `{"node":...}` population entries with full raw records and a resolving population construct. The only remaining `node` fields in that test file are graph owner members, not raw selected-document population entries. This removes unrelated TC-225 fixture regressions without changing TC-229's criterion binding or conformance cases. No new finding.
