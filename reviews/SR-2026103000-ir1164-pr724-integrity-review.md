---
id: SR-2026103000
title: "Integrity analysis of FR-038 criteria table repair"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
---

## Summary

Ticket IR-1164; PR #724, frozen head b9564e28. Reviewed the one-line table-continuity repair and FR-038-AC-363 through FR-038-AC-373. No requirement text changed; Quire now enumerates all eleven criteria exactly once.

## Verdict

**PASS** — no defects in the changed table structure. AC-370 through AC-373 remain untagged and planned for IR-1165; this review does not claim implementation coverage.

## Examined criteria

- `FR-038-AC-363` (examined): PLANNED/UNRUN (IR-1141). The production V2 reader requires a non-empty top-level `name` and rejects `package_id`, `identity_preimage`, `identity_projection` and any other extra top-level member at that member. It exposes the exact name after admission. Equal names with unequal complete JSON packages remain distinguishable by structural comparison, with object member names and ordered array elements preserved. No content-derived package or graph-node id is computed or exposed; admitted structural node ids remain public.
- `FR-038-AC-364` (examined): PLANNED/UNRUN (IR-1141; QSpec FR-322-AC-272/273/274). The reader admits declared, derived, model-owned and imported nodes with QSpec FR-322 structural IDs `{package,path,ordinal}`. Declared paths equal `declaration.qualified_name`; derived ungrouped paths have `[node_tag,semantic_form,...operands]`; each declared group member keeps its own declaration path, so the three declared members of TC-946 K14 share one label but have distinct paths. An anonymous derived member uses the sole declared member's path when exactly one exists, or a producer-chosen path when zero or multiple exist; no grouped
- `FR-038-AC-365` (examined): PLANNED/UNRUN (IR-1141). Reader first-refusal visits order node ids by UTF-8 package, then path elements with strings before nested node ids and proper prefix first, then numeric ordinal; graph array order and complete body values remain unchanged. A two-defect control reverses structural id order relative to structural order and reports the structurally first defect. Nested path operands and recursion-group members resolve by exact structural equality.
- `FR-038-AC-366` (examined): PLANNED/UNRUN (IR-1141). Lock dependency selections are exactly `{identity}` in ascending UTF-8 identity order. Missing supply refuses `missing_import`/`missing-selection`; an extra `package_id` or `version` refuses `unknown_member`. The host supplies a package with matching `name`; a diamond over one identity admits equal complete JSON values and refuses unequal values `invalid_package`/`conflicting-definition` with both complete lock paths. The reader neither records a transitive closure row nor compares content-derived ids.
- `FR-038-AC-367` (examined): PLANNED/UNRUN (IR-1141). A selected domain document is joined by its model-root identity and every selected source by its exact `{authority,identity}` pair. Changing a selected document's declaration or a source body's meaning is checked by the applicable model, owner, body and graph rules; there is no stale-content refusal. The reader exposes no `CheckedSemanticId`, expected/actual digest pair, candidate-key helper or stale-node-key refusal. A valid edited package retains its name and its declared structural ids where their paths are unchanged.
- `FR-038-AC-368` (examined): PLANNED/UNRUN (IR-1141). Every public V2 refusal carries a non-empty ordered `loci` list; its first entry is primary, and no public `path` or `related_path` field remains. A value refusal has one exact RFC 6901 pointer entry; a malformed JSON or non-canonical byte refusal has one entry with no pointer or source region. A graph-node `locus`, cross-document `document_pointer`, code and cause retain their separate values. A source region appears on a location only when that exact value has one admitted region.
- `FR-038-AC-369` (examined): PLANNED/UNRUN (IR-1141). A repeated `lock.refinement_internals` row, a repeated spelling and a repeated dependency identity each report both concrete entry pointers as two `loci` in declaration order, with the first primary. An unequal dependency diamond reports both conflicting lock-selection pointers in declaration order. Every other refusal has one location entry; these changes preserve each refusal's code, cause and first-defect precedence.
- `FR-038-AC-370` (examined): PLANNED/UNRUN (IR-1164/IR-1165). For an admitted selected object with fields declared as `zeta` then `alpha`, `model_object_fields().fields()` returns `zeta`, `alpha`, not name order; `field("alpha")` selects that same second item and `field("absent")` is `None`. In a chain `Base` declaring `b`, `Mid` declaring `m`, and `Leaf` declaring `l`, the Leaf result is `b`, `m`, `l` with authentic declaring structural node IDs Base, Mid, Leaf respectively, matching the admitted graph nodes rather than the requested Leaf ID. Each returned name and `member_type` remains the effective field that the same 
- `FR-038-AC-371` (examined): PLANNED/UNRUN (IR-1164/IR-1165). A subtype redefining an inherited field retains one exposed field at the hidden field’s earliest declaration-order slot, with the redefiner’s name, derived member type and actual redefining owner’s structural node ID; the hidden owner’s ID is not returned. A diamond that reaches one inherited field twice returns it once at the first inherited slot with its original declaring ID. Reversing two authored supertype edges reverses distinct inherited field order but does not change their declaring IDs. The object’s accessor result agrees with the retained `ordered_ex
- `FR-038-AC-372` (examined): PLANNED/UNRUN (IR-1164/IR-1165). A selected document whose `Leaf` inherits `Base.b` admits with only the Leaf object declaration node in the graph, as QSpec FR-322-AC-30 permits; `model_object_fields(Leaf)` returns `MissingDeclaringNode` carrying the selected IR field identity of `Base.b` for the entire call, without inventing a Base ID or omitting `b`. Adding the authentic Base declaration node makes the call succeed with Base’s structural node ID. Two distinct admitted `model`/`object_type` nodes matching the same selected owner declaration identity but bearing distinct structural ordinals y
- `FR-038-AC-373` (examined): PLANNED/UNRUN (IR-1164/IR-1165). A public API fixture calls `declaring_node_id()` on every successful `CheckedModelField` and exhaustively matches the closed `CheckedModelFieldsError` variants, including `MissingDeclaringNode` and `AmbiguousDeclaringNode`; it cannot construct a `CheckedModelField` with a fabricated ID through public fields. The accessor returns equal results on equal calls and a package clone, charges no new work, performs no second model-resolution walk and does not read raw graph JSON or a copied domain schema. The revised declaration-order and typed-identity controls supers

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
