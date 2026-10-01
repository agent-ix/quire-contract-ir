---
id: SR-645
title: "gap analysis of PR 237 (IR-482 ordered_enum operand family)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@9f8699e5736918f3dccf639148297e4d835d2750; crates/quire-contract-model/src/checked_package/v2/operations.rs, tests/it/checked_package_v2_enum_order.rs, spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/tests.md"
review_set: base
---
# SR-645: gap analysis of PR 237

## Summary

Ticket: IR-482. Plan completion: not assessed (planless). FR-038-AC-42 has four clauses. Each is backed by a TC-048-tagged test that fails under at least one mutant of the fix (SR-644):
- Ordered enum `lt/le/gt/ge`, member literals and parameters, admit: `tc_048_ordering_over_an_ordered_enum_admits`.
- Unordered enum refuses `ill_typed`/`operator-ineligible` at arguments/0: `tc_048_ordering_over_an_unordered_enum_refuses`.
- Two different enums refuse at arguments/1: `tc_048_ordering_across_two_enums_refuses`.
- `eq`/`ne` admit, ordered or not: `tc_048_equality_over_an_enum_admits_ordered_or_not`.

`make spec` at head and at base b995245 lists the same 17 unbacked ids: FR-036, FR-037, FR-039, TC-045, TC-055, TC-058, FR-036-AC-1..5, FR-037-AC-6, FR-039-AC-1..4 and FR-019-AC-5. The three FR-154 stray traces are also unchanged. Backed rows go from 212/239 to 216/243, which is the four new symbols. No code was added without an owning requirement. No stub or tautology was found.

Items outside this PR's diff, noted and not counted as findings:
- An enum reached through a `composite_type`/`alias` resolves to no family: `operand_family(Alias)` is `None` and only `bounded_domain` recurses. The operand check is then skipped, so `enum.lt` over an unordered enum behind an alias would admit. This is the same on main, and FR-322 names only the `bounded_domain` base chain. Found by reading the code, not probed. Worth its own ticket if QSL emits aliases over enums.
- `quantity` (a `unit`) and `aggregate` are never produced, so their checks are skipped. That is IR-484.
- FR-322 says a literal's `type` must have a family its `value_kind` admits (`enum` → `enum` or `ordered_enum`). IR has no such check, before or after this PR. That is a separate gap.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Clean. Every clause of the new AC is backed by a test that can fail. The gate set is unchanged.
