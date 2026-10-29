---
id: SR-567875
title: "IR-874 PR 567 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/flat_wire.rs; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs; tests/it/checked_package_v2_protocol_body.rs"
review_set: subset
---

## Summary

Ticket: IR-874. Planless, diff-scoped traceability and reverse-gap audit of PR #567. Plan completion was not assessed. The new source has FR-038-AC-283/284 ownership. New public-reader tests carry TC-048, with no claim that either full AC is discharged.

## Examined units

- **FR-038-AC-283** (`examined`): "Role, channel, finish, terminal, control and compensation declarations use the fixed qualified-name segments of QSpec FR-464; queues have no declaration and generated occurrences, while other protocol members have declaration occurrences. A wrong path or occurrence role refuses at the node."
- **FR-038-AC-284** (`examined`): "The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending node-id digest."
- **TC-048** (`examined`): "Use QSpec FR-464, FR-465 and FR-466 and the authored tables in TC-946, TC-947 and TC-948 as the current independent expected outcomes."

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- Plan completion: not assessed.
- Matrix: quire coverage reports 68 unbacked rows and 0 status lies; the new tests bind TC-048 only; full FR-038-AC-283 and AC-284 remain open, with no new AC tags or completion claim.
- Reverse gap: declaration paths and queue wire distinctions are owned by FR-038-AC-283; ordered stage is owned by FR-038-AC-284.
- Stub/coverage inflation: none found in the changed code. Independent test-oracle deficit is in SR-567874.
- Optional semantic review: not run as a separate repository-wide audit; changed behavior was inspected in SR-567874.

## Verdict

PASS for this diff's trace and reverse-gap claims. The repository-wide strict coverage baseline remains open and is not a full protocol acceptance verdict.
