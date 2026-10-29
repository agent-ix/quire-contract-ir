---
id: SR-612063
title: "IR-1059 slice 1 trace gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 634 tests/it/checked_package_v2_model_members.rs and crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs; FR-069-AC-1,4,6"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-069
    type: reviews
---

## Summary

Ticket: IR-1059. Planless, PR-diff-scoped trace review of slice 1's new TC-940 tests against FR-069's acceptance criteria. The test evidence is local, while the tagged criteria require authoritative published vectors and a broader refusal matrix.

## Verdict

**CONDITIONAL** — TC-940 should trace only the assertions it actually makes, and the remaining FR-069 criteria stay open for the later full conformance work. No full test gate was run during review.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Both new TC-940 tests tag FR-069-AC-1 and AC-4, and the unit test also tags AC-6, but they construct one local Pair/Maybe document rather than read QSpec's authoritative Pair/Swap/Nest/Cell package and vectors. They do not compare declaration keys across aliases, assert union/enum family and ordered cases, or check the published exact refusal matrix. The tags make the computed matrix show those criteria as backed by tests that cannot fail most stated obligations. | crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs:461; tests/it/checked_package_v2_model_members.rs:336 |

## Coverage

- FR-069-AC-1, role: examined. Excerpt: “The authoritative positive model tuple and union package admits with `model`/`tuple_value_type` declarations for `Pair`, `Swap`, `Nest` and `Cell`... The production reader's re-derived declaration node keys equal QSpec `model-tuple-union-vectors.json` and remain unchanged across aliases.” Local tests do not read or compare those vectors.
- FR-069-AC-4, role: examined. Excerpt: “A payload-free variant has family `enum`; a mixed nullary/payload variant has family `union`, case order is ascending UTF-8 name, and payload cases retain exactly one resolved position-zero type.” Tests check only local payload retention and admission.
- FR-069-AC-6, role: examined. Excerpt: “Every published intake and wire mutation returns its exact QSpec code, cause and location at its specified stage.” The unit test asserts only three local refusal codes/causes and does not assert locations.
- FR-069-AC-2, AC-3, AC-5, AC-7 and AC-8, role: context_only. These are outside the slice and remain open.
- Reverse trace: new tuple and variant payload behavior has an owning FR-069; no unowned code was identified in this diff.
- Plan completion: not assessed.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed |, both local slice tests removed `Trace: FR-069-AC-1/4/6` and `#[trace("TC-940")]`; their comments explicitly identify local admission/retention evidence and leave published TC-940 vectors and the full FR-069 matrix open. The added family regression is likewise untagged, so these partial tests no longer falsely back complete ACs in the computed matrix. |

## Disposition verdict

Round 1: FND-001 fixed on inspection. FR-069 AC1–8 are not declared complete by this urgent slice.
