---
id: SR-2026102747
title: "IR-1109 PR 683 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #683; timed_model.rs and conformance_qspec/timed_model.rs"
review_set: subset
---

## Summary

Independent review of PR #683 at exact against FR-038 AC-3, AC-336/342/357-362, TC-048, and QSpec. The published STD-558 CL-25 and STD-559 TM-12/GD-02 cases remain unbacked. No CI workflow, dependency, or vendored artifact changed.

## Verdict

**FAIL** — Equal-region ordering parses arbitrary-size quantity strings into BigUint and cross-multiplies them before charging work. A caller with work remaining after source-map visits can make two large rational bounds consume substantial CPU and memory despite a tiny work limit. Charge/limit numeric construction and comparison before doing it, and add an M/M-1 reader control for this branch.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Equal-region ordering parses arbitrary-size quantity strings into BigUint and cross-multiplies them before charging work. A caller with work remaining after source-map visits can make two large rational bounds consume substantial CPU and memory despite a tiny work limit. Charge/limit numeric construction and comparison before doing it, and add an M/M-1 reader control for this branch. | crates/quire-contract-model/src/checked_package/v2/timed_model.rs:1373-1379, 1346-1348 |

## Coverage

- Examined FR-038 AC-3, AC-336, AC-342, AC-357 through AC-362 and TC-048; QSpec FR-322 AC-260 through AC-266, FR-342 AC-11 and published AO/CL controls.
- Plan completion: not assessed.
- Matrix: `quire matrix --scope . --format json` completed; AC-336 and AC-342 appear, Planned AC-357 through AC-362 do not appear as active matrix rows.
- Rust review: checked repository AGENTS.md/CLAUDE.md, Rust style API, implementation, safety, wire and testing rules; diff has no unsafe code or CI gate edit. Focused conformance 20/20 was run by author, not repeated by reviewer.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Quadratic charge now precedes decimal scanning, BigUint parse and cross multiplication; the 1,000-digit equal-region reader test measures exact M/M-1 work exhaustion. After: `let digits = u64::try_from(value.len()).unwrap_or(u64::MAX);     meter.charge(digits.saturating_mul(digits), || at.clone())?;` |

## Final gate delta review

In the reviewed PR, the delta from reviewed consists solely of `#[ix_trace_rs::string_edge]` on `check_wire` and `comparison_rank`. Both attributes annotate existing string decision boundaries; no branch, parser shape, ordering key, refusal, test oracle, or work charge changed. SR-2026102747 FND-001 remains fixed; no new finding. The code lead owns the aggregate gate.
