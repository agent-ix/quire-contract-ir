---
type: master-requirements
name: quire-contract-ir
org: agent-ix
component_type: semantic-contract-library
implementation_language: rust-json-schema
tags: [contract-ir, contract-governance, provenance, assurance]
depends_on:
  - ix://agent-ix/quire-contract-ir/issues/1
standards_alignment: [iso-iec-ieee-29148]
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/1
    type: depends_on
    cardinality: "1:1"
security_critical: false
---
# Master Requirements Specification

## Purpose

This specification owns PGM-01 and the versioned semantic contract substrate
for the contract-derived verification program. Downstream code generation and
analysis consume this model and shall not invent parallel identity, expression,
or canonicalization semantics.

## Scope

### In Scope

- Wire-schema and crate compatibility, and release order.
- Licensing, third-party provenance, clean-room grammar, and contribution rules.
- Tool and artifact classification and release authority.
- The boundary between reusable qualification support and project decisions.
- Package, requirement, clause, anchor, type, expression, and dependency identity.
- Definedness, canonical encoding, stable digests, schema evolution, and orphan handling.
- The implemented checked-predicate and temporal-clause correspondence
  bridges from native Quire contracts to exact supported TL profiles.
- The bounded, non-authoritative export of the exact temporal-ecosystem
  component/object/interface/contract/evidence graph.
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
- A universal producer runner, common evidence envelope/store, or parallel
  human-decision mechanism.
- The eight downstream repository migrations.
- A validation, accreditation, certification, or release decision.

## System Overview

### System Description

PGM-01 is the repository-independent governance boundary. It assigns static
definition export to Quire, domain execution/results to native producers,
retention/audit/reporting to Quoin, and human decisions to ix-flow. The semantic contract
IR is a versioned, implementation-language-independent model whose Rust API,
JSON representation, diagnostics, canonical encoding, and conformance corpus
share one normative specification. Quire and Quoin remain non-executing.

### Intended Users

Workstream authors consume the policy by reference. Domain tools emit native
structured results. Reviewers verify compatibility, provenance, and evidence limitations.
The named human release owner decides whether an exact candidate may be tagged.

## Requirements Architecture

The canonical policy owns PGM-01-R01 through PGM-01-R10, of which R08 is
withdrawn. Discrete requirements FR-001, FR-003 through FR-007 and
FR-010 provide traceable artifact identities without redefining that policy.
FR-008 carried the withdrawn R08 derivation-evidence envelope and is deleted
with it; the identifier is not reused. TM-001 maps them to automated tests or retained inspection.
Typed review, plan, task, assurance, and gap artifacts preserve the spec-first
workflow. StR-001 through StR-003, FR-011 through FR-020, FR-023, FR-025, and
FR-026 through FR-028, alongside NFR-001 through NFR-005, define the v0.1 semantic substrate.
FR-029 through FR-031 define the bounded-Kani extension boundary: the
profile, finite input ABI, dispatch index and typed outcome. The
family lowerings behind the dispatch index are the codegen backend adapter's.
Profile selection, the finite input/outcome firewall and module dispatch are
implemented against the integrated codegen corpus.
That implementation applies only to its exact selected finite profile and
does not qualify unbounded source semantics or another Kani/options
selection. Each Kani outcome maps to one QSL `qsl_replay::TerminalValue`
(FR-031-AC-5): `Unavailable` by its solver-absent or backend-absent cause
code (FR-030) to `Unsupported`, and a vacuous proof to
`Proved { success_checks: 0 }`, which QSL reads as inconclusive with the
vacuity cause. A construct the profile cannot interpret
settles `unsupported` at negotiation and produces no outcome (FR-029).
Counterexample replay is not a Contract IR operation: the envelope, witness,
replay source, terminal record and obligation identity are QSL's `qsl-replay`
types, and the codegen replay adapter replays through `qsl_replay::replay`.
TC-221, which verified the witness extraction retired with FR-031-AC-4, is
withdrawn; TC-222 verifies the FR-344 refusals and TC-223 the outcome map.
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
the ADR-002 2.0.0 members the V2 wire does not yet carry. FR-019 lists the
model crate's public items and FR-039 the root crate's; the root crate
re-exports no model item, so each public item has one import path. AD-001 is the repository architecture and
versioning description; AD-002 and AD-003 are views inside it.
STD-001 is the stable diagnostic code registry and STD-003 the closed
output-mapping refusal registry. ADR-0053 fixes the formal clause source
profiles. ADR-0054 separates archetype
datatype generation from optional formal type projection, while ADR-0055
sets the Rust compiler baseline. ADR-0056
fixes the subsystem specification layout and registry format that this
repository, Contract Codegen and Contract Runtime follow.
TM-002 maps the substrate to staged verification.

FR-025 implements the checked-predicate-to-Boolean-signal correspondence;
FR-026 implements the native-temporal correspondence boundary. FR-027
implements bounded non-authoritative observational model export, and
FR-028 makes the model/owner/root dependency graph implementable without a
Cargo cycle. None makes TL a user-authored Quire language or grants model output
authority over owner inputs, execution, evidence acceptance, or release.

## References

- [Program umbrella](https://github.com/agent-ix/quire-contract-ir/issues/1).
- [PGM-01 issue](https://github.com/agent-ix/quire-contract-ir/issues/3).
- [Canonical PGM-01 policy](program/PGM-01-governance.md).
- [Contract IR epic](https://github.com/agent-ix/quire-contract-ir/issues/11).
- [Contract IR test matrix](contract-test-matrix.md).
- [Contract IR diagnostic registry](contract/STD-001-diagnostic-registry.md).
- [Native predicate to TL Boolean projection](contract/FR-025-native-predicate-tl-projection.md).
- [Versioned bounded Kani profile](contract/FR-029-versioned-bounded-kani-profile.md).
- [Bounded Kani domains and outcomes](contract/FR-030-bounded-kani-domain-and-outcomes.md).
- [Bounded Kani dispatch and terminal-value map](contract/FR-031-bounded-kani-dispatch-and-terminal-map.md).
- [Output-mapping request admission](contract/FR-032-admit-output-mapping-request.md).
- [Per-obligation output accounting](contract/FR-033-account-for-output-obligations.md).
- [Atomic output package assembly](contract/FR-034-assemble-output-package-atomically.md).
- [Output-mapping foundation test case](contract/TC-043-output-mapping-foundation.md).
- [Complete-V1 backend delivery architecture](assurance/AD-003-complete-v1-backend-delivery.md).
- [Complete-V1 ContractPackage lowering](contract/FR-035-complete-v1-contract-package-lowering.md).
- [CheckedPackage V2 consumption](contract/FR-038-consume-checked-package-v2.md).
- [Exact backend negotiation](contract/FR-036-exact-backend-negotiation-and-emission.md).
- [Canonical backend replay](contract/FR-037-canonical-backend-replay-and-qualification.md).
- [Native temporal to TL correspondence](contract/FR-026-native-temporal-tl-correspondence.md).
- [Bounded temporal ecosystem model export](contract/FR-027-export-bounded-temporal-ecosystem-model.md).
- [Cycle-free Contract IR model package](contract/FR-028-separate-cycle-free-contract-model.md).
- [Repository architecture and versioning](assurance/AD-001-contract-ir-architecture.md).
- [Bounded Kani backend architecture](assurance/AD-002-bounded-kani-architecture.md).
- [Root crate public interface](interface/FR-039-root-crate-public-interface.md).
- [Frame entries, operation anchors and state clauses](contract/FR-040-admit-frame-entries-and-state-clauses.md).
- [Unadmitted ADR-002 2.0.0 members](contract/FR-344-admit-or-refuse-the-adr-002-2-0-0-members.md).
- [Output-mapping refusal registry](contract/STD-003-output-mapping-refusal-registry.md).
- [Formal clause source profiles](decisions/ADR-0053-formal-clause-source-profiles.md).
- [Subsystem spec layout and registry format](decisions/ADR-0056-spec-layout-convention.md).
- [Unadmitted ADR-002 member refusal test case](contract/TC-222-refuse-unadmitted-adr-002-members.md).
- [Kani outcome to QSL terminal value test case](contract/TC-223-kani-outcome-fr331-result-map.md).
- [Root crate interface test case](contract/TC-055-root-crate-public-interface.md).
- [Frame entry and state clause test case](contract/TC-056-checked-package-v2-frame-entries-and-state-clauses.md).
- [Model crate interface test case](contract/TC-058-model-crate-public-interface.md).
