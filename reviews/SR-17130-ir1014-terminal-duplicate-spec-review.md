---
id: SR-17130
title: "Base review of IR-1014 terminal duplicate refusal spec"
type: SpecReview
analysis: base
scope: "quire-contract-ir spec/checked_package/functional/FR-062-carry-terminal-duplicate-identities.md and spec/checked_package/matrix/TC-063-terminal-duplicate-refusal-identities.md at 50bf93f4fe690486ee68f2359a8e53f216b1aa21"
review_set: base
---

## Summary

The proposed typed refusal covers QSpec FR-476's two duplicate forms, ordered
node checks, primary locus and pointer, and earlier-error precedence. Targeted
Quire validation passed. The one public API finding was fixed at 50bf93f4;
the reviewed spec has no remaining findings.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | Name the new public closed-sum type and refusal member or accessor. AC-4 says an external consumer using only the three existing named types can exhaustively match variants, but it must name or import the new variant type. Specify its export and include it in AC-4's compile-time contract. | FR-062 Outputs, FR-062-AC-4, TC-063 Test Procedure | wrong-requirement |

## Disposition

PASS at 50bf93f4. FR-062 now names the public
`quire_contract_model::CheckedTerminalDuplicate` enum and
`CheckedPackageRefusal::terminal_duplicate` field. AC-4 and TC-063 import all
four public types and require a match of both variants without a wildcard.
This resolves FND-001 without narrowing any criterion.
