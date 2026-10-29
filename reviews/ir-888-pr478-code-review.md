---
id: SR-13160
title: "IR-888 quire-contract-ir PR 478 code review (with rust-review lane): FR-035-AC-10/AC-11 reader tests"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir PR #478 vs origin/main; tests/it/complete_v1_contract_package.rs, reviews/26-10-19-ir-888-reader-tests-code-review.md; read-only: crates/quire-contract-model/src/checked_package/v2/contract_package.rs, crates/quire-contract-model/src/checked_package/common.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: reviews
---

## Summary

Ticket: IR-888 (under IR-878). The PR adds two `#[trace("TC-047"...)]` tests and an author-written review file; no production code changes. TC-456, `public_interface.rs` and `identity.rs` are untouched. No `/// Trace:` lines. `rustfmt --check` passes. `cargo clippy --all-targets -D warnings` and `cargo test --test it complete_v1_contract_package` (17 passed) were run in a throwaway worktree.

The AC-10 test is strong: it asserts code, path, echoed version, no cause and no locus on a canonical wire, and checks that the version refusal wins over shape, identity and node defects, plus the numeric `wrong_value_kind` case. All seven AC-10 mutations were killed.

The AC-11 test drives the real 9864-byte, 3-node, 2-edge `mixed_fixture` through all five limit kinds with deterministic limits. Of 20 AC-11 mutations, 16 were killed and four survived. The survivors are three `consumed = limit + 1` substitutions (bytes, edges, work) and the disabled reference-validation work meter. See FND-001 and FND-002.

## Mutation results

Run in a throwaway worktree. Each mutation was applied alone and tested with `cargo test --test it complete_v1_contract_package::reader_`.

| Mutation | Result |
| --- | --- |
| M01 version refusal code -> invalid_package | killed (AC-10 test) |
| M02 version refusal path -> root | killed |
| M03 contract_version echo dropped | killed |
| M04 cause set on version refusal | killed |
| M05 version checked after validate_shapes | killed |
| M06 version checked after validate_identity | killed |
| M07 numeric version -> unknown_contract_version | killed |
| M08 nodes stop reported as Edges kind | killed (AC-11 test) |
| M09 occurrences stop reported as Work kind | killed |
| M10 nodes consumed = limit | killed |
| M11 edges consumed = limit + 1 | SURVIVED |
| M12 occurrences consumed = limit + 1 | killed |
| M13 bytes consumed = limit + 1 | SURVIVED |
| M14 bytes pointer = root | killed |
| M15 nodes pointer = /semantic_graph | killed |
| M16 edges pointer = node, not dependencies | killed |
| M17 edges pointer node index off by one | killed |
| M18 occurrences pointer = /source_map/nodes | killed |
| M19 work pointer = node, not body | killed |
| M20 work (value pass) consumed = limit + 1 | SURVIVED |
| M21 Incomplete returned as Refused | killed |
| M22 to M25 nodes, edges, occurrences, work (value pass) checks removed (admits) | killed (all four) |
| M26 reference-validation work meter disabled | SURVIVED |
| M27 byte limit check removed | killed |

## Reviewed units

- FR-035-AC-10 against `reader_refuses_wrong_string_version_before_shape_identity_and_nodes` (examined; clean)
- FR-035-AC-11 against `reader_incomplete_reports_each_exhausted_meter_and_pointer` (examined; FND-001, FND-002)
- reviews/26-10-19-ir-888-reader-tests-code-review.md (examined; FND-003)

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The AC-11 work branch "or the failed-charge pointer reported by reference validation, which may name another node member" is not asserted. The only work case (limit 0) stops in value validation at `/semantic_graph/nodes/0/body`. On the same fixture, `work = 7` deterministically stops in reference validation with consumed 8 at `/semantic_graph/nodes/2/node_id`. Mutation M26, which disables the reference-validation meter, survives. Add a work case at that limit; it asserts the reference pass's own pointer and consumed, and no cumulative bound, so IR-889 stays open | tests/it/complete_v1_contract_package.rs:198-252, crates/quire-contract-model/src/checked_package/v2/contract_package.rs:2045 |
| FND-002 | medium | AC-11's "`consumed` amount of the meter that stopped" is not told apart from `limit + 1` for bytes, edges and work. Every chosen limit makes consumed exactly limit + 1 (9863 to 9864, 1 to 2, 0 to 1), so M11, M13 and M20 survive. A reader that reports `limit + 1` would pass. Fix it with non-adjacent limits on the same fixture: bytes 100 gives consumed 9864 (from the code: consumed is `bytes.len`), and work 2 gives consumed 7 at `/semantic_graph/nodes/2/body` (measured). For edges, each fixture node has at most one dependency, so this fixture cannot separate them; state that in the test, or add a node with two dependencies | tests/it/complete_v1_contract_package.rs:198-252 |
| FND-003 | low | The PR commits an author-written SpecReview, `reviews/26-10-19-ir-888-reader-tests-code-review.md` (SR-13143), with a PASS verdict for its own change. Its file name carries a typed date prefix, its scope omits the owner, and it claims the tests "fail under targeted version and edge-pointer mutations" without listing them. The independent mutation run here shows five survivors. A self-review in `reviews/` reads as independent review evidence. Drop the file, or limit it to an author note | reviews/26-10-19-ir-888-reader-tests-code-review.md:1-22 |

## Verdict

**CHANGES REQUESTED (medium).** AC-10 is fully and strongly backed. AC-11's kind, pointer and no-admit clauses are backed for all five limits, but two gaps remain. The work reference-validation branch is unasserted (FND-001), and three `consumed` values cannot be told from `limit + 1` (FND-002). Both can be fixed in this test file with limits measured on the existing fixture. Rust-review lane: idiomatic test code; `let... else { panic! }` gives clear failure messages; no tautologies or stubs; the `unreachable!` for `Diagnostics` is sound because that arm is not in `cases`.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | The AC-11 nodes case (limit 2, consumed 3) cannot tell the real `consumed` from `limit + 1`, the same defect class as FND-002, which the round-0 review missed for nodes. Mutation M10b (nodes `consumed = limits.nodes + 1`) survives. Use nodes limit 1: consumed is `nodes.len` = 3, so the case separates the two | tests/it/complete_v1_contract_package.rs:224-229, crates/quire-contract-model/src/checked_package/v2/contract_package.rs:1619-1625 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | adds a work=7 case asserting Work, limit 7, consumed 8, path /semantic_graph/nodes/2/node_id, which is the reader's reference-validation stop; M26 is now killed; no cumulative bound asserted (IR-889 open) |
| FND-002 | fixed | Bytes limit 100 to consumed 9864 and work limit 2 to consumed 7 at /semantic_graph/nodes/2/body; edges uses an admitted two-dependency fixture (9984 bytes, verified admitted) at limit 0 to consumed 2 at /semantic_graph/nodes/1/dependencies; M11, M13 and M20 now killed |
| FND-003 | fixed | reviews/26-10-19-ir-888-reader-tests-code-review.md deleted; the only reviews/ files the PR adds are byte-identical to the reviewer's SR-13160 and SR-13161 |
