---
id: FR-018
title: "Publish the schema and conformance corpus"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-003
    type: traces_to
---
# FR-018: Publish the schema and conformance corpus

## Description

The repository shall publish a Draft 7 JSON schema, representative valid
packages, targeted invalid packages, expected diagnostics, canonical encodings
and their canonical digests, dependency sets, and a reusable conformance runner.

## Inputs

A corpus directory holding fixture inputs and expected outcomes, and the
repository `schemas/` directory holding the package and fixture schemas.

## Outputs

Machine-readable per-fixture results with stable diagnostics and a nonzero exit
status for any mismatch.

## Behavior

The published package schema is JSON Schema Draft 7 with identity
`https://agent-ix.github.io/quire-contract-ir/schemas/contract-package-reference-v1.schema.json`.
It describes the complete `ContractPackage<ReferenceBody>` wire representation
for the supported schema version 1.1, closes every object with
`additionalProperties: false`, uses fixed-width numeric bounds, and carries no
implementation-language names. Schema success never substitutes for semantic validation.

Fixture payloads are validated by Draft 7 schema identity
`https://agent-ix.github.io/quire-contract-ir/schemas/contract-conformance-fixture-v1.schema.json`.
The conformance schema exposes the named subschemas `packageInput`,
`expressionInput`, `coverageInput`, and one corresponding
`*Expectation` subschema for each operation. The runner selects the input and
expectation subschemas from the fixture's operation before any semantic
conversion; every object forbids unknown fields.

The corpus is a directory. Its name is the corpus identity and is a validated
identifier. The runner reads the package and fixture schemas from the schema
directory it is given. Each `inputs/<id>.json` is one fixture whose operation is the `<id>` prefix before
the first `-`, and whose expectation is `expectations/<id>.json`. A fixture
name that does not start with one of the four operations fails, as does any
`inputs/` or `expectations/` entry that is not a UTF-8 `.json` file and any
expectation with no input. Canonical-byte
paths in an expectation are relative to the corpus directory, contain no empty,
`.` or `..` segment, and after symlink resolution remain below it. Each schema,
input, expectation, or canonical-byte file is at most 16777216 bytes, checked
before parsing; the corpus holds at most 10000 fixtures. Unsafe paths, unknown
fields, malformed inputs or expectations, or resource-limit breach fail before
any fixture executes. Every file read shares a 67108864-byte aggregate preload
budget.

The four closed fixture operations are:

| Operation | Declarative input | Comparable result |
|---|---|---|
| `package` | package JSON, optionally wrapped with authored clause-resolution references and a canonical byte limit, or raw package JSON text for decoder-boundary probes | validity, ordered diagnostics, package/requirement/clause canonical bundle and package dependency union |
| `expression` | declarations, expression, expected type, execution point, clause-root flag | validity, ordered diagnostics, separate declaration/expression canonical outputs and expression dependencies |
| `coverage` | reference-body package and artifact traces | ordered diagnostics and sorted requirement/artifact rows |

Expression fixture syntax covers every FR-013/FR-014 declaration, value type,
literal, reference, access, call, numeric, comparison, Boolean, option,
collection, record, and quantifier variant. It is an unvalidated wire model;
conversion invokes the same public constructors and checker as the Rust API.
Fixture IDs never select constructors, expected results, or special behavior.

Each result carries the fixture's `covers`, the observed coverage tokens that
the owned `schemas/conformance-trace-map-v1.json` registry maps for its
operation, and its `trace_ids`, the sorted unique union of the
acceptance-criterion targets those tokens map to. Every inventory token has an
owner; each applicable operation/token pair has exactly one entry, which may
name several relevant criteria. In particular, reference-body dependencies do
not claim typed-expression criteria, and artifact orphan diagnostics do not
claim semantic-reference resolution. A trace identifies a
relevant observation, not proof that every conjunct of that criterion passed;
Quire owns criterion identity and static relationships, and the wider test suite
still owns criteria not exercised by this corpus. Coverage tokens are the closed forms
`construct:<registered-tag>`, `diagnostic:<STD-001-code>`,
`obligation:<DefinednessObligationKind>`, `boundary:<registered-boundary>`, and
`operation:<operation>`. The Rust library
exports the sorted fixed-width registries `PUBLIC_CONSTRUCT_TAGS` and
`CONFORMANCE_BOUNDARIES`; operation tokens come from the four-operation enum,
diagnostic tokens from `DiagnosticCode::ALL`, and obligation tokens from all
four `DefinednessObligationKind` values. Construct tags are qualified by
wire namespace, for example `expression.boolean_literal`, `type.boolean`, and
`clause_kind.precondition`, so equal snake-case variant names cannot collide.
The union of observed tokens over the corpus must equal the inventory; an
absent required token is a corpus failure. Every STD-001 diagnostic has a
failing fixture, every public wire construct has a successful fixture, and the
four obligation values have distinct `potentially_undefined` fixtures. The
inventory is derived exactly from `PUBLIC_CONSTRUCT_TAGS`,
`CONFORMANCE_BOUNDARIES`, the four-operation enum, `DiagnosticCode::ALL`, and
the four-obligation enum.

Coverage tokens are observations, never fixture declarations.
`operation:` is observed only by selecting that declared operation;
`diagnostic:` and `obligation:` are observed only in the actual structured
diagnostic result; and `construct:` requires semantic success plus the named
wire/result construct. Except for the deliberately shape-invalid wire-depth
probes specified below, a valid minimum, maximum, normalization, revision,
schema, canonical-order, or depth boundary requires semantic success and its
exact structural predicate. An invalid boundary requires both its exact
structural predicate and the owning actual diagnostic. Artifact boundaries are
observed only by the coverage operation's artifact result/diagnostic domain;
a package reference diagnostic cannot claim an artifact-trace boundary.

The closed boundary registry is `source_span.minimum`, `source_span.reversed`,
`revision.current`,
`revision.stale`, `schema.1_1`, `schema.zero_major`,
`schema.unknown_major`, `schema.unsupported_minor`, `integer.minimum`,
`integer.maximum`, `integer.out_of_range`, `rational.normalized`,
`rational.zero_denominator`, `rational.maximum_denominator`, `text.maximum`,
`text.over_maximum`, `collection.declared_maximum`,
`collection.declared_out_of_range`, `collection.minimum`,
`collection.maximum`, `collection.over_maximum`, `expression.depth.maximum`,
`expression.depth.over_maximum`, `expression.nodes.maximum`,
`expression.nodes.over_maximum`, `type.depth.maximum`,
`type.depth.over_maximum`, `semantic.nodes.maximum`,
`semantic.nodes.over_maximum`, `semantic_collection.maximum`,
`semantic_collection.over_maximum`, `canonical.escape_controls`,
`canonical.semantic_set_order`, `canonical.sequence_order`,
`canonical.resource_failure`, `artifact.cross_package`, `artifact.missing`,
`artifact.stale`, `artifact.duplicate`, and `artifact.digest_mismatch`.
The decoder registry also includes `wire.depth.maximum` and
`wire.depth.over_maximum`; raw-text package probes exercise exactly 576 and 577
levels without requiring the fixture decoder to materialize the nested value. The at-limit probe
must reach ordinary package-shape decoding at `document`; the over-limit probe must be rejected by
the pre-decode nesting guard at `document.nesting`. This distinction keeps the wire-depth cliff
observable even though both deliberately shape-invalid probe documents share the
`invalid_wire_format` code.

Expectations contain only fields meaningful for their operation. Valid results
carry no diagnostics; invalid results carry the exact authored-order diagnostic
code, semantic path, optional span, related identities, and optional obligation
kind. Successful canonical results carry exact UTF-8 bytes as a fixture path and
lowercase digest. A package result carries the package output plus separately
sorted requirement and clause outputs; an expression result carries the
declaration environment and typed expression outputs separately. Dependency
identities and coverage rows use their normative structural order. The runner
compares values structurally and bytes exactly; it never parses diagnostic
messages.

Downstream tools can execute the process runner without linking the Rust
library. The checked-in generator is owned by this requirement, rejects a
declared token the runner does not observe, and has a scratch-directory
byte-for-byte regeneration gate. Frozen outputs establish regression stability and
determinism for this implementation, not independent semantic correctness.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-018-AC-1 | Running the corpus directory yields one matching row per input with non-empty trace targets, and the union of observed tokens equals the published inventory, so every registered public construct, STD-001 diagnostic, operation, and boundary token is covered; a corpus missing a token's only fixture, a fixture with an unknown operation prefix or a missing expectation, and oversize, over-count and over-budget corpora fail before any row is written. | Test (TC-018) |
| FR-018-AC-2 | Mutation fixtures independently alter schema validity, diagnostic code/path/order/span/obligation, canonical byte, digest, dependency, and coverage row/reason; each produces the exact mismatch result without message parsing. | Test (TC-018) |
| FR-018-AC-3 | Raw package probes pin exact and one-past wire depth, and quoted delimiters do not count toward depth. | Test (TC-018) |

## Dependencies

FR-011 through FR-017 define the normative corpus behavior; FR-019 owns the
stable public registry exports consumed here.
