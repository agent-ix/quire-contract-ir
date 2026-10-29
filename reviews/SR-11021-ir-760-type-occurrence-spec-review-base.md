---
id: SR-11021
title: "Base spec review of IR-760 occurrence-role correction"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/model/functional/FR-053-selected-population-binding.md; spec/model/matrix/TC-453-selected-population-binding.md"
review_set: subset
---
## Summary

Ticket: IR-760. Method: spec-review/base. Frozen PR #393. The edited selector, acceptance criterion, and test step consistently require the population type node’s Type occurrence. The surrounding selector and typed outcome contract remains coherent.

Examined FR-053 Inputs, FR-053-AC-1, and TC-453 Test Procedure step 2. Read the other FR-053 criteria and TC-453 steps as context. Producer evidence: `qsl-semantics/src/check/lowering.rs` records the binder `type_key` as `OccurrenceRole::Type`; `qsl-semantics/src/check/lowering/model.rs` constructs that key as `bounded_domain` / `model_population`; `qsl-package/src/emit.rs` preserves the role and ordinal in `CheckedSourceMapEntry`. Those files are unchanged from QSL origin/main 9925955 in the checked out QSL tree.

## Verdict

**PASS** — The correction is source-backed and the changed criteria remain falsifiable.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
