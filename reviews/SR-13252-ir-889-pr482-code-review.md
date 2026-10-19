---
id: SR-13252
title: "IR-889 IR PR 482 code review (with Rust lane): cumulative V1 reader work meter and occurrence charges"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@4d01ba18aaf6c70a4d9f39e05202cf88b557ffa8 vs origin/main; crates/quire-contract-model/src/checked_package/v2/contract_package.rs, tests/it/complete_v1_contract_package.rs"
review_set: subset
---

## Summary

Ticket: IR-889. PR agent-ix/quire-contract-ir#482. This review covers the reader change and the TC-047 tests, with the Rust lane folded in. Scope is `git diff origin/main...HEAD` only.

What the reviewer checked at the head:

- `ContractPackageWireV1::admit` builds one `WorkMeter` from `limits.work`. It passes that meter to `validate_values` and then, by `&mut`, to `validate_references`. The `WorkMeter::new` in `validate_references` is gone. `admit` is the only caller of both passes. No other non-test `WorkMeter::new` exists on the V1 contract-package path. The one in `mod.rs:1009` belongs to the separate CheckedPackageV2 reader.
- In `validate_values`, the old local `work` counter is replaced by `work_meter.charge(work, || at.key("body"))`. That method uses the same `saturating_add` and `>` test as before.
- In `validate_correspondence`, one count runs through `nodes`, then `requested_items`, then `refusals`. Each occurrence is charged before its regions. A failed charge reports the occurrence pointer or `.../regions/<k>`, never `/source_map`.
- The stale assertions in `reader_incomplete_reports_each_exhausted_meter_and_pointer` were rewritten, not deleted. Occurrence limit 0 now expects 1 at `/source_map/nodes/0/occurrences/0`. Work limit 7 now expects 8 at `/semantic_graph/nodes/0/node_id`.
- The new tests pin the numbers in TC-047, and every stop asserts `limit_kind`, `limit`, `consumed` and `path`:
  - work: 7 to 8, 20 to 21, 26 to 27, and 27 admits;
  - occurrences: 0 to 1, the first `requested_items` stop, 17 to 18 at `/source_map/refusals/1/occurrences/0/regions/0`, and 18 admits.
  The `requested_items` stop limit is counted independently from the fixture JSON.
- The diff uses `#[trace(...)]` tags only and adds no `/// Trace:` lines. It adds no hash, pin, digest, compatibility layer or `Cargo.toml`/`Cargo.lock` change. Admitted bytes are unchanged: the 9864-byte fixture assertion still holds, and the bounded read still admits.

Measured at the head:

- The focused TC-047 tests pass (19 of 19, `cargo test --locked --test it -- complete_v1_contract_package`).
- `make fmt-check` passes.
- `make lint` passes in both the workspace and the model-only lane.

Mutants:

- The new tests were run against the tests-first commit bd44821, which has the old reader. Three tests fail:
  - AC-11 gets occurrences consumed 2.
  - AC-15 gets the reset-meter pointer `/semantic_graph/nodes/2/node_id`.
  - AC-16 gets consumed 2.
- Source-edit mutants could not be applied, because the permission layer refused edits in the review worktree. By inspection, the pinned cases kill each of these mutants:
  - Regions charged before their occurrence: the limit 0 and limit 17 pointers change.
  - `refusals` charged before `requested_items`: the consumed count at the `requested_items` stop changes.
  - The region pointer collapsed to the occurrence pointer: the limit 17 path changes.
  - The occurrence charge dropped: the total becomes 9, so limit 17 admits.

The full CI command was not re-run. No finding needed it.

## Examined criteria

- FR-035-AC-11 (examined): "... carrying the exhausted `limit_kind`, selected `limit`, and `consumed` amount at the failed charge. `path` is absent for bytes, `/semantic_graph/nodes` for nodes, `/semantic_graph/nodes/<i>/dependencies` for edges, and the failing source-map value for occurrences. For work it is `/semantic_graph/nodes/<i>/body` when value validation stops, or the failed-charge pointer reported by reference validation ... The work count and occurrence pointer follow AC-15 and AC-16."
- FR-035-AC-15 (examined): "... admits when `limits.work` equals the independently counted total visits across both passes and returns `Incomplete` when it is one less ... A limit that fits each pass separately but is below their combined count also returns `Incomplete`, never `Admitted`; no second meter restarts at zero."
- FR-035-AC-16 (examined): "... a limit one below the independently counted combined occurrence and region charges returns `Incomplete` with `limit_kind = Occurrences`, the selected limit, `consumed` equal to the count at the first failed charge, and `path` naming the particular occurrence value ... or its `/regions/<k>` member ... A limit equal to that total admits; a stop does not report only `/source_map`."
- TC-047 (examined): the pinned `mixed_fixture()` numbers: value pass 7 and reference pass 20 (27 in total); work stops at 7, 20 and 26; occurrence and region charges total 18, stopping at 0 and 17 and at the first `requested_items` occurrence.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The occurrence and region charges repeat the same increment, compare and `ValidationFailure::incomplete(Occurrences, ...)` block, differing only in the pointer. Fix: a small local closure or helper `charge(&mut count, at)` called once for the occurrence and once per region, so the two charge sites cannot drift | crates/quire-contract-model/src/checked_package/v2/contract_package.rs:1767-1786 |

## Verdict

**PASS.** One meter carries through both passes, with no reset on any path. Occurrences are charged before their regions, in the order the spec requires. Every pinned TC-047 boundary is asserted on `consumed` and on the pointer, and the tests fail against the prior reader. FND-001 is an optional tidy-up and does not block merge.

## Dispositions

Disposition pass 1, against head 4bc2f0f (fix commit 4bc2f0f).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | A shared local closure, `charge(&mut count, at)`, does the increment, the `>` compare and the `Incomplete(Occurrences)` construction. It is called once for the occurrence and once per region, with the same pointers as before, so behaviour is unchanged. At 4bc2f0f the reviewer re-ran the focused TC-047 tests (19 of 19 pass, including the pinned 0→1, `requested_items`, 17→18 and 18-admits cases), `make fmt-check` (passes) and `make lint` (passes, both lanes) |
