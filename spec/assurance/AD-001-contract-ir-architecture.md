---
id: AD-001
title: "quire-contract-ir semantic architecture"
type: ArchitectureDescription
status: proposed
owner: kreneskyp
system: quire-contract-ir v0.1 semantic model, schema, canonicalizer, diagnostics, and corpus runner
relationships:
  - target: ix://agent-ix/quire-contract-ir/AP-001
    type: realizes
---
# quire-contract-ir semantic architecture

## System Boundary

The repository owns unvalidated wire values, validated semantic values,
identity and source-span types, type/definedness checking, dependency derivation,
canonicalization, digests, version migration, coverage classification, schemas,
fixtures, and the conformance runner. Downstream generators, runtimes, solvers,
Quoin, Quire, build infrastructure, and human release decisions remain outside.

## Views

```text
JSON bytes -> wire model -> schema validation -> semantic validation
                                         |-> ordered diagnostics
validated package -> dependency walk -> canonical encoder -> SHA-256 identities
validated package + artifact traces -> shallow/deep/uncovered/orphaned coverage
schema + corpus manifest + fixtures -> process runner -> JSON Lines results
```

The Rust library and process runner share semantic operations. The checked-in
Draft 7 schema is the language-neutral wire boundary. `serde` transports values;
`sha2` computes identities; neither defines semantics. Downstream engines depend
on published types and bytes, not internal modules.

## Decisions

- Separate wire parsing from validated semantic construction.
- Use a closed v0.1 type/operator vocabulary and reject unknown variants.
- Canonicalize with explicit ordering rules instead of serializer defaults.
- Treat source spans as provenance excluded from expression equivalence but
  included in diagnostics and package-level provenance.
- Keep migration explicit and one-way; never interpret unknown majors.
- Keep `publish = false` through the human v0.1 decision.

## Risks

- Schema and Rust model drift: controlled by round-trip and schema mutation tests.
- Canonicalization ambiguity: controlled by golden bytes and property tests.
- Partial-operation unsoundness: controlled by definedness rules and negative fixtures.
- Orphan false coverage: controlled by exact revision identities and separate class.
- External module/tool drift: controlled by exact pins, digest-checked against
  the released Engineering Assurance artifact this repository consumes.
