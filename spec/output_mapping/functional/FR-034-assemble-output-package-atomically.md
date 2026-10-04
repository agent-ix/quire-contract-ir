---
id: FR-034
title: "Assemble one output package atomically"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-032
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-033
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-125
    type: implements
  - target: ix://agent-ix/quire-specification/FR-299
    type: implements
  - target: ix://agent-ix/quire-specification/NFR-060
    type: constrained_by
  - target: ix://agent-ix/quire-specification/NFR-061
    type: constrained_by
---
# FR-034: Assemble one output package atomically

## Description

When every requested obligation has one valid mapping record, the Contract IR
assembler shall expose one immutable generated-output package only after target
bytes, absolute regions, raw digest, generator identity, limits, record order,
and the complete `sha256-jcs` package identity preimage have been verified.

## Inputs

- One admitted [FR-032](FR-032-admit-output-mapping-request.md) request.
- One ordered candidate/record population satisfying
  [FR-033](FR-033-account-for-output-obligations.md).
- One Rust generator identity: the owner that produces the package.
- One cancellation state and the admitted aggregate limits.

## Outputs

- One immutable package containing deterministic target bytes, raw target digest,
  ordered records with absolute regions, exact limits, and derived package identity.
- One typed failure with no partial package, bytes, record population, or identity.

## Behavior

- The assembler shall concatenate represented target fragments in admitted
  obligation order and shall derive absolute record regions with checked arithmetic.
- The assembler shall verify that every region is half-open, within its fragment
  and final target bytes, and aligned to UTF-8 boundaries when the selected
  profile requires UTF-8.
- The assembler shall verify exact one-record-per-requested-obligation
  completeness, record order, target-profile equality, raw target digest,
  generator identity, and admitted limits before constructing package identity.
- The assembler shall include the identity version, source-package reference,
  target profile, generator owner, target raw digest,
  ordered record identities, and
  limits in the package identity preimage.
- Every canonical identity this pipeline computes (the request's, each
  record's and the package's) shall be encoded by `quire-canonical` under the
  request's own byte limit, `maximum_request_bytes`. The request identity is
  those canonical bytes: admission keeps their length (`request_bytes`) and
  computes no digest of them. Each record identity and the package identity is
  the SHA-256 digest of its canonical bytes.
- Identity material whose canonical bytes exceed that ceiling shall refuse
  `request_limit_exceeded` at `request`, `record.identity` or `package.identity`
  and yield no record, identity or package; material of exactly the ceiling
  encodes. An allocation failure stays `allocation_failed` at the same paths.
- `maximum_request_bytes` therefore bounds more than the request: it also bounds
  the canonical material of every record, which is built from mapper-supplied
  candidate material, and of the package, so no identity step runs without a
  ceiling (FR-033 states the record rule).
- The identity material shall spell every `u64` as a decimal string of its
  minimal base-ten digits (QSpec `IntegerString`, as FR-016 does for the model's
  integers): the members of `MappingLimits` and the `start` and `end` of an
  `OutputByteRegion`. A `MappingLimits` member is never zero (FR-032-AC-2
  refuses it), so only a nonzero limit reaches the material; a region `start`
  of zero is admitted and is spelled `"0"`.
- Every other integer of the identity material (a requirement revision, a source
  revision, a byte offset, a line, a column, a schema version) shall be a JSON
  number, which FR-011 and FR-012 hold at or below 2^53, so no encode of
  identity material refuses for an integer. RFC 8785 spells no integer past 2^53
  exactly, so a number for a `u64` limit or region would refuse a legitimate
  value as an artificial cap.
- The assembler shall exclude paths, timestamps, locale, display diagnostics,
  allocation layout, and structural-observer output from target bytes and
  semantic identity.
- The assembler shall expose no package after cancellation, mapping failure,
  arithmetic overflow, allocation failure, resource exhaustion, or malformed
  region/record input.
- A structural observation reference shall name only an already immutable
  package, the observer owner, and the accepted or refused outcome, and shall
  not mutate package bytes, records, dispositions, or identities.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-034-AC-1 | Equal admitted requests, mapper candidates, and generator identities produce byte-identical target output, records, raw digests, and package identities. | Test (TC-043) |
| FR-034-AC-2 | Missing, duplicate, foreign, stale, reordered, cross-profile, malformed-region, digest-mismatched, or over-limit inputs refuse atomically with no package. | Test (TC-043) |
| FR-034-AC-3 | Every zero, exact, just-over, and overflowing aggregate limit is classified without partial output or a smaller successful package. | Test (TC-043) |
| FR-034-AC-4 | Mutating source/profile/generator owner/record/limit/target-byte identity inputs changes or invalidates package identity, while path/time/locale/display/observer changes do not. | Test (TC-043) |
| FR-034-AC-5 | Structural observer acceptance, refusal, absence, and observer owner remain downstream references and cannot establish native truth or mapping preservation. | Test (TC-043) |
| FR-034-AC-6 | Identity material is canonicalized under `maximum_request_bytes` and never without a ceiling: through the public admission and mapping calls, request material whose canonical length equals the limit admits and one byte over refuses `request_limit_exceeded` at `request`, and a record whose material is one byte over refuses at `record.identity`, each with no record, identity or package and not `allocation_failed`; and the package identity step, called with a ceiling equal to the measured length of its material, encodes, and with a ceiling one byte lower refuses `request_limit_exceeded` at `package.identity` with no package. | Test (TC-043) |
| FR-034-AC-7 | The record and package identities are the SHA-256 digests of an expected canonical byte string written out in the test for a fixed record and package, and the request identity material, whose length admission keeps as `request_bytes` and whose digest admission does not compute, is the expected canonical byte string written out in the test for a fixed request (members in UTF-16 order), none of them computed by a call into the code under test; the `MappingLimits` members `18446744073709551615` (`maximum_emitted_bytes` of the request material) and `9007199254740993` (`maximum_emitted_bytes` of the package material), and the `start` and `end` of an `OutputByteRegion`, including `0`, `1`, `9007199254740993` and `18446744073709551615`, appear in the material as the strings `"18446744073709551615"`, `"9007199254740993"`, `"0"`, `"1"`, `"9007199254740993"` and `"18446744073709551615"`; and a request whose `maximum_emitted_bytes` is `18446744073709551615` and one whose is `18446744073709551614` both admit and have distinct request identity material. | Test (TC-043) |

## Dependencies

- [FR-032](FR-032-admit-output-mapping-request.md) fixes request order, profile,
  source identity, and limits.
- [FR-033](FR-033-account-for-output-obligations.md) fixes record completeness and
  disposition invariants.
- `ix://agent-ix/quire-specification/FR-125`, `FR-299`, `NFR-060`, and `NFR-061`
  own deterministic package assembly, aggregate bounds, dependency containment,
  and observer separation.
