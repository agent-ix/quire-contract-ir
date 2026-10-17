---
id: SR-12111
title: "IR-210 conversion slice gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@a773b9051c732d683b545017e973364a8d982cea; crates/quire-contract-model/src/canonical.rs, crates/quire-contract-model/src/decimal.rs, crates/quire-contract-model/src/expression.rs, tests/it/conformance.rs, tests/it/expression.rs, tests/it/identity.rs, spec/model/functional/FR-013-type-system.md, spec/model/functional/FR-016-canonicalization-digests.md, spec/model/functional/FR-019-rust-library-interface.md, spec/core/stakeholder"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-013
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-016
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: references
---

## Summary

Ticket: IR-210 (conversion slice), PR quire-contract-ir#434. This is a planless gap analysis: plan completion was not assessed. Everything was measured with quire 0.36.2 (engine 0.50.2). Base is origin/main fabdbb4, checked out in a detached scratch worktree, and head is a773b90.

`quire coverage --scope . --strict --json` exits 1 on both trees, from the same pre-existing rows:

- **Matrix reference rows.** 43 unbacked and 0 status lies, on base and head alike.
- **Totals.** Backed 508/565 on both. `minted_targets` is identical between base and head, and so is `groups`.
- **Stakeholder VC rows.** These are unchanged. StR-001-VC-1, StR-001-VC-2, StR-002-VC-2 and StR-003-VC-1 are backed. StR-002-VC-1 and StR-004-VC-1 are unbacked.
- **Coverage matrix.** Base and head both have 531 rows. Only two rows change their binder set, and neither loses a binder:
  - FR-013-AC-5 goes from 6 to 8 binders. It gains the two `decimal.rs` tests.
  - FR-016-AC-6 goes from 2 to 3. It gains `canonical.rs` `tc_017_a_limit_and_an_encoder_fault_...`.
- **Unmatched tags.** These drop from 75 to 62, and none is added. The 13 removed are:
  - FR-013-AC-5 and FR-016-AC-6 near-miss lines on the three gaining tests
  - TC-016 on the four `expression` tests, and TC-018 on `wide_public_numeric_values_...`
  - TC-015 on the probe test

`quire trace --file`, base against head, over all six files:

- **Pairs.** The 15 converted tests carry 51 (symbol, id) pairs at head, and every one has the form `rust-trace-attribute`.
- **Nothing lost.** Every pair present on base is present at head.
- **Pairs gained.** There are nine:
  - FR-013-AC-5 on the two decimal tests
  - FR-016-AC-6 on tc_017
  - TC-016 on the four expression tests
  - TC-018 on `wide_public_numeric_values_...`
  - TC-015 on `tc_018_probe_and_coverage_bounds_...`
- **Base forms.** On base the same pairs were `rust-doc-comment-id`, `rust-comment-id`, `rust-trace-line` or `rust-test-name-id`, and some were not recorded at all. The fully qualified attribute was never read.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-013-AC-5 | examined | Integer bounds and literals at `i128::MIN` and `i128::MAX` ... decode and serialize back to the same decimal strings. Each of the eight members given as a JSON number ... or a string outside the minimal-decimal grammar ... refuses `invalid_wire_format`. Each of the eight given `"170141183460469231731687303715884105728"` or `"-170141183460469231731687303715884105729"` refuses `invalid_numeric_bounds`; ... |
| FR-016-AC-6 | examined | Every canonical byte sequence and digest of this crate comes from `quire-canonical`: ...; a byte limit of the exact length returns the bytes and one byte lower returns `canonicalization_resource_exhausted`; and the crate source holds no `CanonicalWriter` ... symbol ... |
| FR-019-AC-1 | examined | Compile-time/API fixtures plus public-source signature inspection show wire/request values are distinct from private-field validated values, unknown members are rejected consistently with the published schema, every conversion is fallible, canonical profiles are explicit, the fixed conformance registries, three semantic-limit constants, and wire-depth constant are stable public exports, and forbidden host/downstream/schema-library vocabulary is absent without requiring nightly rustdoc JSON. |
| STD-001 canonicalization_resource_exhausted | context_only | Canonical byte allocation cannot be reserved without exceeding host resources |
| StR-001-VC-1, StR-001-VC-2, StR-002-VC-1, StR-002-VC-2, StR-003-VC-1, StR-004-VC-1 | examined | Stakeholder validation criteria: backed status compared base against head |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-019-AC-1 has a single backer: the corpus test, whose Trace line this PR rewrites. That body asserts only the registry-export clause and the fallible decoding of invalid UTF-8. It does no public-source signature inspection. It does not check that wire values are distinct from validated values or that unknown members are rejected. It does not assert the semantic-limit or wire-depth constants, or that forbidden vocabulary is absent. The row reads as tagged but is mostly unasserted. This predates the PR, whose credit is unchanged, so it does not block the merge. It needs a follow-up ticket. | tests/it/conformance.rs:135-147 |
| FND-002 | low | `encoder_refusal` behaves in a way no criterion owns. A protocol violation, an internal fault or an over-2^53 integer from the encoder is reported as `canonicalization_resource_exhausted` with the message "canonical encoder failed unexpectedly", distinct from a real limit. STD-001 defines the code only as an allocation or resource condition, and FR-016 says nothing about encoder faults. The only test of this behaviour is tc_017, and its FR-016-AC-6 tag does not fit (SR-12110 FND-001). | crates/quire-contract-model/src/canonical.rs:408-428 |

## Verdict

MERGEABLE ON COVERAGE GROUNDS. Both findings predate this PR or sit outside its change; FND-002 is code with no owning requirement, not an AC gap. Nothing lost credit:

- strict 43/0 holds
- 508/565 holds
- groups and minted targets are byte-identical
- the StR VC rows are unchanged

The two gains are real in the matrix, but they do not move any row between backed and unbacked:

- FR-013-AC-5 and FR-016-AC-6 were already backed.
- FR-013-AC-5's observable clauses are asserted by the `wire.rs` tests, and the corpus fixture schema test also asserts it.
- FR-016-AC-6 is asserted by `tc_017_the_five_kinds_equal_their_written_out_bytes_digests_and_limits` and `tc_017_the_canonical_source_holds_no_encoder_of_its_own`.

So the inflation is cosmetic for coverage. The binding correctness concern is SR-12110 FND-001.

Recommended follow-ups (not in this PR):

- **FND-001.** Open a ticket to give FR-019-AC-1 real inspection evidence.
- **FND-002.** Either add an FR-016 criterion, or add an STD-001 note on the encoder-fault message, and retag tc_017 to it.
