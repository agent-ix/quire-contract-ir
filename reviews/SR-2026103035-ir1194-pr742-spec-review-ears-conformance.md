---
id: SR-2026103035
title: "IR-1194 spec-review/ears-conformance review"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; PR #742"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-352
    type: reviews
---

## Summary

Ticket: IR-1194. Reviewed frozen PR #742 head f16ea8c4a46e17566486aebfe3039dad8298508b. One new FR and eight criteria. No applicable AssuranceProfile is present.

## Verdict

**PASS.** No findings in this method.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-352, role: examined. Excerpt: When a consumer requests the type of a node in an admitted `CheckedPackageV2`, Contract IR SHALL expose a public, read-only projection from that node's `semantic_type: CheckedNodeId` to `quire_exact::ValueType`. The result SHALL retain the identity and domain of the admitted type or return a typed refusal or incomplete result before exposing any partial value-type view. This is an IR-model read over [FR-038](./FR-038-consume-checked-package-v2.md)'s admitted graph and selected declarations, not another admission path. The dependency direction is IR model to Exact; Exact gains no checked-packag
- FR-352-AC-1, role: examined. Excerpt: An external crate reads an admitted node's semantic type through one public IR-model API and receives Exact `Boolean`, `Integer`, or `Int` for independently authored Boolean, Integer, and inclusive ranges `[0,1000]`, `[0,3]`, and `[0,9]`; it inspects no JSON and reparses no source. An endpoint outside Exact's representable domain returns a typed refusal, never a narrowed interval.
- FR-352-AC-2, role: examined. Excerpt: Admitted selected `Receipt`, `Attempt`, `Flag`, and `Tick` record types project to distinct `Composite(NodeKey)` identities matching authoritative selected declaration witnesses. Their complete selected member names, identities, and projected types agree with the admitted records, including `Tick.count`; a witness for a similarly named declaration in another package refuses.
- FR-352-AC-3, role: examined. Excerpt: An admitted reference to selected `Node` projects to `Reference(EffectiveId)` matching its authoritative object-type witness. The object declaration alone is not returned as a standalone Exact value-type variant. A reference to another selected object type, a foreign package, or a dangling target never produces Node's identity.
- FR-352-AC-4, role: examined. Excerpt: Exact structural identity is matched by package, full resolved path, and ordinal. A changed path segment, ordinal, owner package, or selected declaration refuses with a named typed outcome; duplicate or conflicting witnesses and ambiguous selected owners refuse before any value-type result. Alias equality and display-name equality cannot establish a join.
- FR-352-AC-5, role: examined. Excerpt: Admitted String, Uuid, and Timestamp scalar types map to their respective Exact variants when present. A selected declared unit maps through a witnessed declared `UnitId`; a compound unit without its authoritative identity refuses. A selected union uses `Composite` only with a verified union witness; dimension and alias shapes without faithful Exact representations return `UnsupportedExactType` with their own form and node.
- FR-352-AC-6, role: examined. Excerpt: Every other admitted V2 type form either projects to an Exact variant with its full bounds, mode, members and identity, or returns `UnsupportedExactType` before exposing a partial view. No unmatched form maps to Boolean, Integer, Text, Composite or Reference as a default.
- FR-352-AC-7, role: examined. Excerpt: Repeated reads of one admitted package and its clone return equal complete values or equal typed outcomes. A nested or cyclic type either completes within the finite limit or reports typed exhaustion with the charged work and locus; no call reparses wire/source, grows the call stack with type depth, or grants admission to an unadmitted fixture.
- FR-352-AC-8, role: examined. Excerpt: The public-item and dependency inventories show the projection in `quire-contract-model`, its cycle-free dependency on `quire-exact`, and no QSL checked-expression constructor, checker, evaluator or mirror expression type in IR; Exact has no IR admission dependency. QSL FR-390 AC-6 and AC-11 remain separately verified in QSL.
