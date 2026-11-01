---
id: FR-070
title: "Own the certificate-member vocabulary in the model crate"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: traces_to
  - target: ix://agent-ix/quire-specification/FR-331
    type: depends_on
---
# FR-070: Own the certificate-member vocabulary in the model crate

## Description

When a certificate rejection identifies a malformed member, the
`quire-contract-model` crate SHALL own the target-neutral Rust vocabulary for
that member. It SHALL expose a single closed `CertificateMember` enum whose
spellings are the member names admitted by the QSpec backend-provider schema's
`CertificateRejected` `malformed` locus. The model crate SHALL NOT depend on
QSL or on a backend implementation. QSL owns its certificate-locus shape and
may use this shared type when it writes `{ "member": spelling }`; this
requirement does not move locus serialization into the model crate.

## Inputs

The authoritative external source is a `quire-specification` checkout at
commit `65d14b984108733f1988fbc48ed9fe279acfe5bd` or later, specifically
`proposals/backend-provider-v1/schema.json` and FR-331-AC-46. Its
`$defs.CertificateRejected.oneOf` branch with `rule: malformed` and
`at.properties.member.enum` supplies the published spellings. The property
closure is computed from `$defs.ProbabilityCertificate` and
`$defs.ZoneCertificate` and their certificate-owned definitions:
`CertificateIdentity`, `DbmBound`, `ComponentReason`, `RefinementPart`, and
`ProbabilityProductKey`. Shared value definitions and the nested
`TaskProductCertificate` evidence schema do not contribute member names,
as FR-331's Certificate member row specifies.

The conformance check SHALL read the checkout through a configured path and
fail clearly when that path, schema file, relevant definition, or enum is
unavailable or malformed. It SHALL NOT silently skip the check, embed a copy
of the schema, or fetch one at test time.

## Outputs

The default-feature public API SHALL include
`quire_contract_model::CertificateMember`, explicitly re-exported from the
model crate's `certificate_member` module. The enum SHALL have a variant for
each published spelling, `pub const ALL: &'static [Self]`, and
`pub fn as_str(self) -> &'static str`. `ALL` SHALL enumerate every variant
exactly once. Each variant's `as_str()` SHALL return its exact schema spelling
without a path, index, or alias. The type SHALL support `Clone`, `Copy`,
`Debug`, `Eq`, and `PartialEq`.

The model type supplies the spelling used by QSL's `{ "member": spelling }`
certificate locus. The locus remains one member name with no path. A name
outside the published set cannot be represented by this type.

## Behavior

QSpec FR-331-AC-46 and TC-431 BP-42 require **set equality** between the
published `member` enum and the distinct property names in the two
certificate schemas and their certificate-owned definitions. Neither source
specifies an order for that set. The model conformance check SHALL compute
both sets from the referenced QSpec checkout, verify their equality, and
verify that the set of `CertificateMember::ALL` spellings equals that set.
It SHALL fail when an added, removed, or renamed member makes any set differ.
Duplicate spellings or duplicate entries in `ALL` SHALL fail rather than
disappear during set conversion.

Separately, this repository SHALL retain the published schema enum sequence
in `CertificateMember::ALL`. This is an IR public API choice that preserves
the existing QSL enumeration sequence for consumers; it is **not** an
FR-331 requirement. The conformance check SHALL compare ordered spellings
against the schema enum and fail on a reorder even when set equality still
holds. Updating the model sequence in response to an intentional QSpec
reorder restores this IR check without changing the FR-331 set rule.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-070-AC-1 | A default-feature consumer imports `quire_contract_model::CertificateMember` directly, can enumerate its closed `ALL` slice and obtain the exact schema spelling of every variant with `as_str()`, and sees the specified derives. The model crate has no QSL or backend dependency. | Test, Inspection |
| FR-070-AC-2 | With a QSpec checkout at the stated commit or later, a conformance test reads the referenced schema and computes the `ProbabilityCertificate`/`ZoneCertificate` certificate-owned property-name set and the `malformed` member enum set. Both sets and the unique `CertificateMember::ALL` spelling set are equal, with exactly 51 names at the stated commit; a schema add, removal, rename, duplicate, or malformed/missing source fails the check. This is the FR-331-AC-46 and TC-431 BP-42 set obligation. | Test |
| FR-070-AC-3 | The ordered `CertificateMember::ALL` spellings equal the published schema enum sequence. A schema-only reorder fails this IR repository choice until the model enumeration is updated, even though FR-331-AC-46 set equality still holds. | Test |
| FR-070-AC-4 | For each variant, `as_str()` is the sole canonical name used in QSL's one-member `{ "member": spelling }` locus; no path, index, alias, or additional locus member is derived from the model type. QSL's locus serialization remains QSL-owned. | Test, Inspection |

## Dependencies

- QSpec FR-331-AC-46 and TC-431 BP-42 define the certificate-member set and
  its schema-derived equality; the QSpec checkout remains the conformance
  source. This requirement references it and carries no local schema copy.
- FR-019 owns the default-feature model crate public inventory. The QSL
  re-export and removal of its local enum belong to QSL-1437.
