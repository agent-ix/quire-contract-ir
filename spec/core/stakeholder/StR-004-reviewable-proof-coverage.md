---
id: StR-004
title: "Make proof coverage reviewable"
type: StR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-045
    type: satisfied_by
  - target: ix://agent-ix/quire-contract-ir/StR-003
    type: depends_on
---
# StR-004: Make proof coverage reviewable

## Stakeholder Need

When assurance owners assess proof coverage, the baseline shall identify which
requirements, modules, functions, and obligations have qualified evidence and why
any candidate has no proof credit.

## Rationale

A count of passing checks or tagged tests cannot show what was eligible, what was
proved, or what was refused. Reviewers need visible gaps and scope limits before they
can set a meaningful coverage target.

## Validation Criteria

| ID | Criteria | Validation |
| --- | --- | --- |
| StR-004-VC-1 | Given a baseline with a successful production proof, a missing result, and a narrowed or shadow result, a reviewer can identify the credited claim and the reason each other claim lacks unqualified production-proof credit at all four levels. | Demonstration |
