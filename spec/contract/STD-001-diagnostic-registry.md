---
id: STD-001
title: "Contract IR v0.1 diagnostic code registry"
type: Standard
code: contract-ir-diagnostics-v0.1
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-011
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-012
    type: references
---
# STD-001: Contract IR v0.1 diagnostic code registry

## Description

This registry owns the stable machine-readable diagnostic codes introduced by
issues #6 through #10. Implementations may add human context but shall not parse
or synthesize codes from messages. Codes are lowercase ASCII snake case.

## Issue 6 Codes

| Code | Condition | Required location |
|---|---|---|
| `invalid_package_namespace` | Empty or malformed package namespace | package identity path |
| `invalid_wire_format` | JSON syntax or wire shape cannot be decoded | document path |
| `invalid_schema_version` | Zero schema major; schema minor zero is valid | schema-version path |
| `invalid_identifier` | Empty or malformed source-document, requirement, clause, anchor, or dependency-path-segment identifier | offending identity path |
| `invalid_requirement_revision` | Zero or non-increasing requirement revision | requirement revision path |
| `invalid_source_revision` | Zero source-document revision | source revision path |
| `duplicate_requirement` | Two current requirements share one ID | later requirement path and earlier related identity |
| `duplicate_clause` | Two clauses in one requirement share one ID | later clause span and earlier related identity |
| `cross_package_reference` | A reference names a different package | reference span/path |
| `invalid_source_span` | Source endpoints are zero-based-invalid, reversed, or name a different source | source span |
| `floating_executable_clause` | Executable clause has no anchor | clause span |
| `informational_clause_anchored` | Informational clause has an anchor | clause span |
| `incompatible_clause_anchor` | Clause and anchor kinds violate the closed compatibility table | clause span |
| `malformed_reference` | Reference identity is structurally malformed | reference span/path |
| `stale_requirement_revision` | Requirement exists but the exact revision differs | reference span/path and current related identity |
| `orphaned_requirement_reference` | Referenced requirement ID is absent | reference span/path |
| `orphaned_clause_reference` | Requirement revision resolves but its clause ID is absent | reference span/path |

## Application Guidance

Public diagnostics contain a code, severity, message, semantic path, optional
source span, and related identities. New failure classes require a registry row,
an owning requirement criterion, and positive or negative test evidence before
implementation claims coverage.

Diagnostic precedence is identity grammar, then source/span structure, then
package ownership, then exact-revision resolution, then target existence, and
finally clause/anchor compatibility. One condition emits its highest-precedence
primary code; additional context may appear only as related identities or
secondary diagnostics. Thus an empty clause ID is `invalid_identifier`, not
`malformed_reference`, and an empty package namespace is
`invalid_package_namespace`, not `cross_package_reference`.

## Dependencies

- **Upstream**: PGM-01 evidence and human-decision boundaries.
- **Downstream**: FR-013 through FR-020 extend this registry without renaming issue #6 codes.
