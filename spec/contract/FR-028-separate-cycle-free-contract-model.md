---
id: FR-028
title: "Separate the cycle-free Contract IR model package"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-025
    type: supports
  - target: ix://agent-ix/quire-contract-ir/FR-026
    type: supports
  - target: ix://agent-ix/tl-syntax/IF-008
    type: implements
---
# FR-028: Separate the cycle-free Contract IR model package

## Description

When the native owner and Contract IR bridge are composed, the Contract IR repository SHALL expose its existing stable semantic substrate from a dependency-free `quire-contract-model` package and keep the `quire-contract-ir` package as a compatibility-reexporting bridge consumer so the production Cargo graph remains acyclic.

## Package architecture

The repository becomes one Cargo workspace with:

- `quire-contract-model`: the existing package/requirement/clause, type,
  expression, definedness, canonicalization, identity, binding, limit,
  diagnostic, wire and conformance model. It has no dependency on QSL,
  observation, protocol or TL crates.
- `quire-contract-ir`: the existing public package name. It depends on and
  publicly re-exports the complete model API, then owns predicate, temporal and
  ecosystem-model subsystems from FR-025 through FR-027.

Every existing `quire-contract-ir` public type, function, error, feature,
canonical byte, diagnostic and conformance outcome remains available under its
current path. A source-only move changes no wire/schema/profile/identity.

QSL replaces its production package selection with
`quire-contract-model` at the exact reviewed revision while retaining the
dependency key `quire-contract-ir`; its existing Rust imports therefore remain
source-compatible. Contract IR may then depend on the QSL owner crate and the
other owner crates without a reverse production edge. Test-only historical
pins remain explicitly named and cannot enter production code.

## Dependency and admission rules

The workspace Cargo graph SHALL have these directions:

```text
quire-contract-model -> quire-spec-language -> quire-protocol
quire-contract-model -> quire-contract-ir
quire-spec-language + quire-observation + quire-protocol
  + tl-syntax + tl-mltl -> quire-contract-ir
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
| FR-028-AC-2 | The existing Contract IR public API, schemas, canonical bytes, identities, diagnostics and conformance corpus are byte/result identical through `quire-contract-ir` compatibility re-exports. | Test (TC-041) |
| FR-028-AC-3 | QSL builds unchanged source imports against the pinned `quire-contract-model` package alias, while a locked composition build imports both the compatibility bridge and the real QSL owner API without a Cargo cycle. FR-025 owns adding that API as a production bridge dependency when its remaining owners are available. | Test (TC-041) |
| FR-028-AC-4 | Default, all-feature and minimum-version builds prove that no optional, dev or historical dependency leaks into the production graph. | Test (TC-041) |
| FR-028-AC-5 | The split introduces no copied owner wire type, public validation constructor, callback, trait object, trust flag or local QSL/observation/protocol/TL parser. | Test (TC-041) |

## Dependencies

FR-019 supplies the established Rust library surface that the compatibility
package preserves. FR-025/FR-026 consume the cycle-free owner graph. The exact
workspace boundary and owner direction are defined by `tl-syntax` ADR-003 and
IF-008.

## Status

Implemented for `quire-contract-ir#73` as the architecture enablement for
`tl-syntax#52/#64`; production owner integration remains allocated to FR-025
and FR-026.
