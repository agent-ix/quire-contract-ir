---
id: SR-624
title: "code review of PR 186 (reader depth: nesting past the parser cap is incomplete)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@cfb1b713e5048e036b762bb297a795657758fb95; crates/quire-contract-model/src/checked_package/common.rs, crates/quire-contract-model/src/checked_package/shared.rs, tests/it/checked_package_v2_reader.rs"
review_set: subset
---
# SR-624: code review of PR 186

## Summary

Ticket: IR-279. PR agent-ix/quire-contract-ir#186, head cfb1b71, diffed against origin/main 3e7935f. The review covers code-review and its rust-review lane. It replaces `StrictSeed`, `json_depth`, `first_value_at_level` and `Children` with one generic `Strict<S: StrictSink>` visitor. That visitor serves two passes: a `Depth` shape pass on `stacker`/`serde_stacker` with serde's recursion cap lifted, and a `Value` pass that runs only when the document is within the limit. The effective depth limit is capped at `CheckedPackageReadLimits::MAXIMUM_DEPTH = 128`.

What was measured:

- **Dependencies.** No new dependency. `stacker =0.1.15`, `serde_stacker =0.1.11` and serde_json's `unbounded_depth` feature were already exact-pinned in `crates/quire-contract-model/Cargo.toml`, and binding.rs:204, conformance.rs:826 and identity.rs:1281 already use them. `make deny` passes: advisories, bans, licenses and sources are ok, and the one-copy awk passes.
- **Gates.** `make -k ci` on cfb1b71 and on a clean origin/main 3e7935f fails only at `make spec`. Both runs report the same 22 unbacked rows, the lists diff identical, and none of them touches FR-038. fmt, both clippy lanes, the full test suite, corpus, deny, cargo-audit and audit-unsafe pass on both. The PR body names a failing tc_041 pair, but it did not fail in either run.
- **Oracle strength.** 7 of 7 source mutants were killed by the new tests. M1, removing the 128 cap, makes the 100,000-deep test abort the process with a stack overflow, so the cap is load-bearing. M2 keeps serde's cap in the shape pass. M3 records the last pointer instead of the first. M4 sets scalar depth to 0. M5 charges depth before syntax and members. M6 keeps serde's cap in the value pass. M7 refuses the exact limit.
- **Deleted helpers.** Deleting `StrictSeed`, `first_value_at_level` and `Children` loses no behaviour. Duplicate-member detection, the `arbitrary_precision` number token, rejection of non-finite numbers and the first-past-limit pointer (FR-038-AC-26) all carry over. The pointer semantics are pinned by `tc_048_depth_is_charged_at_the_first_value_past_the_limit` at depths 0 to 3.
- **Refusal order.** Byte limit, then strict syntax and members, then depth, then canonical bytes. This matches FR-322 lines 345-350 on quire-specification origin/main 4634f5f.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Quadratic CPU on an over-deep malformed wire. A syntax error at depth d makes serde_json run `end_seq` and build a positioned error at every enclosing level, and each one scans the input from byte 0. A 1,048,001-byte `[`x524000 `x` `]`x524000, within the default 1 MiB limit, takes 6.1 s in a release build. main refuses the same input in 51 us, a regression of about 10^5 on hostile input | crates/quire-contract-model/src/checked_package/common.rs:886-898 |
| FND-002 | medium | The shape pass's grown stack amplifies memory about 190x: a 1 MiB, 524,000-deep document peaks at 201 MB RSS against a 3.3 MB baseline. The doc says memory is "bounded by the byte limit", but the bound is roughly 190x the byte limit, so a caller's 16 MiB byte limit, as in the PR's own test, allows about 3 GB | crates/quire-contract-model/src/checked_package/common.rs:879-892 |
| FND-003 | low | The doc comment links [`strict_json_shape`], which does not exist; the function is `strict_shape`. The PR body names test functions `strict_json_shape_*` that also do not exist | crates/quire-contract-model/src/checked_package/common.rs:220 |
| FND-004 | low | The four new unit tests in `mod depth_tests` carry no `Tracing:` tag. Every other test in the file carries `Tracing: TC-048, FR-038-AC-n`, and they sit in a second test module beside `mod tests` | crates/quire-contract-model/src/checked_package/common.rs:1274-1371 |
| FND-005 | low | Pre-existing, outside the diff. `model_members.rs` still parses selected model documents with serde_json's 128 cap. A strict model document nested deeper than 128 whose digest the lock does select is refused as `stale_dependency`/byte-digest-mismatch, the same misclassification class as IR-279 | crates/quire-contract-model/src/checked_package/v2/model_members.rs:874 |

## Finding Detail

- FND-001: Root cause in serde_json 1.0.151 `de.rs`. `deserialize_any` for `[` evaluates `match (ret, self.end_seq())`, so `end_seq` runs even when `ret` is an error. At the offending byte it builds `peek_error`, and `SliceRead::position_of_index` runs `memrchr` over `slice[..i]`, which is O(n) on canonical single-line JSON. The cost is O(depth x bytes). Release timings for the malformed case: 100k deep 0.55 s, 200k 2.0 s, 400k 13.5 s (under load), 524k 6.1 s (idle). A deep duplicate member or any other deep visitor error takes the same path. The same shape is instant when valid (176 ms). The suite pays for it too: `strict_shape_measures_deep_nesting_and_finds_later_defects` takes 40 s in a debug build. Fix: make the first pass iterative, as a non-recursive syntax, duplicate and depth scanner with an explicit stack, so no error bubbles through d frames. That also removes FND-002. An alternative is a structural nesting ceiling checked before the recursive parse, but FR-322's syntax-before-depth order must still hold.
- FND-002: Measured by running the release test binary directly under `/usr/bin/time -f %M`: 201,028 KB peak for both the valid and the malformed 524k document, against 3,324 KB for a 10-deep document. The peak is stacker segment memory per nesting level. The fix is the same iterative first pass. Otherwise, state the real bound in the doc and in FR-038.
- FND-004: Add `/// Tracing: TC-048, FR-038-AC-3` (or AC-26) to each test, and fold them into `mod tests`.
- FND-005: Not introduced by this PR. File a follow-up ticket rather than fix it here.

## Verdict

Not mergeable at cfb1b71. The classification fix is correct and well tested: 7 of 7 mutants were killed, the refusal order matches FR-322, and the deleted helpers lose nothing. But lifting serde's cap turned an O(1) refusal of hostile deep input into O(n^2) CPU and about 190x memory, and FND-001 must be fixed before merge. The design decision on the 128 cap is covered in SR-625 and SR-626.

## New findings (disposition pass 1)

Reviewed at f9251e58be5d6e2980996e65aba5e1c7578b1c84 (fix round 7f1d923 and the Drop fix f9251e5, based on main 7c70041). The PR merges cleanly onto current origin/main d98c7cc. The merged tree passes every `make ci` target except `make spec`, which reports 17 unbacked rows, the same list as origin/main. Oracle strength: 12 of 13 mutants were killed, and the survivor is FND-008.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | high | Only drop was made safe for deep values. Every other public operation on an admitted deep package recurses on the caller's stack and aborts the process: `clone()`, `==`, `{:?}`, `graph().clone()` (whose drop has no `Drop` impl), `serde_json::to_vec(graph())` and `lower()`. `CheckedPackageV2ReadResult::Admitted` is documented "safe for lowering". Reproduced with the PR's own 10,000-deep aggregate fixture on a 2 MiB thread in debug: all six operations abort, and drop passes. In release on an 8 MiB stack, `lower` and `clone` pass at 10,000 levels and abort at 50,000 | crates/quire-contract-model/src/checked_package/v2/mod.rs:374 |
| FND-007 | medium | The read reserves 256 KiB plus 4 KiB per measured level in a single up-front `stacker` mmap, sized from untrusted depth. A 4 MB, 2,000,000-deep document under `depth: u64::MAX` reserves 8 GiB while using 1.26 GB. Under `ulimit -v 6G` the reader panics ("allocating stack failed", stacker lib.rs:168) instead of returning a ReadResult. On 32-bit the size saturates to usize::MAX | crates/quire-contract-model/src/checked_package/common.rs:251-256 |
| FND-008 | medium | The `Drop` impl's diagnostic-details branch is untested. With `entry.details.iter_mut().for_each(dismantle)` disabled, every test stays green (mutant M8c). The node-body and identity-projection branches are killed by `tc_048_a_deep_admitted_package_drops_on_a_small_stack`, because its fixture splices the deep body into both | crates/quire-contract-model/src/checked_package/v2/mod.rs:394 |
| FND-009 | low | `tc_048_a_deep_syntax_error_is_refused_in_linear_time` asserts wall-clock time (`elapsed < 2 s`) in the default test lane. It measures 0.45 s in an idle debug build, 4.4x headroom, which can fail spuriously under parallel load. A structural oracle, or an `#[ignore]`d bench lane, would be sturdier | crates/quire-contract-model/src/checked_package/common.rs:1628-1647 |

### New finding detail (disposition pass 1)

- FND-006: `CheckedPackageV2` and `CheckedPackageV2ReadResult` derive `Clone, Debug, Eq, PartialEq`, and the pub accessors hand out `&CheckedSemanticGraphV2`, which is `Clone`/`Serialize`, holding `body: Value`. `lower` clones bodies into `CompleteLoweringResultV2`, which has no iterative drop. The Drop fix is sound as far as it goes: nothing moves fields out of the package, since that would not compile, and `dismantle` cannot panic. But honouring a large caller limit now admits packages that most of the public API cannot handle. Options: run `lower` (and document or wrap clone/eq/debug/serialize) under a stack sized from a depth recorded at admission; or document that a caller who raises `depth` must itself run consumers of the package on a stack that large; or test the limit the crate actually supports. A test should cover at least `lower` on a deep admitted package.
- FND-007: Measured with the release lib test binary: `read_value` of `[`x2,000,000 `]`x2,000,000 under `bytes: u64::MAX, depth: u64::MAX` takes 3.2 s at 1.26 GB RSS. Under `ulimit -v 6291456` it panics. The 524,000-deep 1 MiB document peaks at 333 MB. Consider segment-wise growth instead of one reservation sized to the whole depth, or bound the reservation and turn allocation failure into an outcome rather than a panic.
- FND-008: Add a deep diagnostic `details` term to the drop test, or state why details cannot be deep.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 7f1d923 |
| FND-002 | fixed | 7f1d923 |
| FND-003 | fixed | 7f1d923 |
| FND-004 | fixed | 7f1d923 |
| FND-005 | deferred | IR-452 (pre-existing model_members.rs:874 serde cap, outside this diff) |

- FND-001: `strict_shape` is now an iterative scanner with an explicit stack. The 524,000-deep syntax-error document is refused in 23 ms in release (was 6.1 s) and in about 0.45 s in debug. Mutants M5 and M10/M11 are killed; M11 is killed by the 44-text parity test.
- FND-002: The shape pass peaks at 33 MB for the 524k malformed document (was 201 MB), and the false "bounded by the byte limit" claim is gone. The remaining memory for a valid deep read under a raised limit is caller-selected and is tracked as FND-007.
- FND-003: The doc link now reads [`strict_shape`].
- FND-004: Every new unit test carries `Tracing: TC-048, FR-038-AC-3`.

## New findings (disposition pass 2)

Reviewed at ae300d3b00e62b69fbf263718570c45138a12d11, rebased on origin/main d98c7cc. `make -k ci` fails only at `make spec`: 17 unbacked rows, the same list as main. deny, audit, both clippy lanes and the full test suite are green. Oracle strength: 18 mutants, 17 killed. The killed mutants include Debug, Clone, `==` and `lower` each run with a stack sized for depth 0, each of the three `Drop` impls disabled, the ceiling removed, and the per-level stack set to zero. The survivor is FND-010.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-010 | low | No test pins the value of `MAXIMUM_DEPTH`. Every test refers to the constant symbolically, so changing it to 16,383 leaves every test green. FR-038 states the number 16,384 literally | crates/quire-contract-model/src/checked_package/shared.rs:43 |

## Dispositions (disposition pass 2)

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-006 | fixed | ae300d3 |
| FND-007 | fixed | ae300d3 |
| FND-008 | fixed | ae300d3 |
| FND-009 | fixed | ae300d3 |

- FND-006: `CheckedPackageV2` now implements `Clone`, `PartialEq` and `Debug` by hand, and `lower` runs on `on_stack_for(self.depth)`. Node bodies, projections and diagnostic details drop iteratively. I verified at the ceiling, with an 8,189-term aggregate that sits exactly at 16,384 levels, on a 256 KiB thread in debug and in release: read, clone, `==`, `Debug` (of the package and of the ReadResult) and `lower` all pass. `graph().clone()` still aborts. That is documented as caller-owned in shared.rs:17-25, on the type, on `lower` and in FR-038.
- FND-007: The stack reservation is bounded at 256 KiB + 4 KiB x 16,384, about 64 MiB. A 2,000,000-deep document under `depth: u64::MAX` returns `incomplete(Depth, 16384, 2000000)` in 77 ms with 118 MB RSS (the scan) and reserves no stack. A 16,384-deep read takes 9 ms at 13 MB RSS and passes under `ulimit -v` of 1 GiB and of 256 MiB. Residual: under caps of 128 MiB and below, the bounded reservation still fails. At a 64 MiB cap stacker panics ("allocating stack failed"); at 96 and 128 MiB caps an ordinary allocation aborts the process. Both happen only under caps near the size of the reservation itself.
- FND-008: The drop test now covers a deep diagnostic detail, and the mutant that disables the diagnostic `Drop` is killed.
- FND-009: The wall-clock bound is replaced by a ratio: 4x depth must take less than 8x the time, fastest of three runs.

## New findings (disposition pass 3)

Reviewed at a29a37fe8e978206677a9be2314f25baa7c98944, up to date with origin/main d98c7cc.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-011 | medium | Two tests traced to FR-038-AC-3 assert the recorded deviation, so the trace over-claims AC-3 coverage. Each asserts that a caller limit of `u64::MAX` returns `incomplete(Depth, MAXIMUM_DEPTH, ...)`: `tc_048_deep_nesting_is_decided_by_the_callers_limit_to_the_ceiling` (`Tracing: TC-048, FR-038-AC-3`) and `tc_048_nesting_is_charged_against_the_callers_limit_after_syntax_and_members_pass` (`#[trace("TC-048", "FR-038-AC-3")]`). That is behaviour AC-3 as written contradicts. An implementation that honoured AC-3 literally would turn these AC-3 evidence tests red, and `make spec` counts AC-3 as backed by them. The pinned-ceiling test is correctly traced to TC-048 only. Moving the ceiling assertions into that test, or another TC-048-only test, would leave the AC-3 tags covering only AC-3 behaviour | crates/quire-contract-model/src/checked_package/common.rs:1619-1650 |

## Dispositions (disposition pass 3)

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-010 | fixed | 3f2ab85 |

- FND-010: `tc_048_the_reader_ceiling_is_sixteen_thousand_three_hundred_eighty_four_levels` asserts `MAXIMUM_DEPTH == 16_384`, admits 16,384 levels under `u64::MAX` and reports `incomplete(Depth,16384,16385)` one level deeper. I ran the mutants: setting the constant to 16,383 or to 16,385 turns the test red.

## Dispositions (disposition pass 4)

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-011 | fixed | 6c2deac |

- FND-011: Reviewed at 6c2deac98e2e74fbdcbdd4ac6a62415c0775077a, up to date with origin/main d98c7cc. The ceiling assertions moved word for word into two TC-048-only tests: `tc_048_nesting_past_the_ceiling_is_charged_at_the_ceiling` (lib) and `tc_048_nesting_past_the_reader_ceiling_is_charged_at_the_ceiling` (`#[trace("TC-048")]`). No test traced to FR-038-AC-3 now asserts the clamp. Their `u64::MAX` cases are documents within the ceiling, which are admitted or refused on their content. Mutants: reporting the caller's limit past the ceiling turns both new TC-048 tests red, and so do removing the ceiling, setting it to 16,383, refusing the exact limit, an off-by-one depth and charging depth before syntax. So no coverage dropped. Gate: fmt-check, lint, test (192 + 58 + 2 tests), corpus, deny, audit and audit-unsafe pass, and `make spec` reports 17 unbacked rows, identical to main (FR-038-AC-3 is not among them).
