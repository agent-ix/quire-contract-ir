---
id: SR-625
title: "gap analysis of PR 186 (reader depth classification)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@cfb1b713e5048e036b762bb297a795657758fb95; crates/quire-contract-model/src/checked_package/common.rs, crates/quire-contract-model/src/checked_package/shared.rs, tests/it/checked_package_v2_reader.rs, spec/contract/FR-038-consume-checked-package-v2.md"
review_set: subset
---
# SR-625: gap analysis of PR 186

## Summary

Ticket: IR-279. This is a manual acceptance-criteria-to-tests check of the PR diff, scoped to the depth behaviour. FR-322 was read from agent-ix/quire-specification origin/main 4634f5f; the local checkout is stale.

Examined:

- **FR-038-AC-3.** Exact limits admit and a one-over limit is incomplete. Covered by `tc_048_nesting_past_the_limit_is_incomplete_after_syntax_and_members_pass` (tests/it/checked_package_v2_reader.rs:2618), and exact admission by the existing test at :667.
- **FR-038-AC-26.** The depth charge is located at the first value one level past the limit. Covered by `tc_048_depth_is_charged_at_the_first_value_past_the_limit` (common.rs), which checks depths 0 to 3 and whose mutant was killed.
- **FR-322-AC-4.** Malformed and duplicate members refuse. Deep syntax and duplicate refusals are covered, and the mutant that charges depth first was killed.
- **FR-322-AC-6 and FR-322-AC-9.** Incomplete for a one-over depth, and malformed stays distinguishable from incomplete. At the default limit the PR fixes the misclassification IR-279 reports.

Context only: FR-322 lines 319, 345-350 and 652.

Bindings checked: `tc_048_nesting_past_the_limit_is_incomplete_after_syntax_and_members_pass` to FR-038-AC-3 is correct. `tc_048_depth_is_charged_at_the_first_value_past_the_limit` to FR-038-AC-26 is correct.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038-AC-3 and FR-322-AC-6 are unmet for any caller depth limit above 128. With `depth: 200`, a 200-deep document is refused as incomplete instead of admitted, and a one-over document reports `limit: 128`, not the caller's limit. The new test asserts that deviation (`depth: 300` expects `incomplete(Depth,128,200)`). The ticket's stated problem, a raised QSL `V2ReadLimits.depth` having no effect above 128, is reclassified but not fixed | crates/quire-contract-model/src/checked_package/common.rs:238 |

## Finding Detail

- FND-001: FR-322 line 319 says "`read` receives limits from its caller", line 652 says "Exact selected resource limits are admitted", and AC-6 says "Exact selected limits admit the boundary vector". FR-038-AC-3 says "Exact byte, depth, ... limits admit a package; each one-over limit returns `incomplete` with that limit kind, the limit and the consumed counter". The clamp `limits.depth.min(MAXIMUM_DEPTH)` breaks both halves once a caller selects more than 128. Mutation M1 (removing the clamp) aborts the process with a stack overflow on the 100,000-deep test, because the `Value` pass, `serde_json::to_vec`, admission and `Value` drop all recurse on the caller's stack. So the clamp is a sound safety guard: it is documented and not hidden, but it contradicts the requirement. There are two ways to resolve it. (a) Honour the caller's limit by running the post-shape pipeline and the drop of the value on a grown stack, following the binding.rs:204 pattern. This comes after SR-624 FND-001 and FND-002 are fixed, and needs the byte limit to bound the growth honestly. (b) Amend FR-322 upstream to permit a reader-declared maximum depth, with the reported `limit` being the effective one, and then amend FR-038-AC-3 to match. Changing only the local FR-038 prose, as this PR does, is not enough (see SR-626).

## Verdict

At the default limit the PR delivers what the ACs ask: 200-deep input is `incomplete(Depth,128,200)` at the correct pointer, and syntax and duplicate defects refuse first however deep they sit. The tests are strong, killing 7 of 7 mutants. For caller limits above 128 the PR swaps one spec violation (malformed_wire) for another (the selected limit is not honoured). Not mergeable until FND-001 is resolved by (a) or (b) with an owner decision. A downstream note, not a defect here: quire-spec-language `qsl-package/src/checked_v2.rs` pins the old behaviour in `depth_far_past_the_default_limit_is_refused_as_malformed_wire`, so it needs a follow-up when QSL takes this IR revision.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 7f1d923 |

- FND-001: The owner chose option (a). `MAXIMUM_DEPTH` and the clamp are removed. The shape pass and the `incomplete` report use `limits.depth` (common.rs:242-249). `tc_048_a_document_at_the_callers_limit_is_admitted` admits the exact limit and reports the caller's limit one over, at limits 3, 128, 129, 200 and 1,000. The integration test reads a 200-deep document under 200, 300 and u64::MAX. Mutant M1, which restores the 128 clamp, is killed. The deep admitted package's lowering is a separate hazard, raised as SR-624 FND-006.
