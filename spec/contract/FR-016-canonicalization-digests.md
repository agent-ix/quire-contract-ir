---
id: FR-016
title: "Canonicalize contracts and compute stable identities"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-002
    type: traces_to
---
# FR-016: Canonicalize contracts and compute stable identities

## Description

The library shall define one canonical encoding and SHA-256 identity for every
supported package, requirement revision, clause, declaration, and expression.

## Inputs

A validated v0.1 package and the canonicalization profile identity.

## Outputs

Canonical UTF-8 JSON bytes and lowercase SHA-256 digests for every addressable
semantic object.

## Behavior

The only v0.1 profile identity is
`quire.contract.canonical-json/v1`. A canonical object is the UTF-8 encoding of
an envelope whose members, after Unicode-scalar key sorting, are emitted in the
exact order `kind`, `profile`, `value`:
`{"kind":<kind>,"profile":<profile>,"value":<semantic-value>}`. Object member
names always sort by Unicode scalar value; authored member order is never an
exception. Arrays preserve semantic order. JSON
strings emit `\"`, `\\`, `\b`, `\t`, `\n`, `\f`, and `\r`; other U+0000 through
U+001F controls use lowercase four-digit `\u00xx`; every other Unicode scalar
is emitted directly as UTF-8. `/` and non-ASCII scalars are not escaped.
Integers use minimal base-ten spelling, Boolean values are lowercase, and no
floating-point or null value exists in the canonical model.

The closed object kinds are `package`, `requirement`, `clause`, `declaration`,
and `expression`. Every kind has a source-free semantic projection. Package
requirements and requirement clauses sort by identifier. Type, value, and
function declaration namespaces sort by name. Enum variants and record fields
sort by name because their identities, not authored positions, define them.
Function parameters, collection items, call arguments, operands, and nested
expression children retain authored semantic order. Record-literal fields sort
by name. Dependency and discharged-obligation collections are derived and are
not duplicated in expression bytes. Source identities, source spans, binder
spans, typed-node indexes, diagnostics, and review/evidence metadata are not
semantic content and are excluded.

The package semantic value includes `schema_version` as the object
`{"major":<u16>,"minor":<u16>}` in addition to the package namespace and its
complete requirements. Consequently the registered 1.0-to-1.1 migration
changes the package canonical bytes and package digest even when every other
semantic field is preserved.

Every value type and expression variant uses its registered snake-case tag and
all fields that affect type checking or execution. Rational literals use their
normalized numerator and positive denominator. A typed expression canonicalizes
only after successful validation and includes its explicit result type plus the
normalized expression tree. Declaration-environment owner identity is included;
the three declaration namespaces use the ordering above.

The lowercase digest for kind `K` is SHA-256 over the exact byte sequence
`quire-contract-ir`, one zero byte, the profile identity, one zero byte, `K`,
one zero byte, then the canonical object bytes. This domain separation is part
of the profile. Package canonical bytes contain complete sorted requirements
and clauses rather than host map iteration or child digest placeholders.
Requirement and clause digests use the same independently canonicalized
objects, so changing one clause changes that clause, its enclosing requirement,
and its package while unrelated clause digests remain unchanged.

Canonicalization is defined only for values already accepted by FR-011 through
FR-015 and for schema versions 1.0 or 1.1. It performs no repair, migration, or
best-effort interpretation. Repeated canonicalization is side-effect-free and
byte-identical. Public byte lengths use `u64`; implementations must reject a
host allocation failure rather than emit partial bytes.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-016-AC-1 | Exact golden fixtures pin the profile envelope, escaping, minimal integers, normalized rationals, semantic-set ordering, sequence preservation, source exclusion, and SHA-256 digest for every closed object kind; equivalent supported permutations are byte/digest identical. | Test (TC-017) |
| FR-016-AC-2 | A semantic change to a clause changes that clause, requirement, and package digest while unrelated clause digests remain stable; repeated runs and reversed insertion order reproduce identical bytes without host-width or map-order fields. | Test (TC-017) |
| FR-016-AC-3 | A deterministic reservation-failure harness forces canonical byte allocation failure and verifies `canonicalization_resource_exhausted`, no partial public bytes, and no digest. | Test (TC-017) |

## Dependencies

FR-011 through FR-015 define canonicalized semantic content.
