---
id: SR-904101
title: IR-721 PR904 current Base review
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir; PR904; spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
review_set: subset
---

## Summary

Independent current normative review of PR904, branch titan/ir721-closure-spec, after merged IR905/906/907. This review supersedes the earlier boundary assessment only for current semantics; the four historical review files are preserved unchanged. The candidate changes one owning FR357 plus those historical artifacts; no executable change. No applicable AssuranceProfile is declared in the inspected spec assurance context. Review set is the dispatched subset: base, integrity, dependency, manual evidence qualification, and scope-boundary. Code/Rust and executable gap review are inapplicable to this spec-only change. No Cargo, tests, global matrix/coverage/properties/advisor scans, source modifications or gate were run. Explicit FR357 validation passed: 1/1 grammar-clean, zero grammar findings. Lead owns final artifact custody and the one docs-only gate.

## Verdict

PASS current changed normative scope; zero findings. Full IR-721 monitor preparation/closure remains incomplete and Planned.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

FR357 keeps distinct complete-closure and explicit selected-fact obligations. AC1-4 remain PLANNED/UNRUN; AC5-8 keep existing narrow implemented scope; AC9 explicitly names the already executed integration47 model/equality control without granting entry traversal, canonical environment or RT ownership. Inputs/outputs/refusals remain concrete; unchanged IDs are unique AC1..9, verification cells remain Test or Inspection, and existing relative FR038/353/354 links resolve. The new FR352 reference correctly names its selected source-union row contract. No hash/digest/pin/manifest/recorded-revision ceremony is introduced; original semantic source identities and bounded work are functional inputs. No new public carrier or speculative API is specified. This is scoped direct coverage/link inspection, not a full matrix certification.

## Examined units

```yaml
scope:
- id: FR-357-AC-1
  path: spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-721 and QSL same-acquisition witness retention). A strictly
    admitted generated monitor root with four entries yields one complete selected
    type/declaration closure; records, tuples, collections, unions, enum cases, object
    fields and source units retain original identities, producer order, presence,
    shape and member/payload types sufficient to build the canonical `TypeEnvironment`.
    Shared references do not duplicate declarations.
- id: FR-357-AC-2
  path: spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-721 and QSL same-acquisition witness retention). A missing,
    foreign, duplicate, conflicting or stale-after-rewrite producer row, unresolved
    reachable declaration, unrepresentable type or native nominal/Exact refusal returns
    its typed cause and first locus, with no partial view or invented identity.
- id: FR-357-AC-3
  path: spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-721). A large selected closure amid unrelated graph nodes
    and unsupported declarations visits only its reachable nodes/edges and charged
    identity/value material using an iterative worklist. Equal inputs and allowances
    give equal complete results or first work stops; one below the required work refuses
    without a partial view.
- id: FR-357-AC-4
  path: spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-721). Public API and dependency inspection finds a bounded
    IR projection over admitted package facts and original QSL witnesses; it finds
    no IR-owned `TypeEnvironment`, second plan/evaluator, RT dependency, source reparse
    or witness fabrication.
- id: FR-357-AC-5
  path: spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
  role: examined
  excerpt: Implemented (IR-721 selected facts; IR-owned fixture). Given an admitted
    package, explicit IDs for one selected model declaration, its selected record
    field and one equality application/occurrence, and matching IR-owned same-emission
    rows, the public bounded read returns exactly those complete facts. It preserves
    selected document, final structural node, original identities, field order/presence/type,
    exact equality occurrence and opaque FR-353 decision. No monitor-entry traversal
    or generated-source closure is claimed.
- id: FR-357-AC-6
  path: spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
  role: examined
  excerpt: Implemented (IR-721 selected facts). A missing, foreign, duplicate, conflicting
    or pre-rewrite selected declaration, field, equality application or fact row;
    wrong document, form, owner, role, occurrence or source-map correspondence; selected
    unsupported form or unrepresentable type; and native FR-353 refusal each produce
    a named typed outcome with first native locus and no partial view or invented
    identity. An unrelated unsupported graph form or absent witness outside the explicit
    selection leaves the same successful result. Duplicate selected IDs produce one
    item.
- id: FR-357-AC-7
  path: spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
  role: examined
  excerpt: Implemented (IR-721 selected facts). The bounded read inspects the explicitly
    selected admitted facts without traversing monitor entries or scanning unrelated
    graph nodes. Locating selected witnesses in full-emission row slices charges every
    inspected row and index entry, including unrelated rows; adding unrelated rows
    either raises required work or causes a charged first-row-locus stop at the former
    allowance. The exact allowance succeeds and one less refuses with attempted work
    and first uncharged locus. Equal inputs and allowance, including a package clone,
    yield equal results or first stop
- id: FR-357-AC-8
  path: spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
  role: examined
  excerpt: Implemented (IR-721 selected facts; API and dependency inspection). Public
    API and dependency inspection shows the read takes an admitted package, explicit
    selected model declaration/field IDs and equality application/occurrence pairs,
    IR-owned row slices and a work allowance, performs no QSL/RT dependency or source
    recheck, and returns neither an IR environment nor `ProgramParts`. A caller adapts
    QSL-owned equality emission rows without changing their values; IR checks admitted
    joins but does not claim to authenticate arbitrary caller-created Exact payloads.
- id: FR-357-AC-9
  path: spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
  role: examined
  excerpt: 'Implemented (`quire-integration` PR #47; explicit selected-fact scope
    only). A `quire-integration` test obtains a real final `PlanRoot.emission` through
    QSL, selects admitted IDs from its available model declaration/field rows and
    one equality application/occurrence pair, translates QSL equality rows without
    changing their values, and exercises the public IR selected-fact read on those
    explicit IDs. It verifies successful joins for representable selected model facts
    and an opaque FR-353 equality decision or its native typed missing-witness result
    for the selected equality. The executed PR #47 '
```
