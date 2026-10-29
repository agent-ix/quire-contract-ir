---
id: SR-12060
title: "IR-210 slice A code review (rust lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; tests/it/expression.rs, crates/quire-contract-model/src/expression.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-012
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-013
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-014
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-015
    type: references
  - target: ix://agent-ix/quire-contract-ir/NFR-002
    type: references
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: references
---

## Summary

Ticket: IR-210 (slice A), PR quire-contract-ir#432, over merge. One commit. origin/main has since moved: slice D (#429) landed after the branch point, and it touches other files only. The diff touches two test files. 18 TC-016 tests get `#[trace(...)]`: 12 in `tests/it/expression.rs` and 6 in `range_set_tests` in `crates/quire-contract-model/src/expression.rs`. Both files gain `use ix_trace_rs::trace;`. Each `/// Tracing:` block and its bare `/// ID.` lines become one `/// Trace:` line. A line filter over the diff finds no other change: no test body, assertion or function name changed. The rust-review lane is folded into this file. Each kept id and each dropped id was read against its criterion text in `spec/` and against the test body. Coverage accounting is in the gap-analysis artifact (SR-12061).

The attribute spelling is canonical. `quire trace --file` (quire 0.36.2) credits all 18 tests through `rust-trace-attribute` / `canonical`, and every attribute id equals its `/// Trace:` line. `ix-trace-rs` is a dev-dependency of both the root package and `quire-contract-model`.

Most drops are right. One is wrong: StR-001-VC-1 is dropped from `tc_016_closed_types_expressions_scopes_and_dependencies_conform` because the test asserts no revision, anchor or location. But those are the VC's TC-015 legs. The TC-016 legs are a typed expression and its dependency set, which this test checks. Two kept ids over-claim by the same rule the PR uses to drop others.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| StR-001-VC-1 | examined | Every executable clause has a stable requirement revision, named anchor, typed expression, dependency set, and source location. (Contract-model inspection and conformance fixtures (TC-015, TC-016)) |
| FR-012-AC-5 | examined | The typed expression tree implements the dependency-source contract and exposes every input, state, field, enum variant, and pure-function reference. |
| FR-013-AC-1 | examined | Every closed value-type and declaration construct has positive and negative fixtures with deterministic STD-001 diagnostics. |
| FR-013-AC-2 | examined | Public serialized types contain no Rust, GUMBO, AADL, HAMR, solver, or runtime-specific vocabulary. |
| FR-013-AC-3 | examined | Duplicate names/fields/variants, empty enums, zero or over-unsigned-32-bit collection declarations, invalid integer bounds, absent named types, direct or indirect record cycles, and FR-019 semantic node/depth/collection limit breaches fail before expression validation. |
| FR-013-AC-4 | examined | A valid declaration environment round trips structurally through public constructors/accessors without losing declaration identity, types, bounds, overflow policy, or provenance spans. |
| FR-014-AC-1 | examined | Ill-typed operands, result mismatches, invalid scopes, and non-Boolean clause roots fail at the narrowest source span. |
| FR-014-AC-2 | examined | Canonical representation distinguishes short-circuit conjunction/disjunction from total conjunction/disjunction. |
| FR-014-AC-3 | examined | Current/pre/post observation policy, pure-call arity/signatures, record/enum names, and quantifier-local scope are validated with distinct registered diagnostics. |
| FR-014-AC-4 | examined | Rechecking identical declarations, context, expression, and expected type produces structurally equal typed output, ordered diagnostics, and dependency identities. |
| FR-014-AC-5 | examined | Expression trees at the node/depth limits validate normally; the first node beyond either limit fails with `expression_too_large` before recursive typing. |
| FR-014-AC-6 | examined | Typed expression fixtures derive exact input/state-observation, field-owner, enum-variant, and pure-function dependencies once in structural order, satisfying FR-012-AC-5. |
| FR-014-AC-7 | examined | A guarded product of 120 leaves checks successfully for integer and rational operands because range intervals merge; a sum of more than 64 guarded leaves checks successfully because the range set is widened ...; a divisor whose resulting set contains zero reports `non_zero_divisor`; and every case returns a typed result or diagnostic without panic. |
| FR-015-AC-1 | examined | Every partial operation is accepted only when its obligation is statically established by declarations or dominating guards. |
| FR-015-AC-2 | examined | Replacing a short-circuit guard with a total Boolean operator exposes the previously guarded undefined operation. |
| FR-015-AC-3 | examined | True/false facts from conjunction, disjunction, implication, and negation apply only to their conditionally evaluated branch and never escape a quantifier scope. |
| FR-015-AC-4 | examined | Reject-overflow arithmetic uses constant/declaration ranges; saturating add/subtract/multiply/negate are total; and zero divisors or possible minimum-divided-by-negative-one overflow fail at the operator span. |
| FR-015-AC-5 | examined | Every `potentially_undefined` diagnostic carries the exact closed `obligation_kind`; every discharged node retains the deterministic declaration/guard proof span. |
| FR-015-AC-6 | examined | Bottom-up range sets from literals, declaration bounds, exact-subject guards, field/unwrap/index results, and pure calls accept only results wholly contained in the named numeric type. |
| FR-015-AC-7 | examined | Positive and negative bounded-rational fixtures pin normalization plus numerator/denominator propagation for add, subtract, multiply, divide, and negate. |
| NFR-002-AC-4 | examined | Public type and expression schema/API vocabulary contains no Rust, GUMBO, AADL, HAMR, solver, runtime, pointer-width, or operating-system-path vocabulary. |
| STD-001 | context_only | STD-001 is the stable diagnostic code registry. It is not a minted row: quire lists the old STD-001 tags under `unmatched_tags`. |
| tests/it/expression.rs | examined | 12 `#[trace]` attributes, the `use` line and 12 `/// Trace:` lines |
| crates/quire-contract-model/src/expression.rs | examined | 6 `#[trace]` attributes in `range_set_tests`, the `use` line and 6 `/// Trace:` lines |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | StR-001-VC-1 is dropped from tc_016_closed_types_expressions_scopes_and_dependencies_conform for asserting "no revision/anchor/location". Those are the VC's TC-015 legs. This test asserts the TC-016 legs: it type-checks expressions and inspects the dependency set. With slice B #430 at its current head, the row has zero backers. | tests/it/expression.rs:217-231 |
| FND-002 | low | FR-014-AC-6 stays on tc_016_closed_types_... although the body only checks that a Field, EnumVariant or PureFunction dependency exists (`any()`). It does not check exactness, input/state dependencies, "once" or structural order. The PR drops FR-012-AC-5 from this same test for that reason. | tests/it/expression.rs:348-387 |
| FND-003 | low | FR-014-AC-1 ("fail at the narrowest source span") stays on tc_016_closed_types_..., which asserts diagnostic codes only, never a span. The PR drops FR-015-AC-4 elsewhere for being "code only, no span". tc_016_scope_guard_... does assert the InvalidScope narrowest span, but it does not claim the AC. | tests/it/expression.rs:395-437, tests/it/expression.rs:1118-1119 |
| FND-004 | low | The new `/// Trace:` line is 151 characters, beyond the repo's 100-column rustfmt width. rustfmt does not wrap comments, so fmt-check stays green. Slice B (#430) wraps the same shape over two lines. | tests/it/expression.rs:217 |

## Verdict

REQUEST CHANGES, for FND-001. The fix is small and touches doc and attribute lines only. FND-002..004 are low.

**FND-001 detail.** The verification column of StR-001-VC-1 splits the criterion between TC-015 and TC-016. The TC-016 part is "typed expression" and "dependency set". `tc_016_closed_types_...` type-checks a record literal, an enum literal and a call to typed expressions, and reads each one's dependency set. `tc_016_mixed_dependencies_implement_the_structural_source_contract` makes a stronger case. It checks a clause-root expression (`is_clause_root = true`) and pins its exact ordered dependency set. Measured with quire 0.36.2:

- On the merge base, StR-001-VC-1 has two binders: this test and `identity.rs` `tc_015_identity_anchor_dependency_and_reference_contract_conforms`.
- At this head it has one, the tc_015 test.
- Slice B #430, at its current, drops StR-001-VC-1 from that tc_015 test too. On that branch SR-12065 FND-002 asks to restore it, but no fix commit has landed.
- A tree with both heads applied leaves StR-001-VC-1 with no binder in `quire trace --id`. `minted_targets` shows it backed true -> false, and backed rows fall to 504/565. Strict coverage still reads 43 and does not show this.
- SR-12065 FND-002 had judged this row safe because "it is still bound by tc_016_closed_types_...". This PR removes exactly that binding.

The fix is to keep StR-001-VC-1 on `tc_016_closed_types_...`, or better, add it to `tc_016_mixed_dependencies_...`, in both the `#[trace]` and the `/// Trace:` line.

**Every other dropped id was checked against its text and body. Each drop is right:**

- **FR-012-AC-5 from closed_types:** `any()` existence on three kinds only, with no input or state. It is still backed by mixed_dependencies, which pins the exact ordered set (1 backer).
- **FR-014-AC-4 from closed_types, definedness and numeric_policies:** no body rechecks identical input. The only equality in these tests is `same == environment` (line 241), a rebuilt declaration environment. It backs FR-013-AC-4, not a recheck of typed output, diagnostics and dependencies. The 3 -> 0 loss is honest. The PR body states it. It does not cite IR-838, the ticket for the real test.
- **FR-014-AC-1 from definedness:** no ill-typed operand, scope or root case. It is still backed by closed_types and declaration_and_literal; the latter asserts the narrowest span at line 812.
- **FR-015-AC-4 from definedness:** NonZeroDivisor code only, with no saturating case and no span. It is still backed by numeric_policies, which asserts saturating add is total and the proof span at line 995.
- **FR-015-AC-7 from definedness:** `rational_add` is `is_ok()` only. It is still backed by numeric_policies (2/4 normalises to 1/2) and rational_literal_guards.
- **NFR-002-AC-4 from limits_preflight:** no vocabulary assertion. It is still backed by closed_types (forbidden-vocabulary loop, lines 457-468).
- **FR-013-AC-4 from declaration_and_literal:** no equality or round trip is asserted. It is still backed by closed_types (line 241).
- **FR-015-AC-1 from numeric_policies:** accept-only, with no negative to show "only when". It is still backed by definedness (`TotalAnd` exposes NonZeroDivisor).
- **STD-001 from three tests:** not a row. It was an unmatched tag on the base.

**Kept ids:**

- Correct as kept: FR-013-AC-1/2/3/4, FR-014-AC-2/3/5/7, FR-015-AC-2/3/5/6, NFR-002-AC-4, and FR-014-AC-1 on declaration_and_literal.
- FR-014-AC-6 and FR-014-AC-1 on closed_types over-claim (FND-002, FND-003). Both rows stay backed elsewhere, so neither is a coverage lie.

**Rust lane:**

- The change is attribute and comment only, with nothing in production code.
- The import is a single `use ix_trace_rs::trace;` per file, consistent with #426, #429 and #430.
- No new unsafe code, panic or casts.

**Test bindings:** the six `range_set_tests` and four in-process FR-014-AC-7 tests bind FR-014-AC-7 correctly (merge, widening, nonzero split, divisor through widening).

**Gates:**

- Reviewer reran `cargo test --locked --workspace --test it expression`, `cargo test --locked -p quire-contract-model --lib range_set_tests` and `cargo fmt --check` at the head. Results are in the gap-analysis artifact (SR-12061).
- `quire validate` and strict coverage were measured: 43 unbacked rows on the merge base, on origin/main and at the head.

## Dispositions

Round 1: the fix commit, plus a merge of origin/main. Re-measured with quire 0.36.2 on a tree made of origin/main, which now contains #430 and #431, with this head's two expression files on top. `git merge-tree` reports the merge clean.

- Rows backed: 508/565 -> 507/565.
- `minted_targets`: the only change is FR-014-AC-4.
- `groups`: the only change is FR-014 8/9 -> 7/9.
- Strict coverage: 43 unbacked on main, on the merged tree and at the head.
- Gates rerun: `--test it expression` 22 passed, fmt clean, `quire validate` clean.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | StR-001-VC-1 is now on `tc_016_mixed_dependencies_implement_the_structural_source_contract`. That test checks a clause-root typed expression (`is_clause_root = true`) and pins its exact ordered dependency set (lines 1367-1409), which is the TC-016 leg. On the merged tree the row has two binders, both canonical: this test and #430's tc_015. `minted_targets` shows it backed. |
| FND-002 | fixed | FR-014-AC-6 is dropped from closed_types. The row is still backed by mixed_dependencies. |
| FND-003 | fixed | FR-014-AC-1 moved to `scope_guard_...`, which asserts the InvalidScope diagnostic at its narrowest span (lines 1124-1125). The row is backed by scope_guard_... and declaration_and_literal_.... |
| FND-004 | fixed | The long line is split into two `/// Trace:` lines, each under 100 columns. Together they are exactly the attribute's id set, and quire binds every id as `rust-trace-attribute` / canonical. |
