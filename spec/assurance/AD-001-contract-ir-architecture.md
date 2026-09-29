---
id: AD-001
title: "quire-contract-ir repository architecture and versioning"
type: ArchitectureDescription
status: accepted
owner: kreneskyp
system: quire-contract-ir workspace (quire-contract-model and quire-contract-ir crates), its versioned contracts and its boundaries with QSpec, QSL, codegen and runtime
relationships:
  - target: ix://agent-ix/quire-contract-ir/AP-001
    type: realizes
  - target: ix://agent-ix/quire-contract-ir/AD-002
    type: references
  - target: ix://agent-ix/quire-contract-ir/AD-003
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-028
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-039
    type: references
  - target: ix://agent-ix/quire-specification/AD-016
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-011
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-013
    type: references
---
# quire-contract-ir repository architecture and versioning

This is the one architecture description for the repository. AD-002 (the
bounded-Kani profile) and AD-003 (complete-V1 backend delivery) are views of
one boundary each inside it.

## System Boundary

The workspace holds two crates. Each public item has one owning crate and
one import path.

`quire-contract-model` is the cycle-free semantic substrate. It owns
unvalidated wire values, validated semantic values, identity and source-span
types, type and definedness checking, dependency derivation,
canonicalization, digests, coverage classification, the conformance
registries, the target-neutral output-mapping foundation (FR-032 through
FR-034), and the `quire.checked-package/v2` reader and lowerer (FR-038,
FR-040) with the `ContractPackage` it emits (FR-035). Its crate root
re-exports each public item by name from its module, and the items it
re-exports are exactly FR-019's public item table. It has no dependency on
QSL, Quire Observation, Quire Protocol, tl-syntax, tl-mltl or
`quire-contract-ir` (FR-028).

`quire-contract-ir` is the root crate. Its public API is its own owner
integrations, which need other owners' crates (FR-039), and it re-exports no
`quire-contract-model` item. A consumer that needs a model type depends on
`quire-contract-model` and imports it from there.

| Module | Owns | Requirement |
| --- | --- | --- |
| `bridge` | shared owner-bridge errors, digests, limits and contract selections | FR-025, FR-026 |
| `predicate` | native checked predicate to TL Boolean signal projection | FR-025 |
| `temporal` | native temporal clause to TL formula correspondence and result join | FR-026 |
| `ecosystem_model` | bounded, non-authoritative temporal-ecosystem model export | FR-027 |
| `kani` | the `kani-bounded/1` profile, finite input ABI, dispatch index, typed outcome, provenance, and the Kani outcome to QSL terminal-value map | FR-029 through FR-031 |

The conformance runner (`src/bin/`) and its fixtures stay in the root
package.

### What each owner holds

| Owner | Holds | Contract IR's relation |
| --- | --- | --- |
| QSpec (`quire-specification`) | the normative language, the `quire.checked-package/v2` wire contract (FR-322, FR-340 through FR-342), the FR-331 backend-provider envelope, AD-016 | Contract IR reads QSpec's contracts and cites them; it copies none of QSpec's files |
| QSL (`quire-spec-language`) | the compiler that produces checked packages, the owner views `predicate` and `temporal` read, and the `qsl-replay` crate: `Witness`, `ReplaySource`, `WitnessEnvelope`, `ReplayRequest`, `ReplayResult`, `TerminalValue`, `TerminalRecord`, `ObligationIdentity` and the replay facade `qsl_replay::replay` | QSL depends on `quire-contract-model`; the root crate depends on QSL at one pinned revision, names QSL's replay types and defines none of its own |
| codegen (`quire-contract-codegen`) | provider generation, bounded Kani harness emission, the family lowerings for checked arithmetic, collections and objects in its backend adapter, the Kani transcript parser in that adapter, and the replay adapter that builds QSL's envelope and calls `qsl_replay::replay` (QSL ADR-011 E9) | codegen depends on the root crate for the `kani` profile, input ABI, dispatch index, outcomes and provenance, and on `quire-contract-model` for model types |
| runtime (`quire-contract-runtime`) | the `no_std` support library generated oracles link | no dependency in either direction |

Downstream solvers, Quoin, build infrastructure, ambient repository
discovery and human release decisions remain outside.

### Kani boundary

The root crate's `kani` module holds the parts of the bounded-Kani boundary
every backend reads: the `kani-bounded/1` profile and capability matrix
(FR-029), the finite input ABI and typed outcome (FR-030), and the dispatch
index, module descriptors, provenance and outcome map (FR-031). The family
lowerings for checked arithmetic, collections and objects are
Kani-specific and belong to the backend adapter in codegen, which emits the
harness. The dispatch index names each family module and its declared
constructs; the lowering behind a module is codegen's.

A construct for which the profile has no qualified interpretation is settled
at negotiation: the capability matrix records it `unsupported`, its item
settles `unsupported` in the QSpec FR-331 `dispositions` vocabulary with a
warning naming the construct kind, and no artifact, `KaniOutcome` or
`TerminalValue` exists for it (FR-029). Resource exhaustion, cancellation and
an absent solver or backend are run outcomes only (FR-030).

### Replay ownership

The counterexample envelope, witness, replay source, FR-331 terminal record
and obligation identity are QSL's `qsl-replay` types, and Contract IR defines
none of them. The root crate has no `replay` or `witness` module, and
[FR-039](../interface/FR-039-root-crate-public-interface.md) "Items QSL owns"
names each item that therefore has no place in its interface. Kani
transcript parsing is the codegen backend adapter's.

Contract IR's part is the map from a Kani outcome to a QSL `TerminalValue`
(FR-031), and it is total over `KaniOutcome`. An `Unavailable` outcome
carries one of two cause codes, solver absent or backend absent (FR-030), and
maps to QSL's `Unsupported` value with the matching `UnavailabilityCause`.
The only `Inconclusive` outcome is a vacuous proof, a run whose obligation
completed with zero SUCCESS checks; it maps to QSL's `inconclusive` terminal
value carrying the vacuity cause (QSpec FR-331-AC-8) and is never recorded as
`proved`.

Replay runs only through `qsl_replay::replay`, from the codegen replay
adapter; Contract IR has no dependency on `quire_spec_language::runtime` and
calls no executor. The root crate takes `qsl-replay` from the QSL repository
and revision it already pins, so taking it adds no repository edge.

## Views

```text
JSON bytes -> wire model -> schema validation -> semantic validation
                                         |-> ordered diagnostics
validated package -> dependency walk -> canonical encoder -> SHA-256 identities
validated package + artifact traces -> shallow/deep/uncovered/orphaned coverage
checked-package/v2 bytes -> strict reader -> admitted package -> per-item lowering -> ContractPackage v1
output-mapping request -> mapper seam -> per-obligation records -> atomic package
checked owner views -> predicate projection -> explicit Boolean valuation
checked temporal + observations + valuations -> sibling native/TL requests
validated native/TL formula results -> structural correspondence join
exact campaign manifest -> typed graph -> descriptive model + strict re-export
checked clause + kani-bounded/1 + finite input -> dispatch index -> codegen family lowering -> KaniOutcome -> qsl_replay::TerminalValue
schema + corpus manifest + fixtures -> process runner -> JSON Lines results
```

The Rust library and process runner share semantic operations. `serde`
transports values; `sha2` computes identities; neither defines semantics.
Downstream engines depend on published types and bytes, not internal modules.

### Versioned contracts

Four contracts carry a version. Each is described here in its current form
only, and each refuses a version it does not know rather than interpreting
it.

| Contract | Version | Direction | Requirement |
| --- | --- | --- | --- |
| Contract IR wire schema | `contract-package-reference-v1.schema.json`, `schema_version` 1.1; crate version 0.1.0 | authored contract in, canonical JSON (`quire.contract.canonical-json/v1`) out | FR-011 through FR-020 |
| Checked package | `quire.checked-package/v2` only, parsed once from `contract_version`; every other version refuses `unknown_contract_version` | QSL-produced input | FR-038, FR-040 |
| ContractPackage | `quire.contract-ir.contract-package/v1`, holding `quire.contract-ir.lowered-node/v1` nodes | Contract IR output to backend providers | FR-035 |
| Bounded Kani profile | `kani-bounded/1`, whose profile revision, capability matrix, input ABI and module revisions are selected together | checked clause and finite input in, `KaniOutcome` out | FR-029 through FR-031 |

The checked package is the only input from which a `ContractPackage` is
built: its version selects the reader, and a lowered node's preimage names
the lowered-node version, not the checked-package version. The Kani profile
reads checked native clauses and finite inputs, not a `ContractPackage`; its
artifacts bind the checked-clause identity. The wire schema applies only to
the authored-contract interchange.

## Decisions

- Separate wire parsing from validated semantic construction.
- Use closed type/operator/form vocabularies and reject unknown variants.
- Canonicalize with explicit ordering rules instead of serializer defaults.
- Treat source spans as provenance excluded from expression equivalence but
  included in diagnostics and package-level provenance.
- Refuse a contract version the reader does not know; never interpret it.
- Keep owner integrations in the root crate and keep the model crate free of
  QSL, observation, protocol, TL, and self dependencies.
- Give every public item one owning crate and one import path. The model
  crate re-exports its items by name, as FR-019 lists them, with no glob
  re-export. The root crate re-exports nothing from the model: a root
  re-export would duplicate the model's whole public surface under a second
  path and would let two copies of the model crate coexist in one
  dependency graph. Consumers that need model types depend on
  `quire-contract-model` directly.
- Keep the shared bounded-Kani contract in Contract IR: the `kani-bounded/1`
  profile, the finite input ABI, the dispatch index, the typed outcome and
  provenance (`profile`, `abi`, `dispatch`, `outcome`, `provenance`), so any
  backend can read one versioned profile and input firewall. The
  Kani-specific family lowerings for checked arithmetic, collections and
  objects belong to codegen's backend adapter, which emits the harness.
- Map every Kani outcome kind to one existing QSL `TerminalValue`, with no
  new terminal value: `Unavailable` splits by its cause code into solver
  absence and backend absence, a vacuous proof maps to QSL's `inconclusive`
  value with the vacuity cause, and a construct the profile cannot interpret
  settles `unsupported` at negotiation, before any run, and never becomes an
  outcome.
- Construct and strict-read owner artifacts without parsing or evaluating an
  owner language inside Contract IR; preserve typed non-values at every join.
- Use QSL's `qsl-replay` types for every replay, witness, envelope,
  terminal-record and obligation-identity concept; define none locally.
- Keep ecosystem-model output observational: it may describe exact owner,
  dependency and evidence links or seed a proposal, but cannot become an owner
  admission, evaluator, evidence-acceptance or release-decision input.
- Keep `publish = false` through the human v0.1 decision.

## Risks

- Schema and Rust model drift: controlled by round-trip and schema mutation tests.
- Canonicalization ambiguity: controlled by golden bytes and property tests.
- Partial-operation unsoundness: controlled by definedness rules and negative fixtures.
- Orphan false coverage: controlled by exact revision identities and separate class.
- Owner-contract drift against QSpec and QSL: controlled by exact revision
  selections, constructor-private inputs, strict re-derivation, and QSpec
  vectors read at run time from a QSpec checkout rather than copied.
- Native/TL semantic disagreement: retained as a typed conflict rather than
  repaired, coerced, or hidden behind either owner's result vocabulary.
- Self-model authority confusion: controlled by exact external manifest
  selection, constructor-private checked views, absent acceptance fields, and
  no conversion from model/proposal output into executable owner inputs.
