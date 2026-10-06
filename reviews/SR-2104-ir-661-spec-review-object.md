---
id: SR-2104
title: "Spec object review \u2014 IR-661 relationship declaration and relationship_end member"
type: SpecReview
scope: agent-ix/quire-contract-ir@705cef3f7a07370f111f19e8e7d86e1e07fd31d7; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; base 1540b3b6c0e4d167fe1ed9116c296e45e7dff258; Ticket IR-661
review_set: subset
---

## Summary

Cross-reference and semantic-alignment audit of the relationship declaration and `relationship_end` member against FCD 68c0acb (FR-094, FR-095, semantic IR schema), merged QSpec 60630b0 (FR-152, model-complete) and merged QSL bdb910a (FR-094). The installed `SpecReview.analysis` enum has no object-review value, so this artifact omits `analysis`. End fields, required source role, optional target role and the registry split match FCD exactly. The receiver rule diverges from FR-152's effective view, and the frontmatter lacks the new authority edges.

Ticket: IR-661. Method: `spec-review/spec-object-review`. Reviewer model `claude-opus-5-5`, run `236a8098-415e-43ce-aa4b-9a892aa7e251`.

## Verdict

**CONDITIONAL**: one `medium` and one `low` finding. Clean, as examined: `sourceEnd.role` is required with minLength 1 and `targetEnd.role` is optional (schema `relationshipSourceEnd`/`relationshipTargetEnd`); absent inverse with no default name; reader-only checks for presence and non-empty string per FCD FR-094; case-preserving role bytes; the global `relationship/` slot per FCD FR-095; the RelationshipRecord-to-`relation`/`relationship` quotation from QSL FR-094, verified verbatim.

## Scope Examined

- `FR-038#identity-slot` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:975-981
- `FR-038#relationship-ends` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:992-1000
- `FR-038#relationship-end-mapping` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1010-1016
- `FR-038#role-lookup` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1029-1037
- `FR-038#receiver-endpoint` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1039-1047
- `FR-038#navigation-derivation` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1046-1054
- `FR-038#frontmatter` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5-21
- `FR-038-AC-165` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3387
- `FR-038-AC-166` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3388
- `FR-038-AC-169` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3392
- `FR-038-AC-170` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3393
- `FR-038-AC-29` (context_only) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3255
- `FCD FR-095 relationship slot` (context_only) agent-ix/filament-core-data@68c0acba2390eb1593cc003d0a3e144638540a86:spec/functional/FR-095-mint-package-identity-and-provenance.md:111-115
- `FCD FR-094 reader role clause` (context_only) agent-ix/filament-core-data@68c0acba2390eb1593cc003d0a3e144638540a86:spec/functional/FR-094-lower-relationships-operations-and-clauses.md:66-68
- `FCD semantic-ir schema relationship` (context_only) agent-ix/filament-core-data@68c0acba2390eb1593cc003d0a3e144638540a86:schema/semantic/v1/semantic-ir.schema.json:888-920
- `QSpec FR-152 Navigation` (context_only) agent-ix/quire-specification@60630b0d3d5e9cca048d1c314675db3bf9c6e4f2:spec/functional/type-model/FR-152-bind-systems-model-structures.md:118-126
- `QSpec model-complete effective relationship end` (context_only) agent-ix/quire-specification@60630b0d3d5e9cca048d1c314675db3bf9c6e4f2:proposals/quire-v1/definitions/model-complete.md:282-289
- `QSL FR-094 Model declaration nodes` (context_only) agent-ix/quire-spec-language@bdb910ad5ac54e40d22c23480d52cce13c57edb3:spec/functional/FR-094-key-model-owned-reference-population-and-quantity-nodes.md:111-115

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Receiver must be exactly the declared end type; FR-152 effective view admits subtype receivers | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3393 |
| FND-002 | low | FR-038 frontmatter gains no edges for the newly normative FR-152, FR-154, FCD FR-094 and FR-095 | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5-21 |

## Finding Detail

- **FND-001** (medium, confidence medium, check `soundness`, unit `FR-038-AC-170`): Operand 0 must be exactly `Reference<S>` or `Reference<T>` of the declared end type, and "an operand of another endpoint type" refuses `ill_typed`/`operator-ineligible`. Merged QSpec FR-152 resolves `r.name` over `Reference<T>` "in the effective view". model-complete makes a relationship end an end of every effective type that has the relationship member, so a subtype receiver inherits the end. The amendment refuses a subtype receiver with no stated ruling or criterion. That diverges from FR-152 and from FR-038's own inherited-member resolution (AC-29).
- **FND-002** (low, confidence high, check `trace`, unit `FR-038#frontmatter`): The amendment makes merged QSpec FR-152 and FR-154 and FCD FR-094 and FR-095 normative authorities. FR-038's `relationships:` frontmatter gains no edge for any of them (it already lists QSpec FR-322 and QSL FR-094). The dependency is visible only in body prose and is absent from the relationship graph.

## Dispositions

Round 1, reviewed at `agent-ix/quire-contract-ir@c50050da5f20d4af41ab2dd6ea573d5e4b9abacc` (prior review `705cef3f7a07370f111f19e8e7d86e1e07fd31d7`), reviewer model `claude-opus-5-5`, run `ae1a4470-06bc-44c7-90df-81c70be53842`. Every outcome was verified against the fix commit's own text, not the author's receipt. All new criteria and procedures remain PLANNED / UNRUN; no implementation, CI gate or procedure coverage is claimed.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed c50050da5f20d4af41ab2dd6ea573d5e4b9abacc | The exact-endpoint receiver restriction is removed. A receiver qualifies through the endpoint's effective view under quire.model.complete/v1 and FR-152, so an inherited end admits on a subtype and an unrelated receiver still refuses. TC-048 step 6 adds PriorityOrder/PreferredCustomer cases. |
| FND-002 | fixed c50050da5f20d4af41ab2dd6ea573d5e4b9abacc | `references` edges are added for QSpec FR-152, FR-154 and FR-043 and FCD FR-094 and FR-095. All targets exist at the merged revisions and none has a back-edge into quire-contract-ir; the dependency analysis is SR-2107. |
