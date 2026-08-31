---
id: FR-013
title: "Represent the closed v0.1 contract type system"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
---
# FR-013: Represent the closed v0.1 contract type system

## Description

The v0.1 model shall define Boolean, signed and unsigned bounded integer,
rational, text, enum, record, option, bounded collection, input, state, and pure
function declarations without embedding implementation or
architecture-language vocabulary.

## Inputs

Named type declarations, field declarations, numeric bounds, collection bounds,
enum variants, and value-reference declarations.

## Outputs

A validated declaration environment with deterministic name lookup and typed
value-reference identities.

## Behavior

The closed value-type variants are Boolean, integer, rational, text, named enum,
named record, option, and bounded collection. An integer declares signed or
unsigned representation, an inclusive minimum and maximum, and either `reject`
or `saturate` overflow behavior. Its bounds must be ordered and unsigned minima
must be non-negative. Every integer bound and rational numerator bound lies in
the closed range `-9223372036854775808` through `9223372036854775807`; wider
bounds use `invalid_numeric_bounds`. A rational declares inclusive numerator bounds and a
positive maximum normalized denominator; rational arithmetic always uses
`reject` overflow behavior. Its maximum denominator lies in `1` through
`9223372036854775807`. A collection declares a positive finite maximum
item count. Options and collections recursively contain value types.

Text is a sequence of Unicode scalar values with no normalization or locale
folding. A text value contains at most `1048576` scalar values; an over-length
literal uses `text_bound_exceeded`.

Every declaration, enum variant, record field, function parameter, input, and
state carries a source span. Public validation limits are
`maximum_expression_nodes = 10000` and `maximum_expression_depth = 256`; these
are wire-independent semantic constants, not host pointer-sized values.

Enum and record declarations share one type-name namespace. Enums contain at
least one unique variant. Records contain unique fields. Named types must
resolve, and the directed record-containment graph through record, option, and
collection fields must be acyclic. Input and state value declarations share one
value-name namespace and bind one value type. Pure-function declarations have
unique names, unique parameter names, ordered parameter types, and a result
type. Every public name uses the issue #6 identifier grammar.

Validation is deterministic and fail closed. Declaration grammar and bounds
precede duplicate, orphan, and recursive checks. A valid environment round
trips structurally through public constructors and accessors with equality;
FR-016 owns canonical bytes and FR-018 later owns the published wire schema.

A rational is normalized exactly when its denominator is positive and
`gcd(abs(numerator), denominator) = 1`; zero is represented as `0/1`. Every
rational literal and arithmetic result is normalized before numerator and
denominator bounds are checked.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-013-AC-1 | Every closed value-type and declaration construct has positive and negative fixtures with deterministic STD-001 diagnostics. | Test (TC-016) |
| FR-013-AC-2 | Public serialized types contain no Rust, GUMBO, AADL, HAMR, solver, or runtime-specific vocabulary. | Inspection (TC-016) |
| FR-013-AC-3 | Duplicate names/fields/variants, empty enums, invalid integer or collection bounds, absent named types, and direct or indirect record cycles fail before expression validation. | Test (TC-016) |
| FR-013-AC-4 | A valid declaration environment round trips structurally through public constructors/accessors without losing declaration identity, types, bounds, overflow policy, or provenance spans. | Test (TC-016) |

## Dependencies

FR-011 supplies package-scoped declaration identity.
