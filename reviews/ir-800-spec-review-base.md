---
id: SR-11191
title: "Base review of IR-800 native scalar reader requirements"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; FR-038-AC-242..245 and TC-048 native String/UUID/Timestamp section"
review_set: subset
---

## Summary

Ticket IR-800. Reviewed all four new criteria and the TC-048 procedure against QSpec FR-322-AC-31/67 at 9b6a84d, including canonical literal spellings, typed refusal order, selected type ownership, source identity, and the IR/QSL/FCD boundary. The initial review found two adverse-control gaps; both are fixed in the first disposition pass on 3fccf18.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Test the wrong-family operand in both argument positions for each native comparison; a reader that validates only one side can pass the current one-sided mutation procedure. | TC-048 native admission, FR-038-AC-243 |
| FND-002 | medium | Test foreign ordering and arithmetic over String and Timestamp as well as UUID; the requirement refuses them over all three native families, but the TC names only UUID controls. | FR-038-AC-243, TC-048 native admission |

## Checked controls

FR-038-AC-242 retains distinct self-typed scalar nodes and selected member types; FR-038-AC-243 covers six catalog identities and positive same-family comparisons; FR-038-AC-244 covers law/mode mismatch and String versus Text; FR-038-AC-245 covers type-before-value, canonical UUID/Timestamp boundaries and exact String Unicode. The changed spec introduces no copied fixture, alternate FCD schema, or non-identity hash/pin. `quire validate` reports 2/2 changed documents grammar-clean; `make spec` retains the existing 55 unbacked and 0 contradicted strict rows.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | TC-048 now specifies twelve distinct wrong-family cases: each of six native comparisons with a foreign argument at position 0 and then 1, retaining a valid native counterpart and checking the changed argument pointer. |
| FND-002 | fixed | FR-038-AC-243 and TC-048 now specify integer lt and add over String, UUID and Timestamp separately, with `/arguments/0` refusal. |

### After excerpts

FND-001 (`TC-048`):

```text
For **each** of the six admitted identities,
replace argument 0 with an operand of a foreign family, restore it, and then
replace argument 1, keeping the other argument native and every other
catalogued member valid.
```

FND-002 (`TC-048`):

```text
Separately apply both
`quire.op.integer.lt` and `quire.op.integer.add` to a pair of `string`
operands, then to `uuid` operands, then to `timestamp` operands: six more
mutations, each refusing `ill_typed`/`operator-ineligible` at
`/arguments/0`.
```
