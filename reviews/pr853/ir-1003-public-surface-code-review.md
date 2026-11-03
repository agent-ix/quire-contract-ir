---
id: SR-2026110283837
title: code-review of default mapping API evidence
type: SpecReview
analysis: code-review
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

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | The offline scratch consumer resolves an unlocked graph and fails its healthy compilation when the fetched Git cache lacks the moving branch reference, despite the workspace locked graph being available. | tests/it/default_mapping_surface.rs:46 |

The original PASS above assessed warm-cache evidence and did not establish this hosted-cache condition. The hosted full run failed only this new test; cold-cache-before-lock-seed.log independently reproduces the healthy compiler setup failure on quire-canonical's unavailable origin/main reference. This is a test provisioning defect, not absence evidence for FR-019-AC-4. The original finding table and review comment are preserved.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | The scratch consumer now seeds Cargo.lock from the existing workspace Cargo.lock before offline compilation. This reuses the graph the workspace already fetched while retaining the independent workspace and default-feature dependency. Cold-cache-after-lock-seed.log passes the healthy compile and both unchanged precise negative probes; inspection confirms the quire-canonical cache still has no origin/main reference. |

## Disposition validation

Reviewed the seven added lines in the same PR. No manifest feature override, network fallback, skip, weakened diagnostic assertion, repository lock change or new maintained source/version record is introduced. The copied lockfile is ephemeral compiler scratch state, removed with the owned scratch directory; Cargo adapts its package entries to the independent consumer without importing workspace dev features. The existing repository lockfile remains the sole maintained dependency selection.

The cold-cache before and after logs retain the reproduced failure and corrected result (one focused test passed). No concurrent Cargo execution was performed by this reviewer. Final aggregate gate remains the lead's responsibility.
