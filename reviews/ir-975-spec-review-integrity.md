---
id: SR-15724
title: "Integrity review — IR-975 single-body V2 wire"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; PR #521; FR-038/FR-051/FR-349/FR-019 consistency"
review_set: subset
---

## Summary

The seven-member wire description and canonical hash view have distinct, inspectable responsibilities. Two existing FR-038 criteria still describe a former on-wire projection, creating a direct contradiction with the newly added closed-wire criterion.

## Verdict

**PASS ** — AC-123 and AC-133 reconcile with AC-310. The adjacent AC-113 type mismatch recorded in base review SR-15723 was fixed in the second documentation commit.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-123 and AC-133 require patching an on-wire projection while AC-310 refuses any on-wire projection before identity or node-key checks; make their tamper setup derive the hash projection from graph nodes. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4860 |

## Coverage

Examined FR-038-AC-4, AC-107, AC-112, AC-123, AC-133, AC-154 and AC-310; FR-051-AC-4; FR-349-AC-3; FR-019 public interface text. The new seven-field selection, graph-derived hash, old-wire refusal and writer rules otherwise form one consistent contract. Plan completion: not assessed.

## Custody correction

This artifact supersedes the same reviewer method under SR-15717 on IR-975. SR-15717 had already been assigned to IR #522; the original finding text above is unchanged; later dispositions or head confirmations record the current verdict.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | AC-123 and AC-133 now use the seven selection fields plus the mutated graph nodes for package identity, and AC-310 still refuses any shipped projection. |
