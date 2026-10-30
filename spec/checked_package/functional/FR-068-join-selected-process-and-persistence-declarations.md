---
id: FR-068
title: "Join selected process and persistence-interface declarations"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-064
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
---

# FR-068: Join selected process and persistence-interface declarations

## Description

When a checked-package V2 node declares a selected model `process` or
`persistence_interface`, the Contract IR strict reader SHALL join its exact
`ModelOwner` to a declaration of the same meaning and IR node identity in the
authoritative selected domain document under QSpec FR-322. The join SHALL
retain the admitted node and package identities. It SHALL NOT infer a
declaration from the wire node's tag or semantic form alone.

## Inputs

Untrusted V2 package bytes, the locked model selections, their supplied
Semantic IR 2.0.0 documents and bounded reader limits of
[FR-038](./FR-038-consume-checked-package-v2.md). QSpec FR-322-AC-63,
FR-322-AC-65 and FR-322-AC-72 define the authoritative owner and member
contract; their published vectors remain in the QSpec repository.

## Outputs

An admitted typed graph whose model declaration nodes and selected owners
retain their identities, or one typed, located refusal with no admitted
package.

## Behavior

For each selected domain document, the reader SHALL classify declarations by
their admitted Semantic IR meanings. It SHALL validate the closed structural
`ModelDeclarationNode` id for a `model`/`process` node only from a declared
process and for a `model`/`persistence_interface` node only from a declared
persistence interface, using that declaration's exact IR node identity and
the selected package name in its `ModelOwner`. A declared state machine
and a declared record value type SHALL retain their own distinct structural ids under the same rule. The reader SHALL re-validate the form-to-meaning
match at the owner join even if earlier admission classified the document.

The reader SHALL admit a self-typed, empty-body process declaration node
owned by the selected `Flow` process and a persistence-interface declaration
node owned by the selected `Store` interface. It SHALL resolve `Flow`'s
declared field `n` and operation `go` through the existing model-member step,
and SHALL admit the derived `Reference<Flow>` node whose target is the exact
`Flow` declaration node. A process and a state machine SHALL retain the
`object` family in their admitted node forms. The existing selected `Gate`
state-machine and `Address` record-value-type paths SHALL remain admitted,
including `Reference<Gate>`, `Account.address` typed at the `Address` node,
`Address.street` typed at `Text[0, 64; nfc]`, the record node's `record`
family, and an invariant
clause function owned by `Address` where their QSpec forms permit them.

If a node's owner names an absent or unselected declaration, or the selected
declaration has a different meaning from its node form, then the reader SHALL
refuse `missing_declaration`/`missing-selection` at the owner join. A missing
required owner SHALL refuse `malformed_wire` at strict wire validation. The
reader SHALL preserve QSpec's earlier FR-154 intake refusal for an invalid
field type and its later `ill_typed`/`operator-ineligible` refusal for an
unknown field name. It SHALL NOT accept a wire form merely because its
`owner.node` exists among other selected meanings.

After this owner join, the [FR-064](./FR-064-bind-protocol-clause-to-owner-model.md)
clause check SHALL still reject a selected process, state-machine or record-value-type
node used as a protocol clause's `owner_model` with
`invalid_model_binding`/`malformed-declaration` at that edge. That clause
check SHALL NOT bypass or weaken the earlier owner join.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-068-AC-1 | A public read using a real selected domain document admits self-typed `model`/`process` `Flow`, `model`/`state_machine` `Gate`, `model`/`record_value_type` `Address` and `model`/`persistence_interface` `Store` declaration nodes, each using its exact declared structural path and selected `ModelOwner`. Canonical write/read/read retains each structural node id and package name. No node is admitted solely from its wire tag or semantic form. | Test |
| FR-068-AC-2 | The selected `Flow` process's field `n` and operation `go` resolve through QSpec's model-member step; `Reference<Flow>` and `Reference<Gate>` target their exact process and state-machine declaration nodes. The selected `Address` record has family `record`, remains the type of `Account.address`, and its field `street` resolves to `Text[0, 64; nfc]`; an invariant clause function owned by `Address` passes the owner join. | Test |
| FR-068-AC-3 | With valid wire, graph and structural ids, a `process` node owned by an object type, a `state_machine` node owned by a process, a `record_value_type` node owned by an object type, and a `persistence_interface` node owned by a state machine each refuse `missing_declaration`/`missing-selection` at the owner join. Each form owned by a node absent from the selected document and each form with no selected owner also refuses there; an omitted required `owner` refuses earlier as `malformed_wire`. | Test |
| FR-068-AC-4 | An unknown field on a valid record node refuses `ill_typed`/`operator-ineligible`; a record field whose `typeRef` names an event type, process, state machine, persistence interface or namespace refuses `invalid_model_binding`/`malformed-declaration` at FR-154 intake before member derivation. A record node used as an object reference target refuses `invalid_package`/`invalid-derived-shape`. | Test |
| FR-068-AC-5 | A selected `process`, `state_machine` or `record_value_type` node named as a protocol clause's local `owner_model` reaches [FR-064-AC-4](./FR-064-bind-protocol-clause-to-owner-model.md) and refuses `invalid_model_binding`/`malformed-declaration` at that clause edge; a genuinely unselected candidate still refuses at the earlier owner join. | Test |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns strict V2 admission,
  selected-document evidence, structural-id checks and check order.
- [FR-064](./FR-064-bind-protocol-clause-to-owner-model.md) owns the later
  clause-edge refusal and its IR-1053 selected wrong-kind controls.
- QSpec FR-322-AC-63, FR-322-AC-65 and FR-322-AC-72 own the positive and
  adverse forms; QSpec TC-280 owns the published process, state-machine and
  persistence owner cases. No QSpec artifact is copied here.

## Status

Planned. Existing state-machine and record-value-type admission does not
discharge the remaining process and persistence-interface owner joins or
the complete QSpec FR-322-AC-72 process-member and reference behavior.
