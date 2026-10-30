---
id: SR-2026103017
title: "Base specification review of IR-1141 PR #720 AC-375 delta"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/model/functional/FR-019-rust-library-interface.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; spec/model/matrix/TC-058-model-crate-public-interface.md"
review_set: subset
---

## Summary

Ticket IR-1141. Reviewed the spec delta from `57eb27c3b299e304ef617f89ec33e2d482e6457c` to frozen #720 head `af657d196595a6b8ecc28706d90c85cbf0b55121`. No applicable AssuranceProfile or enforced review policy was found for the changed documents.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

READY from spec-review/base. FR-038-AC-375 states a measurable borrowed enum view, retains malformed/semantic refusal paths, and names TC-048/TC-058. FR-019 lists the accessor under the existing public `NominalBody` item without inventing a second export. Both test cases describe positive and adverse evidence. No changed spec text adds a hash, digest, pin, copied QSpec file, or compatibility route.

## Coverage

Examined FR-038's new enum-view behavior, FR-038-AC-375, the adjacent FR-038-AC-374 boundary, FR-019 public item inventory, TC-048 admitted enum view section, and TC-058 nominal public boundary. Targeted Quire validation of four changed docs found one preexisting unrelated `ac:non-canonical-shape` advisory at FR-038 line 4814; the new wording had no grammar finding. No Cargo aggregate gate ran. Source/test evidence gaps are tracked in the existing gap-analysis SR.

## Reverification

At exact clean pushed head `521aa372df1450ebdb6150dc31a5fc83ddf806ff`, the sole post-review delta adds two accessor-specific `compile_fail` doctests. FR-038-AC-375, FR-019, TC-048 and TC-058 spec text is unchanged. The new probes discharge TC-058's borrowed-member mutation and unchecked-construction controls. Rechecked unordered semantics: FR-038 requires byte-ascending members for unordered declarations, so the prior unordered-order gap claim was invalid. No new base finding; verdict READY.
