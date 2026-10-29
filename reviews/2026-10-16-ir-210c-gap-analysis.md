---
id: SR-12071
title: "IR-210 slice C gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; the 10 changed files under crates/quire-contract-model/src, spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md, spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md, spec/output_mapping/functional/FR-034-assemble-output-package-atomically.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-040
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-346
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

## Summary

Ticket: IR-210 (slice C), PR quire-contract-ir#431. This is a planless gap analysis, so plan completion was not assessed. The matrix was measured with quire 0.36.2 (engine 0.50.2) twice: on the merge in a detached scratch worktree, and on the PR head. origin/main adds only slice D files, which do not overlap this diff.

**`quire coverage --strict`**

- **Base:** 508/565 rows backed, 43 unbacked, 0 status lies.
- **Head:** 508/565 rows backed, 43 unbacked, 0 status lies.
- The human output is byte-identical between base and head.

**`quire coverage --json`, compared field by field**

- **Minted targets.** 508/565 are backed on both, and no target changed its `backed` value. That covers the StR VC rows, so their loss would show here even though the strict gate would not show it.
- **Groups.** All 50 groups are identical.
- **Unbacked list and binding census.** Both are identical: 871 candidates, 868 tagged, 867 bound.
- **`unmatched_tags`.** It loses 29 (id, symbol) pairs and gains none. These are the old `/// Tracing:` doc-line ids, which quire reported as `evidence_near_miss` and did not bind.

**Per-criterion binders**

No criterion lost a binder. Fourteen criteria gained one or more:

| Criterion | Binders: base to head |
| --- | --- |
| FR-038-AC-2 | 6 to 9 |
| FR-038-AC-3 | 2 to 5 |
| FR-038-AC-12 | 1 to 2 |
| FR-038-AC-22 | 3 to 4 |
| FR-038-AC-23 | 2 to 3 |
| FR-038-AC-24 | 4 to 6 |
| FR-038-AC-65 | 3 to 4 |
| FR-038-AC-79 | 4 to 5 |
| FR-038-AC-114 | 9 to 10 |
| FR-038-AC-117 | 10 to 15 |
| FR-038-AC-121 | 4 to 6 |
| FR-040-AC-2 | 1 to 2 |
| FR-346-AC-2 | 7 to 8 |
| FR-346-AC-6 | 1 to 3 |

**`quire trace --id`, verifies claims, base to head**

| Id | Base | Head |
| --- | --- | --- |
| FR-038-AC-2 | 6 | 9 |
| FR-038-AC-3 | 2 | 5 |
| FR-038-AC-5 | 5 | 5 |
| FR-038-AC-24 | 4 | 6 |
| FR-038-AC-79 | 4 | 5 |
| FR-038-AC-121 | 4 | 6 |
| FR-346-AC-6 | 1 | 3 |
| FR-040-AC-1 | 1 | 1 |
| FR-034-AC-3 | 3 | 3 |

On head, every near-miss citation from these files is gone. The 2 remaining FR-038-AC-79 citations are prose mentions in `tests/it`, outside this diff.

**`quire trace --file` over all 10 files on head**

- It reports 88 `rust-trace-attribute` verifies pairs and 2 `rust-comment-id` pairs. The comment-id pairs are FR-346 from the doc prose of `tc_225_exactly_one_kind_has_an_abstraction_relation_body`, and FR-012-AC-6 on an untouched TC-043 test.
- Every id of every one of the 28 tests is `rust-trace-attribute`.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-2 | examined | The reader refuses malformed, duplicate-member, unknown-member, noncanonical, ... inputs with exactly those codes ... before exposing a package; ... |
| FR-038-AC-3 | examined | Exact byte, node, edge, occurrence, diagnostic and work limits admit a package; each one-over limit returns `incomplete` with that limit kind, the limit and the consumed counter and no package; no limit kind is a depth (FR-038-AC-117). |
| FR-038-AC-5 | examined | Every nominal preimage of the in-repo nominal fixture re-derives the node key it is keyed by ...; every authored invalid nominal mutation ... refuses as `invalid_semantic_graph`; ... |
| FR-038-AC-24 | examined | Every refusal about a value carries the RFC 6901 pointer of that value, built from the reader's own position ... |
| FR-038-AC-79 | examined | A package document holding the integer 9007199254740993 in a node body refuses `noncanonical_wire` with no pointer, as does ... the float `2.0` in a body ... |
| FR-038-AC-117 | examined | `CheckedPackageReadLimits` has no depth member ...; ... past the strict parse's recursion limit of 128 ... refuses `malformed_wire` with no pointer ... and is never `incomplete` ... |
| FR-038-AC-121 | examined | Under `quire.temporal.timed/v1`, `lower` `{numerator: "2", denominator: "4"}` refuses `invalid_semantic_graph` ...; ... read with a work limit that the GCD or cross-multiplication of its bounds takes past returns `incomplete` naming the `work` limit ... |
| FR-038-AC-12, AC-22, AC-23, AC-65, AC-114 | examined | Kept or added bindings; see SR-12070 for each body-to-text judgement. |
| FR-040-AC-1 | examined | ... a bare node key, an entry of another `kind`, a `field` entry without `name`, a `relationship` entry carrying `name`, and an entry with an extra member each refuse as `invalid_semantic_graph` ... |
| FR-040-AC-2 | examined | A relationship entry naming an object type, a field entry declared on a process, a relationship or a systems node ... refuse as `invalid_model_binding`/`malformed-declaration` ... |
| FR-346-AC-2, FR-346-AC-6 | examined | Abstraction-relation body shape and Rust-spelling refusals (see SR-12070). |
| FR-034-AC-3 | examined | Every zero, exact, just-over, and overflowing aggregate limit is classified without partial output or a smaller successful package. |
| FR-040 Behavior, "`name` is the field's identifier" | context_only | FR-040 lines 77-80: the identifier rule for a `modifies` field entry's `name`. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

CLEAN. No gap is introduced, no row's backing got worse, and every dropped claim's row keeps other binders. Mergeable on this method.

**Dropped claims, with the backers that remain on head**

- **FR-038-AC-3** still has 5 binders: the 3 `natural.rs` work tests and 2 integration tests.
- **FR-038-AC-5** has 5 integration binders. Its count is unchanged, because the dropped `identity.rs` and `natural.rs` ids were never bound.
- **FR-038-AC-2** has 9 binders.
- **FR-040-AC-1** keeps `tc_056_modifies_entries_admit_in_their_closed_shape_only`.
- **FR-034-AC-3** keeps 3 integration binders.
- **NFR-060** is an external quire-specification id, not a target here.

Because each dropped id was only an unmatched near-miss on base, dropping it removes no backing.

**Notes, not findings**

- **Under-claims.** `tc_048_a_syntax_or_duplicate_defect_is_found_before_the_recursion_limit` also asserts FR-038-AC-24's "a repeated member at that member" (`/a`). `tc_048_comparison_work_is_charged_and_stops_at_the_limit` asserts AC-121's work-limit leg at unit level. Neither is claimed. Adding them is optional.
- **Spec gap (pre-existing).** No numbered FR-040 criterion states that a `modifies` field entry's `name` (or a `deletes_scoped` parameter `name`) must be an identifier and refuses otherwise. Only the Behavior prose says so, which is why `tc_056_field_names_are_identifiers` is honestly TC-056 only. A criterion for it is spec work outside this PR.
- **Spec gap (pre-existing).** No numbered criterion states the strict first pass's linear-time bound, which `tc_048_a_deep_syntax_error_is_refused_in_linear_time` tests. The same applies to the V2 literal-number predicate in `tc_048_v2_literal_numbers_are_integers`.
- **Out of slice.** Two non-`tc_` tests in `v2/mod.rs` are still untagged: `dependency_selection_required_members_match_the_type` and `exactly_one_kind_has_a_frame_body`. The second is the frame analogue of the TC-225 test that this PR binds to FR-346-AC-2.
- **Pre-existing doc lines.** Several tests that already carry `#[trace]` keep `/// Tracing:` doc lines, at `operation_catalog.rs:260,306,353` and `output_mapping.rs:2919-3251`. They are outside this diff.
