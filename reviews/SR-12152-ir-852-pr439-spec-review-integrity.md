---
id: SR-12152
title: "IR-852 spec review (integrity)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@edd6bf4fe524637261c7d6a5c0bfbbaa33efe77d (PR #439); spec/checked_package/functional/FR-038-consume-checked-package-v2.md (AC-252 reword, AC-269..278), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

# IR-852 spec review (integrity)

## Summary

Ticket: IR-852. This review checks consistency, completeness and atomicity across
AC-269..278, the reworded AC-252, the TC-048 section and the tests.md rows. Clean:
the ids AC-269..278 are contiguous after AC-268 and collide with nothing on
origin/main. The tests.md FR-038 range gains "FR-038-AC-269 through FR-038-AC-278"
and ten summary rows, each PLANNED / UNRUN to match the criteria table. The
TC-048 procedure covers every AC and says to derive the expected values from the
fixture. The FR-019 Public items row is left to the code change, as AC-267 did for
IR-851. `quire validate` passes on the three changed files (only the
module-duplication warnings, which predate this PR). The refusal order inside
`check_relationship_member` was traced (operations.rs:2132-2286): `declaration` →
form → `name` → operand 0 → operand 1 → direction → destination object → key →
multiplicity → homogeneity → `result_type`. After it, `check_function_call`,
`check_mode_type` and `check_leaf_count` run (operations.rs:868-895). The
structural dependency join runs before every operation-stage check (mod.rs:1989).

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-038-AC-252 (reworded) | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | Implemented (IR-824). ... its operation.member holds a population only through the optional population key that FR-038-AC-269 through FR-038-AC-278 specify, which the typed accessor reaches_population reads. |
| FR-038-AC-269 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | two Type occurrences at ordinals 1 and 4 ... occurrence the Type occurrence at ordinal 1 |
| FR-038-AC-270 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | two reaches applications in one package naming the same population node return equal selectors |
| FR-038-AC-271 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | crate-internal post-admission mutation ... refuses UnresolvedPopulation for each |
| FR-038-AC-272 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | is not a node id, or is a node id with no node in the graph, is refused by the reader with missing_declaration/missing-selection |
| FR-038-AC-273 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | for each of the two |
| FR-038-AC-274 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | including a subtype of that type ... and one over exactly that type is admitted |
| FR-038-AC-275 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | A quire.op.model.navigate application whose operation.member carries a population key is refused |
| FR-038-AC-276 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | a package that omits that id, and a package that lists a population id no application names, are each refused with invalid_semantic_graph and no cause |
| FR-038-AC-277 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | a package that carries an invalid population key and an earlier existing defect is refused for the earlier defect |
| FR-038-AC-278 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | exhaustively matches CheckedReachesPopulationError without a wildcard arm, so a fourth variant fails to compile |
| The reader check | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | after every check it makes today for that application, so that a package without the key ... yield their present outcome at the present pointer and in the present refusal order |
| TC-048 population section | examined | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md | Combine an invalid population with an earlier existing defect and assert the earlier defect is refused. |
| tests.md AC-277 row | examined | spec/checked_package/matrix/tests.md | the reader check runs after the existing checks |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Where the new check sits in the refusal order is underdetermined, and AC-277's "earlier existing defect" cannot be tested as written. The prose puts the check after every check `check_relationship_member` makes. But three application checks run after that function: `check_function_call`, `check_mode_type` and `check_leaf_count` (operations.rs:868-895). So a reaches application with an invalid `population` and a leaf or mode defect refuses at `population`, while the tests.md row says "the reader check runs after the existing checks". The structural join (mod.rs:1989) also runs before every operation check. An invalid `population` whose id is missing from `dependencies` therefore refuses `invalid_semantic_graph` ahead of any earlier operation-stage defect. A test that pairs the two gets the join refusal, not "the earlier defect". AC-277 also bundles three behaviours: idempotence, the regression of the AC-252..268 packages, and the ordering. Fix: name the position. Either "after the `result_type` check and before `check_mode_type`", or "after every check of the application". Make the TC and the tests.md row say the same. State that the invalid-`population` package in the ordering case lists the id in `dependencies` (join-consistent). Split the ordering clause into its own AC. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3370-3375, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4756, spec/checked_package/matrix/tests.md:94, crates/quire-contract-model/src/checked_package/v2/operations.rs:858-895, crates/quire-contract-model/src/checked_package/v2/mod.rs:1981-1990 |
| FND-002 | low | Reworded AC-252 keeps its "Implemented (IR-824)" status, but it now contains a clause, "holds a population only through the optional `population` key ... which the typed accessor `reaches_population` reads", whose behaviour is PLANNED under AC-269..278. Today the reader admits any extra member key unread. So the implemented row now states something the code does not do, and the existing AC-252 tests do not back it. Fix: keep AC-252 as it was and put the cross-reference in the section prose. Or mark the new clause as PLANNED (AC-269..278). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4731, crates/quire-contract-model/src/checked_package/v2/operations.rs:199-207 |
| FND-003 | low | AC-272 names two inputs (a value that is not a node id; a node id with no node) without "for each", while AC-273 says "for each of the two". A single test with one of the inputs would satisfy it as written. Fix: add "for each". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4751 |

## Verdict

PASS WITH FINDINGS. Ids, ranges, PLANNED statuses, the TC procedure and the FR-019
deferral are consistent. FND-001 must be settled before the code PR, because the
test author cannot build AC-277's ordering case without it.

## New findings (disposition pass 1)

Reviewed at fffaf6495052405fa07a0a6a145bbbeb079bcfce.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | high | AC-280's second clause cannot pass. It says "an invalid `population` on an application that also has a mode or leaf-count defect is refused at `population`, because the `population` check precedes `check_mode_type` and `check_leaf_count`". The catalog entries for `quire.op.model.reaches` and `quire.op.model.navigate` have `mode: null` and `leaves: null` (quire-verification-contracts checked-operation-catalog-v1.json). So a mode on such an application is refused `invalid_package`/`operation-mode-mismatch` at `operation/mode` by the identity stage (operations.rs:712-726). A supplied leaf is refused as a law mismatch at `operation/leaves/0` (operations.rs:~659). Both run before `check_relationship_member`. For these operations `check_mode_type` returns at `operation.mode.as_ref()?` and `check_leaf_count` returns at `entry.leaves.as_deref()`, so neither can ever refuse after the population check. The TC-048 step "assert the refusal is at `population`" would fail. Fix: drop the clause, or replace it with the true order (a mode or leaf defect is refused at the identity stage, before `population`). The prose sentence saying `check_mode_type` and `check_leaf_count` "follow it" is true, but those checks are inert for reaches and navigate; say so. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4786, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:2373-2376, crates/quire-contract-model/src/checked_package/v2/operations.rs:655-726, crates/quire-contract-model/src/checked_package/v2/operations.rs:2673-2674, crates/quire-contract-model/src/checked_package/v2/operations.rs:3624-3626 |
| FND-005 | medium | AC-279's "a `max` above `u64`" case does not say how the value is spelled. Written as a JSON number, it is above 2^53, so the whole package is refused `noncanonical_wire` at the wire stage (FR-038-AC-74..80), long before the population check. Only the decimal-string spelling (for example `"18446744073709551616"`) reaches `population_maximum` and the `ill_typed` refusal. The negative case works in either spelling. Fix: say the above-`u64` `max` is the decimal string, matching the admitted decimal-string case. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4785, crates/quire-contract-model/src/checked_package/selected_population.rs:278-282 |

## Dispositions

Round 1, reviewed at fffaf6495052405fa07a0a6a145bbbeb079bcfce.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The position is pinned as "the last step of check_relationship_member, immediately after that function's result_type check". The join and the key stage are stated to run first. AC-277 is split, and the ordering case moved to AC-280 with the invalid id listed in `dependencies` (join-consistent). AC-280's first clause holds against operations.rs:2132-2286. Its second clause is wrong; see FND-004. |
| FND-002 | fixed | AC-252 now says "The optional population member key is specified separately by FR-038-AC-269 through FR-038-AC-280; it is not a charge or multiplicity member and does not change this criterion." No PLANNED behaviour remains in the implemented row. |
| FND-003 | fixed | AC-272 now says "for each of the two (a null value is the first)". |

## New findings (disposition pass 2)

Reviewed at 444e675529d80ac4561e1f98a0e3ff1aeb96e475.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | AC-280 now says that a reaches application with a mode or a leaf "is refused by the identity stage ... whatever its `population`". The TC step tests this "with and without an invalid `population`". But the structural dependency join and the application-key stage run before the identity stage (mod.rs:1989-1990). An invalid `population` whose id the application's `dependencies` do not list is refused `invalid_semantic_graph` by the join, not by the identity stage. Fix: limit "whatever" to a `population` that is a node id listed in `dependencies` (join-consistent), as AC-280's first clause already does. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4811, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:2377-2380, crates/quire-contract-model/src/checked_package/v2/mod.rs:1989-1990 |

## Dispositions (round 2)

Round 2, reviewed at 444e675529d80ac4561e1f98a0e3ff1aeb96e475.

| FND | outcome | reason |
| --- | --- | --- |
| FND-004 | fixed | AC-280 is reversed: a mode or a leaf on a reaches application is refused by the identity stage. I verified this against the code: the catalog gives `mode: null` and `leaves: null`, a mode is refused `operation-mode-mismatch` (operations.rs:712-726), and a leaf is refused `OperationLawMismatch` at `operation/leaves/0` (operations.rs:659-663). The prose says `check_mode_type` and `check_leaf_count` are inert for these operations. One residual is FND-006. |
| FND-005 | fixed | AC-279 now spells the case as the decimal string `"18446744073709551616"` and notes that a JSON number that large is refused earlier as `noncanonical_wire`. |
