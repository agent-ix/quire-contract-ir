---
id: SR-2026110602354640
title: "IR-1354 PR #941 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/monitor_plan/selected_type_closure.rs; spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md; spec/model/functional/FR-019-rust-library-interface.md; TC-470"
review_set: subset
---

## Summary

Ticket: IR-1354. Rust code review with rust-review lane. The equality application storage helper counts application and source-map bytes but omits node_key and location, then select_rows clones the full row. Coder-reported focused tests and Clippy pass; no aggregate gate was run.

## Examined units

- FR-357-AC-1 (examined, spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md): "Implemented (IR-1354; TC-470). One bounded read of an admitted `CheckedMonitorPlanRef` seeds State, Inputs, Scope, Results and all four ordered entry signatures and bodies, then returns exactly the reachable complete type/declaration and admitted equality closure. Source records, tuples, structural collections, sealed unions, source/model enum cases, model objects/fields and units/dimensions/conversions retain original identities, QSL-issued opaque Exact identities, producer order, presence, shape and member/payload types sufficient for the canonical `TypeEnvironment`; shared references yield "
- FR-357-AC-2 (examined, spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md): "Implemented (IR-1354; TC-470). A missing, foreign, duplicate, conflicting or stale producer row; changed member/payload type node; unresolved reached declaration; cross-package reference without an admitted resolution rule; unrepresentable reached type; or native nominal, Exact or equality refusal returns its typed cause and first node/edge/row/occurrence locus, with charged work, no partial view and no invented identity. An unrelated unsupported graph node or absent row outside the reached closure does not refuse; a selected row search can still exhaust at an inspected unrelated row position."
- FR-357-AC-3 (examined, spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md): "Implemented (IR-1354; TC-470). A large selected closure amid unrelated graph nodes visits only reached graph, typed-term, semantic-type and declaration edges using iterative worklists and visited node/edge sets. A deep term does not grow the host stack. One shared native work and storage budget charges every inspected node/edge, row/index, member/payload position and identity/value byte, including nested Exact, nominal and equality reads and output; the exact allowance succeeds and one less names the first uncharged locus. Equal inputs and allowances, including a package clone, give equal comp"
- FR-357-AC-4 (examined, spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md): "Implemented (IR-1354; TC-470 and FR-019 exact inventory). Static API and dependency analysis finds one bounded `CheckedMonitorPlanRef` projection over admitted package facts and IR-owned selected producer rows, with no IR-owned `TypeEnvironment`, second plan/evaluator, QSL or RT dependency, source reparse or witness fabrication. The root exports appear in FR-019's exact implemented inventory."
- TC-470 (examined, spec/checked_package/matrix/TC-470-runtime-type-projection-shared-budget.md): "Verify that runtime type projection, monitor-closure, nominal, and selected unit-definition reads charge one caller-owned native budget before bounded work or any reachable IR-owned storage, and preserve admitted facts within the measured allowance."
- FR-019 (examined, spec/model/functional/FR-019-rust-library-interface.md): "The stable surface is the public API of quire_contract_model in a default-feature build: exactly the items the Public items table below lists."

## Verdict

**FAIL** — One high storage accounting finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Selected equality rows are cloned after storage precharge omits the producer-issued node_key and location. A reached equality with long opaque key/location data can allocate beyond the caller-owned storage allowance; charge those identity bytes before row.clone(). | crates/quire-contract-model/src/checked_package/v2/monitor_plan/selected_type_closure.rs:305-310, 479-503 |

## Coverage

Plan completion: not assessed. Review scope is PR #941 diff, FR-357 AC1-4, TC-470 and FR-019.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At 2d1cfb9cf, preflight_equality_application charges both producer keys before select_rows clones the reached row. |
