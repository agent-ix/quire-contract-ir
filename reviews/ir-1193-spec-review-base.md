---
id: SR-2026103040
title: "IR-1193 PR 741 spec-review/base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #741 d7a4d883 over 755872aa; FR-349-AC-6..9, FR-019; changed Rust, tests and API inventory"
review_set: subset
---

## Summary

Frozen PR #741 head d7a4d88385a5788e7106fd9b6ecae43bc14f8d4d against merged main 755872aae7117ec0aa77c5667880aff219394cc5. Model: gpt-6. Ticket: IR-1193. No applicable AssuranceProfile was found under spec/. IR-1194 owns semantic_type to ValueType projection.

## Verdict

**PASS** — The FR-019 inventory row names the public read surface and its FR-349 owner; no requirement-text defect found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- **FR-349-AC-6 (examined):** For every admitted QSpec FR-322 literal kind, an external crate reads its closed kind, typed value and type node without inspecting `serde_json::Value` or parsing a display string. Exact numeric magnitude and decimal scale survive the read; serializing the package still yields the QSpec bytes.
- **FR-349-AC-7 (examined):** An external crate reads `CheckedBindingName::kind` for `parameters`, `body`, `optional`, `clause`, and an admitted other name; the other name remains available verbatim, and the first four cannot be mistaken for `other`.
- **FR-349-AC-8 (examined):** An external crate reads every admitted operation member and constraint production, identity, mode, ordered laws and ordered leaves through one borrowed typed view. The view contains no JSON payload or map-key lookup; a direct-deserialized or `test-support` fixture operation returns `NotAdmitted`, while the existing specific getters retain their prior results.
- **FR-349-AC-9 (examined):** The FR-019 exact public-item inventory and external compile probes show that IR exports the admitted read views but no QSL checked-expression constructor, `TermOperation` mirror, checker, or evaluator. Dependency-graph checks show that `quire-contract-model` does not depend on `quire-checked` or QSL and `quire-checked` does not depend on IR.
- **FR-019 (examined):** | `checked_package` (V2 typed read views) | types `CheckedBindingName`, `CheckedBindingNameKind`, `CheckedLiteralRead`, `CheckedLiteralValue`, `CheckedOperationRead`, `CheckedOperationMember`, `CheckedOperationConstraint`, `CheckedOperationLeaf`, `CheckedOperationLaw`, `CheckedOperationMode`, `CheckedOperationInterval`, `CheckedOperationRational`, `CheckedIntervalEnd`, `CheckedFairnessKind`, `CheckedFairnessGranularity`, `LawRole`, `OperationMemberKind`, `OperationConstraintKind`; methods `CheckedTerm::literal_read`, `CheckedBindingName::kind`, and `CheckedApplicationOperation::read` | FR-349 |

Plan completion: not assessed.
