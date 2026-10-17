---
id: SR-12061
title: "IR-210 slice A gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@5231586114bf3c83bee5d127cb745ff759f3d4d0; tests/it/expression.rs, crates/quire-contract-model/src/expression.rs, spec/core/stakeholder/StR-001-authoritative-contract.md, spec/core/functional/FR-012-anchors-clauses-dependencies.md, spec/model/functional/FR-013-type-system.md, spec/model/functional/FR-014-expression-semantics.md, spec/model/functional/FR-015-definedness.md, spec/core/non-functional/NFR-002-portability-independence.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-014
    type: references
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: references
---

## Summary

Ticket: IR-210 (slice A), PR quire-contract-ir#432, head 5231586. Planless gap analysis over the computed Test Matrix. Plan completion: not assessed.

All coverage was measured with quire 0.36.2 (engine 0.50.2) on four trees: the merge base 1bd6749, origin/main 0747970, the PR head, and a combined tree (the PR head with slice B #430's three files at its head 406e8d7). The method was to compare `quire coverage --json` `minted_targets`, `groups` and the per-criterion `coverage_matrix` binders before and after. The strict gate is not enough on its own: it does not count stakeholder VC rows or a row that loses its last tag.

**Measured, merge base to PR head:**

- `quire coverage --strict`: 43 unbacked rows and 0 contradicted statuses on the merge base, on origin/main and at the head.
- Rows backed: 508/565 to 507/565.
- `minted_targets`: one change only. FR-014-AC-4 goes from backed to unbacked.
- `groups`: one change only. FR-014 acceptance criteria go from 8/9 to 7/9.
- `binding_census`: unchanged (871 candidates, 868 tagged, 867 bound).
- `unmatched_tags`: the three STD-001 tags are gone, and nothing new is unmatched.
- **Rows that lose binders and stay tagged:**
  - FR-012-AC-5: 2 -> 1 (mixed_dependencies)
  - FR-013-AC-4: 2 -> 1 (closed_types)
  - FR-014-AC-1: 3 -> 2 (closed_types, declaration_and_literal)
  - FR-015-AC-1: 2 -> 1 (definedness)
  - FR-015-AC-4: 2 -> 1 (numeric_policies)
  - FR-015-AC-7: 3 -> 2 (numeric_policies, rational_literal_guards)
  - NFR-002-AC-4: 2 -> 1 (closed_types)
- **StR-001-VC-1:** 2 binders -> 1 (`identity.rs` tc_015). It is still backed.
- **Combined tree (this PR plus #430 at 406e8d7):** rows backed fall to 504/565. FR-014-AC-4, StR-001-VC-1, StR-001-VC-2 and StR-003-VC-1 all flip to unbacked. `quire trace --id StR-001-VC-1` finds no binder. Strict coverage still reads 43. The StR-001-VC-1 part of this comes from this PR and is SR-12060 FND-001.

**Already fully qualified tests:**

- There are four, not the three the PR body names: `wide_public_numeric_values_retain_their_type_and_literal_payloads` and `wide_numeric_operations_prove_reduced_results_and_refuse_unrepresentable_ones` in `tests/it/expression.rs`, and `exact_rational_products_reduce_before_i128_range_check` and `saturated_integer_intermediates_clamp_at_declared_bounds` in `range_set_tests`.
- They are untouched. quire does not read their `#[ix_trace_rs::trace(...)]` attribute at all.
- Their credit comes only from the `/// Trace:` doc line (`rust-trace-line`, legacy). The ids that appear only in the attribute are credited by nothing: TC-016 and TC-018 on `wide_public_...`, and TC-016 on the other three.

**Gates rerun by the reviewer at the head:**

- `cargo test --locked --workspace --test it expression`: 22 passed.
- `cargo test --locked -p quire-contract-model --lib range_set_tests`: 8 passed.
- `cargo fmt --all -- --check`: clean.
- `quire validate`: clean.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-014-AC-4 | examined | Rechecking identical declarations, context, expression, and expected type produces structurally equal typed output, ordered diagnostics, and dependency identities. |
| StR-001-VC-1 | examined | Every executable clause has a stable requirement revision, named anchor, typed expression, dependency set, and source location. |
| FR-012-AC-5 | examined | The typed expression tree implements the dependency-source contract and exposes every input, state, field, enum variant, and pure-function reference. |
| FR-013-AC-4 | examined | A valid declaration environment round trips structurally through public constructors/accessors without losing declaration identity, types, bounds, overflow policy, or provenance spans. |
| FR-014-AC-1 | examined | Ill-typed operands, result mismatches, invalid scopes, and non-Boolean clause roots fail at the narrowest source span. |
| FR-015-AC-1 | examined | Every partial operation is accepted only when its obligation is statically established by declarations or dominating guards. |
| FR-015-AC-4 | examined | Reject-overflow arithmetic uses constant/declaration ranges; saturating add/subtract/multiply/negate are total; and zero divisors or possible minimum-divided-by-negative-one overflow fail at the operator span. |
| FR-015-AC-7 | examined | Positive and negative bounded-rational fixtures pin normalization plus numerator/denominator propagation for add, subtract, multiply, divide, and negate. |
| NFR-002-AC-4 | examined | Public type and expression schema/API vocabulary contains no Rust, GUMBO, AADL, HAMR, solver, runtime, pointer-width, or operating-system-path vocabulary. |
| FR-015-AC-8 | context_only | The id credited on the four fully qualified tests through their `/// Trace:` line |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-014-AC-4 now has no test. Its three former claimants never rechecked identical input, so the drop is correct, but the AC is unbacked: 507/565 rows, FR-014 7/9. The strict list does not show it. The PR body states the loss but does not cite the follow-up IR-838. | spec/model/functional/FR-014-expression-semantics.md, tests/it/expression.rs:241 |
| FND-002 | low | Four tests in the slice's own files keep `#[ix_trace_rs::trace(...)]`, which quire does not read. Their credit rests on a legacy `/// Trace:` line, and the attribute-only ids (TC-016, TC-018) are dead letters. The PR body counts three such tests. | tests/it/expression.rs:1829, tests/it/expression.rs:1938, crates/quire-contract-model/src/expression.rs:4067, crates/quire-contract-model/src/expression.rs:4098 |

## Verdict

Coverage outcome is honest except for SR-12060 FND-001 (StR-001-VC-1). That finding is the blocker. This artifact adds no further blocker.

- **FND-001** is the expected, accepted loss. The fix is IR-838 (a recheck-equality test), so its outcome should be deferred to IR-838. Citing IR-838 in the PR body would let a reader of the PR find the follow-up.
- **FND-002** is outside this diff's lines but inside its files. The fix is to change the four to the imported `#[trace]` form, keeping the `/// Trace:` line equal to the attribute, so the attribute ids are credited. It can be done in this PR or deferred to an IR-210 follow-up slice. Removing the four fully qualified spellings also removes a trap: today an attribute lists ids that nothing credits.

Each reduced-count row was verified still backed, with the remaining binder named in the summary. None of the remaining binders over-claims its row, except where SR-12060 FND-002 and FND-003 note a kept over-claim beside an honest binder.

## Dispositions

Round 1, at head 6b789b9. Re-measured on a tree made of origin/main 2641418 with this head's two expression files on top.

- Rows backed: 507/565.
- `minted_targets`: the only change is FR-014-AC-4.
- `groups`: FR-014 is 7/9.
- Strict coverage: 43 unbacked.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | deferred: IR-838 | FR-014-AC-4 stays unbacked until IR-838 adds a test that checks identical input twice and compares the results. That test is open as PR #433. The PR body now cites IR-838 and #433. |
| FND-002 | deferred: follow-up IR-210 conversion slice | Changing the four fully qualified `#[ix_trace_rs::trace]` tests is out of this slice. The PR body now names all four, in its "Not changed / deferred" section, and defers them to the planned conversion slice. No ticket id is named for that slice. |
