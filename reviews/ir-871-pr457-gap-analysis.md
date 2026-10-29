---
id: SR-12436
title: "Traceability gap analysis of IR PR 457"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; .github/workflows/ci.yml; Cargo.toml; Makefile; tests/conformance_qsl_ir760/main.rs; tests/it/main.rs"
review_set: subset
---

## Summary

Ticket IR-871, PR #457. The source tests and their trace attributes stay in place under an explicit target. Base and head matrices are byte-identical across 562 normalized rows; no new criterion gap or coverage claim is introduced.

## Examined scope

- **IR-871** (examined): The pull_request workflow runs the Rust subset Format, Lint, Test and Unsafe audit steps and the supply-chain checks without a QSL checkout.
- **Cargo-test-targets** (examined): conformance_qsl_ir760 is an explicit test target at tests/conformance_qsl_ir760/main.rs with test = false.
- **Named-lane** (examined): conformance-qsl-ir760 requires QUIRE_SPEC_LANGUAGE_DIR and invokes both the model and root selected-population tests by explicit target name.
- **Test-source-reuse** (examined): The named target includes existing tests/it/support/mod.rs and tests/it/selected_population.rs via relative #[path] modules, with no copied source.
- **Default-suite** (examined): The ordinary integration binary no longer includes selected_population; its other modules remain listed.
- **FR-053-AC-3** (context_only): Absent, stale, missing-occurrence, foreign and non-population selections yield their distinct typed outcomes without extent.
- **TC-453** (context_only): Run QSL authoritative population fixtures and verify the selected finite population through the public model API, including invalid selection outcomes.

## Verdict

**PASS**. The source tests and their trace attributes stay in place under an explicit target. Base and head matrices are byte-identical across 562 normalized rows; no new criterion gap or coverage claim is introduced.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence

At the reviewed PR head, GitHub Actions pull_request run 39102603725 succeeded with Rust Checks and Supply-chain Checks; the Test step passed without a QSL checkout. Cargo metadata reports test=false for both conformance_qsl_ir760 and selected_population_public. With QUIRE_SPEC_LANGUAGE_DIR unset, the named make target exits 2 before building. The base and head quire matrix TSV files compare byte-identically (562 criterion rows); head strict coverage reports 527/596 backed, 40 unbacked, 0 contradicted. The NFR-060/061 unmatched warnings refer to unchanged files and are outside this diff. Git diff --check passes.
