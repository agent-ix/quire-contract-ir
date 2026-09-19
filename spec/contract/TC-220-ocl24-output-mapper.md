---
id: TC-220
title: "Bounded OCL 2.4 output mapper conforms"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-341
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-342
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-343
    type: verifies
  - target: ix://agent-ix/quire-specification/TC-151
    type: references
  - target: ix://agent-ix/quire-specification/TC-155
    type: references
---
# TC-220: Bounded OCL 2.4 output mapper conforms

## Description

Verify that the pure Rust OCL mapper binds one exact admitted request, emits
only the accepted bounded OCL 2.4 fragment, and accounts for every unsupported
or conditional source fact without approximation or foreign-runtime evidence.

## Test Procedure

Construct strict bound packages and exact OCL correspondence catalogs for total
Boolean/scalar invariants, bounded ConfigVersion values, zero-argument operation
pre/post reads, and bounded Sequence queries. Run them through the public common
coordinator and package assembler. Mutate every source-package, native, model,
semantic, target-profile, context, anchor, symbol, field, operation, dependency,
integer-bound, state-observation, collection-order, local-scope, source-state,
work, and output-identity axis independently. Exercise 0, 1000, -1, and 1001;
duplicate values; every supported operator; missing/duplicate/ambiguous/foreign/
stale/invalid/over-limit correspondence; optional/invalid/rational/index/
unsupported collection/graph/temporal/protocol nodes; ParentPrecedes with and
without complete relationship mapping; cancellation and exact/just-over work
limits. Repeat equal inputs while varying path, time, locale and hypothetical
parser/tool/observer outcomes.

## Expected Results

Only wholly admitted expressions emit canonical Complete OCL regions, with
native-domain conditions wherever required and exact typed dependencies.
Binding mismatches and invalid catalogs refuse before dispatch. Every other
unsupported case yields the specified whole-obligation unrepresented/refused
candidate with no substitute bytes. Equal semantic inputs yield byte-identical
records and packages, and no generated or observed OCL becomes native source,
truth, preservation, or cross-target input.
