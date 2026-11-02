---
id: SR-2026110283103
title: "IR-1003 PR 831 review dispositions"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; tests/it/checked_package_v2_refinement.rs; SR-2026110101300/FND-001 and SR-2026110101301/FND-001"
review_set: subset
---

## Summary

Ticket: IR-1003. Disposition round 1 checks the three added refinement dependency tests and their shared anchor fixture against both original FND-001 findings. The original finding documents remain unchanged. This document records the latest outcome for both original findings.

## Verdict

**PASS** for both finding dispositions. Distinct local ensure and operation-target anchors, a protocol-argument-only literal reference, and an adverse protocol target isolate the previously uncovered collection paths. No new defect was found in the fix scope. Final aggregate gates remain with the lead.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| SR-2026110101300/FND-001 | fixed | Three focused tests isolate all four cited dependency collection paths, with admitted controls and exact adverse-stage oracles. |
| SR-2026110101301/FND-001 | fixed | The same evidence closes the scoped whole-criterion dependency gap recorded by gap-analysis. |

## Coverage

Plan completion: not assessed

FR-061-AC-6 (role: examined), spec/checked_package/functional/FR-061-admit-refinement-nodes.md: The exact local dependency set includes body header keys and every direct row declaration/type and expression/member target, excludes external keys and annotations, and is structurally ordered. An omitted, extra or misordered dependency refuses `invalid_semantic_graph` at `/dependencies` before any row-meaning defect.

`refinement_dependency_only_ensure_and_operation_target_keys_are_required` uses a fresh anchor distinct from the source anchor, one role at a time. The operation case uses a literal receiver with no reference edge. Each baseline must admit and exercises the full omission/addition/duplicate/order/precedence matrix and writer preservation helper.

`refinement_dependency_only_protocol_argument_reference_is_required` introduces a fresh literal referenced only by the target argument. It verifies that its key was not already a dependency and exercises the same complete dependency matrix.

`refinement_dependency_only_protocol_target_precedes_catalog_node_mismatch` preserves an admitted valid target control. An admitted declared protocol target must equal its abstract catalog node, so a distinct valid target-only key cannot be authored. Instead, a deliberately wrong local target contributes a distinct dependency: the complete set reaches `InvalidModelBinding` / `MalformedDeclaration` at `/target/node`, while removing that dependency reaches `InvalidSemanticGraph` / no cause at `/dependencies`. Both exact outcomes are verified through reader and public writer. This is truthful adverse-stage evidence, not an invalid positive admission.

Inspected all four recorded collector-removal failures: ensure, operation-target, protocol arm and protocol target. Each compiled and failed its intended behavioral assertion with exit 101, at the dependency baseline for the first three and the expected target-node refusal oracle for the fourth. The restored production source admits all three focused tests. Existing fix-final evidence records 3 passed / 0 failed, workspace all-target clippy and model default-feature clippy passing, and formatting passing. No additional heavy build was run. Production remains unchanged in the fix diff.
