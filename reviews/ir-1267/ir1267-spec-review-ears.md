---
id: SR-2026110101289
title: "IR-1267 PR 823 EARS review"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; IR-1267 PR 823; spec/model/functional/FR-019-rust-library-interface.md, spec/model/matrix/TC-058-model-crate-public-interface.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-058
    type: reviews
---

## Summary

Reviewed the FR-019-AC-24 and TC-058 diff against the current admitted operation-effect carrier and typed encoder. No applicable AssuranceProfile was found under spec/.

## Verdict

**PASS** — no findings in this method.

## Examined scope

```yaml
scope:
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
    as
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

    slice directly into'
- id: FR-019-public-inventory
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: '| `checked_package` (V2 operation effects) | types `CheckedOperationEffect`,
    `CheckedOperationEffectOrigin`, `CheckedOperationEffectProvenance`, `CheckedOperationEffectRef`,
    `CheckedOperationEffectRow`; the constructible row is typed writer input, while
    the private-field admitted view borrows the selected document, declaring node
    ID, original operation name, all four effect collections and provenance; the package
    has an optional borrowed `operation_effects` admitted-view read and the encoder
    sidecars have optional borrowed operation effects input. The associated borrowed
    raw-row'
- id: FR-019-AC-21
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: context_only
  excerpt: A default-feature external producer constructs `CheckedOperationEffectRow`
    inputs and passes them as optional borrowed `CheckedPackageSidecarsRef::operation_effects`
    to the V2 encoder without raw JSON; the fixed provenance origin is emitted with
    the supplied member identity. A default-feature external consumer obtains `None`
    from `CheckedPackageV2::operation_effects` when the optional carrier is absent
    and borrowed `CheckedOperationEffectRef` values when present. It reads selected
    document, declaring `CheckedNodeId`, original operation member, four ordered typed
    frame-entry collections and
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence

The new criterion is a testable outcome statement for an admitted package. Targeted Quire grammar validation reports no new warning.
