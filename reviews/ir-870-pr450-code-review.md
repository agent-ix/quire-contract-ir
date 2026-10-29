---
id: SR-12430
title: "IR-870 code and Rust review of IR PR 450"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir PR 450; five changed Rust test files"
review_set: subset
---

# IR-870 code and Rust review of IR PR 450

## Summary

The PR replaces seven doc-only Trace citations with same-test `ix_trace_rs::trace` attributes and removes 23 doc citations already duplicated by attributes. The exact-base audit found that every removed ID remains in the resulting attribute on the same function. Executable test bodies are unchanged.

## Reviewed scope

I reviewed `tests/conformance_qsl_ir769/main.rs`, `tests/conformance_qspec/main.rs`, `tests/ir575_reader_memory.rs`, `tests/it/checked_package_evidence_insertion.rs`, and `tests/it/checked_package_v2_enum_order.rs`, their 30 changed test declarations, repository Rust conventions, and the exact-base/head Quire traceability result. The changed functions are tests, including already ignored memory and conformance cases; none is a Kani proof. No CI workflow or production code changed.

## Verdict

**CONDITIONAL** — the trace migration preserves IDs and coverage, but six changed doc blocks retain an empty final `///` line.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Six edited doc blocks end with a blank `///` directly above `#[trace]`; remove those now-orphaned doc lines so the prose ends at its last substantive line. | tests/ir575_reader_memory.rs:35, tests/conformance_qspec/main.rs:1043 |

## Evidence

- A function-level script found exactly 30 removed Trace/Tracing doc lines: 7 conversions and 23 redundant removals. For every removed line, its ID set is a subset of the resulting same-function `#[trace]` ID set; no ID is lost.
- `git diff --check` and `cargo fmt --all -- --check` pass. No new `/// Trace:` or `/// Tracing:` line appears.
- Independent `quire coverage --scope . --format json` runs at exact and both report 527/584 backed, 40 unbacked and 0 status lies. All 550 obligations match exactly. The computed matrix and unmatched tags match after line and column locations are removed.
- The six empty doc lines are at `tests/ir575_reader_memory.rs:35,63,127,153` and `tests/conformance_qspec/main.rs:1043,1085`. Each sits directly above a migrated or retained trace attribute.
- The author reports the pre-PR gates; this read-only review ran focused checks and independent Quire measurements without repeating the full Cargo suite.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | deletes exactly the six cited empty `///` lines. The remaining substantive prose and trace attributes are unchanged; a scan of both files finds no empty `///` directly above `#[trace]`. |
