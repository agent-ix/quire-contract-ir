---
id: MP-002
title: "Bounded Kani support and replay parity measurement"
type: MeasurementPlan
status: proposed
owner: kreneskyp
metric: bounded_kani_profile_parity
definition_version: quire-contract-ir.bounded-kani-parity/v2
stage: gate
objective:
  direction: higher
  bound: 1.0
statistical_design:
  population: every declared profile-matrix entry and every manifest-listed valid, invalid, incomplete, resource-boundary, counterexample, and replay corpus case
  sampling: exhaustive
  repetitions: 2
  estimator: proportion
  error_model: profile drift, input-validation omission, generator drift, Kani/tool variation, and native replay disagreement
  uncertainty: each unavailable, timeout, exhausted, cancelled, refused, invalid, incomplete, or inconclusive case remains separately reported
  decision_rule:
    comparator: ge
    threshold: 1.0
relationships:
  - target: ix://agent-ix/quire-contract-ir/AP-002
    type: measures
---
# Bounded Kani support and replay parity measurement

## Decision Use

The measurement informs whether a named bounded Kani profile candidate is ready for human review. It does not turn finite evidence into an unbounded proof or approve a release.

## Population

The population is every matrix entry and every declared valid, invalid, incomplete, resource-boundary, counterexample, and replay corpus case for one exact profile selection.

## Collection Procedure

For every matrix entry, execute its declared native and Kani corpus cases twice with the exact executable/options digests. Compare support/refusal/inconclusive classification, validated input outcome, resource outcome, artifact/provenance identity, and native replay. Record complete per-case evidence rather than only an aggregate rate.

## Interpretation

The target is exact matrix parity and exact counterexample replay agreement. Any non-success outcome is retained as that outcome; it is neither omitted nor counted as Boolean proof. Only the named human owner judges sufficiency.
