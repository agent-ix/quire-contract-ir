---
id: SR-202611081512300001
title: "Code review — IR-1404 PR 999"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; tests/it/checked_package_v2_stage_read_storage.rs; tools/read_path_proof/"
review_set: subset
---

## Summary

Ticket: IR-1404. PR quire-contract-ir#999 at 53410c466a2db86d2e26a9cd995adfcd13e97f74, diff scope origin/main...HEAD only.

## Verdict

**FAIL** — Rust lane folded in. The prior HEAD-SHA finding is fixed: no source_head, qspec head or rev-parse record remains in tools/read_path_proof, and the reuse path that depended on it is gone. The Python tool unit tests pass (46 passed, 18 subtests). The new tests' locus and amount oracles are real: each denies at an observed request one byte short and compares its locus and amount. The findings are about which requests those oracles reach. The Rust test code otherwise follows the file's helpers; no production Rust changed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-503's declarations case denies the inexact-number scan stack, not a declaration table | tests/it/checked_package_v2_stage_read_storage.rs:307-317 |
| FND-002 | medium | AC-503's 'final document-reader reservation' is the retained Arc charge, so the last reader reservation is never denied | tests/it/checked_package_v2_stage_read_storage.rs:318-327 |
| FND-003 | medium | AC-507's test has no equal-limit admit and never asserts the fixed Retained,0 locus or that the last request is retained | tests/it/checked_package_v2_stage_read_storage.rs:332-377 |
| FND-004 | medium | A failed corpus run keeps no failure diagnostics, yet its error says they are retained | tools/read_path_proof/run_coverage_census.py:389-392 |

## Coverage

Examined units: FR-038-AC-503, FR-038-AC-507, tools/read_path_proof/run_coverage_census.py, tools/read_path_proof/compiler_reachability.py, tools/read_path_proof/coverage_census.py.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | high | persisted_class_proof raises KeyError on every bound class, so class-control runs crash | tools/read_path_proof/run_coverage_census.py:461-465 |
| FND-006 | low | The standalone census CLI still requires artifacts the runner no longer leaves | tools/read_path_proof/coverage_census.py:343-362 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified in fix commit b4c684ec47e638bcf13bfc87ee38c9214de56a3c |
| FND-002 | fixed | Verified in fix commit b4c684ec47e638bcf13bfc87ee38c9214de56a3c |
| FND-003 | fixed | Verified in fix commit b4c684ec47e638bcf13bfc87ee38c9214de56a3c |
| FND-004 | fixed | Verified in fix commit b4c684ec47e638bcf13bfc87ee38c9214de56a3c |
