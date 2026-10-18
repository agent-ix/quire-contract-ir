---
id: NFR-006
title: "Bound checked-package V2 producer memory"
type: NFR
quality_attribute: performance_efficiency
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-051
    type: constrains
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---
# NFR-006: Bound checked-package V2 producer memory

## Statement

When a producer encodes a `quire.checked-package/v2` artifact, the Contract IR
identity-projection encoding path SHALL permit it to borrow each semantic graph
node body without retaining a second owned `serde_json::Value` body per node.
When QSL emits with that view, the integrated producer SHALL reduce the median
incremental peak resident memory in the fixed TC-455 emission window by at
least 65,536 KiB (64 MiB) relative to a freshly measured owned-projection
baseline on the same host, while preserving the exact canonical package bytes
and `package_id`.

## Scope

This requirement covers production of the identity preimage and complete V2
envelope from an already checked graph. The performance measurement begins
immediately before QSL's `emit_checked` call and ends when it returns. The
reader's separate FR-038-AC-214 memory ceiling does not measure this path.

The Contract IR encoder supplies the borrowing boundary. The QSL-owned
`emit_checked` path must adopt the borrowed view before the integrated memory
criterion can pass; the available IR API alone does not establish the saving.
Neither a new
compatibility encoding path nor a change to the canonical package identity is
part of this requirement.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Peak incremental resident memory in the TC-455 emission window | At least 65,536 KiB (64 MiB) lower than the fresh owned-projection baseline median | Candidate median ≤ fresh baseline median − 65,536 KiB; the measured control's ceiling is 273,588 KiB (339,124 − 65,536) | Median of three fresh-process runs before and after the change, same fixture, toolchain, release profile, allocator, and host; sample `/proc/self/status` `VmRSS` from just before `emit_checked` until it returns and subtract the starting `VmRSS` |
| Concurrent extra owned copies of each projected graph-node body on the successful producer path | 0 | 0 | Inspect the producer and IR encoder call path; assert no retained `Value::clone` or collected owned identity projection is required to hash and encode |
| Canonical package bytes and `package_id` for the same checked input | Exact equality with the baseline | Any difference fails | Compare complete byte sequences and the typed package identity across the baseline and candidate producer; admit both through the existing V2 reader |

The fixed TC-455 source has 12,500 sum terms, 50,138 source bytes, 12,505
semantic graph nodes, and 40,631,408 emitted bytes. A fresh QSL-main
owned-projection control measured incremental peaks of 339,076, 339,124, and
339,256 KiB (median 339,124 KiB; range 180 KiB). A local, uncommitted QSL
`emit_checked` change using the merged borrowed IR view measured 253,432,
254,692, and 254,796 KiB (median 254,692 KiB; range 1,364 KiB). The observed
median reduction is 84,432 KiB (82.45 MiB, 24.90%), leaving 18,896 KiB below
the 64 MiB target ceiling. All six complete byte sequences and typed
`package_id` values were identical, and the V2 reader admitted each envelope.
Median process high-water RSS was 449,996 KiB for the control and 365,296 KiB
for the candidate.
This local candidate is measurement evidence, not completed QSL adoption.
The trial changed QSL's emitter and preimage call path and advanced its IR
dependency to the merged borrowed API; the measured delta is for that
integrated candidate, not an IR-only microbenchmark.

Median emission time increased from 1,137 ms (owned runs: 1,113, 1,137,
1,157 ms) to 1,544 ms (borrowed runs: 1,625, 1,441, 1,544 ms), a measured
35.8% slowdown. This timing observation is disclosed without an elapsed-time
acceptance bound. The comparison used Rust 1.98.1 in release profile with the
default system allocator (glibc 2.39) on the same x86-64 Intel i7-7700 host,
without custom `RUSTFLAGS` or `MALLOC_ARENA_MAX`. The threshold is relative to
a fresh control on one host, not an absolute cross-machine memory limit. An
earlier owned-only control measured 337,672, 337,700, and 337,736 KiB; the
fresh control above is the one used to derive and evaluate this target.

To remeasure, build one temporary Rust probe against the QSL checkout's public
`qsl-cst`, `qsl-forms`, `qsl-foundation`, `qsl-semantics`, and `qsl-package`
crates, or as an uncommitted `qsl-package` example. Implement the exact source
construction, pre-emission staging, and 1 ms RSS sampler in TC-455. Build the
same probe source in release profile for both the owned QSL-main control and
the candidate QSL emit path, using each checkout's lockfile. Record the compiler
version and use the same compiler, host, and default allocator for all six
fresh processes, with no custom `RUSTFLAGS` or `MALLOC_ARENA_MAX`. Run the
three processes while other large builds are idle. For each, record the probe's
source and graph counts, emitted size, typed identity, starting and sampled
peak `VmRSS`, process high-water RSS, and emission elapsed time; compare all
complete output files byte-for-byte. Keep the byte files as temporary
comparison oracles and discard them after evaluation.

## Verification

[TC-455](../matrix/TC-455-v2-producer-memory-and-identity.md) measures the
integrated producer in a fresh process and checks exact byte and identity
preservation. A diff-scoped implementation review verifies the borrowing path
and that no second owned body remains live per emitted node.
