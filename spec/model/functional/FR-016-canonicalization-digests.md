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

A validated package and the canonicalization profile identity.

## Outputs

Canonical UTF-8 JSON bytes and lowercase SHA-256 digests for every addressable
semantic object.

## Behavior

The only profile identity is
`quire.contract.canonical-json/v1`. A canonical object is the UTF-8 encoding of
an envelope whose members, after key sorting, are emitted in the
exact order `kind`, `profile`, `value`:
`{"kind":<kind>,"profile":<profile>,"value":<semantic-value>}`. Object member
names always sort in RFC 8785 section 3.2.3 order, by UTF-16 code unit; authored
member order is never an exception. Every member name of a canonical object is
an ASCII schema name, so that order is the byte order and the Unicode-scalar
order the profile used before, and the order of every member is unchanged.
Arrays preserve semantic order. JSON
strings emit `\"`, `\\`, `\b`, `\t`, `\n`, `\f`, and `\r`; other U+0000 through
U+001F controls use lowercase four-digit `\u00xx`; every other Unicode scalar
is emitted directly as UTF-8. `/` and non-ASCII scalars are not escaped.
Boolean values are lowercase.

The profile spells every integer the model bounds by `i128` as a JSON
string of its minimal base-ten digits, the `IntegerString` spelling of QSpec
V2 (`^(0|-?[1-9][0-9]*)$`: `0` for zero, no `+`, no leading zero, `-` only
before a nonzero digit). These are the eight members of the profile:
`minimum` and `maximum` of an integer type, `value` of an integer literal,
`numerator_minimum`, `numerator_maximum` and `maximum_denominator` of a
rational type, and `numerator` and `denominator` of a rational literal. An
integer type bounded by `i128::MIN` and `i128::MAX` is written
`"minimum":"-170141183460469231731687303715884105728"` and
`"maximum":"170141183460469231731687303715884105727"`, and a
rational type whose `maximum_denominator` is `i128::MAX` writes
`"maximum_denominator":"170141183460469231731687303715884105727"`. RFC 8785 has no exact number
past 2^53 and `quire-canonical` encodes no integer of larger magnitude, so these
members are strings and no encoder stringifies a number on the caller's behalf.
Every other integer member of a canonical object (the schema version's `major`
and `minor`, a collection's `maximum_items` and a requirement `revision`) is a
JSON number in RFC 8785's spelling, and its type or constructor holds it at or
below 2^53: FR-011 and FR-012 refuse a requirement revision, a source revision
or a byte offset above 2^53. No floating-point or null value exists in the
canonical model: the semantic value is a set of typed structures, none of which
has a member that holds a `null`, a floating-point number or a
`serde_json::Value`, so such a value cannot be constructed rather than being
checked for and refused.

The bytes are `quire-canonical`'s. This crate carries no canonical writer, no
digest helper that serializes through `serde_json` and no second encoder of its
own: each typed structure is encoded by `quire_canonical::Encode`, and the
structures whose nesting follows their input (value types and expressions) do
not take the fixed-depth `FixedShape` path (FR-038, "Canonical encoding of the
wire types", states the two paths). This changes the public v1 wire and the
canonical bytes and digest of every v1 canonical object that holds one of the
eight integer members, once; an object that holds none of them keeps its bytes
and digest (47 of the 75 canonical files and 59 of the 99 inputs of the
published corpus hold one, at the time of writing). Every recorded canonical
fixture and digest of the changed objects is regenerated and no earlier
spelling is read. The v1 depth limits `MAX_WIRE_JSON_DEPTH` and `MAX_SEMANTIC_DEPTH`
(FR-019) are unchanged.

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
complete requirements, so the schema version is part of the package digest
input.

Every value type and expression variant uses its registered snake-case tag and
all fields that affect type checking or execution. Rational literals use their
normalized numerator and positive denominator. A typed expression canonicalizes
only after successful validation and includes its explicit result type plus the
normalized expression tree. Declaration-environment owner identity is included;
the three declaration namespaces use the ordering above.

The lowercase digest for kind `K` is SHA-256 over the exact byte sequence
`quire-contract-ir`, one zero byte, the profile identity, one zero byte, `K`,
one zero byte, then the canonical object bytes. This domain separation is part
of the profile. The prefix is fed to SHA-256 explicitly, followed by the bytes
`quire-canonical` produced; `quire_canonical::sha256_with_domain` is not used,
because it hashes the label behind a big-endian length prefix, a different byte
sequence that would change every digest. Package canonical bytes contain complete sorted requirements
and clauses rather than host map iteration or child digest placeholders.
Requirement and clause digests use the same independently canonicalized
objects, so changing one clause changes that clause, its enclosing requirement,
and its package while unrelated clause digests remain unchanged.

Canonicalization is defined only for values already accepted by FR-011 through
FR-015 and for schema version 1.1. A package constructed in memory with any
other version fails `unsupported_schema_version` at `schema_version` from every
canonical API and produces no bytes or digest. It performs no repair, migration, or
best-effort interpretation. Repeated canonicalization is side-effect-free and
byte-identical. Public byte lengths use `u64`; implementations must reject a
host allocation failure rather than emit partial bytes.

Every canonical operation shall accept an explicit caller-supplied maximum byte
budget. The crate exposes both forms for each of the five closed object kinds —
package, requirement, clause, declaration and expression — as
`canonical_<kind>` and `canonical_<kind>_with_limit`. The crate shall implement
each budget-free form as its budgeted form at `u64::MAX` rather than as an
unbounded path of its own. A budget the canonical bytes exceed returns
`canonicalization_resource_exhausted` with no partial bytes and no digest,
identically to an allocation failure; the budget is passed to `quire-canonical`
as its byte limit and the limit refusal it returns is that code. The budget is a
caller policy: it bounds output size and never changes the bytes a successful
call produces.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-016-AC-1 | Exact golden fixtures, regenerated from `quire-canonical`'s own output, pin the profile envelope, escaping, the string spelling of the eight integer members (FR-016-AC-5) and the number spelling of the others, normalized rationals, semantic-set ordering, sequence preservation, source exclusion, and SHA-256 digest for every closed object kind; equivalent supported permutations are byte/digest identical. | Test (TC-017) |
| FR-016-AC-2 | A semantic change to a clause changes that clause, requirement, and package digest while unrelated clause digests remain stable; repeated runs and reversed insertion order reproduce identical bytes without host-width or map-order fields. | Test (TC-017) |
| FR-016-AC-3 | A deterministic reservation-failure harness forces canonical byte allocation failure and verifies `canonicalization_resource_exhausted`, no partial public bytes, and no digest. | Test (TC-017) |
| FR-016-AC-4 | Each of the five object kinds exposes a budgeted and a budget-free canonical operation; the budget-free result equals the budgeted result at `u64::MAX`; and a zero budget returns `canonicalization_resource_exhausted` with no bytes and no digest for every kind. | Test (TC-017) |
| FR-016-AC-5 | The eight integer members are JSON strings of minimal base-ten digits in every canonical object: an integer type with minimum `-9223372036854775808` and maximum `9223372036854775807` canonicalizes to bytes holding `"minimum":"-9223372036854775808"` and `"maximum":"9223372036854775807"`, and one with minimum `0` and maximum `0` holds `"minimum":"0"` and `"maximum":"0"`; a rational type with `maximum_denominator` `9223372036854775807` holds `"maximum_denominator":"9223372036854775807"` and numerator bounds `-9223372036854775808` and `9223372036854775807` hold the matching strings; an integer literal `-1` holds `"value":"-1"` and a rational literal `-3/4` holds `"numerator":"-3"` and `"denominator":"4"`; no byte sequence of the form `"<one of the eight names>":` is followed by a digit or `-` in any recorded canonical fixture; and the `maximum_items` and `revision` members of the same objects are JSON numbers. | Test (TC-017) |
| FR-016-AC-6 | Every canonical byte sequence and digest of this crate comes from `quire-canonical`: for one fixture of each of the five kinds the bytes returned equal an expected byte string written out in the test member by member from RFC 8785 (members in UTF-16 order, minimal escapes, the eight integer members as strings), not computed by a call into this crate or by a second call into the encoder, and the digest equals the SHA-256 of the domain prefix and those expected bytes; a byte limit of the exact length returns the bytes and one byte lower returns `canonicalization_resource_exhausted`; and the crate source holds no `CanonicalWriter`, `canonical_envelope_bytes`, `digest_json` or `serde_json_canonicalizer` symbol and no `serde_json::to_vec` or `serde_json::to_value` call in `canonical.rs`, `binding.rs` or `output_mapping.rs`, checked by a test that reads those files and the manifests and counts the symbols, which finds none. | Test (TC-017) |
| FR-016-AC-9 | Canonical bytes spell integer type bounds and literal values at `i64::MAX + 1`, `u64::MAX`, `i128::MIN` and `i128::MAX`, rational numerator and denominator members at their permitted i128 endpoints, as minimal JSON strings; the same semantic value previously representable within i64 yields byte-identical output, and changing only one wide value changes the canonical bytes and digest. No node-key preimage counter or other number-field encoding changes. | Test (TC-017) |
| FR-016-AC-7 | The semantic value has no member that can hold a `null`, a floating-point number or a `serde_json::Value`: the canonical types (`ValueType`, `IntegerType`, `RationalType`, `CollectionType`, `Expression`, `ExpressionKind`, `RecordLiteralField`, `TypedExpression`, `DeclarationEnvironment` and the declaration, requirement and clause projections in `canonical.rs`) declare no `f32`, `f64` or `serde_json::Value` member and no `Option` member that serializes as `null` (an absent optional member is omitted), checked by a test that reads their source and finds none; and a requirement revision of 9007199254740992 canonicalizes to the number `9007199254740992` (the refusal above 2^53 is FR-011-AC-3's and FR-012-AC-6's). | Test (TC-017) |
| FR-016-AC-8 | The digest for kind `K` equals SHA-256 over `quire-contract-ir`, a zero byte, the profile identity, a zero byte, `K`, a zero byte and the bytes `quire_canonical::to_vec` returns for the envelope, and differs from `quire_canonical::sha256_with_domain` over the same envelope with the same label. | Test (TC-017) |

## Dependencies

FR-011 through FR-015 define canonicalized semantic content. FR-013 owns the
wire spelling of the eight integer members, and FR-038 ("Canonical encoding of
the wire types") owns which `quire-canonical` encode path a type takes.
`quire-canonical` is the one RFC 8785 encoder.
