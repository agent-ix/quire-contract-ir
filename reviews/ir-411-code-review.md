---
id: SR-983
title: "code review of PR 228 (lower admitted state frame and state_clause nodes)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@7b4ad49ee00ce5537290a5c3cdce6208319623c4; tests/it/checked_package_v2_frame_entries.rs"
review_set: subset
---
# SR-983: code review of PR 228

Former id: SR-624 (cited by the marker on IR-411).

## Summary

Ticket: IR-411. This is a code review with a Rust lane. The diff is one test, `tc_056_admitted_frame_and_state_clause_nodes_lower_under_a_state_profile`, and no production change. The test admits the in-repo `StatePackage` and lowers its `state`/`frame` node and its invariant and precondition `state`/`state_clause` nodes. It asserts `Lowered`, `CheckedNodeTag::State` and exact wire equality for each one.

Measured at the reviewed sha:

- `make fmt-check` and `make lint` pass.
- The TC-056 tests pass, 14 of 14.
- `make spec` fails with 22 unbacked rows. A clean origin/main 3e7935f fails with the same 22 rows. The only difference between the two outputs is one more bound evidence symbol (201 to 202).

Mutation results, run in a scratch copy:

- Refusing every State node as `Unsupported` is caught by the new test and by tc_050/tc_052.
- Refusing only frame and state_clause is caught by the new test and by tc_052_dependencies.
- Refusing only state_clause is caught by the new test alone, 177 passing and 1 failing.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new lowering test is traced to FR-040-AC-8 and FR-040-AC-9. Those are reader-admission criteria, and FR-040 explicitly scopes lowering out. Lowering behaviour is FR-038-AC-6 (TC-050) | tests/it/checked_package_v2_frame_entries.rs:1659-1661 |
| FND-002 | low | The test lowers only two of the five state forms, frame and state_clause (invariant and precondition only). No test lowers operation_anchor, transition, or a postcondition clause, although the StatePackage already carries an operation_anchor node | tests/it/checked_package_v2_frame_entries.rs:1666-1685 |

## Finding Detail

- FND-001: FR-040 is about admission. Its AC-8 and AC-9 describe clause shapes and signatures that admit or refuse. FR-040's Dependencies section says: "Lowering an admitted frame's grants to `kani::modifies` places is a later lowering requirement, not this one." The per-family lowering claim is FR-038-AC-6, which TC-050's tests trace. The binding inflates FR-040 coverage and leaves the new lowering evidence attached to a requirement that disclaims it. Fix: retag the test `#[trace("TC-050", "FR-038-AC-6")]`, and move it next to TC-050 or keep it where it is.
- FND-002: Adding `package.at("state", "operation_anchor")` to `positions` would cover a third state form at no cost. A postcondition node or a transition node would need new fixture nodes, so they are optional.

## Scope

- `tc_056_admitted_frame_and_state_clause_nodes_lower_under_a_state_profile`, examined: its oracle kills a state_clause-only refusal mutant that no other test kills.
- FR-040-AC-8, FR-040-AC-9, examined: the trace targets (FND-001).
- FR-038-AC-6, FR-038-AC-7, context_only: the lowering criteria.
- `crates/quire-contract-model/src/checked_package/v2/lower.rs` lower closure and Unsupported selection, context_only: it is tag-generic, and no per-form refusal exists.
- `tests/it/checked_package_v2_lowering.rs` tc_050 and tc_052, context_only: they already lower a `state`/`snapshot` node, and the all-families frame.

## Verdict

The test is sound and adds real coverage. It is the only test that asserts a state_clause node lowers, and mutation shows it is non-vacuous. Its trace tag points at the wrong requirement. `make spec` fails identically before and after, so the PR adds no unbacked row and changes none.

## Dispositions

Round 1, reviewed at 3edab7efea488e92b885c0af2c2ec053d1672c3d.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 3edab7e |
| FND-002 | fixed | 3edab7e |

- FND-001: the test is now `tc_050_admitted_frame_and_state_clause_nodes_lower_under_a_state_profile`, tagged `#[trace("TC-050", "FR-038-AC-6")]`. `make spec` still reports the same 22 unbacked rows as origin/main, and 202 bound symbols.
- FND-002: `operation_anchor` has been added to the lowered positions, so three of the five state forms are now lowered. Transition and postcondition would need new fixture nodes, and they stay optional. Mutation results at 3edab7e: a lower that refuses state_clause only fails this test alone (177 pass, 1 fails), and so does a lower that refuses operation_anchor only (177 pass, 1 fails).
