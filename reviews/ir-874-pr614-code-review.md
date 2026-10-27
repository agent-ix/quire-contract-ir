---
id: SR-610062
title: "IR-874 PR 614 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; tests/it/checked_package_v2_protocol_body.rs; FR-038-AC-291/292; TC-048; QSpec FR-466-AC-16"
review_set: subset
---

## Summary

Ticket: IR-874. Reviewed test-only PR #614 at df7e4ee68987856433aaef7baa9c1463392fded3 against the IR decision-slot criteria and QSpec's independent oracle. Four public-reader tests cover choice and repeat visible entries and guards for protocol input, finish, compensation-template, and activation record projection, plus the two specified precedence combinations. The bare activation parameter read is pending STD-515 and is not claimed here.

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
