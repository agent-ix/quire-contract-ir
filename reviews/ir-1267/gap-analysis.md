---
id: SR-2026110101296
title: IR-1267 gap-analysis of borrowed operation effect rows
type: SpecReview
analysis: gap-analysis
scope: 'agent-ix/quire-contract-ir; IR-1267 PR #826; crates/quire-contract-model/src/checked_package/v2/mod.rs, tests/it/checked_package_v2_operation_effects.rs, FR-019 public inventory'
review_set: subset
---

## Summary

Ticket: IR-1267. Reviewed PR #826 at 6da15e68. The borrowed accessor and typed round-trip test meet FR-019-AC-24, but the exact public-item inventory gate remains red because the spec table marks associated methods as top-level expected items.

## Verdict

**CONDITIONAL** — one high finding blocks merge until the inventory row is corrected and the focused TC-058 test passes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-019-AC-24 is tagged by two external tests, but TC-058 exact public inventory for FR-019-AC-5 fails on the newly quoted associated method names. The tagged round-trip test does not discharge this required inventory gate. Correct the spec row and rerun TC-058. | spec/model/functional/FR-019-rust-library-interface.md:295; tests/it/public_interface.rs:1017 |

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
  role: context_only
  excerpt: types `CheckedOperationEffect`, `CheckedOperationEffectOrigin`, `CheckedOperationEffectProvenance`,
    `CheckedOperationEffectRef`, `CheckedOperationEffectRow`; the constructible row
    is typed writer input, while the private-field admitted view borrows the selected
    document, declaring node ID, original operation name, all four effect collections
    and provenance; the package has an optional borrowed `operation_effects` admitted-view
    read and the encoder sidecars have optional borrowed operation effects input.
    The associated borrowed raw-row `operation_effect_rows` read is planned until
    IR-1267 lands
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: examined
  excerpt: 'Planned until IR-1267 lands. Compile a default-feature external consumer
    that

    strictly reads otherwise valid packages with the optional `operation_effects`

    carrier omitted and with two distinct admitted rows, one carrying an explicitly

    empty effect. Assert `operation_effect_rows()` returns `None` for omission and

    a borrowed slice in producer order for presence. Compare each typed row''s

    selected document, declaring node, original operation member, four ordered

    effect collections and provenance with the admitted input. Borrow the present

    slice directly into `CheckedPackageSidecarsRef::operation_'
```

## Coverage

Plan completion: not assessed. Quire matrix reports FR-019-AC-24 tagged by tests/it/checked_package_v2_operation_effects.rs:112 and :188. The existing TC-058 inventory test is a separate gate; source parser inspection proves both quoted method names are expected as top-level exports.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The corrected inventory row keeps associated method names as prose, so the parser no longer expects them as top-level exports. Focused TC-058 exact inventory passes in default and test-support lanes. |

## Disposition verdict

**PASS (round 1, source review at 02592137)** — The only finding is fixed. The code lead owns the final aggregate gate.
