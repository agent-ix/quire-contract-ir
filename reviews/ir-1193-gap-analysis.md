---
id: SR-2026103039
title: "IR-1193 PR 741 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #741 d7a4d883 over 755872aa; FR-349-AC-6..9, FR-019; changed Rust, tests and API inventory"
review_set: subset
---

## Summary

Frozen PR #741 head d7a4d88385a5788e7106fd9b6ecae43bc14f8d4d against merged main 755872aae7117ec0aa77c5667880aff219394cc5. Model: gpt-6. Ticket: IR-1193. No applicable AssuranceProfile was found under spec/. IR-1194 owns semantic_type to ValueType projection.

## Verdict

**CONDITIONAL** — The tagged tests leave admitted literal and operation productions without exact read oracles.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The 13-kind exact-value test constructs CheckedTerm::Literal directly; the admitted-package test verifies only Boolean. A reader regression affecting admitted numeric values or decimal scale could pass both tests. | tests/it/checked_package_v2_typed_terms.rs:266-345 |
| FND-002 | medium | The QSpec fixture sweep only counts operations, constraints and leaves. It never checks exact member and constraint variants or the values and order of laws, constraints and leaves; the other external test sees only function.call with empty constraints. | tests/conformance_qspec/main.rs:699-761 |

## Coverage

- **FR-349-AC-6 (examined):** For every admitted QSpec FR-322 literal kind, an external crate reads its closed kind, typed value and type node without inspecting `serde_json::Value` or parsing a display string. Exact numeric magnitude and decimal scale survive the read; serializing the package still yields the QSpec bytes.
- **FR-349-AC-7 (examined):** An external crate reads `CheckedBindingName::kind` for `parameters`, `body`, `optional`, `clause`, and an admitted other name; the other name remains available verbatim, and the first four cannot be mistaken for `other`.
- **FR-349-AC-8 (examined):** An external crate reads every admitted operation member and constraint production, identity, mode, ordered laws and ordered leaves through one borrowed typed view. The view contains no JSON payload or map-key lookup; a direct-deserialized or `test-support` fixture operation returns `NotAdmitted`, while the existing specific getters retain their prior results.
- **FR-349-AC-9 (examined):** The FR-019 exact public-item inventory and external compile probes show that IR exports the admitted read views but no QSL checked-expression constructor, `TermOperation` mirror, checker, or evaluator. Dependency-graph checks show that `quire-contract-model` does not depend on `quire-checked` or QSL and `quire-checked` does not depend on IR.
- **FR-019 (examined):** | `checked_package` (V2 typed read views) | types `CheckedBindingName`, `CheckedBindingNameKind`, `CheckedLiteralRead`, `CheckedLiteralValue`, `CheckedOperationRead`, `CheckedOperationMember`, `CheckedOperationConstraint`, `CheckedOperationLeaf`, `CheckedOperationLaw`, `CheckedOperationMode`, `CheckedOperationInterval`, `CheckedOperationRational`, `CheckedIntervalEnd`, `CheckedFairnessKind`, `CheckedFairnessGranularity`, `LawRole`, `OperationMemberKind`, `OperationConstraintKind`; methods `CheckedTerm::literal_read`, `CheckedBindingName::kind`, and `CheckedApplicationOperation::read` | FR-349 |

Plan completion: not assessed.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | The added admitted QSpec fixture asserts exact Integer and Decimal, but the published positive set also admits Boolean, Rational, Quantity, Float32Bits, Float64Bits, Text and Enum. The general fixture sweep calls literal_read only for type and kind; those values and the remaining directly constructed String, Uuid, Timestamp and None still have no admitted-package exact-value oracle. |
| FND-002 | still-open | The new exhaustive comparator checks exact values for member and constraint kinds present in the 19 QSpec positive fixtures. At c5deb110, the set has no Element or Fairness member and exercises only 7 of 16 catalog constraint kinds. AC-8 requires every admitted production; missing kinds still lack a reader-backed exact-value check. |

| FND-001 | fixed | QSpec c5deb110 positive sweep compares every admitted literal read to its wire value across all nine published kinds and checks canonical bytes; admitted native and option controls assert String, UUID, Timestamp and None. |
| FND-002 | fixed | QSpec positive sweep checks its eight member kinds and eight constraint kinds exactly; reader-admitted controls exercise Element, Fairness and the nine remaining authoritative catalog constraint kinds. |

## Disposition Verdict

**FAIL** — Both original coverage findings remain partially open on d7a0f813.

## Round 2 Verdict

**PASS** — FND-001 and FND-002 are fixed on 047e5dd7b16e15700c88034a8f998545a65e5f1b. Focused tests passed against QSpec c5deb110.
