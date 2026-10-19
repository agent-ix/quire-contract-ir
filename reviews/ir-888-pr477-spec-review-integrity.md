---
id: SR-13140
title: "IR-888 quire-contract-ir PR 477 spec review: FR-035-AC-10/AC-11 reader outcomes"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir PR #477 head 09e3b19 vs origin/main; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md, spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md, spec/checked_package/matrix/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-047
    type: reviews
---

## Summary

Ticket: IR-888 (parent IR-878). The PR is spec-only. It adds a Behavior paragraph and FR-035-AC-10 and FR-035-AC-11 for `ContractPackageWireV1::read`. It extends TC-047 with the reader controls and rewrites the FR-035 and TC-047 rows of `tests.md`. Each clause was checked against `crates/quire-contract-model/src/checked_package/v2/contract_package.rs` (reader at 905-971; limit charges at 1619, 1666, 1676 and 1745; second work meter at 2045) and `common.rs` (`read_typed_prefix` at 331, `unknown_contract_version` at 136). A throwaway probe was run in a separate worktree against the `mixed_fixture` wire: 9864 bytes, 3 nodes and 2 edges. The integrity sub-analysis is folded in here.

Strict coverage, origin/main to head: rows go from 607 to 609. Unbacked rows go from 40 to 42, the additions being FR-035-AC-10 and FR-035-AC-11 only. Contradicted statuses stay at 0. `quire validate` on the three edited files is clean, apart from module-level registry warnings. No spec-reading test reads the edited files, and FR-019 is untouched.

## Reviewed units

- FR-035 Behavior, new paragraph (lines 139-145) (examined)
- FR-035-AC-10 (examined)
- FR-035-AC-11 (examined)
- FR-035-AC-1 through FR-035-AC-9, checked to be unchanged (examined)
- FR-035-AC-2, the contract-version mutation clause, checked for contradiction (examined)
- TC-047 Description, Steps and Expected additions (examined)
- tests.md FR-035 row and TC-047 row (examined)

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The rewritten tests.md FR-035 and TC-047 rows say FR-035-AC-6 through AC-8 are PLANNED/UNRUN. Tagged TC-047 tests for those criteria have been on main since IR-766 (#366): tests/it/complete_v1_contract_package.rs:182 traces AC-6/7/8, and :265 traces AC-7/8. Strict coverage reports AC-6 through AC-8 as backed (FR-035 8/11). The new status text misstates evidence that exists | spec/checked_package/matrix/tests.md:12, spec/checked_package/matrix/tests.md:30 |
| FND-002 | medium | FR-035-AC-11 says `path` "is the failed-charge pointer for the other limits" without naming any pointer, so a test can assert whatever the code emits and the AC cannot fail on `path`. The code gives these pointers. Nodes: `/semantic_graph/nodes`. Edges: `/semantic_graph/nodes/<i>/dependencies`. Work in the value pass: `/semantic_graph/nodes/<i>/body`. Occurrences: only the coarse `/source_map`, not the failing entry. Work in the reference pass: various node members. The AC should name the exact pointer for each limit | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:161 |
| FND-003 | medium | FR-035-AC-11 lists the work limit only "during node, work, edge or source-map occurrence validation" and promises "measured `consumed`". The reader charges work twice. It sums body work in `validate_values` (contract_package.rs:1630-1673), then starts a fresh `WorkMeter::new(limits.work)` at zero in `validate_references` (:2045). In the probe, work limits 7 through 19 all stop in that second pass, at pointers such as `/semantic_graph/nodes/2/node_id`, with `consumed` counting only that pass. A limit of 20 admits even though 27 units were charged in total. The AC and the TC-047 step "lower each ... work ... limit below the measured amount" leave the per-pass semantics and the "measured amount" undefined | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:161 |
| FND-004 | low | FR-035-AC-11 and TC-047 assert "the first exhausted `limit_kind`" but never state the order the reader charges limits in: bytes, then nodes, then work and edges per node, then occurrences, then reference-pass work. A combined-limit test therefore has no oracle in the spec | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:161, spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md:43 |

## Verdict

FR-035-AC-10 is true, word for word, against the code. The probe confirmed `Refused` with code `UnknownContractVersion`, path `/contract_version`, and `contract_version` set to the supplied string, with `cause` and `locus` absent. The check is an object-member lookup after canonical parsing (contract_package.rs:944-956), so it does not depend on field order. It precedes the shape, identity and limit checks after the preparse step. The probe got the same refusal with an extra outer member, with the digest rehashed, and with `nodes: 0`. A numeric `contract_version` gets `invalid_package`/`wrong_value_kind` instead, consistent with the AC's "string" qualifier. The AC can fail, for example on a different code, path or missing version. It does not contradict FR-035-AC-2.

FR-035-AC-11's Incomplete outcome is reachable through ordinary limits, not only a defensive branch. Bytes gives `Bytes`, limit 9863, consumed 9864, path absent. Nodes, edges, occurrences and work each gave `Incomplete` with the limit as selected. The diagnostics limit is not charged by this reader: `diagnostics: 0` admits. The AC does not claim it, but the Behavior sentence "When a caller-selected read limit is exhausted" covers it vacuously.

The ids FR-035-AC-10 and FR-035-AC-11 are the next free ones. AC-1 through AC-9 are byte-identical to main. The TC-047 trace column now correctly lists AC-5 through AC-8, AC-10 and AC-11. The new rows are honestly planned and unbacked.

Not mergeable as-is: FND-001 through FND-003 should be fixed in this PR.
