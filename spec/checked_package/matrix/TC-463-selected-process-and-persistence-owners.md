---
id: TC-463
title: "Selected process and persistence-interface owner joins"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-068
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-064
    type: verifies
  - target: ix://agent-ix/quire-specification/TC-280
    type: references
---

# TC-463: Selected process and persistence-interface owner joins

## Description

Verify [FR-068](../functional/FR-068-join-selected-process-and-persistence-declarations.md)
through Contract IR's public checked-package V2 reader and typed writer,
using real selected Semantic IR documents. Exercise QSpec FR-322-AC-63,
FR-322-AC-65 and FR-322-AC-72 without copying QSpec schemas or fixtures.
Then verify the selected wrong-kind clause-edge controls required by
[FR-064-AC-4](../functional/FR-064-bind-protocol-clause-to-owner-model.md).

## Test Procedure

Select documents that declare `Flow` as a process with field `n` and operation
`go`, `Gate` as a state machine with field `open` and operation `toggle`,
`Address` as a record value type with field `street: Text[0, 64; nfc]`, `Account` as an object
with field `address: Address`, and `Store` as a persistence interface. Build
the corresponding model declaration nodes from each selected declaration's
identity and meaning. Admit them through the public reader, encode the
admitted package, read it again, and compare the structural node ids and complete package value.
Assert the `object` family of the process and state-machine nodes and the
`record` family of the record node. Tag the
public tests with their exact FR-068 criteria and QSpec FR-322-AC-63,
FR-322-AC-65 or FR-322-AC-72 criterion as applicable.
Resolve `Flow.n`, `Flow.go`, `Reference<Flow>`, `Reference<Gate>`,
`Account.address`, `Address.street` and an invariant clause owned by
`Address` through the public read. The test's identity oracle shall derive
the QSpec closed preimage independently of the reader's candidate map.

Rebuild valid identities for nodes whose owner points instead to an object
type, another selected form, or an absent declaration. Test a process owned
by an object type, a state machine owned by a process, a record owned by an
object type, and a persistence interface owned by a state machine. Test each
form with an unselected package and a missing selected declaration. Remove a
required owner without rebuilding the wire. Compare the exact refusal code,
cause, stage and pointer with QSpec's owner-join and wire rules. Test an
unknown record field; a record field typed at an event type, process, state
machine, persistence interface or namespace; and a record used as an object
reference target.

Finally construct protocol clauses whose `owner_model` targets valid selected
process, state-machine and record declaration nodes. Assert that the earlier
owner join admits each target and the later clause step refuses each edge as
`invalid_model_binding`/`malformed-declaration` at `owner_model`. An
unselected target must instead fail the earlier owner join. Keep all wire,
graph, identity and preceding protocol fields valid in these controls.

## Expected Results

The healthy selected declarations, process members and derived references
admit, and canonical write/read/read retains their exact node keys and
package name. Every adverse owner case refuses at its specified earlier stage
or owner join, without a partial package. The three selected wrong-kind
clause targets reach the later [FR-064-AC-4](../functional/FR-064-bind-protocol-clause-to-owner-model.md)
refusal; none is misreported as an unselected owner.
