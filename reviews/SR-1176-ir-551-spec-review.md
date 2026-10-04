---
id: SR-1176
title: "spec review of PR 279 (timed interval form, FR-038-AC-119)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@48f27d676fad56fa5f7e7d69fefb39c254e21880; git diff origin/main...HEAD (base 1117eba64f329fad9f7324b0ec9d2149be66f07f): spec/checked_package/functional/FR-038-consume-checked-package-v2.md (integer-form member rule, new timed-form member rule, interval-bounds stage, timed/v1 profile-fit bullet, Path and locus rows, AC-104, AC-119), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md (Description, Timed interval form section), spec/checked_package/matrix/tests.md (FR-038 and TC-048 rows), spec/tests.md (Checked package row); checked against crates/quire-contract-model/src/checked_package at the same sha and quire-specification origin/main 2f846f8 (FR-370, FR-322, FR-255, TC-303, proposals/checked-package-v2/schema.json)"
review_set: base
---
# SR-1176: spec review of PR 279

## Summary

Ticket: IR-551. Spec-only PR. It replaces FR-038's IR-551 known-gap text with
a reader rule for the timed interval form of merged QSpec FR-370. The form is
`{lower, upper, lower_end, upper_end}` with rational bounds. The PR adds planned
FR-038-AC-119, a TC-048 section, and matrix rows. This review applies the
contradiction, ambiguity, testability, trace and matrix sub-checks. The form
check is SR-1177.

What I measured at the reviewed sha:

- `quire validate` gives 376/377 docs grammar-clean, with the one finding in
  FR-014 line 137, which is already on main. `quire coverage --strict` gives 23
  unbacked rows. Both match the baselines.
- The published schema (QSpec 2f846f8, `schema.json:84,121`) defines the
  `interval` as a `oneOf` of three branches: `null`, `{lower, upper}`, and
  `{lower, upper, lower_end, upper_end}`. Every branch has
  `additionalProperties: false`. The timed branch's bounds are
  `NonNegativeRational`, `{numerator: NonNegativeIntegerString, denominator:
  PositiveIntegerString}`. The ends are `enum [closed, open]`. So numerator and
  denominator are decimal strings, as the PR says. Every AC example uses string
  spellings.
- The "today" statement is accurate. `interval_bound_outside_pattern`
  (`temporal.rs:230-251`) refuses any bound that is not a non-negative integer
  string. A timed form's `lower` is an object, so today's reader refuses it
  `invalid-value` at `.../interval/lower` in the term walk (`common.rs:756-773`),
  under every profile. The text is labelled planned.
- These parts match merged QSpec and have no finding:
  - The profile-fit pairs: the timed form refuses `operation-member-mismatch` at
    the application under the four other profiles, and integer `[a,b]` and
    `[a,*]` refuse it under timed/v1 (FR-370-AC-3, AC-8, and the "Placement and
    profile fit" table).
  - `(3,3]`, `[3,3)`, `(3,3)` and `5/2 > 2/1` refuse `invalid-value` at
    `/semantic_graph/nodes/{n}/body`, after profile fit.
  - `[3,3]` admits (FR-255, a punctual interval).
  - `{1/2}` is an admitted bound. `2/4` refuses with the code
    `invalid_semantic_graph`.
- The examples are arithmetically correct: 1·3 < 2·2, so 1/2 < 2/3; 5/2 > 2/1.
- These edits are complete and correct:
  - AC-104's "pins no outcome" text is replaced by a pointer to AC-119.
  - The KNOWN GAP text is removed everywhere. No "known gap" remains in FR-038,
    TC-048 or the matrices.
  - The FR-038 matrix range, the TC-048 AC list, the TC-048 Description and the
    spec/tests.md row all add AC-119.
- No AC row carries tracker state.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | One IR reading contradicts merged QSpec. The new rule refuses a non-reduced timed bound `invalid_semantic_graph` inside the term walk, in member order. AC-119 then pins that a package with a non-reduced `lower` and a negative `upper` refuses `invalid_semantic_graph` at `lower`. Merged FR-370 says a bound outside its form's pattern "fails the published schema and refuses `invalid_package`/`invalid-value` at that bound's pointer during strict wire validation, before any step of this requirement runs". FR-322 "Identity and validation" validates the closed wire shape before graph admission. FR-322-AC-10 makes a non-normalized rational an `invalid_semantic_graph` refusal, which is a later, semantic check. So QSpec refuses that package `invalid-value` at `upper`. The "IR reading" label does not cover an outcome that merged text decides the other way. FR-038 itself says the term walk is "one stage in effect with strict wire validation", so the reading also reorders refusals across nodes: a non-reduced bound in an early node beats a schema-pattern failure in any later node. Fix: run the lowest-terms check after the whole term walk, before the temporal step, and invert the AC-119 and TC-048 precedence example. The negative-lower case stays as written | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1140-1153, 1174-1177, 1460, 2295; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:798-800, 813-814 |
| FND-002 | medium | AC-119 and TC-048 do not say which bounds the fifth-member case uses, and the member rule gives different outcomes for the two choices. The rule says an interval of any other member set "has no form, its bounds judged against the integer pattern". Step 2 refuses `operation-member-mismatch` only when the bounds "pass the integer pattern". AC-119 ("one with a fifth member") and TC-048 ("a four-member interval with a fifth member") pin `operation-member-mismatch` at `operation.member` without naming the bounds. A tester's natural fixture is a valid timed form plus one member. Its bounds are rational objects, so the rule refuses it `invalid-value` at `.../interval/lower`. A timed form that lacks one end gives the same split. Fix: either say the bounds are integer strings, or discriminate a timed form by the presence of `lower_end`/`upper_end`, so that a mistyped timed interval is not reported as a bad `lower` | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1134-1138, 1154-1160, 2295; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:805-808, 820-822 |
| FND-003 | medium | The shape refusals are labelled "an IR reading, merged text silent", but merged QSpec decides these cases. FR-370 "Published artifacts" makes the published schema carry the interval forms. That schema's `oneOf` has closed branches and an end `enum`, so `lower_end: "half"`, a three-member interval and a five-member interval all fail the published schema at strict wire validation. IR refuses them later instead: at the operation step, after the identity checks and the temporal step, as `operation-member-mismatch` at `operation.member`. The label should name this divergence in stage and order from the schema, as AC-97 and AC-103 name theirs, or the rule should follow the schema | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1154-1160, 2295 |
| FND-004 | medium | AC-119 states several IR readings as fact and attributes them to merged text. (a) It locates numerator and denominator pattern failures at `.../interval/lower` and `.../interval/upper`. The member rule marks this as "an IR reading of 'that bound's pointer'", but AC-119 and Path-table row 1459 do not. A schema validator reports `.../lower/numerator`, and the crate's own `validate_rational` reports `.../numerator` and `.../denominator` (`identity.rs:611-620`). (b) It cites "(merged FR-370-AC-9)" for the denominator `"0"` and null-upper cases, but AC-9 states only negative bounds. The null-upper outcome rests on IR's member-set discrimination, which is an IR reading. Mark each in the AC row as AC-97 and AC-103 do | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1144-1150, 1459, 2295 |
| FND-005 | medium | AC-119 claims that "the comparison is exact and neither a float nor a fixed-width comparison", but its only instances (1/2 against 2/3, and 5/2 over 2/1) cannot fail an f64 or a u64 cross-multiplication. "Orders as less" is also not an observable outcome. AC-97 carries a beyond-2^64 pair for the integer form; AC-119 has none. The TC-048 section has a beyond-2^64 pair, but the AC it traces to does not require it. Fix: state that `[1/2, 2/3]` admits and `[2/3, 1/2]` refuses, and add a pair whose order a 64-bit or f64 comparison gets wrong | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2295; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:800-802 |
| FND-006 | medium | AC-119 is compound. One row holds about 20 outcomes across five stages: admission and the end variants; pattern failures; lowest terms and its precedence; the bounds comparison; and profile fit and shape. A failing test cannot point to one clause. The row also repeats AC-104's integer `{0,3}` and `{0,null}` refusals under timed/v1. It says "under that profile" and "under every profile" in one clause. It leaves "read back unequal" undefined: it could mean member equality, body bytes or node keys. Split it into about four ACs, one per stage, and drop the repeated AC-104 clause | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2280, 2295 |
| FND-007 | low | The integer-form paragraph still says, without qualification, "an `interval` with another member set refuses ... `operation-member-mismatch` at `operation.member`". To the integer rule, a four-member timed form is "another member set". The paragraph now conflicts with the timed rule, and with the new rule's `invalid-value` for an interval whose bounds are objects. Scope the sentence to the integer form | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1109-1112 |
| FND-008 | low | The rule's list "So a timed-form interval is read, in this order" puts Shape at step 2, ahead of profile fit and bounds. Shape defects are refused at the operation step, which runs after the temporal step. So another interval's profile-fit or bounds defect, in the same clause or a later one, is reported before this interval's shape defect. The list is right for one member only. Say so, or move Shape after step 4 | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1139-1166 |
| FND-009 | low | The Path and locus table has no row for the member-set defects (a missing end, a third member or a fifth member), which AC-119 pins at `operation.member` | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1459-1462 |
| FND-010 | low | The new rule adds GCD and unbounded cross-multiplication over digit strings the caller supplies. It does not say this work is charged to the work limit. The crate's existing rational check meters its GCD (`natural.rs:118-143`). No AC or TC case covers a one-over work limit on a timed bound | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1140-1153, 1164-1173 |
| FND-011 | low | The TC-048 expected results say pattern failures refuse "under every profile", but the procedure reads them only under timed/v1 | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:796-800, 810-813 |
| FND-012 | low | Several clauses repeat FR-370 nearly word for word: "so `[a, b]`, `(a, b]`, `[a, b)` and `(a, b)` are four distinct members", and "in lowest terms with a positive denominator as every other checked-package rational is". Restate them or cite them; do not copy | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1125-1133 |
| FND-013 | low | The PR body says the PR carries "refusal pairs and pointers" from merged FR-370. It does not list the IR readings it adds: the stage and locus of lowest terms, the bound pointer, member-set discrimination, shape refusals, null upper and precedence. A reader of the PR would take every pinned outcome as QSpec's | PR body |

## Verdict

Not mergeable as is. FND-001 pins an order that contradicts merged FR-370 and
FR-322. FND-002 to FND-006 leave AC-119 ambiguous, mis-attributed, compound, or
unable to fail an inexact implementation. The admission cases, the profile-fit
pairs, the bounds-check placement and pointers, the arithmetic, the string
spelling, the AC-104 edit, the removal of the known gap, the matrix rows and the
planned "today" statement are all correct.

## New findings (disposition pass 1)

Re-reviewed at agent-ix/quire-contract-ir@9d1879016a15023a9f55fa8185950066e705b4b7, which is fix commit 9d18790 on top of 48f27d6. AC-119 is now split into AC-119 to AC-122.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-014 | low | The rule places the lowest-terms check "after the term walk of every node and after the `package_id` and node-key recomputation, as the graph checks do, and before the temporal step". It does not place the check among the other graph checks, such as the application dependency join and the nominal-preimage and other `invalid_semantic_graph` checks. So a package with a non-reduced timed bound in one node and another graph-check defect in another node has no decided first refusal. Name the check's position in the graph-check sequence, for example "first among the graph checks" or "after the dependency join" | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1153-1166 |
| FND-015 | low | AC-121's beyond-2^64 pair does not state its ends. In f64 both numerators round to 2^64, so a float comparison sees equal bounds. With both ends `closed`, the float outcome is wrong for the refusing order but right for the swap, which admits either way. With an open end it is wrong for the swap only. So "the two swapped admit, outcomes that a float ... gets wrong" holds for neither choice of ends on its own. Pin both ends `closed`, and say that the refusing order is the one a float gets wrong. The swap still defeats a 64-bit parse | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2306 |
| FND-016 | low | AC-120 requires the same refusal under timed/v1, infinite-trace and "each of the three bounded profiles". TC-048's procedure reads the pattern refusals under timed/v1, infinite-trace and "one bounded profile" | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:800-803; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2305 |
| FND-017 | low | AC-121's work-limit clause says "the same package read with a work limit", but AC-121 names about a dozen packages, so "the same package" has no single referent. TC-048 places the clause after the beyond-2^64 pair. Name that package in the AC | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2306 |

## New findings (disposition pass 2)

Re-reviewed at agent-ix/quire-contract-ir@a03203ee8626f2a8acebce0de7310d8eec08db98, which is fix commit a03203e on top of 9d18790. Only FR-038 and TC-048 changed.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-018 | low | The new placement contradicts the frame rules. It says lowest terms runs "after the application dependency join and every other `invalid_semantic_graph` check of the nodes and ahead of the frame and state-clause step", and it concludes "a package with a non-reduced bound in one node and any other graph defect in another node refuses at the other defect". But the frame-body-semantics stage itself refuses `invalid_semantic_graph`: a frame array out of canonical order refuses at the frame body (line 2074), and AC-14 decides that refusal inside the frame step, after meaning-join defects. That stage runs after lowest terms. So a package with a non-reduced bound and a frame canonical-order defect refuses at the bound, against the "any other graph defect" sentence. Fix: scope the sentence to the graph checks that run before the frame step, and name the frame step's `invalid_semantic_graph` refusals as later | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1156-1165, 2072-2076 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 9d18790 |
| FND-002 | fixed | 9d18790 |
| FND-003 | fixed | 9d18790 |
| FND-004 | fixed | 9d18790 |
| FND-005 | fixed | 9d18790 |
| FND-006 | fixed | 9d18790 |
| FND-007 | fixed | 9d18790 |
| FND-008 | fixed | 9d18790 |
| FND-009 | fixed | 9d18790 |
| FND-010 | fixed | 9d18790 |
| FND-011 | fixed | 9d18790 |
| FND-012 | fixed | 9d18790 |
| FND-013 | fixed | 9d18790 (PR body refreshed; it now lists the six IR readings) |
| FND-014 | fixed | a03203e |
| FND-015 | fixed | a03203e |
| FND-016 | fixed | a03203e |
| FND-017 | fixed | a03203e |
| FND-018 | fixed | 008eb31 |

Disposition pass 1 verdict:

- **FND-001.** The lowest-terms check now runs as a graph check, after the term
  walk of every node and after the `package_id` and node-key recomputation, and
  before the temporal step. This agrees with merged QSpec:
  - FR-322 "Identity and validation" orders the checks as wire shape, then
    identity recomputation, then graph admission.
  - FR-322-AC-10 lists non-normalized rationals with stale-body keys as
    `invalid_semantic_graph`.
  - FR-370 says pattern failures come "during strict wire validation, before any
    step of this requirement".
  - FR-038's own order of checks (lines 553-557) ends "then the graph checks".
  - The AC-120 example agrees: an earlier node's non-reduced bound loses to a later
    node's pattern failure. So does AC-121: a non-reduced `lower` with a negative
    `upper` refuses `invalid-value` at `upper`.
- **FND-005.** The beyond-2^64 arithmetic is correct. (2^64+1)/3 against 2^64/3
  cross-multiplies to (2^64+1)·3 > 2^64·3, so it refuses, and the swap admits.
  Both rationals are in lowest terms, because 2^64 ≡ 1 (mod 3). Neither numerator
  fits a u64. The ends are FND-015.
- **FND-010.** The work-limit clause is correct (FR-038-AC-3's `incomplete`
  naming `work`). Its referent is FND-017.
- **FND-006.** The split into four ACs is adequate. AC-119 (admission) and AC-120
  (the term walk) are each one stage. AC-121 spans the graph check and the
  temporal bounds check. AC-122 spans profile fit and the operation step. Each
  groups one concern, and every IR reading is marked in the AC rows.
- **Matrix and gates.**
  - The matrix, TC-048, spec/tests.md and the FR-038 prose all use the
    AC-119..AC-122 range, with no stale single-AC reference.
  - `make spec`: grammar 376/377 (FR-014 only) and strict 23 unbacked, both
    unchanged.

Only the four low findings above remain.

Disposition pass 2 verdict:

- **FND-015, the float claim.** I verified the arithmetic. In f64, 2^64+1 and
  2^64 both round to 2^64, so a float comparison sees `(2^64+1)/3` and `2^64/3`
  as equal. With both ends `closed` it admits the refusing order, which is
  wrong, and it admits the swap, which is right. The AC's float claim is
  correct.
- **FND-015, the 64-bit claim.** Both numerators exceed u64::MAX. A checked
  64-bit parse fails, so it cannot admit the swap, as the AC says. Two caveats:
  - A wrapping parse (to 1/3 and 0/3) gets both orders right.
  - A saturating parse gets the refusing order wrong.

  So "a parse into 64 bits gets wrong" holds for a checked parse only. This
  is explanatory wording; the pinned outcomes are right, so it is not a
  finding.
- **Lowest terms as the last graph check.** This placement agrees with FR-038's
  order sentence ("... `package_id` and node-key recomputation; then the graph
  checks"). It agrees with FR-322, which leaves the order among graph checks
  unstated, and the placement is marked as an IR reading. It agrees with AC-121's
  lower-digest profile-fit example: profile fit runs in the temporal step, after
  every graph check, so `invalid_semantic_graph` still wins. The one exception is
  the frame stage's own `invalid_semantic_graph` refusals (FND-018).
- **Other fixes.** TC-048 now reads the pattern refusals under all five
  profiles. The work-limit clause names the beyond-2^64 package.
- **Nothing else moved.** The diff touches only FR-038 lines 1156-1166, the
  AC-121 row and TC-048 lines 800-814.
- **Gates.** Grammar is 376/377 (FR-014 only) and strict coverage is 23
  unbacked, both unchanged.

Disposition pass 3 verdict (agent-ix/quire-contract-ir@008eb31d68447d0b648b2ff151e49ab7854c5f7f):

- **FND-018 is fixed.** Lowest terms is now the last of the graph checks that
  run before the frame step. The frame step's own `invalid_semantic_graph`
  refusals, such as a frame array out of canonical order (FR-038-AC-14), are
  named as later and lose to the bound.
- **The order is consistent.** It agrees with FR-038-AC-14's frame-step
  precedence and with FR-038-AC-102 ("the temporal step runs after the frame and
  state-clause step"). It still agrees with AC-121's profile-fit example.
- **Nothing else moved.** The delta is FR-038 only: the placement sentence and
  the word "checked" in AC-121. PR body item 1 is updated to match.
- **Gates.** Grammar is 376/377 (FR-014 only) and strict coverage is 23
  unbacked, both unchanged.
- **Status.** All 18 findings are fixed and none is open.
