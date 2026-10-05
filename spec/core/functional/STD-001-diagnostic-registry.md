---
id: STD-001
title: "Contract IR diagnostic code registry"
type: Standard
code: contract-ir-diagnostics
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
  - target: ix://agent-ix/quire-contract-ir/FR-030
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-044
    type: references
---
# STD-001: Contract IR diagnostic code registry

## Description

This registry owns the stable machine-readable diagnostic codes of FR-011
through FR-019, the bounded-Kani cause and outcome error codes FR-030 raises and
the code-form code FR-044 raises. Implementations may add human context but shall not parse
or synthesize codes from messages. Codes are lowercase ASCII snake case, and a
code is carried as a `Std001Code` (FR-044). A registered code is never renamed,
reused for another condition or removed while a release names it.

## Identity and Reference Codes

| Code | Condition | Required location |
|---|---|---|
| `invalid_package_namespace` | Empty or malformed package namespace | package identity path |
| `invalid_wire_format` | JSON syntax or closed wire shape prevents decoding, the 576-level wire-nesting limit is exceeded, or an executable projection holds a value its published schema bounds (a revision or byte offset above 9007199254740992 (2^53), FR-023-AC-6) | document path; `document.nesting` identifies pre-decode depth refusal; `projection` for a projection's schema refusal |
| `invalid_schema_version` | Zero schema major | schema-version path |
| `invalid_identifier` | Empty or malformed source-document, requirement, clause, anchor, or dependency-path-segment identifier | offending identity path |
| `invalid_requirement_revision` | Zero or non-increasing requirement revision, or a requirement revision above 9007199254740992 (2^53, the largest integer RFC 8785 spells exactly) | requirement revision path |
| `invalid_source_revision` | Zero source-document revision, or a source-document revision above 9007199254740992 (2^53) | source revision path |
| `duplicate_requirement` | Two current requirements share one ID | later requirement path and earlier related identity |
| `duplicate_clause` | Two clauses in one requirement share one ID | later clause span and earlier related identity |
| `cross_package_reference` | A reference names a different package | reference span/path |
| `invalid_source_span` | Source endpoints are zero-based-invalid, reversed, name a different source, or a byte offset above 9007199254740992 (2^53) | source span |
| `floating_executable_clause` | Executable clause has no anchor | clause span |
| `informational_clause_anchored` | Informational clause has an anchor | clause span |
| `incompatible_clause_anchor` | Clause and anchor kinds violate the closed compatibility table | clause span |
| `malformed_reference` | Reference identity is structurally malformed | reference span/path |
| `stale_requirement_revision` | Requirement exists but the exact revision differs | reference span/path and current related identity |
| `orphaned_requirement_reference` | Referenced requirement ID is absent | reference span/path |
| `orphaned_clause_reference` | Requirement revision resolves but its clause ID is absent | reference span/path |

## Type and Expression Codes

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

## Version, Canonicalization and Coverage Codes

| Code | Condition | Required location |
|---|---|---|
| `unsupported_schema_version` | A well-formed schema version other than 1.1, read by wire preflight or presented to a canonical API | preflight: `schema_version.major` when the major is not 1, otherwise `schema_version.minor`; canonical API: `schema_version`; no semantic span |
| `canonicalization_resource_exhausted` | Canonical byte allocation cannot be reserved without exceeding host resources | canonicalized object path; source span when the object has one |
| `duplicate_artifact_trace` | A later artifact trace repeats an artifact ID in one classification input | later trace span |

## Semantic Limit Codes

| Code | Condition | Required location |
|---|---|---|
| `semantic_input_too_large` | A complete operation exceeds 25000 semantic nodes, recursive semantic depth 256, or 10000 entries in any semantic collection | first node, depth, or collection path crossing the limit; source span when present |

## Typed Code Form

Every registered code is carried as a `Std001Code` (FR-044), a type of
`quire-contract-model` that is a validated lowercase-ASCII-snake-case string and
not a membership claim: the registered codes outside `DiagnosticCode` are
constants of that type. The code below is the `code` of a `Std001CodeError`.

| Code | Condition | Required location |
| --- | --- | --- |
| `invalid_code_form` | A candidate code is empty, longer than 64 bytes, begins with other than `a` to `z`, holds a byte other than `a` to `z`, `0` to `9` and `_`, ends in `_` or holds two adjacent `_` | none; the error holds no copy of the input |

## Bounded Kani Cause Codes

These codes are the stable cause codes of the `KaniOutcome` the root crate's
`kani` module returns, typed as `Std001Code` (FR-030, FR-044). A code of an
outcome kind other than `proved` and `counterexample` is the outcome's cause.
The map from an outcome to a QSL terminal value reads a code only where its
requirement says so, and no code becomes a QSL `TerminalValue`.

| Code | Condition | Required location |
| --- | --- | --- |
| `kani_proved` | The code every `proved` outcome carries | outcome source identity |
| `kani_counterexample` | The code every `counterexample` outcome carries | outcome source identity |
| `kani_identity_invalid` | A finite input has an empty model or source identity (`invalid_input`) | outcome source identity |
| `kani_population_incomplete` | A finite input declares an incomplete population (`incomplete_input`); this is not a missing replay input, which QSpec owns | outcome source identity |
| `kani_bound_invalid` | A finite input has a zero object or byte bound (`invalid_input`) | outcome source identity |
| `kani_bound_exhausted` | A finite input exceeds a selected byte, object or reference bound (`resource_exhausted`) | outcome source identity |
| `kani_population_invalid` | A finite input holds an object with an empty identity, type or snapshot, or a duplicate object identity (`invalid_input`) | outcome source identity |
| `kani_reference_invalid` | A finite input holds a reference with an empty field, a dangling or foreign endpoint, or a duplicate reference (`invalid_input`) | outcome source identity |
| `kani_capability_request_invalid` | A capability request names an empty or repeated construct (`refused`) | outcome source identity |
| `kani_capability_missing` | A capability request names a construct the profile's matrix does not hold (`refused`) | the requested construct |
| `kani_solver_absent` | An `unavailable` outcome: the run finds no solver satisfying the selected profile's capability negotiation | outcome source identity |
| `kani_backend_absent` | An `unavailable` outcome: the run finds no Kani backend satisfying it | outcome source identity |
| `kani_vacuous_proof` | An `inconclusive` outcome: a proof whose obligation completed with zero SUCCESS checks | outcome source identity |

The family lowerings for checked arithmetic, collections and objects are
codegen's (FR-039), and the codes they raise (`kani_dispatch_*`,
`kani_arithmetic_*`, `kani_definedness_*`, `kani_collection_*`, `kani_graph_*`)
are not registered here: they leave this repository with those lowerings
(IR-347). Codegen's own refusal codes in `KaniOutcome`, such as its
`kani_corpus_*` codes and `kani_profile_input_mismatch`, are well-formed
`Std001Code`s that this registry does not list.

## Bounded Kani Outcome Error Code

This code is the `code` of the `KaniOutcomeError` the root crate's `kani`
module returns. It is raised only by the validated `KaniOutcome` constructors
(FR-030) and never becomes a `KaniOutcome` cause code or a QSL
`TerminalValue`.

| Code | Condition | Required location |
| --- | --- | --- |
| `kani_outcome_invalid` | A `KaniOutcome` constructor is asked for an outcome that breaks FR-030's kind and cause rules: an `unavailable` cause other than `kani_solver_absent` or `kani_backend_absent`, an `inconclusive` cause other than `kani_vacuous_proof`, a `proved` SUCCESS check count of zero, or a `proved` or `counterexample` kind through the non-success constructor; no outcome is built | requested kind and cause code, or the rejected count |

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

Type and expression precedence is declaration/identifier grammar and numeric/collection
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
Within one node the type and expression precedence above selects the primary code. State
observation policy follows resolved-value lookup and precedes operand typing.
Declaration-environment diagnostics precede expression diagnostics and follow
stored declaration/field/variant/parameter order.

Wire precedence is JSON/top-level structure, schema-version numeric
grammar, unsupported version, then semantic
package interpretation. Canonicalization accepts validated values only and
performs no diagnostic recovery; resource exhaustion produces no partial bytes
or digest. Coverage precedence per trace is duplicate artifact ID,
cross-package target, missing requirement, then stale revision. Coverage diagnostics retain authored trace order even though report
rows sort structurally.

## Dependencies

- **Downstream**: FR-013 through FR-019, FR-023 and FR-030 extend or consume this
  semantic registry without renaming its codes. FR-020 defines separate
  runner operational codes that are neither `DiagnosticCode` values nor
  semantic diagnostic shapes.
