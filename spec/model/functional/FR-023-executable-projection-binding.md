---
id: FR-023
title: "Bind derived executable projections to their clause population"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-012
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-014
    type: depends_on
---

# FR-023: Bind derived executable projections

## Description

IR shall own the bounded, versioned boundary between a validated frontend's
derived serialized projection and the immutable executable clause population
consumed by code generation and analysis. Codegen shall not define a second
package wire format or infer clause bindings from conformance JSON output.

The selected design is an explicitly keyed binder, preserving the
existing ReferenceBody package representation. The normal source remains
authoritative Quire Markdown, module-owned models, and source-located formal
clauses, lowered through the validated frontend/model boundary.
The projection is derived interchange, not an expression-sidecar authoring
format. Synthetic projections in binder tests are labelled as such; they do
not demonstrate frontend coverage. Binding does not authenticate a producer or
prove source-language correctness.

## Inputs

`BoundPackage::from_json_bytes` takes a strict UTF-8 JSON object with exactly:

- `format`: `quire.contract.executable-projection/v1`;
- `package`: the FR-011/FR-018 ReferenceBody package wire representation;
- `bindings`: an array of objects containing `clause` (full ClauseRef) and
  `expression` (the published expression-input representation from FR-018).

The published Draft 7 projection schema owns this envelope and references the
existing package and expression-input schemas. Clients consume that IR-owned
schema and decoder; they never import IR-private Rust wire structs.
The decoder compiles this fixed embedded schema once per process and reuses it
for later projections. Each decode indexes its declaration population once;
lookups by declaration name use that index while preserving diagnostic order.

## Behavior

The decoder rejects unknown members, duplicate members at every nesting level,
and trailing JSON. Duplicate members must be refused before any Value-like
normalization can erase their occurrence. The input byte bound is
16777216; the existing 576 wire-depth, semantic node/depth/collection, and
expression node/depth limits remain active. A projection has an aggregate
semantic budget, not a fresh unrestricted allowance per expression. Limit or
schema failure returns structured diagnostics with no partial BoundPackage.
On supported native targets the decoder reserves 16 MiB of same-thread stack
space when necessary for bounded recursive schema/semantic construction, after
the raw byte/depth guard. Protecting only initial JSON parsing is insufficient.
This is a logical input/resource contract, not a guarantee against host-wide
memory exhaustion. No operating-system worker thread is required by this API.

The schema stage is the projection's first gate after the byte, depth and
collection guards, and it decides every value the published schemas bound. A
source or requirement revision, or a byte offset, above 9007199254740992 (2^53,
the largest integer RFC 8785 spells exactly, FR-016) is outside both published
schemas wherever it stands (the package, a binding's clause reference, an
expression's source span), so a projection carrying one is refused
`invalid_wire_format` at path `projection`, with no partial BoundPackage and
before any typed decoder runs. It is never `invalid_requirement_revision`,
`invalid_source_revision` or `invalid_source_span`: those registered codes are
the codes of the constructors and of the typed decoders that run without a
schema stage (FR-011-AC-3, FR-012-AC-6), which a projection reaches only with
a value the schema admits. The schema is not relaxed for these members and the
stages are not reordered: the published schemas are the normative interchange
whose 2^53 bound is the exact-integer rule their consumers rely on, so
relaxing them would let a document pass a published validator that IR then
refuses; the decoder-first order would run the typed decoders over shapes no
schema has bounded and leave FR-023-AC-5's agreement of schema and decoder
unmeasurable. The same stage refuses the same value in the probe and coverage
inputs of the conformance corpus (FR-020-AC-5), where it is an `invalid_corpus`
failure of the run and never reaches a decoder.

The package is validated before binding. Every executable clause requires
exactly one expression. Bindings resolve by package, requirement identity and
revision, and clause identity, never array position. Missing, duplicate,
foreign, stale, orphaned or informational-clause bindings fail. An expression's
declaration owner must equal its clause's requirement reference; its execution
point must equal the validated clause anchor; its expected type must be Boolean
and its clause-root flag must be true. Normal IR typing and definedness checks
remain mandatory. The expression's derived dependency set must equal the
ReferenceBody clause dependency metadata, preventing two conflicting accounts
of the same executable clause.

The immutable result exposes the validated package, all executable BoundClause
entries in structural ClauseRef order, and the separately enumerable
informational clause references. A BoundClause exposes its full clause reference,
kind, anchor, source span, DeclarationEnvironment and TypedExpression. No clause disappears from the population:
information is explicit, and unsupported executable content is a refusal rather
than an omission or an information classification.

The result carries no bound-content digest, digest envelope or declaration and
expression digest fields. Its package name, schema version and full clause
references (including requirement revisions) are the identifiers available to
consumers. The IR binder does not produce a digest-only mismatch refusal or
claim to detect different content under the same name and version. Binding-array
order does not change the structurally ordered result, while a semantic expression or declaration
change remains visible in the exposed typed values. Source provenance remains
available through the package and clause/declaration source spans.

## Outputs

A `BoundPackage` exposing read-only accessors, or structured IR diagnostics.

## Dependencies

- FR-011 and FR-012 own package, requirement, clause and reference identity.
- FR-013 through FR-015 own declarations, typed expressions and definedness.
- FR-016 owns canonical bytes; FR-018 owns the shared wire forms. QSL owns
  source association when it consumes this result; IR defines no replacement
  source-mismatch refusal.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-023-AC-1 | A labelled synthetic projection using shared corpus package/expression forms decodes through the public API into every expected executable clause plus explicit informational references, preserving identity, anchor, source, declarations and typed expression. | Test (TC-035) |
| FR-023-AC-2 | Missing, duplicate, foreign, stale-revision, orphaned, informational, wrong-owner, wrong-anchor, non-Boolean-root and ill-typed bindings are refused with structured diagnostics and no partial result. | Test (TC-035) |
| FR-023-AC-3 | Metadata/expression dependency disagreement is refused; changing binding order preserves the same structurally ordered BoundClause values, and changing a bound expression changes its exposed TypedExpression while leaving the package name, schema version and ClauseRef identifiers unchanged. No bound-content digest is produced. | Test (TC-035) |
| FR-023-AC-4 | Unknown format/members, trailing JSON, invalid UTF-8 and byte/depth/node/collection overruns are refused without panic; aggregate limits cannot be multiplied by adding bindings. | Test (TC-035) |
| FR-023-AC-5 | An external integration test consumes public BoundPackage/BoundClause accessors without private wire imports or a codegen-owned package schema; normative projection schema positive and negative controls agree with the decoder. | Test (TC-035) |
| FR-023-AC-6 | A projection whose package source revision, package requirement revision, binding clause reference revision, expression source-identity revision or expression byte offset is 9007199254740993 (and, for each, `u64::MAX`) is refused `invalid_wire_format` at path `projection`, with no partial BoundPackage, and never `invalid_requirement_revision`, `invalid_source_revision` or `invalid_source_span`; the same projection with that member at 9007199254740992 binds. | Test (TC-035) |
| FR-023-AC-7 | Across repeated valid and invalid projection decodes, the fixed embedded schema is compiled once per process, and a decode with D declarations and B binding/declaration lookups performs at most D+B declaration search visits; the same inputs yield equal ordered BoundClause values or the same ordered diagnostics as before caching and indexing. | Test (TC-035) |
