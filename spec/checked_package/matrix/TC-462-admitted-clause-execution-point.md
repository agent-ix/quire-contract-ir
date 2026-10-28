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

Verify FR-058-AC-1 and AC-3 through AC-6, the QSpec-aligned AC-8 through
AC-10, and the explicit SUPERSEDED dispositions of AC-2 and AC-7, using the
public `CheckedPackageV2` accessor. This test case is Planned and unrun until
IR-1040 wire admission and IR-1013 accessor implementation. Read producer
fixtures from their authoritative checkouts at test time; copy no QSL or
QSpec file.

Pass the selected clause id and one owner-bound claim argument,
`claim: (&CheckedNodeId, &CheckedOccurrence)`, containing the claim owner's
id and its role/ordinal. The public call has exactly the package receiver,
selected clause id and that claim argument. The tuple uses only the public
reference types named by FR-058-AC-5. No unbound-occurrence accessor
signature or caller-supplied point is allowed.

The V2 accessor returns `quire_contract_model::CheckedExecutionPoint` with
the exact nonempty QSpec point name. Existing authored-contract consumers
continue to use `ExecutionPoint` and its identifier-validated `AnchorName`
under FR-011-AC-3. The test SHALL NOT convert between these two point types
or treat an `AnchorName` conversion failure as a V2 accessor refusal.

## Test Procedure

1. Have the authoritative QSL producer emit separate point-qualified clause
   nodes for initialization `startup`, handler `tick`, pre `adjust` and post
   `adjust`. Read QSpec-governed V2 bytes with selected model evidence, then
   call the public accessor with each authentic owner-bound claim and compare the exact
   model variant and name against the QSL checked binding. Record producer
   version and command at run time.
2. For invariant and constraint, produce equal conditions and body anchors
   with initialization and handler points. Assert that each point produces a
   distinct rederived node id and package id; each node's authentic claims
   return only that node's exact point. Change only the handler name and
   rekey, then assert the changed result and identity. Do not assert that
   different points can live on one clause node (SUPERSEDED AC-2). Separately,
   construct an otherwise admissible V2 reader control with a nonempty point
   name outside FR-011's identifier grammar, such as `tick now`, and verify
   the accessor returns it unchanged through `CheckedExecutionPoint`. This
   control does not give QSL producer-origin credit.
3. Query absent, wrong-form, wrong-role, wrong-ordinal and other-node
   selections, including combined faults, to check `UnknownClause`,
   `NotStateClause`, `MissingClaimOccurrence`, then `MissingExecutionPoint`
   in order. For the other-node case, use two state-clause nodes that each
   admit `claim` with the same role and ordinal; pair the other node's id
   with its claim while selecting the pointed node. Expect
   `MissingClaimOccurrence` and no point even though the role and ordinal
   match. Repeat with a different role and ordinal, and with an absent
   selected clause id or a present non-state-clause selected id to verify
   the earlier refusals still win. Assert the typed error retains the
   supplied selected id, claim owner and occurrence. Query two authentic
   claims of one pointed node: both return its one retained point. An
   unpointed node returns `MissingExecutionPoint` for every authentic claim.
4. Duplicate the point JSON member, add a malformed/extra member or put a
   point on another operation member, rekeying where needed. Check the
   strict-wire duplicate and FR-040 schema refusals, exact code/cause/locus,
   no admitted package or first-wins point. A non-claim or other-node claim
   refuses at accessor authenticity, not a fictional point-entry target
   (SUPERSEDED AC-7). For invariant/constraint test initialization, handler
   and the inapplicable pre/post variants; for precondition/postcondition test
   both pre and post, plus the inapplicable initialization/handler variants.
   Exercise operation-name and clause-anchor agreement under FR-040.
5. Compile an external CG-style consumer using only the public accessor and
   exhaustively matching its result/refusal enum. Pass a selected clause
   reference and one owner-bound claim tuple argument, using only public
   `&CheckedPackageV2`, `&CheckedNodeId` and `&CheckedOccurrence` references.
   Repeat calls, clone the
   package and compare values, package equality and canonical identity.
   Drop source bytes and repeat. Inspect for one QSpec-governed node carrier,
   no body JSON read at accessor time, QSL binding copy, `export_bindings`
   call, caller point or alternate compatibility path.

## Expected Results

The accessor returns the exact point retained from the selected admitted
clause node for every authentic claim, including a QSpec-admitted name that
FR-011's `AnchorName` would refuse; two different points imply different
clause node identities. Invalid selections, including a same-valued claim
paired with another node's id, return the named typed refusal
in order. Absent points return `MissingExecutionPoint`. Malformed or
inapplicable node-body carriers refuse admission under FR-040 with no
first-wins value. AC-2 and AC-7's original occurrence-keyed scenarios are
SUPERSEDED and receive no implementation credit. Producer-origin cases stay
UNRUN until QSL emission and IR code land; synthetic packages cannot
discharge them.
