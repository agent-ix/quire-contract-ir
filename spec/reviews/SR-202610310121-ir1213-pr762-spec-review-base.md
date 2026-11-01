---
id: SR-202610310121
title: "IR-1213 PR 762 FR-019 inventory spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #762; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Reviewed the changed FR-019 public-item row against the actual public associated methods, FR-354 callable contract and TC-058 inventory. No applicable AssuranceProfile was found under `spec/`.

## Verdict

**CONDITIONAL** — the public inventory omits the new direct callable read.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The inventory names `callable_table` but omits the public `source_callable` associated read that is necessary for a request-specific pure-function read. | spec/model/functional/FR-019-rust-library-interface.md:241 |

## Coverage

- FR-019 public callable inventory, role: examined. Excerpt: types `CheckedCallable`, `CheckedCallableIdentity`, `CheckedCallableReadError`, `CheckedCallableReadErrorCode`, `CheckedCallableTable`; associated callable_table read.
- FR-354-AC-3, role: context_only. Excerpt: A pure function can be read from its admitted node without that row.
- FR-019-AC-5, role: examined. Excerpt: Public model crate root exports match the documented inventory.

## Evidence

`CheckedPackageV2::source_callable` is public at `callable.rs:213`, and is the direct pure-function read. The changed FR-019 row names only `callable_table`; TC-058 checks exported type names, not associated method completeness.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Head 275cf8e3 names both `source_callable` and `callable_table` associated reads in the FR-019 public-item row. |

## Disposition verdict

**PASS** — the inventory now names the complete public callable read surface.
