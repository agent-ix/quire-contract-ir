---
id: SR-5901
title: "IR-575 PR 340 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir PR 340 at e52e94d985f0b9abf6651022d2891c64ebe3b310; FR-038-AC-214 and changed tests/source"
review_set: subset
---

## Summary

The computed matrix tags FR-038-AC-214 to the explicit memory target and the new canonical control. The memory target exercises a graph-heavy fixture, leaving the no-DOM claim unguarded for large non-node closed fields.

## Verdict

**CONDITIONAL** for the scoped requirement-to-test-to-code chain. Plan completion: not assessed. The repository-wide computed matrix has inherited unbacked criteria; this artifact assesses the PR diff only.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | The sole AC-214 memory fixture concentrates size in `semantic_graph.nodes` and projection, exactly the arrays emptied before the new root `Value` comparison; it cannot fail when a large lock, capability report, or selection list is copied into two root trees. | FR-038-AC-214; `tests/ir575_reader_memory.rs:72-81`; `v2/intake.rs:280-299` | correct-requirement-no-evidence |

## Trace and evidence

`quoin matrix --repo . --json` at e52e94d reports AC-214 statically tagged by two ignored explicit-target functions in `tests/ir575_reader_memory.rs` and one ordinary canonical-spelling test in `tests/it/checked_package_v2_canonical_encoding.rs`. The ignored target was run manually as the criterion specifies; the matrix has no bound run evidence. The separate-process log establishes the graph-heavy fixture peak only. The `test = false` Cargo target is deliberate and not itself a defect.

The direct typed reader is specified by FR-038-AC-214 and FR-038's Reading section, so the changed production code has an owner. The unrelated inherited strict-coverage deficit is outside this diff verdict. The needed test should exercise a large non-omitted closed field, keep an independent memory or allocation oracle, and still check the closed-wire refusal or admission outcome. This finding is the missing control for code-review SR-5900 FND-001; it does not allege the recorded three peaks are false.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 598538f79f520d384bfc39c698b8c9e62d58294b |

`tests/ir575_reader_memory.rs` now generates a separate 32,045,214-byte fixture with 500,001 `capability_report` items and measures `CheckedPackageV2::read` in a fresh process after loading bytes and evidence. It checks the exact `UnknownRequiredCapability` refusal at `/capability_report/1/feature`, valid baseline HWM, and a 786,432 KiB incremental peak ceiling. The recorded new-reader peak is 37,364 KiB; the old reader failed at 810,652 KiB. The node-heavy control remains present and recorded 100,022 admitted nodes at 695,712 KiB additional peak against AC-214's 1,572,864 KiB ceiling. Logs predate the rebase without naming a SHA; the rebase from `4a29d82` to the reviewed local head changed only spec files, leaving the test and production Rust files byte-identical. This disposition did not rerun gates.
