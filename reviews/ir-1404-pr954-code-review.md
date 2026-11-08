---
id: SR-202611080004001
title: IR remediation batch 1 (PR 954) code review
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 954 head b3363171 vs main; batch-only commits a112c052, 9d04a14c, 5f1d4121, be1de6a6, 2162bafb, 66fc007a, 18dd5d2a, 06bc052a, ed18a531, 3149294b, b3363171; integration of #946/#947/#949/#951/#953"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-358
    type: reviews
---

## Summary

Tickets: IR-1404 (stage 1 and group A), IR-1396, IR-1405. Code review with the rust-review lane folded in, over `git diff origin/main...HEAD` at b3363171. One pass. Findings already dispositioned on the folded PRs are not re-raised.

Integration is sound. The catalog gate runs first on every production entry point: `read_with_budget_against`, `read_against` and `ContractPackageWireV1::read_against`. Every other `operation_catalog()` call is in a test fixture. The catalog is threaded through the IR-1398 and IR-1405 validators. The slot resolution runs after the canonical byte check on the raw bytes. The unbudgeted entries delegate through one budgeted implementation with `NativeJoinBudget::new(limits.work, u64::MAX).with_read_storage_limit(u64::MAX)`. A work denial maps back to `Incomplete{Work, limits.work, consumed, path}` with the same numbers as the old meter, and `charge` admits at `consumed + work <= limit`, the same boundary as the old `exceeds`. quire-canonical 6ff8e091 is identical to its main.

The CheckedNodeId `Encode` reserve hook charges each stack growth before the push. Its test proves an exact limit admits and every lower limit refuses with no output. The D10 conversion is faithful: resolving the converted fixture's slots through `operand_rows` reproduces origin/main's emitted fixture exactly (independent check, 34 nodes). The migrated tests (66fc007a, 18dd5d2a, 06bc052a) only switch `graph()` to `resolved_graph()` or add the `CatalogUnavailable` arm, and no assertion is weakened. The one changed expectation, the carried-sum result-mismatch pointer, follows the IR-1398 owner rule.

Probe for FND-001: a throwaway test, now reverted, replaced one integer literal's value in `v2_all_families` once with the number and once with `{"$serde_json::private::Number":"<n>"}`, then called `settle` and read the result. The two reads give identical outcomes, the same refusal at the same pointer. The token object is therefore read as the number, and the canonical-bytes stage no longer refuses it.

## Verdict

NOT MERGEABLE: FND-001 is a regression in the strict reader's canonical guarantee, and FND-002 is a stage-1 behaviour that contradicts the merged AC it is tagged to.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The new envelope drops `read_typed_prefix`, and with it the `$serde_json::private::Number` guard in `require_canonical_tokens` (common.rs:448), which refused any object whose first member is that name as `noncanonical_wire`. `read_envelope` accepts such an object: it is valid JSON and its own RFC 8785 encoding. Then `strict_parse`'s `StrictSeed::visit_map` (common.rs:1705-1709) decodes every node body and diagnostic detail containing it as a number. Probe: an integer literal value spelled `{"$serde_json::private::Number":"5"}` reads exactly like the literal `5`, where the old reader refused `noncanonical_wire`. Consequence: two different byte strings decode to one package. Wherever the number admits, the token form is admitted too, and the admitted package does not re-encode to its input bytes (FR-038-AC-514). The outcome also changes from today's for that input, against AC-495. Fix: restore the guard in the V2 envelope, for example by refusing that first-member name during the `PackageDocument` walk, and add a V2-read-level test. | crates/quire-contract-model/src/checked_package/v2/budgeted_read.rs:499-558 |
| FND-002 | high | FR-038-AC-510 says that at a zero read-storage limit, "every budgeted read that reaches a read-storage charge returns `Exhausted`". `budget_denial` charges a diagnostic of 0 bytes when there is no pointer, and every read-storage denial has none (`denial()` sets `path: None`), so the charge always succeeds and the outcome is `BudgetIncomplete`. The test tagged FR-038-AC-510 asserts that opposite outcome (zero_read_storage_is_denied_at_the_first_request_and_bytes_precede_it, lines 145-150), and commit 5f1d4121 states the deviation. A QSL caller that matches `Exhausted` for a zero-storage budget, as the merged AC promises, gets `BudgetIncomplete`. Fix: either the code returns `Exhausted` (pending None) at zero storage, or the spec is amended in this PR. The test cannot stay tagged to an AC it contradicts. | crates/quire-contract-model/src/checked_package/v2/budgeted_read.rs:287-310 |
| FND-003 | medium | `Exhausted` can never report a `PackageRead` locus. Work charges during validation run with `active_locus` None, because `read_envelope` clears it at success (budgeted_read.rs:552) and nothing sets a stage locus afterwards. So a work denial records `denied_locus` None, and if its pointer diagnostic is unaffordable, the `(false, None)` arm reports locus `Package`/`"diagnostic"`. The comment "A budget denial always records its locus" is false for every work denial. `pending` is always None as well. FR-038-AC-510 requires the `PackageRead` locus and the pending class. No test reaches `Exhausted`. Scenario: a work limit just under the requirement with read storage exhausted gives `Exhausted{locus: Package}`. | crates/quire-contract-model/src/checked_package/v2/budgeted_read.rs:287-331 |
| FND-004 | medium | Stage-1 tests overclaim their tags. `read_storage_one_below_the_requirement_is_denied_at_the_last_request` is tagged FR-038-AC-497, but it only tests one byte below the whole read, which denies at `Canonical`. The `Envelope`/`"package_bytes"` one-below case, the per-reader-class denials and the duplicate-name and number-out-of-range lexeme charges that AC-497 states are never exercised. It also asserts only `consumed < required`, not `consumed + asked == required` (that the denied request is the last one). `read_storage_counter_api_and_catalog_gate` is tagged FR-038-AC-515, but it compares no first-read and later-read deltas, and it does not check that `CatalogUnavailable` on the budgeted entry leaves the counters unchanged. The matrix shows both ACs covered. | tests/it/checked_package_v2_budgeted_read.rs:98-137, 178-210 |
| FND-005 | medium | Diagnostic details are not handles into the package's one retained document. intake.rs:314-325 builds each detail with `CheckedJson::from_value(&strict_parse(..))`, which makes a fresh `Document` per detail through an uncharged `to_vec` and `read` on the read path. `checked_json.rs:57-60` says this constructor is "not on the read path". The retained envelope `Document` is dropped (mod.rs:1387). FR-038-AC-514 ("handles into the package's one retained document") is tagged by checked_json.rs unit tests that never read a package, so the matrix overclaims it. `DecodedWireV2` is dead code (`#[allow(dead_code)]`). Either narrow the tag to the type-level clauses or note in the ticket that the read side is a later stage. Fix the doc comment either way. | crates/quire-contract-model/src/checked_package/v2/intake.rs:314-325 |
| FND-006 | low | `cargo fmt --all --check` fails on batch-only edits: tests/it/checked_package_v2_operand_families.rs:4, 1628 and 1715 (06bc052a), tests/it/checked_package_v2_map_type.rs:1373 (66fc007a), and tests/it/checked_package_v2_model_members/inherited_dispatch.rs:400 and 412 (2162bafb). The lint gate fails. | tests/it/checked_package_v2_operand_families.rs:4 |
| FND-007 | low | The new CheckedNodeId reservation test uses the deprecated `/// Trace: IR-1404.` doc line and has no `#[trace]`, so it binds no criterion. The renumbering also rewrote deprecated `/// Trace:` lines in tests/conformance_qspec/node_index.rs:46, 149, 205 and 233 instead of removing them. | crates/quire-contract-model/src/checked_package/shared.rs:1399 |

## Coverage

- budgeted_read.rs (entries, outcome builder, envelope, `PackageDocument`, `nesting_depth`), examined: FND-001 to FND-003. The depth check keeps serde_json's 128 cut (`>= 128`), and the past-2^53 float-integer refusal matches the old `holds_float_integer_past_2_pow_53`.
- native_join_budget.rs read-storage counter, `unlimited()`, `finish_read`, `abort`, `resume`, examined: deltas and first-unpaid records behave as AC-496 states.
- mod.rs `WorkMeter` over the budget, examined: amounts, order and boundary are unchanged.
- dispatch.rs `read_checked_package_with_budget`, examined: one-to-one mapping, no V1 branch.
- contract_package.rs, lower.rs, `ValidationFailure::Budget` arms, examined: unreachable on the FR-035 and lowering paths; `without_budget` only keeps matches closed.
- shared.rs CheckedNodeId `Encode` `reserve_list`, examined: correct; FND-007 only.
- checked_json.rs, examined: equality, Debug, serde, iterative `value_of` and drop are sound; FND-005.
- be1de6a6, examined: the past-2^53 detail refusal moved to construction; the wire still refuses it as `noncanonical_wire`.
- 2162bafb D10 conversion and its test, examined: the conversion is faithful (independent equality check). The test's round trip is self-consistent, and its resolved-id comparison is a real oracle.
- protocol_locations PL-04/04b/05/05b, examined: they run through the generic admitted and refusal loops.
- Renumbering, examined: complete in code and tests (see SR-202611080004002).
- Rust lane: no unsafe, no new production unwrap or expect (the one `expect` in `charge_read_storage` is bounded by the filter), clippy `large_enum_variant` allowance justified. Focused probe run only; no aggregate gate run.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-008 | medium | When a refusal or `incomplete` diagnostic cannot be paid, the `Exhausted` result reports locus `Package`/`"diagnostic"`. `charge_outcome_diagnostic` always sets the active locus to `NativeJoinLocusSource::Package` before it charges, so `first_unpaid_locus` is that locus. FR-038-AC-510, as amended in this round, still requires `Exhausted` to hold "its `PackageRead` locus". The new test `an_unaffordable_refusal_diagnostic_is_exhausted_with_the_pending_refusal` asserts the `Package` locus (checked_package_v2_budgeted_read.rs:447-452). As with FND-002, code and test contradict the AC they are tagged to. Scenario: an unknown-member refusal one byte short of its pointer gives `Exhausted{locus: Package}`, while a caller following AC-510 expects `PackageRead(<stage>, i)`. Fix: charge the diagnostic at the stage's `PackageRead` source with form `"diagnostic"`, as `budget_denial` does, or amend AC-510 for outcome diagnostics. | crates/quire-contract-model/src/checked_package/v2/budgeted_read.rs:300-326 |
| FND-009 | low | The CheckedNodeId reservation test is now tagged `#[trace("TC-472", "FR-038-AC-516")]`. FR-038-AC-516 is about quire-walk's caller-reservation entry and quire-exact's integer and rational entries, not the canonical writer. What the test proves, an exact limit admits and every lower limit refuses with no output, is FR-358-AC-13: "the strict read passes the caller budget to quire_canonical's reader and writer ... a reservation one byte past ... is denied ... and returns no partial result". Retag it FR-358-AC-13. | crates/quire-contract-model/src/checked_package/shared.rs:1400 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | `PackageDocument::encode_into` refuses an object whose first member is `$serde_json::private::Number` (budgeted_read.rs:498-510). `serde_json_number_token_spelling_is_noncanonical_where_the_number_admits` reads the plain-number package as admitted and the token spelling as `noncanonical_wire` with no pointer. Mutation check by this reviewer: with the guard disabled (`&& false`), the test fails at line 514; with the guard restored, all 13 budgeted_read tests pass. |
| FND-002 | fixed | Lead decision: FR-038-AC-510 and TC-472 are amended. At zero read storage, a denial whose outcome owns no pointer is `BudgetIncomplete`; `Exhausted` is returned only when the outcome's own diagnostic cannot be paid. The code and `zero_read_storage_is_denied_at_the_first_request_and_bytes_precede_it` now agree with the AC. |
| FND-003 | fixed | Stage loci are now set for WireChecks, Lock, Graph, Equivalence, Diagnostics, OperationEffects and CheckedTermData (mod.rs `at_stage` calls). A work denial records a `PackageRead` locus, and `an_unaffordable_work_denial_diagnostic_is_exhausted_at_its_package_read_locus` reaches `Exhausted` there. Refusal and `incomplete` outcomes now carry `pending`. The locus of an unpaid refusal diagnostic is a separate defect: FND-008. |
| FND-004 | fixed | `each_envelope_and_canonical_request_is_denied_exactly_one_byte_below` sweeps every limit below the requirement. It checks each denial's boundary arithmetic and stage order, and the Envelope/`package_bytes` one-below case at the envelope's last request. A separate test charges the duplicate-name and out-of-range-number diagnostics. The AC-498 test now asserts `consumed + asked == required`. AC-515 has equal first and later deltas, and `tc_472_an_unreadable_catalog_leaves_the_budgeted_entrys_counters_unchanged`. |
| FND-005 | fixed | The AC-514 tags are removed from checked_json.rs. The doc comments now say that the read builds a fresh uncharged document per detail through `from_value` until a later stage, and tests.md lists AC-514 as Planned. |
| FND-006 | fixed | `cargo fmt --all --check` is clean at 09488233. |
| FND-007 | fixed | The `/// Trace:` lines are removed from node_index.rs, and the shared.rs test now uses `#[trace]`. The AC that tag names is wrong: FND-009. |
| FND-008 | fixed | 00945af6: `charge_outcome_diagnostic` charges at the active `PackageRead` source with form `"diagnostic"`, falling back to `PackageRead(WireDecode, 0)`, which `admit_bytes` now sets after the envelope. The test asserts `PackageRead(WireDecode, 0)`/`"diagnostic"`, and the 13 budgeted_read tests pass on 00945af6. |
| FND-009 | fixed | 00945af6 retags the CheckedNodeId writer-reservation test `#[trace("TC-472", "FR-358-AC-13")]`. |
