---
id: SR-558004
title: "IR-1014 PR 562 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@8b7a22cbed7355880050d6c5705dbfddc1f408a1; PR #562 Rust reader, public refusal, writer vectors, and call sites"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-059
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-060
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-062
    type: reviews
---

# SR-558004: IR-1014 PR 562 code and Rust review

## Summary

Ticket: IR-1014. The review covers the complete 15-file PR diff at 8b7a22c, with rust-review folded into code-review. QSpec FR-475, FR-476, TC-960 and TC-961 at origin/main d818d8e were used as the wire authority. The repo's AGENTS.md, CLAUDE.md, root manifest, lint and format configuration were read. No CI gate, dependency, schema copy, or QSL source changed. The new closed forms, typed duplicate payload, digest-ordered node traversal, and public read/write path otherwise match the specified shape. The author's focused log reports 10 new tests and one model-member test passed; `git diff --check` passed independently. Final aggregate gates belong to the code lead after fixes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Terminal self-type derivation scans every selected population member for every terminal before duplicate rejection, without charging the reader work meter. An untrusted package with many individually valid terminal nodes and a large selected population document can perform T times P object lookups under a low work limit and return a semantic refusal instead of the required work-incomplete outcome. Cache the selected model's unique self type or meter each traversal before it runs; assert the work limit boundary. | crates/quire-contract-model/src/checked_package/v2/claim_terminal.rs:99 |

## Coverage

Examined FR-059-AC-1 through AC-5, FR-060-AC-1 through AC-5, and FR-062-AC-1 through AC-4 against the changed implementation and tests. FR-038's finite, fallible work requirement was context for FND-001. Public enum variants and inline typed refusal are deliberate API choices; the `large_enum_variant` allowances explain those choices and are not findings. No vendored QSpec material was added. Plan completion: not assessed.

## Verdict

CONDITIONAL. Correct the unmetered terminal self-type walk before the final PR gate.

## Dispositions

Fix round 1, PR head `9303fed058aaa38008d8d43f687d7bc72ae1b97e`:

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | `self_object_key` now charges one work unit for each population and member before inspecting it, propagates `ValidationFailure`, and `terminal_self_population_scan_obeys_work_limit` observes `Work` incomplete at the terminal `parameters` binding. After excerpt: `for members in owner.package.population_members.values() { meter.charge(1, || binding_path(position, 2))?; for object in members { meter.charge(1, || binding_path(position, 2))?;` |

The code-review finding is resolved at this head. Final aggregate gates remain with the lead.

## New findings (disposition pass 2)

At PR head `7ca9a4d8218be042a5d90873ec08e15c833653cb`:

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | QSpec FR-322 says a `protocol`/`protocol_reference` carrier's typed target join runs in the declaration step before frame, state and body steps. The reader calls `validate_protocol_references` after terminal validation, and the new mixed test asserts that a bad terminal wins over an independently malformed carrier. With both defects present, the reader returns the terminal's `invalid_package` instead of the earlier carrier's `ill_typed`/`operator-ineligible` at `/body/target`. Move the carrier target join to the specified declaration point, preserving its earlier graph-shape and key checks; then use a genuine later body form for terminal precedence coverage. | crates/quire-contract-model/src/checked_package/v2/mod.rs:2630; tests/it/checked_package_v2_claim_terminal.rs:1391 |

## Current verdict

FAIL at `7ca9a4d8`: FND-002 is open. FND-001 remains fixed.

## Dispositions (round 3)

At PR head `2c102b478f40c569c451b386e19e74ce199fccbe`:

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-002 | fixed | `validate_protocol_references` now runs after graph-shape, key and declaration-name checks and before model-owner, extension, frame, state and body checks. The mixed vector now expects the carrier's `ill_typed`/`operator-ineligible` refusal before the terminal's invalid mode. The standalone carrier vector proves that defect independently. After excerpt: `validate_timed_bounds_reduced(&graph.nodes, &index, meter)?; validate_protocol_references(&graph.nodes, &kinds, &index)?;` |

## Final reviewed-head verdict

PASS at `2c102b47`. Both code-review findings are fixed. The coder's focused log reports formatting, model and integration-test Clippy, 11 terminal tests and 12 protocol-reference tests passed; `git diff --check` passed independently. Final aggregate gates remain with the lead.

## Final gate regression re-review

At PR head `0add8e8ecd3579e32b7fb4a1432a70518c9af1db`, the only code delta since the reviewed head replaces the node-body `map`/`collect` parse in `decode_borrowed_wire` with a preallocated, ordered loop. Each body still calls `strict_parse` once before its node is pushed; the first malformed body still returns its refusal before later nodes or diagnostics. The vector capacity is the borrowed graph's node count, as before, and the loop does not introduce a full-document `Value` or canonical-byte copy. This preserves FR-038-AC-214's intake shape; the aggregate memory measurement remains the lead's final gate.

The prior final gate at `8749838f` aborted in `tc_048_a_document_nested_past_the_parse_limit_refuses_at_the_parse` on its 256 KiB stack. The coder's focused log reports that this exact test and all 16 flat-wire tests pass after the loop change, along with formatting, model Clippy and diff check. I independently changed only the diagnostic-details depth from 50 to 61 in a disposable worktree at `0add8e8e`; `tc_048_a_deep_details_term_within_the_parse_limit_refuses_at_the_grammar` passed on the same 256 KiB test stack. The disposable worktree was removed. No new finding was established.

PASS at `0add8e8e` for the reviewed code diff. The final aggregate gate belongs to the lead.
