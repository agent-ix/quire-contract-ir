---
id: SR-2026110101297
title: IR-1267 base spec review of public operation effect inventory
type: SpecReview
analysis: base
scope: 'agent-ix/quire-contract-ir; IR-1267 PR #826; spec/model/functional/FR-019-rust-library-interface.md'
review_set: subset
---

## Summary

Ticket: IR-1267. Reviewed the one-cell FR-019 public inventory correction at 02592137. The method names remain prose; only actual top-level public types are quoted for the TC-058 parser. FR-019-AC-5 and AC-24 are unchanged.

## Verdict

**PASS** — no new spec finding. The correction restores the declared exact inventory without changing the required borrowed accessor or wire contract.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined scope

```yaml
scope:
- id: FR-019-AC-5
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: The `quire_contract_model` crate root has no glob re-export, and its default-feature
    public items are exactly those the Public items table lists as implemented at
    the tested revision. Entries explicitly marked planned are excluded until their
    owning code lands; exporting a planned item fails the inventory until its row
    is marked implemented. The public-item inventory fails on an added or missing
    implemented item; no implemented item in the table is reachable through a `quire_contract_ir`
    path. The retired `PACKAGE_DOMAIN_V2` export is absent.
- id: FR-019-AC-24
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: planned until IR-1267 lands. A default-feature external consumer calls
    `CheckedPackageV2::operation_effect_rows(&self) -> Option<&[CheckedOperationEffectRow]>`
    only after strict package admission. It obtains `None` when the optional carrier
    was omitted and a borrowed, nonempty slice of the exact admitted producer rows
    when present, retaining row order, selected document and declaring identities,
    original operation members, all four ordered effect collections and provenance;
    an explicitly empty effect remains a present row. It passes that slice directly
    as `CheckedPackageSidecarsRef::operation_
- id: FR-019
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: types `CheckedOperationEffect`, `CheckedOperationEffectOrigin`, `CheckedOperationEffectProvenance`,
    `CheckedOperationEffectRef`, `CheckedOperationEffectRow`; the constructible row
    is typed writer input, while the private-field admitted view borrows the selected
    document, declaring node ID, original operation name, all four effect collections
    and provenance; the package has an optional borrowed operation_effects admitted-view
    read and the encoder sidecars have optional borrowed operation effects input.
    The associated borrowed raw-row operation_effect_rows read is planned until IR-1267
    lands and
```

## Coverage

Targeted Quire validation of FR-019 passed. Focused TC-058 exact inventory passed in default and test-support lanes. The earlier spec PR #823 already had an independent review; this check covers the one-cell correction. No applicable AssuranceProfile was found in this repository.
