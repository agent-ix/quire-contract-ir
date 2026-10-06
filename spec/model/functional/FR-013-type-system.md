---
id: FR-013
title: "Represent the closed contract type system"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
---
# FR-013: Represent the closed contract type system

## Description

The model shall define Boolean, signed and unsigned bounded integer,
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
the closed range `i128::MIN` (`-170141183460469231731687303715884105728`)
through `i128::MAX` (`170141183460469231731687303715884105727`); wider
bounds use `invalid_numeric_bounds`. A rational declares inclusive numerator
bounds and a positive maximum normalized denominator; rational arithmetic
always uses `reject` overflow behavior. Its maximum denominator lies in `1`
through `170141183460469231731687303715884105727`. A collection declares a
maximum item count in the closed unsigned 32-bit range `1` through
`4294967295`; zero uses
`unbounded_collection`, and a wider wire integer uses
`invalid_numeric_bounds`. Options and collections recursively contain value
types.

On the wire, and in every serialized form of a validated value, an integer's
`minimum` and `maximum`, a rational's `numerator_minimum`, `numerator_maximum`
and `maximum_denominator`, an integer literal's `value` and a rational literal's
`numerator` and `denominator` are JSON strings of minimal base-ten digits: the
QSpec V2 `IntegerString` `^(0|-?[1-9][0-9]*)$`, with `0` for zero, no `+`, no
leading zero and `-` only before a nonzero digit. The same spelling is the one
FR-016 canonicalizes, so the wire and the canonical bytes agree. A member of
those eight that is a JSON number, whatever its value, and one that is a string
outside that grammar (empty, `+1`, `01`, `-0`, `1.0`, `1e3`, or with surrounding
whitespace) refuses `invalid_wire_format`, decided by the member's type: the
loaders make no separate scan of the document for large numerals. A string in
the grammar whose value lies outside the closed range of its member refuses
`invalid_numeric_bounds`. Each of these eight members is decoded as a signed
128-bit value before its type-specific bound is checked. The seven formerly
`i64` members and the formerly `u64` `maximum_denominator`
accept `i128::MIN..=i128::MAX` where their sign and declared bounds permit.
A grammar-valid string one past either i128 endpoint refuses
`invalid_numeric_bounds`, never `invalid_wire_format`. A collection's
`maximum_items` stays a JSON number in the unsigned 32-bit range. No earlier
number spelling of the eight members is read.

Text is a sequence of Unicode scalar values with no normalization or locale
folding. A text value contains at most `1048576` scalar values; an over-length
literal uses `text_bound_exceeded`.

Every declaration, enum variant, record field, function parameter, input, and
state carries a source span. Public validation limits are
`maximum_expression_nodes = 10000` and `maximum_expression_depth = 256`; these
are wire-independent semantic constants, not host pointer-sized values.
FR-019 additionally bounds every complete semantic request, including nested
value types and declaration collections, before recursive validation begins.

Enum and record declarations share one type-name namespace. Enums contain at
least one unique variant. Records contain unique fields. Named types must
resolve, and the directed record-containment graph through record, option, and
collection fields must be acyclic. Input and state value declarations share one
value-name namespace and bind one value type. Pure-function declarations have
unique names, unique parameter names, ordered parameter types, and a result
type. Every public name uses the identifier grammar.

Validation is deterministic and fail closed. Declaration grammar and bounds
precede duplicate, orphan, and recursive checks. A valid environment round
trips structurally through public constructors and accessors with equality;
FR-016 owns canonical bytes and FR-018 later owns the published wire schema.

A rational is normalized exactly when its denominator is positive and
`gcd(the absolute magnitude of numerator, denominator) = 1`; zero is
represented as `0/1`. Every
rational literal and arithmetic result is normalized before numerator and
denominator bounds are checked. The magnitude of `i128::MIN` is `2^127`;
normalization computes that magnitude without signed `abs` or unchecked
negation, so `i128::MIN/1` is a valid single rational literal when its type
contains it.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-013-AC-1 | Every closed value-type and declaration construct has positive and negative fixtures with deterministic STD-001 diagnostics. | Test (TC-016) |
| FR-013-AC-2 | Public serialized types contain no Rust, GUMBO, AADL, HAMR, solver, or runtime-specific vocabulary. | Inspection (TC-016) |
| FR-013-AC-3 | Duplicate names/fields/variants, empty enums, zero or over-unsigned-32-bit collection declarations, invalid integer bounds, absent named types, direct or indirect record cycles, and FR-019 semantic node/depth/collection limit breaches fail before expression validation. | Test (TC-016, TC-018) |
| FR-013-AC-4 | A valid declaration environment round trips structurally through public constructors/accessors without losing declaration identity, types, bounds, overflow policy, or provenance spans. | Test (TC-016) |
| FR-013-AC-5 | Integer bounds and literals at `i128::MIN` and `i128::MAX`, an unsigned type `[0, u64::MAX]` with its maximum literal, rational numerator bounds and literals at both i128 endpoints, and positive maximum denominator and literal denominator at `i128::MAX` decode and serialize back to the same decimal strings. Each of the eight members given as a JSON number (`0`, `1`, `1.0`, `9223372036854775807` and `100000000000000000001`) or a string outside the minimal-decimal grammar (`""`, `"+1"`, `"01"`, `"-0"`, `"1.0"`, `"1e3"`, `" 1"`) refuses `invalid_wire_format`. Each of the eight given `"170141183460469231731687303715884105728"` or `"-170141183460469231731687303715884105729"` refuses `invalid_numeric_bounds`; `"0"` and negative strings for positive denominator members refuse `invalid_numeric_bounds`. | Test (TC-016, TC-018) |
| FR-013-AC-6 | Public `IntegerType` bounds/accessors, `RationalType` numerator bounds and maximum-denominator/accessors, and integer/rational literal payloads retain `i64::MAX + 1`, `u64::MAX`, and the permitted i128 endpoints without narrowing; one-past-endpoint wire values return `invalid_numeric_bounds` without a panic or partial validated value. | Test (TC-016, TC-018) |

## Dependencies

FR-011 supplies package-scoped declaration identity.
