---
id: TC-464
title: "Complete effective operation census through public admission"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-355
    type: verifies
---
# TC-464: Complete effective operation census through public admission

## Description

Exercise the admitted model operation census independently of executable bodies,
operation anchors and dispatch sidecars.

## Test Procedure

Read genuine selected Semantic IR model documents through `CheckedPackageV2::read`
with authentic structural declaration nodes and selection evidence. Request the
census of a two-operation context without bodies or anchors, an empty context, and
inherited/redefined contexts including a diamond. Check producer parameter order,
canonical member domains, result presence and unsupported domains. Independently
exercise ambiguous names, missing and duplicate declaring nodes, foreign/unknown/
wrong-kind contexts, and sufficient versus one-below/zero work provisions. Compile
and execute the calls through the public model facade; run TC-058 and TC-456 for
inventory and portable public names.

## Expected Results

The full effective declaration set is returned in name order, including unreferenced
and unimplemented operations. Parameter and result facts equal the selected model
records and retain their declaring/context identities. Every adverse read returns
its owning typed refusal without a partial census. Existing callable and dispatch
contracts, model dependencies and input wire semantics remain unchanged.
