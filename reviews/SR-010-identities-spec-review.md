---
id: SR-010
title: "Issue 6 identity and anchoring specification review"
type: SpecReview
analysis: architecture-evaluation
scope: "FR-011, FR-012, STD-001, NFR-002, TM-002, TC-015, TASK-006"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/6
    type: reviews
---
# Issue 6 identity and anchoring specification review

## Summary

FR-011 and FR-012 define the issue #6 boundary without importing the typed
expression semantics owned by issue #8. The implementation can represent a
generic clause body that exposes referenced identities, enforce anchoring and
revision rules now, and let the later typed expression tree implement the same
dependency-source contract.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-020 | low | Identity grammar must be validated at every public constructor so malformed values cannot enter validated references. | FR-011-AC-1, TC-015 |
| FND-021 | low | Dependency sets must be derived from recursive clause content and canonically deduplicated, never supplied as cached caller input. | FR-012-AC-2, TC-015 |
| FND-022 | low | Exact requirement revision is part of every downstream clause identity, allowing stale and orphaned failures to remain distinct. | FR-011-AC-2, FR-012-AC-1 |

## Independent Review Disposition

The independent spec review reported ten findings. The specification now owns
or explicitly stages each item: FR-012 distinguishes issue #6 orphaned semantic
references from issue #9 orphaned artifact coverage; STD-001 registers codes;
FR-011 adds negative criteria and defines revision monotonicity; FR-012 defines
the dependency-source abstraction and anchor matrix; TM-002 splits TC-015 and
TC-016 coverage and adds the issue #6 language-independence inspection.

The final pass found five additional gaps. FR-011 now owns a non-canonical JSON
structural round trip and source identity/revision failures; FR-012 separately
owns source-span validation and splits issue #6 dependency traversal from issue
#8 typed-expression conformance; NFR-002-AC-3 gives TC-015 a falsifiable public
vocabulary boundary.

The third pass fixed the v0.1 wire schema at `1.0` while permitting minor zero,
defined diagnostic precedence, made every issue #6 anchor code criterion-owned,
and registered STD-001 in the requirements architecture and TC-015. The
installed process module's known `Coverage Status` versus `Status`
contradiction remains the recorded upstream FND-002 limitation: its archetype
rejects the traceability engine's requested header, so this repository retains
the structurally valid header and does not claim status-classification coverage.

The closing consistency pass corrected duplicate-ID wording, gave TC-016 an
explicit NFR-002-AC-4 owner, expanded this review's declared scope, and added
FR-012's cited FR-017 relationship. No issue #6 specification blocker remains.

No implementation-blocking specification finding remains. Issue #8 remains the
owner of type checking, evaluation semantics, and definedness analysis.
