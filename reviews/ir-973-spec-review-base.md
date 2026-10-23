---
id: SR-15728
title: "Base spec review — FR-035-AC-22 and FR-019 API prose"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; IR-973 PR #523; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md, spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Reviewed the new acceptance criterion and public API prose for ownership, IDs, atomicity, testability, and consistency with the existing FR-330 six-member contract.

## Verdict

**PASS** — The added criterion describes the local keyed lookup and preserves the established wire boundary. The public inventory adds methods on the already exported type without inventing crate-root exports.

## Scope

- FR-035-AC-22 (examined): For every represented local key, producer and strict reader return the exact node and admitted typed term; absent keys return none; retained lookup does not change wire or identity.
- FR-019 public surface (examined): `ContractPackageV1::node` and `ContractPackageV1::term` are methods, not crate-root exports.
- FR-035-AC-19 through FR-035-AC-21 (context only): Existing keyed kinds, accounting tags, and retention invariants.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
