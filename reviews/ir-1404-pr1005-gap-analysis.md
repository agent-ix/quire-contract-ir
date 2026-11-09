---
id: SR-202611081512300009
title: "gap-analysis review of PR #1005 bounded unpaid read memory"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; FR-038-AC-511; FR-038-AC-513; FR-358-AC-12; tests/read_storage_alloc.rs; tests/it/support/read_observation.rs; tools/read_path_proof; IR-1404"
review_set: subset
---

## Summary

Ticket: IR-1404. PR agent-ix/quire-contract-ir#1005 at head 3304ddd541baf60c9214e172e1f8525d9656c832, diffed against origin/code/ir-main-read-accounting. The audit is scoped to the PR. The direction (bound instead of exact equality, removing the syn walker, AC-513 Planned) is the owner's and is not a defect here. One trace gap remains: an unwired tool still claims to verify AC-513.

## Verdict

CONDITIONAL. AC-511 is tagged and exercised by the counting-allocator tests. AC-513 is marked Planned by owner direction, but `quire trace` still credits it with evidence from a tool nothing runs. Semantic review of intent, test and code was done only for the AC-511 priority questions, and its findings are in the code-review and spec-review artifacts.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-513 is declared to have no automated inspection, yet `quire trace` reports eight `verifies` claims from tools/read_path_proof/test_compiler_reachability.py. No Makefile target or active spec line runs that tool, so the computed matrix credits AC-513 with evidence that the spec says does not exist. Remove the tool's stale AC-513 trace tags, or the tool itself, in this PR. | tools/read_path_proof/test_compiler_reachability.py:252; spec/checked_package/matrix/tests.md:23 |

## Coverage

- `FR-038-AC-511` (examined): tagged by the counting-allocator tests in tests/read_storage_alloc.rs.
- `FR-038-AC-513` (examined): Planned by owner direction; see FND-001.
- `FR-358-AC-12` (examined): `quire trace` reports one structural claim, document_de.rs:1222.
- Deleted names `measure_at_unchecked`, `read_path_graph` and `read_path_inspection` have no remaining reference in tools, tests, crates or the Makefile.
- Plan completion: not assessed

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fix commit 6f055ec deletes the compiler-reachability tool and its tagged tests. `quire trace` now shows no Python claim on FR-038-AC-513. One Rust claim remains, at crates/quire-contract-model/src/checked_package/v2/frame.rs:1092; it is a storage test, not a source-path inspection, and was outside this finding. |
