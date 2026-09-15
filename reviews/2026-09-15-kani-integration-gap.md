---
id: SR-046
title: "Gap analysis of bounded Kani integration and replay"
type: SpecReview
analysis: gap-analysis
scope: "#86; src/kani/replay.rs; tests/kani_replay.rs; TC-042"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/86
    type: reviews
---

# SR-046: Gap analysis of bounded Kani integration and replay

## Summary

The integration boundary accepts only a concrete counterexample packet whose
profile revision agrees with the retained finite ABI input. It validates that
input before native reconstruction, invokes the independent QSL
`runtime::execute` itself, and accepts replay only for a completed native
`false`. Packet malformation, a native proof, incomplete native execution, and
all native validation failures therefore remain typed non-Boolean outcomes.

The test constructs an admitted native model, checked native package, complete
snapshot, and false invariant via public QSL APIs. It proves the actual native
runtime path and the packet boundary rather than mocking the executor.

## Verdict

**PASS.** The #86 replay acceptance is covered by the native-runtime corpus;
shared profile/matrix and semantic-family refusal behavior remain covered by
the merged #82 through #85 lanes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-461 | low | No gap found: malformed identity, a non-counterexample callback result, and an actual independently executed native false result are separately covered without manufacturing a Boolean for failure. | FR-031-AC-3, TC-042 |

## Evidence

- `cargo fmt --all -- --check`
- `cargo test --locked --target-dir target-codex-backends --test kani_replay`
- `cargo clippy --offline --locked --target-dir target-codex-backends --all-targets --all-features -- -D warnings`
