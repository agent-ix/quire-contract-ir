---
id: SR-15729
title: "Integrity analysis — FR-035-AC-22 typed view"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; IR-973 PR #523; FR-035-AC-22 and FR-019 public API prose"
review_set: subset
---

## Summary

Checked the added criterion for consistency with FR-035's source/lowered identity boundary, retained graph order, and FR-330's exact wire members.

## Verdict

**PASS** — The new requirement is a read-only typed view over represented local keys; it neither creates a second semantic authority nor conflicts with the existing canonical package contract.

## Scope

- FR-035-AC-22 (examined): Keyed node and optional term lookup for both admitted producer and reader packages, independent of graph ordinal.
- FR-035-AC-5 and FR-035-AC-21 (context only): Six-member wire and prior retained-view invariant.
- FR-019 public surface (examined): Method inventory on `ContractPackageV1`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
