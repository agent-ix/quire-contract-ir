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

A canonical `ContractPackage` plus exactly one of `lowered`, `unsupported`,
`requires_bound`, `invalid_input`, or `failed` for every requested item.

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

## Dependencies

[FR-028](./FR-028-separate-cycle-free-contract-model.md) owns the cycle-free
model boundary. QSpec FR-195 and I12 own the normative complete-V1 lowering and
ContractPackage semantics; this requirement implements them in Contract IR.
