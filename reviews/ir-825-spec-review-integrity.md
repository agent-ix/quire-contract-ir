---
id: SR-11841
title: "Integrity review of the FR-052-AC-3 / AC-24 split (PR #421)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/model/functional/FR-052-source-obligation-correspondence.md (FR-052-AC-3, FR-052-AC-24, Open questions, status prose), spec/model/matrix/TC-452-source-obligation-correspondence.md, spec/model/matrix/tests.md (TC-452 row)"
review_set: subset
---

## Summary

Ticket: IR-825. This review checks that the split of AC-3's
conflicting-inventory clause into the new FR-052-AC-24 is faithful, and that
the split stays consistent across FR-052, TC-452 and the model matrix.

Several things are consistent. AC-3 keeps every other obligation: foreign
source, each dangling endpoint, exact duplicate, conflicting relation facts,
conflicting provenance, inspectability, no accepted relation, and the
outcome-boundary foreign-row sentence. AC-3 points to AC-24. AC-24 is
singular, verified by `Test`, and names the disposition `ConflictingIdentity`
from the FR body. The status prose states its dependency: IR-820 (Backlog,
"SemanticNode carries CheckedSemanticId but CheckedSemanticNodeV2 carries
semantic_type: CheckedNodeId") and then IR-825. The Open questions bullet
links AC-24 to IR-820. TC-452 and tests.md list AC-24. TC-452 forbids tracing
`orphan_or_conflicting_endpoints_do_not_resolve` to AC-24 before IR-820.
The ID is unused in `git log --all -S` before this branch, and #421 is the
only open PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | On origin/main, AC-3's shared consequence "each receive their named deterministic disposition; rejected entries remain inspectable and never enter accepted relations" applied to every listed item, conflicting inventory facts included. FR-052's endpoint rule makes this concrete: an occurrence whose node is `ConflictingIdentity` does not resolve. AC-24 keeps "named deterministic disposition" and "remains inspectable" but drops "never enter accepted relations". The split therefore narrows the clause it moved, contrary to the no-weakening condition. AC-23 still asserts the behaviour synthetically, so the practical loss is small, but AC-24 alone no longer requires it. Append e.g. "and no occurrence under a conflicting node enters an accepted relation" to AC-24. | spec/model/functional/FR-052-source-obligation-correspondence.md:479; spec/model/functional/FR-052-source-obligation-correspondence.md:369-375 |

## Verdict

Not mergeable until FND-001 is fixed. It is a one-clause addition to AC-24.
Everything else in the split is consistent, and it carries its owner and
dependency.
