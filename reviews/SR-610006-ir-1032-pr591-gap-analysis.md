---
id: SR-610006
title: "IR-1032 acceptance evidence gap review at 4eb2ab4a"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-063-dimension-equivalence-node.md, tests/it/checked_package_v2_dimension_equivalence.rs, tests/conformance_qspec/dimension_equivalence.rs; FR-063-AC-1..6"
review_set: subset
---

## Summary

Ticket: IR-1032. Reviewed PR #591 at 4eb2ab4ac003ac9e74ac4f11c6c0eb21dcd63bad. Trace tags exist for AC-1 through AC-4 and AC-6, but the tagged assertions do not establish two specified behaviors. FR-063-AC-5 is Planned and unclaimed pending STD-497.

## Scope

- FR-063-AC-1 (examined): external typed package, selected Time dimensions, public writer/reader round trip, complete retained node and identity.
- FR-063-AC-2 (examined): digest-ordered distinct pair, selected source, closed positive reduced preimage, stable reversed spelling, stale identity refusal.
- FR-063-AC-3 (examined): published TC-233 keys and scales, original Time keys, admitted three-package equivalence class and source-order anchors.
- FR-063-AC-4 (examined): TC-280 adverse pair and graph refusals.
- FR-063-AC-5 (context_only): adverse member and refusal precedence pending STD-497.
- FR-063-AC-6 (examined): closed typed writer and reader with no alternate path or schema/fixture copy.

## Verdict

**CONDITIONAL** — the checked code paths and focused tests pass, but two tagged AC clauses lack a behavior-level oracle. Plan completion: not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-3's published three-package graph is never admitted in a test: the TC-233 test only checks preimage digests and scales, while the admitted graph substitutes three locally constructed source-owned dimensions. This leaves the model-owned Time keys and published graph join unverified. | tests/conformance_qspec/dimension_equivalence.rs:13; tests/it/checked_package_v2_dimension_equivalence.rs:196 |
| FND-002 | medium | AC-2's reversed source declaration is asserted only by calling the same sorting helper twice, without encoding and admitting a second independently constructed package; this assertion cannot detect a writer/reader failure on reversed input. | tests/it/checked_package_v2_dimension_equivalence.rs:180 |

## Coverage

Plan completion: not assessed. FR-063-AC-1, AC-2, AC-3, AC-4 and AC-6 have trace-tagged tests; AC-5 remains Planned and unclaimed. This pass inspected the changed test-to-code bindings and their actual assertions. The repository-wide Test Matrix did not complete within the review window, so no full-repository coverage claim is made. Optional semantic review was not invoked as a separate full-repository exercise; the specific intent-to-test gaps above were found during the required changed-code review.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At fix head 6dbb500e786632e055293850c114c15eb5b73bc7, new `tc_233_published_model_owned_time_graph_admits_as_one_class` builds a package from the six authoritative QSpec vector preimages and keys, selects the three model identities, reads it through the public strict reader, and asserts all retained keys/preimages and anchor ordinals. The focused test passed. |
| FND-002 | fixed | At the same fix head, new `reversed_source_operands_preserve_the_public_writer_and_reader_identity` independently constructs the reversed-pair preimage with reciprocal spelling, encodes and admits both packages through the public APIs, and asserts identical bytes, node keys, preimages, dependencies and package identities. The focused test passed. |

## Disposition Verdict

**PASS for the reviewed acceptance slice at 6dbb500e786632e055293850c114c15eb5b73bc7.** Both original findings remain above as the review record; their latest dispositions are fixed. FR-063-AC-5 remains Planned and unclaimed pending STD-497.
