# PGM-01 implementation plan and delta

Date: 2026-08-30

## Dependency DAG

```text
umbrella #1
  -> PGM-01 requirements and test matrix
  -> composite specification review
  -> canonical policy + v1 envelope schema/corpus
  -> requirement-tagged tests + CI
  -> retained evidence + gap analysis
  -> CODEOWNER review
  -> project Done + issue #3 closure
```

## Plan Delta

- Replace issue #3's undefined `PGM-E` dependency with umbrella issue #1; no
  new program dependency is introduced.
- Retain the existing neutral contribution policy and make its human authority
  enforceable and explicit through `* @kreneskyp` plus protected `main`.
- Express the common envelope as strict JSON Schema plus positive and negative
  conformance fixtures. Do not implement semantic contract IR from #5/#6/#8/#9/#10.
- Validate policy completeness and corpus behavior in the existing Rust CI job.
- Treat human CODEOWNER approval, project Done transition, and issue closure as
  workflow gates after the implementation PR; automation does not impersonate
  those actions.

## Exit Criteria

The Quire specification validates, each matrix row has executable or retained
inspection evidence, `make ci` passes, the gap analysis has no hidden technical
gap, branch protection evidence is retained, and the PR is ready for the named
human CODEOWNER. The source-release decision remains open.

