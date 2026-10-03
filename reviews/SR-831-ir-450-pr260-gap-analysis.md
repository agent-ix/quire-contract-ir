---
id: SR-831
title: "gap analysis of PR 260 (IR-450 a quantity at a position requires a bound)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@e7fc7a69a1dfb6632a5a1b0e69be368918d51ffc; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-050-checked-package-v2-lowering.md, spec/checked_package/matrix/tests.md, crates/quire-contract-model/src/checked_package/v2/lower.rs, tests/it/checked_package_v2_require_bounds.rs"
review_set: subset
---
# SR-831: gap analysis of PR 260

## Summary

Ticket: IR-450. Plan completion: not assessed. This review checks the merged spec (#257)
against the code and tests at e7fc7a69a1dfb6632a5a1b0e69be368918d51ffc. It covers the
FR-038 quantity paragraph, "unbounded by form", FR-038-AC-8, FR-038-AC-73, TC-050's
"Quantity positions" procedure and TC-052.

FR-038-AC-73 maps to tests as follows. Each tagged test is `#[trace("TC-050",
"FR-038-AC-73")]`, and that binding is correct.

| AC-73 clause | Test, case | Result |
| --- | --- | --- |
| field typed at a `unit` | `..._requires_a_bound_naming_the_unit`, `unit_field` | raises, naming the unit |
| parameter typed at a `compound_unit` | same test, `p_cu` | raises, naming the compound unit |
| `Sequence<Quantity>[0,3]` | same test, `seq03` | raises, naming the unit |
| field typed at a domain over a unit | same test, `dom_field`, `dom2_field`, `cdom_field` | names the unit, not the domain |
| requested `unit` | `..._not_a_quantity_position_merely_by_being_reached` | lowers |
| requested `compound_unit` | same test | lowers, with the unit in `dependencies` |
| unit reached only by `literal.type` | same test, `annotated` | lowers |
| application with a quantity `result_type` over a parameter at `Int[0,9]` | same test, `q_app` | lowers |

Further tests cover the least key among offenders (`both`), the cyclic chain, and
`require_bounds: false` lowering every case. The two extra position kinds (option, alias)
also raise.

AC-8 is unchanged and still backed by TC-052. Adding `Unit`/`CompoundUnit` to
`requires_bound()` breaks TC-052, which is the intended guard against folding a quantity
into the eight forms.

The new tests live in `checked_package_v2_require_bounds.rs`. That file already holds 9
TC-050 `require_bounds` tests (AC-39 through AC-41) with the same helpers. TC-050's own
`checked_package_v2_lowering.rs` covers AC-6. The TC-050 matrix row now names that file for
AC-73. The placement fits.

QSL agreement. QSL FR-097-AC-2 says a quantity's domain takes no finite bound. FR-097-AC-6
requires IR to raise exactly where QSL's extent is Unbounded. QSL's TC-440 suite was run
against this head with a `--config patch`. All six `extent_agreement` tests pass with
`--include-ignored`, including the ignored Measure test. Measure fails against IR main
7ed352b, so this PR is what closes the IR side. The per-application agreement (a constant
quantity application lowers) still passes.

Matrix. The FR-038 row moves AC-73 into "AC-70 through AC-73 implemented", deletes the
AC-73 planned sentence, and keeps row-level 🚧 for the AC-45/46-69/62-64 work. The TC-050
row flips to ✅, which is correct: AC-73 was its only open criterion. No backed-row count is
re-added.

Open PRs #250, #253 and #259 each also rewrite the FR-038 row (and TC-048), so whichever
lands second conflicts on that row. Only #260 touches the TC-050 row. #258 has already
merged and conflicts now (SR-830 FND-001).

IR-450 asks to decide which side is right and align. #257 decided the spec and this PR
aligns the code. The QSL un-ignore of TC-440 Measure is QSL-side work (AD-005 R3-Q4:
QSL-247/QSL-238). It is not an IR acceptance, so "Closes IR-450" is justified. The title
carries no bare ticket id.

Cyclic chains. A cyclic `bounded_domain` chain is admissible: FR-038 allows a cycle inside
one explicit `recursion_group`, and the fixture's `loop_a`/`loop_b` is admitted.
`is_recursive_type` covers only `scalar_type`/`composite_type`, so such a closure lowers
under `require_bounds`, as it did on main. Ending the walk without a quantity is safe.
Every node in a cycle is a `bounded_domain` with exactly one `semantic_type`, so no `unit`
or `compound_unit` lies on it and no quantity is missed. Termination is guaranteed by the
`closure.len()` bound.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-038's quantity paragraph is silent on two choices the code makes. First, a cyclic `bounded_domain` base chain ends with no quantity. Second, `model_population` does not continue the chain (`quantity_chain` maps it to `Other`). The spec text says "a `bounded_domain` whose base chain ends at one" with no carve-out. So for an admitted `model_population` whose `semantic_type` is a unit, a pathological shape QSL never emits, the code lowers where a literal reading of the spec raises. The exclusion follows the SR-810 review note, not the spec. Recommend one sentence in FR-038, for example "The base chain continues through every `bounded_domain` form other than `model_population`; a chain that returns to a domain already on it ends without a quantity." Add it in this PR at the rebase if cheap, with a spec-review of the sentence; otherwise as a follow-up | crates/quire-contract-model/src/checked_package/v2/lower.rs:591 |

## Verdict

Every clause of FR-038-AC-73 and of TC-050's "Quantity positions" procedure has a passing
test that fails under the matching mutation. AC-8 and TC-052 are intact. QSL's Measure
agreement now holds against this head. The one finding is a low spec-clarity gap on two
edge cases where the code's choice is safe.

## Dispositions

Round 1 at c3d89a01928d806bba6b0df7859e1eebb72d877b, on main
a41c0e5f7360d37b8a3ce740108d8077329b36c8.

FR-038's quantity paragraph now says: "The base chain continues through every
`bounded_domain` form other than `model_population`; a chain that returns to a domain
already on it ends without a quantity." It sits right after "names the `unit` or
`compound_unit` node at the end of that chain", which is the right place: it defines the
chain the preceding sentence walks.

It matches the code. `quantity_chain` maps `model_population` to `Other` and the other six
forms to `Base`. A cycle exits after at most closure-size + 1 steps with no insert, which is
the same outcome as ending at a revisited domain. It also matches FR-038-AC-73, which is
unchanged and names neither edge case. `tc_050_a_cyclic_domain_chain_ends_without_a_quantity`
backs the cycle clause.

The sentence is a clarification raised by review and adds no criterion. Matrix and the
TC-050 flip are as in the review pass. No new findings.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | c3d89a01928d806bba6b0df7859e1eebb72d877b |
