---
id: SR-11030
title: Code and Rust review of IR-782 protocol reference reader
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/identity.rs,
  crates/quire-contract-model/src/checked_package/v2/lower.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs,
  crates/quire-contract-model/src/checked_package/v2/operations.rs, crates/quire-contract-model/src/checked_package/v2/owner.rs,
  crates/quire-contract-model/src/checked_package/v2/structural.rs, crates/quire-contract-model/src/checked_package/v2/vocabulary.rs,
  tests/it/checked_package_v2_protocol_reference.rs, tests/it/main.rs
review_set: subset
relationships:
- target: ix://agent-ix/quire-contract-ir/FR-038
  type: references
---

## Summary

Ticket IR-782, PR #396, frozen. Reviewed exactly the nine changed files and FR-038-AC-239 through AC-241 / TC-048, with QSpec FR-322-AC-66, FR-330-AC-4 and TC-931 as authority.

## Examined scope

- `FR-038-AC-239` (examined; `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): PLANNED/UNRUN (IR-782). A package with a `protocol`/`protocol_reference` carrier whose closed reference body targets an exact local `protocol` node, with one `expression` occurrence, the same semantic type and exactly that target as its dependency, admits through the public V2 reader as `ProtocolForm::ProtocolReference`. Both a `protocol_clause` target and a different protocol form (`role` or `control`) admit; a reader that accepts only `protocol_clause` fails the second control. For each admitted target, the structural key, identity projection and package id are independently recomputed; chan
- `FR-038-AC-240` (examined; `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): PLANNED/UNRUN (IR-782). An extra dependency or a dependency unequal to the carrier's body target refuses `invalid_semantic_graph` at `/semantic_graph/nodes/{i}/dependencies` in graph-shape validation; a stale carrier key with matching dependency refuses `invalid_package`/`stale-node-key` before the local target join. With both body target and sole dependency changed together and identities independently recomputed, an absent local target refuses `missing_declaration`/`missing-name` at `/semantic_graph/nodes/{i}/body/target`, even if an admitted selected dependency contains that key. A present
- `FR-038-AC-241` (examined; `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): PLANNED/UNRUN (IR-782). A carrier with a non-reference or `dependency_reference` body or an extra body member fails the closed V2 schema. A carrier with an extra occurrence, declaration, owner, nominal preimage or recursion group refuses at its offending member before target resolution. An ordinary node holding a generic reference remains an ordinary node; no second protocol-reference form or separate collection is admitted.
- `TC-048` (examined; `spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md`): For FR-038-AC-239 through FR-038-AC-241, construct a local
`protocol`/`protocol_clause` target and a `protocol`/`protocol_reference`
carrier using the public typed vocabulary. Give the carrier one `expression`
occurrence, a closed `reference` body, the target's exact typed key as body
target and sole dependency, and the target's semantic type. Independently
compute the carrier's QSL FR-092 structural preimage and canonical key,
mirror it in the graph identity projection, refresh the package id, and read
the package through `CheckedPackageV2::read`. Check the typed form, exact
local target, sin
- `FR-322-AC-66` (context_only; `quire-specification/spec/objects/interfaces/FR-322-checked-package-artifact.md`): The worked package with a `protocol`/`protocol_reference` carrier to its local `protocol_clause` target is admitted with one graph collection; its body target, sole dependency and semantic type bind that exact protocol node and its recomputed key changes when the target changes. With the body target and sole dependency changed together and identities recomputed, a missing local target refuses `missing_declaration`/`missing-name` even when a selected dependency contains that key, and a present non-protocol target or unequal semantic type refuses `ill_typed`/`operator-ineligible`. A non-referenc
- `FR-330-AC-4` (context_only; `quire-specification/spec/objects/interfaces/FR-330-contract-package.md`): A package has exactly one semantic node collection ordered by the FR-322 typed node key; each node occurs once, local references resolve to compatible nodes, dependency references retain FR-322 selection semantics, a `protocol`/`protocol_reference` carrier binds by its body target to a local `protocol` node under FR-322-AC-66, and recursion is represented only by permitted keyed references.
- `TC-931` (context_only; `quire-specification/spec/test-cases/TC-931-contract-package-single-semantic-graph.md`): Check that the ContractPackage representation uses the I04 node-tag, key and
reference rules and does not serialize separate graph collections or a separate
`protocol_references` collection.

## Test Procedure

Lower an admitted package containing a type, a typed expression, a protocol
clause, a selected dependency, a local function, and a dependency-exported
function with package-independent signature types. Include a function-call
callee `dependency_reference` to the exported function and an unrelated
lowered sibling. Inspect the ContractPackage and compare its node collection,
identities, r
- `crates/quire-contract-model/src/checked_package/v2/identity.rs` (examined; `crates/quire-contract-model/src/checked_package/v2/identity.rs`): | ProtocolForm::ProtocolReference
- `crates/quire-contract-model/src/checked_package/v2/lower.rs` (examined; `crates/quire-contract-model/src/checked_package/v2/lower.rs`): | ProtocolForm::ProtocolReference
- `crates/quire-contract-model/src/checked_package/v2/mod.rs` (examined; `crates/quire-contract-model/src/checked_package/v2/mod.rs`): K::Protocol(ProtocolForm::ProtocolReference) => true,
- `crates/quire-contract-model/src/checked_package/v2/operations.rs` (examined; `crates/quire-contract-model/src/checked_package/v2/operations.rs`): | ProtocolForm::ProtocolReference
- `crates/quire-contract-model/src/checked_package/v2/owner.rs` (examined; `crates/quire-contract-model/src/checked_package/v2/owner.rs`): if node.node_tag.as_ref == "protocol"
- `crates/quire-contract-model/src/checked_package/v2/structural.rs` (examined; `crates/quire-contract-model/src/checked_package/v2/structural.rs`): Occurrences,
- `crates/quire-contract-model/src/checked_package/v2/vocabulary.rs` (examined; `crates/quire-contract-model/src/checked_package/v2/vocabulary.rs`): ProtocolReference => "protocol_reference",
- `tests/it/checked_package_v2_protocol_reference.rs` (examined; `tests/it/checked_package_v2_protocol_reference.rs`): use crate::support::checked_package::{
- `tests/it/main.rs` (examined; `tests/it/main.rs`): mod checked_package_v2_protocol_reference;

## Verdict

PASS: no findings in the frozen diff. Six focused public-reader tests passed. Production grammar, shape, identity, owner, target join, lowering and operation dispatch were checked. No duplication, unsafe, panic, stub or weakened gate was found in the diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

