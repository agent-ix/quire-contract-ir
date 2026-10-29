---
id: SR-12460
title: "IR-870 PR 452 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/terms.rs; crates/quire-contract-model/src/checked_package/v2/comparison_operands.rs; crates/quire-contract-model/src/checked_package/v2/derived_keys.rs; crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs; crates/quire-contract-model/src/checked_package/v2/mod.rs; crates/quire-contract-model/src/checked_package/v2/model_fields/tests.rs; crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs; crates/quire-contract-model/src/checked_package/v2/model_members/tests/intake_retention.rs; crates/quire-contract-model/src/checked_package/v2/operation_catalog.rs; crates/quire-contract-model/src/checked_package/v2/operations.rs; crates/quire-contract-model/src/checked_package/v2/scalar_operands.rs; crates/quire-contract-model/src/checked_package/v2/temporal.rs; crates/quire-contract-model/src/output_mapping.rs; crates/quire-contract-model/src/wire.rs"
review_set: subset
---

## Summary

Planless diff-scoped gap analysis: all removed trace IDs retain machine-readable binders, coverage and normalized computed matrix are unchanged. One low-severity PR receipt count is inaccurate. Ticket: IR-870. Reviewed the frozen PR head.

## Examined scope

- `crates/quire-contract-model/src/checked_package/terms.rs` (examined)
- `crates/quire-contract-model/src/checked_package/v2/comparison_operands.rs` (examined)
- `crates/quire-contract-model/src/checked_package/v2/derived_keys.rs` (examined)
- `crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs` (examined)
- `crates/quire-contract-model/src/checked_package/v2/mod.rs` (examined)
- `crates/quire-contract-model/src/checked_package/v2/model_fields/tests.rs` (examined)
- `crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs` (examined)
- `crates/quire-contract-model/src/checked_package/v2/model_members/tests/intake_retention.rs` (examined)
- `crates/quire-contract-model/src/checked_package/v2/operation_catalog.rs` (examined)
- `crates/quire-contract-model/src/checked_package/v2/operations.rs` (examined)
- `crates/quire-contract-model/src/checked_package/v2/scalar_operands.rs` (examined)
- `crates/quire-contract-model/src/checked_package/v2/temporal.rs` (examined)
- `crates/quire-contract-model/src/output_mapping.rs` (examined)
- `crates/quire-contract-model/src/wire.rs` (examined)

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | PR body reports 551 matrix rows, while both independent computed matrices contain 550; correct the receipt. The before and after matrices have 550 criteria rows each, with identical status, method, statement, binder file, symbol and kind after source-line normalization. Coverage is 527/584 backed, 40 unbacked and zero contradicted on both sides. | PR #452 body |

## Verdict

CONDITIONAL — Planless diff-scoped gap analysis: all removed trace IDs retain machine-readable binders, coverage and normalized computed matrix are unchanged. One low-severity PR receipt count is inaccurate.

## Coverage

Plan completion: not assessed
Computed matrix: 550 criteria rows on base and head; normalized rows identical.
Coverage: 527/584 backed; strict 40 unbacked, 0 contradicted, on base and head.
Gate evidence: `git diff --check` and `make fmt-check` passed; author supplied full test and lint receipts for the doc-only diff.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | PR #452 body now says 550 criterion rows and 551 TSV lines including the header; read back from GitHub at the same frozen head. |
