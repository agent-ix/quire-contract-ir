---
id: SR-7001
title: base review of IR-742 IEEE admission spec
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir@31a1c95c1e0df6d6b19aa15cf732778d00cb5a11; spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md
review_set: subset
---

## Summary

Examined the frozen FR-038, TC-048 and TestMatrix delta against QSpec FR-322-AC-23 and the checked-operation catalog. Ticket: IR-742. Method: spec-review/base.

## Verdict

**CONDITIONAL** — The finding(s) below need disposition before this spec is accepted.

## Examined scope

```yaml
- id: FR-038-AC-222
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-742). Through `CheckedPackageV2::read`, a `float32_bits` literal declared at a Float32 type and a `float64_bits` literal declared at a Float64 type each admit in otherwise valid packages. Retaining each value kind and changing only its declared `type` to the opposite IEEE width, while re-deriving the affected node and package identities, refuses `ill_typed`/`operator-ineligible` at that literal's `type`, including in an operand position whose catalogued family would otherwise accept the declared type. Neither mutation is admitted or refused merely at an enclosing operand. Thi
- id: FR-038-AC-223
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-742). Through `CheckedPackageV2::read`, valid `quire.op.ieee.to_float32` over Float64 and `quire.op.ieee.to_float64` over Float32 admit when each `type_argument` and `result_type` name the same correctly typed destination node. For each identity, changing only `result_type` to the source-width node while retaining the valid destination `type_argument`, and re-deriving identities, refuses `ill_typed`/`operator-ineligible` at the application's `result_type`; changing both `type_argument` and `result_type` to the source-width node also refuses there by the catalogued `member_fam
- id: FR-038/IEEE-literal-type-and-catalog-result
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "- **IEEE literal type and catalog result.** Independently of an enclosing\n  operand position, a `float32_bits` literal requires a declared `type` in the\n  `float32` family, and a `float64_bits` literal requires one in the `float64`\n  family. A mismatch refuses `ill_typed`/`operator-ineligible` at that\n  literal's `type`. The result form of each catalogued application is checked\n  against its `result_type`. In particular, the catalog's `member` result form\n  for `quire.op.ieee.to_float32` and `quire.op.ieee.to_float64` requires the\n  `result_type` to name the same type node as `operation.member"
- id: TC-048/IEEE-literal-and-catalog-result-admission
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: '## IEEE literal and catalog result admission (FR-038-AC-222 and FR-038-AC-223)


    PLANNED/UNRUN (IR-742). Build otherwise valid V2 packages using the published

    `quire.op.ieee.to_float32` and `quire.op.ieee.to_float64` entries, their selected

    IEEE law, correctly typed `type_argument`, and genuine Float32 and Float64

    type nodes. The Float64→Float32 package with a `float64_bits` source literal

    and Float32→Float64 package with a `float32_bits` source literal must each

    admit through `CheckedPackageV2::read`.

    Then change only a `float32_bits` literal''s declared `type` to Float64, and

    only a `float64_b'
- id: TM-002/Pending-IEEE-admission-coverage
  path: spec/checked_package/matrix/tests.md
  role: examined
  excerpt: '| FR-038 | FR-038-AC-222, FR-038-AC-223 | TC-048 | 🚧 PLANNED/UNRUN; no tagged implementation test yet | Public-reader Float64→Float32 and Float32→Float64 admission with genuine recomputed identities; both opposite-width literal kind/type mismatches at the literal `type`; for each conversion, wrong `result_type` with valid `type_argument` and wrong `type_argument` with matching wrong `result_type`, both at application `result_type`; exact typed code, cause, pointer and locus; no stale-identity substitute. |'
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The prose requires result-form checking for every catalogued application, but AC-223 and TC-048 cover only the two IEEE conversion entries. The current reader explicitly skips general result_forms checks. Either scope the normative clause to those entries or add discriminating cases for the remaining result forms. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3380-3385 |
