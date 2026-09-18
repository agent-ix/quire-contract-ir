---
id: STD-003
title: "Output-mapping refusal code registry"
type: Standard
code: contract-ir-output-mapping-refusals-v0.1
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-032
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-033
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-034
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: references
---
# STD-003: Output-mapping refusal code registry

## Description

This registry owns the stable machine-readable refusal codes the output-mapping
surface emits: the complete closed `MappingRequestErrorCode` catalog reached
through `MappingRequestError`. These codes are a separate catalog from the
semantic `DiagnosticCode` values registered by
[STD-001](./STD-001-diagnostic-registry.md); no spelling is shared between the
two and neither is convertible into the other.

[FR-019](../interface/FR-019-rust-library-interface.md) forbids a caller
recovering an outcome from display, debug, or panic text, so the code spelling
registered here — not the message — is the refusal contract. Implementations may
add human context but shall not parse or synthesize codes from messages, and
shall not emit a code absent from this registry. Codes are lowercase ASCII snake
case and are stable v0.1 API.

Every row names the structural field path the refusal is required to carry.
Where a code is reachable from more than one accounting stage, every path it may
carry is listed and no other path is permitted.

## Request Admission Codes

Emitted by [FR-032](./FR-032-admit-output-mapping-request.md) before any mapper
is dispatched and before any target byte exists.

| Code | Condition | Required location |
|---|---|---|
| `invalid_digest` | A supplied raw digest is not a lowercase 64-character SHA-256 hexadecimal string | `digest` |
| `invalid_source_selection` | A native, model, or semantic selection identity or revision is empty, unbounded, or outside visible ASCII | `selection` |
| `unknown_target_family` | The requested target family is outside the fixed FS06 family set | `profile.target_family` |
| `missing_target_standard` | The profile carries no ordered target-standard reference | `profile.target_standard_refs` |
| `target_profile_mismatch` | The mapping profile does not pair with the requested target family, or a dispatched mapper reports a different profile | `profile`; `mapper.profile` when the mismatch is the mapper's |
| `stale_mapping_revision` | The mapping revision is not the accepted `1-draft.1` | `profile.mapping_revision` |
| `unsupported_capability` | A requested capability is outside the profile's closed capability set | `profile.required_capabilities` |
| `duplicate_capability` | The requested capability set repeats a capability | `profile.required_capabilities` |
| `zero_limit` | A declared aggregate limit is zero | `limits` |
| `empty_obligation_selection` | The obligation selection is empty | `request.obligations` |
| `duplicate_obligation` | The obligation selection repeats an obligation identity | `request.obligations` |
| `informational_obligation` | A selected obligation resolves to an informational clause rather than an executable one | `request.obligations` |
| `foreign_obligation` | The obligation names a package other than the bound package | `request.obligations` |
| `stale_obligation` | The obligation names a requirement present in the bound package at a different revision | `request.obligations` |
| `unknown_obligation` | The obligation resolves to no executable clause and is neither foreign nor stale | `request.obligations` |
| `obligation_order_mismatch` | The retained obligation order differs from the admitted order | `request.obligations` |
| `invalid_qualified_reference` | A qualified owner, identity, revision, condition, or cause code is empty, unbounded, or outside visible ASCII | the offending qualified-reference path |

### Unresolved-obligation precedence

An obligation that resolves to no executable clause is classified by exactly one
code, selected in this order and never pooled:

1. `foreign_obligation` — the identity names a different package.
2. `stale_obligation` — the package holds that requirement id at a different
   revision.
3. `unknown_obligation` — neither of the above holds.

The order is total: an identity that is both foreign and would be stale in
another package is `foreign_obligation`, because package identity is checked
before any revision in the bound package is read.

## Accounting and Mapper-Seam Codes

Emitted by [FR-033](./FR-033-account-for-output-obligations.md) while charging a
dispatched mapper's candidates against the admitted limits.

| Code | Condition | Required location |
|---|---|---|
| `request_limit_exceeded` | The admitted request exceeds its declared request-byte limit | `request` |
| `obligation_limit_exceeded` | The obligation count exceeds its declared limit | `request.obligations` |
| `expression_node_limit_exceeded` | The accounted expression-node count exceeds its declared limit | `request.expression_nodes` |
| `nesting_depth_limit_exceeded` | The accounted nesting depth exceeds its declared limit | `request.nesting_depth` |
| `mapping_work_limit_exceeded` | Aggregate mapping work exceeds its declared limit | `mapping.work`; `request.obligations` when the charge is per obligation |
| `record_limit_exceeded` | The record count exceeds its declared limit | `request.obligations` |
| `emitted_bytes_limit_exceeded` | Accounted emitted bytes exceed the declared limit | `mapping.emitted_bytes` |
| `arithmetic_overflow` | An accounting counter or region endpoint overflows its width | the overflowing counter or region path |
| `allocation_failed` | A deterministic allocation, serialization, or canonicalization step for an accounted structure failed | the structure's path |
| `cancelled` | The caller cancelled the mapping before it completed | `output mapping was cancelled` |
| `zero_mapping_work` | A mapper candidate accounts for zero work | `candidate.work` |
| `duplicate_dependency` | A candidate repeats a dependency identity | `candidate.dependencies` |
| `invalid_disposition` | A candidate's disposition, source state, output fragment, conditions, and causes disagree | `candidate.disposition` |
| `candidate_obligation_mismatch` | A candidate names an obligation other than the one dispatched | `candidate.obligation` |
| `candidate_source_state_mismatch` | A candidate reports a source-fact state other than the retained one | `candidate.source_state` |
| `mapper_failed` | The dispatched mapper returned its own failure | `mapper` |

## Package Assembly Codes

Emitted by [FR-034](./FR-034-assemble-output-package-atomically.md) while
assembling the single atomic output package.

| Code | Condition | Required location |
|---|---|---|
| `invalid_output_region` | An output region is reversed, out of range, not on a character boundary, or its bytes are not UTF-8 | the region or fragment path |
| `invalid_generator` | The declared generator identity is empty, unbounded, or outside visible ASCII | `generator` |
| `invalid_observer` | The declared observer identity is empty, unbounded, or outside visible ASCII | `observer` |
| `package_population_mismatch` | The assembled record population or target-byte length disagrees with the accounted counts | `package.records`; `package.target_bytes` |

## Registry Invariants

- The catalog is closed: `MappingRequestErrorCode::ALL` and the rows of this
  registry are the same set, in the same order, with no spelling in one and not
  the other.
- A refusal carries exactly one code. A condition that satisfies two rows is
  resolved by the precedence stated for that group, never by emitting both.
- No code in this registry is a `DiagnosticCode` spelling, and no
  `DiagnosticCode` spelling appears here.
- A refusal never accompanies a partial package, partial target bytes, or a
  partially admitted request.

## Dependencies

- **Upstream**: [FR-019](../interface/FR-019-rust-library-interface.md) forbids
  message parsing and makes the code spelling the contract.
- **Downstream**: [FR-032](./FR-032-admit-output-mapping-request.md),
  [FR-033](./FR-033-account-for-output-obligations.md) and
  [FR-034](./FR-034-assemble-output-package-atomically.md) emit only codes
  registered here.
