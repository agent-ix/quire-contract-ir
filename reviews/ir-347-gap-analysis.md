---
id: SR-631
title: "gap analysis of PR 233 (witness and replay deletion)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@6a04978b0e34cb4d8e1a179a58ae52dc19fde2df; spec/contract-test-matrix.md, tests/it/kani_replay.rs, src/kani/"
review_set: base
---
# SR-631: gap analysis of PR 233

## Summary

Ticket: IR-347. Plan completion: not assessed. Compared `quire coverage --scope . --strict --json` at origin/main ead7267 and at head 6a04978 row by row. `unbacked_rows` is the same 17 rows (FR-036 functional row and AC-1..AC-5, FR-037 row and FR-037-AC-6, FR-039 row and AC-1..AC-4, FR-019-AC-5, TC-045, TC-055, TC-058) on both sides; `criteria`, `obligations`, `groups`, `minted_targets`, `implements`, `status_lies` (0) and `totals` (161 of 180 backed) are byte-identical. The only differences: `binding_census` 250 to 228 candidates (223 to 201 bound), the TC-042 `shared_trace_ids` group loses the 22 `tc_042_*` symbols of `tests/it/kani_replay.rs`, and `unmatched_tags` drops 8 comment mentions of FR-031-AC-4 and AD-016 from that file.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Clean. None of the 22 deleted tests was the sole backing of any row: TC-042 stays backed by `tests/it/kani_shared.rs`, `kani_arithmetic.rs`, `kani_collections.rs` and `kani_objects.rs`, and FR-029, FR-030-AC-1..AC-3 and FR-031-AC-1 keep their backing. FR-031-AC-4 no longer exists in the spec (the matrix lists FR-031-AC-1 only; AC-5 is retired), so its loss of untraced comment mentions removes no evidence. FR-030-AC-4 is untouched. No production code is left without an owning requirement by the deletion.
