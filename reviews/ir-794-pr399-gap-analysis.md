---
id: SR-11128
title: "Gap analysis — IR-794 owner outcome evidence"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/source_obligation.rs, tests/it/source_obligation.rs, tests/it/public_interface.rs, spec/model/functional/FR-019-rust-library-interface.md; FR-052-AC-9..11, TC-452"
review_set: subset
---

## Summary

PR #399 adds a neutral terminal-retention boundary, but synthetic integration tests claim trace credit for full owner-path criteria before native adapters exist. Ticket: IR-794. Plan completion: not assessed.

## Examined scope

- FR-052-AC-9 (examined): For each owner, the native enumerator or typed adapter issues a `Complete` outcome only after the selected named scope reaches its normal end; the outcome retains the selected source, typed owner and scope, every owner-emitted row and the observed count. A complete zero-row scope and a fully parsed module with zero functions remain complete, while an absent owner report remains absent. Deleting one row after the owner's full output is observed makes the owner-output comparison fail even when the remaining row count is changed to match.
- FR-052-AC-10 (examined): For each owner, stopping before normal exhaustion issues `Incomplete` with retained partial rows and the typed cause the owner actually reports. QSL type-work exhaustion retains dimension, limit, prior, requested charge and site; QSL `TypeDisposition::Unfinished` retains its declaration handle and is distinct. An IR V2 reader limit retains the `CheckedPackageIncomplete` fields. For an owner path that reports `quire_exact::CancelCause`, requested and deadline cancellation remain distinct through `SourceInventoryCancelCause`; the current QSL inventory and `quire-code-rs` adapter do not claim cancellation support. `stop_reason: None` on a provisional envelope cannot be promoted to `Complete`, and a new owner stop type cannot be projected into a string or generic early stop.
- FR-052-AC-11 (examined): The neutral model retains an owner-issued outcome only when its source, owner, scope and observed count match the retained rows; a mismatch is typed and refuses the outcome. Owner adapter tests show that no adapter completion branch accepts an arbitrary caller-supplied row vector/count/completion flag in place of enumerating the selected native owner output. The model does not claim independent authentication of external owner exhaustion or a completed three-owner census.
- TC-452 (examined): Owner-output comparison, typed native stops and deletion oracle remain UNRUN.
- FR-019 public inventory (context_only): Public exports listed.

## Verdict

FAIL — FR-052-AC-10 and AC-11 are falsely backed by neutral synthetic tests. FR-052-AC-9 remains honestly unbacked; no full-census or owner adapter qualification is claimed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-10 trace tag claims owner stop evidence from synthetic enumerator cases only | tests/it/source_obligation.rs:363 |
| FND-002 | high | AC-11 trace tag claims owner adapter and full-output evidence absent from this PR | tests/it/source_obligation.rs:323 |

## Coverage

Diff-scoped matrix audit: the measured pre-spec log has 49 unbacked rows; merged FR-052 adds AC-9..11 for a 52-row expected pre-code baseline. PR #399 logs 50 unbacked rows with zero contradicted statuses because its two synthetic tests tag AC-10 and AC-11. AC-9 is unbacked. The full `make -k ci` attempt remains red only at strict coverage; this review did not rerun it. Plan completion: not assessed.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At 9c1b2a3 the stop test tags only FR-052-AC-12/13. The current matrix leaves AC-10 untagged, and TC-452 plus FR-052 retain owner-native stop evidence as Planned/UNRUN. |
| FND-002 | fixed | At 9c1b2a3 the zero/nonzero/count test tags only FR-052-AC-12/13. The current matrix leaves AC-11 untagged; the new multi-row control compares both distinct retained rows and count 2, while owner full-output comparison remains Planned/UNRUN. |
