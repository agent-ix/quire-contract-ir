---
id: SR-13142
title: "IR-888 quire-contract-ir PR 477 gap analysis: FR-035 reader outcomes vs code"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir PR #477 vs origin/main; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md, spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md, spec/checked_package/matrix/tests.md; read-only: crates/quire-contract-model/src/checked_package/v2/contract_package.rs, crates/quire-contract-model/src/checked_package/common.rs, tests/it/complete_v1_contract_package.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: reviews
---

## Summary

Ticket: IR-888. Plan completion: not assessed. This is a planless gap analysis of the PR's new criteria against the computed matrix and the reader code. FR-035-AC-10 and FR-035-AC-11 are unbacked, as planned (`quire coverage --scope . --strict`: 42 unbacked against 40 on main, 0 contradicted). Existing tests in `tests/it/complete_v1_contract_package.rs` do not exercise `UnknownContractVersion` or any `Incomplete` outcome of `ContractPackageWireV1::read`. The tests-first follow-up PR must add them. A throwaway probe confirmed the code paths both criteria describe.

## Reviewed units

- FR-035-AC-10 against contract_package.rs:944-956 and common.rs:136-145 (examined)
- FR-035-AC-11 against common.rs:331-348 and contract_package.rs:1619-1681, 1724-1752, 2045, plus v2/mod.rs:620-656 (examined)
- tests/it/complete_v1_contract_package.rs tagged tests at 182 and 265 (FR-035-AC-6/7/8) (context_only)

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The caller's single `work` limit is enforced by two independent meters. `validate_values` sums body work locally and compares it with `limits.work`. `validate_references` then creates `WorkMeter::new(limits.work)` starting at 0. Total validation work can therefore reach about twice the selected limit, and `Incomplete.consumed` reports only the current pass's count. On the fixture, the value pass charges 7 and the reference pass 20, so `work: 20` admits a read that charged 27. This contradicts `CheckedPackageReadLimits.work` ("Maximum semantic-term validation visits") and looks accidental. It predates this PR, so either a follow-up should make the meter cumulative or the spec should state the per-pass semantics | crates/quire-contract-model/src/checked_package/v2/contract_package.rs:1630-1673, crates/quire-contract-model/src/checked_package/v2/contract_package.rs:2045 |
| FND-002 | low | The occurrence-limit stop reports `path` `/source_map` whatever entry exhausted the budget. `CheckedPackageIncomplete.path` is documented as "the pointer of the value whose charge failed", but every other limit reports the failing node or member | crates/quire-contract-model/src/checked_package/v2/contract_package.rs:1745-1751 |

## Verdict

The planned rows are honest: no tag, no false backing, and nothing else in coverage moves. The FR-035-AC-10 refusal path is deliberate and not order-dependent. The FR-035-AC-11 Incomplete outcome is reachable for bytes, nodes, edges, occurrences and work. FND-001 and FND-002 are pre-existing code behaviour. Neither blocks this spec PR, but the AC-11 wording must not hide FND-001 (see SR-13140 FND-003).
