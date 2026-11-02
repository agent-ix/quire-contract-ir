---
id: SR-2026110283804
title: "IR-1003 AC-6 dependency review"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; FR-352-AC-1 through AC-11"
review_set: subset
---

## Summary

Ticket: IR-1003; PR #838. Frozen HEAD: 6e19a70e779b3ec68d8f08473b18ca996a58d70b. Scope is the FR-352-only final commit; FR-038 AC-435/436 inherited in the supplied baseline comparison are excluded. No applicable AssuranceProfile was found under spec/.

This is legitimate cross-contract reconciliation: Description and Outputs already distinguish unavailable identity from unsupported shape, and AC-4/5/9 prohibit bare-key or name joins. The revised AC-6 explicitly requires success whenever complete representation and authoritative witnesses are available. It does not permit arbitrary error acceptance or narrow the set of admitted forms. No new identity is reconstructed and no QSL evaluation behavior is copied.

The public source-union reproduction admits and then project_exact_type with empty identities and limit 2 returns UnavailableIdentityWitness at the requested composite_type/union node with work 2 (qualification-logs/ir1003-projection-limits/ac6-union-receipt-output.log). This supports the causal classification only. SelectedExactIdentity::Union and SelectedExactDeclarationForm::Union exist, but matches_admitted has no Union case; AdmittedDeclarationForm has only Model and DeclaredUnit, and admitted_selected_declaration has no source-union route (exact_projection.rs:59-101). Therefore no positive union reader route is established today. QSL FR-390-AC-27 (qualification-inputs/IR-1003-qsl/spec/functional/FR-390-evaluate-a-protocol-node-expression-from-a-checked-package.md:709) remains an outstanding positive identity/member/payload and evaluation parity obligation, expressly preserved by the changed Behavior.

The supplied Quire validation and properties-comparison logs retain all eleven canonical AC IDs and change only AC-6 text. No production code, test, trace tag or computed matrix changes; no coverage or production-conformance gain is claimed. Cargo and hosted execution gates are inapplicable to this Markdown-only review.

Classification: FR-038 admission and FR-349 retained terms are enablement; FR-352 read projection is a feature consumed by QSL FR-390. Order: retained admitted declarations and authoritative producer facts → IR joined projection → QSL reader/evaluation parity. QSL FR-400 construction remains upstream context. No new dependency edge or cycle; FR-390 reference describes external ownership rather than reverse implementation dependency.

## Examined scope

- FR-352-AC-1 (context_only): An external crate reads an admitted node's semantic type through one public IR-model API and receives Exact `Boolean`, `Integer`, or `Int` for independently authored Boolean, Integer, and inclusive ranges `[0,1000]`, `[0,3]`, and `[0,9]`; it inspects no JSON and reparses no source. An endpoint outside Exact's representable domain returns a typed refusal, never a narrowed interval.
- FR-352-AC-2 (context_only): Through the public `ExactTypeProjection`, admitted selected `Receipt`, `Attempt`, `Flag`, and `Tick` return distinct `Composite(NodeKey)` identities and public `SelectedRecordView` values. Each view contains every effective member in selected declaration order with selected IR identity, name, Exact `MemberId`, projected type and presence; `Tick.count: Int[0,9]` appears even with no read of `count`. The view can supply Exact `FieldDeclaration` inputs that reject a complete record missing a required member or carrying an out-of-range `count`; a member identity absent from the producer, or a simi
- FR-352-AC-3 (context_only): An admitted reference to selected `Node` projects to `Reference(EffectiveId)` matching its authoritative object-type witness. The object declaration alone is not returned as a standalone Exact value-type variant. A reference to another selected object type, a foreign package, or a dangling target never produces Node's identity.
- FR-352-AC-4 (examined): Exact structural identity is matched by package, full resolved path, and ordinal. A changed path segment, ordinal, owner package, or selected declaration refuses with a named typed outcome; duplicate or conflicting witnesses and ambiguous selected owners refuse before any value-type result. Alias equality and display-name equality cannot establish a join.
- FR-352-AC-5 (examined): Admitted String, Uuid, and Timestamp scalar types map to their respective Exact variants when present. A selected declared unit maps through a witnessed declared `UnitId`; a compound unit without its authoritative identity refuses. A selected union uses `Composite` only with a verified union witness; dimension and alias shapes without faithful Exact representations return `UnsupportedExactType` with their own form and node.
- FR-352-AC-6 (examined): Every other admitted V2 type form either projects to an Exact variant with its full bounds, mode, members and identity, returns `UnsupportedExactType` when that complete shape cannot be faithfully represented, or returns `UnavailableIdentityWitness` when a required authoritative opaque identity witness is absent. Each refusal names the requested node, first failing type node and form, and charged work, before exposing any partial view. An admitted union without its authoritative declaration and payload identity witness takes the missing-witness outcome; a declaration key alone does not establi
- FR-352-AC-7 (examined): Repeated reads of one admitted package and its clone return equal complete values or equal typed outcomes. A nested or cyclic type either completes within the finite limit or reports typed exhaustion with the charged work and locus; no call reparses wire/source, grows the call stack with type depth, or grants admission to an unadmitted fixture.
- FR-352-AC-8 (examined): The public-item and dependency inventories show the projection in `quire-contract-model`, its cycle-free dependency on `quire-exact`, and no QSL checked-expression constructor, checker, evaluator or mirror expression type in IR; Exact has no IR admission dependency. QSL FR-390 AC-6 and AC-11 remain separately verified in QSL.
- FR-352-AC-9 (examined): A QSL-side selected-model identity producer supplies `SelectedExactIdentities` alongside a checked package; an external caller passes it to the public projection API. The IR join accepts identities only for matching admitted selected-document, declaration form and original IR declaration or field identities. Absent, conflicting, foreign-document and wrong-form entries return the named typed outcome; a caller-supplied bare `NodeKey`, `EffectiveId`, `MemberId`, or node-name pair cannot enter the projection as an authoritative witness.
- FR-352-AC-10 (context_only): A QSL-stage `SelectedOperationMemberRow` for an admitted model-owned operation anchor joins its full structural node, selected document, declaring model form and original model/operation member identities to the selected operation identity; the FR-354 callable read uses the joined row to find the admitted body and frame. Two admitted anchors of one operation at distinct execution points remain distinct rows. A missing row, wrong document/form/member, foreign anchor, duplicate conflicting row, or name-only match returns a named typed outcome without a callable view. A source-declared anchor wit
- FR-352-AC-11 (context_only): QSL checking/emission supplies one original application `NodeKey` and authored `Location` per emitted equality application occurrence, and original dimension-declaration, base-dimension-term and ordered conversion-edge-target `NodeKey`s for every selected quantity fact. Each row carries a full emitted structural node, selected producer document/context and its fact role; application rows also carry exact occurrence. The FR-353 join refuses absent, foreign, wrong-form, duplicate, conflicting or mismatched rows with a typed structural locus and bounded work. Neither `CheckedNodeId`, display name
- FR-352-Behavior (examined): For other admitted scalar and structural forms, including rational, decimal,
float, quantity, text, enum, option, collection, map, bounded quantity, and
population, the projection SHALL produce the corresponding Exact variant only
when its complete bounds, mode, member, unit, and identity facts can be
represented; otherwise it SHALL return `UnsupportedExactType` naming that form
and type node. Where the shape is faithfully representable but a required
authoritative opaque identity witness is absent, the projection SHALL instead
return `UnavailableIdentityWitness`; it SHALL NOT classify a missi

## Verdict

PASS for the scoped specification clarification. Positive union reader/evaluation parity remains outstanding; this review is not production acceptance.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
