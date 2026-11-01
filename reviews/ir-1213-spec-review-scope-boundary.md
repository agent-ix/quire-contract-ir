---
id: SR-2026103038
title: "IR-1213 spec-review/scope-boundary review"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; PR #751"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-352
    type: reviews
---

## Summary

Ticket: IR-1213. PR #751 at 2980a78bfeb8997ea14974c0a96242ba152e6099, base origin/main 6d3229ea8e56513c6b6e1833cf974f094a557bfe. Reviewed the new SelectedOperationMemberRow and AC-10 only, against FR-040 and FR-354. No applicable AssuranceProfile is present.

## Verdict

**PASS.** No findings in this method.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-352, role: examined. Excerpt: - For an admitted model-owned state operation anchor, a QSL checking/model stage   `SelectedOperationMemberRow` from the same selected documents. The row   carries the full anchor `CheckedNodeId`, selected domain document identity,   declaring model's original IR identity and closed declaration form, the   selected operation member's original IR identity, and the stage's selected   operation identity. The latter is the stage's existing operation identity;   this contract does not mint an Exact key for an operation. The row is   producer input alongside the package, not a new QSpec wire member.
- FR-352-AC-10, role: examined. Excerpt: A QSL-stage `SelectedOperationMemberRow` for an admitted model-owned operation anchor joins its full structural node, selected document, declaring model form and original model/operation member identities to the selected operation identity; the FR-354 callable read uses the joined row to find the admitted body and frame. Two admitted anchors of one operation at distinct execution points remain distinct rows. A missing row, wrong document/form/member, foreign anchor, duplicate conflicting row, or name-only match returns a named typed outcome without a callable view. A source-declared anchor wit
- FR-354-AC-3, role: context_only. Excerpt: The callable projection returns each selected function or operation's ordered parameters, admitted body handle, context and frame, including an intentionally empty frame; a dangling body, wrong owner or absent required frame returns a typed outcome.
- FR-040-AC-31, role: context_only. Excerpt: PLANNED/UNRUN (IR-1141). An anchor admits its three required bindings with no point or a fourth execution_point binding after frame holding the closed pre or post object. Reordering the fourth binding or adding another binding refuses invalid_semantic_graph at the exact offending member. The reader retains the point in the complete body; it does not rederive the node id from that body.
- FR-040-AC-32, role: context_only. Excerpt: PLANNED/UNRUN (IR-1141). An anchor with an unknown point variant, missing or extra point member, or empty name refuses `invalid_semantic_graph` with no cause at `/semantic_graph/nodes/{i}/body/members/3/value` before state. A well-formed `initialization` or `handler` point or point whose operation differs from the checked operation refuses `invalid_model_binding`/`malformed-declaration` at `/semantic_graph/nodes/{i}` in state. A duplicate `(context, operation, point)` tuple refuses `ambiguous_declaration`/`ambiguous-name` at the second anchor by ascending structural node id; the same context a

## Boundary

QSL supplies the selected operation identity, IR verifies its join to the admitted anchor, and FR-354 owns the callable read. No Exact operation key or QSpec wire field is introduced. The missing handoff is recorded in the integrity review.
