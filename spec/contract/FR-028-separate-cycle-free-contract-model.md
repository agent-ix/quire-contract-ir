---
id: FR-028
title: "Separate the cycle-free Contract IR model package"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: depends_on
  - target: ix://agent-ix/tl-syntax/IF-008
    type: implements
---
# FR-028: Separate the cycle-free Contract IR model package

## Description

The Contract IR repository SHALL expose its stable semantic substrate only from a dependency-free `quire-contract-model` package, which the `quire-contract-ir` package consumes without re-exporting it, so the production Cargo graph remains acyclic, the root package depends on no crate from the agent-ix/quire-spec-language repository, and every model item has one import path.

## Package architecture

The repository becomes one Cargo workspace with:

- `quire-contract-model`: the existing package/requirement/clause, type,
  expression, definedness, canonicalization, identity, binding, limit,
  diagnostic, wire and conformance model. It has no dependency on QSL,
  observation, protocol or TL crates.
- `quire-contract-ir`: the root package. It depends on the model package and
  on no crate from the agent-ix/quire-spec-language repository, in any dependency kind, re-exports none of its items, and owns the bounded-Kani subsystem from
  FR-029 through FR-031 (FR-039).

A model type, function, error, feature, canonical byte, diagnostic and
conformance outcome is reached through `quire_contract_model` only (FR-019).
The package split changes no wire/schema/profile/identity.

QSL's production graph depends on `quire-contract-model` and not on
`quire-contract-ir`. The root package depends on no crate from the agent-ix/quire-spec-language repository, so no path or git
split of the model crate can arise in this repository's `Cargo.lock`: the
workspace path model is the only copy. Test-only dependencies cannot enter
production code.

## Dependency and admission rules

The workspace Cargo graph SHALL have these directions:

```text
quire-verification-contracts -> quire-contract-model
quire-contract-model -> quire-contract-ir
```

Arrows point from dependency to consumer. No package may depend, directly or
transitively, on itself. The model package admits no optional feature that
reintroduces an owner/bridge dependency. The bridge accepts owner values only
through their constructor-private public APIs; the package split adds no
callback, trait-object validator, wire mirror, trust flag or local owner parser.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-028-AC-1 | Cargo metadata for every production feature combination is acyclic and contains no owner or TL dependency reachable from `quire-contract-model`. | Test (TC-041) |
| FR-028-AC-2 | Schemas, canonical bytes, identities, diagnostics and the conformance corpus are byte/result identical through `quire_contract_model` paths, and the root package's runner produces the same results from them. | Test (TC-041) |
| FR-028-AC-3 | The root package's own manifest names no dependency in any kind on a crate from the agent-ix/quire-spec-language repository, by name or by git source. | Test (TC-041) |
| FR-028-AC-4 | Default, all-feature and minimum-version builds prove that no optional, dev or historical dependency leaks into the production graph. | Test (TC-041) |
| FR-028-AC-5 | The split introduces no copied owner wire type, public validation constructor, callback, trait object, trust flag or local QSL/observation/protocol/TL parser. | Test (TC-041) |

## Dependencies

FR-019 supplies the model crate's Rust library surface, which the root
package does not re-export. The exact
workspace boundary and owner direction are defined by `tl-syntax` ADR-003 and
IF-008.

## Status

AC-1, AC-3 and AC-4 are implemented; AC-5 is tagged but only partly tested (see the matrix). AC-3 is the root manifest's absence of QSL-repository dependencies, backed by TC-041. Planned work, not implemented and outside this repository: that QSL's production graph has no `quire-contract-ir`, and the cross-repo composition build that imports both the root package and the real QSL owner API, are to be built in agent-ix/quire-integration (Linear IR-358). AC-2 is planned: TC-041 reaches
the model through the root package's `pub use quire_contract_model::*`
re-export, which FR-039 excludes.
