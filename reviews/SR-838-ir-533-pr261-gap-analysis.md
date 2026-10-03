---
id: SR-838
title: "gap analysis of PR 261 (IR-533 encode the v2 wire types through quire-canonical)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@82e60e791442a63426f7a739a8cb9256aa0aa8f8; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/model/functional/FR-019-rust-library-interface.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, crates/quire-contract-model/src/checked_package/v2/encode.rs, crates/quire-contract-model/src/checked_package/common.rs, tests/it/checked_package_v2_canonical_encoding.rs"
review_set: subset
---
# SR-838: gap analysis of PR 261

## Summary

Ticket: IR-533. Plan completion: not assessed. This review checks the merged spec (#258,
squash a41c0e5f7360d37b8a3ce740108d8077329b36c8) against the code and tests at
82e60e791442a63426f7a739a8cb9256aa0aa8f8. It covers FR-038 "Canonical encoding of the
wire types" with its per-type table, "Canonical bytes are quire-canonical's bytes", the
FR-019 paragraph, FR-038-AC-74 through FR-038-AC-80, and TC-048 "Canonical encoding".

The tests map to the ACs as follows. Every test is `#[trace("TC-048", <AC>)]`, and each
binding was read and is correct.

| AC | Test(s) | Result |
| --- | --- | --- |
| AC-74 | `tc_048_preimage_bytes_equal_serde_json_and_hash_to_the_package_id`; `tc_048_a_preimage_that_differs_from_the_one_the_package_id_covers_is_stale` | byte identity for 3 fixtures plus crafted bodies with astral strings and ±2^53; `sha256` equals the recorded id and differs from `sha256_with_domain`; one changed member gives `stale_dependency` at `/package_id/digest` |
| AC-75 | `tc_048_graph_bytes_equal_serde_json_for_every_fixture_and_crafted_graph` | fixtures plus a crafted graph with all four nominal versions, `declaration`, `recursion_group`, every scalar kind, and 0, -1, ±2^53; no `null` for absent members |
| AC-76 | `tc_048_lock_source_map_capability_semantic_id_and_diagnostics_bytes_equal_serde_json` | one assertion per type per fixture; diagnostics with nested `details` and non-empty `loci` |
| AC-77 / AC-78 | three `tc_048_..._100000_deep_..._256_kib_stack` tests | expected text by repetition; exact ceiling returns bytes; one byte lower gives `Limit(CanonicalBytes)` with bound = length-1; 20,000 deep encodes; ±(2^53+1) gives `IntegerMagnitudeAboveMaximum(value)`; 2^53 written |
| AC-79 | `tc_048_an_integer_past_2_pow_53_..._noncanonical`; `tc_048_the_canonical_bytes_of_those_values_are_not_refused_as_noncanonical`; `tc_048_members_in_utf16_code_unit_order_pass_the_canonical_bytes_check` | each document also carries a stale id and a non-term body, and `noncanonical_wire` comes first with no pointer; the canonical counterparts reach `stale_dependency`, and as literals with a fresh id they admit; UTF-16 order reaches the body grammar refusal |
| AC-80 | `tc_048_quire_canonical_is_a_branch_main_git_dependency_...`; `tc_048_fixed_depth_types_derive_...`; `tc_048_the_wire_types_are_on_the_path_the_compiler_sees`; `tc_048_no_copy_of_quire_canonical_...` | manifests, `deny.toml`, the lock count, the derive list, no `impl FixedShape` and no `const DEPTH`, no wrapper, compile-time positive and negative `FixedShape` checks; `make deny` passes (run by this review) |

FR-019. The seven wire types implement `quire_canonical::Encode`, the fixed-depth ones
through the crate's blanket impl over `FixedShape`. That holds, because `quire_bytes::<T:
Encode>` compiles for all seven in the AC-74 to AC-76 tests.

Matrix. The FR-038 row moves AC-74 through AC-80 from planned to the implemented list and
adds one evidence sentence. The TC-048 row does the same, and only these ACs change.
Row-level 🚧 stays for the IR-503, IR-505, IR-530 and IR-535 work, and no backed-row count
is re-added. The coder's gate shows `spec` at main's baseline: 23 unbacked rows and 0
contradicted.

Open PRs. Each of #250, #253 and #259 rewrites the FR-038 and TC-048 rows, so all three
conflict on `spec/checked_package/matrix/tests.md` against this head (`git merge-tree`).
#253 (held) and #259 also conflict in `v2/mod.rs`, and #253 in `shared.rs`, because the
derive lines sit on the types they change. Whichever lands second must rebase. When #253
lands, it must derive `FixedShape` on its new reference types and update this PR's
`FIXED_DEPTH` list. FR-038 already says the types are named by role.

"Closes IR-533". IR-533's widened scope is covered:

- the preimage, graph, lock, source map, capability, diagnostics and semantic id;
- byte identity per type;
- the `deny.toml` entry;
- no wrapper and no `DEPTH`.

Its coordination asks (tell the IR planner whether quire-canonical #6 resolves IR-274's
depth blocker, and send the merge SHA to QSL) belong to the lead, not to code acceptance.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038's first bullet ("An integer whose magnitude exceeds 2^53 anywhere in the document ... refuses `noncanonical_wire`") has no test past the `i64`/`u64` range. The implementation does not satisfy it there: `100000000000000000000` passes the canonical-bytes check and then refuses with a grammar or identity code (SR-837 FND-001, reproduced). FR-038-AC-79 names only ±9007199254740993, so the AC passes while the FR statement is contradicted. Add the case to AC-79's test (and to the AC text if the spec is amended to define "integer" by token) | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:400, tests/it/checked_package_v2_canonical_encoding.rs:632 |

## Verdict

FR-038-AC-74 through FR-038-AC-80 each have passing tests with correct trace tags. Those
tests fail under the matching mutation, traced by reading the oracles. Source mutation
probes were not re-run, because the review worktree refused edits. TC-048's "Canonical
encoding" procedure is followed and FR-019's paragraph holds. One medium gap remains: the
FR-038 statement about integers past 2^53 is untested, and contradicted, for tokens
outside the `u64` range.

## Dispositions

Round 1 at 0c2508e107b480d51e8ddb718c89d29048253823, rebased onto main
1042762d0d77e1774d6e3d1c5af1155917f8c8a1.

FND-001. The FR-038 bullet "An integer whose magnitude exceeds 2^53 anywhere in the
document ... refuses `noncanonical_wire`" is now backed past the `i64`/`u64` range. The
backing test is `tc_048_an_integer_spelled_past_the_64_bit_range_is_noncanonical`,
`#[trace("TC-048", "FR-038-AC-79")]`, a correct binding. It covers four integer spellings
past 64 bits, each refused `noncanonical_wire` ahead of a stale id and a non-term body.
The non-whole float `1.5` still reaches the stale refusal. That is correct: FR-038 names
only whole floats, and `1.5` is canonical RFC 8785 text that the literal grammar refuses.
The AC-79 text is unchanged. Its named values remain a subset of the FR statement, which
the added test now covers, so no spec amendment is needed. The matrix rows are as in the
review pass after the rebase. No new findings.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 0c2508e107b480d51e8ddb718c89d29048253823 |
