---
id: SR-701
title: "gap analysis of PR 244 (IR-448 unbacked rows)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@e94752093bd5ed619945964b9a6a035e48af5597; spec/core/non-functional/NFR-001-determinism.md, spec/core/matrix/tests.md, spec/kani/matrix/tests.md, spec/model/matrix/tests.md, spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md, spec/kani/functional/FR-037-canonical-backend-replay-and-qualification.md, spec/kani/functional/FR-039-root-crate-public-interface.md, spec/model/functional/FR-019-rust-library-interface.md, spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md, tests/it/kani_shared.rs, tests/it/conformance.rs, tests/it/canonicalization.rs, tests/it/cycle_free_model.rs, src/lib.rs, crates/quire-contract-model/src/lib.rs"
review_set: subset
---
# SR-701: gap analysis of PR 244

## Summary

Ticket: IR-448. Reviewed head e947520 against origin/main 968ba9b. Quire 0.33.0 (engine
0.47.1).

I ran `make spec` myself at origin/main and at the head. Both runs exit 2. Both report
"Coverage: 163/184 rows backed" and "17 unbacked row(s)", and the 17 rows are the same:

- FR-036-AC-1..5, FR-037-AC-6, FR-039-AC-1..4 and FR-019-AC-5
- the matrix rows FR-036, FR-037 and FR-039
- the test cases TC-045, TC-055 and TC-058

Diffing the two outputs shows exactly two changes. The three NFR-001
`uncatalogued-verification-method` warnings are gone, and so are the three "`FR-154` matches no
declared row" warnings. Nothing else changed. No ID, AC or matrix row is removed.

I checked the coder's "already done" claims at origin/main, and they hold:

- FR-322 and FR-038 notes on non-test symbols use `Implements:`. There are seven sites, for
  example `v2/mod.rs:951` and `common.rs:726`.
- `.gitignore` lists `worktrees/`.
- No `MP-001` or `MP-002` remains under spec/ or plan/. #219 deleted them.
- FR-036, FR-037, FR-039, FR-019-AC-5, FR-030-AC-4/5, TC-045, TC-055, TC-058 and the NFR rows
  carry planned statuses with reasons.

Can an existing test honestly back any of the 17 rows? None can:

- **FR-039-AC-1 and FR-039-AC-3.** `src/lib.rs:12` is still `pub use quire_contract_model::*`,
  and `src/kani/` still holds `arithmetic.rs`, `collections.rs` and `objects.rs`.
- **FR-019-AC-5.** The model crate root has seven glob re-exports
  (`crates/quire-contract-model/src/lib.rs:31-47`).
  `tc_041_bridge_reexports_the_exact_model_api_and_keeps_model_sources_single` asserts the
  opposite.
- **FR-036.** Nothing implements symbolic range generation, per-item provider accounting or
  FR-290 capability negotiation. `CapabilityDisposition` is the TC-042 profile matrix, not
  FR-036's provider.
- **FR-037-AC-6.** It is true by absence: no `replay` or `witness` module exists. But no test
  asserts it.
- **FR-039-AC-2.** The manifest check in `tc_041_model_dependency_graph_is_cycle_free_and_owner_free`
  (`tests/it/cycle_free_model.rs:68-84`) implies the AC's first clause. It does not test the
  source-text clause, so planned is right.

There is a gap outside the 17 rows, though. Four acceptance criteria are minted
`backed: false`: FR-030-AC-4, FR-030-AC-5, NFR-001-AC-2 and NFR-002-AC-1. That accounts for
184 - 163 = 21 against the gate's 17. The strict gate does not report these four, because the
test cases they verify through (TC-223 and TC-019) count as backed. Both of those test-case
tags contradict the matrix's own status text.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-223 counts as backed because `tc_223_every_kani_outcome_kind_maps_to_its_one_fr331_result` carries `#[trace("TC-223")]`. The TC-223 matrix row says that test "verifies the retired `KaniProviderResult` map and does not back this case". So the strict gate does not report FR-030-AC-4 and FR-030-AC-5, although `quire coverage --json` mints both as `backed: false`. They are planned in prose but pass the gate on a tag the spec calls false. The fix removes no row: drop or repoint the stale tag, which brings the gate to 19 unbacked rows. Or track it in a follow-up ticket. This predates the PR but is inside IR-448's FR-030 scope | tests/it/kani_shared.rs:225-227; spec/kani/matrix/tests.md:24 |
| FND-002 | medium | `tc_018_the_corpus_runs_deterministically_and_every_fixture_matches` carries a `TC-019.` tag, so TC-019 is `backed: true`. TC-019's status says "has no executable test", and the NFR-001 metric this PR edits says "TC-019 is planned". Because of the tag, the gate does not report NFR-001-AC-2 or NFR-002-AC-1. tc_018 runs the complete corpus twice and asserts `first.stdout == second.stdout` (lines 178-187), which is exactly NFR-001-AC-1, yet it carries no NFR-001-AC-1 tag. The current NFR-001-AC-1 backer, tc_017 (`canonicalization.rs:83`), only canonicalizes one package twice in process, which is not "two complete corpus runs". Honest backing: tag tc_018 `NFR-001-AC-1` and drop its `TC-019` tag | tests/it/conformance.rs:135-180; tests/it/canonicalization.rs:83; spec/core/matrix/tests.md:42 |

## Verdict

The coder's claims hold. The 17 rows are the same before and after, each one is genuinely
unimplemented and planned with a reason, and the PR removes no row. No existing test honestly
backs any of the 17.

Both findings predate the PR. Each is a place where a row passes the gate on a tag that its own
matrix text disowns, which is the "back honestly" goal of IR-448. Fix them in this PR (tag
edits only, no row removed) or defer them to a ticket. Neither blocks this PR on its own.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | `tc_017_canonical_bytes_digests_ordering_and_resource_failure_conform` still carries an `NFR-001-AC-1.` tag. It canonicalizes a single package twice in process and never runs the corpus, so it does not witness "two complete corpus runs ... byte-identical". tc_018 now backs the AC honestly, so this tag only overstates the evidence. Drop it, or move it to the FR-016 rows the test does verify (it already carries FR-016-AC-1..4) | tests/it/canonicalization.rs:83 |

## Dispositions

Round 1, reviewed at 4941351784736182f05385b670f7860889f04cdf (fix commit 4941351).

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | still-open | The `#[trace("TC-223")]` attribute and `Tracing:` line are gone, but the function is still named `tc_223_every_kani_outcome_kind_maps_to_its_one_fr331_result`, and quire's `rust-test-name-id` form binds that name to TC-223. At 4941351, `quire coverage --json` still reports TC-223 `backed: true`, and FR-030-AC-4/5 (`backed: false`) are still absent from the strict list. A scratch probe that renamed only the function (no `tc_223_` prefix) made TC-223, FR-030-AC-4 and FR-030-AC-5 unbacked, 23 rows in all. Rename the function |
| FND-002 | fixed | 4941351 |
| FND-003 | still-open | New this round; no fix yet |

Round 2, reviewed at 632a7e4521cd536da952f7f2305dd82ff1e552ba (fix commit 632a7e4).

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 632a7e4 |
| FND-003 | fixed | 632a7e4 |
