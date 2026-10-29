---
id: SR-610062
title: "IR-874 PR 614 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/shared.rs; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs; tests/it/checked_package_v2_protocol_body.rs; FR-038-AC-291/292; TC-048; QSpec FR-466-AC-14/16"
review_set: subset
---

## Summary

Ticket: IR-874. Reviewed test-only PR #614 against the IR decision-slot criteria and QSpec's independent oracle. Four public-reader tests cover choice and repeat visible entries and guards for protocol input, finish, compensation-template, and activation record projection, plus the two specified precedence combinations. The bare activation parameter read is pending STD-515 and is not claimed here.

## Examined units

- **FR-038-AC-291** (`examined`): “In a binders-only slot, including each `choice` or `repeat` `visible` entry and guard, a direct read of `over` or model state refuses `ill_typed`/`operator-ineligible` at the read before binder-scope or decision-fact checks.”
- **FR-038-AC-292** (`examined`): “In each of the four `choice`/`repeat` `visible`/guard slots a direct read of protocol input `over` refuses `ill_typed`/`operator-ineligible` at the read, naming the slot, never `wrong_snapshot`.” Also examined finish, compensation-template and activation fact refusals and paired precedence in the same criterion.
- **TC-048** (`examined`): “Assert the specified first refusal and retained stage, owner, slot and reason details rather than only a refusal code (FR-038-AC-291/292; QSpec FR-466-AC-14/16).”
- **QSpec FR-466-AC-16** (`context_only`): “Refusal precedence in one `visible` entry or guard” with four direct `over` slots, activation fact, finish and template scope, and paired precedence.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | New refusal tests check code, cause and partial protocol detail but omit the required exact path, graph locus and retained owner; a refusal at the wrong node or owner would pass. | tests/it/checked_package_v2_protocol_body.rs:434; tests/it/checked_package_v2_protocol_body.rs:465; tests/it/checked_package_v2_protocol_body.rs:723 | correct-requirement-no-evidence |

## Verdict

**CONDITIONAL** — FND-001 leaves the TC-048 location and owner oracle unenforced. The tests otherwise use the public reader, admit each baseline before mutation, assert real typed outcomes, and trace the relevant criteria. `git diff --check origin/main...HEAD` passed. Focused tests, Clippy and fmt were reported by the coder but not rerun in this review because the shared build slot is controlled by the code lead. Hosted Rust Checks were pending at this head.

## Review scope

No production, API, Cargo, CI or spec files change. The helper that constructs `visible` reads wraps non-expression targets in a graph reference; the guard reads target the graph node directly. The two-reader precedence cases use real conjunction nodes. The QSpec DF6b/DF7d–DF7f case set was checked against the four test functions. The full repository gate is pending at the review head.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At PR #614, `assert_protocol_refusal_at` checks exact `/semantic_graph/nodes/{position}` and typed node locus for every new refusal. `BinderScope` assertions compare the clause owner, and `DecisionFact` compares the decision role owner, field node and binder key. The existing code/cause/stage/slot/reason checks remain. |
| FND-002 | fixed | In the reviewed PR, `record_parameter` no longer excludes `self.captures`. `bare_record_capture_refuses_while_its_field_and_scalar_capture_admit` uses the public reader and checks a Boolean-rooted bare record capture in each choice/repeat visible/guard slot for exact `ill_typed` cause, path, locus and slot; projected record fields and scalar captures admit in those same four slots. |

## Current verdict

**PASS** for the reviewed diff and the recorded finding. The assertion helper derives expected position and key from the fixture's targeted node, so a wrong reader path or locus fails. This is a narrow same-PR disposition; hosted checks and the later integrated-head gate remain the code lead's merge checks.

## Integrated-head delta

**PASS** at PR #614 after merging main's #603 commit. The merge conflict changed only the `quire_contract_model` import list: it retains main's `CheckedPackageEvidence` and #614's `CheckedNodeId` and `CheckedPackageRefusal`. A byte-for-byte comparison of the five #614 helper functions and four test functions found all unchanged; the four tests still assert exact refusal paths, loci and owners. A comparison of the seven #603 helper and test functions added or renamed in this file against merge parent found all unchanged. `git diff --check` passed on both parent comparisons. Hosted Rust Checks and the aggregate gate were pending at this audit.

## Expanded STD-515 review

QSpec FR-466-AC-14/16 and TC-948 DF7g/DF7h require a bare read of **any** record binder to refuse after scope and model-state checks. The new public-reader tests cover `opened` and `got` in all choice/repeat visible/guard slots, both mixed operand orders, typed code/cause, exact path/locus and slot. They retain field projection and receive admission controls. The new reader pass is placed after `slot` and before `decision_facts`, which preserves the specified stage order. No unsafe code, public API shape or CI gate changed; `shared.rs` only broadens the documentation of an existing detail variant. The focused test/Clippy/fmt outcomes were reported by the coder; this review did not duplicate Cargo work while the aggregate gate waits.

## New findings (expanded review)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-002 | medium | `record_parameter` excludes every protocol capture, so a readable capture typed as a record can be read bare in a decision slot and pass the new check despite the “any record binder” rule. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:3898; tests/it/checked_package_v2_protocol_body.rs:953 | implementation-bug-despite-evidence |

## Expanded verdict

**CONDITIONAL** — FND-002 is a reachable decision-scope edge. A record-typed protocol capture is included in `input` for the choice/repeat slot and is not `over`; `slot` accepts it. The new pass returns false in `record_parameter` solely because `self.captures.contains(&position)`, and `decision_facts` sees no field projection. A Boolean-rooted expression using that capture bare can therefore admit, contrary to FR-038-AC-292/QSpec FR-466-AC-16. The tests use activation and receive binders only, so this edge has no direct evidence.

## Final disposition

**PASS** for all recorded code-review findings on the expanded PR head. FND-001 remains fixed; FND-002 is fixed by removing the capture exclusion and adding the four-slot adverse case with paired admission controls. The reviewed PR delta passes the whitespace check. Hosted and aggregate gates remain the code lead's final-head checks.
