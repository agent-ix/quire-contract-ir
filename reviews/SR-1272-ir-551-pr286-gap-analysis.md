---
id: SR-1272
title: "gap analysis of PR 286 (IR-551 FR-038-AC-119 through AC-122 to tests)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@65a034d7b14d64c150646ebef77f8403b9e3257d; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, spec/tests.md, tests/it/checked_package_v2_temporal.rs, crates/quire-contract-model/src/checked_package/v2/natural.rs"
review_set: subset
---
# SR-1272: gap analysis of PR 286

## Summary

Ticket: IR-551. This review maps FR-038-AC-119 through AC-122 and TC-048 "Timed interval
form" to the tests, checks the planned-to-implemented flips, and measures how strong the
test oracles are. It was done by hand (quoin 0.24.1, spec-artifacts-process@v0.26.0)
over `git diff origin/main...HEAD` at 65a034d. Sources read: the merged spec (the
FR-038 member rule "The timed interval form" with its stages 1 to 5, AC-104,
AC-119..AC-122, the Path and locus rows, AC-3) and merged QSpec FR-370 (AC-3, AC-8,
AC-9), FR-322 (AC-10; lines 810-811 on the order of checks) and the schema's
`NonNegativeRational`, all at quire-specification origin/main 2f846f8.

Criterion by criterion:

- **AC-119** (`tc_048_the_timed_form_admits_under_the_timed_profile_on_each_interval_operator`):
  all eight operators, all four end pairs, `lower` 1/2, `[3,3]` and `null`. Every clause
  is backed.
- **AC-120** (`tc_048_a_timed_bound_outside_the_rational_pattern...`,
  `tc_048_a_pattern_failure_in_a_later_node...`): all seven listed inputs under all five
  profiles, the two-failure input at `lower`, the negative numerator on all eight
  operators, and the cross-node case in both position orders. The "before any identity
  check" clause is not met for `package_id` and is not tested (FND-001).
- **AC-121** (three integration tests plus the unit tests
  `tc_048_compares_rationals_exactly_beyond_2_pow_64` and
  `tc_048_comparison_work_is_charged_and_stops_at_the_limit`): every listed input with its
  exact code, cause and pointer, including the beyond-2^64 pair in both orders. The
  work-limit case checks `incomplete`, `work`, `consumed == needed` and the body pointer at
  exactly one under the limit, for both the cross-multiplication and the GCD. Mutation
  runs: moving lowest terms after the temporal step fails the profile-fit case, and giving
  `check_bounds` an unmetered meter fails the work-limit test. A float comparison
  (admitting the 2^64+1 pair) or a checked 64-bit parse (refusing the swap) would each fail
  `tc_048_the_timed_bounds_compare_exactly...`.
- **AC-122** (`tc_048_the_timed_form_fits_only_the_timed_profile`,
  `tc_048_a_timed_form_member_set_defect...`, `tc_048_a_member_set_defect_is_reported_after...`):
  the fit under all five profiles; `half` in each end, the missing end and the fifth member
  under infinite-trace and timed, each as `operation-member-mismatch` at `operation.member`,
  and with rational bounds as `invalid-value` at `lower`. The later-clause ordering test
  checks its digest precondition and skips salts where it does not hold.
- **Expectations are hand-written.** Pointers are built from positions and codes are
  literals. No oracle calls the implementation.
- **Spec edits.** The 🚧 marks and the "until that code lands" text are removed from every
  place that held them: FR-038 member rule, Interval-bounds step, profile-fit step, the
  five Path rows and the AC-119..122 rows; TC-048 intro and section; the FR-038 and TC-048
  matrix rows; and `spec/tests.md`. A repo-wide grep for "IR-551" planned wording,
  "until that code", `NullOnly` and `IntervalFit::Any` finds nothing.
  `quire coverage --scope . --strict` gives 23 unbacked rows at both the base 4762ce2 and
  the head. FR-038 goes from 115/119 to 119/119, so AC-119 to AC-122 are now backed; they
  were never in the unbacked list because they were planned rows. Total coverage moves
  from 260/285 to 264/285. `quire validate` passes with one grammar finding (FR-014,
  `ac:vague-response`), which predates this PR.
- **spec-review not run separately.** The `spec/**` changes only flip status and remove
  "planned" wording. They add no requirement statement, AC or relationship. They are
  checked here instead.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-120 is marked implemented, but its stage clause "in the term walk before any identity check" (FR-038 stage 1: "before every identity check") is not met. The term walk runs after the `package_id` recomputation, so a pattern failure in a package with a stale `package_id` refuses `stale_dependency` (probe, see SR-1271 FND-001). No test builds a package with stale identities, so nothing catches this. AC-116 already treats the `package_id` check as an identity check | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1155-1169; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2329 |
| FND-002 | low | The FR-038 stage-2 rule "at the first such bound in ascending `node_id` digest order and then member order" has no test with two non-reduced nodes. A mutant that iterates `validate_timed_bounds_reduced` in position order passes all 42 temporal tests. No AC states this ordering, so this is a test-strength gap, not an AC gap | crates/quire-contract-model/src/checked_package/v2/temporal.rs:351 |
| FND-003 | low | Two digest arrangements the spec names hold only by accident and are not asserted. AC-121 says the profile-fit defect is "in a lower-digest node", and in `beside_fit_defect` the fit node (position 23) does have the lower digest today (probe), but the test does not check it. TC-048 AC-120 names "the lower-digest node" while the test varies position; both digest arrangements happen to occur. A fixture change could reverse these without any test failing. Assert the digest order, or build both orders, as `tc_048_a_member_set_defect_is_reported_after...` does | tests/it/checked_package_v2_temporal.rs:2607-2640; tests/it/checked_package_v2_temporal.rs:2470-2505 |

## Verdict

AC-119, AC-121 and AC-122 are fully backed by strong, hand-written tests, and the
mutation runs confirm they discriminate. AC-120 is backed for every listed input and
pointer, but its stage clause is unmet for the `package_id` identity check (FND-001, the
same root cause as SR-1271 FND-001). The spec flips, matrix rows and coverage are
consistent: 23 unbacked rows before and after, and FR-038 at 119/119.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | The fix round added "before any identity check" to AC-97, the integer form. The integer cases that back it (`closed("1.5", "0")` and `closed("0", "-2")` with stale identities) are in `tc_048_a_bound_outside_the_pattern_is_refused_ahead_of_every_identity_check`, which traces only FR-038-AC-120. Add FR-038-AC-97 to that test's `#[trace]` so the amended AC-97 clause has a traced test | tests/it/checked_package_v2_temporal.rs:2936-2937; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2309 |

## Dispositions

Round 1, reviewed at 4ce54d157870be2ae3acc00bf5d2e731db184fd8.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 4ce54d1. AC-120 and FR-038 stage 1 now read "in strict wire validation, before the `package_id` recomputation, any node-key check and any temporal step (so also in a package whose `package_id` and node ids are all stale)". The code meets this (SR-1271 FND-001). The new test `tc_048_a_bound_outside_the_pattern_is_refused_ahead_of_every_identity_check` is a real oracle: a control with good bounds refuses at the identity check (`stale-node-key` at the root's `node_id` when everything is stale, `stale_dependency` when only `package_id` is stale), and timed and integer bad bounds refuse `invalid-value` at the bound in both stale shapes. It fails when the check is moved after `package_id` |
| FND-002 | fixed | 4ce54d1. `tc_048_the_first_non_reduced_bound_is_the_lower_digest_nodes` builds two non-reduced nodes in both digest arrangements, asserts the arrangement, and expects the lower-digest node. The position-order mutant now fails it |
| FND-003 | fixed | 4ce54d1. The AC-121 fit-defect case now builds both arrangements (fit node lower and higher digest) and asserts each. The AC-120 cross-node test asserts `root < added` (position, as the AC words it), and TC-048 now says "the first node ... a later node, and the two swapped". Moving lowest terms after the temporal step still fails the AC-121 test |

Round 2, reviewed at 43646a2b3fca1c475de55718d5307217840a6e57.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-004 | fixed | 43646a2. `tc_048_a_bound_outside_the_pattern_is_refused_ahead_of_every_identity_check` now carries `#[trace("TC-048", "FR-038-AC-97", "FR-038-AC-120")]` and a matching "Tracing:" line. Its integer stale-identity cases back AC-97's "before any identity check" clause. TC-048's matrix row already lists FR-038-AC-97. Strict coverage: 23 unbacked rows, FR-038 119/119 |

