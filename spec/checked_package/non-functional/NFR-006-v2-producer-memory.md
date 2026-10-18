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
The integrated producer SHALL stay within the measured no-regression bound on
the fixed TC-455 workload while preserving the exact canonical package bytes
and `package_id`. A numerical reduction target SHALL be set from the changed
producer's measurement before the optimization is claimed complete.

## Scope

This requirement covers production of the identity preimage and complete V2
envelope from an already checked graph. The performance measurement begins
immediately before QSL's `emit_checked` call and ends when it returns. The
reader's separate FR-038-AC-214 memory ceiling does not measure this path.

The Contract IR encoder supplies the borrowing boundary; the QSL producer must
adopt it before the integrated memory criterion can pass. Neither a new
compatibility encoding path nor a change to the canonical package identity is
part of this requirement.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Peak incremental resident memory in the TC-455 emission window | **PLANNED:** reduction target to be set from the slice 2 borrowing-path measurement | Candidate median no greater than the freshly remeasured baseline median plus that baseline's three-run range (337,764 KiB against this control: 337,700 + 64 KiB) | Median of three fresh-process runs before and after the change, same fixture, toolchain, release profile, allocator, and host; sample `/proc/self/status` `VmRSS` from just before `emit_checked` until it returns and subtract the starting `VmRSS` |
| Concurrent extra owned copies of each projected graph-node body on the successful producer path | 0 | 0 | Inspect the producer and IR encoder call path; assert no retained `Value::clone` or collected owned identity projection is required to hash and encode |
| Canonical package bytes and `package_id` for the same checked input | Exact equality with the baseline | Any difference fails | Compare complete byte sequences and the typed package identity across the baseline and candidate producer; admit both through the existing V2 reader |

The baseline is the unoptimized QSL producer measured by TC-455 with 12,500
sum terms: 50,138 source bytes, 12,505 semantic graph nodes, and 40,631,408
emitted bytes. Three fresh-process emission-window peak increments were
337,672, 337,700, and 337,736 KiB (median 337,700 KiB). Median process
high-water RSS was 447,976 KiB. The three complete byte sequences and typed
`package_id` were identical. This control used Rust 1.98.1 in release profile
with the default system allocator (glibc 2.39) on an x86-64 Intel i7-7700
host. The comparison threshold uses fresh runs on one host and is not an
absolute cross-machine memory limit. The 64 KiB tolerance is the observed
baseline range, not an estimated optimization benefit. The numerical
reduction target remains planned until the borrowed path has been measured.

To remeasure, build a fresh external Cargo probe against the QSL checkout's
public `qsl-cst`, `qsl-forms`, `qsl-foundation`, `qsl-semantics`, and
`qsl-package` crates. Its input construction and RSS sampler are specified
in TC-455. Record the compiler version used and use that same compiler for
the fresh baseline and candidate runs. Use the release profile, the default
system allocator, no custom `RUSTFLAGS` or `MALLOC_ARENA_MAX`, and the same
host for both sets of runs. Run:

```sh
export CARGO_TARGET_DIR=/home/peter/dev/worktrees/ir875-qsl-readonly/target
export CARGO_BUILD_JOBS=2
IR875_PROBE_DIR=/tmp/ir875-probe
cargo build --release --locked --manifest-path "$IR875_PROBE_DIR/Cargo.toml"
for run in 1 2 3; do
  /usr/bin/time -v -o "$IR875_PROBE_DIR/time-${run}.txt" "$CARGO_TARGET_DIR/release/ir875-probe" 12500 "$IR875_PROBE_DIR/bytes-${run}.json" 2> "$IR875_PROBE_DIR/run-${run}.txt"
done
cmp "$IR875_PROBE_DIR/bytes-1.json" "$IR875_PROBE_DIR/bytes-2.json"
cmp "$IR875_PROBE_DIR/bytes-1.json" "$IR875_PROBE_DIR/bytes-3.json"
```

Replace the two absolute directory assignments for another host. Run the
three processes while other large builds are idle, as in the baseline run.
The probe prints source bytes, semantic node count, emitted byte length,
typed `package_id`, starting and sampled peak `VmRSS`, and emission elapsed
time. The three output files are compared with `cmp`. The baseline and
candidate use the same probe source and dependencies; only the producer path
under test changes. The temporary byte files are discarded after comparison.

## Verification

[TC-455](../matrix/TC-455-v2-producer-memory-and-identity.md) measures the
integrated producer in a fresh process and checks exact byte and identity
preservation. A diff-scoped implementation review verifies the borrowing path
and that no second owned body remains live per emitted node.
