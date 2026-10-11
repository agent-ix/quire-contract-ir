---
id: SR-8041
title: "Gap analysis of IR-491 residual leaf source evidence"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@a5f0d9c486b86185e0866d159e35b532d3911ffd; FR-038-AC-180, FR-038-AC-228 through FR-038-AC-231; crates/quire-contract-model/src/checked_package/v2/, tests/it/checked_package_v2_recursive_leaves.rs, tests/it/checked_package_v2_composite_operands.rs; IR-491"
review_set: subset
---

## Summary

Planless audit of the changed residual operation leaf evidence. The computed strict matrix reports 54 repository-wide untagged criteria. The changed five criteria have tags, but those tags conceal four public-reader branches that the stated acceptance controls require.

## Verdict

**FAIL repository-wide; CONDITIONAL for the changed IR-491 slice** — Four medium evidence gaps remain in this slice. The repository-wide strict matrix fails on 54 untagged criteria outside this slice; plan completion was not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-228 public controls cover only eq with integer and ne with text for value/type references; no type-change control proves fresh leaf selection | tests/it/checked_package_v2_recursive_leaves.rs:409 |
| FND-002 | medium | AC-229's family-fitting unresolved compared/inner type and exact/one-less structural work controls exist only at a private helper seam, with no public-reader evidence | tests/it/checked_package_v2_recursive_leaves.rs:999 |
| FND-003 | medium | AC-230's unresolved set-like result element and structural work controls likewise have no public-reader test | tests/it/checked_package_v2_recursive_leaves.rs:685 |
| FND-004 | medium | AC-231's collection.contains IEEE case tests empty leaves only, and lacks its independent graph/identity admission control | tests/it/checked_package_v2_recursive_leaves.rs:958 |

## Coverage

- Reconciliation: `quire matrix --scope /home/peter/dev/worktrees/ir491-residual-leaf-sources --strict` exited 1 with 54 repository-wide untagged criteria and zero criteria tagged only by ignored tests. Tag presence for the changed five criteria was checked against the changed test bodies.
- Plan completion: not assessed
- Examined: FR-038-AC-180 and FR-038-AC-228 through FR-038-AC-231. AC-180 has the fifteen public forms, three private IEEE type-walk controls and public IEEE refusals; no gap found in the changed slice.
- Reverse gap and stubs: the changed production branches are owned by FR-038 and contain no placeholder implementation.
- Semantic review: scoped acceptance-to-test inspection of these five criteria, as requested in the review brief.

## Finding scenarios

FND-001: A regression could preserve the existing eq/integer and ne/text fixtures while returning stale leaves after a declared type changes. The criterion and TC-048 ask for both identities and declared type selection controls.

FND-002: The new model-unit test synthesizes `None` at the private leaf helper. It cannot establish that a reader-admissible, family-fitting operand reaches the same refusal or that budget exhaustion keeps its typed Work result.

FND-003: The new model-unit test similarly synthesizes an unresolved result element. The public set-like tests exercise only a resolved element and do not measure structural work precedence.

FND-004: `collection.contains` over IEEE runs with `leaves: []` only. A regression that handles the empty case but returns an `operation-law-mismatch` for a supplied leaf would pass. Other shapes have a graph-only control, but this case does not.

## Examined criteria

- FR-038-AC-180 (examined): Implementation exists in `composite_application_operands`; the corrected 18-form code evidence is PLANNED/UNRUN (IR-491), and the prior 18-form public-admission claim does not discharge it. Tuple, direct Option and optional-presence record edges preserve the specified ordinals without wrapper segments; List with optional tail has Depth key [] and reentry [1,0]. Tree with two optional self-fields retains both [0,0] and [1,0] at Depth key []; shared nonrecursive siblings retain distinct paths without Depth. Boolean introduces no position. Enum retains ordered flag and exact preimage member order, including unordered Blue, Green, Red. Each of the eleven scalar and seven bounded-domain forms has its specified type-walk descriptor or absence, including one Whole at a `text_bounds` wrapper and a Node-keyed Whole without Population key for population-typed leaves. A public reader-admitted comparison over each of the fifteen non-IEEE forms returns its independently enumerated introduced-key set; isolated private type-walk unit controls over `float32`, `float64` and `float_rounding` return their tabled descriptors. Public structural comparisons reaching any of those three forms instead refuse `ill_typed`/`operator-ineligible` at `operation.leaves` under AC-231; no public accessor result is claimed for them.
- FR-038-AC-228 (examined): PLANNED/UNRUN (IR-491). Through `CheckedPackageV2::read`, otherwise valid `structural.eq` and `structural.ne` packages whose first operand is, separately, a reference to a structural value, a reference directly to its type node, a reference to an expression node with an application at its body root whose `result_type` equals that node's `semantic_type`, or an option-typed `none` literal, each derive the leaves of the selected declared type. An all-integer record admits with `leaves: []`, and a record with one pinned text field admits with exactly that field's selected-law and mode leaf, while removing the required leaf refuses `invalid_package`/`operation-law-missing` at `operation.leaves`. Replacing the first operand's reference with an inline `application` refuses `malformed_wire` at `arguments/0` before leaf checking. Each package's operand and result identities are independently re-derived; changing a declared type does not silently retain the original comparison.
- FR-038-AC-229 (examined): PLANNED/UNRUN (IR-491). For `structural.eq`, `structural.ne` and `collection.contains`, a first operand replaced with a `binding`, `aggregate`, `frame` or `abstraction_relation` never admits by skipping type resolution: after any earlier applicable grammar/reference check, it refuses `ill_typed`/`operator-ineligible` at `arguments/0` if it reaches the operand-family check. A `dependency_reference` in that position is refused by the earlier dependency-reference placement check. Separately, a first operand whose family fits but whose required compared or `inner:0` type remains unresolved after earlier checks refuses `ill_typed`/`operator-ineligible` at `operation.leaves`, with `leaves` empty or nonempty; exhausted structural-resolution work remains `incomplete`/`work`.
- FR-038-AC-230 (examined): PLANNED/UNRUN (IR-491). For every catalogued `result_inner` identity (`collection.set`, `.bag`, `.ordered_set`, `.map`, `.flat_map`, `.flatten`, `.convert`), a result of `set`, `bag` or `ordered_set` over pinned text admits exactly the selected element-type leaf at the empty path and refuses an omitted one as `operation-law-missing` at `operation.leaves`; where the catalogued result form permits a valid `sequence` result over that text, it admits with `leaves: []` and refuses one supplied leaf as `operation-law-mismatch` at `operation.leaves/0`. A set-like result whose required element type remains unresolved after earlier checks refuses `ill_typed`/`operator-ineligible` at `operation.leaves` rather than admitting either leaf list; exhausted structural-resolution work remains typed Work. Each positive control admits through the public reader with current catalog and lock selections.
- FR-038-AC-231 (examined): PLANNED/UNRUN (IR-491). An otherwise valid `structural.eq` or `structural.ne` over a record, tuple, option or sequence whose selected compared type reaches Float32 or Float64 at any depth, and `collection.contains` over a sequence whose element type does, each refuse `ill_typed`/`operator-ineligible` at `operation.leaves` with `leaves: []` and with a supplied text leaf; changing only that nested IEEE type to an integer type and re-deriving identities admits with its exact empty leaf list. A valid `collection.flatten` result of `sequence<Float32>` or `sequence<Float64>` admits with no leaves because `result_inner` selects no compared type for a sequence result. A `set`, `bag` or `ordered_set` of an IEEE-bearing element is not a positive control: its element type is ineligible under QSpec FR-144. Each adverse comparison is first independently shown to pass strict wire, graph and identity admission without the comparison, so an earlier malformed package cannot satisfy the refusal.

## Dispositions — round 1

Reviewed fix revision: `b078e3f6647721530fb4363c09e83dbccd4f3a35` against the original findings at `a5f0d9c486b86185e0866d159e35b532d3911ffd`. Only `tests/it/checked_package_v2_recursive_leaves.rs` changed in this round. The focused receipt at `/tmp/ir491-review-fixes-focused.log` reports 15 residual tests and the one structural-forwarding test passed; no full gate was rerun for this disposition.

| FND | Outcome | Evidence |
| --- | --- | --- |
| FND-001 | fixed b078e3f6647721530fb4363c09e83dbccd4f3a35 | The public reader now admits both `structural.eq` and `.ne` over integer and pinned-text records using value and direct-type references, and refuses omitted text leaves for both sources. Different declared record and comparison keys are asserted. |
| FND-002 | fixed b078e3f6647721530fb4363c09e83dbccd4f3a35 | New public reads reach `operation.leaves` with unresolved `operand:0` and `inner:0` types for empty and supplied leaves. The tagged forwarding reader test measures exact and one-less work bounds and a Work refusal at the operation locus. |
| FND-003 | fixed b078e3f6647721530fb4363c09e83dbccd4f3a35 | A public `collection.set` read reaches `operation.leaves` for its unresolved result element with empty and supplied leaves. The tagged forwarding reader test covers `result_inner` exact and one-less Work. |
| FND-004 | fixed b078e3f6647721530fb4363c09e83dbccd4f3a35 | `collection.contains` now checks both empty and supplied text leaves for Float32 and Float64, admits an integer control, and separately admits each IEEE-bearing graph without the comparison. |

No new finding arose from this fix diff. The original repository-wide strict-matrix result and review verdict above describe the first review revision; this disposition does not claim a new repository-wide matrix or full-gate result.
