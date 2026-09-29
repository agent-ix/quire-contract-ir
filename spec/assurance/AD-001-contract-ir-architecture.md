---
id: AD-001
title: "quire-contract-ir repository architecture and versioning"
type: ArchitectureDescription
status: proposed
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

The workspace holds two crates.

`quire-contract-model` is the cycle-free semantic substrate. It owns
unvalidated wire values, validated semantic values, identity and source-span
types, type and definedness checking, dependency derivation,
canonicalization, digests, version migration, coverage classification, the
conformance registries, the target-neutral output-mapping foundation
(FR-032 through FR-034), and the `quire.checked-package/v2` reader and
lowerer (FR-038, FR-040) with the `ContractPackage` it emits (FR-035). It has
no dependency on QSL, Quire Observation, Quire Protocol, tl-syntax, tl-mltl
or `quire-contract-ir` (FR-028).

`quire-contract-ir` is the root crate. It re-exports the model (FR-019) and
adds the owner integrations that need other owners' crates (FR-039):

| Module | Owns | Requirement |
| --- | --- | --- |
| `bridge` | shared owner-bridge errors, digests, limits and contract selections | FR-025, FR-026 |
| `predicate` | native checked predicate to TL Boolean signal projection | FR-025 |
| `temporal` | native temporal clause to TL formula correspondence and result join | FR-026 |
| `ecosystem_model` | bounded, non-authoritative temporal-ecosystem model export | FR-027 |
| `kani` | the `kani-bounded/1` profile, finite input firewall, dispatch index, provenance, family lowerings, and the Kani outcome to QSL terminal-value map | FR-029 through FR-031 |

The conformance runner (`src/bin/`) and its fixtures stay in the root
package.

### What each owner holds

| Owner | Holds | Contract IR's relation |
| --- | --- | --- |
| QSpec (`quire-specification`) | the normative language, the `quire.checked-package/v2` wire contract (FR-322, FR-340 through FR-342), the FR-331 backend-provider envelope, AD-016 | Contract IR reads QSpec's contracts and cites them; it copies none of QSpec's files |
| QSL (`quire-spec-language`) | the compiler that produces checked packages, the owner views `predicate` and `temporal` read, and the `qsl-replay` crate: `Witness`, `ReplaySource`, `WitnessEnvelope`, `ReplayRequest`, `ReplayResult`, `TerminalValue`, `TerminalRecord`, `ObligationIdentity` and the replay facade `qsl_replay::replay` | the root crate depends on QSL at one pinned revision; it names QSL's replay types and defines none of its own |
| codegen (`quire-contract-codegen`) | provider generation, bounded Kani harness emission, the Kani transcript parser in its backend adapter, and the replay adapter that builds QSL's envelope and calls `qsl_replay::replay` (QSL ADR-011 E9) | codegen depends on the root crate for `kani` lowerings and outcomes |
| runtime (`quire-contract-runtime`) | the `no_std` support library generated oracles link | no dependency in either direction |

Downstream solvers, Quoin, build infrastructure, ambient repository
discovery and human release decisions remain outside.

### Replay ownership

The counterexample envelope, witness, replay source, FR-331 terminal record
and obligation identity are QSL's `qsl-replay` types, and Contract IR defines
none of them. [FR-039](../interface/FR-039-root-crate-public-interface.md)
"Items QSL owns" names each root-crate item that therefore has no place in
the interface; at this revision `src/kani/replay.rs` and `src/kani/witness.rs`
still define them. Kani transcript parsing is the codegen backend adapter's.
Contract IR's part is the map from a Kani outcome to a QSL `TerminalValue`
for the outcomes whose target is decided (FR-031); the others are OQ-3.
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
checked clause + kani-bounded/1 + finite input -> dispatch -> lowering -> KaniOutcome -> qsl_replay::TerminalValue (decided outcomes)
schema + corpus manifest + fixtures -> process runner -> JSON Lines results
```

The Rust library and process runner share semantic operations. `serde`
transports values; `sha2` computes identities; neither defines semantics.
Downstream engines depend on published types and bytes, not internal modules.

### Versioned contracts

Four contracts are versioned independently. A change to one never implies a
version change in another, and each refuses a version it does not know rather
than interpreting it.

| Contract | Version | Direction | Requirement |
| --- | --- | --- | --- |
| Contract IR wire schema | `contract-package-reference-v1.schema.json`, `schema_version` 1.1 current, 1.0 migrated one-way to 1.1; crate version 0.1.0 | authored contract in, canonical JSON (`quire.contract.canonical-json/v1`) out | FR-011 through FR-020 |
| Checked package | `quire.checked-package/v2` only, parsed once from `contract_version`; every other version refuses `unknown_contract_version` | QSL-produced input | FR-038, FR-040 |
| ContractPackage | `quire.contract-ir.contract-package/v1`, holding `quire.contract-ir.lowered-node/v1` nodes | Contract IR output to backend providers | FR-035 |
| Bounded Kani profile | `kani-bounded/1`, whose profile revision, capability matrix, input ABI and module revisions are selected together | checked clause and finite input in, `KaniOutcome` out | FR-029 through FR-031 |

The checked package is the only input from which a `ContractPackage` is
built: its version selects the reader, and a lowered node's preimage names
the lowered-node version, not the checked-package version. The Kani profile
reads checked native clauses and finite inputs, not a `ContractPackage`; its
artifacts bind the checked-clause identity. The wire schema and its
migration apply only to the authored-contract interchange.

## Decisions

- Separate wire parsing from validated semantic construction.
- Use closed type/operator/form vocabularies and reject unknown variants.
- Canonicalize with explicit ordering rules instead of serializer defaults.
- Treat source spans as provenance excluded from expression equivalence but
  included in diagnostics and package-level provenance.
- Keep migration explicit and one-way; never interpret unknown majors.
- Keep owner integrations in the root crate and keep the model crate free of
  QSL, observation, protocol, TL, and self dependencies.
- Construct and strict-read owner artifacts without parsing or evaluating an
  owner language inside Contract IR; preserve typed non-values at every join.
- Use QSL's `qsl-replay` types for every replay, witness, envelope,
  terminal-record and obligation-identity concept; define none locally.
- Keep ecosystem-model output observational: it may describe exact owner,
  dependency and evidence links or seed a proposal, but cannot become an owner
  admission, evaluator, evidence-acceptance or release-decision input.
- Keep `publish = false` through the human v0.1 decision.

## Open questions

### OQ-1: The root crate's model re-export

`src/lib.rs` describes the root crate as a "Compatibility bridge", and both
crates re-export whole modules with `pub use …::*`: `pub use
quire_contract_model::*` in the root, and seven globs in the model
(`binding`, `canonical`, `checked_package`, `coverage`, `expression`,
`identity`, `output_mapping`) beside two explicit lists (`conformance`,
`limits`).

- **(a) Keep the glob re-exports** and reword the doc comment to "owner
  integrations over the model". Cheapest; the public surface stays whatever
  the modules happen to make `pub`, so FR-019 and FR-039 inventories remain
  hard to check.
- **(b) Replace every glob with an explicit list** in both crates and drop
  the "compatibility" wording. The surface becomes exactly what FR-019 and
  FR-039 list, and an accidental `pub` no longer leaks.
- **(c) Remove the root re-export**, so consumers import model types from
  `quire-contract-model` directly. Cleanest ownership, but it changes import
  paths in codegen and in QSL, which already depends on the model crate under
  the `quire-contract-ir` package alias.

**Recommendation: (b).** It makes the interface requirements checkable with
no cross-repo change; (c) can follow once consumers are ready.

### OQ-2: Placement of the Kani-specific lowerings

`src/kani/{arithmetic,collections,objects,dispatch,abi}.rs` are consumed by
codegen's `bounded_kani_corpus.rs`. They are Kani-backend-specific, while
AD-003 gives codegen the bounded Kani harnesses and the transcript parser
is codegen's.

- **(a) Keep all five in Contract IR.** No move; Contract IR stays the owner
  of a backend-specific lowering.
- **(b) Move all five to codegen's backend adapter.** Contract IR keeps only
  target-neutral output; the `kani-bounded/1` profile and finite-input
  firewall then have no shared owner.
- **(c) Split:** keep the profile, finite-input ABI and dispatch index
  (`profile.rs`, `abi.rs`, `dispatch.rs`, `outcome.rs`, `provenance.rs`) in
  Contract IR as the shared versioned contract, and move the family lowerings
  (`arithmetic.rs`, `collections.rs`, `objects.rs`) to codegen.

**Recommendation: (c).** The versioned profile and input firewall stay a
contract any backend can read, and the Kani-specific lowering moves to the
backend that emits the harness.

### OQ-3: Kani outcome kinds with no QSL terminal value

QSpec FR-331 records `unsupported` for an `Unavailable` run with a cause
naming the absent solver or backend, and admits `inconclusive` as a result.
QSL's `TerminalValue` needs an `UnavailabilityCause` of `SolverAbsent` or
`BackendAbsent` that `KaniOutcomeKind::Unavailable` does not distinguish,
and has no arm for a non-vacuous `inconclusive`, which a capability-declared
`Inconclusive` disposition produces.

- **(a) Extend QSL's `TerminalValue`** with an `inconclusive` arm carrying a
  typed cause, and split Contract IR's `Unavailable` by its cause code into
  solver or backend absence.
- **(b) Remove the `Inconclusive` capability disposition** from
  `kani-bounded/1` and map every `Unavailable` to `BackendAbsent`.

**Recommendation: (a).** It is what FR-331 already states; the
`TerminalValue` change is QSL's to make.

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
