---
id: TC-054
title: "Kani counterexample replay through the QSL replay facade (withdrawn from Contract IR)"
type: TC
status: withdrawn
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-031
    type: references
  - target: ix://agent-ix/quire-specification/AD-016
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-098
    type: references
---
# TC-054: Kani counterexample replay through the QSL replay facade (withdrawn from Contract IR)

## Description

This test case verified the retired criterion FR-031-AC-3: a serialized Kani
counterexample reproducing through the QSL executor with the same outcome and
witness, or returning a typed non-success disagreement. The counterexample
envelope, replay source and replay result are QSL `qsl-replay` types, and the
crossing runs from the codegen replay adapter through `qsl_replay::replay`
(QSL ADR-011 E9, FR-098). The test belongs to those owners, so this case is
withdrawn from Contract IR and no Contract IR test backs it.

## Test Procedure

None in this repository. The crossing is exercised by QSL's replay executor
tests (QSL FR-098) and by the codegen replay adapter's tests.

## Expected Results

No Contract IR test carries a `TC-054` trace, and no Contract IR code
constructs a counterexample envelope or invokes a replay executor.

## Status

Withdrawn from Contract IR.
