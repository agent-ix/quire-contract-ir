---
id: TC-465
title: "Named option admission and cross-reader vectors"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-356
    type: verifies
---
# TC-465: Named option admission and cross-reader vectors

## Description

Read option model declarations through the public strict reader and compare the
production result against the owning QSpec corpus, without copying it.

## Test Procedure

Run self-contained reader regressions for Integer options, canonical equality with
presence optional, aliases, nesting, Clock, malformed inners and exact refusal paths.
Run the named QSpec conformance target against an explicitly selected current
checkout's proposals/checked-package-v2/model-option-type-vectors.json. Exercise
all eight field-type rows and all 33 intake rows, including supported compound
positions and shape/typeRef/presence refusal order. Record the checkout's HEAD in
the gate evidence and report every case identifier and mismatch. Build Semantic IR
and checked-package inputs from the published projection; do not copy vectors or
normalize a refusal into an admission. Run the full IR integration target or wait
for the PR's CI Test step before merge.

## Expected Results

All OP-01..41 cases agree on admission, canonical member domain/identity or the exact
refusal code, cause and selected-document pointer. The ordinary reader regressions
and full integration gate pass, and the FCD leg remains separate.
