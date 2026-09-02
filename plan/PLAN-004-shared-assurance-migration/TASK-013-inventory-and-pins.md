---
id: TASK-013
title: "Inventory the repository against the decision table and pin the accepted releases"
type: Task
status: done
track: common
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/PLAN-004
    type: part_of
---

# TASK-013: Inventory the repository against the decision table and pin the accepted releases

Classify every script and schema against the accepted migration decision table
before touching anything, write the inventory up on issue #39, and pin the
shared components at the exact accepted releases. Record the Engineering
Assurance artifacts this repository reads by digest, and the drift between the
`v0.2.0` tag and the acceptance record that landed after it.

Author FR-022, revise FR-009 and PGM-01-R09 to stop prescribing a local
verifier, and extend the test matrix with TC-029 through TC-034.

## Acceptance

Every script and schema has a decision and a reason; the pins resolve from the
public registry with no `npm.ix` reference anywhere; and the specification no
longer prescribes machinery this plan removes.

## Completion

The inventory is comment `#39` on the migration issue. `assurance/pins.json`
records engineering-assurance 0.2.0 and the two artifacts read from it, both
byte-identical at the tag and at `main`. `schemas/README.md` classifies all five
schemas. FR-022, the revised FR-009, PGM-01-R09-AC-2, and TC-029..TC-034 are in
the specification and the matrix.
