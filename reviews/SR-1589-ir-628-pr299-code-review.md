---
id: SR-1589
title: "code review of PR 299 (IR-628 typed accessor for a model object type's fields)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@656fd549e65d0736993f87021b549dc4c2272b19; crates/quire-contract-model/src/checked_package/v2/model_fields.rs, crates/quire-contract-model/src/checked_package/v2/model_fields/tests.rs, crates/quire-contract-model/src/checked_package/v2/model_members.rs, crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs, crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs, tests/it/checked_package_v2_model_fields.rs, tests/it/main.rs"
review_set: subset
---
# SR-1589: code review of PR 299

## Summary

Ticket: IR-628. This is the code review with its Rust lane (`rust-review`) folded in. It
covers `git diff origin/main...656fd549e65d0736993f87021b549dc4c2272b19`: two commits,
13 files, +3096/-59. The base is main at a71ae81539847a92d1e26426491d6cae46b9b33b, which
is the merged IR-628 spec, #296. The review checked the code against FR-038's "Typed
accessor for a model object type's fields" section (items 1 to 37), and re-measured every
claim in the PR text and in the coder's report.

What was verified, and how:

- **History order.** 6089cf6 (10:07) is the parent of 656fd54 (10:27). 6089cf6 adds only
  the differential test and its 26 recorded lines to `model_members/tests.rs`. The test
  passes at 6089cf6, where `DomainModel::resolve` still has the pre-change walk, so those
  lines are what the old `resolve` answers. 656fd54 moves the lines into the const
  `RECORDED` unchanged, and checks both the new `resolve` and the tables against it.
- **Independent differential.** The reviewer wrote 7 more documents and ran them through
  the old `resolve` at 6089cf6 and the new one at 656fd54. The documents cover
  same-target redefiners in a more derived branch, two own redefiners of one target, a
  cycle that reaches a cycle, a cycle with an outside redefiner, a redefinition of an
  unrelated target, a redefinition of an own field, and a diamond with a direct base edge.
  All 28 resolutions were byte-identical.
- **Cost formula.** Recomputed by hand: N(N+1)/2 + N - 1 gives 13 (N=4), 43 (N=8),
  501,499 (N=1000) and 1,127,249 (N=1500). A self-cycle costs 2 and a two-cycle costs 4.
  `build_component` charges own fields plus declared edges, then each outside supertype's
  table entries, before copying them. The unit and integration tests measure the same
  numbers, and they pass.
- **Stage placement.** `build_field_tables` runs after `validate_domain_packages`,
  `admit_dependencies` and the definition-ref checks, as the last act of
  `validate_lock` (mod.rs:916), and before `validate_graph`.
- **Accessor.** `UnknownNode` comes from the node search, and `NotModelObjectType` from
  the index lookup through `check_declaration_node` and the form check.
  `check_declaration_node` is now the single rule, shared with `ModelOwners::recover`.
  Fields are sorted by name, and `None` passes through.
- **Visibility and closure.** The three public enums have no `non_exhaustive`, and the
  struct fields are private. The five types are re-exported and listed in FR-019's V2
  reader row. The accessor charges nothing.
- **Rust lane.** `model_fields.rs` has no `unwrap`, `expect`, `panic`, indexing or
  `unsafe` outside tests. Every count goes through `saturating_add` or `units()`. The
  Tarjan search is iterative. Memory is bounded by work charged: every entry copied into
  `inherited` or `hidden` is charged first. `fmt`, `clippy -D warnings`, `deny`, `audit`
  and `audit-unsafe` all pass. The diff has no `.github` edits, no build artifacts and no
  compatibility layer.
- **`read` helper.** The edit in `model_members/tests.rs` builds the tables under an
  unlimited meter after `read_semantic_ir`. A field `resolve` needs the tables now, so the
  edit is required. The tests that measure charges (`tc_048_reading_and_resolving_...`)
  measure `resolve` alone, relative to its own consumption, so the helper does not weaken
  any assertion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The comment in `element_type` says `ModelOwners::new` derives the same key earlier and refuses the read, "so no refusal reaches this point". That is now false. `build_field_tables` calls `field_type`, and so `declaration_key`, in the lock stage, before `ModelOwners::new` runs in the graph stage. A key past the byte limit gives `None` silently there, and admission refuses later. The behaviour is correct, but the comment states an ordering that no longer holds | crates/quire-contract-model/src/checked_package/v2/model_members.rs:814 |

## Verdict

The code is correct as far as this review could measure. Tests re-run by the reviewer
pass, and an independent differential of 28 resolutions found no divergence between
the old `resolve` and the new one. The cost numbers, the stage placement and the accessor
semantics match the spec. There is one low finding, a stale comment. The test-oracle
gaps, including one high, are recorded in SR-1590 (gap analysis), and the status and
mutation-row defects of the spec in SR-1591.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 65622b8230443d661fa00aa55c2f02169b3fba3e |
