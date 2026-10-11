---
id: SR-7301
title: gap-analysis review of IR-703 PR 347
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir@8d59a9dac16e8a7d6940a67b1a1e80377ebc38ed; crates/quire-contract-model/src/checked_package/v2/comparison_operands.rs,
  crates/quire-contract-model/src/checked_package/v2/model_fields.rs, crates/quire-contract-model/src/lib.rs,
  tests/it/checked_package_v2_comparison_operands.rs, tests/it/checked_package_v2_frame_entries.rs,
  Makefile
review_set: subset
---

## Summary

Scoped criterion-to-test audit: all eight ACs have static trace tags, but four tagged obligations lack behavioral oracles. Plan completion: not assessed.

## Verdict

**CONDITIONAL** — Four behavioral evidence gaps require tests before acceptance.

## Examined units

- FR-038-AC-202: PLANNED/UNRUN (IR-703). For an admitted invariant, precondition or postcondition clause with an authentic claim occurrence and a direct condition reference to a binary integer.eq/ne/lt/le/gt/ge application, the typed accessor returns the actual clause id, claim, kind, comparison id and closed operator; the comparison id alone supplies no observation.
- FR-038-AC-203: PLANNED/UNRUN (IR-703). For two eligible reads, the accessor returns exactly ordinals 0 and 1 in authored order, preserving distinct StateField or OperationInput provenance and exact child identities; reversing arguments reverses entries without sorting or deduplication. CG maps only typed StateField to State and OperationInput to Input for its FR-008 census; no name-based V1 kind inference occurs.
- FR-038-AC-204: PLANNED/UNRUN (IR-703). For a fresh QSL-produced and IR-admitted ConfigVersion postcondition `self.versionNumber = pre(self.versionNumber)`, the shared record.project id appears in both entries, the direct path is Post and the state.pre path is Pre. Synthetic hand-built topology is separately labelled and cannot discharge this producer-origin assertion.
- FR-038-AC-205: PLANNED/UNRUN (IR-703). A read whose selected model field or operation-parameter type supplies a finite integer range returns its exact inclusive i128 endpoints, including i128::MIN and i128::MAX; a literal in either position returns an exact i128 singleton with its graph or inline identity. Unbounded Integer, an out-of-i128 literal, and defensive unavailable model member type produce their distinct typed refusals without truncation, body-derived guess or partial result.
- FR-038-AC-206: PLANNED/UNRUN (IR-703). The accessor accepts direct self.field reads and operation-parameter references in the reader-checked FR-341 aggregate, plus one state.pre wrapper around a StateField in a postcondition. Direct StateField is Current in invariant/precondition and Post in postcondition; direct OperationInput is Current. An alias, result slot, unrelated parameter, nested project or other wrapper is IneligibleOperand. A read plus literal is eligible; two literals return NoRead.
- FR-038-AC-207: PLANNED/UNRUN (IR-703). Unknown clause, wrong node form, absent claim, unsupported clause kind, missing comparison, unknown or ineligible operator, invalid arity, missing child and malformed operand yield their respective typed refusals in the stated first-defect order. Each carries clause id, claim, structural condition/operand path and reached child id when available, without a claimed unique child source region or partial result; malformed post-admission graph never panics.
- FR-038-AC-208: PLANNED/UNRUN (IR-703). Two clause claim occurrences over one content-addressed comparison produce equal contextual operands apart from the supplied claim. The accessor returns no comparison, project or literal expression occurrence and claims no claim-to-child source-map association; a separate source-map lookup may enumerate regions.
- FR-038-AC-209: PLANNED/UNRUN (IR-703). An external Rust consumer calls the public accessor with &CheckedNodeId and &CheckedOccurrence and exhaustively matches closed operator, provenance, observation, operand identity and refusal types without body JSON. Repeated and cloned calls return equal values and preserve package equality and admission; CG needs no private V2 decoder or new per-use wire member.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-445 does not exercise invariant or precondition success; only a postcondition reaches a positive accessor assertion. Current observation for a direct StateField in those two clause kinds can regress while this suite remains green. | tests/it/checked_package_v2_comparison_operands.rs:224 |
| FND-002 | medium | TC-445 asserts one producer pair of StateField reads and one StateField plus OperationInput, but never reverses operands or asserts two eligible read identities in swapped authored order. An accessor that sorts by read identity would pass. | tests/it/checked_package_v2_comparison_operands.rs:344 |
| FND-003 | medium | The only i128::MIN/MAX test calls the private literal_range helper; no admitted selected field or operation input supplies those endpoints. Unbounded Integer has no behavioral assertion, and a graph-child literal identity is not asserted. A wrong member_range branch would pass TC-445. | crates/quire-contract-model/src/checked_package/v2/comparison_operands.rs:802 |
| FND-004 | medium | The mutation test checks unknown clause, absent claim, unsupported kind, ineligible operator, missing child, out-of-i128 literal and NoRead. Wrong node form, missing comparison, unknown operator, invalid arity and malformed operand have no asserted typed refusal or locus; the external enum match is compile coverage only. | crates/quire-contract-model/src/checked_package/v2/comparison_operands.rs:723 |

## Coverage

Plan completion: not assessed. Quoin matrix returned tagged static bindings for FR-038-AC-202..209; no run evidence is bound in Quoin. The named QSL gate is separately reported as passed in the coder brief, not rerun here.
