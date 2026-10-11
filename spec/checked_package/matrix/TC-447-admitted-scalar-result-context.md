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
and record the driver's item occurrence, add key and
`conversion_context() -> Option<&OccurrenceKey>`. Require `Some` of the
direct narrow occurrence and retain its role and ordinal. First lower only
`[add]`: read the ordinary six-field FR-330 package and assert that its add
forward closure has six nodes, operand `[0, 9]`, and neither the narrow nor
result `[0, 10]`. Then lower explicit roots `[add, narrow]` together: read the
ordinary FR-330 package, assert two exact feature-accounting records, eight
represented nodes, the unchanged six-node add forward closure, and exact
source occurrences for the narrow and result bound. Query the admitted
context with that item's application and direct-narrow occurrences. Assert
that the add-only package returns `IncompleteContext` for the same `Some`
input. Independently derive expected
`[0, 10]` and `[0, 9]` from the QSL declaration, not from the context accessor.

For separate reader-admitted packages, exercise QSL `None` with an own bounded
type, an own scalar type, and a wrong-form own bounded type. Exercise QSL
`Some` with a valid direct narrow over an own bounded type, a wrong-form or
incompatible target, and unrelated represented conversions or other local
consumers. Confirm only the selected occurrence determines the result. In
defensive IR cases, change one supplied `Some` occurrence at a time to an
absent, non-lowered, wrong-role, wrong-ordinal, wrong-site, wrong-shape or
wrong-argument conversion, and assert `IncompleteContext` before any bound
is returned. Check QSL's zero/multiple directly wrapping narrow failure as
a QSL pre-admission refusal, rather than manufacturing an admitted IR package
for a source QSL could not emit.

Build the QSL case with two requirement occurrences sharing one add key and
different narrow occurrences and result bounds. Keep both narrows represented
in one FR-330 package and query each item using its own application and
conversion occurrences, then swap the two contexts and require
`IncompleteContext` rather than the other item's bound. Exercise non-lowered,
unrequested and absent expression keys, post-admission inconsistent
source/body edges and work-limit failures. Include a selected external
reference, an identically keyed local node, and missing, stale and wrong
selected dependencies; assert reader refusal before any result-context query
for unavailable selection.

Before and after every query, compare the complete six-field wire and
canonical identity. Compare the add-only package to the contextual package
separately: the extra root changes represented nodes, accounting and identity
as FR-035 requires. Independently enumerate the preexisting add forward
closure and confirm the reverse conversion remains outside it. Compile an
external consumer that obtains the typed result from the admitted FR-330
package and the checked request's application occurrence plus
`Option<&OccurrenceKey>` conversion context, with no original checked graph
or second lowering call. Assert that the context-only narrow creates no extra
routed item, claim or proof obligation.

## Expected Results

The QSL positive lowers `[add, narrow]` and returns the exact result bound
`[0, 10]`, while the operand bound remains `[0, 9]`. Lowering `[add]` alone
cannot satisfy the same `Some` context. The other cases produce the exact
`DirectBounded`, `Unnarrowed`, `Narrowed`, `MissingBound`, `IncompleteContext`,
non-lowered, absent or typed error outcomes in FR-049, without a partial or
guessed bound. Selected external
references retain their package/key identity and never resolve through a
local collision. Source/provenance is exact. Each FR-330 package has the
ordinary six-field wire; the contextual package has different accounting and
identity from add-only. The wire, identity and add forward closure are
unchanged by each query.
