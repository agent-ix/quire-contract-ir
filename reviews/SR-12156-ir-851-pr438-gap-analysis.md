---
id: SR-12156
title: "IR-851 gap analysis of IR PR #438"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@9151014918ad8a70ef19a48cf587a621051a7342; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Typed relationship end of a reaches application, AC-262..268), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, spec/model/functional/FR-019-rust-library-interface.md, crates/quire-contract-model/src/checked_package/v2/{relationship_end.rs,model_fields.rs,model_members.rs,operations.rs}, tests/it/checked_package_v2_model_members.rs, tests/it/checked_package_v2_model_fields.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

# IR-851 gap analysis of IR PR #438

## Summary

Ticket: IR-851. PR quire-contract-ir#438 is at 9151014918ad8a70ef19a48cf587a621051a7342. This is a planless gap analysis, so plan completion was not assessed. The authority is FR-038 "Typed relationship end of a reaches application", AC-262..268 and the TC-048 section, all as merged in spec PR #435 (merge base d1f2f83).

**Computed traceability at head** (quire 0.36.2, engine 0.50.2):

- `quire trace --id FR-038-AC-262` through `-268` each resolve to `[rust-trace-attribute]` claims and no citations:
  - AC-262: the in-crate test and the tests/it test.
  - AC-263: the tests/it test.
  - AC-264: the in-crate test and the tests/it test.
  - AC-265: the in-crate test.
  - AC-266, AC-267 and AC-268: one tests/it test each.
- No fully-qualified `#[ix_trace_rs::trace]` spelling was added.
- `quire coverage --strict` reports 43 unbacked and 0 contradicted. `quire coverage --json` totals are backed 515 of 572.
- `quire matrix --strict` reports 54 untagged and 1 tagged only by ignored tests. None of AC-262..268 is untagged.
- The PLANNED to Implemented flips in FR-038, TC-048 and tests.md cover exactly AC-262..268, and each of them is traced.

**Mutants.** Hand mutants were run in a throwaway worktree with its own target dir, serialized by the heavy-build lock at `-j2`. Each was run against the model lib `relationship_end` tests and `it -- checked_package_v2_model_members`. The rows marked "full" were re-run against the full model lib and the full `it` suite. The unmutated control passes.

| Mutant | Result |
| --- | --- |
| ends swapped (destination := receiver end) | killed (2 in-crate + AC-262 + AC-263) |
| destination key from the wrong end | killed in-crate only; equivalent on admitted packages (reaches is homogeneous, AC-256) |
| role from the wrong end | killed (in-crate, AC-262, AC-263) |
| lower +1 / lower -1 / upper -1 / upper := None | killed (in-crate, AC-262, AC-263) |
| unique flipped / ordered flipped / forward flipped | killed (in-crate, AC-262, AC-263) |
| relationship := application id | killed |
| member kind check removed | killed (AC-265 unit) |
| both roles resolve forward / neither role resolves inverse | killed (AC-265 unit) |
| missing operation.member maps to NotReachesApplication | **survived** (FND-002) |
| non-text name / unresolved declaration maps to NotReachesApplication | killed |
| missing retained key returns an empty digest | killed (AC-265 unit) |
| reaches identity check removed / navigate accepted as reaches | killed (AC-264 tests) |
| unknown id maps to NotReachesApplication | killed (3 tests) |
| node-id filter in node_at removed | survived; equivalent (the index is built from the same wire) |
| RetainedModels::new swaps source_key/target_key | survived; equivalent on admitted reaches (homogeneous ends) |
| RetainedModels::relationship drops the Relationship form check | survived (FND-001) |
| RetainedModels::relationship drops check_declaration_node (full) | survived (FND-001) |
| retention empty | killed |
| retention in package equality | killed (in-crate equality assertion) |
| retention charges 1 work per relationship | killed (AC-266 work boundary, plus an existing navigation work test) |
| end_declaration_key always ObjectType form (full) | survived; pre-existing gap, see Verdict |
| refactor: None to key_refused (full) | survived; pre-existing gap, see Verdict |
| refactor: Err to ineligible (full) | survived; pre-existing gap, see Verdict |

**Base checks on the merge base d1f2f83, in the same throwaway worktree.**

- *Work baseline.* A probe test copied the PR's AC-266 binary search over the same package: the `{1,1}`/`{2,3}`-unique-`peer` relationship with a reaches application naming `links`. It measured a least admitting work limit of 97 before this change. The pinned `REACHES_ADMISSION_WORK = 97` is therefore the real pre-change baseline, and head admits at the same 97.
- *Pre-existing gaps.* The two inline-equivalent mutants (undeclared destination type mapped to `key_refused`, and the interface form forced to `ObjectType`) also survive the full suite at base.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-262 | examined | relationship_end over a forward and an inverse admitted reaches application of one bidirectional homogeneous relationship, source {1,1}, target {2,3} unique role peer: Ok with application, relationship, forward, lower/upper/unique/ordered/role of the destination end; destination_type derived independently. |
| FR-038-AC-263 | examined | {0,3} unique, {0,3} not unique, {2,3}, {0,1}, {1,1} bounds; omitted role gives None; subtype operand gives the declared end type's derived id, also when the graph holds no node with that id. |
| FR-038-AC-264 | examined | Unknown id gives UnknownNode; navigate application, integer.add application, relationship node, object declaration node, field-read node and value/literal node give NotReachesApplication. |
| FR-038-AC-265 | examined | Crate-internal post-admission mutation of an admitted reaches application: member absent, kind not relationship_end, declaration not a node id / resolving to no retained relationship, name non-text / matching neither role / matching both roles each give UnresolvedEnd. |
| FR-038-AC-266 | examined | Two calls and a clone are equal, the package equals its pre-call self, no work is charged, and the AC-252..257 outcomes are unchanged. |
| FR-038-AC-267 | examined | External fixture: exhaustive error match without a wildcard, field-by-field nine-field construction. |
| FR-038-AC-268 | examined | model_object_fields returns no relationship end; no selected_population item is pub-used at the model root or re-exported from the quire-contract-ir root. |
| TC-048 "Typed relationship end of a reaches application" | examined | In crate-internal unit tests beside the accessor, where the private wire is available, mutate an admitted application so its member is absent, ... and assert UnresolvedEnd and no partial value for each. |
| FR-038 "Typed relationship end of a reaches application" | context_only | Retention / Result / refusal paragraphs |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-265 is evidenced only on a hand-built package that was never admitted, and that package carries a hand-built retained record. `fixture_package` deserializes a private wire directly and installs `RetainedModels::with_relationship(&"5".repeat(64), 0, "REL", ..)`, with synthetic keys `"8"*64` and `"9"*64`. AC-265 and the TC-048 section require "a crate-internal post-admission mutation of an admitted reaches application". The "name matches both roles" state is also not produced by mutating the application. It replaces the retained record with one whose two ends share a role. As a result, the production path from `RetainedModels::new` through `RetainedModels::relationship` to `UnresolvedEnd` is never exercised on retention that admission built. Two mutants survive: dropping the Relationship form check and dropping `check_declaration_node` in `RetainedModels::relationship`, the latter even against the full suite. The sibling accessor (IR-832 `parameter_declaration`) used the same hand-built pattern, so this follows precedent, but it does not meet the TC's "admitted" wording. A fix: admit an in-crate package (or reuse an admitted fixture through a `#[cfg(test)]` constructor), mutate its wire for the seven AC-265 states, and add one state that points `declaration` at an admitted object-type declaration node. | crates/quire-contract-model/src/checked_package/v2/relationship_end.rs:181-239,306-370; crates/quire-contract-model/src/checked_package/v2/model_fields.rs:521-533 |
| FND-002 | low | The "member absent" case sets `operation.member` to JSON `null` instead of removing the key. A `null` member fails the `kind` check, so the `pointer("/operation/member").ok_or(UnresolvedEnd)` branch for an actually absent member is never run. A mutant that maps a missing member to `NotReachesApplication` survives. Remove the key (`as_object_mut().remove("member")`) in this case, and keep `null` as an extra case if wanted. | crates/quire-contract-model/src/checked_package/v2/relationship_end.rs:309-311 |
| FND-003 | low | Three assertions in the AC-266 test cannot fail because of retention: `first.package_id() == second.package_id()` and `first.package_id().digest == package["package_id"]["digest"]`. `package_id()` reads the wire. `RetainedModels::new` runs after the package-id check (`mod.rs:940-971`) and has no access to the wire, so no change to retention can alter the id. They are harmless, but they record a guard that guards nothing. Either drop them, or say in the test that the id is pinned by the reader's own content check and is listed for completeness. | tests/it/checked_package_v2_model_members.rs:3622-3626 |
| FND-004 | low | The in-crate test `tc_048_relationship_end_reads_the_retained_destination_end` is tagged `FR-038-AC-262` and `FR-038-AC-264`, but it runs on the hand-built package of FND-001. Both criteria are about admitted applications: AC-262 asks for "an admitted ... application" with a destination id "derived independently", and here the key `"9"*64` is invented. The tests/it tests already carry the real evidence for both, so the extra tags inflate the backing count without adding admitted-package evidence. Keep the in-crate tags to AC-265 (and the equality clause of AC-266, which this test does check). | crates/quire-contract-model/src/checked_package/v2/relationship_end.rs:252-254 |

## Verdict

PASS with one medium and three low findings. The admitted-package evidence for AC-262, 263, 264, 266, 267 and 268 is real:

- **Independent oracles.** `expected_end` is built from authored constants. `relationship_id` and `order_declaration_id` come from `structural_key`, not from the accessor or from `end_declaration_key`.
- **Each package is admitted.** Every package goes through `CheckedPackageV2::read`.
- **Missing-node variant.** It removes the declared end type's node and its dependents, then refreshes the source map and identity before re-admitting.
- **AC-264.** All six node kinds are covered.
- **AC-267.** It destructures and rebuilds without `..` and matches the error without a wildcard.
- **AC-268.** It checks the model root `pub use` lists against the `selected_population` facade's export list, and checks the IR root.

The AC-265 list in the spec has seven states. The PR's unit test has eight: the extra state, "retained destination key is absent", is reachable only through a hand-built record. It is defensive, not a criterion.

**Survivors that are not findings.**

- `end_declaration_key` with the form forced to `ObjectType` (no systems-interface relationship end is admitted anywhere in the suite).
- The `None` to `key_refused` swap (no reaches/navigate end whose type is undeclared).
- The `Err` to `ineligible` swap (no key derivation over the byte limit).

All three survive against the full suite. All three test the same lines that were inline in `check_relationship_member` before this PR, so they are pre-existing gaps in the AC-252..257 evidence, not a regression. The refactor is byte-identical by reading (SR-12155 Verdict item 3).

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | `tc_048_an_absent_retained_destination_key_refuses_unresolved_end` is tagged `FR-038-AC-266`, but it checks none of AC-266's clauses (repeat and clone equality, pre-call equality, no work, the AC-252..257 outcomes). It checks a defensive `UnresolvedEnd` state that is outside AC-265's list and that an admitted package cannot reach. The tag inflates AC-266's backing. Drop the tag, or leave the test untagged, as `tc_048_unknown_and_non_reaches_nodes_refuse_by_variant` already is. | crates/quire-contract-model/src/checked_package/v2/relationship_end.rs:524-528 |
| FND-006 | low | The new comment in the AC-266 tests/it test says "The guards that retention can break are equality of independent admissions (above) and the charged work (below)". Two independent admissions carry equal retention, so `first == second` cannot catch retention leaking into equality. A mutant that adds `models` to `PartialEq` passes this test. It is killed only by the in-crate `tc_048_admitted_retention_is_read_and_outside_package_equality` (package vs `models` cleared) and by `tc_227`. Reword the comment to point at the in-crate equality test. | tests/it/checked_package_v2_model_members.rs:3622-3625 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, fe9a8e4. `admitted()` builds a package and requires `CheckedPackageV2::read` to return `Admitted`; it panics otherwise. The package has a real domain document (`Order`, relationship `LINKS` from `links` to `peer`, and relationship `SAME` whose ends share a role), node keys hashed from their own preimages, the package id and the evidence. The 11 AC-265 states mutate that admitted package's wire after admission, and the record they read is the one admission retained. Covered states: member removed or null; wrong kind; declaration not a node id, outside the index, an indexed object-type declaration, or a relation node with a broken declaration shape; name non-text, matching no role, matching both roles (via the admitted `SAME` record), or matching another relationship's role. Re-run in a fresh throwaway worktree: the `check_declaration_node` drop in `RetainedModels::relationship` is now KILLED. The author deleted the Relationship-form check as redundant. That claim holds: `model.relationships` is keyed only by identities that pass `valid_relationship_identity` (`ix://<pkg>/relationship/<name>`), and the reader refuses an object type in that namespace. A probe that added type `ix://acme/orders/relationship/Other` was refused `InvalidModelBinding`/`MalformedDeclaration`; the control type `ix://acme/orders/Other` was admitted. So an indexed declaration of another form cannot share a key with a retained relationship. The one hand-built state (retained key absent) starts from the admitted record and edits only `target_key`. Its doc says admission cannot produce it. That is honest. Its tag is FND-005. |
| FND-002 | fixed | Round 1. "member removed" now calls `.as_object_mut().remove("member")`, and "member null" is kept as a separate case. The mutant that maps a missing member to `NotReachesApplication` is now KILLED. |
| FND-003 | fixed | Round 1. The `package_id` assertions are removed and replaced by a comment saying why, and the test is renamed `..._changes_neither_equality_nor_work`. Re-run: retention-in-equality is KILLED (by the in-crate equality test and tc_227, not by this test, see FND-006). Retention-charging-work is KILLED (by this test's 97 boundary and by existing work-limit tests). |
| FND-004 | fixed | Round 1. `quire trace`: AC-262, AC-263 and AC-264 are now backed only by their tests/it tests over admitted packages. The in-crate tests carry only AC-265 (the post-admission mutation test) and AC-266 (the equality test, plus the tag in FND-005). Strict coverage stays at 43/0 and backed at 515/572. |
| FND-005 | fixed | Round 2, 2ee542d. The `#[trace("TC-048", "FR-038-AC-266")]` attribute and its `Trace:` line are removed from `tc_048_an_absent_retained_destination_key_refuses_unresolved_end`, and its doc now says "It backs no acceptance criterion by itself." `quire trace --id FR-038-AC-266` now lists only the in-crate equality test and the tests/it work test. Strict coverage stays at 43/0, backed at 515/572, and coverage warnings at 12. |
| FND-006 | fixed | Round 2, 2ee542d. The comment now says equal admissions carry equal retention, so the equality assertion cannot catch retention leaking into equality, and it names `tc_048_admitted_retention_is_read_and_outside_package_equality` as the test that does. Verified in a fresh throwaway worktree with a mutant that adds only the retained relationship records to `PartialEq`. The `it` suite does not catch it; its one failure was the environmental `selected_population` test without `QUIRE_SPEC_LANGUAGE_DIR`. Two model lib tests do: the named in-crate test and `tc_227_the_retained_tables_take_no_part_in_equality`. The comment does not claim the named test is the only one, so leaving out tc_227 is accurate. fmt, workspace clippy (`-D warnings`), the model lib suite (219) and the `it` model-members and public-interface tests (55) all pass at 2ee542d. |
