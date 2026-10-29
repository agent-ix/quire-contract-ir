---
id: SR-15718
title: dependency review of IR-941 PR 522
type: SpecReview
analysis: dependency
scope: agent-ix/quire-contract-ir; spec/conformance/functional/FR-020-json-conformance-interface.md;
  spec/conformance/matrix/tests.md; spec/tests.md
review_set: subset
---
# SR-15718: dependency review of IR-941 PR 522

## Summary

At the reviewed PR head, examined FR-020 shared CLI boundary, AC-16 through AC-19, and both changed matrix declarations against ix-cli-kit main. The findings below need a same-PR fix.

## Scope

- FR-020-AC-16 (examined): The root binary uses the authoritative kit's caller-owned exit API for statuses 0, 1 and 2; matched corpus and version invocations exit 0, complete mismatches exit 1, and every existing operational code, including invalid invocation, exits 2. No route requests 5 or implicitly terminates through the kit's pass-through error.
- FR-020-AC-17 (examined): Shared compact serialization preserves the exact existing result and diagnostic envelopes, serialized field order and one-newline framing; result serialization errors discard the private buffer, while stdout write/flush failure retains exit 2 and `fixture_io` at `stdout`. Existing fallback diagnostics are retained.
- FR-020-AC-18 (examined): Only the root bridge package depends on `ix-cli-kit`, from its authoritative Git source with the selection in Cargo.lock; the model has no kit dependency, and no kit implementation or alternative argv grammar is copied into the repository.
- FR-020-AC-19 (examined): Adoption leaves all published conformance inputs, expectations, schemas and canonical files unchanged; process tests retain the closed argv rejection cases, exact version line, byte-identical repeated corpus output and complete mismatch payload behavior.
- FR-020 (examined): The root bridge package SHALL consume `ix-cli-kit` from its authoritative `agent-ix/ix-cli-kit` Git source, with the selected dependency recorded in `Cargo.lock`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new shared stdout-helper obligation depends on ix-cli-kit FR-003, but the dependency edges and Dependencies section name only FR-017 and FR-006. Add the FR-003 dependency so the owner and prerequisite are explicit. | spec/conformance/functional/FR-020-json-conformance-interface.md:8-13,107-112,199-203 |

## Verdict

Findings open; PR is not ready to merge.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The FR-020 frontmatter now declares `depends_on` to `ix://agent-ix/ix-cli-kit/FR-003`, and Dependencies names FR-003 as owner of `streams::write_primary_stdout`. |

## Narrow re-review

At PR on main, the prior finding is fixed. The remaining FR-020 AC-16 through AC-19 text and planned matrix claims are unchanged relative to the reviewed patch, apart from these two repairs.
