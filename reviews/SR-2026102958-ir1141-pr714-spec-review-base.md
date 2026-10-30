---
id: SR-2026102958
title: "Base spec review of IR-1141 public-boundary reconciliation"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; IR-1141 PR #714 head 6eb0b5e1; FR-019, TC-048, TC-058; FR-046 and FR-048 context"
review_set: subset
---

## Summary

FR-046 AC-1/3/4/5 and FR-048 AC-1..5 remain live in merged main, and the PR does not edit or withdraw them. FR-046 AC-2/6 remain the only withdrawn criteria. The changed TC-048 adverse body case and encoder/reader limits preserve the structural controls. The public inventory and TC-058 need corrections below.

## Examined units

FR-019 public-item rows and structural paragraph (examined); TC-048 structural section (examined); TC-058 status and consumer sections (examined); FR-038-AC-367/368 and FR-046/048 ACs (context).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-019 still lists `DimensionPreimage`, `UnitPreimage`, `NominalIdentityPreimage`, `DimensionEquivalencePreimage` and similar V2 identity-preimage types as required public items, while its new structural identity paragraph says no identity preimage is exported. Remove all retired preimage exports from the planned inventory, and keep the current-export discrepancy explicit until code lands. | spec/model/functional/FR-019-rust-library-interface.md:229-230,272 |
| FND-002 | medium | FR-038-AC-368 requires public `loci` location records, but FR-019 omits the named location type from the `checked_package` root inventory. Add the public location type expected by the code cutover, marked Planned until exported. | spec/model/functional/FR-019-rust-library-interface.md:215 |
| FND-003 | medium | FR-019 retains the obsolete claim that the numeric API change does not change a node-key preimage counter encoding. The structural contract has no such counter. Remove this helper-specific assertion. | spec/model/functional/FR-019-rust-library-interface.md:85 |

## Verdict

FAIL until findings are repaired in the same PR. No applicable AssuranceProfile found in the affected spec tree.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-019 removes retired V2 identity-preimage types from the required target inventory and explicitly marks current code exports pending. |
| FND-002 | fixed | FR-019 adds planned CheckedPackageRefusalLocation to its V2 refusal-location root inventory, excluded until FR-038-AC-368 code exports it. |
| FND-003 | fixed | FR-019 numeric API paragraph now ends with “The change adds no alternate constructor.” The node-key preimage counter claim is gone. |
