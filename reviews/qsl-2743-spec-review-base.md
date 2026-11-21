---
id: SR-967175026302197768900474181941935084516
title: "QSL-2743 PR 1117 spec-review base checklist of the changed IR rows"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; QSL-2743; PR 1117 head 31a74a3c6d48f8f272f82092cd28ba9e377d2516; FR-352-AC-31, FR-358-AC-15, FR-069-AC-11, FR-019 inventory"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-352", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-358", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-069", type: references }
---

## Summary

Base checklist review of the IR spec rows PR 1117 adds or changes. Ticket: QSL-2743. Two rows name a TC id in their Verification cell; FR-358-AC-15 and FR-352-AC-31 are weaker or vaguer than the FR-830 contract they implement; FR-069-AC-11 has no error path. No hash, digest or pin use was found. No AssuranceProfile exists under spec/, so none applies.

## Verdict

**CONDITIONAL** — medium and low findings only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The Verification cell reads 'Test (FSpec FR-830, TC-2112)'; the checklist requires the cell to name a method only, with no TC- id. | spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md:627 |
| FND-002 | low | The Verification cell reads 'Test (FSpec FR-830, TC-2112)'; the checklist requires the cell to name a method only, with no TC- id. | spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md:256 |
| FND-003 | medium | FR-358-AC-15 is weaker than the contract it implements: it requires only 'the first unpaid kind and locus', while FR-830-AC-11 and Work and storage require the full attempted amount, the channel (model_tuples vs model_unions) and the sidecar row index, the closure-edge boundary (MUW-B-02), charges kept after either channel refuses, and accumulation across joins. Two implementers could both pass this AC and disagree with FR-830. | spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md:256 |
| FND-004 | medium | FR-352-AC-31 does not state the payload scope: it says 'payload original type against the side-input witness table' but not that a data payload is any admitted type but Clock (option, collection, map, bounded, carried-quantity and enum included), nor the Clock vs nullary distinction. The code refuses several admitted forms UnsupportedExactType and still satisfies the AC as written. | spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md:627 |
| FND-005 | low | FR-069-AC-11 states only the admission; it has no error-path clause for a payload naming an object type outside the selected package or document, so the refusal boundary of the intake change is unstated and untestable. | spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md:166 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | medium | IR criteria were narrowed to match the available evidence, without contract or architecture authority. The original FR-352-AC-31 required a record or object field of an admitted union to project to Composite(type_handle); FR-352-AC-35 now says only a record field and an optional record field, after the object-field case was dropped from the test fixture. FR-352-AC-37 likewise keeps only the refusal half of validate_and_rebind, with no positive rebind and no union-completeness stage, which the old FND-010 scope required. | spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md:631, 633 |
| FND-007 | low | FR-069-AC-12 names no refusal code, cause or locus, unlike its FR-069 siblings (AC-8 states code/cause/location). Its test asserts only that the read is not Admitted, so any refusal (or a panic-free unrelated failure) satisfies it. | spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md:167; tests/conformance_fspec/model_union_witness.rs:1551-1589 |

## New findings (disposition pass 3)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-008 | medium | FR-358-AC-17 still says a budget one below the measured edges refuses 'at the ModelUnionRow locus of the row being closed'. 6e9a31c09 locates every closure-edge charge at NativeJoinLocusSource::GraphNode of the graph node examined, and budget_case_b02, which is bound to AC-17, asserts GraphNode and not ModelUnionRow. FR-830 allows either locus source, but the IR criterion now states a locus that neither the code nor its own bound test produces. | spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md:258; crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:193-207; tests/conformance_fspec/model_union_witness.rs:966-1060 |
| FND-009 | medium | 58e981bbf widens the shared record-field projection (project_record_field_type), which every record, object, tuple-position, runtime selected-type and selected-fact projection uses. Float32, Float64, Bytes, JsonObject, RationalRange, TextBounds and CarriedRational fields used to refuse UnsupportedExactType; they now project to Exact types. The bare-Float exact mode matches filament-exact (ieee.rs: a bare Float32/Float64 is strict exact). But no IR criterion states the new record-field behavior: FR-352-AC-32 covers union payloads only, and four existing it tests and one reached test were re-pointed from Float32 to Clock instead. The CarriedQuantity field form has no positive test. | spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md:628; crates/filament-contract-model/src/checked_package/v2/exact_projection.rs:3450-3500 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 2 (head 16a686c). Fixed; the verbatim after_excerpt is in the round-2 data block below and in the round-2 tracker comment. |
| FND-002 | fixed | Round 2 (head 16a686c). Fixed; the verbatim after_excerpt is in the round-2 data block below and in the round-2 tracker comment. |
| FND-003 | fixed | Round 2 (head 16a686c). Fixed; the verbatim after_excerpt is in the round-2 data block below and in the round-2 tracker comment. |
| FND-004 | fixed | Round 2 (head 16a686c). Fixed; the verbatim after_excerpt is in the round-2 data block below and in the round-2 tracker comment. |
| FND-005 | fixed | Round 2 (head 16a686c). Fixed; the verbatim after_excerpt is in the round-2 data block below and in the round-2 tracker comment. |
| FND-006 | fixed | Round 3 (head 70b2445). Fixed; the verbatim after_excerpt is in the round-3 data block below and in the round-3 tracker comment. |
| FND-007 | fixed | Round 3 (head 70b2445). Fixed; the verbatim after_excerpt is in the round-3 data block below and in the round-3 tracker comment. |

## Disposition round 2 verdict

**CONDITIONAL — MERGEABLE=FALSE.** Round 2 (head 16a686c): all five original findings are fixed by the atomic rewrite. New: FND-006 medium (criteria narrowed to match evidence: FR-352-AC-35 object field, FR-352-AC-37 rebind) and FND-007 low (FR-069-AC-12 names no code or locus).

## Disposition round 3 verdict

**CONDITIONAL — MERGEABLE=FALSE.** Round 3 (head 70b24456c): FND-006 and FND-007 are fixed. New: FND-008 medium (FR-358-AC-17 states a ModelUnionRow locus that the code and its bound test do not produce) and FND-009 medium (the widened record-field projection has no IR criterion).

## Machine-readable data

```yaml
{
  "clean": false,
  "scope": [
    {
      "id": "FR-352-AC-31",
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "role": "examined",
      "excerpt": "Implemented (QSL-2743). Joining the selected model tuple and model union witness rows (FSpec FR-830) returns the complete rows keyed by structural node, retaining each type handle, settled key, case rank, position, name, original case, member and payload as the producer's opaque identities. Tuple rows are admitted first, then union identity, case census, per-case checks, payload original type against the side-input witness table, and coverage: a full scope refuses an uncovered union with `IncompleteIdentityWitness` and a reached scope with `UnavailableIdentityWitness`. Direct projection and a "
    },
    {
      "id": "FR-358-AC-15",
      "path": "spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md",
      "role": "examined",
      "excerpt": "Implemented (QSL-2743). The model tuple and model union row join charges one shared `NativeJoinBudget`: every row of both channels is charged in the common pass before allocation, then the selected clone and the prior-copy publish. The measured exact cumulative work and storage succeed and one unit below either refuses at the first unpaid kind and locus with no partial publish; a zero budget refuses before any row is retained."
    },
    {
      "id": "FR-069-AC-11",
      "path": "spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md",
      "role": "examined",
      "excerpt": "Implemented (QSL-2743). A model union variant payload naming an object type of the selected package is admitted as a `Reference` payload, so a union such as `Mix.Held(Holder)` passes intake instead of being refused for an undeclared payload type."
    },
    {
      "id": "FR-019-inventory",
      "path": "spec/model/functional/FR-019-rust-library-interface.md",
      "role": "examined",
      "excerpt": "| `checked_package` (V2 reader) | types `AdmissionBinding`, `CheckedCollectionKind`, `CheckedDeclaration`, `CheckedDependencyClockSelection`, `CheckedDependencySelection`, `CheckedDiagnosticCause`, `CheckedDiagnosticCode`, `CheckedDiagnosticStage`, `CheckedDiagnosticV2`, `CheckedDiagnosticsV2`, `CheckedJson`, `CheckedJsonError`, `CheckedMemberType`, `CheckedModelField`, `CheckedModelFieldsError`, `CheckedModelObjectFields`, `CheckedModelRecordField`, `CheckedModelRecordFields`, `CheckedModelRecordFieldsError`, `CheckedModelVariantBody`, `CheckedModelVariantCase`, `CheckedVariantPayloadType`, `"
    }
  ],
  "findings": [
    {
      "fnd": "FND-001",
      "severity": "low",
      "confidence": "high",
      "method": "spec-review/base",
      "check": "other",
      "artifact_id": "FR-352-AC-31",
      "related_ids": [],
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "lines": "627",
      "excerpt": "Implemented (QSL-2743). Joining the selected model tuple and model union witness rows (FSpec FR-830) returns the complete rows keyed by structural node, retaining each type handle, settled key, case rank, position, name, original case, member and payload as the producer's opaque identities. Tuple rows are admitted first, then union identity, case census, per-case checks, payload original type against the side-input witness table, and coverage: a full scope refuses an uncovered union with `IncompleteIdentityWitness` and a reached scope with `UnavailableIdentityWitness`. Direct projection and a ",
      "finding": "The Verification cell reads 'Test (FSpec FR-830, TC-2112)'; the checklist requires the cell to name a method only, with no TC- id."
    },
    {
      "fnd": "FND-002",
      "severity": "low",
      "confidence": "high",
      "method": "spec-review/base",
      "check": "other",
      "artifact_id": "FR-358-AC-15",
      "related_ids": [],
      "path": "spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md",
      "lines": "256",
      "excerpt": "Implemented (QSL-2743). The model tuple and model union row join charges one shared `NativeJoinBudget`: every row of both channels is charged in the common pass before allocation, then the selected clone and the prior-copy publish. The measured exact cumulative work and storage succeed and one unit below either refuses at the first unpaid kind and locus with no partial publish; a zero budget refuses before any row is retained.",
      "finding": "The Verification cell reads 'Test (FSpec FR-830, TC-2112)'; the checklist requires the cell to name a method only, with no TC- id."
    },
    {
      "fnd": "FND-003",
      "severity": "medium",
      "confidence": "high",
      "method": "spec-review/base",
      "check": "soundness",
      "artifact_id": "FR-358-AC-15",
      "related_ids": [
        "FR-830-AC-11"
      ],
      "path": "spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md",
      "lines": "256",
      "excerpt": "Implemented (QSL-2743). The model tuple and model union row join charges one shared `NativeJoinBudget`: every row of both channels is charged in the common pass before allocation, then the selected clone and the prior-copy publish. The measured exact cumulative work and storage succeed and one unit below either refuses at the first unpaid kind and locus with no partial publish; a zero budget refuses before any row is retained.",
      "finding": "FR-358-AC-15 is weaker than the contract it implements: it requires only 'the first unpaid kind and locus', while FR-830-AC-11 and Work and storage require the full attempted amount, the channel (model_tuples vs model_unions) and the sidecar row index, the closure-edge boundary (MUW-B-02), charges kept after either channel refuses, and accumulation across joins. Two implementers could both pass this AC and disagree with FR-830."
    },
    {
      "fnd": "FND-004",
      "severity": "medium",
      "confidence": "high",
      "method": "spec-review/base",
      "check": "ambiguous",
      "artifact_id": "FR-352-AC-31",
      "related_ids": [
        "FR-830-AC-4",
        "FR-830-AC-9"
      ],
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "lines": "627",
      "excerpt": "Implemented (QSL-2743). Joining the selected model tuple and model union witness rows (FSpec FR-830) returns the complete rows keyed by structural node, retaining each type handle, settled key, case rank, position, name, original case, member and payload as the producer's opaque identities. Tuple rows are admitted first, then union identity, case census, per-case checks, payload original type against the side-input witness table, and coverage: a full scope refuses an uncovered union with `IncompleteIdentityWitness` and a reached scope with `UnavailableIdentityWitness`. Direct projection and a ",
      "finding": "FR-352-AC-31 does not state the payload scope: it says 'payload original type against the side-input witness table' but not that a data payload is any admitted type but Clock (option, collection, map, bounded, carried-quantity and enum included), nor the Clock vs nullary distinction. The code refuses several admitted forms UnsupportedExactType and still satisfies the AC as written."
    },
    {
      "fnd": "FND-005",
      "severity": "low",
      "confidence": "medium",
      "method": "spec-review/base",
      "check": "coverage",
      "artifact_id": "FR-069-AC-11",
      "related_ids": [],
      "path": "spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md",
      "lines": "166",
      "excerpt": "Implemented (QSL-2743). A model union variant payload naming an object type of the selected package is admitted as a `Reference` payload, so a union such as `Mix.Held(Holder)` passes intake instead of being refused for an undeclared payload type.",
      "finding": "FR-069-AC-11 states only the admission; it has no error-path clause for a payload naming an object type outside the selected package or document, so the refusal boundary of the intake change is unstated and untestable."
    }
  ],
  "bindings": []
}
```

## Disposition round 2 data

```yaml
{
  "round": 2,
  "head": "16a686c7641e9b07841c130d28ad26ebf1a198a1",
  "dispositions": [
    {
      "fnd": "FND-001",
      "outcome": "fixed",
      "after_excerpt": "position, name, original case, member and payload as the producer's opaque identities, with the closure-edge count of every published positive vector. | Test |"
    },
    {
      "fnd": "FND-002",
      "outcome": "fixed",
      "after_excerpt": "nels, the common pass over every row, the reached closure edges, then the row join with retention for in-scope rows. A denied charge consumes nothing. | Test |"
    },
    {
      "fnd": "FND-003",
      "outcome": "fixed",
      "after_excerpt": "An exhausted charge reports the attempted amount and a locus naming the channel, the original sidecar index and the form, so that the first unpaid charge is identifiable in both channels."
    },
    {
      "fnd": "FND-004",
      "outcome": "fixed",
      "after_excerpt": "A model union case whose payload is data of any admitted type except `Clock` (scalar, quantity, record, tuple, union, object reference, option, collection, bounded, carried-quantity or enum) joins when its payload node and original type agree with the type its own member form derives; a `Clock` payload is retained as a Clock payload and is never read as data or as a nullary case. A wrong payload node or original type refuses with the published code."
    },
    {
      "fnd": "FND-005",
      "outcome": "fixed",
      "after_excerpt": "A model union variant payload naming an object type that no selected-package declaration provides is refused by the strict read and is not admitted."
    }
  ],
  "new_findings": [
    {
      "fnd": "FND-006",
      "severity": "medium",
      "confidence": "high",
      "method": "spec-review/base",
      "check": "soundness",
      "artifact_id": "FR-352-AC-35",
      "related_ids": [
        "FR-352-AC-37",
        "FR-352-AC-31"
      ],
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "lines": "631, 633",
      "excerpt": "A full scope refuses an uncovered union with `IncompleteIdentityWitness` at `/model_unions`, and a reached scope with `UnavailableIdentityWitness`. Direct projection, a record field and an optional record field of a joined union return `ValueType::Composite(type_handle)` and never an enum; the runtime selected-type declarations declare each joined union once, by that handle.",
      "finding": "IR criteria were narrowed to match the available evidence, without contract or architecture authority. The original FR-352-AC-31 required a record or object field of an admitted union to project to Composite(type_handle); FR-352-AC-35 now says only a record field and an optional record field, after the object-field case was dropped from the test fixture. FR-352-AC-37 likewise keeps only the refusal half of validate_and_rebind, with no positive rebind and no union-completeness stage, which the old FND-010 scope required."
    },
    {
      "fnd": "FND-007",
      "severity": "low",
      "confidence": "medium",
      "method": "spec-review/base",
      "check": "untestable-ac",
      "artifact_id": "FR-069-AC-12",
      "related_ids": [
        "FR-069-AC-8"
      ],
      "path": "spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md",
      "lines": "167; tests/conformance_fspec/model_union_witness.rs:1551-1589",
      "excerpt": "A model union variant payload naming an object type that no selected-package declaration provides is refused by the strict read and is not admitted.",
      "finding": "FR-069-AC-12 names no refusal code, cause or locus, unlike its FR-069 siblings (AC-8 states code/cause/location). Its test asserts only that the read is not Admitted, so any refusal (or a panic-free unrelated failure) satisfies it."
    }
  ]
}
```

## Disposition round 3 data

```yaml
{
  "round": 3,
  "head": "70b24456c7132d4603f0ddcaa5f3ba87aba3befc",
  "dispositions": [
    {
      "fnd": "FND-006",
      "outcome": "fixed",
      "after_excerpt": "`validate_and_rebind` over a package whose model declares unions returns an equal witness when every family is complete; refuses a witness that lacks only a union row with `IncompleteIdentityWitness` at that union node, form `selected_model_union`, after the tuple stage; re-validates every retained union row against the package it is rebound to, before completeness, and refuses a row that is wrong for that package with the row's own refusal; and never mutates the receiver."
    },
    {
      "fnd": "FND-007",
      "outcome": "fixed",
      "after_excerpt": "A model union variant payload naming an object type that no selected-package declaration provides is refused by the strict read with `UnsupportedConstruct`, cause `DeclarationForm`, at the payload `typeRef` of the offending variant in the document and at the `owner` of the union declaration node in the package; the same union naming the selected-package object is admitted."
    }
  ],
  "new_findings": [
    {
      "fnd": "FND-008",
      "severity": "medium",
      "confidence": "high",
      "method": "spec-review/base",
      "check": "soundness",
      "artifact_id": "FR-358-AC-17",
      "related_ids": [
        "FR-830-AC-11"
      ],
      "path": "spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md",
      "lines": "258; crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:193-207; tests/conformance_fspec/model_union_witness.rs:966-1060",
      "excerpt": "The reached closure charges one edge per typed member of every reached union (scalars included, `Clock` none); a budget one unit below the measured edges refuses at the `ModelUnionRow` locus of the row being closed.",
      "finding": "FR-358-AC-17 still says a budget one below the measured edges refuses 'at the ModelUnionRow locus of the row being closed'. 6e9a31c09 locates every closure-edge charge at NativeJoinLocusSource::GraphNode of the graph node examined, and budget_case_b02, which is bound to AC-17, asserts GraphNode and not ModelUnionRow. FR-830 allows either locus source, but the IR criterion now states a locus that neither the code nor its own bound test produces."
    },
    {
      "fnd": "FND-009",
      "severity": "medium",
      "confidence": "high",
      "method": "spec-review/base",
      "check": "missing-requirement",
      "artifact_id": "FR-352-AC-32",
      "related_ids": [
        "FR-352-AC-14",
        "FR-357-AC-5"
      ],
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "lines": "628; crates/filament-contract-model/src/checked_package/v2/exact_projection.rs:3450-3500",
      "excerpt": "A model union case whose payload is data of any admitted type except `Clock` (scalar, quantity, record, tuple, union, object reference, option, collection, bounded, carried-quantity or enum) joins when its payload node and original type agree with the type its own member form derives; a `Clock` payload is retained as a Clock payload and is never read as data or as a nullary case. A wrong payload node or original type refuses with the published code.",
      "finding": "58e981bbf widens the shared record-field projection (project_record_field_type), which every record, object, tuple-position, runtime selected-type and selected-fact projection uses. Float32, Float64, Bytes, JsonObject, RationalRange, TextBounds and CarriedRational fields used to refuse UnsupportedExactType; they now project to Exact types. The bare-Float exact mode matches filament-exact (ieee.rs: a bare Float32/Float64 is strict exact). But no IR criterion states the new record-field behavior: FR-352-AC-32 covers union payloads only, and four existing it tests and one reached test were re-pointed from Float32 to Clock instead. The CarriedQuantity field form has no positive test."
    }
  ],
  "nonregression": "FND-001..005 remain fixed at 70b24456c: Verification cells are Test, FR-358-AC-19 and FR-352-AC-32 are unchanged, and FR-069-AC-12 now states the exact code, cause and loci."
}
```
