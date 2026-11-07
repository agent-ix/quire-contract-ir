---
id: TC-472
title: "Inherited operation dispatch witness admission"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-354
    type: verifies
---
# TC-472: Inherited operation dispatch witness admission

## Description

Verify that strict dispatch admission preserves a call's static receiver
identity while validating its distinct selected operation-family witness,
and refuses mismatched witnesses outside that family.

## Test Procedure

Use an authentic selected-model package with `Sub` inheriting the `size`
operation family declared by `ConfigVersion`, and a checked call retaining
`Sub` as its operation member declaration. Supply the captured dispatch row
whose declaring witness is `ConfigVersion`, static receiver context `Sub`,
complete candidates including the `ConfigVersion` family candidate, matching
candidate witnesses, genuine selected model documents, and sufficient reader
limits. Read the package through the public strict reader, then read the
exact application and occurrence through the public dispatch projection.
Compare its complete operation witness, context and candidate sequence with
the supplied row, and inspect the retained call member declaration. Include a
direct operation whose member and table declaring identities are equal.

Starting from the genuine inherited fixture, change only the table's declaring
witness to a selected unrelated type with a same-named operation. Independently
construct a mismatch with no admitted inherited family relationship, including
an ancestor's same-named independent operation without a connecting declared
`redefines` edge. Keep the rest of each package admitted and compare the full
typed refusal code, cause and pointer against independently stated expected
values. Do not replace the original static receiver identity, synthesize a
successful family relation from names, or skip the strict reader.

## Expected Results

The inherited and direct packages admit. The inherited package retains `Sub`
as the call member declaration and receiver context and `ConfigVersion` as the
selected table witness; the family candidate need not conform to `Sub`; its candidate order and complete selected-operation witnesses are
unchanged. Both independently malformed witnesses refuse `InvalidPackage`
with `InvalidValue` at
`/checked_term_data/dispatch_tables/0/operation`, with no admitted package or
partial dispatch view. Equal names and bare ancestry cannot authorize an
unrelated family witness. The implementation uses the bounded retained-model
resolver and preserves the reader's charged exhaustion outcome.
