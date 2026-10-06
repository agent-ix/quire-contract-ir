---
id: SR-2105
title: "Failure-domain analysis \u2014 IR-661 relationship identity, ends and navigation"
type: SpecReview
analysis: failure-domain
scope: agent-ix/quire-contract-ir@705cef3f7a07370f111f19e8e7d86e1e07fd31d7; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; base 1540b3b6c0e4d167fe1ed9116c296e45e7dff258; Ticket IR-661
review_set: subset
---

## Summary

Identity keys, unstated failure modes and bounded work for the relationship declaration read and navigation. Work stays on the existing selected-row meter with exact and one-less procedures, and no new limit. Two gaps: the relationship identity key has two published, conflicting definitions upstream, and the relationship's non-end members have no read or refusal rule.

Ticket: IR-661. Method: `spec-review/spec-failure-domain-analysis`. Reviewer model `claude-opus-5-5`, run `236a8098-415e-43ce-aa4b-9a892aa7e251`.

## Verdict

**CONDITIONAL**: two `medium` findings. Clean, as examined: package-wide uniqueness, `conflicting-binding` across owners and missing identity as malformed (AC-168); unresolved and wrong-meaning end types with node-order independence (AC-167, TC step 3); the work accounting and `incomplete` pointer with no partial package; finite result shapes only, unbounded or ordered refused.

## Scope Examined

- `FR-038#identity-slot` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:975-981
- `FR-038#relationship-ends` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:992-1000
- `FR-038#role-lookup` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1029-1037
- `FR-038#navigation-derivation` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1046-1054
- `FR-038#declaration-order-accounting` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1065-1072
- `FR-038#operation-order-paths` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1078-1086
- `FR-038-AC-165` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3387
- `FR-038-AC-166` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3388
- `FR-038-AC-167` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3389
- `FR-038-AC-168` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3390
- `FR-038-AC-170` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3393
- `FR-038-AC-171` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3394
- `TC-048#fcd-step-3` (examined) spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:970-977
- `TC-048#fcd-step-4` (examined) spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:981-988
- `TC-048#fcd-step-7` (examined) spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1031-1039
- `QSpec FR-154 declaration identity` (context_only) agent-ix/quire-specification@60630b0d3d5e9cca048d1c314675db3bf9c6e4f2:spec/functional/type-model/FR-154-admit-domain-package-model.md:108-115
- `QSpec FR-154 declaration refusals` (context_only) agent-ix/quire-specification@60630b0d3d5e9cca048d1c314675db3bf9c6e4f2:spec/functional/type-model/FR-154-admit-domain-package-model.md:84
- `FCD contracts-v1 Identity minting members` (context_only) agent-ix/filament-core-data@68c0acba2390eb1593cc003d0a3e144638540a86:docs/semantic-data-system/contracts-v1.md:137-145
- `FCD FR-095 relationship slot` (context_only) agent-ix/filament-core-data@68c0acba2390eb1593cc003d0a3e144638540a86:spec/functional/FR-095-mint-package-identity-and-provenance.md:111-115
- `FCD semantic-ir schema relationship` (context_only) agent-ix/filament-core-data@68c0acba2390eb1593cc003d0a3e144638540a86:schema/semantic/v1/semantic-ir.schema.json:888-920

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Owner-nested relationship identity refused per FCD FR-095 alone; QSpec FR-154 and FCD prose disagree | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3387 |
| FND-002 | medium | No read or refusal rule for relationship direction/category/composite/origin; direction gates navigation | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:992-1000 |

## Finding Detail

- **FND-001** (medium, confidence medium, check `soundness`, unit `FR-038-AC-165`): Refusing an owner-nested relationship identity as malformed follows FCD FR-095 alone. Merged QSpec FR-154 says "a member's identity is its owner's identity, `/` and the member name". FCD's own contracts-v1 Identity minting prose lists relationship among members that "mint no slot segment", although its table and FR-095 use the global `relationship/` slot. The amendment neither acknowledges this split nor names an upstream ticket that reconciles it, so the identity key the reader enforces has two contradicting published definitions.
- **FND-002** (medium, confidence high, check `other`, unit `FR-038#relationship-ends`): The declaration read covers ends and multiplicities. It states no rule for the relationship's own `direction`, `category`, `composite` and `origin` members, although `direction` alone decides navigation eligibility. A relationship whose `direction` is absent, or outside the four schema values, has no specified refusal or pointer, so navigation over it is undefined. No criterion covers a malformed non-end member.

## Dispositions

Round 1, reviewed at `agent-ix/quire-contract-ir@c50050da5f20d4af41ab2dd6ea573d5e4b9abacc` (prior review `705cef3f7a07370f111f19e8e7d86e1e07fd31d7`), reviewer model `claude-opus-5-5`, run `ae1a4470-06bc-44c7-90df-81c70be53842`. Every outcome was verified against the fix commit's own text, not the author's receipt. All new criteria and procedures remain PLANNED / UNRUN; no implementation, CI gate or procedure coverage is claimed.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed c50050da5f20d4af41ab2dd6ea573d5e4b9abacc | The upstream conflict is now named in FR-038: QSpec FR-154's generic member nesting and FCD contracts-v1's pre-table prose against FR-095's specific relationship slot. The allocation is explicit and single: the relationship slot governs relationships, and owner nesting governs fields, operations and parameters. There is no dual reader. Both texts were checked against merged FCD 68c0acb (contracts-v1 lines 137-143 and the table row) and QSpec 60630b0 (FR-154 lines 108-110). Routing the upstream correction is recorded in private IR-661 comment c31761a6, which is data, not an exception. |
| FND-002 | fixed c50050da5f20d4af41ab2dd6ea573d5e4b9abacc | `direction`, `category`, `composite` and `origin` are now validated by reference against the FCD schema. That includes the common `origin` oneOf of `source` and `generated`, which matches common.schema.json at 68c0acb. An invalid value refuses `malformed-declaration` at the selection row before any application resolution, no default direction is supplied, and metadata is never fabricated. AC-173 and TC-048 step 9 add the cases. |
