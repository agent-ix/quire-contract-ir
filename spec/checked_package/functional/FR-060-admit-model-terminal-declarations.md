---
id: FR-060
title: "Admit checked-package model terminal declarations"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-476
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
---

# FR-060: Admit checked-package model terminal declarations

## Description

When a V2 checked package carries a `temporal`/`terminal` node, the Contract
IR reader SHALL admit exactly QSpec FR-476's declaration body and joins, or refuse the
whole package at the first defect in FR-476's reader order. The canonical
writer SHALL retain the admitted node's complete body and occurrences. The
reader obtains the model's terminal declaration from this node alone.

## Inputs

The untrusted `quire.checked-package/v2` bytes, evidence and limits of
[FR-038](./FR-038-consume-checked-package-v2.md). The form and body are
specified by `ix://agent-ix/quire-specification/FR-476` and the identity,
graph and refusal ordering by `ix://agent-ix/quire-specification/FR-322`.

## Outputs

An admitted package retaining the terminal node, its exact semantic body,
dependencies, Boolean type and claim occurrences; or one typed refusal with
code, cause when defined and RFC 6901 pointer. No partial package.

## Behavior

The reader SHALL add `terminal` to the closed `temporal` form vocabulary
while continuing to refuse every unknown `temporal` form as
`invalid_semantic_graph` at `/semantic_graph/nodes/{n}/semantic_form`. A
terminal body is an aggregate of four bindings in fixed order: `model`,
`mode`, `parameters`, `condition`. `model` references the selected model
node. `mode` is a text literal `when` or `any`. `parameters` is empty or holds
one reference to the level-0 `self` `value`/`parameter` node, typed
`Reference<M::T>` for the referenced model's FR-366 object type. It is empty
for `any`. `condition` is a reference to a Boolean `expression` for `when`
and the empty aggregate for `any`.

The node SHALL have Boolean `semantic_type`, no `declaration`, one `claim`
occurrence per source declaration, and exactly the unique structural-id-ascending
set of body reference targets as `dependencies`. The written terminal name
exists only in its source occurrence; renaming it changes neither the structural node id nor package name. A valid change to model, mode or condition changes the complete body; the reader validates its dependencies and meaning without re-deriving an id. The node, not a separate request field, is the
carrier used by the deadlock-freedom subject.

The reader SHALL run FR-322's earlier strict-wire, graph-shape and structural-id checks
before the terminal body step. The terminal step SHALL follow the state-graph,
temporal and abstraction steps and precede the step-relation, hybrid,
timed-model, protocol and schedulability steps. It SHALL visit terminal
nodes by ascending structural node id. For each node it SHALL check exact body
shape; mode; mode fit of `parameters` then `condition`; targets in body order;
and, after every node's own checks, uniqueness per model. Shape and mode-fit
defects refuse `invalid_semantic_graph` at the body or offending binding.
Unknown mode refuses `invalid_package`/`invalid-value` at `body/members/1`.
A non-model `model` target refuses `invalid_model_binding`/
`malformed-declaration`; a wrong `self` target or non-expression condition
refuses `ill_typed`/`operator-ineligible`; a non-Boolean expression condition
refuses `ill_typed`/`non-boolean-root`, each at the target. A second terminal
node for one model refuses `ambiguous_declaration`/`ambiguous-name` at the
second node in structural-id order, naming both; two `claim` occurrences on one
terminal node refuse the same at that node, naming the first two
occurrences. A dependency mismatch refuses at `/dependencies` in the
earlier graph-shape step.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-060-AC-1 | The reader admits QSpec TC-961 MT-01 through MT-03 as `temporal`/`terminal` nodes with the exact four-binding body, `when` and `any` modes, level-0 `self` parameter only when read, Boolean type, absent `declaration`, exact dependency set and `claim` occurrence. The writer emits the complete admitted shape, and a read of its output returns the same nodes and occurrences. | Test |
| FR-060-AC-2 | QSpec TC-961 MT-07 through MT-16 refuse with FR-476's exact code, cause and pointer after identities are rebuilt: swapped bindings, invalid mode, wrong mode fit, wrong model, wrong or mistyped `self`, non-Boolean condition, and both kinds of duplicate declaration are separately observed. An unknown `temporal` form still refuses at `semantic_form`, and no malformed terminal node is omitted or reinterpreted. | Test |
| FR-060-AC-3 | The reader observes FR-322 and FR-476 precedence: graph-shape defects precede the terminal step; state-graph, temporal and abstraction body defects precede it; each terminal node's own checks precede one-per-model checks; nodes visit in structural node-id order; and a terminal defect precedes step-relation and later body defects. | Test |
| FR-060-AC-4 | Withdrawn by the IR-1141 structural-identity decision under QSpec #517: the reader does not recompute identity from body content or refuse a retained structural id solely because a valid body changed. This id is retained and never reused. | Withdrawn |
| FR-060-AC-5 | IR-owned positive and negative read/write vectors for QSpec TC-961 MT-01 through MT-16 assert the published wire shapes and refusals through the public V2 reader and writer, without copying QSpec fixture or schema files into IR. | Test |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns the V2 reader, writer,
  identity and shared refusal machinery.
- `ix://agent-ix/quire-specification/FR-476` owns the exact terminal body,
  reader step and target rules; `ix://agent-ix/quire-specification/FR-322`
  owns the enclosing package and earlier checks.

## Status

Planned. The current reader rejects `temporal`/`terminal` at its closed form
gate; no reader or writer acceptance is claimed by this spec.
