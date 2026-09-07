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

The selected design for issue #50 is an explicitly keyed binder, preserving the
existing ReferenceBody package representation. The normal source remains
authoritative Quire Markdown, module-owned models, and source-located formal
clauses, lowered through the validated frontend/model boundary owned by #52.
The projection is derived interchange, not an expression-sidecar authoring
format. Synthetic projections in binder tests are labelled as such; they do
not demonstrate frontend coverage. Binding does not authenticate a producer,
prove source-language correctness, or replace shared provenance/assurance.

## Inputs

`BoundPackage::from_json_bytes` takes a strict UTF-8 JSON object with exactly:

- `format`: `quire.contract.executable-projection/v1`;
- `package`: the FR-011/FR-018 ReferenceBody package wire representation;
- `bindings`: an array of objects containing `clause` (full ClauseRef) and
  `expression` (the published expression-input representation from FR-018).

The published Draft 7 projection schema owns this envelope and references the
existing package and expression-input schemas. Clients consume that IR-owned
schema and decoder; they never import IR-private Rust wire structs.

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
informational clause references. A BoundClause exposes its full identity, kind,
anchor, source span, DeclarationEnvironment, TypedExpression and canonical
declaration/expression identities. No clause disappears from the population:
information is explicit, and unsupported executable content is a refusal rather
than an omission or an information classification.

The bound identity uses a versioned canonical envelope of the package's
canonical identity and structurally ordered full clause references with their
canonical declaration and expression identities. Binding-array order does not
change this identity; a semantic expression or declaration change does. Existing
canonical profile rules govern source exclusion; source provenance remains
available separately through the package and clause/declaration source spans.
This digest identifies bound semantics, not a frontend execution or a proof.

The v1 bound identity is SHA-256 over compact UTF-8 JSON (no BOM, whitespace or
trailing newline), with object keys in lexical order. Its exact envelope is
`{"bindings":[{"clause":<full ClauseRef>,"declaration":<digest>,"expression":<digest>}],"canonical_profile":"quire.contract.canonical-json/v1","package":<digest>,"profile":"quire.contract.bound-identity/v1"}`.
All digests are lowercase 64-character hexadecimal strings. Bindings use
structural ClauseRef order; the nested ClauseRef uses the published wire fields
in lexical key order. This profile is separate from the existing FR-014 package,
declaration and expression identity profiles, which remain unchanged.

## Outputs

A `BoundPackage` exposing read-only accessors, or structured IR diagnostics.
No conformance result, attestation, test verdict, retained evidence, package
publication or source-release decision is emitted by the binder.

## Dependencies

- FR-011 and FR-012 own package, requirement, clause and reference identity.
- FR-013 through FR-015 own declarations, typed expressions and definedness.
- FR-016 owns canonical semantic identity; FR-018 owns the shared wire forms.
- Issue #52 owns the authoritative source/frontend/model join. This binder does
  not claim that frontend work is implemented.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-023-AC-1 | A labelled synthetic projection using shared corpus package/expression forms decodes through the public API into every expected executable clause plus explicit informational references, preserving identity, anchor, source, declarations and typed expression. | Test (TC-035) |
| FR-023-AC-2 | Missing, duplicate, foreign, stale-revision, orphaned, informational, wrong-owner, wrong-anchor, non-Boolean-root and ill-typed bindings are refused with structured diagnostics and no partial result. | Test (TC-035) |
| FR-023-AC-3 | Metadata/expression dependency disagreement is refused; changing binding order preserves the bound semantic digest and changing a bound expression changes it. | Test (TC-035) |
| FR-023-AC-4 | Unknown format/members, trailing JSON, invalid UTF-8 and byte/depth/node/collection overruns are refused without panic; aggregate limits cannot be multiplied by adding bindings. | Test (TC-035) |
| FR-023-AC-5 | An external integration test consumes public BoundPackage/BoundClause accessors without private wire imports or a codegen-owned package schema; normative projection schema positive and negative controls agree with the decoder. | Test (TC-035) |
