---
id: AP-002
title: "Bounded Kani backend assurance profile"
type: AssuranceProfile
status: proposed
owner: kreneskyp
profile_version: 0.2
profile_kind: general
scope: one exact kani-bounded profile selection, finite input ABI, generated harness population, and native replay corpus
impact_assessments:
  - id: impact-invalid-population-erasure
    scenario: invalid or incomplete model population is assumed away and reported as proof
    severity: material
    verifiability:
      class: cheap-conclusive
      stochastic_dependency: none
    detect_before_harm:
      expected: true
      control_ref: ix://agent-ix/quire-contract-ir/MP-002
  - id: impact-bounded-semantic-substitution
    scenario: a scalar or bounded approximation claims unsupported object, graph, or collection semantics
    severity: material
    verifiability:
      class: cheap-conclusive
      stochastic_dependency: none
    detect_before_harm:
      expected: true
      control_ref: ix://agent-ix/quire-contract-ir/MP-002
  - id: impact-unreplayable-counterexample
    scenario: a Kani witness cannot reproduce through the selected native runtime
    severity: material
    verifiability:
      class: cheap-conclusive
      stochastic_dependency: none
    detect_before_harm:
      expected: true
      control_ref: ix://agent-ix/quire-contract-ir/MP-002
review_policy:
  mode: require
  operations: [spec-review, code-review, gap-analysis]
relationships:
  - target: ix://agent-ix/quire-contract-ir/AD-002
    type: references
---
# Bounded Kani backend assurance profile

## Decision Boundary

This profile qualifies only the exact bounded profile selection and declared finite corpus. It cannot prove a property over an undeclared or unbounded population, qualify an alternate Kani version/options set, or approve a source, release, solver, runtime, or downstream consumer.

## Impact Scenarios

The declared impacts require controls against population erasure, bounded semantic substitution, and unreplayable counterexamples.

## Evidence Policy

Evidence records profile/matrix and source/model identities, executable and generator digests, all options and assumptions, exact bounds, every typed outcome, native replay result, and review findings. Corpus parity requires explicit support/refusal/inconclusive classification and never aggregates a non-success result into pass.

## Exceptions

An exception names the omitted capability, affected profile/corpus, owner, rationale, expiry, and compensating evidence. It cannot turn refusal, inconclusive, unavailable, timeout, resource exhaustion, invalid input, or incomplete input into proof.
