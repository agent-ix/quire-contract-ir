---
id: SR-15429
title: 'Code and Rust review of IR-960 PR #508'
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; IR-960 PR #508; crates/quire-contract-model/src/checked_package/, tests/conformance_qspec/extension.rs, spec/checked_package/functional/FR-348-admit-extension-form-nodes.md
review_set: subset
relationships:
- target: ix://agent-ix/quire-contract-ir/FR-348
  type: references
---
# SR-15429: Code and Rust review of IR-960 PR #508

## Summary

Independent review of PR #508 against merged FR-348/TC-459 and QSpec FR-469/FR-322 at fixture checkout. The PR changes Rust reader code and tests; spec artifacts are unchanged in this diff.

## Verdict

**FAIL.** The extension clause type join rejects valid record subtypes and can admit an unrelated record shape with a matching name.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Extension record clause checking uses declaration-name equality instead of FR-322 record conformance: a selected descendant record_value_type with a declared supertype is refused, while a composite_type/record with the same name can pass without the selected-model conformance join. | crates/quire-contract-model/src/checked_package/v2/extension.rs:281-300 |

## Examined scope

- **FR-348-AC-1** (examined): With the FR-467 `retention` definition selected and held, the QSpec `retention Audit` form reads as one `extension`/`extension_form` node with the `Audit` declaration, source owner, one declaration occurrence and whole-form source region, `subject` then `days` reference clauses to Text and bounded-Integer subnodes, and exactly those dependencies in digest order; the full reader admits it. A second extension node kind uses the same closed tag and form without changing the schema or reader enum.
- **FR-348-AC-2** (examined): The reader recomputes the `ExtensionFormNode` key and package identity from the definition, node kind, declared name, owner and clause target keys; changing each one with recomputed identities changes both ids, while changing only the source region changes neither. A retained old key refuses `invalid_package`/`stale-node-key` at that node.
- **FR-348-AC-3** (examined): With no held definition, a selected extension node refuses `unknown_required_capability` with no cause, the node id as `locus` and its body definition pointer; without the matching `definition_selections` row it instead refuses `missing_declaration`/`missing-selection` there. No extension identity appears in `required_features`, and neither case is `unknown_required_feature`. Equal repeated evidence is idempotent and conflicting evidence is rejected without replacement.
- **FR-348-AC-4** (examined): An undeclared node kind refuses `missing_declaration`/`missing-name` at `/body/node_kind`; reordered, unknown and repeated clause names each refuse `invalid_package`/`invalid-value` at the first offending clause, while a missing trailing clause refuses `invalid_package`/`missing-member` at `/body/clauses` naming it.
- **FR-348-AC-5** (examined): A `days` reference to a Text subnode refuses `ill_typed`/`type-mismatch` naming `days`, `Int[1, 3650]` and Text; an absent target refuses `missing_declaration`/`missing-name`; and an extension-kind clause referencing another kind refuses `ill_typed`/`type-mismatch`. Each reports its clause value pointer.
- **FR-348-AC-6** (examined): An inline clause value, wrong form, second or wrong-role occurrence each refuse `malformed_wire`; missing clause dependency refuses `invalid_semantic_graph` at `/dependencies`; a record of the form's name refuses `ambiguous_declaration`/`ambiguous-name` at the later node by node-id order.
- **FR-348-AC-7** (examined): Two extension nodes with failures report the first node by ascending node-id digest and its first failing join; a package with an earlier strict-wire, graph-shape, structural-key or protocol-reference failure reports that stage before the extension join, and a frame failure reports after it.
- **FR-348-AC-8** (examined): The reader loads QSpec's selected-checkout `positive-all-families.json` and `positive-extension-nodes.json` directly, without copying their bytes or schemas into this repository. The test constructs the X01..X17 controls described in QSpec TC-951 against those fixtures, checks X02's identity-only branches through the published preimage rule, and checks each specified reader admission or refusal by its result, code, cause, path and locus as applicable; both positive fixtures admit with held definitions.

- **TC-459** (examined): selected QSpec fixtures, reader admission and X01..X17 refusal controls.

## Code and Rust checks

The reviewed diff adds typed extension evidence and wire forms, validates the body before identity, preserves node-key/source-region separation, and exercises direct QSpec fixtures through the public reader. The record branch in `type_matches` at lines 281–300 is the exception. QSpec FR-469 “Clause subnode types” delegates record types to FR-322 “Record conformance”, which requires the same selected model document and a directional declared-supertype walk. Existing `operations::conforms_to_type` shows the model-owner and budgeted conformance path. The new branch neither resolves the selected model nor walks its supertypes. A descendant value passed to a parent-typed extension clause is rejected as `ill_typed/type-mismatch`; a `composite_type/record` whose declaration string matches can be accepted outside the rule. Add a behavioral parent/child and unrelated-record control, then use the selected model conformance path for the extension join. No other code, Rust idiom, panic, unsafe, or gate-scope findings were identified in the changed diff.


## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The fix replaces declaration-name equality with selected `model/record_value_type` owner recovery and a budgeted, directional `DomainModel::conforms_to` walk. It removes the `composite_type/record` name shortcut. The same PR adds positive Base and Child, negative Unrelated and reverse-ancestor, and composite same-name reader controls. Reviewed the PR head. |
