---
id: "SR-13232"
title: "IR PR 479 integrity: ids, FR backing, TC-047 and matrix rows"
type: SpecReview
analysis: integrity
review_set: subset
scope: "agent-ix/quire-contract-ir@f5229f01c640aa772fe7ba1c109a3e94dfe75b26; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md, spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md, spec/checked_package/matrix/tests.md (diff origin/main...HEAD)"
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: reviews
---

# SR-13232: IR PR 479 integrity

## Summary

Ticket: IR-893. PR: agent-ix/quire-contract-ir#479 at f5229f0.

Checked:

- **Ids.** AC-11 is the highest FR-035 id on main, so AC-12 to AC-14 are the next free ids. The
  open IR PR #480 (IR-889) also adds FR-035-AC-12 and AC-13. This is recorded as an
  informational collision only: the lead merges #479 first and renumbers #480.
- **Existing criteria.** AC-1 to AC-11 are byte-identical to main. The diff adds only rows
  and one note after the table.
- **Classes.** Each new refusal class (missing top-level member, leading whitespace, accounting
  and request-order mismatch) is stated by exactly one new criterion. AC-5's general clause
  "invalid or incomplete `source_map` correspondence refuses admission" overlaps AC-14 in
  topic only, not in the stated fields.
- **AC-2 reconciliation.** AC-2's text is unchanged. TC-047 now lists AC-2 in its Traces To.
  Its Status and the tests.md rows say that the AC-2 TC-047 retags are PLANNED/UNRUN, and
  that the stale-case assertions in
  `reader_refuses_malformed_outer_shape_and_stale_own_identity` and
  `selected_dependencies_are_bound_by_library_identity_before_graph_checks` are still tagged
  FR-035-AC-5. Both statements were verified in `tests/it/complete_v1_contract_package.rs`
  (lines 567 and 944).
- **TC-044.** Its row is unchanged, and it is still "PLANNED / UNRUN for FR-330".
- **TC-047.** The Description lists AC-2, AC-5 to AC-8 and AC-10 to AC-14. This matches
  the tests.md Traces To. The procedure has one step per new criterion and two for AC-2.
  The Expected Results add matching sentences. TC-047 states no step or case counts, so
  there was no count to reconcile.
- **tests.md.** The FR-035 row reads "AC-1 through AC-14". The TC-047 Traces To list is
  consistent with the TC-047 Description.

## Verdict

ACCEPTABLE WITH TWO LOW FINDINGS. The structure, ids and matrix rows are consistent, and the
status claims are honest.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-12 to AC-14 have no backing statement in FR-035's Behavior section. AC-10 and AC-11 each have a "When reading a canonical `ContractPackageWireV1`, Contract IR SHALL ..." sentence there. The new criteria do not, so the FR's requirement text does not say that a missing top-level member, noncanonical bytes or a request-order mismatch in the accounting refuse, or with which fields | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:139-146 |
| FND-002 | low | AC-12 names only `source_map`. The reader gives the same `invalid_package`/`""`/`missing_member` refusal for each of the six required top-level members. This was reproduced for all six, and `contract_version` takes its own branch at contract_package.rs:956. The criterion is not over-general, but the "missing top-level member" class has a criterion for one member only. The other five members, and the separate `contract_version` branch, stay unstated. Consider "omits any of the six required top-level members" | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:163; crates/quire-contract-model/src/checked_package/v2/contract_package.rs:956 |
