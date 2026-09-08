---
id: SR-034
title: "Base review of issue 54 boundary correction and Rust baseline"
type: SpecReview
analysis: base
scope: "ADR-0054, ADR-0055, ADR-0053, FR-019, NFR-002, NFR-005, STD-001, TM-002"
review_set: base
relationships:
  - target: ix://agent-ix/quire-contract-ir/ADR-0054
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/NFR-005
    type: reviews
---
# SR-034: Base review of issue 54 boundary correction and Rust baseline

## Summary

The owner selected the base checklist review: identifier/link integrity,
requirement quality, and the six test-coverage rules, with no optional analysis
lenses. The first draft incorrectly treated Filament archetype schema/codegen as
a formal semantic authority and proposed a new Contract IR model layer. The
rerun removes that requirement and records the narrower boundary: a modeling
language may explicitly project schema facts needed by a real proof, while
Contract IR remains producer-neutral and accepts only its own closed formal
declarations.

The reviewed #54 disposition is an architecture correction, not a new reader or
adapter implementation. The Rust 1.98.1 requirement is separately specified
and remains subject to its implementation tests.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-341 | high | **Closed in rerun:** the first draft confused archetype schema/datatype generation with formal semantics and introduced an unnecessary lossless model layer in Contract IR. FR-024 and its diagnostics/tests were removed; ADR-0054 now preserves the ownership boundary. | ADR-0054 Context/Decision; deleted FR-024 | wrong-requirement |
| FND-342 | high | **Closed in rerun:** a newer stable Rust release triggers the real compatibility matrix within seven days and blocks later candidates absent adoption or a time-bounded owner-approved hold. | ADR-0055; NFR-005-AC-5 | missing-requirement |
| FND-343 | medium | **Closed in rerun:** Rust observations are explicitly preliminary and name the environment, compiler, repository revisions, operations, outcomes, and non-qualification boundary. | ADR-0055 Preliminary compatibility observations | correct-requirement-no-evidence |
| FND-344 | high | **Closed in rerun:** issue #54 no longer makes the generic language parser/compiler wait for a concrete Filament adapter. FR-013, FR-019, and FR-023 are the available producer-neutral boundary; a concrete archetype projection is specified only when a real proof case needs it. | ADR-0054 Decision/Consequences | wrong-requirement |
| FND-345 | high | **Closed by owner clarification:** the internal `agent-ix-semantic-ir` crate and its license are not a Contract IR gate. No reader, copied schema, artifact negotiation, or dependency is proposed here. | ADR-0054 Context/Decision | wrong-requirement |
| FND-346 | high | **Closed in rerun:** `ConfigVersion.parent` and the one-sided `versionNumber` bound cannot be inferred into executable types. The source may be retained, while executable projection refuses until an owning modeling-language profile supplies reviewed reference and finite-bound semantics. | ADR-0054 ConfigVersion consequence; ADR-0053 Reference and bounds decision | wrong-requirement |
| FND-347 | medium | **Closed in rerun:** `Domain` and `StateMachine` illustrate the missing ownership distinction. Filament owns their schema-generated representation; the modeling language owns their meaning; an explicitly qualified lowering owns correspondence to proof constructs. | ADR-0054 Decision | wrong-requirement |

## Checklist Result

- IDs and relationships in the reviewed artifact set are unique and valid.
- ADR-0054 records the accepted owner boundary without manufacturing a new FR,
  test population, diagnostic family, or upstream license gate.
- Existing FR-013/FR-019/FR-023 inputs, outputs, validation behavior, and tests
  remain the generic Contract IR interface; this decision changes none of their
  implemented semantics.
- The negative `ConfigVersion` cases are explicit and do not infer bounds,
  recursive records, host widths, runtime populations, or generated-type
  semantics.
- NFR-005 and TC-036/TC-037 cover the independent Rust toolchain change,
  including compatibility failures, release triggers, and trace attributes.
- All findings from both review passes are closed. No optional analysis was
  selected.

## Intake and Evidence Boundary

- Selected review set: `base`.
- Optional analyses selected: none.
- Applicable profile: `spec/assurance/AP-001-contract-ir-v01.md`; it requires a
  spec review operation but contains no `review_selection`, so the owner's base
  selection controls this run.
- Draft base revision: `decc99a430a0894de489102dbab04e83d1fb804f` plus the
  specification changes in this branch.
- `quoin write . --types SpecReview` supplied the live SpecReview authoring
  contract for this rerun.
- No new Filament adapter, formal projection, Rust implementation,
  qualification decision, hosted workflow, or release evidence is claimed.

## Rerun Result

**PASS.** The base specification-quality and applicable coverage checks pass
after remediation. Issue #54 requires no Contract IR implementation and no
longer blocks Agent A. Rust 1.98.1 implementation and execution evidence remain
required by NFR-005 before a source release claims toolchain qualification.
