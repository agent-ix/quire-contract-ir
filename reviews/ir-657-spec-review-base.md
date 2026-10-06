---
id: SR-2021
title: "spec-review/base review of IR-657 catalog spelling"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@56a7a5a96f85282618adcdd7c42ebf41c39f381f; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; context: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

Ticket: IR-657, PR #305. Requirements quality and catalog-identity check. Reviewed the two edited FR-038 passages and context against the authoritative `quire-verification-contracts/contracts/checked-operation-catalog-v1.json`.

## Verdict

**PASS** — The edited identities match the catalog; no defect found in this method's scope.

## Coverage

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038 statement | examined | When a caller supplies an admitted application node id and one of that node's `CheckedOccurrence` keys, `CheckedPackageV2::scalar_application_operands` shall return a typed result in `body.arguments` order for the integer `add`, `sub`, `mul` and `negate` operation identities published in `quire-verification-contracts/contracts/checked-operation-catalog-v1.json` and used by QSL [FR-357](ix://agent-ix/quire-spec-language/FR-357). |
| FR-038-AC-159 | examined | For admitted `quire.op.integer.add`, `quire.op.integer.sub`, `quire.op.integer.mul` and `quire.op.integer.negate` applications whose every operand has finite bounds representable in `i128`, `scalar_application_operands` returns one entry per argument, in wire argument order, with ordinals `0..n-1`, typed identities and exact inclusive `i128` ranges; the unary operation returns one entry and the binary operations return two. Swapping two arguments swaps their entries and ordinals without sorting by child id. |
| TC-048 | context_only | Read an admitted package containing integer add, subtract, multiply and negate applications. For each, call `scalar_application_operands` with its node id and an actual occurrence key. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
