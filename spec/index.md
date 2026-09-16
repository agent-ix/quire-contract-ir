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

This specification owns PGM-01, the shared-assurance ownership boundary, and the versioned semantic contract substrate
for the contract-derived verification program. Downstream code generation and
analysis consume this model and shall not invent parallel identity, expression,
or canonicalization semantics.

## Scope

### In Scope

- Wire-schema and crate compatibility, dependency pins, and release order.
- Licensing, third-party provenance, clean-room grammar, and contribution rules.
- Tool and artifact classification, domain-result provenance, shared assurance
  ownership, and release authority.
- The boundary between reusable qualification support and project decisions.
- Package, requirement, clause, anchor, type, expression, and dependency identity.
- Definedness, canonical encoding, stable digests, schema evolution, and orphan handling.
- The implemented checked-predicate and temporal-clause compatibility bridges
  from native Quire contracts to exact supported TL profiles.
- The bounded, non-authoritative export of the exact temporal-ecosystem
  component/object/interface/contract/evidence graph.
- The cycle-free semantic-model package and compatibility bridge boundary.
- Public Rust, serialized JSON, and conformance-runner interfaces.
- A versioned bounded-Kani profile boundary for finite checked native clauses,
  including explicit support/refusal/inconclusive capability classification,
  finite input validation, typed outcomes, provenance, and native replay.
- A target-neutral output-mapping request, per-obligation loss record, bounded
  mapper seam, atomic generated-package assembler, and downstream observer reference.

### Out of Scope

- Native/TL source parsing, semantic evaluation, rewriting, code generation,
  solver execution or production monitoring.
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

The canonical policy owns PGM-01-R01 through PGM-01-R11, of which R08 is
withdrawn. Discrete requirements FR-001 through FR-007, FR-009, FR-010, FR-021,
and FR-022 provide traceable artifact identities without redefining that policy.
FR-008 carried the withdrawn R08 derivation-evidence envelope and is deleted
with it; the identifier is not reused. TM-001 maps them to automated tests or retained inspection.
Typed review, plan, task, assurance, and gap artifacts preserve the spec-first
workflow. StR-001 through StR-003, FR-011 through FR-020, FR-023, FR-025, and
FR-026 through FR-028, alongside NFR-001 through NFR-005, define the v0.1 semantic substrate.
FR-029 through FR-031 define the bounded-Kani extension boundary implemented by
Contract IR PRs #88 through #92 and the integrated codegen corpus at
`73c82ad`. That implementation applies only to its exact selected finite
profile and does not qualify unbounded source semantics or another Kani/options
selection.
FR-032 through FR-034 define the cycle-free target-neutral FS06 coordinator,
record, and atomic package foundation. They implement accepted QSpec AD-004 and
FR-120/121/125/269/297/298/299 without implementing any target-specific
correspondence or admitting generated target text as source.
FR-035 through FR-037 adopt QSpec AD-010 and FR-195 through FR-197 for the
complete-V1 target-neutral ContractPackage, exact provider negotiation, and
canonical native replay. They define the producer-before-consumer contract for
Contract IR #100, runtime #16, codegen #48 through #50, and #101; they do not
reopen the closed bounded-Kani profile or claim target-specific mapping support.
FR-038 consumes the QSpec I04 `quire.checked-package/v2` contract through an
explicit version dispatcher, strict V2 reader, typed V1-to-V2 migration outcome
and per-item V2 lowering, without widening the frozen FR-035 V1 reader.
STD-001 is the stable diagnostic code registry. ADR-0054 separates archetype
datatype generation from optional formal type projection, while ADR-0055
separates and pins the supported Rust minimum and qualification compiler.
TM-002 maps the substrate to staged verification.

FR-025 implements the checked-predicate-to-Boolean-signal correspondence;
FR-026 implements the native-temporal correspondence boundary. FR-027
implements bounded non-authoritative observational model export, and
FR-028 makes the owner/bridge dependency graph implementable without a Cargo
cycle. None makes TL a user-authored Quire language or grants model output
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
- [Bounded Kani dispatch, replay, and provenance](contract/FR-031-bounded-kani-dispatch-replay-provenance.md).
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
