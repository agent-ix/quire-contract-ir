---
id: SR-610049
title: "IR-1032 PR 612 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@90bbf5f1f317431813b44d1c248473bf2ef041fb; crates/quire-contract-model/src/checked_package/v2/{identity.rs,mod.rs}, tests/it/{checked_package_v2_dimension_equivalence.rs,checked_package_v2_reader.rs}; FR-063-AC-5"
review_set: subset
relationships:
  - type: references
    target: ix://agent-ix/quire-contract-ir/FR-063
---
# SR-610049: IR-1032 PR 612 code and Rust review

## Summary

Ticket: IR-1032. Reviewed PR #612 exact head `90bbf5f1f317431813b44d1c248473bf2ef041fb` against IR FR-063-AC-5 and QSpec FR-322-AC-139 at QSpec `origin/main` `60383a35`. The rust-review lane is folded into this code-review. Examined all four changed files. No vendoring, added dependency, unsafe code, or CI workflow change. `git diff --check` passes. Focused dimension-equivalence tests were reported 9/9 by the author; reviewer did not run Cargo while the shared heavy-build lock is occupied. Owner-version test and Clippy were not yet run at this head.

Examined unit: FR-063-AC-5 (full criterion in `spec/checked_package/functional/FR-063-dimension-equivalence-node.md:94`). Context unit: QSpec FR-322-AC-139 (`quire-specification/spec/objects/interfaces/FR-322-checked-package-artifact.md:2501`). The accepted shape, string-edge checks, and model-owner version routing agree with their named positive and malformed examples. The findings below concern remaining wire member presence and graph refusal location.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | `declaration: null` is a present member forbidden by QSpec's declaration-presence rule, but `Option<CheckedDeclaration>` decodes it as `None` and `check_nominal_wire` tests only `is_some()`. The equivalence node then passes this wire check; with a recomputed package identity it can be admitted, although AC5 requires every present declaration member to refuse `malformed_wire` at the node. Check raw member presence for equivalence nodes before typed decode discards it, and test the null mutation alongside the non-null case. | crates/quire-contract-model/src/checked_package/v2/identity.rs:266-276; mod.rs:1265 | implementation-bug-despite-evidence |
| FND-002 | medium | The schema-valid graph refusals named by AC5 do not have the specified node location. `validate_dimension_equivalence` returns `/nominal_identity_preimage/root_scale` for unreduced ratio, `/dependencies` for a missing dependency, `/semantic_type` for non-self type, and `/nominal_identity_preimage/owner` for an unselected owner; the stale key reports `/node_id`. QSpec FR-322-AC-139 and IR AC5 say these refuse at the equivalence node. Pin the public refusal pointer with tests for each case and align the implementation with the contracted node location. | crates/quire-contract-model/src/checked_package/v2/identity.rs:727-758; identity.rs:668-675 | implementation-bug-despite-evidence |

## Verdict

CONDITIONAL. FND-001 is a wire-contract mismatch. FND-002 needs an exact pointer decision against the normative AC and a public-reader test. The first review pass is complete; no code edits or merge were made by the reviewer.

## Coverage

The new tests assert absent scale, one-entry pair, zero/negative numerator and denominator, non-null declaration, three `-0` sites and model-owner version. The graph half and mixed-defect precedence have no new AC5-tagged regression tests; see SR-610050. Full gates are pending on a reviewed head. Plan completion: not assessed.
