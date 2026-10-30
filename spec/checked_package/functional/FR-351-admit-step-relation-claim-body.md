---
id: FR-351
title: "Admit the checked-package step-relation claim body"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-040
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-346
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-349
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-477
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
  - target: ix://agent-ix/quire-specification/FR-341
    type: references
  - target: ix://agent-ix/quire-specification/FR-402
    type: references
---
# FR-351: Admit the checked-package step-relation claim body

## Description

When the Contract IR V2 reader receives a `quire.checked-package/v2` package
containing a `claim` node with `semantic_form: "step_relation"`, it shall admit
the node only when its body and joins satisfy QSpec FR-477, and retain the
admitted claim as a closed `ClaimForm::StepRelation` with a typed body. The
reader shall return the first applicable typed refusal under QSpec FR-322's
package order and FR-477's node order, with no partially admitted package.
This requirement specifies IR's consumption and writing of the QSpec artifact;
QSpec FR-477 owns production, and QSpec FR-402 owns obligation identity.

## Inputs

Canonical checked-package V2 bytes, the existing FR-038 read limits and
evidence, including selected model documents needed for declaration joins.

## Outputs

An admitted `CheckedPackageV2` with a typed step-relation claim and its
existing node, occurrence and identity data; or one existing V2 refusal or
incomplete result.

## Behavior

### Form and body

The reader shall add `step_relation` to the closed `claim` form vocabulary and
shall retain its body as a distinct typed production, not as a caller-inspected
JSON object. The body shall be an aggregate of exactly four ordered bindings:
`subjects`, `executions`, `invariants`, `body`. `subjects` is an aggregate of
bindings from each distinct model alias, in first-named order, to a reference
to its model node. `executions` is an aggregate of bindings in source order,
one per execution variable. Each execution value is an aggregate of exactly
six ordered bindings: `alias` (text literal), `operation` (operation-anchor
reference), `parameters` (aggregate of value/parameter references),
`preconditions` and `postconditions` (aggregates of state-clause references),
and `implementation` (abstraction-relation reference or empty aggregate).
`invariants` is an aggregate of one binding per subject in subject order,
holding invariant-clause references. `body` references a Boolean expression.
The node's `semantic_type` shall name the Boolean scalar type; its
`declaration` shall be the declared relation name; its required source owner
shall be retained; and it shall have one declaration and one claim occurrence.

The reader shall require every alias to name a subject and every subject to be
named by an execution. It shall require each subject target to be a model node,
each operation target to be a `state`/`operation_anchor` whose context is a
declaration of that alias's model, and each referenced implementation to be a
`correspondence`/`abstraction_relation` with a `frames` entry keyed by that
operation. For every execution it shall require the `parameters` names and
types in FR-341 postcondition binder order: `self`, optional `result`, then
declared operation parameters. Levels shall start at zero and continue without
reset across executions; each execution shall reference its own nodes, even
when two executions name one operation. `preconditions` and `postconditions`
shall be exactly the matching state clauses anchored at that operation,
ascending by structural node id. Each subject's `invariants` shall be exactly the
invariant state clauses anchored at an object type, process or state machine
of its model, ascending by structural node id. The `body` target shall be an
expression node with Boolean `semantic_type` and its reads of each execution's
arguments, receiver states and result shall resolve to that execution's own
parameter nodes. The reader shall derive the code-claim request only when
every execution's `implementation` is a reference; otherwise only the model
claim is requested. It shall preserve the model and code items' distinct
FR-402 obligation identities over the one claim occurrence.

### Identity and order

The reader shall apply FR-322's graph, reference, dependency, owner, source-map and structural-id checks to this form. The node's dependencies shall be exactly the unique structural-id-ascending body reference targets. Its id is the retained `{package,path,ordinal}` label; the reader does not rederive it from owner, declaration or body. A changed invariant, precondition, execution order, binder or implementation changes the complete typed body and package value, while the declared path and package name may remain unchanged. Source whitespace and comments do not alter the typed body. The writer shall emit the same canonical body, dependencies, structural id and V2 bytes from the retained typed package, and its output shall read back with the same typed claim. The public reader shall also admit canonical bytes emitted by the external QSL producer under this QSpec contract.

The step-relation body step shall run after the terminal step and before the
protocol step, in FR-322 body-step order. It shall visit `claim`/
`step_relation` nodes by ascending structural node id. Within one node the reader
shall check shape, aliases, targets in body order, premises, then Boolean body,
reporting the first failure. Shape errors refuse `invalid_semantic_graph` at
the first binding breaking layout. An alias with no subject refuses
`missing_declaration`/`missing-name` at its alias binding; an unused subject
refuses `invalid_semantic_graph` at that subject binding. Wrong target kinds,
operation-to-model joins, implementation frame joins and premise sets or
binder lists refuse `invalid_model_binding`/`malformed-declaration` at the
offending target or aggregate. A non-Boolean body target refuses
`ill_typed`/`non-boolean-root` at that target. Earlier package stages retain
their precedence; a malformed or duplicate structural id or wrong dependency list is refused at its FR-322 stage before this body's semantic joins.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-351-AC-1 | Reading a canonical two-execution determinism relation over `V::Vault::step(i)` with an abstraction binding exposes one typed `claim`/`step_relation` node: four ordered root bindings; `V` once in subjects; `x` and `y` in source order, each with six ordered bindings, the same operation anchor, distinct `self` and `i` nodes at levels `[0, 1]` and `[2, 3]`, the operation's exact precondition/postcondition sets, the Vault invariant set, and a bound implementation; declaration `Det`, its source owner, one declaration and one claim occurrence are retained. Its body reading `pre(x.self.l) = pre(y.self.l)` reaches distinct self nodes. | Test |
| FR-351-AC-2 | Without an abstraction binding, both implementation members are empty aggregates and the reader exposes the model claim alone; with the binding, both model and code claims are requested from the one claim occurrence with different FR-402 obligation identities. | Test |
| FR-351-AC-3 | PLANNED/UNRUN (IR-1141). Adding a Vault invariant, changing step precondition or swapping x and y changes the complete step-relation body; whitespace and comment changes leave that body unchanged. Writing an admitted typed package and reading its canonical bytes preserves body, typed form and structural id; canonical output from the external QSL producer admits. | Test |
| FR-351-AC-4 | A missing execution `implementation` binding refuses `invalid_semantic_graph`; an unknown alias refuses `missing_declaration`/`missing-name`; an operation target naming an object type and an implementation relation lacking the operation's frame each refuse `invalid_model_binding`/`malformed-declaration`; omitted precondition or invariant references refuse the same at their aggregate; an integer-typed body expression refuses `ill_typed`/`non-boolean-root`, each at FR-477's locus and first applicable reader step. | Test |
| FR-351-AC-5 | Replacing `y`'s `self` and `i` parameter references with `x`'s nodes, while updating the exact dependencies, refuses `invalid_model_binding`/`malformed-declaration` at `y`'s `parameters`; a simultaneous earlier-stage defect wins before this semantic refusal. | Test |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns V2 intake, limits,
  identity, generic refusal fields and writer rules.
- [FR-040](./FR-040-admit-frame-entries-and-state-clauses.md) owns the anchor
  and state-clause semantics consumed by the premise joins.
- [FR-346](./FR-346-admit-abstraction-relation-body.md) owns the typed
  implementation relation and its frame entries.
- [FR-349](./FR-349-retain-typed-checked-v2-terms.md) owns typed body retention.
- QSpec FR-477 and TC-962 define the step-relation vectors; QSpec FR-322,
  FR-341 and FR-402 define graph identity, binders and obligation identities.
