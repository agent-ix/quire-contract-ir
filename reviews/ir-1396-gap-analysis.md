---
id: SR-20261107060894602
title: IR-1396 PR 946 carried_member gap analysis
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 946 head 186b6d2b; FR-038-AC-495, FR-038-AC-58, FR-038-AC-65, TC-048, crates/quire-contract-model/src/checked_package/v2/operations.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Ticket: IR-1396. A planless, PR-scoped check of the requirement, its tagged tests and the code. FR-038-AC-495 has four `#[trace("TC-048", "FR-038-AC-495")]` tests: two in operation_catalog.rs and two in operations.rs, all passing. AC1 holds: the production catalog at QVC 5769cb8 reads, and `carried_member` carries the FR-322 meaning. AC3 holds: the spec lines are in this PR, and there is no pin, shim or catalog copy (`CATALOG_BYTES` still comes from the QVC crate). AC2 is covered in the code review (SR-20261107060894601 FND-001).

## Verdict

CONDITIONAL: the PR's own criterion is traced and backed by tests. One gap from before this PR, which no ticket owns, remains in the owner's carried-sum admission rules.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | This gap predates the PR and no ticket owns it. QSpec FR-322 L3278-3322 and FR-322-AC-391 (TC-187 CS-04..CS-06) require a reader to refuse `ill_typed`/`operator-ineligible` in three cases: a carried-sum binder bound that is not `Q` (another carrier bound, unit, carrier kind or rounding, or a unit node), and `sum.quantity` over a carried `type_argument`. This reader leaves `bound_is_member` as a no-op (operations.rs:2277), so a carried sum whose binder is bounded by a different carried node is admitted. A downstream consumer (CG IR-1391) relies on that admission. The `result_type`=`Q` rule is enforced, but it refuses at `/body/result_type` rather than "at the application". IR FR-038 claims none of FR-322-AC-389..391, and a Linear search finds no IR ticket for it. Not introduced here and does not block this PR. The lead should file an IR ticket that owns FR-322 carried-sum admission. | crates/quire-contract-model/src/checked_package/v2/operations.rs:2269-2282 |

## Coverage

- FR-038-AC-495, examined: decode `carried_member`; the production catalog's carried sum carries it with empty operands and null family; a carried `type_argument` admits; a `unit`/`bounded_quantity`/other node refuses; an unknown kind is a typed catalog read error.
- FR-038-AC-65, context: existing unknown-word typed-error test that AC-495's third sentence repeats.
- FR-038-AC-58, context: documents the `operation_catalog()` build-defect assert.
- TC-048 Catalog words section, examined: new paragraph matches the four tests.
- spec/checked_package/matrix/tests.md AC-495 row, examined.
- Changed production code is owned by FR-038-AC-495; no stub or unowned feature added.
- Plan completion: not assessed.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | deferred | IR-1398, "IR: V2 reader enforces QSpec FR-322 carried-quantity sum admission (bound_is_member, AC-389..391)", now owns it. The gap predates this PR and is outside IR-1396's ACs. |
