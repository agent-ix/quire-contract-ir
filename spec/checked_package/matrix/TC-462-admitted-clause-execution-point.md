---
id: TC-462
title: "Admitted V2 clause execution-point accessor"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-058
    type: verifies
---
# TC-462: Admitted V2 clause execution-point accessor

## Description

Verify FR-058-AC-1 through FR-058-AC-6 using the public
`CheckedPackageV2` accessor. This test case is Planned and unrun until
QSL-1082 and its QSpec V2 carrier contract land. Read producer fixtures
from their authoritative checkouts at test time; copy no QSL or QSpec file.

## Test Procedure

1. Have the authoritative QSL producer check and emit clauses whose accepted
   execution points are initialization `startup`, handler `tick`, pre
   `adjust`, and post `adjust`. Obtain the QSpec-governed V2 bytes and selected
   model evidence, require `CheckedPackageV2::read` to admit them, and retain
   each actual `(clause id, claim occurrence)` selection. Call the public
   accessor and independently compare each returned model value with the
   QSL accepted binding. Record the producer version and command at run time.
2. Produce two claims sharing one invariant node but with different accepted
   points: initialization `startup` and handler `tick`. Assert the node id,
   kind and body anchor are shared, while the accessor returns two distinct
   points selected by claim. Produce a second handler name and assert that
   exact name appears. A synthetic equivalent is a local API control only.
3. On admitted and crate-local defensive packages, query absent, wrong-form,
   wrong-role, wrong-ordinal and other-node selections; combine faults to
   assert the documented first-refusal order. Remove only the retained
   producer point for one authentic claim in a crate-local defensive probe,
   then substitute points associated with another package, clause and claim
   in turn. Check `MissingExecutionPoint`, with no success or fallback.
4. Compile an external CG-style consumer that uses the public accessor and
   exhaustively matches its result and refusal enum. Repeat calls and clone
   the package, compare values, package equality and canonical identity.
   Inspect for one QSpec-governed carrier, no body JSON read, QSL binding
   copy, `export_bindings` call or new caller point.

## Expected Results

The accessor returns the exact accepted QSL point for each tuple, preserving
the invariant initialization/handler and distinct handler-name cases. Each
invalid selection returns its named typed refusal in order and no point.
Absent or foreign producer evidence returns `MissingExecutionPoint`.
Authoritative producer-origin cases remain UNRUN until QSL-1082 and QSpec
publish the carrier; synthetic packages cannot discharge them.
