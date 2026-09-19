---
id: FR-342
title: "Admit exact OCL 2.4 correspondence"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-341
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-122
    type: implements
  - target: ix://agent-ix/quire-specification/NFR-061
    type: constrained_by
---
# FR-342: Admit exact OCL 2.4 correspondence

## Description

When constructing an OCL 2.4 mapper, the mapper shall admit only profile
`quire.output.ocl24/v1` revision `1-draft.1` and a finite typed correspondence
catalog bound to the exact [FR-341](FR-341-bind-output-mapper-to-request.md)
request context.

## Inputs

- One complete mapper binding whose target family is OCL 2.4.
- A finite catalog containing exactly one entry per requested obligation, with
  its context, constraint name, optional zero-argument operation anchor, values,
  fields, optional total-definedness rule, relationship endpoints, and selected
  Sequence-operation correspondences.
- Exact owner, contract, revision, raw digest, and dependency kind for every
  authoritative correspondence used by a mapping decision.

## Outputs

- One immutable OCL mapper with canonical catalog order and constructor-private
  correspondence entries.
- One typed construction refusal for an absent/extra obligation entry,
  duplicate, ambiguous, foreign, stale, ill-formed, over-limit, or cross-request
  correspondence catalog.

## Behavior

- The mapper shall accept only nonempty ASCII identifiers matching
  `[A-Za-z_][A-Za-z0-9_]*`, excluding the mapper's closed case-sensitive OCL 2.4
  keyword set, and qualified names composed only from those identifiers.
- The mapper shall reject arbitrary OCL snippets, expressions, headers,
  comments, and files as correspondence data.
- The mapper shall bind every context, operation, value, field, and selected
  Sequence operation to one exact source symbol and one exact target name.
- The mapper shall require an exact context and anchor correspondence for each
  represented invariant, precondition, or postcondition.
- The mapper shall preserve every actually used model, type, anchor, field,
  operation, semantic-profile, and definedness dependency in the candidate's
  ordered dependency set.
- The mapper shall canonicalize catalog order without resolving duplicates by
  first-wins, last-wins, display-name fallback, case folding, or ambient model
  lookup.
- The mapper shall reject more than 10,000 catalog entries, more than 10,000
  correspondence members in one entry, or more than 1,048,576 aggregate bytes
  of target identifiers before exposing a mapper.
- The mapper shall contain no OCL parser, JVM, Maven, Eclipse, Electron, network,
  filesystem, clock, locale, environment, or plugin dependency.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-342-AC-1 | Equal correspondence entries in any input order construct one equal mapper binding and produce byte-identical mapping candidates. | Test (TC-220) |
| FR-342-AC-2 | Missing/extra obligation entries, duplicate, ambiguous, stale, foreign, cross-request, invalid-identifier, reserved-word, 10,001-entry/member, and 1,048,577-byte catalogs refuse construction without a usable mapper. | Test (TC-220) |
| FR-342-AC-3 | Every represented context, anchor, value, field, operation, type, and semantic rule appears as an exact typed dependency in the resulting mapping record. | Test (TC-220) |
| FR-342-AC-4 | Arbitrary target snippets and all ambient/foreign-runtime inputs are absent from the public construction and mapping interfaces. | Test (TC-220) |

## Dependencies

- [FR-341](FR-341-bind-output-mapper-to-request.md) binds catalog meaning to
  the exact admitted source/model/semantic selections.
- `ix://agent-ix/quire-specification/FR-122` owns the selected bounded OCL
  correspondence; `NFR-061` prohibits a foreign-runtime semantic dependency.
