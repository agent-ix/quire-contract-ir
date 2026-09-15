---
id: TC-045
title: "Exact backend capability negotiation conforms"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-036
    type: verifies
  - target: ix://agent-ix/quire-specification/TC-218
    type: references
---
# TC-045: Exact backend capability negotiation conforms

## Description

Verify that provider negotiation completes prior to artifact emission and that
finite Kani domains exactly derive from admitted model domains and proof bounds.

## Test Procedure

Negotiate positive boundary domains and mutate each bound, encoding,
capability, tool lock, option, dependency, and requested claim. Exercise
missing bounds, unsupported graph/temporal/protocol encodings, and mixed item
requests before inspecting any generated artifact.

## Expected Results

Every supported artifact has exact independently traceable domains and pins.
Every missing or unsupported item has a separate `requires_bound`,
`unsupported`, or `invalid_request` record and no generated artifact.
