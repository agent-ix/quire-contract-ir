---
id: SR-583
title: "PR #108 review — CheckedPackage V1 deletion and V2 domain package consumption (AD-006)"
type: SpecReview
analysis: gap-analysis
scope: "Contract IR PR #108 diff (a5154d39...48e0cdc): FR-038, TC-048, TC-050"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: reviews }
  - { target: "ix://agent-ix/quire-contract-ir/TC-048", type: reviews }
  - { target: "ix://agent-ix/quire-contract-ir/TC-050", type: reviews }
---

## Summary

Code review, Rust review and gap analysis of the Contract IR PR #108 diff,
base `a5154d39` to head `48e0cdc`. The PR deletes CheckedPackage V1 and the
V1-to-V2 migration contract under the owner's 2026-09-17 ruling: prerelease
software keeps no migration path, compatibility layer, fallback or frozen
legacy version. Contract IR now consumes exactly one contract, QSpec I04
`quire.checked-package/v2` at `5626bc8f` (AD-006 `sha256-jcs` domain package
model selections). FR-038 is the sole consumer requirement; TC-048 covers the
strict reader and identity re-derivation, TC-050 covers independent per-item
lowering.

Checked against upstream and against the repository's own spec:

- **Vendored bytes.** All vendored blobs equal the upstream tree at
  `5626bc8`. `PROVENANCE` names every vendored path, its blob and its
  SHA-256, and records every upstream proposal file this repository
  deliberately does not vendor, with a reason for each.
- **Strict ModelRef.** `CheckedDomainPackageRef` uses `deny_unknown_fields`. A
  lock reference or owner carrying `authority`, `revision` or `export`
  refuses as `unknown_member`. A foreign digest domain refuses as
  `digest_domain_mismatch`, checked before shape. An empty version refuses as
  `malformed_wire`. A version with no evidence refuses as `stale_dependency`.
  Each case is also checked against the vendored schema.
- **Typed sha256-jcs evidence.** `CheckedDomainPackageLocator` keys a separate
  `domain_packages` map. `validate_domain_package` never reads raw artifact
  evidence. A test proves that equal digest bytes attested only as a raw
  artifact refuse.
- **Owner join.** A model owner joins by domain package identity and needs a
  nonempty `node`. An unselected identity, an empty lock or an empty node
  refuses as `invalid_semantic_graph`.
- **Package identity.** Every preimage member changes the recomputed package
  id once re-derived, and is refused as `stale_dependency` when it is not,
  including the model selection
  (`tests/checked_package_v2_reader.rs:672-697`).
- **Complete-V1 lowering.** `tests/complete_v1_checked_package.rs` lowers
  every public node family against the current V2 reader with exact source,
  type, dependency, bound and claim correspondence, and reports exact and
  one-over resource accounting, including the lowering work budget's
  one-over `consumed` value.

Targeted tests run (Rust 1.98.1): `checked_package_v2_reader`,
`complete_v1_checked_package`, `checked_package_v2_lowering` — the three test
targets this PR leaves for the CheckedPackage surface. All pass. `cargo fmt
--check` is clean on the touched Rust files.

## Verdict

**PASS**. The code and tests are sound. This file replaces
`reviews/2026-09-16-pr-108-v2-domain-package-review.md` (`id: SR-581`), which
reviewed a superseded revision (`ea91a5c`) of this PR: its id collided with
`spec/reviews/checked-package-v2/base.md`'s own `SR-581`, its scope and a
relationship cited `TC-047` and `TC-049`, both deleted by this PR, it
narrated the deleted migration design, it listed four test targets this PR
deletes, and it published a `FAIL` verdict on a finding already fixed at this
head. That file is removed; this file is the current review of record.

## Findings

Every finding below was raised against this PR's prior revisions and is
fixed at this head.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The predecessor review (`id: SR-581`, `reviews/2026-09-16-pr-108-v2-domain-package-review.md`) reviewed a superseded revision: id collision, references to deleted `TC-047`/`TC-049`, migration-design narration, four deleted test targets, and a stale `FAIL`. Replaced by this file. | reviews/SR-583-pr-108-checked-package-deletion.md |
| FND-002 | medium | `spec/index.md:99-101` described a version dispatcher, a typed V1-to-V2 migration outcome and a frozen FR-035 V1 reader, none of which exist. Rewritten to the strict-reader/independent-lowering description that matches FR-038. | spec/index.md |
| FND-003 | medium | `tests/fixtures/checked-package/PROVENANCE`'s "Not vendored" list omitted three upstream migration-contract paths this tree deliberately does not vendor. Added, each with a one-sentence reason. | tests/fixtures/checked-package/PROVENANCE |
| FND-004 | low | `tests/complete_v1_checked_package.rs`'s `work: 27` and `tests/checked_package_v2_reader.rs`'s `limits.work = 27` were independent hand-maintained literals over the same all-families fixture and the same read-limits counter. Both now reference one shared `ALL_FAMILIES_READ_WORK` constant in `tests/support/checked_package.rs`, so the two sides cannot silently diverge. | tests/support/checked_package.rs |
| FND-005 | low | Bookkeeping: three FR-035-AC-2/TC-044 adverse cases from the pre-deletion suite are not present in the restored `tests/complete_v1_checked_package.rs`. See Coverage below — the same three cases exist against `TC-048`/`FR-038` in `tests/checked_package_v2_reader.rs`, and FR-035-AC-2 does not enumerate them, so the Test Matrix stays fully backed. | tests/checked_package_v2_reader.rs; FR-035-AC-2 |
| FND-006 | low | `tests/complete_v1_checked_package.rs` asserted `Failed { limit: 4, .. }` for the one-over lowering work case without pinning `consumed`. Now asserts `consumed: 5`. | tests/complete_v1_checked_package.rs |
| FND-007 | low | FR-038 Behavior said "the read context's raw artifact evidence" while Inputs said "package evidence" for the same, deleted-`CheckedPackageReadContext`-era concept. Unified on "package evidence". | spec/contract/FR-038-consume-checked-package-v2.md |
| FND-008 | low | FR-038-AC-2, TC-048 and a `tests/checked_package_v2_reader.rs` comment narrated the deleted design with "retired". Restated as current behaviour: a lock reference or owner carrying `authority`, `revision` or `export` refuses as `unknown_member`. | spec/contract/FR-038-consume-checked-package-v2.md; spec/contract/TC-048-checked-package-v2-strict-reader.md; tests/checked_package_v2_reader.rs |
| FND-009 | low | `spec/contract-test-matrix.md`'s TC-050 row credited only Contract IR PR #107 while the FR-038 and TC-048 rows credit #107/#108. Made consistent. | spec/contract-test-matrix.md |
| FND-010 | low | `spec/reviews/checked-package-v2/base.md`'s `evaluated_revision`, `review_date` and closing checklist bullet were stale against that file's own post-deletion Summary, its own FND-003 and its own checklist. Made truthful. | spec/reviews/checked-package-v2/base.md |

## Coverage

- Touched criteria: FR-038-AC-1 through FR-038-AC-6 are each backed by
  `tc_048_*` tests in `tests/checked_package_v2_reader.rs` or `tc_050_*`
  tests in `tests/checked_package_v2_lowering.rs`.
- Narrowed adverse set (FND-005): the cross-domain `lock.sources[0].digest_domain`
  mutation (`DigestDomainMismatch`), an `unsupported` capability disposition
  (`UnknownRequiredCapability`) and an unknown top-level member
  (`UnknownMember`) traced `TC-044`/`FR-035-AC-2` before this PR. They are
  not restored against `TC-044`; the same three cases exist in
  `tests/checked_package_v2_reader.rs` tracing `TC-048`/`FR-038`
  (`UnknownMember` at line 226, `UnknownRequiredCapability` at lines 271 and
  484, `DigestDomainMismatch` at line 1070), and FR-035-AC-2 does not
  enumerate them, so the Test Matrix stays fully backed.
- Untraced behaviours / stubs: 0 in the diff.
- Semantic review: done for the reader, identity and lowering paths above;
  intent, test and code agree.
