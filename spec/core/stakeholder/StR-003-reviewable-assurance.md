---
id: StR-003
title: "Make contract failures reviewable"
type: StR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: depends_on
---
# StR-003: Make contract failures reviewable

## Stakeholder Need

Assurance reviewers need source-located diagnostics, explicit unsupported and
orphan states, and reproducible conformance results.

## Rationale

Silent repair, best-effort interpretation, or false coverage can conceal an
invalid contract.

## Validation Criteria

| ID | Criteria | Validation |
|---|---|---|
| StR-003-VC-1 | Malformed, ill-typed, undefined, unsupported, and orphaned inputs yield stable source-located diagnostic codes. | Negative corpus and diagnostic review (TC-016, TC-018) |
