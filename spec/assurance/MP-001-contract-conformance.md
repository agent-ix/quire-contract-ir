---
id: MP-001
title: "Contract IR conformance measurement plan"
type: MeasurementPlan
status: proposed
owner: kreneskyp
metric: contract_ir_conformance
definition_version: quire-contract-ir.measurement/v3
stage: gate
ground_truth_kind: mechanical
objective:
  direction: higher
  bound: 1.0
statistical_design:
  population: every manifest-listed valid and invalid v0.1 fixture plus generated canonicalization properties
  sampling: exhaustive fixture execution and deterministic seeded property cases
  repetitions: 2
  estimator: proportion
  error_model: schema/model drift, platform serialization differences, fixture omission, and test-harness defects
  uncertainty: report each case and repetition; no aggregate hides an invalid, skipped, or inconclusive case
  decision_rule:
    comparator: ge
    threshold: 1.0
protected_apparatus:
  - Makefile
  - src/bin/quire-contract-conformance.rs
  - crates/quire-contract-model/src/conformance.rs
  - schemas/conformance-trace-map-v1.json
  - scripts/generate_conformance_corpus.py
  - corpus/contract-v0.1/manifest.json
  - corpus/contract-v0.1/inventory.json
  - corpus/contract-v0.1/inputs/**
  - corpus/contract-v0.1/expectations/**
  - corpus/contract-v0.1/canonical/**
  - corpus/contract-v0.1/schemas/**
negative_controls:
  - kind: apparatus-edit
    description: >-
      the Makefile recipe, the runner binary and the model crate's
      conformance module that reads the manifest, derives observed coverage
      and compares expectations, the embedded coverage-to-criterion trace map,
      the corpus-generating script, and every manifest, inventory, input,
      expectation, canonical byte file and schema are protected, so editing
      one alongside the change it grades changes the recorded digests; the
      manifest also pins the SHA-256 of each schema, inventory, input,
      expectation and canonical byte file, and the runner refuses a corpus
      file whose bytes no longer match its pinned digest
  - kind: suppressed-observation
    description: >-
      the runner derives observable coverage from each fixture's declarative
      input and actual result and rejects an unobserved `covers` token before
      comparing expectations, so a run that exercises fewer manifest-listed
      fixtures or boundary tokens than declared is visible rather than passing
  - kind: selective-reporting
    description: >-
      the complete conformance runner is required to run twice
      (`statistical_design.repetitions: 2`) and exits non-zero when any
      fixture mismatches, so a clean repetition cannot be reported in place of
      one that produced an unmatched expectation
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

The `proportion` estimate is exactly matched expectations divided by declared
expectations. A declared fixture that is missing, or a canonical byte or digest
that drifts from its golden, is an unmatched expectation. The candidate gate
fails when `statistical_design.decision_rule` does not hold, and it also fails
on any requirement criterion without a backing test or any unresolved blocking
review finding; those two conditions are checked alongside the ratio because
they are not expectations and never enter its denominator.

The target is 100% expectation match, 100% trace backing, zero orphan false
coverage, zero public panic, and zero unresolved blocking finding. A skipped
platform, unavailable external service, or disabled CI remains explicit and
cannot be inferred as success. Only `@kreneskyp` decides sufficiency.
