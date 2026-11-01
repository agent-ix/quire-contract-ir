---
id: FR-356
title: "Admit named option value type definitions"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-349
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-154
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
---
# FR-356: Admit named option value type definitions

## Description

When a checked-package V2 reader selects a Semantic IR model containing a type
definition of `kind` `option`, Contract IR SHALL admit that definition as
`Option<T>` precisely under QSpec FR-154-AC-39..41 and FR-322-AC-298. It SHALL
retain the existing three typeRef forms and SHALL NOT introduce a native Option
constructor spelling, flatten nested options or infer new source semantics.

## Inputs

A checked package selecting an identity-bound Semantic IR model document through
FR-038 admission evidence, the document's option definitions and references, and
the caller's existing read provisions. An option definition has one `inner` typeRef.
The published option corpus is read by reference from the owning QSpec checkout;
no vector or schema is copied into Contract IR.

## Outputs

An admitted package whose canonical member-type derivation wraps the named inner
value type in one option. Where the existing typed projection cannot represent
an otherwise admitted inner domain it retains its existing unavailable result.
A selected-document refusal retains the owning code/cause and exact document
pointer in addition to the selecting model-root locus, without a partial package.

## Behavior

The reader SHALL resolve aliases and named definitions without changing a typeRef's
syntax. An option's inner may name a native value type other than Clock, a unit,
or a package value type including record, tuple, variant, sequence, map and reference
forms. The option may occur as a field, parameter, result, alias target, sequence
item, map value, tuple position or variant payload.

The reader SHALL refuse direct, alias-mediated and self nesting of option types,
Clock and aliases of Clock, and non-value inner targets under the QSpec option
row. It SHALL refuse a reference target naming an option. A well-formed unresolved
package inner returns missing-declaration/missing-name rather than malformed-declaration.

For one option definition, the reader SHALL preserve QSpec's refusal order: shape
members, constraints, inner form, inner resolution, then inner kind. Forbidden shape
members include fields, identityFields, operations, supertypes, clauses, positions,
variants, items, values and target, even when the supplied list is empty. Nonempty
option constraints and a constraint applying to an option refuse at their own
constraints or appliesTo member. Absent, null, empty, non-string, array and object
inner forms refuse at inner. Earlier checked-package stages retain their precedence.

A required option-typed field and an optional inner-typed field at multiplicity
[1,1] SHALL derive the same canonical composite_type/option node. Presence optional
on an option-typed field at [1,1] refuses at presence after typeRef checks, regardless
of ordered/unique flags. Collection multiplicity wraps the option element; optional
presence on a collection wraps the collection. Neither case is flattened into the
other or into a nested option.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-356-AC-1 | The public reader admits named Integer, unit and record option definitions and references in fields, parameters, results, aliases, sequence/map definitions, tuple positions and variant payloads under QSpec FR-154-AC-39. Existing native/package/unit typeRef forms remain unchanged. | Test |
| FR-356-AC-2 | Required named options and optional inner fields share the canonical option node; Option<Integer>, Option<Quantity<ms>>, Sequence<Option<Integer>>[0,3], and optional collection wrapping match OP-01..08 and FR-322-AC-298 without flattening. | Test |
| FR-356-AC-3 | Direct/alias/self nested options, Clock/alias of Clock, a namespace inner, reference-to-option, forbidden native Option spellings and optional single-valued option fields refuse with the published codes, causes and exact selected-document paths, including typeRef-before-presence precedence. | Test |
| FR-356-AC-4 | Missing/null/empty/non-string/array/object inner, unresolved inner, forbidden shape members, option constraints and appliesTo-option constraints produce their owning code/cause/locus in shape/constraints/form/resolution/kind order. Earlier package-stage refusals remain earlier. | Test |
| FR-356-AC-5 | A cross-reader gate reads every OP-01..41 row from current QSpec by reference, checks all expected admissions/types/refusals and exact document loci through production admission, and reports the QSpec checkout HEAD separately from implementation data. No copied vectors, schema snapshots, compatibility path or FCD changes are introduced. | Test |

## Dependencies

FR-038 owns strict admission and limits; FR-349 owns retained typed terms and
canonical member-type interpretation. QSpec FR-154 owns option admission and
FR-322 owns structural option identity. The FCD kind/schema/extractor/API leg of
IR-1239 is separate and is not implemented by this requirement.
