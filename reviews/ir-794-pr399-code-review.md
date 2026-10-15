---
id: SR-11127
title: "Code and Rust review — IR-794 neutral owner outcomes"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/source_obligation.rs, tests/it/source_obligation.rs, tests/it/public_interface.rs, spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Reviewed PR #399 at 0248e68 against FR-052-AC-9..11, TC-452 and FR-019. The neutral retention code validates owner/scope, row provenance, typed stop owner and observed count without claiming external owner authentication. No source defect found.

## Examined scope

- FR-052-AC-9 (examined): For each owner, the native enumerator or typed adapter issues a `Complete` outcome only after the selected named scope reaches its normal end; the outcome retains the selected source, typed owner and scope, every owner-emitted row and the observed count. A complete zero-row scope and a fully parsed module with zero functions remain complete, while an absent owner report remains absent. Deleting one row after the owner's full output is observed makes the owner-output comparison fail even when the remaining row count is changed to match.
- FR-052-AC-10 (examined): For each owner, stopping before normal exhaustion issues `Incomplete` with retained partial rows and the typed cause the owner actually reports. QSL type-work exhaustion retains dimension, limit, prior, requested charge and site; QSL `TypeDisposition::Unfinished` retains its declaration handle and is distinct. An IR V2 reader limit retains the `CheckedPackageIncomplete` fields. For an owner path that reports `quire_exact::CancelCause`, requested and deadline cancellation remain distinct through `SourceInventoryCancelCause`; the current QSL inventory and `quire-code-rs` adapter do not claim cancellation support. `stop_reason: None` on a provisional envelope cannot be promoted to `Complete`, and a new owner stop type cannot be projected into a string or generic early stop.
- FR-052-AC-11 (examined): The neutral model retains an owner-issued outcome only when its source, owner, scope and observed count match the retained rows; a mismatch is typed and refuses the outcome. Owner adapter tests show that no adapter completion branch accepts an arbitrary caller-supplied row vector/count/completion flag in place of enumerating the selected native owner output. The model does not claim independent authentication of external owner exhaustion or a completed three-owner census.
- TC-452 (context_only): Full owner-output comparison remains UNRUN.
- FR-019 public inventory (examined): Neutral public exports are listed.

## Verdict

PASS — no code or Rust finding in the changed implementation. The separate gap-analysis artifact records overstated test trace credit; this verdict does not qualify owner adapters or the full gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
