---
id: FR-067
title: "Admit Boolean state-transition application bodies"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-040
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-051
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-341
    type: references
  - target: ix://agent-ix/quire-specification/FR-342
    type: references
  - target: ix://agent-ix/quire-specification/FR-464
    type: references
---

# FR-067: Admit Boolean state-transition application bodies

## Description

When a checked-package V2 node declares `state`/`transition`, the Contract IR
reader SHALL admit exactly the Boolean model-field transition body and joins of
QSpec FR-341, or refuse the whole package at the first applicable check in its
published order. The public typed writer SHALL retain the admitted transition
through a read/write/read cycle. This requirement adds no source-language
transition semantics; QSL-1163 owns emission from an authored QSL claim.

## Inputs

The untrusted checked-package V2 bytes, selected model packages, checked lock,
evidence and bounded reader limits of [FR-038](./FR-038-consume-checked-package-v2.md).
The published QSpec state-transition package and mutation definitions are the
external wire and identity oracles.

## Outputs

An admitted package with a typed `state`/`transition` node, its complete
application body and claim occurrences; or one typed refusal with its defined
code, cause and RFC 6901 locus. No partial package is returned.

## Behavior

The strict reader SHALL recognize `transition` only as a `state` node form. Its
body root SHALL be one `quire.op.state.transition` application with operator
class `state_transition`, exactly one `protocol_profile` law equal to the
selected lock row, member `{kind: "profile_operator", operator:
"transition"}`, `null` mode, empty leaves, Boolean `result_type` equal to
the node's `semantic_type`, and five `binding` arguments in this exact order:
`anchor` (reference), `subject` (reference), `inputs` (ordered aggregate of
references), `pre` (empty aggregate or one-reference aggregate), and `post`
(reference). A package with a transition SHALL select `protocol_profile`.
The reader SHALL retain QSpec's closed-wire/schema and operation-catalog
refusals for unknown, omitted, repeated, extra, out-of-order or wrongly shaped members; the
reader SHALL NOT accept the body as a generic term or another node form.

The transition's `anchor` SHALL name the no-point `state`/`operation_anchor`
of the same operation and context as the transition. The `subject` SHALL be
an `expression`/`query` projection (`quire.op.record.project`) of a
`quire.op.model.deref` of that anchor's `self` parameter of type
`Reference<C>`. Its field declaration and name SHALL resolve among the
context's exposed effective members to a required, single-valued Boolean
field, and the anchor's frame SHALL `modifies` that exact field. An optional
Boolean, collection-valued Boolean or non-Boolean field SHALL NOT qualify.
The `inputs` list SHALL equal the operation's Boolean parameters in declared
order, including the empty list for a parameterless operation. The optional
`pre` and required `post` SHALL name the operation's precondition and
postcondition `state_clause` nodes. Each clause's anchor SHALL have the same
context and operation; its accepted execution point MAY differ from the
transition's no-point anchor, including one-sided `pre`/`post` presence.
The precondition's subject read denotes the pre-state field value and the
postcondition's subject read denotes the post-state field value. The
transition SHALL state no third state.

The reader SHALL retain exactly one `claim` occurrence per checked source
declaration of the transition, including each occurrence ordinal and source
region. Equal checked transition content MAY share one node while retaining
distinct occurrences. The node SHALL have no declaration, and its exact
dependencies SHALL be the unique digest-ascending set of body-reference
targets: the anchor, subject, every input, present `pre` and `post`. The
application-node preimage SHALL include the operator, identity, selected
profile-law definition, member operator, five bindings and Boolean result
type; the package identity projection SHALL include the complete body and
omit occurrences. Re-keying an operation, subject, input, clause or profile
selection SHALL change the affected transition key and package identity;
changing only source spelling, region or occurrence count SHALL NOT.

After strict wire, graph-shape, declaration, stale-key and frame checks,
the state step SHALL check placement of transition applications before anchor
checks, then every anchor, every state clause and every transition in their
respective ascending node-id digest order. For each transition, the reader SHALL
check the `anchor`, `subject`, `inputs`, `pre` and `post`, then uniqueness of
`(anchor, subject)`, in that order. The reader SHALL refuse missing declared
targets as `missing_declaration`/`missing-name` at the target; wrong
anchor, subject, input or clause forms or joins SHALL refuse
`invalid_model_binding`/`malformed-declaration` at the target; a subject or
input type mismatch SHALL refuse `ill_typed`/`operator-ineligible` at the
transition node; and the second `(anchor, subject)` pair SHALL refuse
`ambiguous_declaration`/`ambiguous-name` at the later transition node.
A transition application at the root of another node SHALL refuse
`ill_typed`/`operator-ineligible` there before anchor checks; a nested
transition application SHALL refuse `malformed_wire` at that application
in strict wire validation. The operation step SHALL follow the state step
and enforce the profile law, member operator, result type and term families.
The reader SHALL retain QSpec's exact code, cause and locus when a more
specific catalog or schema rule applies.

The public typed writer SHALL emit the complete admitted application,
dependencies, selected profile, source occurrences and exact package/node
identities. Reading its output SHALL recover the same typed transition,
including Boolean subject, input order, optional `pre`, required `post`,
point-bearing clause anchors and no-point transition anchor. It SHALL NOT
infer a source claim, add a point, erase a point or substitute a sidecar value.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-067-AC-1 | The reader admits the published QSpec TC-233 `XT-01` Boolean `Switch.set(flag: Boolean)` transition under an authentic selected `acme/orders` model, retaining its no-point anchor, required Boolean `on` subject, ordered `flag` input, `flag || on` precondition, `on` postcondition, one selected `protocol_profile` law and Boolean type. A package with a point-bearing pre/post clause anchor and a distinct no-point transition anchor also admits. | Test |
| FR-067-AC-2 | QSpec TC-233 `XT-02` through `XT-07`, `XT-16` and `XT-18` produce their published schema, `malformed_wire`, `operation-member-mismatch`, placement or unselected-law outcomes; no malformed transition is accepted as another form or a generic term. A nested transition application refuses at the nested application, and a root application on another node refuses at that node before anchor checks. | Test |
| FR-067-AC-3 | QSpec TC-233 `XT-10` through `XT-15` independently refuse wrong anchor, unmodified or misbound subject, Integer or optional Boolean subject, wrong or misordered inputs, wrong clause, and duplicate `(anchor, subject)` with FR-341's first code, cause and locus. A valid required Boolean subject and declared-order Boolean inputs admit. | Test |
| FR-067-AC-4 | With two independent defects, strict wire, graph/dependency, selected-declaration and stale-key checks outrank frame; frame outranks state; within state, transition placement outranks anchor, anchor outranks clause, and clause outranks transition. For two transition defects, ascending node-id digest order wins; within one transition, anchor, subject, inputs, clauses and uniqueness decide the first refusal. The operation-catalog law/member/type checks follow state. | Test |
| FR-067-AC-5 | QSpec TC-233 `XT-08` and `XT-09` retain the published identity distinction: changing the anchor's operation rekeys anchor, clauses, transition and package; moving only a source region changes none of their identities. Independently changing the field, an input, `pre`, `post` or selected profile rekeys the affected transition and package. Retaining an old node key after a body change refuses `invalid_package`/`stale-node-key` before state checks. | Test |
| FR-067-AC-6 | The public typed reader returns each `claim` occurrence and its original source region/ordinal; equal-content source claims share a transition node while preserving separate occurrences. A non-`claim` occurrence fails the published closed form. A public typed write/read/read of the admitted transition preserves the five bindings, exact dependency set, point/no-point anchors, occurrences, node keys and package identity without parsing an untyped JSON body. | Test |
| FR-067-AC-7 | IR-owned tests use the authoritative QSpec TC-233 transition package and mutation definitions directly from the selected QSpec checkout, and a QSL-1163-authored source/claim when available; they copy no QSpec or QSL schema, fixture or binary into IR. The QSL-authored positive case exposes pre-state Boolean inputs, post-state Boolean result and actual transition subject through the public IR carrier. | Test |

## Dependencies

[FR-038](./FR-038-consume-checked-package-v2.md) owns closed admission,
identity, source occurrences and the public reader; [FR-040](./FR-040-admit-frame-entries-and-state-clauses.md)
owns existing frame, anchor, clause and execution-point validation;
[FR-051](./FR-051-encode-checked-package-v2-envelope.md) owns canonical
typed writing. QSpec `ix://agent-ix/quire-specification/FR-341` fixes the
transition body and reader order, FR-342 the anchor and FR-464 the profile
operator. QSL-1163 owns source emission and is prerequisite to the
QSL-authored portion of FR-067-AC-7.

## Status

Planned. The state-transition vocabulary alone does not satisfy this reader,
writer or test contract.
