---
id: SR-1590
title: "gap analysis of PR 299 (IR-628 typed accessor for a model object type's fields)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@656fd549e65d0736993f87021b549dc4c2272b19; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Typed accessor section, FR-038-AC-136 to FR-038-AC-144), spec/checked_package/matrix/TC-227-checked-package-v2-model-object-fields-accessor.md, crates/quire-contract-model/src/checked_package/v2/model_fields.rs, crates/quire-contract-model/src/checked_package/v2/model_fields/tests.rs, crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs, tests/it/checked_package_v2_model_fields.rs"
review_set: subset
---
# SR-1590: gap analysis of PR 299

## Summary

Ticket: IR-628. Plan completion: not assessed. This run is planless. The dispatcher asked
for a test-oracle check, so every mutation row of FR-038-AC-136 to FR-038-AC-144 was
applied as a source mutant in a throwaway worktree. Each mutant was reverted with
`git checkout HEAD -- <file>`. The mutated build then ran the 24 `tc_227` tests: 12 unit
tests and 12 integration tests. Every survivor was run again against the full workspace
suite: 379 integration tests and 161 unit tests.

Traceability: `quire trace --id` gives each of AC-136 to AC-144 its own direct claims,
with 1 to 12 tests per AC. No AC is backed only through the shared TC-227.
`quire coverage --strict` drops from 47 unbacked rows to 37. The 37 that remain are the
same rows main already had unbacked, and are a separate, pre-existing failure.

Mutation results (K = killed, S = survived):

| Row | Mutant | Result | Killing test(s) |
| --- | --- | --- | --- |
| AC-136 | fields from node body | K | 9 tests |
| AC-136 | field no read names omitted | K | 8 tests |
| AC-136 | declaration (reversed) order | K | 7 tests |
| AC-136 | identity order (sort removed) | S | none, full suite included |
| AC-137 | inherited omitted | K | 15 tests |
| AC-137 | redefined base beside redefiner | K | 6 tests |
| AC-137 | base type for redefined field | K | tc_227_the_effective_set_is_the_one_admission_resolves |
| AC-137 | less derived redefiner returned (same-target rule removed) | S | none, full suite included |
| AC-138 | bounds via i64 | K | every_member_kind, returned_type |
| AC-138 | bounds via f64 | S | none (see SR-1591) |
| AC-138 | past-i128 saturated | K | every_member_kind, returned_type |
| AC-138 | Option dropped | K | every_member_kind, returned_type |
| AC-138 | Collection kind swapped | K | every_member_kind, returned_type |
| AC-138 | None field omitted | K | every_member_kind |
| AC-138 | None field refuses call | K | every_member_kind, returned_type |
| AC-139 | UnknownNode collapsed | K | names_what_is_wrong, tells_an_unknown_node |
| AC-139 | ambiguity returns first | K | ambiguous_name_refuses, pure_total_and_does_not_resolve |
| AC-139 | ambiguity returns the other fields | K | same two |
| AC-139 | absent field as error | not applicable | `field()` returns `Option`, so the mutant does not type-check |
| AC-139 | non-empty body returns fields | K | step_two_would_refuse, names_what_is_wrong |
| AC-140 | bounds from a bounded_domain body | K | does_not_read_a_node_body |
| AC-141 | optional wrapper dropped | K | (as AC-138 Option) |
| AC-141 | collection bounds dropped | K | every_member_kind, returned_type |
| AC-141 | Reference target changed | K | every_member_kind, returned_type |
| AC-142 | `#[non_exhaustive]` on CheckedMemberType | K | compile error in tests/it |
| AC-142 | variant added to CheckedCollectionKind | K | compile error in tests/it |
| AC-142 | both CheckedModelField fields public | K | compile_fail doctest |
| AC-142 | CheckedModelObjectFields field public | K | compile_fail doctest |
| AC-142 | only `CheckedModelField::name` public | S | doctest still fails to compile on the other private field |
| AC-143 | unwrap of a missing owner | K | names_what_is_wrong |
| AC-143 | recursive search | S | depth 200 (and 1500) cannot overflow |
| AC-143 | resolves per call | K (proxy) | counter test; only a proxy mutant is possible |
| AC-143 | equality gains tables | S | equivalent mutant (SR-1591) |
| AC-144 | tables built uncharged (after admission) | K | smallest_limit, long_chains, precedence |
| AC-144 | own-field/edge charge omitted | K | 4 tests |
| AC-144 | copied-entry charge omitted | K | 5 tests |
| AC-144 | tables charged after graph stage | K | tables_follow_every_step_one_and_precede_the_graph_stage |
| AC-144 | per-type ancestor re-walk | K | 5 tests |
| AC-144 | ambiguous table refuses admission | K | 5 tests |
| AC-144 | field resolution still walks | K | field_read_charges, field_entry_resolves |
| AC-144 | field resolution charges nothing | K | same two |
| AC-144 | cycle refused at admission | K | 4 tests |
| AC-144 | built per member of a cycle | K | cycle_is_charged_and_built_once_per_component |
| AC-144 | cyclic table differs from resolve | K | 3 tests |
| AC-144 | built without termination | not run | it would hang the test, so it is trivially caught |
| extra | AmbiguousField carries the largest name | S | none, full suite included |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The most-derived-redefiner rule for two redefiners of one target is re-implemented in this PR (`by_target`: an own redefiner hides every inherited redefiner of the same target), and no test can fail when it is deleted. With that block removed, all 24 `tc_227` tests and the full suite (379 + 161) pass. AC-137's mutation row "a less derived redefiner returned" survives. The cause: the AC-137 fixture redefines in a chain (Order/limit redefines Mid/limit), and none of the 9 differential documents gives a more-derived owner that redefines the same target as a less-derived one. The reviewer's document `branch_dominance` does: Base{x}; L ext Base, x redefines Base/x; M ext L, x2 redefines Base/x; X ext [M, L]; Y ext [L, M] with y. Recorded by the pre-change `resolve` at 6089cf6, it gives `X: x2=M/x2` and `Y: x2=M/x2 y=Y/y`. The mutant would also expose `x=L/x` there | crates/quire-contract-model/src/checked_package/v2/model_fields.rs:341-362, tests/it/checked_package_v2_model_fields.rs:612, crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs:620 |
| FND-002 | medium | The ascending-name order (item 20) is never tested where identity order (`<owner>/<name>`) differs from name order. Deleting `fields.sort_by(name)` in the accessor passes every test, including the full suite. Every multi-owner fixture happens to sort the same both ways: AC-137's `Base/inherited` sorts before `Order/limit`. `CheckedModelObjectFields::field` binary-searches by name, so that mutant would also make `field()` miss inherited fields. A fixture with an inherited `zeta` on `Base` and an own `alpha` on `Order` kills it | crates/quire-contract-model/src/checked_package/v2/model_fields.rs:634, crates/quire-contract-model/src/checked_package/v2/model_fields.rs:558 |
| FND-003 | low | `AmbiguousField` carries the smallest ambiguous name (a coder decision documented on the variant), but no test has two ambiguous names. Replacing `.min()` with `.max()` survives the full suite, so the documented determinism is not gated | crates/quire-contract-model/src/checked_package/v2/model_fields.rs:369-375 |
| FND-004 | low | The AC-142 struct-literal fixtures do not catch a single public field on `CheckedModelField`. With only `name` public, the `compile_fail,E0451` doctest still fails to compile, on the private `member_type`, so the row "a public field on either struct" survives for that struct. The two compile_fail doctests also carry no trace tag, so AC-142's `quire trace` claim covers the enums and the signature, not struct privacy | crates/quire-contract-model/src/checked_package/v2/model_fields.rs:512-517 |

## Verdict

FAIL. The FAIL comes from FND-001, which is high. Every AC from 136 to 144 is traced
directly by tests that pass, and 37 of the 43 applicable mutation rows are killed. One of
the 37 (resolves per call) is killed only by a proxy mutant. The survivors are these:

- FND-001: the same-target most-derived-redefiner rule, re-implemented here and untested
  anywhere in the repository.
- FND-002: identity order versus name order.
- FND-003: which ambiguous name the error carries.
- FND-004: a single public field on `CheckedModelField`.
- Three rows the spec claims can be caught but cannot: f64, equality, recursion (SR-1591).

The differential fixture is genuine: it was committed first, it was recorded by running
the old `resolve`, and the new tables match it. The coder's caveats were checked one by
one:

- AC-138's relationship-typed field cannot exist in an admitted package. True:
  `check_type_ref` refuses it as `malformed`.
- AC-140 runs on a hand-built package. Its mutant is still killed.
- The compile_fail doctests are untagged. True (FND-004).
- AC-144's frame, abstraction and relationship-edge charges are checked only at
  `resolve_member` and `resolve`. True. Those readers have no other path, so the risk is
  low; see SR-1591 for the status wording.

Semantic review was performed through the mutation runs, at the dispatcher's request.

## Coverage

- Plan completion: not assessed
- Criteria examined: FR-038-AC-136 to FR-038-AC-144, all directly traced (`quire trace
  --id`). None is untagged.
- Mutation rows: 37 of 43 applicable rows killed, one of them (resolves per call) only by
  a proxy mutant; 5 survive (identity order, same-target redefiner, f64, single public
  field, recursion), plus 1 equivalent mutant (equality). One extra mutant (largest name)
  survives.
- Reverse gaps: none found. Every new non-test item in `model_fields.rs` is owned by an
  FR-038 item.
- Stubs: none found.
