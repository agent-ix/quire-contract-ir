---
id: SR-796
title: "gap analysis of PR 253 (IR-530 artifact references, IR-503 catalog words)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@26c603a0dada88fd7d5af48354cd682e354eef1a; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/tests.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/model/functional/FR-019-rust-library-interface.md, spec/model/matrix/tests.md, spec/model/matrix/TC-058-model-crate-public-interface.md, spec/core/matrix/tests.md, spec/tests.md, spec/assurance/AD-004-checked-package-seam.md, tests/it/checked_package_v2_artifact_refs.rs, tests/it/checked_package_v2_catalog_words.rs, crates/quire-contract-model/src/checked_package/v2/operations.rs, crates/quire-contract-model/src/checked_package/v2/operation_catalog.rs, crates/quire-contract-model/src/lib.rs"
review_set: subset
---
# SR-796: gap analysis of PR 253

## Summary

Ticket: IR-530 (code also covers IR-503). Reviewed head 26c603a0dada88fd7d5af48354cd682e354eef1a
against origin/main 35c098f, which is current. I ran `quire coverage --scope . --strict` at
both commits. Base: 163/209 backed, 23 unbacked. Head: 186/209 backed, 21 unbacked. The two
rows that left the unbacked list are TC-058 and FR-019-AC-5. The other 21 match the coder's
gate log (`scratchpad/ir-530-final-ci.log` ends `head=26c603a... exit=2`, and only `spec`
fails). FR-038's own document rises from 42/67 to 63/67 backed, which is AC-46..61 plus
AC-65..69.

Every test bound to AC-46..61 and AC-65..69 was read against the AC text and the TC-048
procedure, and each binding is correct. AC-62..64 (IR-535) and AC-45 plus AC-5's
model-owner clause (IR-505; PR #250 is not merged) stay planned in the FR-038, TC-048 and
spec/tests.md rows.

FR-035 says "no golden records a region source". That holds: no JSON under `corpus/`,
`schemas/` or `tests/` contains `quire.source.bytes`. The `revision` keys under
`corpus/contract-v0.1` are v0.1 entity revisions, not source refs. AD-004's evidence row
already states the `{authority, identity}` shape.

There is no compatibility layer, no new pin or SHA beyond the one lock entry, no vendored
catalog and no QSL dependency. Cargo.toml is unchanged. The PR title has no bare ticket id,
the body opens "Part of IR-530 and IR-503", the PR is draft and held, and the body uses
full SHAs only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The `#[trace("TC-058", "FR-019-AC-5")]` tag on `tc_058_the_artifact_reference_member_sets_are_exact` backs only AC-5's member-set clause. The strict gate now counts FR-019-AC-5 and TC-058 as backed (baseline 23 to 21), so the three clauses that are still planned leave the gate's unbacked list: no glob re-export, the public-item inventory, and the `quire_contract_ir` path probe. The model crate root still has seven globs. The matrix prose is honest (🚧, "remain planned"), but the only red signal for planned work is now prose. Split the member-set clause into its own AC, or keep a gate-visible marker for the planned clauses | tests/it/checked_package_v2_artifact_refs.rs:591, spec/model/matrix/tests.md:17,29 |
| FND-002 | low | The FR-038 matrix row this PR edits still lists its ACs as "... FR-038-AC-35 through FR-038-AC-68", which omits AC-69, though the status now calls AC-69 implemented. It also keeps "(42 of 62 FR-038 rows)", but quire now counts 63/67 backed for FR-038. Both are stale (the range is pre-existing). Update the range and the count, or drop the count | spec/checked_package/matrix/tests.md:15 |
| FND-003 | low | AC-66's body-root clause requires the 17 identities to refuse "whether its laws, mode, member, leaves and arguments agree or contradict". The unit test's contradicting case varies laws, mode, member and arguments, but never `leaves`. It also runs at the operation step, not through a package read as the TC-048 procedure says ("Build a package ... and read each"). Only one identity per class goes through a package read. A regression that compared leaves before the refusal would pass. Add a contradicting `leaves` value | crates/quire-contract-model/src/checked_package/v2/operations.rs:5007-5050 |

## Verdict

The matrix flips match real backing for AC-46..61 and AC-65..69, and every bound test can
fail (see SR-795's mutation probes). The coverage numbers reconcile. FND-001 is a
coverage-gate weakening that needs a decision before merge. FND-002 and FND-003 are small
and can be fixed in the PR.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | The doc comment on the renamed `tc_018_the_artifact_reference_member_sets_are_exact` still opens "(FR-019-AC-6, TC-058's member-set probes)". After the split, the member-set probes belong to TC-018 and TC-058 has none, so the comment contradicts the trace note five lines below and TC-058's own text. Quire does not bind it (TC-058 stays unbacked), so this is wording only. Change it to "TC-018's member-set probes" | tests/it/checked_package_v2_artifact_refs.rs:582 |

## Dispositions

Round 1, reviewed at a851fe47219bfd411d0327fb40cd6a9f464bc207 (fix delta f7286bb..a851fe4; the
rebase onto ccf34fe is excluded). `quire coverage --strict`: 185/213 backed, 23 unbacked, with
FR-019-AC-5 and TC-058 back on the unbacked list. FR-019 is 4/6 with AC-6 backed. FR-038 is
63/70. The grammar baseline is 1 (FR-014, pre-existing). `make test` passes: it 199, model 96,
doctests 7. The coder's log ends `head=a851fe4... exit=2`, and only `spec` fails.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | a851fe4: FR-019-AC-6 holds the member-set clause, with verification Test (TC-018); TC-018 covers FR-018..FR-020. AC-5 keeps no glob, inventory and path only. The test is renamed `tc_018_...` and traced TC-018/FR-019-AC-6. TC-058's text, row and cases have dropped the member-set probes, and spec/tests.md and core StR-003 are consistent |
| FND-002 | fixed | 6cf7afc: the row now lists "FR-038-AC-35 through FR-038-AC-72" and "(63 of 70 FR-038 rows)", which matches quire's 63/70 |
| FND-003 | fixed | 6cf7afc: the AC-66 unit test's contradicting case now sets `leaves`, and the TC-048 procedure states the two levels (all 17 at the operation step, one per class through a package read) |

Round 2, reviewed at 46c490a359f918f9a724f40803131aedd31cdf28. The delta from a851fe4 is one
commit that changes one comment line, with no rebase. origin/main is still ccf34fe, and the
merge is clean. `quire coverage --strict`: 185/213 backed, 23 unbacked, which is the
baseline. fmt-check passes. No remaining text credits member-set probes to TC-058.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-004 | fixed | 46c490a |
