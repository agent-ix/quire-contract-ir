---
id: SR-15311
title: "IR-943 PR 502 dependency signature trace gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/dependency_references.rs; tests/it/checked_package_v2_dependency_reference.rs; tests/it/checked_package_v2_reader.rs; QSpec FR-322-AC-84..87"
review_set: subset
---

## Summary

Ticket: IR-943. Planless diff-scoped trace audit of the changed reader, its tests, and the merged QSpec criteria. FR-322 resides in the QSpec repository, so IR's computed local matrix does not supply criterion rows for it; the bindings were checked directly from `#[trace]` tags and test assertions.

## Verdict

**CONDITIONAL** — the AC-87 tag on the new reader test claims the full compiler criterion although the test checks only preservation of values supplied in the fixture.

## Coverage

- Examined FR-322-AC-84: the new test checks two missing-selection cases and stale entry 0 ahead of missing entry 1.
- Examined FR-322-AC-85: the existing TC-048 test's paths and refusal causes match the criterion; the PR adds its trace tag.
- Examined FR-322-AC-86: negative and positive tests cover declared parameter/result types, a type dependency closure, and unrelated body/dependency references.
- Examined FR-322-AC-87: the new reader test does not exercise compiler step-row derivation or source-order swapping.
- Context only: FR-322-AC-14 owns the source-map exclusion from package identity that one assertion in the reader test does check.
- Reverse gap and stub scan of the changed implementation found no orphan behavior or stub.
- Plan completion: not assessed.
- Optional semantic review: not run; the test-to-criterion assertion inspection above is confined to this PR diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `FR-322-AC-87` trace overstates coverage: the reader test assigns ordinal 1 and source regions by hand, so it cannot verify compiler derivation from step order or behavior when source order swaps. Scope the tag to a reader-owned criterion or add the compiler evidence under the criterion's owner. | tests/it/checked_package_v2_reader.rs:53-104 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | On PR #502, the test's trace cites FR-322-AC-14 and its comment explicitly confines the assertion to reader preservation. It no longer claims compiler derivation under FR-322-AC-87. |

## Follow-up review at PR

**PASS.** Changed tests now cite local FR-038-AC-4/37/38. The generated-occurrence reader test cites FR-038-AC-4 only for the source-map region's exclusion from package identity and still states that compiler step-order derivation is separate. The dependency-reference tests cite FR-038-AC-37 for missing/stale supply and refusal loci, and FR-038-AC-38 for eligible and ineligible signatures. No changed test carries an external QSpec FR-322 trace ID. The original FND-001 fixed disposition remains effective; no new trace finding. Plan completion: not assessed.
