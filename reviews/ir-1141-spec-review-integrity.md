---
id: SR-3006
title: "IR-1141 PR #727 spec-review-integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/model/functional/FR-019-rust-library-interface.md, spec/model/matrix/TC-058-model-crate-public-interface.md"
review_set: subset
---

## Summary

PR #727 at 3ffa2477fd6ccf25d9b74657f839237ccbbd7e4f against c69c6e3e627e7a53511b8b8ca108aef78fd346f2. Four spec files changed; no source or test implementation changed. QSpec current main 686e49e812ecf88b89618e00cddddea32db84b90 and IR FR-063 supply the closed dimension-equivalence wire rule. No applicable AssuranceProfile was found under the changed modules.

## Verdict

**FAIL** — The new public item row duplicates an existing named item. Other public API inventory and planned/implemented distinction remain explicit.

## Scope

- `FR-038-AC-374` (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: PLANNED/UNRUN (IR-1141). An external default-feature consumer constructs each of the four source nominal bodies through its typed fallible constructor, wraps each in `CheckedTerm::NominalBody`, and observes exact closed-body serialization. The same body read from a conforming package equals the constructed typed body. Empty or duplicate enum members, a nonidentifier member, a noncanonical exponent, an invalid nested structural node id, a noninteger rational numerator and a nonpositive rational denominator each return `malformed_wire` at the offending body-relative pointer with no body. Public 
- `FR-038-AC-5` (context_only), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: Each nominal node carries a structural `{package, path, ordinal}` id. A source `enum`, `dimension` or `unit` has the closed typed body stated in "The flat wire"; an empty aggregate on one, an empty enum member list, a missing unit scale, an extra unit body member or a non-integer dimension exponent refuses `malformed_wire` at the schema step (QSpec FR-322-AC-269, NB-01..NB-05). An enum with `ordered: true` admits as `ordered_enum` (NB-06). After valid wire shape, the reader validates owner, declaration, semantic type, dependencies, enum case and unit rules independently; a semantic mismatch re
- `FR-038` (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: The model crate SHALL expose an opaque `NominalBody` for producing these four source-authored body forms. Its separate fallible `try_enum`, `try_dimension`, `try_unit` and `try_dimension_equivalence` constructors SHALL accept typed members and return a typed refusal with the offending body-relative RFC 6901 pointer when a member does not have the closed shape above. In particular, they SHALL check non-empty unique identifier enum members, canonical integer exponents, structural node-id shape, and the integer numerator and positive denominator of every rational. The constructors validate body s
- `FR-019` (examined), spec/model/functional/FR-019-rust-library-interface.md: | `checked_package` (V2 nominal body fields) | types `CheckedRational`, `DimensionTerm` | FR-038-AC-5 |
- `FR-019-AC-5` (context_only), spec/model/functional/FR-019-rust-library-interface.md: The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists as implemented at the tested revision. Entries explicitly marked planned are excluded until their owning code lands. The public-item inventory fails on an added or missing implemented item; no implemented item in the table is reachable through a `quire_contract_ir` path.
- `TC-048` (examined), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md: PLANNED/UNRUN (IR-1141). In a default-feature external consumer, call each fallible nominal constructor with typed values and compare its serialized `CheckedTerm::NominalBody` against an independently authored closed V2 body. Read a conforming package containing that body and compare the retained typed term. For each malformed member named in AC-374, assert `malformed_wire` and the exact body-relative pointer without receiving a body. Construct a valid shape with an unresolved graph reference or wrong owner, then verify that the ordinary reader gives its existing semantic refusal, not construc
- `TC-058` (examined), spec/model/matrix/TC-058-model-crate-public-interface.md: ## Nominal body public boundary (FR-038-AC-374)  PLANNED/UNRUN (IR-1141). The default-feature consumer imports `NominalBody` and its four fallible constructors from `quire_contract_model` after the owning code lands. Compile-fail probes reject construction of a raw nominal variant, mutation of an opaque body, unchecked conversion and direct deserialization into `NominalBody`; the root `quire_contract_ir` crate does not export it. Re-run FR-019-AC-5's exact named inventory with this planned entry included only after it is implemented.
- `FR-063` (context_only), spec/checked_package/functional/FR-063-dimension-equivalence-node.md: The closed V2 wire schema SHALL require `root_scale` with positive-integer `numerator` and `denominator` strings, exactly two `dimension_node_ids`, and no `declaration` member. A missing scale, zero or negative scale component, one-entry pair or present declaration SHALL refuse `malformed_wire` at the node before graph checks. The graph check SHALL require a schema-valid `root_scale` to be reduced, with the exact ratio determined by the two base roots. For equivalences between derived dimensions, it SHALL require equal normalized maps and

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new FR-019 nominal-body-fields row lists `CheckedRational` a second time; the existing V2 rational-value row already lists it. An exact named public inventory should assign one listing and one owning requirement set to this type. | spec/model/functional/FR-019-rust-library-interface.md:217 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-019 now lists `DimensionTerm` alone in the nominal-fields row; `CheckedRational` appears once in the existing V2 rational-value row. |

Disposition verified at PR #727 head `f8f3742df6b9a69cc30521cdd466d59628a6722e` by inspecting the exact fix delta; targeted Quire validation passed. No new finding in the fix delta.
