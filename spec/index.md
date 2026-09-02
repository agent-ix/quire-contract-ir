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
- Public Rust, serialized JSON, and conformance-runner interfaces.

### Out of Scope

- Runtime, code generation, solver, parser, rewrite, or evaluator behavior.
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
workflow. StR-001 through StR-003, FR-011 through FR-020, and NFR-001 through
NFR-004 define the v0.1 semantic substrate. STD-001 is the stable diagnostic
code registry, and TM-002 maps the substrate to staged verification.

## References

- [Program umbrella](https://github.com/agent-ix/quire-contract-ir/issues/1).
- [PGM-01 issue](https://github.com/agent-ix/quire-contract-ir/issues/3).
- [Canonical PGM-01 policy](program/PGM-01-governance.md).
- [Contract IR epic](https://github.com/agent-ix/quire-contract-ir/issues/11).
- [Contract IR test matrix](contract-test-matrix.md).
- [Contract IR diagnostic registry](contract/STD-001-diagnostic-registry.md).
