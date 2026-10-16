---
id: SR-11835
title: "Code review (with Rust lane) of IR-824 reaches over finite multi-valued ends (PR #422)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/operations.rs (check_relationship_member), tests/it/checked_package_v2_model_members.rs, spec/checked_package/functional/FR-038-consume-checked-package-v2.md (AC-252..257 status rows), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md (item 11 status), spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

Ticket: IR-824. PR agent-ix/quire-contract-ir#422 at 3a79b1f, diff
`origin/main...HEAD`. Independent code review with the rust-review lane folded
in. Checked against FR-038-AC-252..257, the FR-038 reaches paragraph, TC-048
item 11 and merged QSpec (origin/main) FR-322-AC-57, FR-152 "Reachability over
a relationship end", TC-281 RN-18 and RN-20..27, TC-197 Y18.

Code, examined and clean: `check_relationship_member` now checks operand 0
and then, for `reaches`, operand 1 against the RECEIVER endpoint type
(`conforms_to(.., &receiver.type_ref, ..)`, subtype admitted through
ancestors) before direction (operations.rs:2226-2244); multiplicity is decided
only by the existing `RelationshipDecl::navigation_type`
(model_members.rs:252, not duplicated), which refuses `unbounded` and
`ordered` as `unsupported_construct`/`expression-form` at `member.name`; the
homogeneity (type) check follows it (operations.rs:2265); the expected result
stays `MemberType::Boolean`. The `Reference | Option` restriction is removed,
so finite Set/Bag shaped ends admit unique or not. No new refusal
vocabulary, no wildcard, no `unwrap`/`expect`/panic in production code; the
`navigate` path is unchanged (operand 1 block is `reaches`-only, and
`navigation_type` was not edited). The node shape is unchanged; nothing
multiplicity- or charge-specific is added. Single-defect outcomes are
unchanged (heterogeneous end with two `Order` operands still refuses at
`member.name`).

Tests: the only changed pre-existing assertion is the Set/Bag refusal in
`tc_048_a_selected_self_relationship_reaches_through_either_named_end`, now an
admission; its other cases (both roles, subtype pair, unrelated operand 0/1)
and every `navigate` test remain. Focused runs at head: 45/45
`checked_package_v2_model_members`, 2/2 catalogue differential, 9/9 crate
`relationship` unit tests, `cargo fmt --check` and clippy `-D warnings` on
both crates clean.

Mutation oracle (throwaway worktree, removed): each went red —
Reference|Option restriction left in (4 tests); operand-1 check moved after
direction/multiplicity/type (AC-253 test); operand 1 compared with destination
type (AC-253 and AC-254..256 tests); type before multiplicity (order test);
direction after multiplicity (order test); `unbounded` admitted (order test);
`ordered` admitted (order test); reader requiring a multiplicity member on the
node (4 tests). One extra mutation survived; see gap-analysis SR-11836
FND-001 (AC-257). Status rows AC-252..257 now read Implemented and are backed;
the AC-172 row is unchanged and keeps its AC-252 supersession pointer.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The AC-252 test's node-shape assertions (`operator`, member kind, operand count, `body.get("multiplicity").is_none()`, `body.get("charge").is_none()`, body equal to the baseline) inspect the body the test fixture itself authored, which no multiplicity input changes; only the admission assertion exercises the reader. The Y18 claim is backed by admission of an identical body, not by these asserts. | tests/it/checked_package_v2_model_members.rs:1392-1409 |
| FND-002 | low | AC-253's wrong-direction multi-defect case traverses `linkedBy`, whose destination is the `[0,1]` source end, so it is not a wrong-direction multi-valued end as AC-253 frames the case; the operand-1-first order is still pinned (it went red when the operand-1 check was moved). | tests/it/checked_package_v2_model_members.rs:1428-1437 |

## Verdict

PASS with two low test-clarity findings. The production change is correct
against FR-322-AC-57 order, RN-18 and RN-20..27 and Y18; no blocking defect.
