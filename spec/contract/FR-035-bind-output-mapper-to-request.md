---
id: FR-035
title: "Bind a target mapper to one admitted request context"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-032
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-033
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-120
    type: implements
  - target: ix://agent-ix/quire-specification/FR-122
    type: implements
---
# FR-035: Bind a target mapper to one admitted request context

## Description

When dispatching an admitted request to a target mapper, the Contract IR
coordinator shall compare an immutable mapper request identity with the
canonical identity derived from the request's exact source package, ordered
obligations, native-source selection, model selection, semantic selection,
target profile, and limits before invoking the mapper.

## Inputs

- One admitted [FR-032](FR-032-admit-output-mapping-request.md) request.
- One target mapper exposing a constructor-private binding over the exact
  admitted request identity and target profile.

## Outputs

- Mapper dispatch only when every binding member equals the admitted request.
- One typed whole-operation refusal naming the mismatched binding, with no
  mapper invocation, mapping record, fragment, or package.

## Behavior

- Request admission shall retain a typed `sha256-jcs` identity over the complete
  existing `quire.output.mapping-request-identity/v1-draft.1` material.
- The common mapper seam shall require every target mapper to expose one
  complete request identity and target profile binding.
- If a binding is absent, wildcard, callback-based, or mutable, then the
  coordinator shall refuse it before mapper dispatch.
- The coordinator shall compare request identity and complete target profile
  before the first obligation and before each later invocation.
- The coordinator shall reject a mapper reused across a stale, foreign, or
  merely profile-equal request when any non-profile binding member differs.
- Presentation path, time, locale, diagnostic text, installed tool state, and
  downstream observer facts shall not occur in the mapper binding.
- The mapper-binding contract shall remain target-neutral and cycle-free so the
  OCL, SysML/KerML, and FRETish implementations consume the same guard.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-035-AC-1 | A mapper whose immutable request identity and target profile equal the admitted request is invoked once per selected obligation in source order. | Test (TC-044) |
| FR-035-AC-2 | A source-package, obligation, native, model, semantic, target-profile, or limit mutation changes the request identity and refuses before the mismatched mapper invocation with no partial result. | Test (TC-044) |
| FR-035-AC-3 | Reusing one mapper across a profile-equal request with a different model or semantic selection refuses rather than inheriting the prior correspondence. | Test (TC-044) |
| FR-035-AC-4 | Path, time, locale, display, installed-tool, and observer changes cannot alter binding equality or supply an omitted member. | Test (TC-044) |

## Dependencies

- [FR-032](FR-032-admit-output-mapping-request.md) owns strict request
  admission and immutable selection members.
- [FR-033](FR-033-account-for-output-obligations.md) owns complete mapper
  dispatch and record accounting after this guard succeeds.
- `ix://agent-ix/quire-specification/FR-120` and `FR-122` require exact target
  selection and authoritative model correspondence.
