---
id: SR-12125
title: "IR-841 code review (rust lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@b3dd66f55d8896849e66450e9a8917791f1bff60; crates/quire-contract-model/src/checked_package/common.rs (strict_shape_depth, Scan, charge, depth_tests::tc_048_a_deep_syntax_error_is_refused_in_linear_time)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

## Summary

Ticket: IR-841, PR quire-contract-ir#437, head b3dd66f, one commit over origin/main, one file changed. The rust-review lane is folded into this file.

The PR replaces the wall-clock assertion in `tc_048_a_deep_syntax_error_is_refused_in_linear_time` with a work counter. A `#[cfg(test)]` thread-local `SCAN_WORK` and a `charge(units)` helper are added; in non-test builds `charge` is an empty `#[inline(always)]` fn. Five `charge` calls go into production functions: the outer loop of `strict_shape_depth`, its closing loop, `Scan::peek` (per skipped whitespace byte), `Scan::pointer` (`open.len()`) and `Scan::token` (`byte_offset()`). The test asserts `shallow >= 131_000` and `3.5 <= deep/shallow <= 4.5` at depths 131,000 and 524,000. The `Instant` import is gone.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| common.rs:1240-1258 | examined | `#[cfg(test)] thread_local! { static SCAN_WORK ... }`, `#[cfg(test)] fn charge(units: usize)`, `#[cfg(not(test))] #[inline(always)] fn charge(_units: usize) {}` |
| common.rs:1297-1391 `strict_shape_depth` | examined | outer `loop { charge(1); ... }` and closing `loop { charge(1); ... }`; otherwise unchanged |
| common.rs:1400-1470 `impl Scan` | examined | `peek` charges per whitespace byte, `pointer` charges `open.len()`, `token` charges `tokens.byte_offset()` on success; `scalar`, `member_name`, `number_token` unchanged |
| common.rs:1964-1999 `tc_048_a_deep_syntax_error_is_refused_in_linear_time` | examined | resets `SCAN_WORK`, reads `[`*n `x` `]`*n, asserts `malformed`, compares counts at 131,000 and 524,000 |
| common.rs:280-351 `read_value`, `read_typed_prefix` | context_only | both call the same first pass, `strict_shape_depth` |
| v2/model_members.rs:835-853 `probe` | context_only | existing repo precedent: `#[cfg(test)] mod probe` thread-local counter, `#[cfg(test)] probe::walked();` at the call site |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Only one of the five charge sites is load-bearing. The input `[`*n `x` `]`*n fails inside `serde_json` at `x`, before `token`'s `charge(byte_offset())` runs. It has no whitespace, never reaches the closing loop, and never builds a pointer. Removing the closing-loop, whitespace, pointer or token charge each leaves the test green (mutants d2-d5). Removing the outer-loop charge kills it (d1). So the test counts exactly one unit per `[`, and four of the five hooks added to production code are not exercised by any test. | crates/quire-contract-model/src/checked_package/common.rs:1351, 1407, 1415, 1426, 1974-1999 |
| FND-002 | medium | The oracle changed from elapsed time, which grows with any superlinear work, to charged units, which grow only with work routed through `charge`. A real quadratic regression on this test's own path survives. A per-level rescan of the open stack (`scan.open.iter().filter(..).count()` at each `[`) passes at 4,000/16,000. It also passes at the test's full depths, where the test took 2,178 s against 0.12-0.55 s at head. The doc comment says "the count is exact" and that the pass "costs work linear in the input". That overclaims: the count is exact only for the charged steps. The PR body states the caveat; the code comment does not. The ticket asks to keep a mutant showing a quadratic parser is caught. The PR's mutant (`charge(1 + open.len())`) changes only the meter, not the parser. A real quadratic that goes through a charged path, building `scan.pointer()` at each `[`, is caught (ratio 16). | crates/quire-contract-model/src/checked_package/common.rs:1967-1975, 1305-1309 |
| FND-003 | low | The lower bound of 3.5 rejects some linear parsers: an affine cost `a*n + b` fails once `b > 0.2*a*131000`, about 26,200 units at slope 1. A fixed charge of 50,000 at the start of the pass kills the test (ratio 3.17), and 20,000 passes. At the other end, an `n log2 n` charge passes at ratio about 4.47, 0.03 under the upper bound. Neither bound is reachable from today's code, which measures exactly 131,001 and 524,001 (ratio 4.000). But the bounds are tied to a zero intercept, not just to linearity. | crates/quire-contract-model/src/checked_package/common.rs:1995-1998 |
| FND-004 | low | `charge` reuses the module's work-limit vocabulary. In this file "charged" and "work" already mean the reader's FR-038 work limit (lines 147, 901, 942, 1129, 1179). A reader can take `charge(1)` in the first pass to mean the pass is billed to that limit, which it is not. A name like `count_scan_work` would separate the two. The `units as u64` cast is lossless on every supported target. The crate's own helper for the same job is `u64::try_from(len).unwrap_or(u64::MAX)` (v2/model_members.rs:831). | crates/quire-contract-model/src/checked_package/common.rs:1249-1258 |

## Verdict

Mergeable with changes recommended. FND-001 and FND-002 are medium. They do not make the test wrong, but the test now proves less than its name and doc comment claim. FND-003 and FND-004 are low.

Suggested fixes, smallest first:

- State the limit in the doc comment: the count covers only charged steps.
- Either add inputs that reach the other four charge sites (whitespace, a closing run, a deep object with member names, a duplicate member, which builds a pointer), or drop the charges that no test asserts.
- Replace the charge-only mutant with a real one in the PR notes: per-level `scan.pointer()`, which the counter does catch.
- If the uncharged-work blind spot matters, keep the old wall-clock test as an `#[ignore]` test. `make test` already runs `--include-ignored`, so that would bring the flake back into the gate. A separate opt-in target would not.

Production code (verified):

- `git diff origin/main...HEAD` touches `strict_shape_depth` and `Scan` only by adding the five `charge(..)` lines. Every other line in both is byte-for-byte unchanged. There is no API change and no new dependency (Cargo.toml and Cargo.lock are unchanged).
- In a non-test build, `charge` is `fn charge(_units: usize) {}` with `#[inline(always)]`. `SCAN_WORK` does not exist outside `cfg(test)`. The arguments (`1`, `self.open.len()`, `tokens.byte_offset()`) are side-effect-free getters, so an optimised build drops them. A debug build keeps an empty call, which is negligible. `cargo build --locked --release -p quire-contract-model` succeeds, and so does `cargo clippy --locked --workspace --all-targets -- -D warnings`, with no warnings, so there is no unused-argument or dead-code lint in either cfg.
- Repo idiom: neither CLAUDE.md nor AGENTS.md forbids test-only hooks in production functions. There is precedent: `v2/model_members.rs` keeps a `#[cfg(test)] mod probe` thread-local counter and calls `#[cfg(test)] probe::walked();` inside the production `ancestors`. This PR uses a `cfg(not(test))` no-op twin instead of `#[cfg(test)]` at each call site. The effect is the same. The hook only counts and never changes behaviour under test, so the rust-review rule on `cfg(test)` branches that change behaviour does not apply. No existing limit could be used instead: the first pass is not billed to any `CheckedPackageReadLimits` member, so measuring through a limit would mean adding one.

Thread-local isolation (verified): libtest runs each test on its own thread. The test also resets `SCAN_WORK` to 0 right before each measurement and reads it right after, with only `read_value` in between on the same thread. Parallel tests cannot leak counts into it, and a leftover count cannot either.

Mutants, run in a throwaway worktree with its own target dir (since removed):

| Mutant | Depths | Result |
| --- | --- | --- |
| a1 real quadratic, uncharged: `scan.open.iter().filter(..).count()` at each `[` | 4,000/16,000 | survives (passes) |
| a1 same | 131,000/524,000 | survives (passes), but the test took 2,178 s, against 0.12-0.55 s at head |
| a2 real quadratic through a charged path: `let _ = scan.pointer();` at each `[` | 4,000/16,000 | killed: 8,002,001 vs 128,008,001 (ratio 16) |
| b author's charge-only stand-in: `charge(1 + open.len())` | full | killed: 8,580,696,501 vs 137,288,786,001 |
| c constant factor `charge(4)` | full | survives (expected) |
| d1 drop outer-loop charge | full | killed (`shallow` = 0) |
| d2 drop closing-loop charge | full | survives |
| d3 drop whitespace charge | full | survives |
| d4 drop pointer charge | full | survives |
| d5 drop token charge | full | survives |
| e1 linear slope 3 `charge(3)` | full | survives (expected) |
| e2 affine, fixed `charge(50_000)` at start | full | killed: 181,001 vs 574,001 (ratio 3.17) |
| e3 affine, fixed `charge(20_000)` | full | survives |
| f `n log2 n` charge | full | survives (ratio about 4.47) |

A note on a1. A first attempt used `scan.open.iter().count()`, which is O(1) because the slice iterator's `count` returns its length. It was replaced with a filtered count, which really walks the stack.

Flake re-run: 5 unloaded runs, plus 5 runs with four copies of the 187-test `checked_package` suite running at the same time. All 10 passed, as expected for a clock-free count. `checked_package::common` with `--test-threads=1` passes (10 tests), and so does the full `checked_package` suite in parallel (187 tests).

Rust lane: `cargo fmt --all --check` passes. There is no `unwrap` or `expect` in the touched production code. No `unsafe`. `saturating_add` on the counter cannot overflow-panic. The failure messages hard-code "131000 levels", which matches the constants.

## Dispositions

Round 1, against head c3738d742f2dd8a582c59ce9fe57151819a08929 (fix commit c3738d7; 7774351 merges origin/main and touches no reviewed code). I re-ran the mutants against the new test in a throwaway worktree with its own target dir (since removed).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The test now runs four documents (bottom syntax error, closers then error, whitespace, duplicate member) at 131,000/262,000/524,000 levels. Each has a per-level floor equal to today's exact cost: 1, 5, 8 and 5. Dropping any one of the five count sites now fails: outer loop (0 steps), closing loop (3 per level against a floor of 5), whitespace (5 against 8), pointer (4 against 5), token (4 against 5). |
| FND-002 | fixed | The comment above `SCAN_STEPS` and the test's doc comment now say that the count covers only `count_scan_step` sites and does not prove the pass linear. The test is renamed `tc_048_the_counted_steps_of_a_deep_document_grow_linearly`. The PR body states the uncounted-rescan blind spot and cites the real `scan.pointer()`-per-`[` mutant, which I re-ran at 4,000/8,000/16,000: killed, 8,002,001 / 32,004,001 / 128,008,001 steps. The uncounted rescan still survives (re-run at small depth: passes). That is the stated, accepted limit of a step count, not an overclaim any more. |
| FND-003 | fixed | Linearity is now checked by second difference: `abs(second/2 - first) * 20 <= first`, where first = s262 - s131 and second = s524 - s262. A fixed cost cancels: fixed `count_scan_step(50_000)` and `(5_000_000)` both stay green. A quadratic gives second/2 = 2 x first and fails. `n log2 n` fails too (about 5.3% curvature against a 5% tolerance), which is correct for a linearity check. Today's counts are exactly affine (for example 131,001 / 262,001 / 524,001), so the difference is 0 and the check is not brittle. The per-level range `least..=4*least` keeps a x4 constant-factor change green; x5 on the outer loop fails, as documented ("most is four times that"). |
| FND-004 | fixed | Renamed to `count_scan_step` / `SCAN_STEPS`. The comment says it is not the FR-038 work limit and bills nothing to it. The cast is now `u64::try_from(steps).unwrap_or(u64::MAX)`. |

Round-1 checks: in production code, `strict_shape_depth` and `Scan` differ from b3dd66f only by the `charge` to `count_scan_step` rename, and from origin/main only by the five count calls. `cargo fmt --check`, the release build and clippy `--workspace --all-targets -D warnings` all pass. Flake: 5 unloaded runs (4.0-5.5 s each in debug, up from about 0.5 s, because the test now reads 12 documents) and 5 runs alongside four parallel copies of the `checked_package` suite (10.5-12.1 s) all passed. So did `checked_package` in parallel (187 tests) and `checked_package::common` with `--test-threads=1` (10 tests). A first flake attempt reused a stale mutant test binary; it was discarded and re-run on a rebuilt head binary. `make spec`: strict 43 unbacked / 0 contradicted and 508/572 rows backed, identical to origin/main eb99d45 (the row total rose from 565 because of the merge from main, not this PR).
