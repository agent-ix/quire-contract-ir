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
of incremental peak RSS. Keep source generation and startup outside the
measurement window. Use the baseline's exact bytes only as a temporary
comparison oracle, not as a repository copy of another artifact.

For the candidate path, compare the complete canonical bytes byte-for-byte
with baseline output and compare the typed `package_id`. Read both envelopes
with the existing V2 reader and the same valid evidence. Inspect the IR and
QSL producer paths for a retained per-node `Value::clone` or a collected owned
identity projection. Exercise one body with nested values and a canonical
encoding refusal at the IR encoder boundary to ensure the borrowing path
retains the existing error and no-partial-bytes behavior.

## Expected Results

All six fresh processes emit the same canonical bytes and `package_id`; both
versions' bytes are reader-admitted. The changed producer's median incremental
peak RSS does not exceed the freshly remeasured baseline median plus that
baseline's three-run range. The current control's median is 337,700 KiB and
range is 64 KiB, yielding a 337,764 KiB no-regression bound on this host.
The reduction target remains **PLANNED** until the changed path is measured.
The fixed source emits 40,631,408 bytes with typed `package_id`
`ecc79a978fb59047c27e588cb17ccce1d0385eaa8427817cc4591fe1e0dc079a`.
No second owned body is retained per projected node, and canonical refusal
returns no bytes. The recorded baseline measurement is reproduced before the
candidate's result is evaluated.
