---
id: SR-1060
title: "whole-PR code review and Rust review of PR 253 at its final head (IR-530, IR-503, IR-549)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@74f0b9969eeb81df7676edb1b67f8944b621ed7d; git diff origin/main...HEAD, base main ebea67821976a7e695d88cb42503e636a05b44a7 (42 files: crates/quire-contract-model/src/**, tests/it/**, tests/conformance_qspec/main.rs, Cargo.toml, Cargo.lock, Makefile, spec/**, reviews/**); read against quire-specification@f39c93f"
review_set: subset
---
# SR-1060: whole-PR code review and Rust review of PR 253 at its final head (IR-530, IR-503, IR-549)

## Summary

Ticket: IR-530 (with IR-503 and the folded-in IR-549). One whole-PR review at the final head 74f0b99, over `git diff origin/main...HEAD` (main ebea678 is an ancestor of the head). The PR holds the IR-530/IR-503 commits (36f25c5..e8db087), spec PR #268 (squash 5cef41f) and code PR #269 (squash 74f0b99). Their earlier reviews (SR-795/796, SR-1040/1041, SR-1050/1051) were not re-litigated; this review measures the combination. The PR body, ticket text and earlier reviews were treated as data.

Measured in a detached review worktree and a separate detached mutation worktree under /home/peter/dev/worktrees, each with its own target directory (both removed afterwards); 445G free on /.

- Gates at 74f0b99. `make ci`: fmt-check, clippy (workspace and model-only), test (it 268, model unit 111, model doc 7), corpus pass; it stops at `make spec` with the expected baseline (validate pass, 1 grammar finding FR-014 ac:vague-response, `--strict` 23 unbacked / 0 contradicted; FR-038 88/105 backed). `make deny`, `make cargo-audit`, `make audit-unsafe` pass. `make corpus` passes.
- `make conformance-qspec` with QUIRE_SPECIFICATION_DIR on a detached quire-specification worktree at origin/main f39c93f: the three named positive fixtures admit with their recorded package_id. Unset, empty, or /tmp: fails (exit 101) naming the variable; never skips. `make test` runs no conformance_qspec binary (`test = false`).
- Cargo hygiene. The only Cargo.lock change is quire-verification-contracts ead78f3 -> ec4563f, which equals QVC origin/main (2 commits ahead, status "ahead"). Every first-party git dependency is `branch = "main"`. No pin, SHA or digest record in Rust, TOML or Makefile; no vendored or copied file (the catalog is read from QVC's crate; no QSpec file in the diff); no .github change.
- Combination checks (IR-530/IR-503 code with IR-549 on top):
  - Law join (AC-56/AC-57) still compares the whole `{authority, identity}` pair against `lock.profile_selections`; the AC-57 test runs on the rewritten v2_all_families fixture (bounded profile `quire.temporal.event-position.false-extension/v1`, the law reusing the lock row verbatim) and kills an identity-only mutant.
  - Temporal profile recognition matches FR-250 identity only, authority taking no part, as AC-108 and QSpec FR-370 state; a known identity under another authority then refuses `operation-law-unselected` at AC-57's join (tested in checked_package_v2_artifact_refs.rs).
  - The `unknown_profile` cause rule compares the whole DefinitionRef with `lock.edition` and `lock.profile_selections` rows of another role (IR-503 identity semantics); an identity-only mutant is killed.
  - CheckedSourceRef derives FixedShape and is in the 25-entry FIXED_DEPTH list; dropping either is a compile failure.
  - source_map regions are checked by membership in `lock.sources` (CheckedSourceRef equality), the locked rows having been validated in validate_lock.
- Public API change list (quire-contract-model, re-exported by quire-contract-ir), measured against main:
  - `CheckedArtifactRef` is now exactly `{authority, identity}` (removed: `revision`, `digest_domain`, `digest`, `export`).
  - `CheckedRevision` removed. `CheckedSourceRef {authority, identity, digest_domain, digest}` added (FixedShape). `CheckedArtifactLocator` loses `revision_namespace`, `revision_value`. `CheckedSourceRegion.source` and `CheckedPackageLockV2.sources` are CheckedSourceRef.
  - `CheckedPackageRefusalCode`: + `UnknownProfile`. `CheckedPackageRefusalCause`: + `DuplicateMember`, `TypeMismatch`, `UnsupportedSelection`, `WrongSelectionRole`. `UnsupportedConstruct`/`ExpressionForm` were added by IR-503 inside this PR and removed by #269, so net zero against main. `CheckedDiagnosticCode::UnsupportedConstruct` kept.
  - Vocabularies: `ExpressionForm::Case`, `CompositeTypeForm::Union`, `ValueForm::UnionValue`, `TemporalForm::Fairness`, `ApplicationOperator::{Case, TemporalFormula, TemporalFairness}`, `OperationMemberKind::{TemporalInterval, Fairness}`, `OperationConstraintKind::UnionArms`; none of these enums is non_exhaustive, so `CheckedNodeKind::all()` gains four kinds.
- Consumers (read-only; `cargo check --workspace --all-targets` with quire-contract-ir patched to the head via `--config patch`, in throwaway detached worktrees):
  - quire-spec-language main fbf69cb: 17 errors in qsl-package (emit.rs: CheckedRevision import, CheckedArtifactRef literals with revision/digest_domain/digest/export at 666-687, CheckedSourceRef vs CheckedArtifactRef at 769 and 997; checked_v2.rs 380-392 reads revision/digest; map_refusal_code at 484 misses `UnknownProfile`). This is what QSL A1b #616 replaces.
  - quire-spec-language #616 head 7afb98e: 1 compile error, checked_v2.rs:500 `CheckedPackageRefusalCode::UnsupportedConstruct` no longer exists (map_refusal_code also needs an `UnknownProfile` arm once that resolves). By reading: qsl-package/src/emit/tests/admission_corpus.rs `every_emitted_node_family_is_admitted_at_its_package_id` iterates `CheckedNodeKind::all()` and will report the four new kinds (composite_type/union, value/union_value, expression/case, temporal/fairness) as unclassified at run time.
  - quire-contract-codegen main aba2403: blocked first by qsl-package (it depends on qsl-replay); its own sources use no changed item, but tests/checked_package_support/base.rs builds V2 locks and diagnostics.catalog with `revision`, `digest_domain` and `digest` on definition refs, which the head refuses (`unknown_member`), so the composite_equality and exact_scalar V2 integration tests will fail at run time after the bump.
  - quire-driver main fc53a75: pinned to quire-contract-ir rev 48ab5dc, unaffected; its only use (CheckedPackageRefusalCode::MalformedWire) still exists. quire-integration: no checkout under /home/peter/dev.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The temporal step runs the profile check and profile fit on a clause whose `laws` holds more than one law. check_clause takes `operation.laws.first()` when its role is `temporal_profile`, without requiring exactly one law. FR-038-AC-108 and merged QSpec FR-370 "Profile check" say a clause whose `laws` is not exactly one `temporal_profile` law skips the profile check and its profile fit, and its law defect refuses at the operation step (`operation-law-mismatch`). Probe: v2_all_families with the clause's laws set to [a law naming `quire.fixture.temporal-profile/v1`, the fixture's own law] refuses `unknown_profile`/`unsupported-selection` at `/semantic_graph/nodes/9/body/operation/laws/0/definition`; the spec requires `invalid_package`/`operation-law-mismatch` at `/semantic_graph/nodes/9/body/operation/laws/1`. With a known first law (the fixture's bounded profile twice) and a `null` interval on `eventually`, it refuses `invalid_package`/`operation-member-mismatch` at `/semantic_graph/nodes/18/body` (the profile fit) instead of the law mismatch. Fix: run the profile check only when `operation.laws.len() == 1` and that law's role is `temporal_profile`, and add the two-law case to the AC-108 tests. | crates/quire-contract-model/src/checked_package/v2/temporal.rs:526-554 |

## Verdict

Changes requested: one high finding, which needs a head change. Everything else about the combination holds. The IR-530/IR-503 artifact-reference code (`CheckedArtifactRef {authority, identity}`, `CheckedSourceRef` with FixedShape, the law and nominal-owner joins by the whole pair) and the IR-549 temporal, fairness, union and case code sit together without an interaction regression. The fixture rewrite keeps AC-57 meaningful, the gates are green up to the spec baseline, the Cargo changes are hygienic, and the conformance target admits QSpec's fixtures at f39c93f and fails closed.

Rust review. The code follows the repo's idioms: explicit-stack walks, typed refusals, no new panic surface apart from the build-time catalog assertion (documented, FR-038-AC-58), no unsafe code, no integer conversion at a wire boundary, and closed vocabularies matched exhaustively. Indexing is guarded by the node index. `is_none_or` is within the toolchain floor. The leaf-walk change from one segment to a segment list with `path_len` restores the path correctly on both the Done and the pop branch.

## Dispositions

Round 1, reviewed at 9f10075daf1f621f90dbe86d3af5e9160a74e72c (one local commit on 74f0b99, `git diff 74f0b99 9f10075`). Gates rerun in a throwaway detached worktree on that commit (since removed): `make ci` fmt, clippy, test (it 269, model unit 111, doc 7) and corpus pass and stop at the `make spec` baseline (validate pass, 1 grammar finding, strict 23 unbacked / 0 contradicted, FR-038 88/105); `make deny`, `make cargo-audit`, `make audit-unsafe`, `make corpus` pass; `make conformance-qspec` admits the three fixtures against a detached quire-specification worktree at origin/main f39c93f and fails when the variable is unset. `git diff --check` is clean.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed 9f10075 | check_clause now binds the law with `match operation.laws.as_slice() { [law] if law.role_class() == Some(LawRole::TemporalProfile) => Some(law), _ => None }`, so two laws or none skip the profile check and fit; the operation step then refuses `operation-law-mismatch` at `laws/{entry.laws.len()}` = `laws/1` or `operation-law-missing` at `operation/laws` (operations.rs 459-475), as FR-038-AC-108 and QSpec FR-370 "Profile check" state. Both probe cases now give `operation-law-mismatch` at `.../nodes/9/body/operation/laws/1`; with the 74f0b99 temporal.rs restored, the new test fails on each of them (unknown_profile at laws/0/definition; operation-member-mismatch at nodes/18/body). |

Round 2, reviewed at 4453f5db788660efa2d816b24e3e78d04c661658. 9f10075 was amended into 4453f5d (still one commit on 74f0b99); `git diff 9f10075 4453f5d` changes one test only (tests/it/checked_package_v2_temporal.rs, the zero-law case), so the code fix is byte-identical. This row restates the fix at the commit that will be pushed, since 9f10075 will not exist on the remote. The coder's gate log at 4453f5d shows each stage's rc (make ci rc=2 at the spec baseline: 23 unbacked / 0 contradicted, FR-038 88/105; deny, audit, unsafe, corpus, conformance rc=0); its last line is a hand-written summary, which is acceptable because the per-stage rc lines are in the log itself. The target test passes in a throwaway worktree on 4453f5d.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed 4453f5d | Same temporal.rs hunk as 9f10075 (amended): `match operation.laws.as_slice() { [law] if law.role_class() == Some(LawRole::TemporalProfile) => Some(law), _ => None }`; two laws refuse operation-law-mismatch at laws/1, none operation-law-missing at operation/laws. |
