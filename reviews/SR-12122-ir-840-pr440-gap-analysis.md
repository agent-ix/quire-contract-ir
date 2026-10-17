---
id: SR-12122
title: "IR-840 gap analysis of IR PR #440"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@e1e0eddf3a0672be7648f3bcaf1804d944f548d9 (PR #440); git diff origin/main...HEAD: spec/checked_package/matrix/tests.md:32 (TC-048 Traces To cell); computed matrix (quire matrix --scope . --format tsv) and trace tags (#[trace(...)], /// Trace:) under tests/ and crates/ for the FR-038 criteria in and around the cell"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

# IR-840 gap analysis of IR PR #440

## Summary

Ticket: IR-840. This is a planless gap analysis of a spec-only PR that edits one
index cell. Plan completion was not assessed. The PR changes no production or
test code, so it adds no reverse gap and no stub.

I checked the tests behind each criterion in the cell, and the ones the cell
leaves out, against the computed matrix and the trace tags (quire 0.36.2,
engine 0.50.2):

- Of the 188 ids in the cell, 184 compute `tagged`. AC-193, AC-194 and AC-201
  compute `method-without-symbol`, because they are Inspection. AC-195
  computes `untagged`.
- `quire coverage --strict --scope .` reports 43 unbacked and 0 contradicted,
  the same on origin/main, at the PR head, and in a scratch copy carrying the
  corrected 206-id cell. The cell is an index that coverage does not check.
- Every id the PR added that has a TC id in its tags is tagged to TC-048. The
  exception is AC-183 and AC-185, which also carry one TC-226 tag each, and FR-038
  verifies AC-183 as `Test (TC-048, TC-226)`.
- The ids the PR left out that have TC-048-tagged tests: AC-9
  (checked_package_v2_reader.rs:2120), AC-21 (:2615 and four more), AC-214
  (ir575_reader_memory.rs, checked_package_v2_canonical_encoding.rs:864),
  AC-225..227 (conformance_qspec/main.rs:505, :563, checked_package_v2_union.rs)
  and AC-242..245 (checked_package_v2_operand_families.rs:230 and others). The
  cell omission is reported once, in SR-12120 FND-001..003.
- AC-202..209 are tagged with no TC id, in checked_package_v2_comparison_operands.rs,
  checked_package_v2_frame_entries.rs and comparison_operands.rs. TC-445:112-113
  names that file, so the binding by file is TC-445, not TC-048. Leaving them out
  is correct.
- AC-153/154 are tagged TC-228 (checked_package_v2_owners.rs). Leaving them out
  is correct.
- AC-150 and AC-155 also carry some TC-048 tags beside their owning
  TC-226/TC-228 tags. These are incidental cross-tags on shared fixtures, outside
  this diff, and no change to the cell is wanted.

## Coverage

Plan completion: not assessed

- Criteria in the cell: 188. tagged 184, method-without-symbol 3 (AC-193, 194,
  201), untagged 1 (AC-195).
- Criteria the cell should add (SR-12120): 18 of them. 14 compute tagged (AC-9,
  21, 214, 225..231, 242..245). AC-232..235 are untagged and PLANNED/UNRUN under
  IR-691, which matches how the cell already lists PLANNED AC-165..173, AC-224
  and AC-246.
- Reverse gap: none added, because the diff touches no code.
- Semantic review: skipped. The diff is a trace index, and no test or code
  changed.

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| TC-048 Traces To cell | examined | spec/checked_package/matrix/tests.md:32 | 188 FR-038-AC ids |
| AC-9 binding | examined | tests/it/checked_package_v2_reader.rs:2120 | #[trace("TC-048", "FR-038-AC-9", "FR-038-AC-26")] |
| AC-21 binding | examined | tests/it/checked_package_v2_reader.rs:2615 | #[trace("TC-048", "FR-038-AC-21")] |
| AC-214 binding | examined | tests/ir575_reader_memory.rs:36 | /// Tracing: TC-048, FR-038-AC-214 |
| AC-225 binding | examined | tests/conformance_qspec/main.rs:505 | #[trace("TC-048", "FR-038-AC-225")] |
| AC-242/243 binding | examined | tests/it/checked_package_v2_operand_families.rs:230 | #[trace("TC-048", "FR-038-AC-242", "FR-038-AC-243")] |
| AC-202..209 bindings | examined | tests/it/checked_package_v2_comparison_operands.rs | trace tags with FR-038-AC-20x and no TC id; TC-445 names this file |
| AC-195 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4561 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN ... The CODE slice passes the repository's model-consumer Clippy lane ... Test |
| AC-177..182, 224, 228..231, 236..238 tags | examined | tests/it/checked_package_v2_composite_operands.rs, tests/it/checked_package_v2_recursive_leaves.rs, tests/it/checked_package_v2_model_members.rs | trace tags carrying only FR-038-AC ids |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-038-AC-195 is now indexed under TC-048, but no test carries its trace tag, and the matrix computes it `untagged`. It is one of the 43 baseline unbacked rows. Its FR-038 row says IMPLEMENTED with Verification `Test`, but the obligation is a Clippy lane run (TC-048 "For AC-195, the CODE slice shall run both repository Clippy lanes"), which no trace tag can name. Pre-existing. Fix in a follow-up: either change the verification to a method that needs no symbol, or tag the lane's evidence. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4561, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1953 |
| FND-002 | low | The tests for FR-038-AC-177..182, AC-224, AC-228..231 and AC-236..238 carry only criterion ids in their trace tags, with no TC id. Their tie to TC-048 rests on the TC-048 prose section and the test file alone, so nothing machine-checks the index claim for these criteria. Pre-existing. | tests/it/checked_package_v2_composite_operands.rs, tests/it/checked_package_v2_recursive_leaves.rs, tests/it/checked_package_v2_model_members.rs:543 |

## Verdict

CONDITIONAL. This PR adds no gap of its own. The two low findings are
pre-existing tag-quality issues and can be deferred. Every TC-048 binding the
cell claims has tagged tests behind it, except AC-195 and the Inspection
criteria. The cell's merge blocker, the 21 ids it still leaves out, is reported
in SR-12120.
