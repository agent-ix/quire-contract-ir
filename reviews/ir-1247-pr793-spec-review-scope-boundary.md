---
id: SR-202611010115
title: "IR-1247 PR 793 scope-boundary review"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-contract-ir; PR #793; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Checked responsibility allocation between QSpec wire authority, an admitted dependency package, the IR strict reader, the nominal writer, and the public Rust API. No applicable AssuranceProfile was found under `spec/`.

## Verdict

**PASS** — QSpec owns the closed carrier, IR owns direct-dependency resolution and typed preservation, and the importing graph owns only local NodeRefs.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-038 dimension operand paragraph, role: examined. Reader, writer, selected dependency, and importing graph responsibilities are separate.
- FR-038-AC-387, role: examined. Positive lookup and typed preservation do not import a foreign graph node.
- FR-038-AC-388, role: examined. Direct-selection failures cannot be satisfied by closure or display names.
- FR-038-AC-389, role: examined. Self-minted foreign NodeRefs cannot cross the graph boundary.
- FR-019 planned API paragraph, inventory row, and AC-17, role: examined. The public type boundary exposes distinct local and package operands only after code lands.
- QSpec FR-322-AC-299, role: context_only. It remains the authority for wire form and vector outcomes.

## Evidence

The new FR-038 paragraph explicitly forbids minting a foreign node into the importing graph and excludes package operands from local `dependencies`. FR-019 names the future public types without claiming they are currently exported.
