---
id: FR-014
title: "Type and validate executable contract expressions"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-013
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-012
    type: depends_on
---
# FR-014: Type and validate executable contract expressions

## Description

The model shall represent literals, value/local references, state observations,
field and option access, collection length and indexing, pure calls, arithmetic,
comparisons, implication, bounded quantification, and distinct short-circuit
and total Boolean operators.

## Inputs

An expression tree, declaration environment, expected result type, and source
span for every node.

## Outputs

A typed expression tree or an ordered set of source-located diagnostics.

## Behavior

Every node carries a source span. The closed nodes are Boolean, bounded integer,
bounded rational numerator/positive denominator, text, enum-variant,
option-none, option-some, record, and collection literals; value and local
references; record-field access; option is-present and unwrap; collection
length and index; pure-function call; numeric add, subtract, multiply, divide,
remainder, and negate; equal, not-equal, less, less-equal, greater, and
greater-equal comparisons; Boolean negate; short-circuit and total
conjunction/disjunction; implication; and finite universal/existential
quantification.

A value reference names an input or state declaration and an observation:
`current`, `pre`, or `post`. Inputs allow only `current`. State observation is
closed by the owning FR-012 execution point:

| Execution point | Permitted state observations |
|---|---|
| initialization | `current`, `post` |
| handler | `current`, `pre`, `post` |
| pre | `current`, `pre` |
| post | `current`, `pre`, `post` |

A local reference resolves only inside its active quantifier scope.

Every integer, rational, text, enum, record, and collection literal explicitly
names its value type; option-none names its option type and option-some names
the same option type plus a value. Integer/rational values must fit their named
bounds after rational normalization. Enum variants must exist. Record literals
must provide every declared field exactly once and no unknown field. Collection
literal items must have the element type and their count must not exceed the
declared maximum. Violations use the issue #8 precedence and registered codes.
Out-of-bound numeric literals use `invalid_numeric_bounds`; an absent enum
variant or missing/unknown record field uses `ill_typed_expression`; a repeated
record-literal field uses `duplicate_field`; and excessive collection items use
`collection_bound_exceeded`.

Field access requires a record, option access requires an option, length and
index require a bounded collection, and calls require an existing pure function
with exact ordered argument types and arity. Numeric operands are compatible
only when their complete types are equal: signedness, bounds, and overflow
policy for integers; normalized numerator bounds and denominator maximum for
rationals. Every numeric operator returns that same operand type. Remainder is
integer-only; numeric negate accepts signed integers and rationals. Ordering
requires equal integer, rational, or text types; equality
requires equal operand types; Boolean operators, implication, and quantifier
predicates require Boolean operands/results. Division of integers returns the
same integer type; rational division returns the same bounded rational type.
Collection length and index operands use a derived unsigned integer type with
minimum zero, maximum equal to the collection maximum item count, and `reject`
overflow. An index expression must have that exact type.

A quantifier domain is explicitly `elements(collection)` or
`indices(collection)`. An element-domain local has the element type. An
index-domain local has the derived index type and contributes an automatic
in-bounds fact for indexing that exact collection. The uniquely named local
exists only within the predicate.

A quantifier local may shadow an input or state declaration and local lookup
wins. A local name shall not repeat any enclosing quantifier local;
`invalid_scope` is emitted at the inner binder span. Locals never escape their
predicate.

Short-circuit conjunction/disjunction and implication retain conditional
right-hand evaluation as distinct variants from total conjunction/disjunction.
The checker returns an expression whose type is explicit at every node.
Structural equality and deterministic declaration lookup make repeated checks
of identical inputs equal without claiming FR-016 canonical bytes.

Before recursive typing, an explicit-stack preflight counts nodes and depth.
More than 10000 nodes or depth greater than 256 emits `expression_too_large` at
the first node crossing the limit; no recursive walk begins on rejected input.

The typed expression implements the FR-012 dependency-source contract. A state
dependency contains its exact requirement/declaration identity and
`current`/`pre`/`post` observation; an input dependency uses `current`; a field
dependency path is `[owning-record-type, field]`; an enum dependency path is
`[enum-type, variant]`; and a pure-function dependency path is `[function]`.
Dependencies deduplicate and sort by the structural tuple `(requirement,
dependency kind, observation, path)`, matching the issue #6 ordered-set rule.
Kind order is input, state, field, enum variant, then pure function. Observation
order is absent, current, pre, then post. Each path segment compares by Unicode
scalar value; a shorter equal prefix sorts first.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-014-AC-1 | Ill-typed operands, result mismatches, invalid scopes, and non-Boolean clause roots fail at the narrowest source span. | Test (TC-016) |
| FR-014-AC-2 | Canonical representation distinguishes short-circuit conjunction/disjunction from total conjunction/disjunction. | Test (TC-016) |
| FR-014-AC-3 | Current/pre/post observation policy, pure-call arity/signatures, record/enum names, and quantifier-local scope are validated with distinct registered diagnostics. | Test (TC-016) |
| FR-014-AC-4 | Rechecking identical declarations, context, expression, and expected type produces structurally equal typed output, ordered diagnostics, and dependency identities. | Test (TC-016) |
| FR-014-AC-5 | Expression trees at the node/depth limits validate normally; the first node beyond either limit fails with `expression_too_large` before recursive typing. | Test (TC-016) |
| FR-014-AC-6 | Typed expression fixtures derive exact input/state-observation, field-owner, enum-variant, and pure-function dependencies once in structural order, satisfying FR-012-AC-5. | Test (TC-016) |

## Dependencies

FR-013 defines the closed type system.
