---
id: MP-002
title: "Bounded Kani support and replay parity measurement"
type: MeasurementPlan
status: proposed
owner: kreneskyp
metric: bounded_kani_profile_parity
definition_version: quire-contract-ir.bounded-kani-parity/v3
stage: gate
ground_truth_kind: mechanical
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
protected_apparatus:
  - Makefile
  - tests/it/main.rs
  - tests/it/kani_shared.rs
  - tests/it/kani_arithmetic.rs
  - tests/it/kani_objects.rs
  - tests/it/kani_collections.rs
  - tests/it/kani_replay.rs
  - tests/fixtures/native-rule-model.json
  - tests/fixtures/kani-concrete-playback.txt
negative_controls:
  - kind: apparatus-edit
    description: >-
      the Makefile test recipe, the integration-test module list, the
      shared and per-family bounded-Kani contract tests that declare the
      profile matrix and its cases, and the native-model and captured
      concrete-playback fixtures that replay is checked against are
      protected, so editing one alongside the change it grades changes the
      recorded digests; the `src/kani` implementation is the subject under
      measurement and is deliberately not protected
  - kind: suppressed-observation
    description: >-
      a declared profile-matrix entry with no executed case counts as a
      disagreeing case rather than an omission from the denominator, and the
      matrix, its cases and the module list that compiles each family's tests
      are protected apparatus, so skipping a matrix entry or dropping a whole
      family lowers the ratio or registers as an apparatus change instead of
      leaving the ratio unaffected
  - kind: selective-reporting
    description: >-
      the proportion is taken over both repetitions of every matrix entry's
      native and Kani corpus cases together
      (`statistical_design.repetitions: 2`), so a clean repetition cannot be
      reported in place of one that disagreed
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

The `proportion` estimate is cases with exact per-case agreement divided by declared cases, reported separately for each typed outcome kind. A case agrees only when its classification and outcome match the matrix, its artifact/provenance identity matches the declared digests, it assumed nothing about invalid input, and native replay agrees. A declared matrix entry with no executed case counts as a disagreeing case, never as an omission from the denominator.

The target is exact matrix parity and exact counterexample replay agreement. Any non-success outcome is retained as that outcome; it is neither omitted nor counted as Boolean proof. Only the named human owner judges sufficiency.
