---
id: TC-046
title: "Canonical backend counterexample replay (withdrawn from Contract IR)"
type: TC
status: withdrawn
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-037
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-055
    type: references
  - target: ix://agent-ix/quire-specification/TC-219
    type: references
---
# TC-046: Canonical backend counterexample replay (withdrawn from Contract IR)

## Description

This case verified FR-037's retired replay criteria. The replay properties are
verified by quire-specification:TC-219 and QSL's replay executor tests.
FR-037-AC-6, that Contract IR defines no replay type and calls no executor, is
the same public-surface and source check FR-039 makes, so TC-055 verifies it
and this case is withdrawn.

## Test Procedure

None in this repository; see TC-055.

## Expected Results

No Contract IR test carries a `TC-046` trace.

## Status

Withdrawn from Contract IR.
