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
  - target: ix://agent-ix/quire-contract-ir/FR-013
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-014
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-015
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

## Issue 8 Codes

| Code | Condition | Required location |
|---|---|---|
| `duplicate_type_declaration` | Two enum/record declarations share one type name | later declaration span |
| `duplicate_value_declaration` | Two input/state declarations share one value name | later declaration span |
| `duplicate_function_declaration` | Two pure functions share one name | later declaration span |
| `duplicate_field` | Two record fields share one name | later field span |
| `duplicate_variant` | Two enum variants share one name | later variant span |
| `duplicate_parameter` | Two function parameters share one name | later parameter span |
| `empty_enum` | Enum declaration has no variants | enum declaration span |
| `invalid_numeric_bounds` | Integer bounds/domain, rational denominator, or collection maximum exceeds its closed numeric range | type/literal span or path |
| `text_bound_exceeded` | Text literal contains more than 1048576 Unicode scalar values | text literal span |
| `unbounded_collection` | Collection maximum is zero or absent | type span/path |
| `collection_bound_exceeded` | Collection literal contains more items than its declared maximum | collection literal span |
| `orphaned_type_reference` | Named enum/record type does not resolve | type span/path |
| `recursive_type` | Record containment graph has a direct or indirect cycle | participating field span |
| `orphaned_value_reference` | Input/state reference does not resolve | expression span |
| `orphaned_function_reference` | Pure-function reference does not resolve | call span |
| `invalid_state_observation` | Input/state observation is not permitted by the FR-014 execution-point table | reference span |
| `invalid_scope` | Local name is absent, duplicated, or escapes its quantifier | local/quantifier span |
| `arity_mismatch` | Pure-function argument count differs from its declaration | call span |
| `ill_typed_expression` | Operand, access, argument, field, variant, or quantifier-domain type is invalid | narrowest expression span |
| `result_type_mismatch` | Checked expression type differs from the expected type | root expression span |
| `non_boolean_clause_root` | Executable clause body does not have Boolean type | clause-root span |
| `potentially_undefined` | A partial-operation obligation is not statically discharged; diagnostic includes mandatory `obligation_kind` (`option_presence`, `non_zero_divisor`, `index_in_bounds`, or `checked_range`) | partial-operation span |
| `expression_too_large` | Expression exceeds 10000 nodes or depth 256 | first node crossing the limit |

## Issue 9 Codes

| Code | Condition | Required location |
|---|---|---|
| `unsupported_schema_version` | Wire preflight reads a nonzero schema major other than 1 | `schema_version.major` path; no semantic span |
| `unregistered_migration` | Major 1 minor is not 0/1, or a requested migration edge is not the registered 1.0-to-1.1 edge | `schema_version` or migration request path |
| `canonicalization_resource_exhausted` | Canonical byte allocation cannot be reserved without exceeding host resources | canonicalized object path; source span when the object has one |
| `duplicate_artifact_trace` | A later artifact trace repeats an artifact ID in one classification input | later trace span |
| `stale_trace_digest` | A deep trace's requirement digest differs from the resolved current requirement digest | digest-token span |

## Issue 10 Codes

| Code | Condition | Required location |
|---|---|---|
| `semantic_input_too_large` | A complete operation exceeds 10000 semantic nodes, recursive semantic depth 256, or 10000 entries in any semantic collection | first node, depth, or collection path crossing the limit; source span when present |

## Application Guidance

Public diagnostics contain a code, closed severity `error`, message, semantic
path, optional source span, related identities, and optional
`obligation_kind`. The obligation field is present if and only if the code is
`potentially_undefined` and uses the four-value closed enum. New failure classes
require a registry row, an owning requirement criterion, and positive or
negative test evidence before implementation claims coverage.

Diagnostic precedence is identity grammar, then source/span structure, then
package ownership, then exact-revision resolution, then target existence, and
finally clause/anchor compatibility. One condition emits its highest-precedence
primary code; additional context may appear only as related identities or
secondary diagnostics. Thus an empty clause ID is `invalid_identifier`, not
`malformed_reference`, and an empty package namespace is
`invalid_package_namespace`, not `cross_package_reference`.

Issue #8 precedence is declaration/identifier grammar and numeric/collection
bounds, then duplicates, named-type resolution, containment cycles, local/value/
function name resolution, call arity, operand/access typing, expected-result
typing, and definedness last. At an executable clause root,
`non_boolean_clause_root` takes precedence over `result_type_mismatch`. Local
lookup precedes value lookup; an absent syntactic local is `invalid_scope`, an
absent declared value is `orphaned_value_reference`, and an absent call target
is `orphaned_function_reference`. Arity precedes argument typing. An ill-typed
partial node is `ill_typed_expression`, never `potentially_undefined`.

Expression diagnostics are emitted in authored pre-order: one primary
diagnostic per node, siblings in stored order. A failed child suppresses its
parent's typing/definedness diagnostic, while independent siblings continue.
Within one node the issue #8 precedence above selects the primary code. State
observation policy follows resolved-value lookup and precedes operand typing.
Declaration-environment diagnostics precede expression diagnostics and follow
stored declaration/field/variant/parameter order.

Issue #9 wire precedence is JSON/top-level structure, schema-version numeric
grammar, unsupported major, unregistered minor/migration edge, then semantic
package interpretation. Canonicalization accepts validated values only and
performs no diagnostic recovery; resource exhaustion produces no partial bytes
or digest. Coverage precedence per trace is duplicate artifact ID,
cross-package target, missing requirement, stale revision, then deep-digest
mismatch. Coverage diagnostics retain authored trace order even though report
rows sort structurally.

## Dependencies

- **Upstream**: PGM-01 evidence and human-decision boundaries.
- **Downstream**: FR-013 through FR-019 extend or consume this semantic registry
  without renaming issue #6 codes. FR-020 defines separate
  runner operational codes that are neither `DiagnosticCode` values nor
  semantic diagnostic shapes.
