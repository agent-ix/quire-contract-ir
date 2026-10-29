---
id: SR-15426
title: 'Base spec review of IR-960 PR #507'
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-348-admit-extension-form-nodes.md,
  spec/checked_package/matrix/TC-459-checked-package-extension-nodes.md
review_set: subset
relationships:
- target: ix://agent-ix/quire-contract-ir/FR-348
  type: reviews
---
# SR-15426: Base spec review of IR-960 PR #507

## Summary

Independent spec-only review of against QSpec #341 FR-469 and TC-951, QSpec FR-467 and Contract IR FR-038. QSpec #363 subsequently reverted FR-469 and fixtures; this PR describes the reader contract for their coordinated restoration.

The review examined FR-348-AC-1 through AC-8 and TC-459. Initial TC-459 wording called all X01..X17 controls negative and treated X02 identity-only mutations as admission outcomes; the author corrected both on this reviewed head. TC-459 now explicitly checks equal repeated held-definition evidence, conflict rejection, and retained-definition admission.

## Verdict

**PASS.** No findings in this analysis lens. The two changed documents pass Quire validation (2/2 grammar-clean; 0 grammar findings).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined scope

- **FR-348-AC-1** (examined): With the FR-467 `retention` definition selected and held, the QSpec `retention Audit` form reads as one `extension`/`extension_form` node with the `Audit` declaration, source owner, one declaration occurrence and whole-form source region, `subject` then `days` reference clauses to Text and bounded-Integer subnodes, and exactly those dependencies in digest order; the full reader admits it. A second extension node kind uses the same closed tag and form without changing the schema or reader enum.
- **FR-348-AC-2** (examined): The reader recomputes the `ExtensionFormNode` key and package identity from the definition, node kind, declared name, owner and clause target keys; changing each one with recomputed identities changes both ids, while changing only the source region changes neither. A retained old key refuses `invalid_package`/`stale-node-key` at that node.
- **FR-348-AC-3** (examined): With no held definition, a selected extension node refuses `unknown_required_capability` with no cause, the node id as `locus` and its body definition pointer; without the matching `definition_selections` row it instead refuses `missing_declaration`/`missing-selection` there. No extension identity appears in `required_features`, and neither case is `unknown_required_feature`. Equal repeated evidence is idempotent and conflicting evidence is rejected without replacement.
- **FR-348-AC-4** (examined): An undeclared node kind refuses `missing_declaration`/`missing-name` at `/body/node_kind`; reordered, unknown and repeated clause names each refuse `invalid_package`/`invalid-value` at the first offending clause, while a missing trailing clause refuses `invalid_package`/`missing-member` at `/body/clauses` naming it.
- **FR-348-AC-5** (examined): A `days` reference to a Text subnode refuses `ill_typed`/`type-mismatch` naming `days`, `Int[1, 3650]` and Text; an absent target refuses `missing_declaration`/`missing-name`; and an extension-kind clause referencing another kind refuses `ill_typed`/`type-mismatch`. Each reports its clause value pointer.
- **FR-348-AC-6** (examined): An inline clause value, wrong form, second or wrong-role occurrence each refuse `malformed_wire`; missing clause dependency refuses `invalid_semantic_graph` at `/dependencies`; a record of the form's name refuses `ambiguous_declaration`/`ambiguous-name` at the later node by node-id order.
- **FR-348-AC-7** (examined): Two extension nodes with failures report the first node by ascending node-id digest and its first failing join; a package with an earlier strict-wire, graph-shape, structural-key or protocol-reference failure reports that stage before the extension join, and a frame failure reports after it.
- **FR-348-AC-8** (examined): The reader loads QSpec's selected-checkout `positive-all-families.json` and `positive-extension-nodes.json` directly, without copying their bytes or schemas into this repository. The test constructs the X01..X17 controls described in QSpec TC-951 against those fixtures, checks X02's identity-only branches through the published preimage rule, and checks each specified reader admission or refusal by its result, code, cause, path and locus as applicable; both positive fixtures admit with held definitions.

- **TC-459** (examined): public reader, selected-checkout fixture intake, constructed TC-951 X01..X17 controls, identity and refusal oracles, held-definition evidence repeat/conflict checks.

## Base checks

The tag/form and typed body are delegated to FR-469; the reader-specific acceptance criteria state observable admission, key, evidence and refusal outcomes. TC-459 names one verification method (`Test`) and a direct reader procedure. No new hash catalog, pin, fixture copy or compatibility reader is introduced; structural digests here are canonical node/package identity values.
