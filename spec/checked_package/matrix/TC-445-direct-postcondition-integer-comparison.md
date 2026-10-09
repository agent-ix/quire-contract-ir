---
id: TC-445
title: "Typed direct postcondition integer comparison operands"
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
---
# TC-445: Typed direct postcondition integer comparison operands

## Description

Verify FR-038-AC-202 through FR-038-AC-209 through the public
`CheckedPackageV2` accessor over reader-admitted packages and bounded
crate-local post-admission mutations. The QSL `ConfigVersion` clause
`post VersionUnchanged { self.versionNumber = pre(self.versionNumber) }`
is the producer-shaped positive case; the test constructs its own V2 package
or reads fresh output from the authoritative QSL producer. It does not copy a
QSL fixture or schema into this repository.

## Test Procedure

Read a package with a postcondition `state_clause` whose third argument
references an integer comparison node. Supply that clause's id and an actual
`claim` occurrence. Vary the comparison identity through exactly integer
eq/ne/lt/le/gt/ge. Give the two operands direct `self` field projects and
admitted `IntRange` member types in the selected model document. Reverse the
two comparison argument references in a separately admitted package. Call the
accessor for each and inspect typed results only.

For the `ConfigVersion` case, build the exact QSL FR-093/FR-105 topology:
the left operand references `record.project(versionNumber)` over
`model.deref(self)`; the right references `state.pre` over that same project
node. Give the comparison and project the multiple `expression` occurrences
the producer can emit, and give the clause two `claim` occurrences. Call once
per claim occurrence. Compare the returned project ids and snapshot labels
independently of source-map regions.

Change the selected document's admitted field range between independent
packages, including endpoint cases at `i128::MIN` and `i128::MAX`. Give a
different direct field an unbounded `Integer` type and assert the typed
refusal. In a crate-local post-admission mutation, remove or make unavailable
the retained field type and assert defensive `MissingRange`; an out-of-`i128`
bound is a possible source of an unavailable retained type, but the reader's
model-member join refuses its direct field read before this accessor. Mutate
the project node body's unverified range after admission without changing
the selected document's field declaration, and assert that the accessor uses
the document-derived range or refuses rather than returning the mutated
value.

In separate admitted packages, place a `let` alias, another parameter,
nested project, extra `pre` or other wrapper, and an inline term in the
comparison operand position. Assert `IneligibleOperand` and its structural
path. Exercise an unknown clause id, wrong node form, absent claim occurrence,
precondition or invariant clause, non-reference condition root, catalogued
operator outside the six, and an unknown operation identity, invalid
comparison arity and dangling child through crate-local
post-admission mutation. Combine defects in two operands and assert operand
0 wins; combine an earlier clause or comparison defect with a later operand
defect and assert the stated precedence. A separate post-admission mutation
may check defensive malformed-body behavior; it is not reader admission
evidence.

Compile an external API consumer that matches every public comparison,
snapshot and refusal variant and reads each result field without
`graph().nodes[*].body` or `serde_json::Value`. Call repeatedly and on a
cloned package; compare values and package equality. Inspect production
source for a contextual walk from clause condition and for absence of an
added QSpec per-use metadata field or a codegen JSON decoder.

## Expected Results

Each positive call returns its actual clause and comparison identities,
selected claim occurrence, operator, two authored-order entries and exact
inclusive `i128` ranges. The shared project receives `Post` on the direct
path and `Pre` through `state.pre`, without a node-global snapshot. Claim
occurrences select no unique child expression occurrence or region. Every
negative call returns the specified distinct typed refusal, structural path
and reached child id when available, without a partial result or panic.
Unbounded and unavailable ranges remain refusals. The external consumer
uses only typed public values; repeated calls preserve package equality.
