---
id: FR-059
title: "Admit checked-package state-graph claim nodes"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-475
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
---

# FR-059: Admit checked-package state-graph claim nodes

## Description

When a V2 checked package carries a `claim`/`state_graph` node, the
Contract IR reader SHALL admit exactly QSpec FR-475's body and joins, or refuse the whole package
at the first defect in FR-475's reader order. The canonical writer SHALL retain
the admitted node's complete body and occurrences. The reader uses the
published QSpec contract and does not infer a claim from source text.

## Inputs

The untrusted `quire.checked-package/v2` bytes, evidence and limits of
[FR-038](./FR-038-consume-checked-package-v2.md). The form and body are
specified by `ix://agent-ix/quire-specification/FR-475` and the identity,
graph and refusal ordering by `ix://agent-ix/quire-specification/FR-322`.

## Outputs

An admitted package retaining the state-graph node, its exact semantic body,
dependencies, Boolean type and claim occurrences; or one typed refusal with
code, cause when defined and RFC 6901 pointer. No partial package.

## Behavior

The reader SHALL add `state_graph` to the closed `claim` form vocabulary while
continuing to refuse every unknown `claim` form as `invalid_semantic_graph` at
`/semantic_graph/nodes/{n}/semantic_form`. A state-graph body is an aggregate
of four bindings in the fixed order `kind`, `over`, `from`, `target`. `kind` is
a text literal of `possible`, `always-possible` or `unique-path`. `over` is a
reference to a `value`/`parameter` node or the empty aggregate. `from` is the
empty aggregate for `possible` and a reference to a Boolean predicate node
otherwise. `target` is a reference to a Boolean predicate node. Predicate
targets are only `expression` or `value`/`literal` nodes with Boolean
`semantic_type`; a Boolean `temporal`, `state`, `claim` or `function` node is
not a predicate. The `from` reference of an `always-possible` claim written
without `from` is the constant-true literal node, as QSpec FR-475 specifies.

The node SHALL have Boolean `semantic_type`, no `declaration` or `owner`,
exactly the unique structural-id-ascending set of body reference targets as
`dependencies`, and one `claim` occurrence per checked source declaration.
Equal checked content may share a node and is distinguished by occurrence
ordinal. The form's node id is structural and the package identity is its declared name; the reader SHALL not substitute an obligation identity,
which includes the request subject and an `over` instance and belongs to the
request layer.

The reader SHALL apply FR-322's earlier strict-wire, graph-shape and key
checks before the state-graph body step. The state-graph step SHALL precede
the temporal, abstraction, terminal and later body steps. It SHALL visit
state-graph nodes by ascending node-id digest and check one node in this
order: exact body shape; `kind`; form fit of `over`, `from`, `target`; and
target meanings in body order. Shape defects refuse `invalid_semantic_graph`
at the body; an invalid `kind` refuses `invalid_package`/`invalid-value` at
`body/members/0`; form-fit defects refuse `invalid_semantic_graph` at the
offending binding; a non-parameter `over` target refuses
`invalid_model_binding`/`malformed-declaration` at the target; and a
non-predicate or non-Boolean `from` or `target` refuses
`ill_typed`/`operator-ineligible` at the target. A dependency mismatch
refuses at `/dependencies` in the earlier graph-shape step.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-059-AC-1 | The reader admits QSpec TC-960 SN-01 through SN-05 and SN-15 as `claim`/`state_graph` nodes with the exact four-binding body, form-specific `kind` and `from`, Boolean type, absent `declaration` and `owner`, exact dependency set and `claim` occurrence ordinals; a second declaration of equal content shares one node. The writer emits that complete admitted shape, and a read of its output returns the same nodes and occurrences. | Test |
| FR-059-AC-2 | QSpec TC-960 SN-08 through SN-14 and SN-17 refuse with FR-475's exact code, cause and pointer after identities are rebuilt: binding reordering, unknown kind, wrong form fit, non-predicate target and missing dependency are separately observed. An unknown `claim` form still refuses at `semantic_form`, and no malformed node is omitted or lowered as another form. | Test |
| FR-059-AC-3 | The reader observes FR-322's precedence: graph-shape dependency failure precedes the state-graph step; within that step the first failed check wins and nodes visit in structural node-id order; a state-graph defect precedes a temporal, abstraction, terminal or later body-step defect. | Test |
| FR-059-AC-4 | QSpec TC-960 SN-06, SN-07 and SN-16 retain the node-id distinctions fixed by QSpec: kind changes the node id, source respelling does not, and a subject or `over` instance change does not alter the node. The IR reader and writer preserve the checked node bytes used by the request layer to bind distinct obligation identities; IR does not compute those request identities. | Test |
| FR-059-AC-5 | IR-owned positive and negative read/write vectors for QSpec TC-960 SN-01 through SN-17 assert the published wire shapes and refusals through the public V2 reader and writer, without copying QSpec fixture or schema files into IR. | Test |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns the V2 reader, writer,
  identity and shared refusal machinery.
- `ix://agent-ix/quire-specification/FR-475` owns the exact state-graph body,
  reader step and target rules; `ix://agent-ix/quire-specification/FR-322`
  owns the enclosing package and earlier checks.

## Status

Planned. The current reader rejects `claim`/`state_graph` at its closed form
gate; no reader or writer acceptance is claimed by this spec.
