---
id: SR-12003
title: "IR-832 spec gap and falsifiability review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir (PR #425); spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

Ticket: IR-832. Plan completion: not assessed. Each new AC was checked for whether a test could fail it and whether TC-048's procedure can reach it from a reader-admitted fixture, against `origin/main` code. All four ACs are PLANNED/UNRUN and untagged, as stated. No code was in scope.

Checked: `scalar_application_operands` (`v2/scalar_operands.rs`) computes a `GraphChild` parameter operand's range with `type_range(semantic_type)`. That returns `UnboundedRange` for a `scalar_type/integer` type and `MissingRange` for anything except a `bounded_domain/integer_range`. `parameter_defect` admits a `bounded_domain` binder type, so an AC-261 fixture is possible only when the parameters are typed at a finite integer range.

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-038-AC-258 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | For an admitted `value`/`parameter` node, `parameter_declaration` returns `Ok` whose `node` equals the supplied id ... a level of `10` is returned as `"10"` and not read as an integer. |
| FR-038-AC-259 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | `parameter_declaration` over an id absent from the graph refuses `UnknownNode`; over an existing `value`/`literal`, an `application` node and a `function` node it refuses `NotParameter` for each; ... `MalformedBody` for each. |
| FR-038-AC-260 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | Two functions in one admitted package whose parameters share a name, level and type reference one `value`/`parameter` node, and `parameter_declaration` over it returns the same `name` and `level` regardless of which function was used to reach it ... |
| FR-038-AC-261 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | For an admitted `quire.op.integer.add` application whose two operands reference parameters `a` and `b`, calling `parameter_declaration` on each `GraphChild` id returned by `scalar_application_operands` yields `a` then `b` ... An `InlineLiteral` operand has no node id to supply, so the caller's join sees no declaration for it. |
| TC-048 "Typed parameter declaration" | examined | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md | ... add applications over parameters `a` and `b` in both argument orders (separately admitted packages). ... Do not construct the expected name or level from the accessor under test. |
| scalar_operands.rs type_range | context_only | crates/quire-contract-model/src/checked_package/v2/scalar_operands.rs | if node.node_tag == "scalar_type" && node.semantic_form == "integer" { return Err(Error::UnboundedRange); } |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-261 and the TC-048 procedure require `scalar_application_operands` to return `Ok` over parameters `a` and `b`, but neither states that the parameters must be typed at a finite `bounded_domain/integer_range`. A parameter typed `Int` makes the call refuse `UnboundedRange`, so the AC cannot be exercised as written. State the binder type (for example `Int[0, 9]`, as in QSL P4). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4497; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:2258-2260 |
| FND-002 | low | AC-260's first clause can fail only if the accessor is impure. Both functions reference one node id, so two calls with that id return equal results whatever the implementation. Whether the two functions share one node is a property of the self-built fixture, not of the accessor. The falsifiable parts are the distinct ids at levels 0 and 1 and the no-function-member inspection. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4496 |
| FND-003 | low | AC-261's last sentence ("the caller's join sees no declaration for it") describes caller behaviour that no IR test can fail. This contradicts the section's statement that the join is not an IR obligation. Either drop it or restate it as an IR-observable fact, such as "the accessor is not called for an `InlineLiteral` operand, which carries no `CheckedNodeId`". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4497 |

## Verdict

PASS WITH FINDINGS. AC-258 and AC-259 are falsifiable and reachable, including the level `"10"` text case and crate-internal mutation for `MalformedBody`. AC-259 bundles several refusal variants, which matches the sibling FR-038-AC-162 style. FND-001 should be fixed before the code PR so its test author does not discover it through a refusal.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 : AC-261 and TC-048 now type `a` and `b` at `Int[0, 9]` (a finite `bounded_domain`/`integer_range`) and say why: an unbounded Integer refuses `UnboundedRange`. |
| FND-002 | fixed | Round 1 : the tautological shared-node clause is gone from AC-260 and from TC-048. AC-260 now asserts distinct ids and levels for same-name parameters, plus a compiled field-by-field construction showing that the result has no function, owner or declaration member. |
| FND-003 | fixed | Round 1 : AC-261's last sentence is now a type fact: `CheckedScalarOperandChild::InlineLiteral` carries no `CheckedNodeId` to pass to the accessor. |
