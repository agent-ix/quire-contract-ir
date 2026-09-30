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
