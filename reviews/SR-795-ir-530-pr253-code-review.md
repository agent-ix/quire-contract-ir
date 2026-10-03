---
id: SR-795
title: "code review of PR 253 (IR-530 artifact references, IR-503 catalog words)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@26c603a0dada88fd7d5af48354cd682e354eef1a; Cargo.lock, crates/quire-contract-model/src/checked_package/common.rs, crates/quire-contract-model/src/checked_package/shared.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs, crates/quire-contract-model/src/checked_package/v2/operation_catalog.rs, crates/quire-contract-model/src/checked_package/v2/operations.rs, crates/quire-contract-model/src/checked_package/v2/vocabulary.rs, crates/quire-contract-model/src/lib.rs, tests/it/checked_package_v2_artifact_refs.rs, tests/it/checked_package_v2_catalog_words.rs, tests/it/support/checked_package.rs, tests/it/checked_package_v2_reader.rs, tests/it/complete_v1_checked_package.rs, tests/it/checked_package_v2_lowering.rs, tests/it/checked_package_v2_dependency_reference.rs, tests/it/checked_package_v2_dependency_selections.rs"
review_set: subset
---
# SR-795: code review of PR 253

## Summary

Ticket: IR-530 (code also covers IR-503). Reviewed head 26c603a0dada88fd7d5af48354cd682e354eef1a.
The base is current: the merge base equals origin/main 35c098f51a0e0e126a3d05ef9a3647aa3527df39.
The rust-review lane is folded into this file. Every check below was run in a separate detached
worktree, not taken from the PR body.

Shape. `CheckedArtifactRef` is exactly `{authority, identity}` under `deny_unknown_fields`.
`CheckedRevision` is deleted. `CheckedSourceRef {authority, identity, digest_domain, digest}`
is the member type of `lock.sources` and of every `source_map` region and diagnostics locus
`source`. `CheckedArtifactLocator` drops its revision members. The `export` member and
`validate_unexported` are gone. No legacy reader or relabel path exists.

Checks. `validate_locked_artifact` checks the domain first, then empty members, then digest
hex (AC-53). `validate_definition_ref` checks only that `authority` and `identity` are
nonempty. A region or locus source must be a member of the lock rows (`BTreeSet::contains`
or `Vec::contains`, comparing all four members), and anything else refuses
`invalid_source_map`. Law joins compare `CheckedArtifactRef` equality, which is
`authority` plus `identity`. Law-definition shape errors go through the `OperationWire`
decode and stay `invalid_semantic_graph` (AC-36/AC-49).

Catalog. `read_catalog(bytes)` returns `CatalogReadError` with a serde path and the
offending word. The production catalog is read once through it, in `OnceLock`. The crate
holds no catalog copy: `CATALOG_BYTES` comes from `quire_verification_contracts`.
Cargo.lock changes one line, quire-verification-contracts ead78f3 to ec4563f. `make deny`
passes, including the one-copy awk gate.

IR-503. The six words are added. `is_unsupported` is exhaustive. `UnionArms` refuses
`ill_typed`/`operator-ineligible` in `check_operands`. The `TemporalInterval`/`Fairness`
member arm refuses the same way. Neither is a no-op arm, and both are unreachable behind
the operator refusal, which AC-69's wording allows. A body root refuses after the
unknown-operation and class checks and before laws, mode, member and arguments. A nested
term refuses in `validate_term` at its own `operator`. `UnsupportedConstruct` and
`ExpressionForm` are added. The refusal enums have no wire-string mapping in this repo, as
the coder said: there is no Serialize and no `as_str`. The `temporal.clause` operand 0
special case and the `Formula` to `temporal` family change are present.

Tests. `make test` passes: it 198, model lib 96, doctests 7. Six mutation probes were each
reverted, and all six were killed:

- domain check moved after the empty-member check: AC-53 test fails;
- `case` made admitted: both catalog_words package tests fail;
- nested refusal disabled: the nested test fails;
- a lenient `revision` re-added to `CheckedArtifactRef`: the AC-48 and AC-49 tests, the
  AC-58 unit test and the positive doctest fail;
- the clause operand-0 special case removed: the AC-68 unit test and 35 fixture tests fail.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The nested-application refusal also fires on `diagnostics.entries[].details[]` terms, because `validate_term` is shared and details are walked with `is_body_root: false`. Probe: a details term `{term: application, operator: case, ...}` refuses the whole package `unsupported_construct`/`expression-form` at `/diagnostics/entries/0/details/0/operator`. The same holds for a details root, not only for a nested term. FR-038 "Catalog words" names the positions as node body root, `arguments` element, `binding` and `aggregate`, and never mentions diagnostic details. So this refusal is unspecified and untested. It is fail-closed and probably the right behaviour, but a spec-silent refusal of the whole package in a PR whose spec lists positions needs a decision. Either amend FR-038 to name details and add a test, or exempt details | crates/quire-contract-model/src/checked_package/common.rs:699-712, crates/quire-contract-model/src/checked_package/v2/mod.rs:2046 |
| FND-002 | low | The `compile_fail,E0560` further-member probes do not check their error code on the pinned stable toolchain (1.98.1). With `revision: Option<Box<str>>` added to `CheckedArtifactRef`, the `revision: "r".into()` probe still passed, failing with E0277 instead. The positive doctest and `tc_058` caught that mutation, so the probe set as a whole holds, but each further-member probe alone proves only "does not compile". State in the doc comment that the positive literals carry the member-set check, or drop the misleading code annotations | crates/quire-contract-model/src/lib.rs:42-66 |

## Verdict

The code is correct against FR-038-AC-46..61 and AC-65..69 on every point listed above, and
no test was weakened. The fixture moves in TC-044 and TC-048 still pin exact locations. FND-001
is a spec/code scope question, not a wrong result. Decide it before merge, either with a
one-line spec amendment plus a test or with a details exemption. FND-002 is a nit. Mergeable
once FND-001 is dispositioned. The PR is held/draft by design.

## Dispositions

Round 1, reviewed at a851fe47219bfd411d0327fb40cd6a9f464bc207. The branch was rebased onto main
ccf34fe57c0837bff56503568c7f03c732779a59. This round reviews only the fix delta f7286bb..a851fe4
and excludes the rebase. The range-diff shows only matrix-text conflict resolution in the
rebased commits.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 6cf7afc: the leader decided to keep the refusal fail-closed. FR-038 prose, AC-66 and the TC-048 procedure now name `diagnostics.entries[].details[]` terms, root and nested, at `/diagnostics/entries/{i}/details/{j}/operator`. The amendment only adds, and narrows no other position. The new test `tc_048_a_diagnostic_detail_of_a_refused_class_refuses_at_its_operator` covers a root, an aggregate and a binding for all three classes. Mutation probe: exempting detail roots (`is_body_root: true` at v2/mod.rs:2049) fails the test |
| FND-002 | fixed | 6cf7afc: the lib.rs doc now states that stable rustc does not check the named code, and that the positive doctest plus `tc_018_the_artifact_reference_member_sets_are_exact` are the real oracles |

Round 2, reviewed at 46c490a359f918f9a724f40803131aedd31cdf28: the delta is a single
comment line. FND-001 and FND-002 stay fixed, and no new rows are added.
