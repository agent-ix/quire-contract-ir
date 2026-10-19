---
id: "SR-13243"
title: "IR PR 480 gap analysis: FR-035 read-limit coverage delta"
type: SpecReview
analysis: gap-analysis
review_set: subset
scope: "agent-ix/quire-contract-ir PR #480; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md, spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md, spec/checked_package/matrix/tests.md, tests/it/complete_v1_contract_package.rs, crates/quire-contract-model/src/checked_package/v2/contract_package.rs (diff origin/main...HEAD)"
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: reviews
---

# SR-13243: IR PR 480 gap analysis

## Summary

Ticket: IR-889. PR: agent-ix/quire-contract-ir#480 at the reviewed PR head. Plan completion: not assessed.

Renumbering note: after IR PR #479 (IR-893) merged with its own FR-035-AC-12 to AC-14, this
PR's criteria were renumbered. The AC-12 and AC-13 named in this review, including its findings
and dispositions, are now FR-035-AC-15 and FR-035-AC-16. This review records the ids as they
stood when it was written.

Computed matrix delta, from `quire matrix --format tsv` (quire 0.36.2) at the reviewed merge base
and at the PR head:

- Exactly two rows were added: FR-035-AC-12 and AC-13, each `untagged`.
- FR-035-AC-11 stays `tagged` by `reader_incomplete_reports_each_exhausted_meter_and_pointer`
  (tests/it/complete_v1_contract_package.rs:215-298).
- No other row changed, and 0 are contradicted.

The exact CI command,
`cargo test --locked --workspace --all-targets --jobs 2 -- --include-ignored`, ran in the
reviewer worktree with its own target dir, under `flock /tmp/luna-ir-heavy-3.lock nice -n 10`.
It passed: 701 integration, 228 model unit and 1 public-consumer test, with zero tests in the
other targets.

The tagged AC-11 test asserts behaviour that the amended AC-11 now forbids. It asserts the
`/source_map` occurrence pointer with consumed 2 at limit 0. The prototype gives consumed 1 at
`/source_map/nodes/0/occurrences/0`. It also asserts that work limit 7 stops at
`/semantic_graph/nodes/2/node_id`. The cumulative prototype stops at
`/semantic_graph/nodes/0/node_id`. The IR-889 code PR must rewrite both assertions.

## Verdict

ACCEPTABLE WITH ONE LOW FINDING. The coverage delta is two new planned rows with 0 contradicted,
and the PR says so in prose. The computed matrix, however, still reports the amended AC-11 as
tagged by a test of its superseded behaviour.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | After the amendment, `quire matrix` and `quire coverage` still count FR-035-AC-11 as tagged and backed by `reader_incomplete_reports_each_exhausted_meter_and_pointer`. That test asserts the occurrence path `/source_map` and the work limit-7 stop at `/semantic_graph/nodes/2/node_id`, which the amended text contradicts. Only the status prose says the amended clauses are PLANNED/UNRUN, so the machine-readable coverage overstates AC-11 until the IR-889 code PR rewrites the test. The fix is to land the code PR promptly, or to state the stale assertions explicitly in the TC-047 Status so a matrix reader is not misled. | tests/it/complete_v1_contract_package.rs:215-298; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:175 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fix commit 36d5036 names the stale assertions in the TC-047 Status and both tests.md rows: occurrence limit 0 expects `/source_map` and consumed 2, and work limit 7 expects `/semantic_graph/nodes/2/node_id`. It says they do not back amended AC-11, and it marks AC-11 through AC-13 PLANNED/UNRUN until the code PR rewrites the test. |
