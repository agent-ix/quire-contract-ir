---
id: SR-15732
title: Incremental gap analysis of IR-941 PR 524
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; Cargo.lock, Cargo.toml, deny.toml, spec/conformance/matrix/tests.md,
  spec/tests.md, src/bin/quire-contract-conformance.rs, tests/it/conformance.rs, tests/it/cycle_free_model.rs
review_set: subset
---
# SR-15732: Incremental gap analysis of IR-941 PR 524

## Summary

Reviewed the frozen PR diff. The four new FR-020 criteria have concrete tagged binders at the candidate head. Tests exercise the root binary and actual dependency graph; AC-19 static unchanged-file claim holds by the eight-path PR diff. No newly unbacked row, unsupported test tag, stub, or unowned new behavior was found. Plan completion: not assessed.

## Scope

- FR-020-AC-16 (examined): The root binary uses the authoritative kit's caller-owned exit API for statuses 0, 1 and 2; matched corpus and version invocations exit 0, complete mismatches exit 1, and every existing operational code, including invalid invocation, exits 2. No route requests 5 or implicitly terminates through the kit's pass-through error.
- FR-020-AC-17 (examined): Shared compact serialization preserves the exact existing result and diagnostic envelopes, serialized field order and one-newline framing; result serialization errors discard the private buffer, while stdout write/flush failure retains exit 2 and `fixture_io` at `stdout`. Existing fallback diagnostics are retained.
- FR-020-AC-18 (examined): Only the root bridge package depends on `ix-cli-kit`, from its authoritative Git source with the selection in Cargo.lock; the model has no kit dependency, and no kit implementation or alternative argv grammar is copied into the repository.
- FR-020-AC-19 (examined): Adoption leaves all published conformance inputs, expectations, schemas and canonical files unchanged; process tests retain the closed argv rejection cases, exact version line, byte-identical repeated corpus output and complete mismatch payload behavior.
- FR-020 (context_only): The repository shall expose a versioned JSON package interface and a process-level conformance runner whose output is independent of Rust type names and debug formatting.

Changed paths examined: Cargo.lock, Cargo.toml, deny.toml, spec/conformance/matrix/tests.md, spec/tests.md, src/bin/quire-contract-conformance.rs, tests/it/conformance.rs, tests/it/cycle_free_model.rs.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Strict coverage measured with `quire coverage --scope . --strict --json` on exact trees: old main exit 1, 45 unbacked/0 status lies; PR exit 1, 59/0; PR exit 1, 55/0. Base→head removes only FR-020-AC-16, -17, -18, -19 from unbacked rows and adds zero. Old→base adds FR-020-AC-16..19, FR-056-AC-1..8, and core-matrix FR-056 and TC-460: 14 new unbacked rows, zero removed. This is an inherited strict-coverage FAIL, never a green gate.

## Verdict

**PASS (incremental gap review).** Repository-wide strict coverage remains failed on inherited rows.
