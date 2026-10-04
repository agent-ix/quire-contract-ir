---
id: SR-1362
title: "PR #288 FR-020-AC-3 status spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@9adfb5a0259d84da4fc68471f886b56f7e4a73f6; spec/conformance/functional/FR-020-json-conformance-interface.md, spec/conformance/matrix/tests.md, spec/core/matrix/tests.md, spec/tests.md (git diff origin/main...HEAD)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-020
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/StR-002
    type: references
---
# SR-1362: PR #288 FR-020-AC-3 status spec review

## Summary

Ticket: IR-568. PR agent-ix/quire-contract-ir#288.

The spec diff has four hunks. Each one removes the partial qualifier or the gap note
for the last clause of FR-020-AC-3. The normative AC text is otherwise
byte-identical, so the obligation did not change and was not weakened. Only its
status changed.

Checked:

- **FR-020-AC-3.** "its last clause only partly (the matrix names the gap)" is
  removed. The clause still requires the expected bytes to be written in the test,
  and SR-1361 measures that this now holds.
- **Conformance matrix, FR-020 row.** It reads "✅ implemented". The evidence
  sentence names the test and states that the 47 files equal written-out bytes, not
  encoder output. That is accurate. The sentence "re-recorded from `quire-canonical`'s
  bytes" stays true as history: the files were recorded with the encoder, and the
  test pins them independently.
- **Core matrix, StR-002.** It reads "✅ implemented". It traces FR-016 through
  FR-018 and FR-020, and every one of those rows is implemented. No other StR-002 gap
  remains.
- **spec/tests.md, Conformance line.** It reads implemented and agrees with the
  conformance matrix.
- `quire validate` passes over `spec/**/*.md`. `make spec` stops only at main's
  strict-23 baseline.

The sub-analyses were not run. The diff adds no requirement statement, structure,
domain object or relationship, so EARS, integrity, object and dependency analysis
have nothing in scope.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Clean. The status edits are true at this head.
