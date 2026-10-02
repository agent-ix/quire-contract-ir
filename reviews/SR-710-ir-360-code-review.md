---
id: SR-710
title: "code review of PR 245 (IR-360 numeric operand ranges)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@9f57b4518882cb6bbd0dba6c6be6988336cacc80; crates/quire-contract-model/src/expression.rs"
review_set: subset
---
# SR-710: code review of PR 245

## Summary

Ticket: IR-360. I reviewed head 9f57b45 against origin/main 4233b56, which is the merge
base. This file also holds the rust-review lane. The diff touches one file,
`crates/quire-contract-model/src/expression.rs` (+82/-31). `check_numeric` now reads each
operand's `range` along with its value type. It passes the integer or rational intervals,
already pulled out of the range, to `check_integer_operator` and `check_rational_operator`.
That removes the four `unreachable!()` let-else arms. If the shapes do not match, the code
returns the existing `IllTypedExpression` refusal ("operator requires compatible numeric
operands").

**Can any input reach the old arms?** No. I read every place a `Checked` is built (`leaf` and
`with_children`/`with_child_vec`) and the one later write to `.range` (line 1957). In each case
an Integer or Rational value type gets a range of the matching shape:

- integer and rational literals (lines 1411, 1714)
- value references, locals, field access, unwrap, length, index and call results. Each takes
  `numeric_range(&type)`, and `refine_range` keeps the shape (lines 1953, 1998, 2040, 2113,
  2162, 2235, 2307, 3306)
- operator results (lines 2512, 2712)
- negate (lines 2930, 2976), which builds the result from a `(type, range)` tuple match

Every non-numeric type gets `None`. `numeric_range` returns `None` for everything else. The
expression grammar has no if, match or conditional form that could join ranges of different
shapes. `check_numeric` refuses operands whose types differ before it dispatches, and the
coder's claim that the other readers of `.range` already match type and range together holds:
there are only two, at lines 2207 and 2891.

**Does the change preserve behaviour?** Yes:

- For every operand pair the old code accepted, the new code computes the same result. The
  operator bodies only swapped a borrow or clone for an owned value.
- The old arms panicked. The new arms refuse.
- The `_` arm for non-numeric types and for rational Remainder keeps the same code and
  message.

Measured:

- **Corpus.** `make corpus` output at the head is byte-identical (`cmp`) to the output with
  base `expression.rs`: 100 fixtures.
- **Mutation.** I replaced both new fall-through arms (lines 2379 and 2394) with `panic!`.
  The workspace tests (`--all-targets --include-ignored`) and the corpus still pass, and no
  MUTANT panic fires. The arms are never executed. That is consistent with the reachability
  analysis.
- **Gates.** `make fmt-check lint corpus test` passes at the head.
- **Coder's log.** ir-360-ci.log has `head=9f57b45... exit=2`. Only `spec` fails, with the 23
  planned unbacked rows, none in expression.rs.

**The v2/mod.rs claim holds.** The four `.expect("already validated by validate_frame_body")`
sites and `validate_frame_body` itself were removed on main by #205 (IR-89, FR-040 V2 reader)
(`git log -S`).

**Remaining panic sites.** I searched the reader for panic sites: `expect(`, `unwrap()`,
`unreachable!`, `panic!` and slice indexing, in checked_package/ and expression.rs, outside
`#[cfg(test)]`. None is reachable from untrusted input:

- `v2/lower.rs:308` is `expect` on serialising an all-string-keyed preimage. It is on the
  write path and cannot fail.
- `v2/operation_catalog.rs:171` parses the embedded build-time catalog.
- `expression.rs:2627` calls `unwrap` on the min/max of a fixed `[i128; 4]`.
- Indexing such as `arguments[position]` and `arguments[required..]` in `v2/operations.rs`
  (1045, 1052, 1075) is guarded in the same function by an arity check or by
  `position >= arguments.len()`.
- `nodes[position]` uses positions from an index built over the same `nodes`.
- `widen_ranges` reads `closed[index - 1]` only when `widened` is non-empty, so `index >= 1`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The ticket asks for a structure that cannot compile the mismatch ("restructure so it cannot compile ... Checked.range becomes a CheckedNumeric enum keyed on the value type"). The PR keeps `Checked { value_type, range: Option<NumericRange> }`, so an Integer type with a Rational or `None` range can still be represented. The new `_ => Err(operands_not_numeric(..))` arms are the old `unreachable!()` relabelled as a refusal: never executed (mutation-checked), untestable, and still resting on the same by-convention invariant over about 12 construction sites. If the invariant ever breaks, the arm blames the author's expression with a user-facing IllTypedExpression for what is an internal defect. It no longer panics, which meets NFR-003-AC-1's no-panic intent. But the PR body calls it "structural" and closes IR-360, whose stated remedy it did not take. Either do the CheckedNumeric pairing, or record the decision to stop here on IR-360 (accepted-no-change) and say in a code comment that the arms are an invariant-break guard, not an input refusal | crates/quire-contract-model/src/expression.rs:2366-2398 |
| FND-002 | low | The PR body contradicts itself. It says "Does not close IR-360 on its own judgement: the v2 half was already gone; the remaining half is this PR. Closes IR-360". Reword it to one statement (for example "Closes IR-360: the v2 half was removed by #205; this PR is the remaining half") | PR #245 body |

## Verdict

Correct and behaviour-preserving. There is no panic and no output change, and the gates pass.
Mergeable once FND-001 is dispositioned. The lead or owner either accepts the
runtime-refusal form and records that on IR-360 with a one-line code comment, or the coder
does the CheckedNumeric pairing. FND-002 is a one-line body edit. The rust-review lane found
no other smells: no new clones beyond one bounded interval-vector clone per operand, no lint
suppressions, and no new `pub` surface.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | The PR body says "Re-measured on origin/main 4233b56", an abbreviated commit id. It is also stale: IR main is now a91d5bd39be24ac85463fb007aade51c9e1fd8d8 (PR #247). Use the full id or drop it. Body edit only | PR #245 body |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | accepted-no-change | Round 1, head bc5e0b7e7d61204197b9fd1f95e739045bab6944. The finding is real and the code still does not pair them structurally: `Checked.range` is still `Option<NumericRange>`. The leader took the finding's second option, as the PR body and the commit at the head show. Commit bc5e0b7e7d61204197b9fd1f95e739045bab6944 adds comments at both fall-through arms. They say each arm guards an internal invariant break, not user input, and that the compiler does not enforce the pairing. The comments are accurate (reachability was re-checked in SR-710). The body now says plainly that the CheckedNumeric pairing is not done and that the PR does not close IR-360. Accepting this is sound only because the PR is labelled "Part of IR-360" and IR-360 stays open (In Progress), so the structural remedy is still owed there. If IR-360 closes on this merge, this outcome no longer holds. The branch is fix/ir-360-untrusted-input-panics, and Linear's branch link can move the issue to Done on merge, so check IR-360's state after merge. One part is accepted as is: a broken invariant would still surface as a user-facing IllTypedExpression |
| FND-002 | fixed | No commit (PR body edit). The body now opens "Part of IR-360 (audit IR-317)." and ends "this PR therefore does not close IR-360." It says nothing contradictory about closing |
| FND-003 | still-open | Found this round. The PR body still says "origin/main 4233b56" (abbreviated and stale). Body edit only, no code change |
