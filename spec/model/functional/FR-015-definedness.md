---
id: FR-015
title: "Track partial-operation definedness"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-014
    type: depends_on
---
# FR-015: Track partial-operation definedness

## Description

Validation shall compute and discharge definedness obligations for option
access, collection indexing, division, remainder, bounded arithmetic, and
guarded subexpressions.

## Inputs

Typed expressions, numeric overflow policy, collection bounds, and guard facts
available at each evaluation point.

## Outputs

An executable expression with explicit definedness obligations or structured
potential-undefined diagnostics.

## Behavior

Each partial node creates one explicit obligation: option presence; non-zero
divisor; non-negative and below-length index; or result within checked integer
bounds. Declaration facts and dominating guard facts may discharge an
obligation. Saturating addition, subtraction, multiplication, and negation are
total. Division and remainder always raise the non-zero-divisor obligation
under both integer overflow policies; `reject` integer arithmetic additionally
needs a range proof, including the minimum-divided-by-negative-one case.
Bounded rational arithmetic always needs a numerator/denominator range proof.
Constant ranges and declared numeric bounds provide range facts.

Checked-range analysis is a closed bottom-up finite interval-set calculation.
Literal ranges are singleton values. Reference ranges begin at declared bounds
and intersect every dominating lower/upper fact whose resolved structural
subject is identical. A non-zero fact splits a range containing zero into at
most two ranges ending at `-1` and beginning at `1`. A divisor range that still
contains zero fails `checked_range` after its non-zero obligation.

Add, subtract, multiply, negate, integer divide/remainder, and normalized
rational operations apply mathematical interval arithmetic to every range
pair. Field access, option unwrap, collection index, collection length, and
pure-function call start at the declared bounds of their numeric result type
and then intersect dominating exact-subject facts by the same rule as
references; no node other than an exact-subject fact narrows that range.
The interval calculation uses mathematical endpoints, including when an
endpoint operation, `abs(i128::MIN)`, negation, cross-product, or gap width
exceeds signed 128-bit storage. For an exact singleton rational result, the
checker establishes the mathematical fraction and normalizes it before the
final bounds check, using cancellation or sufficient precision when raw
products do not fit i128. Raw intermediate width alone cannot refuse a
singleton whose normalized numerator and denominator fit the declared type.
For a non-singleton range, the checker uses a sound envelope; if it cannot
establish that every normalized result fits the declared bounds, it reports
`potentially_undefined` with the checked-range obligation. No calculation
wraps, panics, silently clamps a checked result into the bounds, or loses an
obligation. A saturated integer operation clamps to the declared type bounds
even when its mathematical intermediate is outside i128.

For normalized rational operands `a/b` and `c/d`, add/subtract use numerator
`a*d +/- c*b` and denominator `b*d`; multiply uses `a*c` and `b*d`; divide uses
`a*d` and `b*c`; and negate preserves `b`. The checker applies these operations
to bound endpoints, rejects a possible zero `c`, normalizes exact singleton
results before judging their bounds, and otherwise uses a sound unreduced worst
case. A checked-range obligation
is discharged exactly when every result range is contained in the named
numerator/integer bounds and every possible positive denominator stays within
its declared maximum.

Guard extraction recognizes option-presence tests, equality/inequality with
zero, index comparisons with zero and collection length, and ordering
comparisons between a numeric operand and a literal bound. Short-circuit
conjunction and implication pass facts true of their left/antecedent into the
right branch. Short-circuit disjunction passes facts false of its left into the
right branch. Negation swaps true/false facts. Total Boolean operators validate
both operands under the incoming facts and contribute no cross-operand facts.
Quantifier predicates inherit outer facts but do not leak local facts.

A fact discharges an obligation only when the guarded subexpression and the
obligation subject are structurally identical after declaration resolution,
including `current`/`pre`/`post` observation and every field, index, and call
node. Facts mentioning a quantifier local match only within that local's
predicate. Index-domain quantifiers contribute their automatic fact only for
the exact domain collection and bound local.

Unchecked option access, a possible zero divisor, an index not proven within
bounds, or checked arithmetic whose result range can exceed its declared type
fails with `potentially_undefined` at the partial node span. Successful typed
nodes retain their discharged obligation kind and the declaration or guard span
that discharged it. When several facts contribute, the retained proof span is
the innermost contributing guard; ties use the earliest authored pre-order. The
declaration span is retained only when no guard contributed.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-015-AC-1 | Every partial operation is accepted only when its obligation is statically established by declarations or dominating guards. | Test (TC-016) |
| FR-015-AC-2 | Replacing a short-circuit guard with a total Boolean operator exposes the previously guarded undefined operation. | Test (TC-016) |
| FR-015-AC-3 | True/false facts from conjunction, disjunction, implication, and negation apply only to their conditionally evaluated branch and never escape a quantifier scope. | Test (TC-016) |
| FR-015-AC-4 | Reject-overflow arithmetic uses constant/declaration ranges; saturating add/subtract/multiply/negate are total; and zero divisors or possible minimum-divided-by-negative-one overflow fail at the operator span. | Test (TC-016) |
| FR-015-AC-5 | Every `potentially_undefined` diagnostic carries the exact closed `obligation_kind`; every discharged node retains the deterministic declaration/guard proof span. | Test (TC-016) |
| FR-015-AC-6 | Bottom-up range sets from literals, declaration bounds, exact-subject guards, field/unwrap/index results, and pure calls accept only results wholly contained in the named numeric type. | Test (TC-016) |
| FR-015-AC-7 | Positive and negative bounded-rational fixtures pin normalization plus numerator/denominator propagation for add, subtract, multiply, divide, and negate. | Test (TC-016) |
| FR-015-AC-8 | At an `i128::MIN` endpoint, integer `reject` negation and division by `-1` report `potentially_undefined` with the checked-range obligation at the operator span; `saturate` negation returns the declared maximum when its mathematical result exceeds that bound. Rational `i128::MIN/1` is admitted and normalizes without panic. In the rational type with numerator bounds `[0, i128::MAX]` and maximum denominator `i128::MAX`, the singleton product `i128::MAX/(i128::MAX - 1) * (i128::MAX - 1)/i128::MAX` is admitted as normalized `1/1` despite both raw products exceeding i128. In that type, `i128::MAX/1 * 2/1` reports `potentially_undefined` with the checked-range obligation at the multiplication span because its normalized numerator exceeds the bound. Negation of rational `i128::MIN/1` under bounds `[i128::MIN, i128::MAX]` likewise reports that obligation at the negation span. None wraps or panics. | Test (TC-016) |

## Dependencies

FR-014 defines evaluation semantics.
