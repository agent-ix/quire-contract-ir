---
id: TC-469
title: "Reached model and unit prerequisite"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-357
    type: verifies
---
# TC-469: Reached model and unit prerequisite

## Description

Exercise the package-private bounded model and unit fact read from an admitted
monitor root with IR-owned producer rows. The root reaches a selected record,
object, source unit and derived model unit through typed Inputs fields. A second Inputs
field reaches the same record; a selected variant and another model declaration
remain outside the closure.

## Test Procedure

Build the admitted monitor package and its selected document in the IR test
builder. Supply opaque declaration, field and unit rows. Read at an ample
allowance, the measured exact allowance, and one less. Repeat on an equal
package clone, with an inspected unrelated row, and with an unrelated selected
model inserted before the reached model. Omit required rows, alter selected
owner and original identity claims, supply stale field and unit rows, duplicate
a declaration row, and reach the variant without its QSL-issued shape and case
variants. Reach field, operation and ancestry references to declared but
unemitted model nodes; omit the conversion target's unit identity row; and
reach a field whose admitted Float32 type cannot be represented in Exact.

## Expected Results

The read returns one record, one object and their ordered complete fields,
own object operations with admitted effects, one source unit and a two-unit
model conversion-target chain with distinct owners and exact conversions. It
retains the supplied Exact identities, deduplicates the shared record, and excludes unrelated
selected declarations and variants. The exact allowance succeeds; one less
refuses with an exhausted typed cause. Each missing, foreign, duplicate or
unwitnessed reached variant claim refuses at a native node or row, with no
partial facts. The package clone agrees and an unrelated inspected row adds
charged work without changing returned facts. The unrelated selected model
does not change the exact allowance. Changed owner/original, stale rows,
unresolved reached references and unrepresentable reached types refuse with
their typed native causes and no partial facts.
