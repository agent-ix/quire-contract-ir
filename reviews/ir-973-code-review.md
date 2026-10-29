---
id: SR-15726
title: "Code review — retained ContractPackageV1 terms and node index"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-973 PR #523; crates/quire-contract-model/src/checked_package/v2/contract_package.rs, contract_package/result_context.rs, tests/it/complete_v1_contract_package.rs; FR-035-AC-22, FR-019"
review_set: subset
---

## Summary

Reviewed the frozen PR diff, including Rust idioms, admission ordering, typed source-to-lowered association, node-index construction, work-meter semantics, and the new traced test. No code defect was found in this diff.

## Verdict

**PASS** — The producer copies admitted typed terms by represented typed key; the strict reader retains the terms decoded during its preexisting validation pass. The node index is built only after admission and re-used in closure traversal, whose prior charge remains. No production wire type or CI lane changes.

## Scope

- FR-035-AC-22 (examined): For every represented local key, both the producer's and strict reader's admitted `ContractPackageV1` return the exact node and the same optional `CheckedTerm` admitted from its source body through keyed lookups, independent of graph ordinal. An absent key returns no node or term. These retained lookups change no wire member, canonical byte or identity.
- FR-035-AC-21 (context only): Retaining kinds and unsupported tags changes none of the six FR-330 wire members, canonical bytes or identity; malformed values preserve strict refusal and exhausted limits remain incomplete.
- FR-019 public surface (examined): `ContractPackageV1::node` and `ContractPackageV1::term` are methods of the already exported type.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Checks

Rust review covered public docs, keyed ownership, absence behavior, panic and conversion surfaces, unchanged wire serialization, retained-node uniqueness established by admission, and preservation of closure work charge. No Cargo build was run in the reviewer lane because the shared heavy lock was occupied; the author owns the final-head gate.
