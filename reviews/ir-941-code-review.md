---
id: SR-15714
title: code-review review of IR-941 PR 522
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; spec/conformance/functional/FR-020-json-conformance-interface.md;
  spec/conformance/matrix/tests.md; spec/tests.md
review_set: subset
---
# SR-15714: code-review review of IR-941 PR 522

## Summary

Exact. Examined FR-020 shared CLI boundary, AC-16 through AC-19, and both changed matrix declarations against ix-cli-kit main. No defect found under this method.

## Scope

- FR-020-AC-16 (examined): The root binary uses the authoritative kit's caller-owned exit API for statuses 0, 1 and 2; matched corpus and version invocations exit 0, complete mismatches exit 1, and every existing operational code, including invalid invocation, exits 2. No route requests 5 or implicitly terminates through the kit's pass-through error.
- FR-020-AC-17 (examined): Shared compact serialization preserves the exact existing result and diagnostic envelopes, serialized field order and one-newline framing; result serialization errors discard the private buffer, while stdout write/flush failure retains exit 2 and `fixture_io` at `stdout`. Existing fallback diagnostics are retained.
- FR-020-AC-18 (examined): Only the root bridge package depends on `ix-cli-kit`, from its authoritative Git source with the selection in Cargo.lock; the model has no kit dependency, and no kit implementation or alternative argv grammar is copied into the repository.
- FR-020-AC-19 (examined): Adoption leaves all published conformance inputs, expectations, schemas and canonical files unchanged; process tests retain the closed argv rejection cases, exact version line, byte-identical repeated corpus output and complete mismatch payload behavior.
- FR-020 (examined): The root bridge package SHALL consume `ix-cli-kit` from its authoritative `agent-ix/ix-cli-kit` Git source, with the selected dependency recorded in `Cargo.lock`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Pass for this method.

## Narrow re-review

PR on main changes the reviewed patch only by adding the FR-003 dependency and changing AC-19 verification to Inspection, Test. The earlier clean verdict under this method carries.
