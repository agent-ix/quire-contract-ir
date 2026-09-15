---
id: FR-034
title: "Assemble one output package atomically"
type: FR
relationships:
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
- One exact Rust generator identity, semantic version, immutable revision, and
  raw executable/source digest.
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
  target profile, generator, target raw digest, ordered record identities, and
  limits in the package identity preimage.
- The assembler shall exclude paths, timestamps, locale, display diagnostics,
  allocation layout, and structural-observer output from target bytes and
  semantic identity.
- The assembler shall expose no package after cancellation, mapping failure,
  arithmetic overflow, allocation failure, resource exhaustion, or malformed
  region/record input.
- A structural observation reference shall name only an already immutable
  package and exact observer/tool result and shall not mutate package bytes,
  records, dispositions, or identities.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-034-AC-1 | Equal admitted requests, mapper candidates, and generator identities produce byte-identical target output, records, raw digests, and package identities. | Test (TC-043) |
| FR-034-AC-2 | Missing, duplicate, foreign, stale, reordered, cross-profile, malformed-region, digest-mismatched, or over-limit inputs refuse atomically with no package. | Test (TC-043) |
| FR-034-AC-3 | Every zero, exact, just-over, and overflowing aggregate limit is classified without partial output or a smaller successful package. | Test (TC-043) |
| FR-034-AC-4 | Mutating source/profile/generator/record/limit/target-byte identity inputs changes or invalidates package identity, while path/time/locale/display/observer changes do not. | Test (TC-043) |
| FR-034-AC-5 | Structural observer acceptance, refusal, absence, version, and rights state remain downstream references and cannot establish native truth or mapping preservation. | Test (TC-043) |

## Dependencies

- [FR-032](FR-032-admit-output-mapping-request.md) fixes request order, profile,
  source identity, and limits.
- [FR-033](FR-033-account-for-output-obligations.md) fixes record completeness and
  disposition invariants.
- `ix://agent-ix/quire-specification/FR-125`, `FR-299`, `NFR-060`, and `NFR-061`
  own deterministic package assembly, aggregate bounds, dependency containment,
  and observer separation.
