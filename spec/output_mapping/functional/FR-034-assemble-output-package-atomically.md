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
bytes, absolute regions, generator owner name, limits, record order,
and one-record-per-obligation completeness have been verified.

## Inputs

- One admitted [FR-032](FR-032-admit-output-mapping-request.md) request.
- One ordered candidate/record population satisfying
  [FR-033](FR-033-account-for-output-obligations.md).
- One Rust generator identity: the owner that produces the package.
- One cancellation state and the admitted aggregate limits.

## Outputs

- One immutable package containing deterministic target bytes, ordered records
  with absolute regions, exact limits, and the generator owner name.
- One typed failure with no partial package, bytes, or record population.

## Behavior

- The assembler shall concatenate represented target fragments in admitted
  obligation order and shall derive absolute record regions with checked arithmetic.
- The assembler shall verify that every region is half-open, within its fragment
  and final target bytes, and aligned to UTF-8 boundaries when the selected
  profile requires UTF-8.
- The assembler shall verify exact one-record-per-requested-obligation
  completeness, record order, target-profile equality, generator owner name,
  and admitted limits before exposing the package.
- The package shall retain the source-package name and schema version, target
  profile, generator owner name, ordered records, limits and exact target
  bytes. It carries no content digest or derived package identity.
- Request, record and package material used for byte-budget checks shall be
  encoded by `quire-canonical` under the
  request's own byte limit, `maximum_request_bytes`.
- The admission step shall encode the request material only to meter
  it and keep its length as `request_bytes`, without retaining those bytes.
- Material whose canonical bytes exceed that ceiling shall refuse
  `request_limit_exceeded` at `request`, `record` or `package`
  and yield no record or package; material of exactly the ceiling
  encodes. An allocation failure stays `allocation_failed` at the same paths.
- `maximum_request_bytes` therefore bounds more than the request: it also bounds
  the canonical material of every record, which is built from mapper-supplied
  candidate material, and of the package, so no bounded encoding step runs without a
  ceiling (FR-033 states the record rule).
- The canonical material shall spell every `u64` as a decimal string of its
  minimal base-ten digits (QSpec `IntegerString`, as FR-016 does for the model's
  integers): the members of `MappingLimits` and the `start` and `end` of an
  `OutputByteRegion`. A `MappingLimits` member is never zero (FR-032-AC-2
  refuses it), so only a nonzero limit reaches the material; a region `start`
  of zero is admitted and is spelled `"0"`.
- Every other integer of the canonical material (a requirement revision, a source
  revision, a byte offset, a line, a column, a schema version) shall be a JSON
  number, which FR-011 and FR-012 hold at or below 2^53, so no encode of
  canonical material refuses for an integer. RFC 8785 spells no integer past 2^53
  exactly, so a number for a `u64` limit or region would refuse a legitimate
  value as an artificial cap.
- The assembler shall exclude paths, timestamps, locale, display diagnostics,
  allocation layout, and structural-observer output from target bytes and
  deterministic target bytes or record content.
- The assembler shall expose no package after cancellation, mapping failure,
  arithmetic overflow, allocation failure, resource exhaustion, or malformed
  region/record input.
- A structural observation reference shall name only an already immutable
  package, the observer owner, and the accepted or refused outcome, and shall
  not mutate package bytes, records, or dispositions.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-034-AC-1 | Equal admitted requests, mapper candidates, and generator owner names produce byte-identical target output and equal ordered typed records, with no target, record or package digest. | Test (TC-043) |
| FR-034-AC-2 | Missing, duplicate, foreign, stale, reordered, cross-profile, malformed-region or over-limit inputs refuse atomically with no package; the assembler does not add a digest-mismatch refusal. | Test (TC-043) |
| FR-034-AC-3 | Every zero, exact, just-over, and overflowing aggregate limit is classified without partial output or a smaller successful package. | Test (TC-043) |
| FR-034-AC-4 | Mutating a source name or version, target profile, generator owner name, record, limit or target byte changes the corresponding exposed typed value or output byte, or refuses admission; path, time, locale, display and observer changes do not alter the package content. | Test (TC-043) |
| FR-034-AC-5 | Structural observer acceptance, refusal, absence, and observer owner remain downstream references and cannot establish native truth or mapping preservation. | Test (TC-043) |
| FR-034-AC-6 | Request, record and package material is canonicalized under `maximum_request_bytes` solely for byte-budget enforcement. Material whose length equals the ceiling admits, and one byte over refuses `request_limit_exceeded` at `request`, `record` or `package` respectively, with no partial record or package and not `allocation_failed`; the bounded encoder receives the caller ceiling and never `u64::MAX`. No encoded material is used as a content identity. | Test (TC-043) |
| FR-034-AC-7 | The request, record and package canonical material each equals an expected byte string written out in the test from its members, without a call into the code under test; the request length retained as `request_bytes` equals the written request text length. Distinct `MappingLimits` members use values 40, 1100, 130, 4200 and 36 so a copied or swapped member is caught by exact byte equality. No digest or content-identity field is generated. | Test (TC-043) |
| FR-034-AC-8 | In the written-out request, record and package material each `MappingLimits` member and each `OutputByteRegion` bound is the decimal string of its value under its own member name. The request control uses `maximum_obligations` 40, `maximum_expression_nodes` 1100, `maximum_nesting_depth` 130, `maximum_mapping_work` 4200, `maximum_records` 36 and `maximum_emitted_bytes` 18446744073709551615. The package control uses `maximum_emitted_bytes` 9007199254740993 and the same other limits; the record controls use region bounds 0/1 and 9007199254740993/18446744073709551615. Each spelling matches its decimal string exactly, with no numeric coercion or member swap. | Test (TC-043) |
| FR-034-AC-9 | Requests whose `maximum_emitted_bytes` are `18446744073709551615` and `18446744073709551614` both admit; their canonical request material differs in exactly that member while all other members stay equal. | Test (TC-043) |

## Dependencies

- [FR-032](FR-032-admit-output-mapping-request.md) fixes request order, profile,
  source identity, and limits.
- [FR-033](FR-033-account-for-output-obligations.md) fixes record completeness and
  disposition invariants.
- `ix://agent-ix/quire-specification/FR-125`, `FR-299`, `NFR-060`, and `NFR-061`
  own deterministic package assembly, aggregate bounds, dependency containment,
  and observer separation. Their current record/package content-identity clauses
  require a QSpec revision before this digest-free package contract can be
  implemented.
