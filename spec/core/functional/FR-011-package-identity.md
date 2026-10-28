---
id: FR-011
title: "Identify versioned contract packages and requirement revisions"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
---
# FR-011: Identify versioned contract packages and requirement revisions

## Description

The model shall identify a contract package, its schema version, every
requirement ID and revision, and the source document revision from which it was
derived.

## Inputs

Package namespace, schema major/minor, requirement identifier, positive current
requirement revision, and source-document identity including its exact revision.

## Outputs

Validated package and requirement identities suitable for serialization and
downstream citation.

## Behavior

Identity validation rejects empty namespaces, malformed identifiers, a zero
schema major, zero source or requirement revisions, duplicate requirement identifiers, and
references to a different package. Changing a requirement revision changes every derived downstream
identity that cites it. A package contains one current positive revision per
requirement. Revision gaps are legal; when a caller advances a known prior
revision, the replacement shall be strictly greater than the prior value.
The public `ExecutionPoint` has four variants with nonempty string names and
is also the checked-package V2 point vocabulary (FR-058). The broader V2 name
domain does not relax authored-contract identity validation. Authored clause
constructors and the package, expression-operation and binding-expression
wire decoders SHALL validate each execution-point name using the existing
`AnchorName` identifier rule before admitting it, and SHALL return the
registered `invalid_identifier` diagnostic for an empty or malformed authored
anchor. A validated `AnchorName` supplies the canonical point name without
creating a second public point type or another name-validation rule. V2
admission instead applies QSpec's closed four-variant, nonempty-name shape;
it SHALL retain a valid name even when the authored identifier rule would
reject it.
The registered codes of this requirement are those of the constructors and of
the typed decoders that run with no schema stage before them (the package
decoder, the expression operation decoder, the binding expression decoder). A
whole executable projection is checked against its published schema first
(FR-023), so a revision above 2^53 in a projection is refused
`invalid_wire_format` there (FR-023-AC-6), not with the registered code.
A source or requirement revision is at most 2^53 (9007199254740992), the
largest integer RFC 8785 spells exactly: a larger value is refused at
construction and in the non-canonical JSON value representation, so a canonical
object (FR-016) never holds a revision it cannot encode.

The model owns an implementation-language-independent, non-canonical JSON value
representation for its public identities and package structure. A valid value
round trips with structural equality. JSON member order and byte spelling are
explicitly non-normative here; FR-016 later owns canonical bytes and digests,
and FR-018/FR-020 later publish the complete schema and conformance interface.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-011-AC-1 | A package round trip through the non-canonical JSON value representation preserves namespace, schema version, requirement ID, requirement revision, and source-document identity/revision with structural equality. | Test (TC-015) |
| FR-011-AC-2 | Incrementing one requirement revision changes its clause and dependency identities without changing unrelated requirement identities. | Test (TC-015) |
| FR-011-AC-3 | Empty or malformed package, source-document, requirement, clause, authored anchor, or dependency-path-segment identities; a zero schema major; zero source or requirement revisions and a revision above 9007199254740992 (`invalid_source_revision`, `invalid_requirement_revision`, through the constructors, the package decoder, the expression operation decoder and the binding expression decoder; a whole executable projection is FR-023-AC-6's); duplicate requirement or clause identifiers; non-increasing revision advances; and cross-package references fail with their registered structured diagnostic codes. An empty or malformed authored execution-point name, including one supplied through the public canonical point to an authored clause constructor, returns `invalid_identifier` under the existing `AnchorName` rule. The same nonempty name may still be admitted through V2 when QSpec permits it. Schema minor zero is valid. | Test (TC-015) |

## Dependencies

None.
