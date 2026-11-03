---
id: SR-2026110283838
title: gap-analysis of default mapping API evidence
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; PR 853; tests/it/default_mapping_surface.rs, tests/it/main.rs,
  FR-019-AC-4
review_set: subset
---

## Summary

Reviewed PR 853 for IR-1003 against FR-019-AC-4. The real default-feature model consumer checks both excluded allocation-injection names; no defects found. Application quire-contract-ir / quire-contract-model 0.1.0.

## Verdict

**PASS** for this evidence slice. The healthy consumer compiles before two separate negative probes. Cargo JSON checks require exactly one error, the intended consumer target and source, the expected E0432 or E0599 code, and the exact primary source span. Missing dependencies, provisioning failures, unrelated compiler errors, and an unexpectedly exported symbol fail the test. The dependency keeps defaults enabled, so a future default fault-injection regression is detectable.

The standalone scratch workspace avoids root dev-feature unification; the private target avoids the outer target lock. Process identity and a monotonic in-process serial claim a fresh directory without deleting another run's directory. RAII removes the owned scratch tree after success or an assertion failure. No copied external fixtures, new dependencies, source records, production changes, or narrowed gates occur. Searched existing compiler/scratch helpers in tests/it: none provides this independent default-feature Cargo JSON oracle; corpus scratch helpers serve corpus copies rather than consumer workspaces.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

- Examined FR-019-AC-4, spec/model/functional/FR-019-rust-library-interface.md: A default-feature build of `quire-contract-model` exports neither `MappingAllocationPoint` nor `MappingExecutionControl::fail_allocation_at`: code naming either fails to compile without the `fault-injection` feature.
- Examined tests/it/default_mapping_surface.rs and tests/it/main.rs, including the TC-018 / FR-019-AC-4 trace binding.
- Context only: crates/quire-contract-model/Cargo.toml, crate-root feature-gated exports and output_mapping.rs constructors; root dev-dependency enables fault-injection.

## Coverage

The computed matrix independently reports FR-019-AC-4 tagged by default_mapping_surface_excludes_allocation_fault_injection. The source-level binding is correct. Candidate coverage removes exactly that row: 99 to 98 unbacked, 22 retired unchanged, zero status lies; population unchanged. Repository strict coverage remains red; this PASS makes no repository-wide completeness claim. No applicable AssuranceProfile found. No optional repository-wide semantic analysis was run; scoped requirement/test alignment was examined as dispatched.

Plan completion: not assessed

## Validation

Inspected retained focused-test.log: one passing integration test, including all three real compiler checks. Inspected clippy.log: successful workspace conformance-cli integration-target check; fmt.log has no errors. The initial failed relative-path matcher run is retained, and the corrected matcher accepts absolute or relative primary span paths without weakening symbol/diagnostic attribution. No redundant Cargo gate rerun; the lead owns the final complete native integration run. Matrix read used the lead-specified quire executable. No specification or verification-method changes; spec-review and specification sub-analyses are inapplicable.
