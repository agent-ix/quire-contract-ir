---
id: SR-15430
title: 'Gap analysis of IR-960 PR #508'
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir@f85a1b91c2b83c1f171a04215900f5831c17f0a2; IR-960 PR #508; crates/quire-contract-model/src/checked_package/, tests/conformance_qspec/extension.rs, spec/checked_package/functional/FR-348-admit-extension-form-nodes.md
review_set: subset
relationships:
- target: ix://agent-ix/quire-contract-ir/FR-348
  type: references
---
# SR-15430: Gap analysis of IR-960 PR #508

## Summary

Independent review of PR #508 at `f85a1b91c2b83c1f171a04215900f5831c17f0a2` against merged FR-348/TC-459 and QSpec FR-469/FR-322 at fixture checkout `053b59869f5bc767ae1f35b30c7d00afbe63fda3`. The PR changes Rust reader code and tests; spec artifacts are unchanged in this diff.

## Verdict

**FAIL.** The extension clause type join rejects valid record subtypes and can admit an unrelated record shape with a matching name.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | TC-459 has no positive descendant-record clause or negative unrelated same-name record control, leaving FR-348-AC-1 admission and QSpec FR-469 record conformance unverified through the public reader. | tests/conformance_qspec/extension.rs:483-521 |

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

## Coverage

Plan completion: not assessed (planless review). FR-348-AC-1 through AC-8 have `#[trace]`-tagged TC-459 tests, and the positive fixtures and constructed X01..X17 controls exercise the public reader. The `tc_459_extension_clause_targets_are_typed_and_present` test checks Text versus bounded Integer and missing targets, and the extension-kind test checks a mismatched kind. No test constructs a selected parent and descendant `model/record_value_type` clause or a same-name record from another model document. This gap allows the code defect in SR-15429 to survive the focused controls. No untagged new TC-459 test, stub, or reverse-ownership gap was identified in the changed diff.


## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The new TC-459 controls exercise direct selected-model Base and Child admission, unrelated and reverse-ancestor refusal, and composite same-name refusal through the public reader. The two new targeted tests passed independently on the fix head. Exact reviewed head: `8826263`. |
