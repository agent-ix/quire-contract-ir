---
id: SR-202611010103
title: "IR-1237 PR 799 probabilistic claim sidecar specification review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; IR-1237; PR #799; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Reviewed PR #799 frozen head `c1462303` against authoritative QSpec origin/main `6fee29e3`, FR-322-AC-288/297 and the published PC vectors. No applicable AssuranceProfile was found under `spec/`. The PR changes three requirement documents and no code.

## Verdict

**FAIL** — FR-038-AC-394 names a complete QSpec checked-term-data fixture that does not exist. The QSpec vector contains sidecar rows and graph context, but no complete checked-package package fixture. Other changed requirements cover the four-array wire, graph admission, typed joins and loci, reader/writer round trip, and planned public inventory consistently.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-394 requires tests to read a published QSpec checked-term-data fixture, but QSpec publishes only the sidecar vector and no complete checked-package fixture with this sidecar. State how the test constructs a complete package from authoritative rows. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4644 |

## Coverage

- FR-038-AC-390, examined: required four-array closed sidecar and absent/present empty distinction.
- FR-038-AC-391, examined: two positive probabilistic claims and no name/span identity inference.
- FR-038-AC-392, examined: schema and wrong-reference refusals with typed loci.
- FR-038-AC-393, examined: duplicate and ordering refusals.
- FR-038-AC-394, examined: by-reference vector test and work limit; FND-001.
- FR-038-AC-395, examined: narrow claim graph/body admission and earlier-stage precedence.
- FR-051-AC-4 through AC-6, examined: four-array writer and nonempty round trip.
- FR-019-AC-18 and planned public-item row, examined: public row and vocabulary inventory.
- QSpec FR-322-AC-288, AC-292 and AC-297; PC-01/02, PC-S1/S2 and PC-R1..R7, context_only: authoritative requirements and vectors.

## Evidence

QSpec `proposals/checked-package-v2/checked-term-data-vectors.json` has `positive.probabilistic_claims` and per-case `checked_claim_context`; its root keys are `positive`, `probabilistic_claim_order`, `reader_refusals`, `schema_invalid` and `work_limit`. The QSpec `proposals/checked-package-v2/fixtures/` directory contains no complete checked-package fixture with `checked_term_data` or `probabilistic_claims`. FR-038-AC-394 explicitly names a separate "published checked-term-data fixture" and vector. A test cannot read the named fixture from the selected checkout. The criterion can instead specify a complete IR-built package assembled from the authoritative vector rows by reference, with no copied QSpec content.

No Cargo or Make build was run for this spec-only review.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | PR #799 head `c4a19af6` rewrites FR-038-AC-391 and AC-394 to construct a complete valid V2 package in IR tests from QSpec vector row identities and resolved-node context by reference. It explicitly says the vector is not a complete checked-package fixture. The revised document validates and the diff has no whitespace error. |

## Current verdict

**PASS** at PR #799 head `c4a19af6`; no remaining findings in this review scope.
