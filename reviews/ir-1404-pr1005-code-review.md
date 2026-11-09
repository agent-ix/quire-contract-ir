---
id: SR-202611081512300008
title: "code-review and rust-review of PR #1005 bounded unpaid read memory"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; tests/read_storage_alloc.rs; tests/it/support/read_observation.rs; tests/it/support/qspec_read_proof.rs; tests/it/support/read_allocation_proof.rs; tests/it/checked_package_v2_artifact_refs.rs; tests/it/admitted_result_context.rs; tests/it/checked_package_v2_canonical_encoding.rs; IR-1404"
review_set: subset
---

## Summary

Ticket: IR-1404. PR agent-ix/quire-contract-ir#1005 at head 3304ddd541baf60c9214e172e1f8525d9656c832, diffed against origin/code/ir-main-read-accounting. The Rust lane is folded in. The diff touches tests and spec only; no production code changed. The owner's direction (bound instead of exact equality, deleting the syn walker, AC-513 Planned, no Clippy lint) is not raised.

Of the eleven focused tests selected in tests/read_storage_alloc.rs, seven passed: the new growth test, consecutive reads, small package, two selected documents, 64 direct dependencies, 10000 diagnostics and the 1 MiB string. Four did not finish within 34 minutes on a debug build and were stopped: chain_100000, flat graph and source map 10000, node with 10000 dependencies, and the over-1-MiB sequence. No result is claimed for those four.

## Verdict

CHANGES REQUIRED. One re-enabled test fails deterministically, contrary to the PR description, and the new test has less power than its name claims.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | One of the ten un-ignored tests fails deterministically (two runs): with the unconfigured profile, the extra-member case refuses at /semantic_graph/nodes/9/body/operation, not .../operation/laws/0/definition/revision. The PR says all ten pass. The total of 1389 plus 24 equals the 1413 tests in the binary, so it is one of the 24 red tests, but it is in none of the reported categories. The old ignore reason (AC-511 allocation proof) was unrelated to this pointer defect. | tests/it/checked_package_v2_artifact_refs.rs:211; tests/it/checked_package_v2_artifact_refs.rs:48 |
| FND-002 | medium | `unpaid_read_memory_does_not_grow_with_input_size` asserts only an absolute ceiling and never compares sizes. At 3200 nodes a leak of up to 40 bytes per node (about 4 to 5 percent of input bytes) passes. The red proof leaked 100 percent of the input, so it shows nothing about a modest leak. Measured unpaid is 0, so assert that unpaid at the largest size does not exceed unpaid at the smallest. | tests/read_storage_alloc.rs:117; tests/read_storage_alloc.rs:130 |
| FND-003 | medium | UNPAID_BOUND = 131072 has no derivation in code or spec. Measured unpaid is 0 on the shapes run, and the largest prior delta reported was 65636, so the value is about twice an unexplained number. Against the shapes that finished it permits 12.5 percent of a 1 MiB string or 13 bytes per diagnostic. Document what fixed allocation it covers, or lower it to the justified maximum. | tests/it/support/read_observation.rs:49 |
| FND-004 | medium | `function_call_parameter_check_has_no_unpaid_allocation` became vacuous when the wrapper stopped asserting: it only checks Admitted and drop, never `accounted`. Reintroducing an unpaid parameter-type allocation passes it. Add `read_allocation_proof::accounted`, as the protocol-location tests do. | tests/read_storage_alloc.rs:1042; tests/read_storage_alloc.rs:1095 |
| FND-005 | medium | Whole-read exact equality survives in `shared_budget_reads`, and exact prefix equality in `every_request`, so an allocation within the amended bound still fails them while the spec says only the bound applies. Convert them to `unpaid` or state why they stay exact. | tests/it/support/read_allocation_proof.rs:459; tests/it/support/read_allocation_proof.rs:179 |
| FND-006 | low | The stale name `the_actual_tc048_catalogue_and_refusal_corpus_is_exactly_accounted` still claims exact accounting after the helper became a bound. | tests/it/checked_package_v2_whole_read_accounting.rs:1406 |

## Coverage

- tests/read_storage_alloc.rs (examined): FND-002, FND-004.
- tests/it/support/read_observation.rs (examined): FND-003.
- tests/it/support/read_allocation_proof.rs (examined): FND-005.
- tests/it/support/qspec_read_proof.rs (examined): the removed `accounted` call is owner direction.
- Deleted inspection files (examined): no remaining reference.
- The ten un-ignored tests (examined): nine pass; FND-001 for the tenth.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | medium | The fix for FND-005 weakened `every_request` at the denied prefix from `stopped.requested == before` to `unpaid(&stopped) <= UNPAID_BOUND`. That assertion was the corpus-wide proof of the FR-038-AC-511 clause "no request is observed for a denied charge", which still stands. A denied charge of up to 4096 bytes that allocates anyway now passes the probe; only the named fixtures that assert `requested` exactly still catch it. FND-005's suggestion was wrong for this line. Compare against the read's own constant, for example `unpaid(&stopped) <= unpaid(&reference)`. | tests/it/support/read_allocation_proof.rs:180 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-008 | medium | No test bounds the unpaid memory of a refinement or step-relation refusal, which is what this round rewrote. `refinement_rows_leave_no_more_unpaid_memory_than_the_empty_refinement` reads only well-formed packages under storage denials, and the refusal tests go through `read_and_account`, whose `observe` no longer calls `accounted`. Putting one site back to an unpaid `ValidationFailure::refused_at` would pass every test. The new test also never asserts that the fixtures admit. Add a refused refinement and a refused step relation to a bound check, and assert Admitted on the ample reads. | tests/it/checked_package_v2_refinement.rs:8833; tests/it/support/qspec_read_proof.rs:295 |
| FND-009 | low | The PR body is stale and now false: it says the bound is 4096 bytes and "no production fix beyond the stacked #1004 was needed", while this round rewrites refinement.rs and step_relation.rs after a probe found unpaid memory growing with content and sets the bound to 1024. | PR #1005 description |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | The code lead routed tc_048_a_clock_row_precedes_a_malformed_temporal_law_definition to the follow-on PR. The PR body now states nine pass and this one stays red, so the false claim is gone; the ignore is not restored. The pointer defect itself is unfixed. |
| FND-002 | fixed | Fix commit 6f055ec adds a smallest-to-largest comparison and a string shape. |
| FND-003 | fixed | UNPAID_BOUND is 4096, with its derivation in the doc comment. |
| FND-004 | fixed | The test now calls `accounted`. |
| FND-005 | fixed | Both sites use the bound. FND-007 records a consequence at line 180. |
| FND-006 | fixed | Renamed to `..._is_accounted_within_the_bound`. |
| FND-007 | still-open | New this round; no fix has landed. |
| FND-007 | fixed | Commit ec7931d5 holds each denied-prefix read to the ample read's own unpaid memory. |
| FND-008 | still-open | New this round; no fix has landed. |
| FND-009 | still-open | New this round; the PR body is unchanged. |
| FND-008 | fixed | Commit ced1a8a bounds a refused refinement and a refused step relation and asserts the ample reads admit. Its power is against exceeding the bound and growth; a constant unpaid refusal under 1024 bytes passes, which the code lead recorded as what AC-511 allows. |
| FND-009 | fixed | The PR body now states 1024 bytes and names the production fixes. |
