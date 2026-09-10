---
id: SR-036
title: "Failure-domain review of issue 64 native temporal TL correspondence"
type: SpecReview
analysis: failure-domain
scope: "FR-025 and TC-038"
review_set: all
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-025
    type: reviews
---
# SR-036: Failure-domain review of issue 64

## Summary

The review probed identity confusion, external execution, invalid and unknown
inputs, incomplete topology, and state-transition boundaries. Three gaps were
closed without adding runtime or evaluator ownership to Contract IR.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-361 | high | **Closed:** equal formula bytes or Boolean outcomes could have reused a projection after a material source/model/profile/capture change. The correspondence reference now names every identity dimension and mutation invalidates the decision. | FR-025 Behavior; FR-025-AC-7/8 | missing-requirement |
| FND-362 | medium | **Closed:** the draft did not explicitly rule out bridge callbacks, ambient lookup, state reads, or evaluator invocation. Inputs are now validated immutable values; unknown profiles fail closed and invalid inputs return existing diagnostics before decision construction. | FR-025 Inputs, Behavior | missing-requirement |
| FND-363 | high | **Closed:** a closed but incomplete observation could have been sent to a closed-trace evaluator and converted into a Boolean by false extension. FR-025 now requires complete closure and otherwise returns unsupported with the affected closure/history dimension. | FR-025 capability table and closure rule; FR-025-AC-6 | missing-requirement |

## Failure-Domain Result

- There is no extension hook or user-supplied callback in the bridge.
- The correspondence key is structural, immutable, and includes source,
  semantic, observation, TL, evaluator, proposition-map, and supported-formula
  identity.
- The bridge performs no user logic; predicate evaluation and TL evaluation
  remain external typed inputs/consumers.
- No graph traversal is introduced. Existing FR-023 aggregate size/depth/count
  limits bound the supplied clause and predicate population.
- Missing issue #63 capability, history, captures, closure completeness, or TL
  profile support is visible and never converted to false, true, or omission.

## Result

**PASS after remediation.** No open failure-domain finding authorizes
implementation before the dependency gates recorded by SR-038.
