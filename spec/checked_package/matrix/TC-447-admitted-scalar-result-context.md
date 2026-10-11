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
and record its one driver requirement occurrence, add key and authoritative
enclosing narrow key from the reviewed QSL-710 request API. First lower only
`[add]`: read the ordinary six-field FR-330 package and assert that its add
forward closure has six nodes, operand `[0, 9]`, and neither the narrow nor
result `[0, 10]`. Then lower explicit roots `[add, narrow]` together: read the
ordinary FR-330 package, assert two exact feature-accounting records, eight
represented nodes, the unchanged six-node add forward closure, and exact
source occurrences for the narrow and result bound. Query the admitted
context for the same requirement occurrence. Independently derive expected
`[0, 10]` and `[0, 9]` from the QSL declaration, not from the context accessor.

For separate reader-admissible packages, exercise no conversion, one direct
conversion, two conversions to one target, two distinct targets, a conversion
with a second reference consumer, an unrelated reference only, and an
expression directly typed by a bounded domain both with and without a reverse
conversion, plus a direct bounded domain of the wrong expected form. Exercise
one valid plus one wrong-form or incompatible conversion,
including where valid conversions name two distinct bounds. Exercise wrong
form, unreadable target and scalar type mismatch at the private
post-admission seam only when the public reader would already refuse that
shape; assert the reader's independent refusal first. Omit one local consumer
from an otherwise valid contextual root selection and assert typed
incompleteness, without accepting a narrowed or unnarrowed result. Exercise
two QSL requirement occurrences that share an expression key but carry
different result bounds and conversion selections; retain distinct contexts
by occurrence. Exercise non-lowered, unrequested and absent keys, plus
post-admission inconsistent source/body and work-limit failures. Include a
selected external reference, an identically keyed local node, and missing,
stale and wrong selected dependencies; assert reader refusal before any
result-context query for unavailable selection.

Before and after every query, compare the complete six-field wire and
canonical identity. Compare the add-only package to the contextual package
separately: the extra root changes represented nodes, accounting and identity
as FR-035 requires. Independently enumerate the preexisting add forward
closure and confirm the reverse conversion remains outside it. Compile an
external consumer that obtains the typed result from the admitted FR-330
package and its validated occurrence-scoped context, with no original checked
graph or second lowering call. Assert that the context-only narrow creates no
extra routed item, claim or proof obligation.

## Expected Results

The QSL positive lowers `[add, narrow]` and returns the exact result bound
`[0, 10]`, while the operand bound remains `[0, 9]`. Lowering `[add]` alone
cannot produce that result context. The other cases produce the exact `DirectBounded`,
`Unnarrowed`, `Narrowed`, `MissingBound`, `AmbiguousBound`, non-lowered, absent or typed error
outcomes in FR-049, without a partial or guessed bound. Selected external
references retain their package/key identity and never resolve through a
local collision. Source/provenance is exact. Each FR-330 package has the
ordinary six-field wire; the contextual package has different accounting and
identity from add-only. The wire, identity and add forward closure are
unchanged by each query.
