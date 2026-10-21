---
id: SR-14662
title: 'IR-539 operation required-nullable fields: spec-review-base'
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir; IR-539; OperationWire, public reader controls,
  FR-038/TC-048, computed repository matrix
review_set: subset
---

## Summary

Ticket: IR-539. PR493 frozen candidate reviewed against its actual main parent. Reviewer-only source/Git/Quire inspection; no Cargo, rustfmt, metadata, native test, aggregate or Kani command was run. No applicable AssuranceProfile was found under spec/. Coder receipts were independently inspected: formatting and locked workspace/all-target plus model Clippy passed; 23 healthy passing executions (22 distinct tests), old production compiled and failed the new test because omitted mode admitted, exact healthy restoration passed. Peak recorded memory 3,462,295,552 bytes under 16 GiB/Swap0, zero events. These are inspected execution receipts, not fresh reviewer executions; final aggregate remains the lead's gate.

## Verdict

**PASS** — No defect found in the examined status-only specification change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Status-only specification review: existing FR-038-AC-81 and TC-048 switch PLANNED to implemented/exercised, supported by the actual public-reader test and recorded old-production negative control. No new requirement meaning, ID, relationship, domain object or EARS requirement statement is introduced; separate EARS, integrity, dependency, object and other lenses are therefore not applicable. Both changed documents validate. Canonical application/node/package identity regeneration is the existing semantic identity requirement, not a new file-tracking mechanism. AC83/84 catalog-required explicit-null refusals and distinct OperationLeafWire remain unchanged.

## Examined scope

- `FR-038-AC-81` (examined) `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`: through `CheckedPackageV2::read`, independently omitting `operation.mode` or `operation.member` from an otherwise valid application refuses `invalid_semantic_graph`, with no cause, at `/semantic_graph/nodes/{i}/body/operation`, before catalog semantic checks; the otherwise identical application with both fields present as `null` admits when its catalog entry requires neither. Re-derive all affected application/node and package identities after each mutation so a stale identity cannot mask the operation shape refusal. Explicit-null required-mode and required-member semantic refusals remain AC-8
- `FR-038-AC-83` (examined) `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`: Observed at the operation check, `quire.op.decimal.add`, which catalogues the mode kind `rounding`, with `mode` `null` refuses `invalid_package`/`operation-mode-mismatch` at `operation.mode`; with the mode `{kind: rounding, value: toward-zero}` over a first operand typed at a `bounded_domain` over a `decimal` whose body binds `rounding` to `nearest-even`, reached by a `reference` to the domain and, separately, by a `literal` whose `type` names it, it refuses `invalid_package`/`operation-mode-type-mismatch` at `operation.mode/value`.
- `FR-038-AC-84` (examined) `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`: Observed at the operation check, `quire.op.quantity.convert`, which catalogues the member kind `type_argument`, with its `rounding` mode supplied and `member` `null` refuses `invalid_package`/`operation-member-mismatch` at `operation.member`; `quire.op.record.project` with a `field` member whose `name` is a field the declaring record type does not declare refuses `ill_typed`/`operator-ineligible` at `operation.member.name`; and `quire.op.model.reaches_field` with a `field` member whose `declaration` names no node of the graph refuses `ill_typed`/`operator-ineligible` at `operation.member.decla
- `TC-048` (examined) `spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md`: The old
production decoder admits the omission and fails this test; restoring the
healthy decoder restores the passing result.
