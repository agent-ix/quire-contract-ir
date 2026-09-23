---
id: FR-035
title: "Lower complete-V1 contracts into a versioned target-neutral package"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/AD-003
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-028
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-195
    type: references
  - target: ix://agent-ix/quire-specification/FR-330
    type: references
---
# FR-035: Lower complete-V1 contracts into a versioned target-neutral package

## Description

When Contract IR lowers a checked complete-V1 request, the lowerer shall emit a
cycle-free versioned `ContractPackage` with exact selected meaning or one
source-bound terminal refusal for each requested item.

## Inputs

An immutable checked package, requested item set, lowering profile, resource
limits, and exact upstream revision/profile identities.

## Outputs

A canonical `ContractPackage` plus exactly one record for every requested item,
drawn from the closed seven-member vocabulary
[FR-038](./FR-038-consume-checked-package-v2.md) declares: `lowered`,
`unsupported`, `requires_bound`, `invalid_input`, `failed`, `invalid_body` and
`body_incomplete`.

The `ContractPackage` half of this output is a separate obligation from the
per-item records. One lowering call assembles exactly one package from its
`lowered` records alone: each lowered node once, ascending by key, plus the
admitted nodes those reach that were not themselves lowered, so every reference
inside the package resolves inside it. A refused request contributes no node of its own; its node appears
only when a lowered node reaches it, as exact reached meaning. A reached node
carries its family and source correspondence but no `ir_id` or closure of its
own. The package records the source package identity and its schema version
(`quire.contract-ir.contract-package/v1`); its canonical bytes are the RFC 8785
canonical bytes of the whole package, the same canonical form
[FR-038](./FR-038-consume-checked-package-v2.md) requires of the checked
package, and its identity is the SHA-256 of those bytes in that version's
domain. Nodes refer to one another only by key, so the package is cycle-free
as a value. This package is the single
Contract IR input that complete-V1 code generation consumes.

## Behavior

The package shall represent scalar and composite types, bounded domains, typed
expressions, pure functions, model identities and relations, state frames and
transitions, temporal formulas, protocol control/queues/obligations, claims,
and source maps. Nodes shall carry a schema version, semantic type, stable node
identity, exact source loci, and identity-based references. A lowered item shall
include its reachable compatible dependencies and canonical semantic digest.
Unknown tagged nodes may survive transport but consumers that lack their
declared version shall refuse them. When a requested item exceeds a declared
resource limit, the lowerer shall return `failed` for that item without exposing
a substitute node. The lowerer shall not substitute a node,
change a source/type/anchor/bound, or affect a sibling disposition.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-035-AC-1 | Every V1-BACK-001 through V1-BACK-004 and V1-BACK-014 request has an exact semantic vector or a per-item refusal. | Test (TC-044) |
| FR-035-AC-2 | Mutation of source, type, anchor, identity, bound, dependency, or version refuses before a backend artifact is emitted. | Test (TC-044) |
| FR-035-AC-3 | Mixed supported and unsupported requests retain independent sibling records and expose no placeholder semantics. | Test (TC-044) |
| FR-035-AC-4 | Every reference resolves by stable identity to a reachable version-compatible node and every represented node has exact source correspondence. | Test (TC-044) |
| FR-035-AC-5 | One lowering call over a mixed request emits a single canonical cycle-free versioned `ContractPackage` carrying every `lowered` node of that call, whose canonical bytes and digest are stable across repeated identical calls and change when any represented node changes. | Test (TC-047) |

## Dependencies

[FR-028](./FR-028-separate-cycle-free-contract-model.md) owns the cycle-free
model boundary. QSpec FR-195 and I12 own the normative complete-V1 lowering and
ContractPackage semantics; this requirement implements them in Contract IR.
