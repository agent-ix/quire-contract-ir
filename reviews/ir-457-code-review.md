---
id: SR-634
title: "code review of PR 235 (remove digest staleness checks from the checked-package reader)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@b50cd28fd9debf730c4e3f72d9fb2abe6c17e7ce; crates/quire-contract-model/src/checked_package/{common.rs,evidence.rs,shared.rs,v2/mod.rs,v2/dependency_references.rs,v2/model_members.rs}, tests/it/{checked_package_v2_reader.rs,checked_package_v2_dependency_reference.rs,checked_package_v2_dependency_selections.rs,support/checked_package.rs}"
review_set: base
---
# SR-634: code review of PR 235

## Summary

Ticket: IR-457 (also closes IR-72's concern). Code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD` (origin/main 8371caa, head b50cd28; merge base is current main). The PR deletes `CheckedPackageEvidence::insert_artifact_bytes` / `insert_artifact_digest`, the `artifacts` map, the `ArtifactDigests` trait and the lock-vs-evidence digest comparison in `validate_locked_artifact`, threads the removed `context`/`evidence` parameter out of `validate_locked_artifact`, `validate_source_map_entries` and `validate_unexported`, drops the `version` parameter from `insert_dependency_package` (removing `SuppliedDependencyPackage` and the supplied-version comparison at `dependency_references.rs:111`), and adds a doc paragraph to `admit_document`.

Owner ruling (relayed by the IR planner, not re-litigated here): digest staleness checks are removed. This review checks the removal was done correctly and that no kept behaviour was lost.

Gates run by the reviewer at b50cd28 with CARGO_TARGET_DIR inside the review worktree: `make fmt-check lint test corpus deny` exit 0 (test results 170 + 58 + 2 passed; conformance corpus 99 rows `match`, 0 mismatch; advisories, bans, licenses, sources ok; one-copy check ok).

Kept-check mutation run (each check disabled with `if false && ...`, then `cargo test -p quire-contract-model --lib checked_package` and `cargo test -p quire-contract-ir --test it checked_package`):

| Mutation | Result |
| --- | --- |
| M1 `v2/mod.rs:761` recomputed package_id vs preimage | red: 3 tests (dependency_selections ascending, complete_v1 tc_044, reader injected faults) |
| M2 `v2/mod.rs:861` empty `lock.sources` | red: `tc_048_v2_reader_refuses_every_structural_mutation` |
| M3 `v2/mod.rs:867` lock vs `identity_preimage` mirror | red: 3 tests |
| M4 `v2/mod.rs:1110` same identity+version, different digest | red: `tc_048_duplicate_model_selection_refuses_as_malformed_wire` |
| M5 `v2/mod.rs:1834` `identity_projection` vs graph | red: 2 tests |
| M6 `model_members.rs:864` digest domain in `admit_document` | **green: no test fails** (FND-002) |
| M7 `model_members.rs:889` document digest vs lock digest | red: `tc_048_a_selection_admits_only_the_document_it_names` (unit) |
| M8 `dependency_references.rs:111` package_id binding | red: `tc_048_the_selected_dependency_is_supplied_and_binds_to_its_entry` |

(M1 also tripped `tc_048_a_deep_syntax_error_is_refused_in_linear_time`, a pre-existing wall-clock test that passed in the gate run; load noise, not caused by M1, and outside this diff.)

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `CheckedPackageRefusalCause::RevisionMismatch` is now never constructed: its only producer (`dependency_references.rs:111`) was deleted, but the public variant and its doc ("a supplied dependency package's version differs from its `dependency_selections` entry's") remain, describing behaviour that no longer exists. No consumer outside IR names it (grep of /home/peter/dev excluding worktrees), so it can be deleted with the check. The wire enum `CheckedDiagnosticCause::RevisionMismatch` (v2/mod.rs:307) is a different type and stays. | crates/quire-contract-model/src/checked_package/shared.rs:195-197 |
| FND-002 | low | The kept `admit_document` digest-domain check has no test: disabling it turns nothing red. It is also unreachable through `CheckedPackageV2::read`, because `validate_domain_packages` (v2/mod.rs:1083-1090, class 3) refuses any non-`sha256-jcs` selection before `admit_selection` runs. Pre-existing, but the per-site table claims it KEPT as a real check; either delete it as dead or give it a unit test. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:864-870 |
| FND-003 | low | `CheckedPackageRefusalCode::StaleDependency` doc still reads "A locked source, definition, or model did not match supplied bytes"; locked sources and definitions are no longer compared with anything. The code now covers package_id/preimage/projection/lock-mirror identity failures, a self-contradictory model lock, a domain-package document digest mismatch and a dependency package_id mismatch. | crates/quire-contract-model/src/checked_package/shared.rs:91-92 |
| FND-004 | low | `CheckedArtifactLocator` doc still reads "Exact source material used to prove a lock entry is not stale", and the struct is still `pub` although no public API takes or returns it any more (its only remaining use is internal source-map overlap grouping in `common.rs:528-545`). Fix the doc; consider `pub(crate)` once QSL-343 drops its last import. | crates/quire-contract-model/src/checked_package/shared.rs:323-336 |
| FND-005 | low | Test comment still justifies the same-identity/version, different-digest refusal with "The package evidence can attest only one digest per identity/version locator ... by the pre-existing per-item digest check"; the PR rewrote the matching FR-038 class-2 paragraph to "the lock contradicts itself" but left this comment. | tests/it/checked_package_v2_reader.rs:1386-1390 |

## Verdict

Mergeable after FND-001; FND-002 to FND-005 are low and may be folded into the same fix round. Checked and correct:

- Deletions are complete: no remaining reference to `insert_artifact_bytes`, `insert_artifact_digest`, `ArtifactDigests`, `artifact_digest`, `SuppliedDependencyPackage` or "digest store" outside historical `reviews/`. No unused import, parameter or helper is left (`std::borrow::Cow` removed from common.rs and evidence.rs; `digest_bytes` still used by `lower.rs:314`; `artifact_locator` still used by source-map grouping; `evidence` is still needed by `validate_lock` for models and dependencies). Clippy `-D warnings` passes on both lanes.
- Each kept site is identity or structure binding, not a lock-vs-itself comparison, and seven of eight go red when disabled (table above). M4 (same identity+version, different digest) is a real contradiction check: two documents can each claim one identity/version, so the lock can name two contents for one locator.
- `dependency_references.rs:111` deletion: the compared version was the caller's own claim, and `CheckedPackageV2` carries no self-declared library version, so the `package_id` binding at :111 (was :121) is the only content binding available. Correct per the ruling.
- Public API: `insert_dependency_package(identity: impl Into<Box<str>>, package: impl Into<Arc<CheckedPackageV2>>)`; the map now holds `Arc<CheckedPackageV2>` directly; `dependency_package(&str) -> Option<&CheckedPackageV2>` via `AsRef::as_ref`; `Arc` sharing preserved. `CheckedPackageEvidence` has no serde derive, so no serde impact. Derives (`Clone, Debug, Default, Eq, PartialEq`) still hold with `Arc<CheckedPackageV2>`.
- Downstream list in the PR body is complete: QSL `qsl-package/src/emit.rs:967`, `qsl-package/src/checked_v2.rs:970`, `qsl-package/src/checked_v2/tests.rs:211,212,216,1326`, `qsl-package/src/emit/tests.rs:208`, `tests/it/config_version_spine.rs:820`; CG `tests/exact_scalar_support/package.rs:1069`, `tests/composite_equality_support/package.rs:516`. QSL `emit/extent_agreement.rs` uses `CheckedPackageEvidence` but none of the removed methods. No other repo under /home/peter/dev references the removed API or `CheckedPackageRefusalCause::RevisionMismatch`.
- `stale_dependency` stays the code on kept checks (rename is upstream FR-322); only the doc text in FND-003/FND-004/FND-005 still says stale/attest in the removed sense.
- No shim, no retained no-op, no compatibility layer.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | A test comment still says the caller attests source digests: "A raw source digest is outside the preimage: every reference to the source moves with it, and the caller attests the new bytes digest." After this PR the caller attests nothing; the edited package admits because no reader check reads the raw source digest. Several other test comments in the same file still use "attest" for supplying a domain package document (lines 856, 1427, 1436-1437, 1487, 1512, 1619, 1633, 1647, 1664, 1712); those are loose but not wrong. | tests/it/checked_package_v2_reader.rs:809-810 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 32e3371: `CheckedPackageRefusalCause::RevisionMismatch` and its doc deleted; the wire `CheckedDiagnosticCause::RevisionMismatch` stays |
| FND-002 | fixed | 32e3371: the digest-domain check deleted from `admit_document`. Verified dead: `admit_document` is `pub(super)` with one caller, `admit_selection`, which is `pub(super)` with one production caller, `validate_domain_packages` (v2/mod.rs:1124). That function sweeps every selection's `digest_domain` at mod.rs:1085 and returns before its admit loop. The only path to it is read -> validate -> validate_lock (mod.rs:950), and the version dispatcher reaches V2 only through the same read. The dependency path (`admit_dependencies`) never calls it. Gate green after the deletion |
| FND-003 | fixed | 32e3371: the `StaleDependency` doc now lists package_id, preimage, projection, domain package document, dependency package_id and self-contradictory lock |
| FND-004 | fixed | 32e3371: doc rewritten to the source-map grouping role. `pub` is kept because CG test helpers still import the type until they drop `insert_artifact_digest`; that reason is accepted |
| FND-005 | fixed | 32e3371: comment now reads "The lock selects two documents for one identity/version locator" |
