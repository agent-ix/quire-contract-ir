---
id: TC-447
title: "Admitted scalar result context identifies the exact reverse bound"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-049
    type: verifies
---
# TC-447: Admitted scalar result context identifies the exact reverse bound

## Description

PLANNED/UNRUN (IR-769). Verify FR-049-AC-1 through FR-049-AC-6 against the
public reader-admitted QSL-to-IR path and focused adversarial consumer shapes.

## Test Procedure

Check and emit the QSL function in FR-049-AC-1, admit its checked package,
lower the exact add expression into one FR-330 package, read that package, and
query its typed scalar result context. Independently derive the expected
`[0, 10]` result and `[0, 9]` operand ranges from the QSL declaration, then
compare exact typed keys, admitted source occurrences, source package identity
and lowering profile. Do not compute the expected result by calling the
context accessor or reusing its bound selection.

For separate reader-admissible packages, exercise no conversion, one direct
conversion, two conversions to one target, two distinct targets, a conversion
with a second reference consumer, an unrelated reference only, and an
expression directly typed by a bounded domain both with and without a reverse
conversion, plus a direct bounded domain of the wrong expected form. Exercise
one valid plus one wrong-form or incompatible conversion,
including where valid conversions name two distinct bounds. Exercise wrong
form, unreadable target and scalar type mismatch at the private
post-admission seam only when the public reader would already refuse that
shape; assert the reader's independent refusal first. Exercise non-lowered,
unrequested and absent keys, plus post-admission inconsistent source/body
and work-limit failures. Include a selected external reference, an identically
keyed local node, and missing, stale and wrong selected dependencies; assert
reader refusal before any result-context query for unavailable selection.

Before and after every query, compare the complete six-field wire and
canonical identity. Independently enumerate the preexisting forward closure
and confirm the reverse conversion remains outside it. Compile an external
consumer that obtains the typed result using only `ContractPackageV1`, the
requested key and expected form, with no original checked graph.

## Expected Results

The QSL positive returns the exact result bound `[0, 10]`, while the operand
bound remains `[0, 9]`. The other cases produce the exact `DirectBounded`,
`Unnarrowed`, `Narrowed`, `MissingBound`, `AmbiguousBound`, non-lowered, absent or typed error
outcomes in FR-049, without a partial or guessed bound. Selected external
references retain their package/key identity and never resolve through a
local collision. Source/provenance is exact. The FR-330 wire, identity and
forward closure are unchanged by every query.
