---
id: SR-12085
title: "IR-838 FR-014-AC-4 recheck test code review (rust lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@313c53552ceaad178fde05ff1d9d91f99530ef42; tests/it/fr014_recheck.rs, tests/it/main.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-014
    type: references
---

## Summary

Ticket: IR-838, PR quire-contract-ir#433, head 313c535, one commit over origin/main. The diff adds `tests/it/fr014_recheck.rs` with one test, `rechecking_identical_input_gives_equal_typed_output_diagnostics_and_dependencies`, and registers the module in `tests/it/main.rs`. Nothing else changes. The rust-review lane is folded into this file. Coverage accounting is in the gap-analysis artifact (SR-12086).

The test checks 12 cases (5 accepted, 7 rejected) three times each: against a shared environment, against the same environment again, and against a rebuilt identical environment. Accepted cases compare expression, value type, nodes, obligations and dependencies. Rejected cases compare the full `Vec<Diagnostic>` with derived `PartialEq`, so code, severity, message, path, span, related and obligation kind are all compared, in order. The oracle is strong for typed output, dependencies and per-diagnostic content. Only the "ordered diagnostics" leg of the AC is not exercised: every rejected case returns exactly one diagnostic (FND-001).

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-014-AC-4 | examined | Rechecking identical declarations, context, expression, and expected type produces structurally equal typed output, ordered diagnostics, and dependency identities. |
| FR-014 Behavior (dependency order) | context_only | Dependencies deduplicate and sort by the structural tuple `(requirement, dependency kind, observation, path)`. |
| tests/it/fr014_recheck.rs | examined | 12-case recheck test, `use ix_trace_rs::trace;` plus `#[trace("TC-016", "FR-014-AC-4")]` |
| tests/it/main.rs | examined | `mod fr014_recheck;` |
| crates/quire-contract-model/src/expression.rs check_expression | context_only | `DeclarationEnvironment::check_expression` returns `Result<TypedExpression, Vec<Diagnostic>>`; `check_call` and `check_record_literal` accumulate diagnostics across arguments and fields |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The "ordered diagnostics" leg of FR-014-AC-4 is never exercised: all 7 rejected cases return exactly one diagnostic, so a recheck that reorders diagnostics passes (reorder mutant survived), although the checker does emit ordered multi-diagnostic output | tests/it/fr014_recheck.rs:196-340 |
| FND-002 | low | The anti-vacuity guard `accepted >= 4 && rejected >= 4` is looser than the declared table (5 accepted, 7 rejected); dropping a case from either side still passes | tests/it/fr014_recheck.rs:427 |
| FND-003 | low | The `dependencies: Vec<String>` Debug projection and its "dependency order" assertion cannot fail on their own: dependencies are a `BTreeSet`, whose iteration order is fixed by content, so line 406's set equality already decides line 407 | tests/it/fr014_recheck.rs:345-369, 406-407 |

## Verdict

CHANGES REQUESTED on FND-001. FND-002 and FND-003 are cheap to fix in the same round.

What the test gets right:

- **It asserts what FR-014-AC-4 says, and no more of other ACs.** Identical declarations come from the rebuilt `environment()`. Context is the fixed `pre()` execution point. The expression is a fresh clone per check. The expected type and clause-root flag are fixed per case. The extra assertions (accepted/rejected flags per case, a dependency set with Field and PureFunction kinds and at least 3 entries, `ArityMismatch` on the arity case) are anti-vacuity guards, not claims on other criteria.
- **The three check modes are independent.** Measured with mutants (below): a leak keyed only on the rebuilt environment is killed by the `a == c` comparison, and a leak keyed only on repeating the same environment and input is killed by `a == b`.
- **The tag is canonical.** `quire trace --id FR-014-AC-4 --json` binds the test as `rust-trace-attribute`, provenance `canonical`. The `//!` line is only a `mention` citation, and the `/// Trace:` line adds no second claim.
- **Fixtures** use the crate's public constructors (`SymbolName::new`, `Expression::new`, `DeclarationEnvironment::new`, and so on). The crate has no expression builder API and `tests/it/support` has none either. The small helpers repeat the private helpers in `tests/it/expression.rs`, which is this repo's per-file convention.
- **No clock-based assertion, no assertion in any other file.** `main.rs` gains one `mod` line.

FND-001 detail.

- Mutant `reorder_diag` (reverse `Vec<Diagnostic>` on later checks) **survived** in both the repeat-input and the per-call mode. A probe printed every rejected result: each of the 7 codes (ArityMismatch, IllTypedExpression, InvalidScope, InvalidStateObservation, NonBooleanClauseRoot, PotentiallyUndefined, ResultTypeMismatch) comes with a vector of length 1.
- The checker can return more than one diagnostic: `check_call` collects one per ill-typed argument and `check_record_literal` one per bad field. A probe calling a two-parameter integer function with two Boolean arguments returned 2 diagnostics.
- The PR body says the test asserts "ordered diagnostics". Today that is equality of one-element vectors.
- Fix: add a rejected case that yields at least two diagnostics (for example a two-parameter pure function called with two ill-typed arguments, or a two-field record with two wrong-typed fields), and guard it with `assert!(diagnostics.len() >= 2)`, so the order leg can fail.

FND-002 detail. The case flags already pin each case's outcome, so the counts are 5 and 7 today. `assert_eq!((accepted, rejected), (5, 7))` makes the guard match the table.

FND-003 detail. All four dependency mutants (drop first, drop last, and both keyed on a repeated input) failed at line 406, never at 407. The Debug-string projection compares the same set in the same content-defined order. Either drop it, or keep it with a comment saying the order is structural through `BTreeSet`, not separately checked.

Mutant evidence. One throwaway worktree with its own target dir, both deleted afterwards. `check_expression` was wrapped so an env var picks a mutant that changes later results. Three trigger modes were used: the result repeats an input already seen on the same environment (`rep`), every call after each case's first (`ctr`), and only the rebuilt environment or only even calls (`rebuilt_env`, `parity`). Then `cargo test --test it fr014_recheck` was run.

- **Killed in `rep` and `ctr` modes:** message, code, path and span changed on the first diagnostic; diagnostic dropped; diagnostic duplicated; nodes reversed; node dropped; obligation dropped; obligations reversed; first dependency removed; last dependency removed; value type changed; typed expression span changed.
- **Killed in other modes:** rebuilt-environment-only node and message leaks, parity node reversal, and a stale process-wide cache that returns the first result for every later call.
- **Survived:** `reorder_diag` in every mode (FND-001).
- **A known limit, not a finding:** a process-wide cold/warm step leak, where the first call in the process differs from all later calls, is caught only on the first case, which is accepted. On the rejected path, all three checks of each case are "warm" and equal. In a shared `it` binary run with parallel tests, the test cannot own the process's first call anyway. Per-input and per-call leaks, the realistic shapes, are all killed.

Rust lane:

- **Integer conversions:** `start as u32 + 1` in `span()` works on small test constants and copies the `expression.rs` helper. No wire or persistence boundary.
- **Panic surface:** test-only `unwrap()` on constructors with valid constants.
- **Unsafe, locks, async:** none.
- **Boxing:** `Outcome::Typed` boxes `TypedExpression` to keep the enum small. clippy is clean.

Gates on head 313c535, run in the throwaway worktree:

- `cargo fmt --all -- --check` is clean.
- `cargo clippy --workspace --all-targets -- -D warnings` is clean.
- `cargo test --test it fr014_recheck` passes 1 of 1.
- `cargo test --test it expression` passes 22 of 22.
