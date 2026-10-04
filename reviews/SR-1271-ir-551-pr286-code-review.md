---
id: SR-1271
title: "code review of PR 286 (IR-551 timed/v1 rational interval bounds)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@65a034d7b14d64c150646ebef77f8403b9e3257d; crates/quire-contract-model/src/checked_package/v2/mod.rs, crates/quire-contract-model/src/checked_package/v2/natural.rs, crates/quire-contract-model/src/checked_package/v2/temporal.rs, crates/quire-contract-model/src/checked_package/v2/vocabulary.rs, crates/quire-contract-model/src/checked_package/common.rs (context), tests/it/checked_package_v2_temporal.rs"
review_set: subset
---
# SR-1271: code review of PR 286

## Summary

Ticket: IR-551. Code review with the rust-review lane folded in, over
`git diff origin/main...HEAD` at 65a034d7b14d64c150646ebef77f8403b9e3257d (merge base
4762ce25ff7f8098fe6b3e4db3a0bfc15dacaa23, which is current `main`; one commit).
`gh pr view 286`: MERGEABLE, mergeStateStatus BLOCKED (reviewDecision REVIEW_REQUIRED;
the CLA checks pass).

What was checked, and what holds:

- **Pattern stage.** `interval_bound_outside_pattern` picks the pattern by member set
  through `IntervalForm::of`. Exactly the four timed members use `Rational::read`: a closed
  two-member object, `numerator` an `IntegerString` that is not negative, and `denominator`
  not negative and not zero. Taken together that is `^(0|[1-9][0-9]*)$` over
  `^[1-9][0-9]*$`, and `"-0"` and `"01"` are refused by `IntegerString::parse`. Every
  other member set keeps the integer pattern, and its `upper: null` exemption applies
  only outside the timed form. Every AC-120 example refuses at the right bound: `-1`, `01`,
  denominators `0` and `-2`, a JSON integer, the string `"3"`, a `null` upper with four
  members, and the two-failure case at `lower`. The check is still called from the
  existing term walk in `common.rs:798`, which runs per application term in position
  order. The PR adds no recursion: the new code is flat loops over nodes and limbs.
- **Lowest terms.** `validate_timed_bounds_reduced` iterates `index.values()`, which is
  ascending `node_id` digest order (probe: with two non-reduced nodes it reported the
  lower-digest one, at position 23 ahead of position 18). It checks `lower` and then
  `upper`, charges the GCD at `/semantic_graph/nodes/{n}/body` and refuses
  `invalid_semantic_graph` at `.../interval/lower|upper`, with the node's key as locus. It
  runs after the structural, application-key and dependency-join, nominal and
  declaration-name checks, and before the frame, state, temporal, abstraction and
  operation steps. Moving it after `validate_temporal` makes
  `tc_048_a_bound_not_in_lowest_terms...` fail at the fit-defect case (a mutation run),
  so the ordering is pinned. See FND-002 for one wording gap.
- **Profile fit.** `IntervalFit::{ClosedOnly, NullOrTimed, IntegerForms}` with the `match`
  in `interval_fits` written out with no wildcard. Under `timed/v1` a `null` or timed
  interval admits and `[a,*]` or an integer `[a,b]` is a mismatch, as AC-104 and merged
  FR-370-AC-3 say. Under infinite-trace the timed form is a mismatch and the three
  integer shapes admit. Under the bounded profiles only an integer `[a,b]` admits.
  `IntervalEnd` is a `closed_vocabulary!` with `closed`/`open`.
- **Bounds.** `check_bounds` replaces `bounds_defect`. For the timed form,
  `Ordering::Greater` refuses, and `Equal` refuses when either end is `open`. `[3,3]`,
  and `[0,0]` in a probe, admit, while `(0,0]` refuses `invalid-value` at the body.
  `compare_rationals` parses the four magnitudes and cross-multiplies.
  `Natural::multiply` is correct schoolbook arithmetic. Each row's final carry goes to
  slot `i + len(other)`, which no earlier row wrote. The bound `(2^32-1)^2 + 2(2^32-1) =
  2^64-1` stated in the comment holds. A zero operand (an empty limb vector) gives an
  empty product. `Ord for Natural` compares by limb count and then from the most
  significant limb, and is sound because both `parse` and `multiply` normalize. The work
  `work(a)*work(b)` is charged before the `len(a)+len(b)` allocation, and the parse cost
  that bounds those lengths was charged first, so no cost goes unmetered. The PR adds no
  `unwrap`, `expect`, `as` cast or unchecked index: `get`/`get_mut` with `saturating_add`
  and `low32` through `try_from`. The `graph.nodes[position]` index in `check_bounds` was
  already there in `bounds_defect`.
- **Spec point (b).** A four-member interval with a bad end and a non-reduced bound (probe:
  `{2/4, 3, half, closed}`) refuses `invalid_semantic_graph` at `lower`. With a bad end and
  `lower > upper`, the temporal step skips it and it refuses `operation-member-mismatch`
  at `operation.member`. Both follow the FR-038 stage list: the timed form is decided by
  member set alone, stage 2 comes before stage 5, and the temporal step skips an interval
  it cannot read. AC-122 constrains only the valid-bounds case, so this is consistent and
  is not a finding.
- **Gates**, run in this review's own worktree and target dir: `make fmt-check`,
  `make lint` (both lanes), `make test` (149 + 358 + 7 plus doctests, all pass),
  `make corpus` (107 match), `make deny` (with one-copy), `make cargo-audit` and
  `make audit-unsafe` all pass. `make conformance-qspec` against a fresh worktree of
  quire-specification origin/main 2f846f8 passes 6/6.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The timed bound-pattern refusal runs after the `package_id` recomputation, not "before every identity check" as FR-038 stage 1 and AC-120 state, and as merged QSpec FR-322 orders wire shape ahead of the `package_id` recomputation. Probe: a timed `lower` numerator `"-1"` in a package whose `package_id.digest` is stale refuses `stale_dependency` at `/package_id/digest`, not `invalid-value` at the bound. The integer form behaves the same, which predates this PR (IR-549), but this PR marks AC-120's stage clause implemented. Either move the pattern check into `flat_wire::check`, which runs before `package_id` (as AC-116 does for the body grammar), or narrow the spec to "before the node-key checks, after the `package_id` recomputation" as a marked IR reading. Add a stale-`package_id` case either way | crates/quire-contract-model/src/checked_package/v2/mod.rs:707-726; crates/quire-contract-model/src/checked_package/common.rs:798-811 |
| FND-002 | low | FR-038 stage 2 makes lowest terms "the last of the graph checks that run before the frame step". The code runs it before `ModelOwners::new`, which also runs before the frame step and can refuse `invalid_semantic_graph` at `/lock/model_selections/{i}` when a model declaration key's preimage is past the byte limit. A package with both defects therefore refuses at the bound, while a literal reading of the spec says it should refuse at the selection. The coder's reading (that this is model selection, not a node check) is defensible, because the spec qualifies the list as "check[s] of the nodes", and that refusal is probably unreachable from an admitted lock. State it in the stage-2 text ("before the model declaration key derivation") or move the call after `ModelOwners::new` | crates/quire-contract-model/src/checked_package/v2/mod.rs:1738-1747 |

## Verdict

Approve with changes. The arithmetic, the work metering, the member-set form
discrimination, the profile-fit table and the refusal pointers are correct, and the PR
introduces no panic or overflow surface. FND-001 is a real stage-ordering divergence
between the code and the stage clause of AC-120, which this PR marks implemented. It is
an edge case (it needs a stale `package_id` together with a bad bound), and the fix is
either to move the check or to amend the wording. FND-002 is a spec-wording nit.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | Two places still put the bound-pattern refusal in the term walk. FR-038 "The temporal step" says "Strict wire validation (here the term walk's negative-bound refusal) precedes the step". The doc comment on `interval_bound_outside_pattern` says "The term walk refuses it `invalid-value` at that bound, in strict wire validation (`flat_wire`)". Both should name the flat wire pass | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1297-1298; crates/quire-contract-model/src/checked_package/v2/temporal.rs:315 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | Four more comments still say the bound-pattern refusal belongs to the term walk, which has not been true since 4ce54d1. My round-1 grep missed them. They are the `with_node_locus` doc ("a bound outside the interval pattern (`invalid_package`/`invalid-value`, the term walk's)") and three test comments: "Negative bounds are refused in the term walk", "`invalid-value` at the bound in the term walk" and "A negative bound is refused in the term walk of the body". These are comments only, with no behaviour change. Reword them to strict wire validation / the flat wire pass | crates/quire-contract-model/src/checked_package/v2/mod.rs:1509-1511; tests/it/checked_package_v2_temporal.rs:588; tests/it/checked_package_v2_temporal.rs:1546; tests/it/checked_package_v2_temporal.rs:1574 |

## New findings (disposition pass 3)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | This predates the PR: it came in with #285 and is on main at 4762ce2. Three places still say a nested `case` application is "refused in the term walk": the FR-038 Path row, the FR-038 temporal-step prose ("the nested case is refused in the term walk of the body, where nested applications are refused") and a test comment. Since #285 the walk that refuses it is `flat_wire::nested_application_refusal` (`Case => operator_ineligible`), which is strict wire validation ahead of every identity check (AC-116). `common.rs` `validate_term` no longer handles `case` at all. The code, refusal and pointer the spec states are correct; only the stage name is stale. Reword it to strict wire validation (the flat wire pass) | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1485; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1554-1555; tests/it/checked_package_v2_temporal.rs:959 |

## Dispositions

Round 1, reviewed at 4ce54d157870be2ae3acc00bf5d2e731db184fd8.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 4ce54d1. The pattern check (one function covering the timed and integer forms) moved out of the term walk into `flat_wire::check_interval_bounds`. It runs at the end of `flat_wire::check`, after every nested-application refusal in nodes and then details, and before the `package_id` recomputation. Reproduced: a timed `-1` with only `package_id` stale, and with every node id and `package_id` stale, refuses `invalid-value` at `.../interval/lower`; an integer `-1` with `package_id` stale does the same; good bounds refuse `stale_dependency` at `/package_id/digest`. Coverage is unchanged: every nested application, and every application in `details`, is refused earlier in the same pass, so only body-root applications ever reached the term-walk check. Running "nested applications first, bound pattern after" is consistent with FR-038 "The flat wire", AC-114..116 (still `malformed_wire` before identity), AC-120 and QSpec FR-370 ("during strict wire validation"), because both are strict wire validation and FR-038 now marks their relative order as an IR reading. Mutation: moving the call after `package_id` fails the new test |
| FND-002 | fixed | 4ce54d1. FR-038 stage 2 and AC-121 now say lowest terms runs before the model-selection owners step, which derives each selected model declaration's node key and can refuse `invalid_semantic_graph` at `/lock/model_selections/{i}`. This is marked as an IR reading, and the spec says a package with both defects refuses at the bound. That matches the code |

Round 2, reviewed at 43646a2b3fca1c475de55718d5307217840a6e57.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-003 | fixed | 43646a2. FR-038 "The temporal step" now reads "Strict wire validation (here the interval bound-pattern refusal, `flat_wire::check_interval_bounds`) precedes the step". The `interval_bound_outside_pattern` doc now reads "Strict wire validation (`flat_wire::check_interval_bounds`) refuses it". Both are true of the code. The other instances of the same wording are FND-004 |

Round 3, reviewed at 0a8ee5c09883d24ad9d81065632342aad132566c.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-004 | fixed | 0a8ee5c. The `with_node_locus` doc now names `flat_wire::check_interval_bounds`, and the three test comments say strict wire validation (`flat_wire::check_interval_bounds`). The flat_wire doc now reads "an IR reading". A wide grep of spec/, crates/, src/ and tests/ for "term walk", "term-walk" and "per-term walk" finds no remaining description of the bound-pattern check as a term-walk check. The remaining hits are the term grammar walk itself, historical placement prose, and the nested-`case` wording (FND-005) |


Round 4, reviewed at f7b734fbd1cfabca5266ec77d55579354a017704.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-005 | fixed | f7b734f. The FR-038 Path row now says "refused in strict wire validation, `flat_wire`". The temporal-step prose now says "refused in strict wire validation (`flat_wire`), where nested applications are refused, ahead of every identity check (FR-038-AC-116)", and the test comment says "refused in strict wire validation". All three are true: `flat_wire::nested_application_refusal` maps `Case` to `ill_typed`/`operator-ineligible` at the nested `operator`, and `flat_wire::check` applies `with_node_locus`, so the locus is the holder's key. A wide grep finds no remaining term-walk statement about nested applications or `case` |
