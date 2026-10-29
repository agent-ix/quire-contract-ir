---
id: SR-2026102748
title: "IR-1109 PR 683 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #683 at 43c2f46c; timed_model.rs and conformance_qspec/timed_model.rs"
review_set: subset
---

## Summary

Independent review of PR #683 at exact head 43c2f46c148b95dad9f8a1ff871cb7d141fea858 against FR-038 AC-3, AC-336/342/357-362, TC-048, and QSpec 86645fc1. The published STD-558 CL-25 and STD-559 TM-12/GD-02 cases remain unbacked. No CI workflow, dependency, or vendored artifact changed.

## Verdict

**CONDITIONAL** — The test tagged FR-038-AC-362 skips the exact published CL-25 oracle, although AC-362 requires exact admission/refusal for CL-01..CL-53. The PR correctly calls CL-25 unbacked, but this trace tag claims backing prematurely. Remove AC-362 from partial tests until STD-558 is repaired and the full vector control passes; keep the pending case explicitly asserted.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The test tagged FR-038-AC-362 skips the exact published CL-25 oracle, although AC-362 requires exact admission/refusal for CL-01..CL-53. The PR correctly calls CL-25 unbacked, but this trace tag claims backing prematurely. Remove AC-362 from partial tests until STD-558 is repaired and the full vector control passes; keep the pending case explicitly asserted. | tests/conformance_qspec/timed_model.rs:140-142 |

## Coverage

- Examined FR-038 AC-3, AC-336, AC-342, AC-357 through AC-362 and TC-048; QSpec FR-322 AC-260 through AC-266, FR-342 AC-11 and published AO/CL controls.
- Plan completion: not assessed.
- Matrix: `quire matrix --scope . --format json` completed; AC-336 and AC-342 appear, Planned AC-357 through AC-362 do not appear as active matrix rows.
- Rust review: checked repository AGENTS.md/CLAUDE.md, Rust style API, implementation, safety, wire and testing rules; diff has no unsafe code or CI gate edit. Focused conformance 20/20 was run by author, not repeated by reviewer.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The partial stage test no longer claims FR-038-AC-362; CL-25 remains explicitly asserted as pending STD-558 and AC-362 stays unbacked. After: `/// Trace: FR-038-AC-358, TC-048. AC-362 awaits STD-558.` |
