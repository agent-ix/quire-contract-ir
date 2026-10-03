---
id: SR-1050
title: "code review and Rust review of PR 269: the v2 reader admits temporal formulas, fairness and control.case (IR-549)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@ddc7399fdc8ac05f8ae09f2c31ca21de01ba10cb; git diff origin/spec/ir-549-admit-temporal-and-case...ddc7399 (25 files: crates/quire-contract-model/src/checked_package/**, tests/it/**, tests/conformance_qspec/main.rs, Cargo.toml, Makefile, spec matrix rows); base spec PR 268 b4d2cee on held PR 253 e8db087; read against quire-specification@b1da9c8"
review_set: subset
---
# SR-1050: code review and Rust review of PR 269: the v2 reader admits temporal formulas, fairness and control.case (IR-549)

## Summary

Ticket: IR-549. PR 269 is the code change. Its base is spec PR 268 (b4d2cee), which is stacked on held PR 253 (e8db087). Scope: `git diff origin/spec/ir-549-admit-temporal-and-case...ddc7399`, 25 files. The spec is the FR-038 text in the base branch: ACs 96 to 108, the amended AC-67, 68, 69, 81 and 87, and TC-048's 'Temporal, fairness and case admission' section. The PR body and the author's report were treated as data, and every claim below was measured.

What I measured, in a detached throwaway worktree under /home/peter/dev/worktrees with its own target directory (since removed):

- Gates. `make ci` passed fmt-check, clippy (workspace and model-only), test (it 261, model unit 111, doc 7) and corpus (99/99 match). It stopped at `make spec`. That stop is the baseline: validate passes, there is one grammar finding (FR-014 ac:vague-response), and `--strict` reports 23 unbacked and 0 contradicted. `make deny`, `make cargo-audit` and `make audit-unsafe` all pass. There was more than 400G free on /.
- Conformance. The shared quire-specification checkout is clean, on main, at b1da9c8, which equals origin/main after a fetch. With `QUIRE_SPECIFICATION_DIR=/home/peter/dev/quire-specification make conformance-qspec`, tc_048_qspec_positive_fixtures_admit passes, and all three named fixtures admit with their recorded package_id. With the variable unset or empty, or pointing at /tmp, the target fails (exit 101, a test failure naming the variable). It does not skip.
  - tests/conformance_qspec has `test = false`, and the `make test` log has no conformance_qspec run.
  - No QSpec file is in the diff, and .github/ is untouched.
  - Probe: QSpec's other three positive fixtures also admit (control-operations, nominal-identities, operation-identities).
- QSpec negative fixtures (adverse.json, applied to positive-all-families):
  - All 5 structural mutations refuse with QSpec's expected code: unknown_contract_version, digest_domain_mismatch x2, unsupported_node_tag and invalid_semantic_graph.
  - All 10 body-grammar variants refuse; none admits. QSpec expects malformed_wire; the reader refuses earlier, on stale keys, dependencies or identity, as before.
  - No adverse mutation touches temporal, case or union.
- Mutation probes in the throwaway worktree, running the full workspace suite each time: 25 mutants, 22 killed, 3 survived. The survivors are M08 (lock.edition row ignored), M16 (wrong-selection-role matched on identity only) and M11 (the unreached-formula bounds sweep removed).
  - Killed: skipping the negative-bound term-walk check; the bound order upper-before-lower; lexicographic bound comparison; bounds before profile fit; placement interleaved with clause checks; over after fairness; an unknown profile read as bounded; a nested case accepted; union_arms dropped; binder-type and arm-body-type checks dropped; duplicate-member and payload-type checks dropped; the union-value payload type check; the member: leaf segment; the diagnostics misplacement check; formula and fairness reference placement; bounded profiles admitting fairness; timed admitting {lower, null}; profile fit on the root formula only; the interval shape check.
- Spec trace hygiene. No `AC-66` trace remains in any .rs file. The matrix lists AC-96 through AC-108 as implemented, and AC-66 as retired with the FR-031-AC-5 pattern.
- Process. The committed diff has no whitespace errors (`git diff --check` is clean). Every changed file ends with a single newline. There are no heredoc markers, and the appended unit-test module in temporal.rs is rustfmt-clean, so there is no artifact finding.
- Consumers, read-only, via a sub-survey of origin refs. quire-spec-language breaks (SR-1051 FND-001). quire-contract-codegen, quire-driver, quire-protocol and quire-analyze are not affected.
- The author's choices where the spec was silent. Two are within the spec: the clause checks run only when operation.identity is catalogued with operator temporal (anything else is the operation step's refusal), and a fairness declaration must be a model declaration node, which the spec words. The rest are flagged readings and are recorded as low findings in SR-1051: identity-only profile matching, union-only cycles, malformed union bodies, unresolvable arm types, the unreached-formula sweep, and the fixture growth.


## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Two branches of AC-108's package-alone cause rule have no test, and mutation probes confirm it. (1) AC-108 says the cause is wrong-selection-role when the law's definition is a row of lock.profile_selections 'or the lock.edition row'. Replacing `lock.edition.definition == *definition` with `false` in selected_under_another_role leaves every test green (probe M08). (2) The rule matches the same {authority, identity}. Comparing the identity alone also leaves every test green (probe M16), because every test row uses authority agent-ix on both sides. Add a case where the law names the lock.edition definition, expecting wrong-selection-role. Add a case where a profile_selections row shares the identity under another authority, expecting unsupported-selection. | crates/quire-contract-model/src/checked_package/v2/temporal.rs:471-482 |
| FND-002 | low | The runtime assertions of tc_048_the_reader_has_no_unsupported_construct_code_and_no_expression_form_cause are vacuous. They check that three hand-picked variants do not map to 'unsupported_construct' or 'expression-form', which no implementation could fail. The real oracle is compile-time: code_word and cause_word match every variant with no wildcard, and the enums are not non_exhaustive, so a variant added back fails to build. The doc comment says so, but the test body reads as if the asserts carried the weight. Assert something that can fail, for example that every word code_word and cause_word return parses back through the wire serde into the same variant. Or drop the asserts and keep the exhaustive matches as the oracle. | tests/it/checked_package_v2_temporal.rs:991-1007 |
| FND-003 | low | The sweep that checks lower > upper on a formula node no clause reaches has no test. Replacing `&& !covered.contains(&position)` with `&& false` leaves every test green (probe M11). It is one of the author's own spec-silent choices (see SR-1051 FND-006), so neither side of it is pinned. Either add a test, for example an unreached `once {3, 0}` refusing invalid-value at its body, or drop the sweep once the spec rules on it. | crates/quire-contract-model/src/checked_package/v2/temporal.rs:306-318 |

## Verdict

Changes requested, one medium. The code matches FR-038 AC-96 to AC-108 and the amended AC-67, 68, 69, 81 and 87, AC by AC. Measured:

- Placement is checked over the whole graph first, then clause by clause, in digest order. Within a clause the order is profile identity, over, fairness, profile fit, bounds.
- The negative bound is refused in the term walk, lower before upper, ahead of placement.
- lower > upper is checked after profile fit, using unbounded integer comparison.
- union_arms and the FR-440 joins are enforced. The member:<Name>/position:i union leaves are in place.
- unknown_profile causes come from the package's own lock.
- The refusal code and cause variants are removed, and CheckedDiagnosticCode::UnsupportedConstruct is kept.

The Rust is idiomatic: walks use explicit stacks, there is no panic surface outside index-map-guarded indexing, and there is no unsafe code and no integer conversion. The gates are green up to the spec baseline. 22 of 25 mutants are killed.

FND-001 is the one real gap: AC-108's lock.edition branch and its authority half are untested, and their mutants survive. FND-002 and FND-003 are low test-quality items.

## New findings (disposition pass 1)

Found at 27ecc3030d27b42575bfe292dd103a9f41a2bf9b, rebased onto spec base a2a6db1b225f3540135864497bf438a23ca19539.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | The new malformed-declaration refusal for a fairness declaration that is no model/object_type node puts the right path but the wrong locus. #268 a2a6db1 locates it 'at that target (as FR-342 step 1 refuses an anchor's context)'. FR-342 step 1 says 'The locus is the target', and this repo's FR-040 convention (tests/it/checked_package_v2_frame_entries.rs, 'a context naming a frame') puts the path at the referencing member and the locus at the target node's key. The code puts the locus at the fairness node, graph.nodes[fair].node_id, not at member.declaration. No test asserts the locus: mutant M29 (locus = member.declaration) survives, and so does the code as written. Set the locus to member.declaration and assert it in tc_048_a_fairness_member_resolves_on_its_declaring_model_node, or have the spec say explicitly that the locus is the fairness node. | crates/quire-contract-model/src/checked_package/v2/temporal.rs:572-584 |

## New findings (disposition pass 3)

Found at 44e7817d56edfed491d037dbfd0ce23bbb77772d.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | Dead branch. validate_diagnostics walks each details term with validate_term(..., is_body_root = false) before misplaced_in_details runs. The term walk now refuses every case application that is not a body root, at the same /diagnostics/entries/{e}/details/{d}/.../operator pointer with the same code, cause and empty locus. So a case application in a details term never reaches the `// operator == Case` arm. Mutant R14, which removes the arm, survives with every test green. Behaviour is correct and matches the 'Path and locus' row. Either drop the arm and note in misplaced_in_details's doc that the term walk refuses case, or keep it as defence in depth and say so. Not blocking. | crates/quire-contract-model/src/checked_package/v2/temporal.rs:misplaced_in_details, the `// operator == Some(ApplicationOperator::Case)` arm |

## Dispositions

Round 1, reviewed at 27ecc3030d27b42575bfe292dd103a9f41a2bf9b (rebased onto spec #268 head a2a6db1b225f3540135864497bf438a23ca19539). The gates were rerun in a throwaway worktree, since removed. make ci passed fmt, clippy, test (it 266, model 111, doc 7) and corpus (99/99), and stopped at make spec on the baseline (validate passes, grammar 1, strict 23 unbacked, 0 contradicted). make deny, make cargo-audit and make audit-unsafe pass. make conformance-qspec passes against quire-specification b1da9c8, which is clean. git diff --check is clean.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed 27ecc30 | tc_048_the_cause_matches_the_edition_row_and_the_authority_of_a_definition: a law naming the lock's edition definition refuses unknown_profile/wrong-selection-role; the edition identity under another authority, and a binding_contract row of quire.package.composed/v1 under authority 'other', refuse unsupported-selection. Mutants M08 and M16 are now killed by it. |
| FND-002 | fixed 27ecc30 | The AC-101 test states that its oracle is compile-time. It asserts the four new cause words and unknown_profile through code_word/cause_word, and that a diagnostics entry whose cause_tag is 'expression-form' is refused. Each of those can fail. |
| FND-003 | fixed 27ecc30 | tc_048_an_unreached_formula_node_with_lower_above_upper_is_refused: an unreached once {3, 0} refuses invalid_package/invalid-value at its /body and {0, 3} admits. Mutant M11 is now killed. The spec (#268 a2a6db1, AC-97) states the sweep. |

Round 2, reviewed at ddaf828d0fbe6d0732753bd1c169079b1d6c505e (rebased onto spec #268 head dd74358160ab74dcdda001e0d2e07d105c5ef53c). The net change since 27ecc30 is mod.rs (with_node_locus), temporal.rs and this PR's own temporal tests. No pre-existing test file was edited. Gates were rerun in a throwaway worktree, since removed. make ci passed fmt, clippy, test (it 266, model 111, doc 7) and corpus 99/99, and stopped at make spec on the baseline (validate passes, grammar 1, strict 23, 0 contradicted). deny, cargo-audit and audit-unsafe pass. conformance-qspec passes, with three fixtures admitted against b1da9c8. git diff --check is clean. Mutants M29 and M31 to M38 (locus per row, and with_node_locus widened to every term-walk refusal) are all killed. M34 is caught by three pre-existing reader tests, so the locus of other refusals is unchanged.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-004 | fixed ddaf828 | resolve_fairness returns FairnessRefusal::at_declaration(ModelRefusal::malformed(), true), so check_clause refuses with path /semantic_graph/nodes/{fair}/body/operation/member/declaration and locus member.declaration. tc_048_a_fairness_member_resolves_on_its_declaring_model_node asserts the locus through expect_located for a scalar_type and a model/value_type declaration. Mutant M29 is now killed. The rest of #268 dd74358's 'Path and locus' table is implemented row by row, and mutants M31 to M38 are all killed. |

Round 3 (final), reviewed at 44e7817d56edfed491d037dbfd0ce23bbb77772d. Its base is feat/artifact-ref-authority-identity at 5cef41f166dd94ffa18ca3009626091046ac9c4b, which holds merged #268 and so the final FR-038 with its 'Path and locus' table; QSpec is merged at f39c93f. Measured in a throwaway worktree, since removed:
- make ci passed fmt, clippy, test (it 268, model 111, doc 7) and corpus 99/99. It stopped at make spec on the baseline: validate passes, 344/345 grammar-clean (1 finding), strict 23 unbacked, 0 contradicted.
- deny, cargo-audit and audit-unsafe pass.
- conformance-qspec admits all three fixtures against b1da9c8 and against a detached QSpec worktree at f39c93f, and fails when QUIRE_SPECIFICATION_DIR is unset.
- f39c93f's new adverse mutation negative-temporal-interval-bound refuses invalid_package/invalid-value at node 29's interval/lower, with the formula node's key as locus. All 5 structural mutations refuse as QSpec expects.
- git diff --check is clean, there are no conflict markers or EOF artifacts, and every file ends with one newline. The PR's spec diff is only TC-048, the matrix row and spec/tests.md, so `-X ours` overwrote no base text. No AC-66 trace remains.
- The record/tuple recursion tests (checked_package_v2_recursive_leaves.rs) are untouched and pass.
- Every edited pre-existing test file follows a spec change: artifact_refs, catalog_words, lowering, reader, complete_v1, support, and the operations.rs unit tests (the AC-67 node operator moves from case to unary because the case form contradiction now comes first).
- Mutants R01 to R15: 14 killed, R14 survived (FND-005). The killed ones cover the bound pattern and its order, upper null, case placement and the form contradiction, the nested-case path and locus, the null member at the application, the fairness no-node rule, the details reference rules, timed null-only and union recursion.
Latest outcomes: every SR-1050 FND-001 to 004 is fixed; SR-1051 FND-001 is deferred (QSL) and FND-002 to 010 are fixed. No row needs a new outcome.

Round 4, reviewed at ceb46c3509ff5b7d0433ee57411db955df93762d, on the same base 5cef41f. `git diff --stat 44e7817 ceb46c3` touches one file, temporal.rs (+5 -6): the dead arm is removed from misplaced_in_details and the doc comment is updated, nothing else. The new code is exactly mutant R14 from round 3, which compiled and passed the full workspace suite, so behaviour is unchanged and no gates were rerun.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-005 | fixed ceb46c3 | if application_operator(term).is_some_and(is_placed_class) { return Some(at.key("operator")); } — the `// operator == Some(ApplicationOperator::Case)` arm is gone, and the doc comment says a case application in details is refused earlier by the term walk at the same pointer. |
