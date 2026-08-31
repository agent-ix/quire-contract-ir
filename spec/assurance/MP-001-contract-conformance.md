---
id: MP-001
title: "Contract IR conformance measurement plan"
type: MeasurementPlan
status: proposed
owner: kreneskyp
metric: contract_ir_conformance
definition_version: quire-contract-ir.measurement/v1
stage: gate
statistical_design:
  population: every manifest-listed valid and invalid v0.1 fixture plus generated canonicalization properties
  sampling: exhaustive fixture execution and deterministic seeded property cases
  repetitions: 2
  estimator: exact matched expectations divided by declared expectations
  error_model: schema/model drift, platform serialization differences, fixture omission, and test-harness defects
  uncertainty: report each case and repetition; no aggregate hides an invalid, skipped, or inconclusive case
  decision_rule: any mismatch, missing fixture, digest drift, unbound criterion, or blocking review finding fails the candidate gate
relationships:
  - target: ix://agent-ix/quire-contract-ir/AP-001
    type: measures
---
# Contract IR conformance measurement plan

## Decision Use

Measurements inform whether a named candidate is ready for human source-release
review. They do not approve a tag, publish a crate, validate a downstream tool,
or accredit a consuming project.

## Population

The population is every manifest-listed positive, malformed, boundary,
revision, orphan, short-circuit, and partial-operation fixture; every golden
canonical byte/digest/dependency expectation; all requirement-tagged unit,
integration, property, and mutation tests; and all typed specification and
assurance artifacts.

## Collection Procedure

Run formatting, clippy, Rust/Python tests, license and unsafe audits, Quire
validation/coverage, schema mutations, the complete conformance runner twice,
cross-platform golden comparisons when remote CI is deliberately dispatched,
code review, and gap analysis. Retain exact subject, commands, tool/environment
identities, per-case outputs, checksum graph, findings, and limitations.

## Interpretation

The target is 100% expectation match, 100% trace backing, zero orphan false
coverage, zero public panic, and zero unresolved blocking finding. A skipped
platform, unavailable external service, or disabled CI remains explicit and
cannot be inferred as success. Only `@kreneskyp` decides sufficiency.
