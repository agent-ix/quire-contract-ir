---
type: master-requirements
name: quire-contract-ir
org: agent-ix
component_type: governance-contract
implementation_language: markdown-json-schema
tags: [contract-governance, provenance, assurance]
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

This specification owns PGM-01, the common governance contract for the eight
repositories in the contract-derived verification and temporal assurance
program. Workstream specifications reference PGM-01 and shall not redefine it.

## Scope

### In Scope

- Wire-schema and crate compatibility, dependency pins, and release order.
- Licensing, third-party provenance, clean-room grammar, and contribution rules.
- Tool and artifact classification, evidence identity, and release authority.
- The boundary between reusable qualification support and project decisions.

### Out of Scope

- The semantic contract IR owned by issues #5, #6, #8, #9, and #10.
- Runtime, code generation, solver, parser, rewrite, or evaluator behavior.
- A validation, accreditation, certification, or release decision.

## System Overview

### System Description

PGM-01 is a repository-independent policy and a strict JSON evidence-envelope
contract. It makes transformation identities inspectable while retaining a
human decision boundary.

### Intended Users

Workstream authors consume the policy by reference. Tool authors emit the
envelope. Reviewers verify compatibility, provenance, and evidence limitations.
The named human release owner decides whether an exact candidate may be tagged.

## Requirements Architecture

The canonical policy owns PGM-01-R01 through PGM-01-R10. Discrete requirements
FR-001 through FR-010 provide traceable artifact identities without redefining
that policy. TM-001 maps them to automated tests or retained inspection.
Typed review, plan, task, and gap artifacts preserve the spec-first workflow.

## References

- [Program umbrella](https://github.com/agent-ix/quire-contract-ir/issues/1).
- [PGM-01 issue](https://github.com/agent-ix/quire-contract-ir/issues/3).
- [Canonical PGM-01 policy](program/PGM-01-governance.md).
