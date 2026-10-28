---
id: SR-612117
title: "IR-1088 PR 655 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #655 at 0991926b; typed test operation fixtures, TC-058 and public feature boundary"
review_set: subset
---

## Summary

Reviewed IR-1088 code PR #655 against FR-019-AC-14 through AC-16, TC-058, and IR-1088 acceptance criteria 1 and 2. CG replacement and focused consumer tests under ticket AC3 are owned by CG and outside this PR. No CI workflow or dependency graph gate changed.

## Verdict

**PASS** for the changed code and Rust review. No confirmed defect in the scoped diff. The `test-support` capability is default-off, the constructor validates the specified present fields before creating typed views, and the fixture provenance is private and omitted from the serialized operation. A fixture does not create an admitted package. Aggregate gates remain with the code lead.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-019-AC-14 (examined): “With `test-support`, an external consumer constructs typed fixture operations with absent identity, mode, law definitions or member declaration, and with the identity, exact/non-exact rounding modes, foreign and conflicting division definitions, and wrong-kind/wrong-value/matching mode inputs used by the four CG FR-014-AC-18/48/50 scalar derivation controls. Typed `identity`, `mode`, `law_definitions`, and `member_declaration` reads return the authored values in order, including intentional absence; `is_test_fixture` is true in every case. No direct JSON view injection is required.”
- FR-019-AC-15 (examined): “The test fixture constructor rejects an empty present identity, an empty present mode value, a law definition with empty authority or identity, and a present member key with a wrong domain, wrong digest length, uppercase or nonhexadecimal digest by typed error without yielding an operation. A fixture with intentionally absent optional fields remains constructible and distinguishable through `is_test_fixture` from direct-serde non-admitted operations. A fixture operation cannot be supplied as an already admitted `ContractPackageV1` or `CheckedPackageV2`, and serializing it into a package wire does not bypass strict production admission or its typed refusal.”
- FR-019-AC-16 (examined): “`test-support` is disabled by default; a default-feature external compile probe cannot import its fixture types or call its constructor or provenance query. A feature-enabled external probe can use exactly the documented fixture exports and methods; the existing default-feature inventory, the independent `fault-injection` delta, and the absence of a model API through `quire_contract_ir` remain enforced.”
- TC-058 (examined): typed fixture consumer procedure and feature boundary cases, as authored in `spec/model/matrix/TC-058-model-crate-public-interface.md`.
- FR-019-AC-5 (context only): default public inventory and no root-model bridge.
- IR-1088 AC1/2 (examined): typed validated fixture support and strict admission boundary.
- IR-1088 AC3 (context only): CG-owned consumer replacement.
- Plan completion: not assessed.
