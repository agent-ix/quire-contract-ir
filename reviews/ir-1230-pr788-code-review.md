---
id: SR-202610310154
title: "Code review of IR-1230 temporal clock binding reader PR 788"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1230; PR 788 head c86bccab; crates/quire-contract-model/src/checked_package/v2/intake.rs, mod.rs, model_fields/tests.rs, temporal_binding.rs, crates/quire-contract-model/src/lib.rs, tests/conformance_qspec/main.rs and temporal_clock_bindings.rs; QSpec FR-322-AC-294"
review_set: subset
---

## Summary

The Rust diff was reviewed against QSpec FR-322-AC-294, the repository conventions, and the Rust wire, API, safety, and testing guidance. No applicable AssuranceProfile was found under `spec/`. The typed reader admits the published three clock kinds and reports the tested malformed and semantic refusal loci, but the public writer cannot emit the new required member and the new row/profile scan is not charged to the work limit.

## Verdict

FAIL. Two production findings need fixes on this PR; one source-header finding also applies. The code lead owns the aggregate gate after the fixes are reviewed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The public complete V2 encoder always omits `temporal_bindings`, so it cannot emit a valid package containing a clock-configured temporal clause. | crates/quire-contract-model/src/checked_package/v2/mod.rs:770 |
| FND-002 | medium | Each temporal row scans all profile selections without a `WorkMeter` charge, allowing row-by-profile work beyond the request limit. | crates/quire-contract-model/src/checked_package/v2/temporal_binding.rs:270 |
| FND-003 | low | The new Rust module lacks the SPDX and copyright header required by the Rust style guidance. | crates/quire-contract-model/src/checked_package/v2/temporal_binding.rs:1 |

## Evidence

`encode_checked_package_v2` documents that it encodes a complete checked-package V2 envelope. Its parameters offer no temporal row input and it sets the new envelope field to `None`. For a graph with a clock-configured temporal clause, FR-322-AC-294 requires that member, so this output cannot pass the new strict reader. A round-trip or writer-side refusal test for this case is absent.

`temporal_binding::validate` compares each admitted row with a linear `lock.profile_selections.iter().any()` scan. Neither that scan nor the event-position and timestamped-event row loops charge `meter`; only rational reduction does. Thus R rows and P selections add O(R × P) unmetered work after earlier stages have consumed their budget. The package byte limit bounds input size but does not enforce the separate caller work limit on this new cross product.

The new `temporal_binding.rs` begins with `//!` and has no license header. The Rust style reference requires the repository's SPDX and copyright header on every new source file.

The focused by-reference TCB test passed on this frozen head with `QUIRE_SPECIFICATION_DIR` at current QSpec `6a021ef8` (one test, 162 filtered). The coder reported formatting and workspace Clippy passing. No CI workflow, dependency, vendored artifact, or hash/pin record changed. QSpec #571 added `temporal_bindings` to the configured positive timed fixtures; the prior fixture gap is resolved. The code lead owns the final aggregate gate.

## Scope and bindings

Examined QSpec FR-322-AC-294: clock-configured temporal declarations require exact typed rows, structural declaration keys, selected-profile joins, exact refusals and borrowed read access. QSpec FR-322 temporal clock bindings prose and FR-370 clause arguments were context. Examined every changed file listed in `scope`, the public writer call sites, and the existing work meter. The changed conformance test traces `FR-322-AC-294`, `TC-134`, and `TC-233`; the repository's computed Quire matrix cannot mint the external QSpec criterion, so that binding was checked manually.
