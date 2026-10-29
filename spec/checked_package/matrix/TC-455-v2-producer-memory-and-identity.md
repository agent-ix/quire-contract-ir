---
id: TC-455
title: "Checked-package V2 producer memory and identity"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/NFR-006
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-051
    type: references
---
# TC-455: Checked-package V2 producer memory and identity

## Description

Measure the integrated QSL producer's V2 emission memory on a fixed checked
source, and verify that removing retained projection-body copies leaves the
canonical envelope and `package_id` exactly unchanged. The Contract IR API
portion alone does not satisfy the integrated memory result until QSL adopts it.

## Test Procedure

In a fresh process, construct the exact UTF-8 source as the prefix
`language "ix:native" edition "1-draft";\nprofile v = "quire.value.complete/v1";\nfunction f using v(a: Boolean, x: Integer): Integer pure { `,
followed by `x + ` repeated 12,499 times and the suffix `x }\n`.
Use fixed source identity (`a`, `u`, `git`, `1`) and unit name `unit.native`.
Set parser source-byte and token limits to at least the source byte length,
and its node limit to at least sixteen times that length. Check with
`CheckingLimits::new(u64::MAX).with_input_bytes(u64::MAX).with_work_budget(u64::MAX)`;
assemble with no selected models or imports. Parse, build forms, assemble,
check, and link the graph before the measurement window. Record source bytes
and graph-node count.

Immediately before `emit_checked`, record `/proc/self/status` `VmRSS` and start
a 1 ms RSS sampler. Stop it when emission returns. Record the maximum sampled
RSS minus the starting RSS, the process high-water RSS, emitted byte length,
elapsed emission time, complete emitted bytes, and typed `package_id`. Run the
baseline and changed producer three times each as separate release-profile
processes on the same host, allocator and Rust toolchain. Compare the medians
of incremental peak RSS and report the median emission time for each path;
time is an observation, not a gated result. Keep source generation and startup
outside the measurement window. Use the baseline's exact bytes only as a
temporary comparison oracle, not as a repository copy of another artifact.

For the candidate path, compare the complete canonical bytes byte-for-byte
with baseline output and compare the typed `package_id`. Read both envelopes
with the existing V2 reader and the same valid evidence. Inspect the IR and
QSL producer paths for a retained per-node `Value::clone` or a collected owned
identity projection. Exercise one body with nested values and a canonical
encoding refusal at the IR encoder boundary to ensure the borrowing path
retains the existing error and no-partial-bytes behavior.

## Expected Results

All six fresh processes emit the same canonical bytes and `package_id`; both
versions' bytes are reader-admitted. Once QSL adopts the borrowed view, the
changed producer's median incremental peak RSS is at least 65,536 KiB (64 MiB)
below the freshly remeasured owned-projection baseline median on the same host.
The fresh control's median is 339,124 KiB, giving this measurement a candidate
ceiling of 273,588 KiB. The local uncommitted candidate's median is 254,692 KiB,
an 84,432 KiB (24.90%) reduction; this demonstrates the target with the trial
patch but does not complete QSL-owned adoption. The candidate's median emission
time was 1,544 ms versus the baseline's 1,137 ms, a 35.8% slowdown disclosed
without a time bound.
The fixed source emits 40,631,408 bytes. Independently recompute its typed
`package_id` from the emitted canonical identity preimage and compare it with
each process's published value; no fixed expected digest is retained.
No second owned body is retained per projected node, and canonical refusal
returns no bytes. A fresh baseline measurement is made before the candidate's
result is evaluated.
