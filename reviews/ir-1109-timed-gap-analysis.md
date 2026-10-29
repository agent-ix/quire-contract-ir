---
id: SR-2026102748
title: "IR-1109 PR 683 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #683; timed_model.rs and conformance_qspec/timed_model.rs"
review_set: subset
---

## Summary

Independent review of PR #683 at exact against FR-038 AC-3, AC-336/342/357-362, TC-048, and QSpec. The published STD-558 CL-25 and STD-559 TM-12/GD-02 cases remain unbacked. No CI workflow, dependency, or vendored artifact changed.

## Verdict

**CONDITIONAL** — The test tagged FR-038-AC-362 skips the exact published CL-25 oracle, although AC-362 requires exact admission/refusal for CL-01..CL-53. The PR correctly calls CL-25 unbacked, but this trace tag claims backing prematurely. Remove AC-362 from partial tests until STD-558 is repaired and the full vector control passes; keep the pending case explicitly asserted.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The test tagged FR-038-AC-362 skips the exact published CL-25 oracle, although AC-362 requires exact admission/refusal for CL-01..CL-53. The PR correctly calls CL-25 unbacked, but this trace tag claims backing prematurely. Remove AC-362 from partial tests until STD-558 is repaired and the full vector control passes; keep the pending case explicitly asserted. | tests/conformance_qspec/timed_model.rs:140-142 |

## Coverage

- Examined FR-038 AC-3, AC-336, AC-342, AC-357 through AC-362 and TC-048; QSpec FR-322 AC-260 through AC-266, FR-342 AC-11 and published AO/CL controls.
- Plan completion: not assessed.
- Matrix: `quire matrix --scope. --format json` completed; AC-336 and AC-342 appear, Planned AC-357 through AC-362 do not appear as active matrix rows.
- Rust review: checked repository AGENTS.md/CLAUDE.md, Rust style API, implementation, safety, wire and testing rules; diff has no unsafe code or CI gate edit. Focused conformance 20/20 was run by author, not repeated by reviewer.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The partial stage test no longer claims FR-038-AC-362; CL-25 remains explicitly asserted as pending STD-558 and AC-362 stays unbacked. After: `/// Trace: FR-038-AC-358, TC-048. AC-362 awaits STD-558.` |
| FND-002 | still-open | Remove the FR-038-AC-342 trace from partial tests, or make the exact duplicate-place control pass after STD-559. |
| FND-003 | still-open | Remove the FR-038-AC-336 trace from partial tests, or make the exact duplicate-place control pass after STD-559. |
| FND-002 | fixed | The AO/GD control now carries only `#[trace("TC-048")]`; no FR-038-AC-342 tags remain in timed test code while STD-559 blocks the duplicate-place oracle. |
| FND-003 | fixed | The AO/GD control now carries only `#[trace("TC-048")]`; no FR-038-AC-336 tags remain in timed test code while STD-559 blocks the duplicate-place oracle. |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | `timed_aggregate_order_published_vectors_read_at_exact_loci` tags FR-038-AC-342, but its GD-02 branch accepts `malformed-clock-place` instead of the criterion's required `duplicate-clock-place` because STD-559 remains pending. This makes a partial control appear backed in the computed matrix. | tests/conformance_qspec/timed_model.rs:322,381-390; FR-038-AC-342 |
| FND-003 | medium | The same test tags FR-038-AC-336 while its GD-02 duplicate-place control still receives the earlier `malformed-clock-place` refusal. AC-336 requires the distinct same-path place to refuse `duplicate-clock-place`; the trace is premature. | tests/conformance_qspec/timed_model.rs:322,381-390; FR-038-AC-336 |

## QSpec TM-30 delta review

At PR, the QSpec `TM-30` repeated-reference control from is applied to the public reader and checks exact `invalid_semantic_graph`/null-cause/locus. The SL-01..07/SR-01..04 set is asserted exactly. The later trace cleanup removes all FR-038-AC-336/342/360 tags from partial timed tests while STD-558/559 remain unresolved. FND-001 through FND-003 have fixed latest dispositions; no new finding remains.
