---
id: SR-1080
title: "code review of PR 259 after rebase (IR-535 selections bind by identity)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@22ed70872d8e5cc42fcd68c882428399facec766; crates/quire-contract-model/src/checked_package/shared.rs, crates/quire-contract-model/src/checked_package/v2/identity.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs, crates/quire-contract-model/src/checked_package/v2/model_members.rs, crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs, tests/it/checked_package_v2_canonical_encoding.rs, tests/it/checked_package_v2_dependency_reference.rs, tests/it/checked_package_v2_dependency_selections.rs, tests/it/checked_package_v2_frame_entries.rs, tests/it/checked_package_v2_model_members.rs, tests/it/checked_package_v2_reader.rs, tests/it/checked_package_v2_temporal.rs, tests/it/support/checked_package.rs"
review_set: subset
---
# SR-1080: code review of PR 259 after rebase

## Summary

Ticket: IR-535. Reviewed head 22ed70872d8e5cc42fcd68c882428399facec766. It is one commit
whose parent is origin/main cbcd790a8fc9d07c83c7c7f6238023ed85fa0bad, so the two-dot and
three-dot diffs are the same. There are 14 files, +510/-309. The rust-review lane is folded
into this file. Every measurement was taken in a detached worktree outside the repo, with
its own target dir. The PR body and the author's gate log were treated as claims and
re-measured. This review replaces SR-820, which reviewed the pre-rebase head 558f45b.

Wire. `CheckedDomainPackageRef` is now `{identity, digest_domain, digest}` and
`CheckedDependencySelection` is `{identity, package_id}`. Both keep `deny_unknown_fields`.
Both encode through the `FixedShape` derive, so the canonical preimage bytes follow the
struct and no hand encoder needs changing. `DEPENDENCY_SELECTION_MEMBERS` is
`["identity", "package_id"]`. The `version` emptiness checks in
`validate_dependency_selections` and `validate_domain_packages` are gone.
`semantic_ir_identity` returns `package.identity` only. `admit_document` compares the
identity alone, and on a mismatch it still calls `quire_canonical::drop_value` (#266) before
returning `wrong-model-selection` at `identity`. `DomainModel.version` is removed. No other
selection family changed. `CheckedSelection {role, definition}`, with
`CheckedArtifactRef {authority, identity}` (#253), carries no version on main, and the
`unknown_profile` join over `lock.profile_selections` and `lock.edition` is outside the diff.
The only `version` fields left in `v2/mod.rs` are `schema_version`, the preimage's
`quire.checked-package-id/v2` tag and `graph_version`.

The class-2 sweep is deleted. On main, class 2 refused a same-identity, different-version
pair. Besides that, its only job was to guarantee one row per identity for the `Model`
owner join in `identity::validate_owner`. That guarantee still holds. Take two rows of one
identity. If every member is equal, the class-1 whole-item repeat refuses them. If their
`digest_domain` differs, one of them is not `sha256-jcs`, so class 2 (domain) refuses them.
If only the digest differs, the identity-keyed pair sweep in `validate_domain_packages`
refuses them as `stale_dependency` at the later row's `/digest`. That sweep runs after the
domain and shape sweeps and before the first `admit_selection`. So no function of the old
class 2 is lost, and the merged FR-038 text ("Selections bind by identity", four classes)
states this order. The `BTreeMap` key changed from `(identity, version)` to `identity`, and
the `Entry` import was removed.

`package_id`. Every package with non-empty selections now has a different preimage, so a
different `package_id`. No recorded constant covers such a package. The recorded
`identity_digests.rs` fixtures (all-families, nominal, operations) have empty selection
arrays, so they are unchanged and still pass. I recomputed one value independently. I took
QSpec's `dependency-selection-vectors.json` at 974fb24, which is positive-all-families plus
two `{identity, package_id}` entries. Python RFC 8785 plus SHA-256 over that preimage gives
0606043af7f4996c38c54d7207600badadaee17bf90955263bbfd652b209abf2, the value QSpec
records. With a `version` member on each entry it gives 1ed1ecc3…, which differs. A
throwaway probe fed this reader the package under each digest. Under QSpec's digest it
passed the `package_id` check and refused at the next step, `missing_import` at
`/lock/dependency_selections/0`, because the probe supplied no dependency packages. Under
the with-version digest it refused `stale_dependency` at `/package_id/digest`. So the
reader derives QSpec's version-free `package_id`. The in-repo tests' `refresh_identity`
hashes the JSON preimage with serde_json and does not use the reader's types, so it is an
independent oracle within the repo.

Process. The diff has no conflict markers, artifacts or unrelated churn. `operations.rs`
("taken from main") has no diff against main. The conflict-resolved hunks (`identity.rs`
`validate_owner` comment, `mod.rs` `CheckedDependencySelection`/`validate_lock`, five
`model_members.rs` hunks, its tests, `frame_entries.rs`, `model_members` it test) are
limited to removing `version`. The fixtures that are new on main (`temporal.rs`,
`canonical_encoding.rs`, `reader.rs`) build their rows without `version`. No test fixture in
`tests/` or in the src test modules builds a selection row with `version`. The remaining
`"version"` strings are Semantic IR `package.version` in documents, which is deliberate,
and the AC-63 old-shape probe.

Gates I ran at this head:

- `make ci`: `fmt-check`, `lint` and the workspace build passed. One model unit test failed
  once: `common::depth_tests::tc_048_a_deep_syntax_error_is_refused_in_linear_time`, a
  timing ratio (14 ms vs 120 ms) at load average 15. `common.rs` is untouched by the diff,
  and the test passed on three isolated reruns.
- Re-run test target: it 285, model lib 121, doc 7, all passing.
- `make corpus` exits 0 (99 match lines).
- `make spec` stops on the strict baseline: 23 unbacked, 0 contradicted, the same as main.
  Coverage is 220/261 against main's 217/261, and FR-038 is 98/108 against 95/108.
- `make deny` exits 0 (including one-copy). `cargo audit` exits 0, and `make audit-unsafe`
  exits 0.
- `make conformance-qspec` with `QUIRE_SPECIFICATION_DIR` at a detached QSpec 974fb24: 1
  passed (all three positive fixtures admit).

The author's log matches. Its last line is a hand-written summary. These results are mine.

Mutation probes. Each was run in the throwaway tree and reverted. All seven were killed:

- M1, `deny_unknown_fields` removed from the model row (version accepted): AC-62 test and
  owner-join test.
- M2, the same removed from the dependency entry: AC-63 test and AC-32 test.
- M3, `package.version` required again in `semantic_ir_identity` (bind by version): the
  "needs no package version" test.
- M4, identity-pair sweep disabled: AC-20 test and AC-10 duplicate test.
- M5, old-shape reclassification off: AC-63 test and the shape test.
- M6, pair sweep keyed by digest instead of identity: AC-20 and AC-10 tests.
- M7, document identity not compared: the unit admission test, the AC-27/64 test and the
  owner-join test.

Status of SR-820:

- FND-001 (rebase onto #250): resolved. #250's tests no longer set a row `version`.
- FND-002 (`WrongModelSelection` doc): fixed in `shared.rs`.
- FND-003 (long comment lines): partly fixed. One line remains (FND-002 below).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The edited `validate_owner` comment still says the model owner "carries no version, which stays selection evidence in the lock". After this PR the lock row carries no version either, so the clause states the reverse of the code. The PR rewrote the next sentences of the same comment and left this one | crates/quire-contract-model/src/checked_package/v2/identity.rs:546-547 |
| FND-002 | low | The `admit_document` doc paragraph is left ragged. One added line is 110 columns ("Parsing is charged to the reader's `work` limit before it starts, one unit per [`DOCUMENT_BYTES_PER_WORK`]"), past the 100-column style. rustfmt does not wrap comments. This is carried over from SR-820 FND-003 | crates/quire-contract-model/src/checked_package/v2/model_members.rs:1019-1020 |
| FND-003 | low | In `tc_048_a_selected_document_needs_no_package_version`, the assertion that the five graphs are equal is tautological. `package_over` builds `semantic_graph` from `IDENTITY` and `ORDER` alone, and the document enters only the lock digest, so the graphs are equal by construction under any reader. The per-version admission is the real oracle, and M3 shows it is load-bearing. The doc comment's "yields the same model-owned node keys (the graph is equal)" over-claims what the assertion checks | tests/it/checked_package_v2_model_members.rs:353-382, tests/it/checked_package_v2_model_members.rs:150-175 |

## Verdict

The change is correct and complete against QSpec's merged wire. QSpec's `schema.json`
`ModelRef` is `{identity, digest_domain, digest}` and its `DependencySelection` is
`{identity, package_id}`. Both have `additionalProperties: false` and no `version`. The
change is also correct against FR-038 "Selections bind by identity". The deletion of the
class-2 sweep loses nothing: the single-selection-per-identity invariant is kept by
classes 1, 2 and 4. The reader derives QSpec's recorded version-free `package_id`. Every
mutant was killed. All three findings are low, comment and test-intent nits, and none
blocks the merge. The PR stays HELD only for the QSL lockstep.

## Dispositions

Round 1, reviewed at ae9e3f16b7af1e5f06cfb8e04c543a19735a7518 (one commit on main
cbcd790, force-pushed over 22ed708). The tree delta 22ed708..ae9e3f1 touches three files
only: identity.rs, model_members.rs and tests/it/checked_package_v2_model_members.rs.

Checks run at the new head:

- `cargo fmt --check` and `clippy --workspace --all-targets -D warnings` pass.
- The model_members it tests pass (15), and the model lib checked_package tests pass (114).
- `quire coverage --strict` is unchanged: 220/261, FR-038 98/108, 23 unbacked, 0
  contradicted.
- M3 (`package.version` required again) is re-run against the new test and is still
  killed: the "no version" case refuses `invalid_model_binding`, where the test expects
  admission.
- The longest added line is 87 columns.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ae9e3f16b7af1e5f06cfb8e04c543a19735a7518 |
| FND-002 | fixed | ae9e3f16b7af1e5f06cfb8e04c543a19735a7518 |
| FND-003 | fixed | ae9e3f16b7af1e5f06cfb8e04c543a19735a7518 |
