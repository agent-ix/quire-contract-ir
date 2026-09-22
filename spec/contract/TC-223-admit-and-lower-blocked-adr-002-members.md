---
id: TC-223
title: "Admit and lower each ADR-002 member once its wire-carrier requirement publishes"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-344
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/issues/120
    type: references
---

# TC-223: Admit and lower each ADR-002 member once its wire-carrier requirement publishes

## Description

Verify FR-344-AC-3 through FR-344-AC-7: admission, refusal and lowering
for the five ADR-002 2.0.0 members this reader has no wire encoding for —
supertype list, abstractness flag, subsets edge, redefines edge and
population node.

This test case cannot be written today. Each of the five acceptance
criteria it verifies names a QSpec `quire.checked-package/v2` wire-carrier
requirement — the FR-340 counterpart for that member — as a precondition,
and none is published as of this writing (2026-09-21). Writing a concrete
fixture or a concrete `(node_tag, semantic_form, body)` shape ahead of
that publication would be exactly the "guessed member shape" issue #120
and this FR's Inputs section rule out. This file records the shape the
procedure will take once each requirement exists, not the procedure
itself.

## Test Procedure

For each of supertype list, abstractness flag, subsets edge, redefines
edge and population node, once its own QSpec wire-carrier requirement
fixes the member's `(node_tag, semantic_form, body)` encoding:

1. Construct one fixture using exactly that requirement's published shape
   on an otherwise-admitted V2 document; admit it and inspect the
   resulting declaration.
2. Construct one malformed instance of the same shape (a reference to no
   declared node, a value outside the member's declared type, or — for
   supertypes/redefines — a cycle or an upper-bound-widening redefinition)
   and confirm the requirement's own named refusal code and locus.
3. Request lowering of a node carrying the member and confirm the result
   is exactly one of FR-038's seven existing lowering dispositions
   (`lowered`, `unsupported`, `requires_bound`, `invalid_input`, `failed`,
   `invalid_body`, `body_incomplete`) — this FR does not add an eighth.

For subsets, redefines and population specifically, an additional
precondition applies before step 1 can use a real (non-synthetic) fixture:
FCD's own extraction pipeline does not yet produce a real instance of any
of the three from a spec artifact (`filament-core-data#193`'s "Out of
scope" notes). Until that FCD follow-up lands, steps 1–3 for these three
members can only run against a hand-constructed fixture conforming to the
published wire shape, not one traceable to a real spec.

## Expected Results

Not yet determined for any of the five members: each depends on its own
not-yet-published wire-carrier requirement's exact shape and refusal
causes, which this test case does not invent.

## Status

Planned; blocked on five independent QSpec publications (one
`quire.checked-package/v2` wire-carrier requirement per member), none of
which exists as of this writing. No reader or lowerer implementation is
proposed until at least one publishes; this file is reserved so the
Coverage tooling has a named, traceable row rather than an absent one.
