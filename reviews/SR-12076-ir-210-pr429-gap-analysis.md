---
id: SR-12076
title: "IR-210 slice D gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs, crates/quire-contract-model/src/checked_package/v2/model_members/tests/intake_retention.rs, spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/tests.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

## Summary

Ticket: IR-210 (slice D), PR quire-contract-ir#429. This is a planless gap analysis, so plan completion was not assessed. The matrix was measured with quire 0.36.2 (engine 0.50.2) twice: on origin/main, in a detached scratch worktree, and on.

`quire coverage --scope. --strict`:

- **Base:** 508/565 rows backed, 43 unbacked, 0 contradicted.
- **Head:** 508/565 rows backed, 43 unbacked, 0 contradicted. The full output is byte-identical to base, so the unbacked list is unchanged.

`quire trace --file` was run over both changed files on base and on head, and the (id, symbol) pairs were compared:

- Base has 71 pairs and head has 70.
- One pair was lost: FR-038-AC-189 on `tc_048_intake_retains_actual_nested_declaration_not_owner_or_missing_target`. That is the intended drop. AC-189 is still backed by `tc_048_intake_group_origin_requires_every_valid_equal_candidate`, now through `rust-trace-attribute`.
- No pair was gained.
- Every other touched FR-038 pair moved from `rust-trace-line` to `rust-trace-attribute` on the same symbol: AC-27, AC-28 x5, AC-29 x2, AC-30, AC-174 x2, AC-175 x2, AC-186, AC-187, AC-188, AC-189, AC-190, AC-191 x2, AC-192 x2 and AC-196.
- Each of the 17 TC-048 pairs moved from `rust-test-name-id` to `rust-trace-attribute`.

My measurements match the coder's `ir210d_file_before.tsv` and `ir210d_file_after.tsv` traces.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-27 | examined | A domain package selection is admitted only by the document the evidence supplies under its digest... and a matching document admits. |
| FR-038-AC-28 | examined | A domain package document's declarations refuse at the row, in FR-154's order... two nodes with no identity refuse `malformed-declaration`, never `conflicting-binding`. |
| FR-038-AC-29 | examined |... an inherited field resolves on a subtype and a subtype conforms to its supertype in either operand order; a field typed at an `Int[lo, hi]` value type has the integer-range member type `[lo, hi]`. |
| FR-038-AC-30 | examined | Reading a selected domain package and resolving a model-owned member are charged to the `work` limit... and the exact work admits. |
| FR-038-AC-174 | examined |... retains that exact FCD identity in the private refusal returned by `admit_selection`... Missing or non-string identity yields None... |
| FR-038-AC-175 | examined | Each private `SelectionRefusal` constructor... preserves the selected retained metadata record... Pre-declaration constructors preserve genuine metadata absence. |
| FR-038-AC-186 | examined |... exact typed source member values and optional-member presence..., including all four end-coordinate presence combinations and the existing admitted 2^53 boundary. |
| FR-038-AC-187 | examined |... exact generator identity/version text and input identity sequence... Input order/repetition remains unchanged and no source span is present. |
| FR-038-AC-188 | examined |... retains the actual offending field, operation, parameter or relationship's supplied identity and valid origin. Missing nested metadata remains None... |
| FR-038-AC-189 | examined | Given a located refusal with missing or malformed origin, the reader returns None for origin while independently retaining any available declaration identity.... |
| FR-038-AC-190 | examined | For every same-identity type group... retains origin only if every candidate has a valid origin and all typed values and optional-member presence agree.... |
| FR-038-AC-191 | examined | Retaining metadata does not change the existing first code, cause or selection-row member in node/table/member-path order.... |
| FR-038-AC-192 | examined | Retained metadata is acquired and released within the existing bounded document/read/work accounting... the first unpayable charge preserves the existing incomplete result and no partial package. |
| FR-038-AC-196 | examined | A relationship identity repeated sequentially... retains the later actual offending relationship's supplied identity and its own valid origin... |
| TC-048 | examined | TC-048 Test Case Summary row in `spec/checked_package/matrix/tests.md` ("Traces To" lists FR-038-AC-174, FR-038-AC-175, not AC-186..196) |
| spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md | context_only | "These Test and Inspection procedures back FR-038-AC-174/175 and FR-038-AC-186 through FR-038-AC-196." |
| spec/checked_package/matrix/tests.md FR-038 row | context_only | FR-038 Functional Requirement Coverage row lists "FR-038-AC-186 through FR-038-AC-196" with Test Cases TC-048, TC-050... |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

CLEAN. No gap is introduced and no claimed id is lost. Mergeable on this method.

- **Coverage.** Strict coverage is 43 unbacked rows on both base and head, and the output is byte-identical. The PR changes only the binding form, so no row gains or loses backing. Every touched id keeps all its symbols. The one exception is AC-189, which loses only the nested-declaration test, and that drop is honest (see SR-12075). AC-189 stays backed by the group-origin test.
- **TC-048 summary row (note, not a finding; no spec row fix needed in this PR).** The "Traces To" cell of TC-048 in `spec/checked_package/matrix/tests.md` lists AC-174 and AC-175 but not AC-186 to AC-196. It also leaves out other ids that TC-048 does back, such as AC-176 to AC-185, AC-202 to AC-213 and AC-222 to AC-251. That cell is a non-exhaustive index, and the omission predates this PR. The authoritative bindings already name AC-186 to AC-196 under TC-048:
  - the FR-038 Functional Requirement Coverage row (line 21), which lists "FR-038-AC-186 through FR-038-AC-196" with TC-048;
  - the TC-048 procedure artifact (lines 1865-1875).

  Strict coverage reports 0 contradicted statuses, so quire does not read that cell as a constraint either. Bringing the summary cell in line is an optional cleanup and belongs with whoever next edits the TC-048 summary, not with this attribute-only PR.
- **Known spec gap (note, pre-existing).** `tc_048_a_declaration_key_over_the_byte_limit_refuses_at_its_selection_row` backs a behavior that no numbered FR-038 criterion states: a declaration key over the byte limit refuses `invalid_semantic_graph` at `/lock/model_selections/0`. The FR-038 prose section "Every identity digest is computed through quire-canonical" says "`invalid_semantic_graph` at the node for a node key". For a model declaration key, that is the selection row only by implication. The test's doc comment already records this, and the TC-only tag reflects it honestly. Adding a numbered criterion is spec work outside this PR.
- **PR body (note).** The PR body says the TC-048 matrix row covers "186..196". That is true of the FR-038 coverage row and the TC-048 artifact, but not of the TC-048 summary row. It also counts "16 tests", which excludes the TC-only byte-limit test that the diff also tags. Neither affects the code.
