---
id: SR-591001
title: "IR-1032 code and Rust review at 4eb2ab4a"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/, crates/quire-contract-model/src/lib.rs, tests/it/, tests/conformance_qspec/; FR-063-AC-1..6"
review_set: subset
---

## Summary

Ticket: IR-1032. Reviewed PR #591 at 4eb2ab4ac003ac9e74ac4f11c6c0eb21dcd63bad against FR-063, QSpec FR-322-AC-118, TC-233 and TC-280. The code adds a closed typed form and bounded exact graph validation. FR-063-AC-5 remains Planned and unclaimed pending STD-497.

## Scope

- FR-063-AC-1 (examined): external typed package, selected Time dimensions, public writer/reader round trip, complete retained node and identity.
- FR-063-AC-2 (examined): digest-ordered distinct pair, selected source, closed positive reduced preimage, stable reversed spelling, stale identity refusal.
- FR-063-AC-3 (examined): published TC-233 keys and scales, original Time keys, admitted three-package equivalence class and source-order anchors.
- FR-063-AC-4 (examined): TC-280 adverse pair and graph refusals.
- FR-063-AC-5 (context_only): adverse member and refusal precedence pending STD-497.
- FR-063-AC-6 (examined): closed typed writer and reader with no alternate path or schema/fixture copy.

## Verdict

**FAIL** — the new conformance test duplicates six digest values from the authoritative QSpec fixture in this repository. Focused 6 integration tests, one TC-233 vector test, all-target compile and model Clippy passed in the coder's logs; this review did not run an aggregate gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The conformance test copies six published QSpec digest constants into IR even though it reads the authoritative vector rows; this creates a second digest catalog that can drift and must be removed in favor of the fixture's own `sha256` oracle. | tests/conformance_qspec/dimension_equivalence.rs:17 |

## Method

Applied code-review with rust-review and rust-style, including API, exhaustive closed vocabulary, exact arithmetic, graph traversal, panic/resource boundaries, wire decoding and trace-bearing assertions. No applicable AssuranceProfile was found under `spec/`. No CI workflow or gate file changed. The existing source-owned synthetic graph tests pass, but the separate acceptance evidence gaps are recorded in SR-591002.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At fix head 6dbb500e786632e055293850c114c15eb5b73bc7, the six copied digest literals are gone. The vector test now reads `expected_key` directly from each authoritative QSpec row's `sha256` and compares the typed preimage digest to it. |

## Disposition Verdict

**PASS for the reviewed code slice at 6dbb500e786632e055293850c114c15eb5b73bc7.** The original finding remains above as the review record; its latest disposition is fixed. FR-063-AC-5 and broad QSpec conformance are outside this pass.
