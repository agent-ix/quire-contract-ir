---
id: StR-003
title: "Make contract failures and evidence reviewable"
type: StR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: depends_on
---
# StR-003: Make contract failures and evidence reviewable

## Stakeholder Need

Assurance reviewers need source-located diagnostics, explicit unsupported and
orphan states, reproducible conformance results, and evidence that never turns
an automated result into a release decision.

## Rationale

Silent repair, best-effort interpretation, or false coverage can conceal an
invalid contract. Reviewable failure states preserve the human decision
boundary established by PGM-01.

## Validation Criteria

| ID | Criteria | Validation |
|---|---|---|
| StR-003-VC-1 | Malformed, ill-typed, undefined, unsupported, and orphaned inputs yield stable source-located diagnostic codes. | Negative corpus and diagnostic review (TC-016, TC-018) |
| StR-003-VC-2 | Evidence reports exact subject, tool, environment, outcomes, and limitations while leaving release approval open. | Assurance packet inspection (TC-020) |

## Dependencies

PGM-01 defines evidence identity and human release authority.
