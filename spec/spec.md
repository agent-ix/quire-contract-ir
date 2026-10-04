---
type: master-requirements
name: quire-contract-ir
org: agent-ix
component_type: semantic-contract-library
implementation_language: rust-json-schema
tags: [contract-ir]
standards_alignment: [iso-iec-ieee-29148]
security_critical: false
---
# Master Requirements Specification

## Purpose

This specification owns the versioned semantic contract substrate
for the contract-derived verification program. Downstream code generation and
analysis consume this model and shall not invent parallel identity, expression,
or canonicalization semantics.

## Scope

### In Scope

- Package, requirement, clause, anchor, type, expression, and dependency identity.
- Definedness, canonical encoding, stable digests, schema evolution, and orphan handling.
- The cycle-free `quire-contract-model` package and the `quire-contract-ir`
  root crate, each with its own public item list and no item under two paths.
- Public Rust, serialized JSON, and conformance-runner interfaces.
- A versioned bounded-Kani profile boundary for finite checked native clauses,
  including explicit supported/refused/unsupported capability classification,
  finite input validation, the dispatch index, typed outcomes, and
  the map from each outcome to its QSL terminal value.
- A target-neutral output-mapping request, per-obligation loss record, bounded
  mapper seam, atomic generated-package assembler, and downstream observer reference.

### Out of Scope

- Native/TL source parsing, semantic evaluation, rewriting, code generation,
  solver execution or production monitoring.
- The Kani family lowerings for checked arithmetic, collections and objects,
  which are the codegen backend adapter's.

## System Overview

### System Description

The semantic contract
IR is a versioned, implementation-language-independent model whose Rust API,
JSON representation, diagnostics, canonical encoding, and conformance corpus
share one normative specification.

### Intended Users

Code-generation, analysis and runtime maintainers consume the model. Domain
tools emit native structured results.

## Requirements Architecture

StR-001 through StR-003, FR-011 through FR-020, FR-023 and FR-028, alongside
NFR-001 through NFR-003, define the semantic substrate.
FR-029 through FR-031 define the bounded-Kani extension boundary: the
profile, finite input ABI, dispatch index and typed outcome. The
family lowerings behind the dispatch index are the codegen backend adapter's.
Profile selection, the finite input/outcome firewall and module dispatch are
implemented against the integrated codegen corpus.
That implementation applies only to its exact selected finite profile and
does not qualify unbounded source semantics. The map from a Kani outcome to a QSL `qsl_replay::TerminalValue` is owned by
`agent-ix/quire-contract-codegen` (tracked there under Linear IR-358); Contract
IR exposes only its own Kani outcome types (FR-031-AC-5 is retired). A construct the profile cannot interpret
settles `unsupported` at negotiation and produces no outcome (FR-029).
Counterexample replay is not a Contract IR operation: the envelope, witness,
replay source, terminal record and obligation identity are QSL's `qsl-replay`
types, and the codegen replay adapter replays through `qsl_replay::replay`.
TC-222 verifies the FR-344 refusals and TC-223 the outcome constructors.
FR-032 through FR-034 define the cycle-free target-neutral FS06 coordinator,
record, and atomic package foundation. They implement accepted QSpec AD-004 and
FR-120/121/125/269/297/298/299 without implementing any target-specific
correspondence or admitting generated target text as source.
FR-035 through FR-037 adopt QSpec AD-010 and FR-195 through FR-197 for the
complete-V1 target-neutral ContractPackage, exact provider negotiation, and
the rule that backend counterexamples reach replay only through QSL's
envelope. They do not reopen the closed bounded-Kani profile or claim
target-specific mapping support.
FR-038 consumes the QSpec I04 `quire.checked-package/v2` contract through a
strict reader that parses `contract_version` once, admits only
`quire.checked-package/v2`, re-derives every package and nominal node
identity, and lowers admitted items independently per request. FR-040 admits
QSpec FR-340's frame `modifies` entries, FR-342's operation anchors and
FR-341's state clause and parameter bodies in that reader, and FR-344 refuses
the ADR-002 2.0.0 members the V2 wire does not yet carry. FR-346 admits
QSpec FR-451's abstraction relation body in that reader. FR-019 lists the
model crate's public items and FR-039 the root crate's; the root crate
re-exports no model item, so each public item has one import path. AD-001 is the repository architecture and
versioning description; AD-002 and AD-003 are views inside it.
STD-001 is the stable diagnostic code registry and STD-003 the closed
output-mapping refusal registry. ADR-0053 fixes the formal clause source
profiles. ADR-0054 separates archetype
datatype generation from optional formal type projection. ADR-0056
fixes the subsystem specification layout and registry format that this
repository, Contract Codegen and Contract Runtime follow.
The subsystem matrices, indexed by [tests.md](tests.md), map the substrate to
staged verification.

FR-028 makes the model/owner/root dependency graph implementable without a
Cargo cycle.

## Subsystems

### Subsystem Registry

| Subsystem | Path | Role | Owning crates/modules | ADs | Owner |
| --- | --- | --- | --- | --- | --- |
| Core | `spec/core/` | Stakeholder needs, package, anchor, clause and dependency identity, resource limits, the diagnostic code registry, and the determinism, portability and diagnostic-integrity properties every subsystem shares | `quire-contract-model::identity`, `quire-contract-model::limits` | AD-001, ADR-0056 | Contract IR lane |
| Model | `spec/model/` | The type system, expression semantics, definedness, canonical encoding and digests, version and orphan handling, executable-projection binding, the model crate's public interface and the cycle-free model/root package split | `quire-contract-model::expression`, `quire-contract-model::canonical`, `quire-contract-model::wire`, `quire-contract-model::coverage`, `quire-contract-model::binding` | AD-001, AD-005, ADR-0053, ADR-0054 | Contract IR lane |
| Conformance | `spec/conformance/` | The conformance corpus and the JSON conformance interface | `quire-contract-model::conformance`, `quire-contract-ir` binary `quire-contract-conformance` | AD-001 | Contract IR lane |
| Checked package | `spec/checked_package/` | The strict `quire.checked-package/v2` reader, its frame, anchor and state-clause admission, the refusal of unadmitted ADR-002 members, and complete-V1 ContractPackage lowering from that input | `quire-contract-model::checked_package` | AD-001, AD-003, AD-004 | Contract IR lane |
| Output mapping | `spec/output_mapping/` | The target-neutral output-mapping request, per-obligation record, mapper seam, atomic package assembly and the refusal code registry | `quire-contract-model::output_mapping` | AD-001 | Contract IR lane |
| Kani | `spec/kani/` | The versioned bounded-Kani profile, finite input and outcome firewall, dispatch index and typed outcomes, provider-side exact backend negotiation, the root crate's public interface and the rule that Contract IR defines no replay type | `quire-contract-ir` root crate (`lib.rs` and `kani`) excluding the conformance binary | AD-001, AD-002, AD-003, AD-006 | Contract IR lane |

## References

- [Contract IR test matrix index](tests.md).
- [Complete-V1 backend delivery architecture](assurance/AD-003-complete-v1-backend-delivery.md).
- [Repository architecture and versioning](assurance/AD-001-contract-ir-architecture.md).
- [Bounded Kani backend architecture](assurance/AD-002-bounded-kani-architecture.md).
- [QSpec to IR checked-package seam](assurance/AD-004-checked-package-seam.md).
- [Formal clause source profiles](decisions/ADR-0053-formal-clause-source-profiles.md).
- [Subsystem spec layout and registry format](decisions/ADR-0056-spec-layout-convention.md).
- [IR to QSL seam: what QSL takes from the model crate](assurance/AD-005-qsl-consumption-seam.md).
- [IR to codegen seam: what codegen consumes](assurance/AD-006-codegen-consumption-seam.md).
