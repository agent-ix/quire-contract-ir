---
id: SR-202611010316
title: "Base specification review of IR-1226 wire specification PR 811"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Examined FR-038's selected operation-effect carrier behavior and AC-406 through AC-408, plus FR-019-AC-21, against QSpec STD-640 and TC-977. Checked omitted, empty and malformed carriers; exact selected owner/member and provenance; effect entry checks; bounded work; producer input and admitted consumer output. The new criteria are direct, testable assertions and cite the QSpec vectors by reference without the duplicated QSpec AC number.

## Scope

| ID | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-406 | examined | planned until the IR-1226 reader lands. The strict reader admits QSpec STD-640 positive vectors OEC-01..07 by reference, including each of the four effect kinds, an intentionally empty effect, multiple own operations and equal member names under distinct selected documents or declaring objects. A present row exposes the exact producer frame entries and provenance through a borrowed typed read; an absent optional carrier yields no effect rows and a typed unavailable-witness outcome for an operation-effect request, never an inferred empty effect. The canonical writer emits the same top-level mem |
| FR-038-AC-407 | examined | planned until the IR-1226 reader lands. The strict reader applies QSpec TC-977's OES-01..14 schema refusals and OER-01..26 semantic refusals to complete selected packages, checking each published code, cause and primary RFC 6901 locus. An explicitly `null`, empty or non-array `operation_effects` refuses `malformed_wire` at `/operation_effects`, unlike omission. It refuses missing/extra row or effect members, wrong entry kinds or provenance, foreign/wrong-kind declaring nodes, undeclared or inherited-only operations, dropped/invented effect entries, repeated or descending keys and missing cover |
| FR-038-AC-408 | examined | planned until the IR-1226 reader lands. Each visited carrier row and effect entry consumes one validation visit before inspection; a limit one below the required work returns incomplete at the first uncharged locus without a partial effect view. Equal admitted packages and their clones yield equal borrowed row data or equal typed outcomes. TC-048 reads the QSpec STD-640 vector file through the selected checkout and holds no local vector or schema copy. |
| FR-019-AC-21 | examined | planned until the IR-1226 reader lands. A default-feature external producer constructs `CheckedOperationEffectRow` inputs and passes them as optional borrowed `CheckedPackageSidecarsRef::operation_effects` to the V2 encoder without raw JSON; the fixed provenance origin is emitted with the supplied member identity. A default-feature external consumer obtains `None` from `CheckedPackageV2::operation_effects` when the optional carrier is absent and borrowed `CheckedOperationEffectRef` values when present. It reads selected document, declaring `CheckedNodeId`, original operation member, four order |
| FR-038 | examined | Selected operation-effect carrier: optional top-level operation_effects, strict checked row admission and borrowed read. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Clean. The requirements distinguish carrier absence from an admitted empty effect, require published typed refusals and loci, and keep the source frame authoritative. The one grammar warning reported by scoped validation is on pre-existing FR-038-AC-364 outside the diff.
