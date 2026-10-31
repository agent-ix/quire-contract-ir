---
id: FR-349
title: "Retain typed checked-package V2 terms"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-400
    type: references
---
# FR-349: Retain typed checked-package V2 terms

## Description

When the V2 reader admits a checked package, Contract IR SHALL retain each
semantic node body as a closed, typed term whose variant and members correspond
to the QSpec body production selected by the node's tag and form. The retained
term SHALL let IR consumers distinguish the admitted productions without
looking up the `term` string in an untyped JSON object.

The V2 wire carries one complete body per `semantic_graph.nodes` entry and the package's required name. The reader and writer retain the structural node ids and bodies; neither computes an identity projection.

## Behavior

The reader SHALL distinguish literal, reference, application, aggregate and
dependency-reference terms, and binding members nested under aggregates.
The reader SHALL distinguish the frame, abstraction-relation and extension-form bodies
selected by their node kinds. Each variant SHALL retain its production's
required members with typed node identities and closed member kinds where
those types exist in the IR model. An in-group `group_reference` is resolved through the graph member ordinal; it is not a content-hash preimage.

The reader SHALL validate the wire grammar and report the existing first
refusal and path before constructing an admitted typed body. Serialization and
canonical encoding of a typed body SHALL yield the same JSON and JCS bytes as
the corresponding QSpec wire body. The reader SHALL preserve the existing
bounded-input and nonrecursive processing of deeply nested untrusted terms.

When a consumer reads a literal from an admitted `CheckedPackageV2`, Contract
IR SHALL expose a closed typed value paired with its admitted `LiteralKind`
and `CheckedNodeId` type. The read SHALL preserve every QSpec FR-322 literal
value, including exact numeric magnitude and decimal scale, without requiring
the consumer to inspect `serde_json::Value`, parse a display string, or
revalidate the literal. Package serialization SHALL retain the QSpec wire
encoding; the read adds no second literal wire form.

When a consumer reads an admitted binding, Contract IR SHALL expose
`CheckedBindingName::kind` as a public closed vocabulary with distinct
`parameters`, `body`, `optional`, `clause`, and `other` cases. The `other`
case SHALL retain the exact admitted name, so a consumer cannot confuse it
with the four contextual names.

When a consumer reads an admitted application operation, Contract IR SHALL
expose one general borrowed typed view of its complete admitted record:
identity, mode, law definitions in order, member kind and typed member
fields, constraints, and leaves in order. Contract IR SHALL expose member
and constraint choices as closed typed variants. The view SHALL require neither
`serde_json::Value` nor a string-key lookup by its consumer. The existing
specific operation views SHALL retain their meanings. If the operation has
no reader-installed admission view, the general read SHALL return
`CheckedOperationViewError::NotAdmitted`; direct deserialization and a test
fixture do not confer package admission.

QSL [FR-400](ix://agent-ix/quire-spec-language/FR-400) places QSL checked
expression terms in `quire-checked`, with construction restricted to the QSL
checker. Contract IR SHALL retain and read the QSpec checked-package V2 wire
through its own admitted `CheckedTerm` vocabulary and expose read-only typed
projections at its public boundary. Contract IR SHALL NOT
construct a QSL checked expression, duplicate the QSL checker or evaluator,
export a QSL `TermOperation` mirror, or require `quire-checked` to depend on
IR. QSL owns conversion of admitted IR reads into its checked expression
input and owns execution semantics; IR owns V2 admission and typed reads.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-349-AC-1 | Reading a conforming V2 package returns a typed body variant for every admitted root production; aggregate members and application arguments expose typed nested productions, including bindings and dependency references. | Test |
| FR-349-AC-2 | Frame, abstraction-relation and extension-form nodes expose distinct typed bodies with their admitted members, and no admitted body is selected by a caller's string-key lookup. | Test |
| FR-349-AC-3 | Each admitted single-body package serializes and canonically encodes to the published QSpec bytes and structural ids, with each body shipped once in semantic_graph.nodes. Typed-term retention does not change those body bytes or the retained structural ids. | Test |
| FR-349-AC-4 | A malformed or unknown term still refuses at the same first code, cause and JSON pointer as the existing reader, before an admitted typed package is returned. | Test |
| FR-349-AC-5 | A nested term at the configured input limit is processed without call-stack overflow, and an over-limit term refuses under the existing read limit. | Test |
| FR-349-AC-6 | For every admitted QSpec FR-322 literal kind, an external crate reads its closed kind, typed value and type node without inspecting `serde_json::Value` or parsing a display string. Exact numeric magnitude and decimal scale survive the read; serializing the package still yields the QSpec bytes. | Test |
| FR-349-AC-7 | An external crate reads `CheckedBindingName::kind` for `parameters`, `body`, `optional`, `clause`, and an admitted other name; the other name remains available verbatim, and the first four cannot be mistaken for `other`. | Test |
| FR-349-AC-8 | An external crate reads every admitted operation member and constraint production, identity, mode, ordered laws and ordered leaves through one borrowed typed view. The view contains no JSON payload or map-key lookup; a direct-deserialized or `test-support` fixture operation returns `NotAdmitted`, while the existing specific getters retain their prior results. | Test |
| FR-349-AC-9 | The FR-019 exact public-item inventory and external compile probes show that IR exports the admitted read views but no QSL checked-expression constructor, `TermOperation` mirror, checker, or evaluator. Dependency-graph checks show that `quire-contract-model` does not depend on `quire-checked` or QSL and `quire-checked` does not depend on IR. | Test |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns the reader order,
  limits, refusal envelope and structural graph.
- QSpec FR-322 owns the admitted V2 body grammar, wire members and structural ids.
- QSL FR-400 owns checked-expression placement and constructor authority;
  QSL FR-390 owns evaluation parity over an original and contract-read package.
