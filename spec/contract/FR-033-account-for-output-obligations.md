---
id: FR-033
title: "Account for every mapped obligation"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-032
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-121
    type: implements
  - target: ix://agent-ix/quire-specification/FR-269
    type: implements
  - target: ix://agent-ix/quire-specification/FR-298
    type: implements
---
# FR-033: Account for every mapped obligation

## Description

When dispatching an admitted request, the Contract IR coordinator shall pass
each obligation to exactly one matching target mapper and shall derive exactly
one source-bound mapping record from the returned candidate without defaulting,
repairing, or approximating its disposition.

## Inputs

- One admitted [FR-032](FR-032-admit-output-mapping-request.md) request.
- One target-neutral mapper whose exact profile equals the request profile.
- One mapper candidate per requested obligation containing deterministic fragment
  bytes, local byte regions, exact dependencies, one disposition, conditions,
  causes, independently typed adequacy references, and charged work.

## Outputs

- One ordered record per requested obligation, with a derived
  `sha256-jcs` record identity.
- One typed whole-operation refusal with no partial record set or target package.

## Behavior

- The coordinator shall invoke only a mapper whose complete profile equals the
  admitted request profile.
- The coordinator shall pass obligations to the mapper exactly once and in the
  admitted source order.
- The record constructor shall bind the exact source obligation, ordered actual
  dependencies, target profile, disposition, conditions, causes, generated
  regions, source-fact state, and separately qualified observation/protocol
  adequacy references into the record identity preimage.
- The record constructor shall accept only `preserved` with no conditions or
  causes and represented output, `conditional` with nonempty conditions and
  represented output, `unrepresented` with nonempty causes and no output, or
  `refused` with nonempty causes and no substitute output.
- The record constructor shall reject `preserved` or `conditional` when the
  source fact is pending, incomplete, or refused.
- The record constructor shall distinguish observation adequacy from protocol
  adequacy by type and shall accept no bare adequacy value.
- The coordinator shall reject missing, duplicate, unrequested, cross-wired,
  stale, foreign, over-budget, or malformed candidates without exposing a
  partial record set.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-033-AC-1 | Every requested obligation is dispatched once in source order and yields exactly one record; no missing, duplicate, unrequested, or cross-wired record is accepted. | Test (TC-043) |
| FR-033-AC-2 | Preserved, conditional, unrepresented, and refused records satisfy their exact condition/cause/output invariants, and absent classification never defaults to preserved. | Test (TC-043) |
| FR-033-AC-3 | Mutating any source, dependency, profile, disposition, condition, cause, region, source state, or adequacy-domain member changes the record identity or refuses construction. | Test (TC-043) |
| FR-033-AC-4 | Pending, incomplete, or refused source facts and plausible target text cannot become preserved, conditional, or Boolean success. | Test (TC-043) |
| FR-033-AC-5 | Observation and protocol adequacy remain independently typed, and an omitted value remains absent without a cross-domain default. | Test (TC-043) |

## Dependencies

- [FR-032](FR-032-admit-output-mapping-request.md) supplies the admitted request
  and exact target profile.
- `ix://agent-ix/quire-specification/FR-121`, `FR-269`, and `FR-298` own the
  complete accounting behavior, closed disposition vocabulary, and record identity.
