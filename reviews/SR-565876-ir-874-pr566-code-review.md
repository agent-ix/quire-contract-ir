---
id: SR-565876
title: IR-874 PR 566 code-review review
type: SpecReview
analysis: code-review
scope: 'agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/flat_wire.rs,
  crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs,
  tests/it/checked_package_v2_protocol_body.rs, tests/it/support/checked_package.rs;
  IR-874 PR #566 head 30ac95e4'
review_set: subset
---

## Summary

The protocol validation slice has a refusal-location defect for malformed enum values.

## Verdict

**CONDITIONAL** — The protocol validation slice has a refusal-location defect for malformed enum values.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Invalid protocol enum spellings refuse at the whole body, rather than the offending term | crates/quire-contract-model/src/checked_package/v2/flat_wire.rs:145 |

## Scope

- FR-038-AC-281 (examined): A reader-admitted protocol member has a `quire.op.protocol.control` body root, exactly one `protocol_profile` law matching the selected lock row, `null` mode, no leaves, Boolean result and semantic types, and a `profile_operator` member. Under the published `quire.protocol.finite-global/v1` selection the admitted operators are exactly `protocol`, `role`, `channel`, `queue`, `finish`, `terminal`, `sequence`, `choice`, `parallel`, `repeat`, `await`, `check`, `send`, `receive`, `attempt`, `effect`, `event`, `commit`, `fence` and `compensate`; an unknown operator refuses `operation-member-mismatch
- FR-038-AC-282 (examined): Each of the seven protocol member forms admits only the FR-464, FR-465 or FR-466 fixed argument names, order and term shapes for its selected operator: thirteen clause bindings, the role, channel, queue, finish and terminal layouts, thirteen control layouts and thirteen compensation bindings. A missing, extra, reordered, wrongly shaped or misspelled binding refuses `malformed_wire`; an optional Group admits zero or one member only, an interval has non-negative integer bounds with lower no greater than upper, and a set or sorted-record list repeated or out of its specified ascending order refus
- FR-038-AC-284 (examined): The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending node-id digest. A package with both a refinement-step and protocol defect reports the refinement defect; one with both a
- FR-038-AC-286 (examined): A role admits the FR-465 static and replicated shapes only, including a model of an eligible form, replication maximum at least one, a reference-typed binder and an `until` term exactly for an `until` lifetime; a channel admits only same-clause role endpoints, an eligible message type, explicit delivery and ordering, and a FIFO key with equality-stable type; a queue admits one channel, literal or symbolic capacity, explicit overflow and a loss selector exactly for `loss`. The reader admits the positive variants and refuses the wrong form, missing premise, wrong binder type, non-Boolean lifetim
- FR-038-AC-287 (examined): A clause names exactly one `finish` obligation whose closing-record binder and Boolean constraint read only the binders available after the root control; its optional `terminal` obligation admits `when` with one Boolean condition or `any` with none. A missing or extra condition refuses `malformed_wire`, a non-Boolean condition or constraint refuses `ill_typed`/`non-boolean-root`, an unjoined branch binder read by finish refuses `missing_declaration`/`missing-name`, and a terminal target in the finish position or an unattached finish refuses by the FR-464 membership rules.

Rust lane: rust-review applied; focused protocol_body unit tests 9/9 pass. No spec files changed in this diff.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified at PR #566 head 156e59c0; see new term-pointer logic or narrowed trace tag. |

## Supplemental verification at 79e29b9c

Reviewed only the added IR987 golden and admitted-result test changes since 156e59c0. The new `scalar.json` equals the canonical bytes of `fixture()` in the focused test; that fixture now contains 30 nodes versus 27, with 26 shared node ids having unchanged full node content, one rekeyed protocol clause and three added nodes. The changed lock profile selection matches the new declared protocol fixture. `context.json` equals the canonical bytes of the result contract; its changed top-level members are package source, identity digest, and source map. The test's scalar operand, result context, provenance and missing-conversion assertions remain unchanged. The bounded-read work threshold changed from 263 to 339 and is still checked against the exact admitted/incomplete boundary. The renamed focused test passed 1/1; `git diff --check` passed. No new findings.

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | A duplicate protocol declaration preempts an earlier protocol placement refusal | crates/quire-contract-model/src/checked_package/v2/mod.rs:2656 |

A package can contain a protocol node whose body applies another root operator (QSpec TC-946 V2) and two distinct protocol nodes with the same declaration path (K7). Strict wire admits the well-shaped application, structural validation has no protocol declaration-body rule, and rekeyed application nodes pass the key stage. `validate_declaration_names` includes all declarations and returns `ambiguous_declaration` before `validate_protocol_nodes` sees the placement defect. QSpec TC-946 O2 and FR-038-AC-284 require `ill_typed`/`operator-ineligible` from placement first. The fix must preserve the existing earlier declaration ordering for non-protocol families.

## Dispositions (round 2)

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | still-open | No fix for the protocol placement versus duplicate-declaration order exists at 79e29b9c. |

## Dispositions (round 3)

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | At 5b1680e2, the generic name sweep skips protocol nodes, and the protocol step checks protocol-involved duplicates after placement and operator checks. The combined-defect test now yields IllTyped/OperatorIneligible; a valid duplicate yields AmbiguousDeclaration. Existing non-protocol declaration precedence tests still pass. |

## Supplemental review at 39996a46

Reviewed only the hosted-CI repair diff since 57a5afd3. The selected protocol profile is borrowed as `CheckedArtifactRef`; the law definition is compared by its exact two-field `{authority, identity}` shape and values, matching the typed reference contract without serializing the lock. The `ProtocolLayoutWireDefect` rename changes no refusal branch. The claim-terminal tests replace the former fixed insertion indices with the length of their immutable base fixture; node-key, package-id, source-map, dependency, duplicate-locus and precedence assertions remain in place. Frame, TC-226, complete-v1 edge and byte oracles retain explicit exact-budget or exact-pointer assertions. The byte oracle changes only the measured canonical length.

Focused reviewer checks: protocol-body unit tests 9/9; claim-terminal integration tests 11/11; the selected-frame-work, scoped-delete-work, TC-226 tree-work, complete-v1 edge and contract-package byte-limit cases each passed 1/1; `git diff --check` passed. No new findings.
