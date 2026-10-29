---
id: SR-16271
title: "IR-984 PR 528 code review with Rust lane"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/{shared.rs,v2/{contract_package.rs,derived_keys.rs,flat_wire.rs,hybrid.rs,identity.rs,lower.rs,mod.rs,operations.rs,structural.rs,temporal.rs,vocabulary.rs}}, tests/conformance_qspec/{main.rs,hybrid.rs}, tests/it/checked_package_v2_temporal.rs; FR-038-AC-311..317; QSpec FR-481/TC-966"
review_set: subset
---
# SR-16271: IR-984 PR 528 code review with Rust lane

## Summary

Independent diff-scoped code and Rust review of IR-984 PR #528 the exact head above. QSpec FR-481 and TC-966 at #415 govern the hybrid reader. The focused conformance test passed 16/16; focused library Clippy and rustfmt passed. The production reader has two material defects below.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Capture matching scans every parameter in the package, so an unrelated same-name/same-level parameter of another type refuses a valid claim | crates/quire-contract-model/src/checked_package/v2/hybrid.rs:797-837; FR-038-AC-316 |
| FND-002 | medium | Activation and capture matching rescan the full graph without work charges, allowing quadratic reader work under a small work limit | crates/quire-contract-model/src/checked_package/v2/hybrid.rs:771-805,830-837; CheckedPackageReadLimits::work |

## Finding Detail

- FND-001: For each capture, the implementation inserts all graph parameters with the same name and level into `allowed`, then rejects if *any* such graph parameter has a scalar base type different from the initializer. A second independent claim can carry a same-named capture at the same level with a different type; that unrelated node makes this claim refuse, while the QSpec check is about this claim's capture. The lookup needs an association to the claim's actual capture node or read closure, not every package node.
- FND-002: `validate_claims` charges once per temporal node, yet each model variable and each capture runs a full `index` scan, and capture type checking repeats it. With up to 10,000 nodes under the default node cap and many claims or captures, work grows as claims × variables/captures × all nodes without charging those visits. Reuse an index or charge the visits so `work` remains a meaningful bound.

## Coverage and Gate Evidence

- Focused: `QUIRE_SPECIFICATION_DIR=/home/peter/dev/worktrees/ir984-qspec414 cargo test --locked --test conformance_qspec qspec_hybrid -- --nocapture`: 16 passed, 0 failed.
- Focused: `cargo clippy --locked -p quire-contract-model --lib -- -D warnings`: passed. `cargo fmt --all --check` and `git diff --check`: passed.
- Full aggregate gate: pending review and held by author; not run in this review.

## Scope

- All 15 changed files listed in frontmatter, examined for wire admission, identity, temporal ordering, model and claim checks, public type additions and tests.
- FR-038-AC-311 through FR-038-AC-317 and QSpec FR-481/TC-966, examined as behavioral authority.
- No CI workflow change, copied QSpec fixture, or new dependency was in the diff.

## Verdict

CONDITIONAL while medium findings remain open.

## Dispositions

At PR #528, same-reviewer round 1 inspected the fix diff and reran the focused QSpec hybrid tests: 20 passed, 0 failed.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Claim reads now match the referenced parameter's name, level, and scalar base type against this claim's permitted bindings; the unrelated same-name/level parameter admission test passes. |
| FND-002 | fixed | The full-index scans were removed. Read traversal, type hops, and called-function parameter checks charge the work meter. |

## Final head confirmation

PR #528 adds a metered traversal rooted in this claim's formula and capture initializers. It checks reachable capture parameter types at the capture binding before the fairness step. The combined capture mismatch and fairness defect test asserts `IllTyped`/`TypeMismatch` at `/body/arguments/3/members/0`. This preserves the round-1 FND-001/002 fixes. Focused QSpec hybrid tests pass 20/20; `git diff --check` is clean. No new finding in this delta. Code/Rust review: PASS at this head. The author retains the final aggregate gate.

## Gate-fix head confirmation

PR #528 changes only six files over the reviewed custody. Hybrid claim work is charged after classification, restoring TC-226's exact work boundary. The flat-wire timed-null exception is limited to raw formula references reachable from a hybrid claim; a nonhybrid formula still refuses at its upper bound. `HybridForm` is exported from the model root and matched by FR-019's implemented inventory. The TC-226 boundary, TC-048 timed-bound, TC-058 inventory and 21 QSpec hybrid focused tests all pass. `git diff --check` is clean. No new code or Rust finding; PASS at this head. The final aggregate gate remains author-owned.
