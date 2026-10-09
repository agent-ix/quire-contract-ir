---
id: TC-445
title: "Typed clause-context integer comparison operands"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
  - target: ix://agent-ix/quire-specification/FR-341
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-093
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-105
    type: references
  - target: ix://agent-ix/quire-contract-codegen/FR-008
    type: references
---
# TC-445: Typed clause-context integer comparison operands

## Description

Verify FR-038-AC-202 through FR-038-AC-209 through the public
`CheckedPackageV2` accessor. The named `ConfigVersion` producer assertion
requires a fresh QSL-produced V2 package from the authoritative QSL checkout
and its actual selected model evidence, then an admitted read by the
production IR reader. Locally constructed V2 cases are **synthetic**
admission and defensive-API tests; they do not substitute for the named QSL
producer assertion. No QSL schema, binary or fixture is copied here.

## Test Procedure

At test time, invoke the authoritative QSL producer on its own
`examples/config-version` source with `post VersionUnchanged {
self.versionNumber = pre(self.versionNumber) }`, or its equivalent current
QSL source fixture. Supply the producer's selected model document to
`CheckedPackageV2::read`, require `Admitted`, find the actual postcondition
clause id and an authentic claim occurrence, and call the typed accessor.
Assert the returned comparison is integer.eq, both operand project ids are
identical, their authored ordinals are 0 and 1, their observations are Post
and Pre, their typed provenance is StateField, and their inclusive bounds
are 0..=1000. Record the producing QSL source revision and command in the
test result rather than checking in produced bytes. If the producer or its
model evidence is unavailable, this producer-origin case is unrun, never
credited from a synthetic fixture.

Build separate **synthetic, reader-admitted** packages for every catalogued
integer comparison operation. Exercise invariant, precondition and
postcondition clauses, each with an authentic claim occurrence. Use direct
`self.field` StateField reads in all three and direct operation-parameter
OperationInput reads in preconditions/postconditions. Check Current on a
direct state field in invariant/precondition and on direct operation inputs;
check Post on a direct state field in a postcondition and Pre through one
`state.pre` wrapper in a postcondition. Construct two different reads and
reverse their argument references in a separately admitted package. Check
that the field provenance carries the model declaration id and field name,
and input provenance carries the checked operation-parameter node and its
ordinal from FR-341's parameter aggregate. Give a parameter the text name
`self` outside the self slot and verify that its name cannot turn it into a
StateField. Use the returned provenance to check the CG FR-008 boundary
mapping StateField to State and OperationInput to Input; no V1 kind is read
from V2 wire or inferred from a name.

For each operand side, exercise both a graph `value`/`literal` reference and
an inline integer literal with one read on the other side. Check the literal
singleton and typed identity, including two inline positions of the same
value. Two literals return NoRead. A Boolean or text literal returns
IneligibleOperand. Change admitted model field and checked input-parameter
ranges between independent packages, including endpoint cases at
`i128::MIN` and `i128::MAX`. Assert exact inclusive bounds, an unbounded
integer refusal and an out-of-`i128` literal refusal. In a
crate-local post-admission mutation, make a retained field type unavailable
and assert defensive MissingRange. An out-of-`i128` model field has no
direct reader-admitted field read, so it is not a positive accessor case.
Mutate an unverified project body after admission without changing the
selected model declaration, and verify that no returned range is taken from
that body.

In separate synthetic packages, put a `let` alias, result slot, unrelated
parameter, nested project, extra `pre` wrapper, `pre` around an operation
input and `pre` in an invariant or precondition in an operand position.
Assert IneligibleOperand at the structural path. Exercise an unknown clause
id, wrong node form, absent claim occurrence, unsupported clause kind,
non-reference condition root, unknown and catalogued ineligible operator,
invalid comparison arity and a dangling child through crate-local
post-admission mutation. Combine earlier and later defects to assert the
first-refusal order; combine operand defects to assert operand 0 wins.
Defensive post-admission mutations are not evidence of reader admission.

Give one clause two claim occurrences and its comparison and project several
expression occurrences. Assert that switching the claim changes the returned
claim but never assigns a unique child expression occurrence or region.
Compile an external Rust consumer that exhaustively matches the public
operator, read provenance, observation, operand identity and refusal types,
and reads all result fields without `graph().nodes[*].body` or
`serde_json::Value`. Repeat calls and clone the package; compare values and
package equality. Inspect production source for contextual traversal from
the clause condition and absence of a new V2 per-use wire member.

## Expected Results

The fresh QSL result proves the named shared-node Post/Pre topology from an
authoritative producer. Synthetic cases establish the remaining closed
operators, clause kinds, Current/Post/Pre observations, read/literal
combinations, typed Input/State provenance mapping and exact finite bounds.
Every refusal returns its distinct typed variant, supplied clause/claim,
structural path and reached child id when available, without a partial
result, guessed source region, narrowed endpoint or panic. The external
consumer uses typed public values, and repeated calls preserve package
equality and reader admission.
