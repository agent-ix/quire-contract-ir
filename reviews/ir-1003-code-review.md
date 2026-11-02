---
id: SR-2026110101300
title: "IR-1003 PR 831 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; tests/it/checked_package_v2_refinement.rs and tests/conformance_qspec/timed_model.rs; FR-061-AC-6 and FR-038-AC-342"
review_set: subset
---

## Summary

Ticket: IR-1003. Scoped independent review of the two changed test files against the two complete criteria. Production, specifications, dependencies and CI workflows are unchanged. The Rust lane is folded into code-review. No applicable AssuranceProfile was found.

## Verdict

**CONDITIONAL**. The timed aggregate matrix is covered, but the refinement dependency criterion still has uncovered collection branches. The measured two-row binding reduction is mechanically real; it is not yet evidence that both entire criteria are discharged.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-061-AC-6 is not completely discharged: no tagged dependency test isolates local ensure operation keys or protocol_step target and argument dependencies. Removing the ensure loop or ProtocolStep dependency arm can survive the new matrix because those paths are absent or their keys are supplied by other rows. | tests/it/checked_package_v2_refinement.rs:2096; crates/quire-contract-model/src/checked_package/v2/refinement.rs:3783 |

## Coverage

Plan completion: not assessed

- FR-061-AC-6 (role: examined), spec/checked_package/functional/FR-061-admit-refinement-nodes.md: The exact local dependency set includes body header keys and every direct row declaration/type and expression/member target, excludes external keys and annotations, and is structurally ordered. An omitted, extra or misordered dependency refuses `invalid_semantic_graph` at `/dependencies` before any row-meaning defect.

- FR-038-AC-342 (role: examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: PLANNED/UNRUN (IR-1096; amended by IR-1109). The public V2 reader admits QSpec's selected `AO-01` timed-slot package and `AO-05` tie-on-start package: all six root reference aggregates ascend by each node's minimum source-region triple, followed on a tie by FR-322-AC-265's structural member key. It refuses otherwise-valid `AO-02`, `AO-03`, `AO-04`, `AO-06` and `AO-07` as `invalid_semantic_graph` with no cause at the first out-of-order member locus; structural id order cannot admit `AO-03`, tied starts use ends in `AO-06`, and `AO-07` uses the earlier region of the additional source-map entry. `AO-08` admits tied-region places in path order; `AO-09` and `AO-10` refuse reversed or equal structural place keys at the second member before a duplicate-path check. For each root `resets`, `guards`, `invariants` and `constraints` branch, an admitted two-distinct-node aggregate and its otherwise-valid reversed aggregate prove admission and `invalid_semantic_graph` at the exact second-member locus; two distinct nodes with a tied minimum region admit in structural-key order and refuse when reversed, while two references with equal full keys refuse at the second member. An ordered repeated place path reaches `malformed-clock-place` at its forged row after aggregate order; `duplicate-clock-place` remains the post-row-join rejection, unreachable for a conforming second place. A root missing a guard reaches the unlisted-guard refusal. For every `AO-02`, `AO-03`, `AO-04`, `AO-06` and `AO-07`, malformed wire, missing dependency and duplicate structural id each refuse at their exact earlier-stage locus while its order defect remains.

The dependency helper checks admitted controls, preservation through the public writer and reader, every omitted member of each authored dependency union, unrelated local additions, duplicate entries, adjacent swaps, and dependency-before-row refusal. History references, application references/member declarations/populations, excluded literal/result annotations and concrete semantic type, local protocol/catalog keys and external selected keys were inspected. The six traced fixtures do not exercise dependency-only local ensure keys or protocol_step argument references. The operation expression control uses the existing source anchor as its target, so it also does not independently isolate that collection path.

For FR-038-AC-342, inspected all AO-01 through AO-10 selected cases, every listed earlier-stage defect against five adverse AO vectors, source identity/start/end/multiple-entry ordering, place ties and duplicate keys, all four remaining root clause branches (source-order, structural ties and repetition), forged place-row and unlisted-guard controls. The stale PLANNED/UNRUN label is existing spec-owner work routed outside this PR, not a new defect or an invitation to edit the spec.

Read the existing coverage receipt: unbacked records 94 to 92, contradictions zero, exactly these two row bindings removed and no additions. The 92 residual records are outside this review and are not new findings. No global completeness claim is made. Existing focused logs show corrected refinement and timed cases passing; earlier failed development attempts also remain in those logs. No additional heavy builds were run. Final hosted Test and local conformance/compile/corpus/spec gates remain the lead's responsibility. Semantic intent review was explicitly requested for these two criteria; no broader semantic audit ran.
