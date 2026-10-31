---
id: SR-2026103039
title: IR-1196 PR 740 gap analysis
type: SpecReview
analysis: gap-analysis
scope: 'agent-ix/quire-contract-ir; PR #740 head 7cffebec; FR-038 Debug clause; FR-038-AC-117; crates/quire-contract-model/src/checked_package/shared.rs; crates/quire-contract-model/src/checked_package/v2/mod.rs'
review_set: subset
---

## Summary

Scoped requirement-to-code-and-test check of IR-1196 PR #740. The ticket's Debug guarantee is exercised by a named 100,000-depth test on a 512 KiB stack.

## Verdict

**PASS** for the changed behavior examined here. This is a scoped manual check, not a repository-wide matrix verdict.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed.

- **IR-1196** (`examined`, Linear ticket): "make Debug for CheckedPackageV2 and CheckedNodeId linear and non-recursive, each node written once with children by position, so a 100k-depth package formats on a 512 KiB stack." The new test directly exercises both public Debug implementations and checks node count and positional references.
- **FR-038** (`examined`, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): "Every value an admitted package holds nests to a depth the grammar fixes, so its clone, comparison, `Debug` rendering, lowering and drop need no stack that follows the package's size". The test covers the Debug clause.
- **FR-038-AC-117** (`context_only`, same path): "a package holding a chain of 100000 nodes, each referencing the previous ... on a thread whose stack is 256 KiB, is admitted and lowered from its last node with no outcome naming a depth and no stack overflow". This PR does not change the reader or lowering parts of AC-117.
- `quoin matrix --repo . --json` was attempted but did not return promptly and was stopped. Traceability of the new test was checked manually; repository-wide matrix coverage is unavailable for this review.
- No plan was supplied. Reverse gap: the changed Debug implementations have the FR-038 Debug clause and IR-1196 as owners. No stubs or coverage inflation were found in the changed code.
