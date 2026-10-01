---
id: SR-637
title: "code review of PR 234 (merge and cap range sets in the expression checker)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@d8b1f16c424733c9e4d807e11ecc02ff52f6c6e0; crates/quire-contract-model/src/expression.rs, tests/it/expression.rs"
review_set: base
---
# SR-637: code review of PR 234

## Summary

Ticket: IR-473. Code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD` (origin/main 8371caa is the merge base; main has not moved). The PR sorts and merges integer and rational-numerator range sets after every binary operator (`canonical_ranges`, `bounded_ranges`), refuses an operand or result of more than 64 disjoint intervals (`MAX_RANGE_SET_SIZE`) with `potentially_undefined` / `checked_range`, splits the failure cause into `RangeFailure::{Intermediates, SetTooLarge}`, and canonicalizes after the `!= 0` guard split.

Measured by the reviewer, all memory experiments under `ulimit -v 3000000`:

- Old behaviour reproduced: a scratch harness against origin/main 8371caa, `x>=-1 && x<=1 && x!=0 && (x*...*x)>0` with 32 leaves, printed `memory allocation of 2147483648 bytes failed` and aborted (exit 134). 20 leaves took 632 ms.
- At head: 32 leaves pass in 1.3 ms, 120 leaves in 23 ms; 2000 leaves stop at preflight (`expression_too_large`).
- Random expression trees (add, subtract, multiply, divide, remainder, negate over three guarded integer inputs and two guarded rational inputs, Reject and Saturate types, guard widths 1 to 3 and point guards `|x| = 1`): 3000 trees of up to 400 leaves and 3000 trees of up to 2500 leaves. No abort, worst check 209 ms, 276 refusals by the new limit.
- Gates at head with CARGO_TARGET_DIR in the review worktree: `make fmt-check lint test corpus` exit 0 (235 tests passed, conformance corpus all match), `make deny` exit 0, `make spec` 17 unbacked rows, the same count as origin/main.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Refusing a range set of more than 64 intervals rejects defined expressions where a sound widening loses nothing: a sum of 64 guarded `x` leaves (`x` in {-1, 1}, type range +/-1000000) compared `>= -1000` is reported `potentially_undefined` / `checked_range`. With `OverflowPolicy::Saturate` the operator carries no `checked_range` obligation, so the diagnostic names an obligation that does not exist | crates/quire-contract-model/src/expression.rs:2522-2528 |
| FND-002 | low | `canonical_ranges` after the `!= 0` split in `refine_integer_ranges` and `refine_rational_ranges` never changes anything: `refine_range` is only ever passed `numeric_range(declared type)`, a single interval, so the split yields at most two disjoint intervals. Deleting both lines leaves every test green | crates/quire-contract-model/src/expression.rs:3332 |

## Finding Detail

- FND-001: reproduced with `K=1` sums of 63, 64 and 100 leaves under both Reject and Saturate: 63 passes, 64 and 100 fail with `integer range set exceeds the checked range-set limit`. Recommended fix, which needs no new diagnostic code: when a canonical set has more than 64 intervals, coalesce the narrowest gaps until 64 remain, rather than returning `SetTooLarge`. The result is a sound superset. Its minimum and maximum are unchanged, so the named-bounds check (`within`) and the static index-bound check give the same answer. The only proof that can be lost is a hole at zero, and that loss surfaces as the divisor's own `non_zero_divisor` obligation, which is truthful. `check_integer_operator` already widens a Saturate result to the full type range (expression.rs:2450-2457), so widening is not new to this function. Memory stays bounded at 64 x 64 pairs per operator. If refusal is kept, it should at least not run for Saturate types, and FR-014-AC-7 and STD-001 should say that the checker declines rather than that an obligation is undischarged.
- FND-002: harmless. Either delete the two lines or keep them as defensive code; the PR body and FR-014 present them as load-bearing ("after every non-zero guard split").

## Verdict

Approve the safety fix. Medium FND-001 should be fixed in this PR; it is a small change. The checks below were made and found correct:

- Merge semantics: `canonical_ranges` sorts by `(min, max)` and merges when `next.min <= last.max + 1` with a saturating add. Over integers, that produces exactly the same set, neither wider nor narrower. At `last.max == i128::MAX` every later interval is contained, so the saturating comparison is correct. Rational numerator ranges are integer numerator sets: `contains_zero`, the bound checks, the `exact` filter in `refine_range` and the `!= 0` split all treat them as integers. Adjacent-merging them therefore changes no represented value.
- Operator coverage: Add, Subtract, Multiply, Divide and Remainder all go through `integer_pair_range` / `rational_worst_case`, and the result is merged once. Divide only runs after `contains_zero(right)` has refused a zero-spanning divisor. Negate maps intervals one to one, so the count never grows. Its output is not re-sorted, and Saturate clamping can produce duplicate intervals, but the next binary operator canonicalizes it. Comparisons produce no range. Lower and upper guards only trim.
- Overflow: endpoints are `i128`, and every stored range is bounded by an `i64` type range: Reject refuses a range outside it, and Saturate widens to it. Products of two endpoints therefore fit in i128, and any deeper overflow uses checked operations and becomes `RangeFailure::Intermediates`. Remainder's `abs()` gets `i64`-bounded inputs.
- Boundedness: every growth path goes through a binary operator, leaves carry at most 2 intervals and negate preserves the count. Each operator therefore allocates at most 64 x 64 = 4096 pairs plus a sort. Repeated operators cost at most about 4096 log 4096 steps per node within the 10000-node preflight, and the fuzz worst case was 209 ms. No other range-set representation exists in the file: `NumericRange` is the only one, and it never reaches `TypedExpression` output.
- Oracle strength (mutations run in the throwaway worktree and reverted): merge only on overlap (`+0`) turns the unit test red; over-merge (`+2`) turns unit and integration tests red; cap 65 turns both red; integer guard split `min <= -1` to `< -1` turns the integration tests red. Survivors: the rational result merge and cap (SR-638 FND-001), the refine canonicalization (FND-002 above), the operand-size pre-check (redundant, because the result cap catches the same input and operands never exceed 64 through the public API), and the rational guard split (pre-existing, SR-638 FND-002).
- Rust idioms: `RangeFailure` is a small typed `Copy` enum with static messages, and no new panic, unsafe or blocking surface is added. Messages are distinct per family. The existing `unreachable!()` and `extrema` unwrap are unchanged.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | The capped child-process test is not portable. It runs `sh -c "ulimit -v 3000000 && exec ..."` with no platform gate. On macOS, `ulimit -v` either fails, which turns `make test` red, or is accepted but not enforced, so a merge regression would run uncapped and exhaust host memory. On Windows there is no `sh`, so `.output().unwrap()` panics | tests/it/expression.rs:1751-1765 |
| FND-004 | low | The widening tie-break closes the leftmost gaps first, so a symmetric set loses its hole at zero even when 64 other gaps could close instead. In the targeted differential, 323 of 4000 trees with point guards checked under exact ranges and reported `non_zero_divisor` after widening. This is sound but avoidably imprecise | crates/quire-contract-model/src/expression.rs:2496-2505 |

### Detail (disposition pass 1)

- FND-003: on Linux the guard works. The mutations that drop the merge or the widening turn this test red, and the child stays within its cap. Not run on macOS here, so confidence is medium. Fix: put the capped child behind `#[cfg(target_os = "linux")]`. Elsewhere, run the check in-process only at leaf counts whose unmerged growth is bounded, such as 16 leaves (2^16 intervals). The repeated-product unit test already pins the merge itself. A failed `ulimit` exits non-zero and so fails loudly. A renamed test path makes the child report "0 passed", which also fails loudly.
- FND-004: when choosing which gaps to close, treat a gap that straddles zero as the widest, so it closes last. This keeps divisor proofs wherever 64 other gaps can be closed.

### Over-approximation check per consumer (disposition pass 1)

Every endpoint of a widened interval is an endpoint of an original interval. The widened set is a superset with the same minimum and maximum.

| Consumer | Direction under widening | Why it is sound | Probe |
| --- | --- | --- | --- |
| `contains_zero` on an integer divisor (expression.rs:2407) and a rational divisor (2604) | can refuse more, never less | a superset contains zero whenever the exact set does | differential: only exact ok -> widened `non_zero_divisor` |
| Named-bound `within` checks on integer and rational operators and on negation | unchanged | the checks read every interval's endpoints, the hull is unchanged, and pairwise extrema use original endpoints. A divisor interval cannot get closer to zero unless a closed gap spans zero, which `contains_zero` refuses first | differential: no change in `checked_range` outcomes |
| Saturate collapse to the type range (2447-2454) | unchanged | it is driven by `within` | differential covers the Saturate policy |
| Saturated negation clamp plus `canonical_ranges` (2864) | count does not grow | clamping a superset gives a superset | mutation removing the canonicalization survives, which is harmless |
| Static index bound on a collection literal (2207) | can refuse more, never less | every interval must lie in 0..len, which is stricter for a superset. Index-type sets are a single interval anyway: `!= 0` on an unsigned type leaves [1, N] | inspection |
| Guards `!= 0`, `>=`, `<=` (`refine_range`) | never see a widened set | refinement only applies to leaf declared ranges, a single interval, never to operator results. A `!= c` guard with c not zero produces no range fact at all | inspection |
| Comparisons, "always true/false" folding | not a consumer | `check_compare` and `guard_facts` are syntactic and read no range set | grep: no `NumericRange` use outside the rows above |
| Rational `exact` filter (`refine_range`) | not affected | `exact` is `None` on every range-computed result, and the filter runs only on refined leaf ranges | inspection |

Probe: exact-range build (MAX_RANGE_SET_SIZE set to 1000000 in a scratch copy) against the head build. Both ran under `ulimit -v 3000000` with types of +/-300, over 12000 targeted trees: wide guarded sums used as divisor, dividend, remainder divisor and multiplicand, under Reject and Saturate, guard widths 1, 2 and random. The only difference was 332 trees that checked under exact ranges and reported `non_zero_divisor` after widening. No tree that the exact build refused was admitted by the widened build. No consumer admits or proves more. No HIGH.

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | medium | The FND-003 fix leaves the module-level constants `RANGE_SET_CHILD` and `PRODUCT_TEST` used only inside the `#[cfg(target_os = "linux")]` block. On macOS they are dead code, so `make lint` (`clippy --all-targets -- -D warnings`) fails with "constant ... is never used" | tests/it/expression.rs:1726-1728 |

### Detail (disposition pass 2)

- FND-005: reproduced with a minimal crate of the same shape, run with `cargo clippy --all-targets --target aarch64-apple-darwin -- -D warnings`: `error: constant RANGE_SET_CHILD is never used` and the same for `PRODUCT_TEST`. A full-workspace macOS clippy could not run here, because `psm` needs a C cross-compiler. Fix: put `#[cfg(target_os = "linux")]` on both constants, or move them inside the Linux block. The non-Linux runtime path itself is correct: it runs 16 leaves in-process with no `sh` or `ulimit`.
- Round-2 soundness: zero ranking only changes which gaps close. Closing any set of gaps adds values between existing intervals and leaves the first and last endpoints unchanged, so the result is still a superset with the same min and max. The exact-vs-widened differential at 53c8d06 (12000 targeted trees, `ulimit -v 3000000`) again showed only exact ok -> widened `non_zero_divisor`, 332 trees, and never the reverse. The count is the same as round 1: a later `+-1` operator re-covers zero, as the coder noted. The ranking helps the direct case only.

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 4b79f5f |
| FND-002 | fixed | 4b79f5f |
| FND-003 | fixed | 53c8d06 |
| FND-004 | fixed | 53c8d06 |
