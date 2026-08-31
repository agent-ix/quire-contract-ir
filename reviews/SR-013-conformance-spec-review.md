---
id: SR-013
title: "Issue 10 schema, corpus, and conformance interface specification review"
type: SpecReview
analysis: architecture-evaluation
scope: "FR-011 through FR-020, STD-001, NFR-001 through NFR-004, TM-002, TC-018, TASK-009"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/10
    type: reviews
---
# Issue 10 conformance specification review

## Summary

This review covers issue #10's package schema, conformance schema and corpus,
stable Rust surface, JSON Lines runner, and TC-018 gates. It does not authorize
crate publication, automatic CI triggers, downstream repository changes,
execution engines, or the human release decision.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-106 | high | Non-package operation inputs and expectations lacked schema-pinned wire shapes. | FR-018 |
| FND-107 | high | Rejection timing for malformed expectations was undefined. | FR-018, FR-020 |
| FND-108 | high | Manifest failures were not mapped exactly to runner exit-2 codes. | FR-020 |
| FND-109 | medium | STD-001 ambiguously implied runner operational codes extend semantic diagnostics. | STD-001, FR-020 |
| FND-110 | medium | Inventory path and digest were not bound by the manifest. | FR-018 |
| FND-111 | medium | A single potentially-undefined diagnostic token could omit obligation kinds. | FR-018, FR-014 |
| FND-112 | medium | Boundary coverage used categories rather than an exact registry. | FR-018 |
| FND-113 | medium | The manifest had no byte cap enforced before parsing. | FR-018 |
| FND-114 | low | `tool_environment` lacked a distinct classification boundary. | FR-020 |
| FND-115 | low | Package and expression canonical comparable results were ambiguous. | FR-018, FR-016 |
| FND-116 | high | Four registered collection and identifier boundaries had no normative numeric limits. | FR-011, FR-018 |
| FND-117 | high | Schema and inventory files lacked byte caps before parsing or digesting. | FR-018, FR-020 |
| FND-118 | medium | The corpus required public registries that the stable Rust API did not own or inspect. | FR-018, FR-019 |
| FND-119 | high | Recursive value types and semantic collections had no fixed preflight limits despite the no-panic invariant. | FR-013, FR-017, FR-019, NFR-003 |
| FND-120 | high | A collection declaration's maximum count had no fixed-width upper bound or over-range diagnostic. | FR-013, STD-001, FR-018 |
| FND-121 | low | The disposition's finding count became stale after later closing rounds. | SR-013 |
| FND-122 | low | Review scope omitted FR-013 and FR-017 after findings required normative edits there. | SR-013, FR-013, FR-017 |
| FND-123 | low | Review scope still omitted FR-011, FR-014, and FR-016 cited by retained findings. | SR-013, FR-011, FR-014, FR-016 |
| FND-124 | medium | Equal 10000-node and 10000-entry caps made an exact-at-collection-limit structured input impossible. | FR-019, STD-001, FR-018 |

## Independent Spec Review Disposition

All nineteen findings are fixed and retained; none is waived:

- FND-106: the conformance schema has named subschemas for every operation's input and expectation.
- FND-107: all expectations validate before any fixture executes.
- FND-108: each operational condition has one exact exit-2 mapping.
- FND-109: STD-001 now separates semantic diagnostics from runner operational codes.
- FND-110: the manifest binds the inventory path and digest.
- FND-111: coverage has one token for each obligation kind.
- FND-112: FR-018 enumerates the exact boundary registry.
- FND-113: the manifest byte cap is checked before parsing.
- FND-114: `tool_environment` was removed, leaving six operational codes.
- FND-115: package child outputs and expression declaration outputs are explicit.
- FND-116: the four tokens without normative numeric bounds were removed from the closed boundary registry.
- FND-117: every schema, inventory, payload, expectation, and canonical-byte file has the same pre-parse/pre-digest byte cap.
- FND-118: FR-019 owns and inspects the three stable fixed-width registry exports, and FR-018 declares that dependency.
- FND-119: FR-019 fixes the global 25000-node, depth-256, and 10000-entry collection limits so an exact-at-collection-limit input can remain below the global node cap; STD-001 registers the failure and exact boundary tokens force both edges.
- FND-120: collection declarations use the closed unsigned 32-bit range, wider wire integers use `invalid_numeric_bounds`, and exact declaration-boundary tokens force both edges.
- FND-121: the disposition heading is synchronized with the current retained finding set.
- FND-122: the declared review scope now includes FR-013 and FR-017.
- FND-123: the declared review scope now includes every FR from FR-011 through FR-020.
- FND-124: the global node cap is 25000 while the per-collection cap remains 10000, so exact collection boundaries remain constructible under the 16 MiB file cap and independently testable.

The pre-implementation closing check returned exactly `No actionable findings.`
for FND-106 through FND-123. Implementation then exposed FND-124, which is
retained and fixed above; implementation review must re-check this delta.
TASK-008 is the upstream dependency; epic #11 and its human release decision
remain downstream.
