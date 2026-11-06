---
id: SR-2026110602354621
title: "IR-1353 PR 936 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 936; FR-357 AC13/AC14; reached model/unit read and tests"
review_set: subset
---

## Summary

Independent review of frozen PR #936 at f7f24b80e. Examined FR-357-AC-13 and FR-357-AC-14, TC-468, and the changed Rust and test files.

## Verdict

CONDITIONAL. The reached-model unit path loops over `self.models.selected_models()` for every reached unit, charging each unrelated model. A package with the same reached facts and an unrelated selected model before the owner changes the work result or refuses at the old exact allowance. This conflicts with AC14’s reached-only, no package-wide discovery scan boundary.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Reached model unit projection scans every selected model before finding its owner, so unrelated selected documents consume work and can exhaust an exact reached-only allowance | runtime_type_projection.rs:1334 |

## Coverage

- FR-357-AC-13 (examined): reached record, object, and distinct source/model unit facts with shared declarations.
- FR-357-AC-14 (examined): typed refusals, reached-only traversal, shared allowance, and clone agreement.
- TC-468 (examined): reached-model and unit prerequisite procedure and expected results.
- Plan completion: not assessed.
- Focused fmt, Clippy, unit, integration, and scoped Quire checks were reported by the coder and accepted without rerun.
- No applicable AssuranceProfile was found under spec/.
