---
id: SR-558007
title: "IR-1014 PR 564 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #564 at a08376c2; FR-061 trace bindings and new refinement tests"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-061
    type: reviews
---

## Summary

Compared the changed FR-061 trace tags with their actual test assertions and the explicit partial scope in PR #564. The test matrix can count a tagged AC as backed even where the new tests intentionally exercise only an early subset of that AC. This review does not assess plan completion or claim that the remaining work is out of scope for IR-1014.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Remove or narrow `FR-061-AC-1` from the row-empty refinement test and the state-machine-only owner test. AC-1 requires a positive populated refinement with population, object, operation step, history and fairness rows and public read/write/read identity retention; these tests build no such rows, and one has no refinement node. The current trace falsely backs the complete AC. | tests/it/checked_package_v2_refinement.rs:406, 417; FR-061-AC-1 |
| FND-002 | medium | Remove `FR-061-AC-8` from the row-empty and one-step negative tests until an admitted proof-eligible step owner and an ineligible owner exercise the required occurrence sets and all adverse cases. The one-step fixture is deliberately refused for its missing catalog, so it cannot establish the positive n-step clause; tagging the whole AC inflates coverage. | tests/it/checked_package_v2_refinement.rs:300, 340, 417; FR-061-AC-8 |
| FND-003 | medium | Remove `TC-062` from all eight new `#[trace]` attributes. `quire coverage --scope . --strict` reports each as an orphan: TC-062 is a TestCase artifact, not a declared criterion row. Keep only actual FR acceptance criterion IDs that each test proves. | tests/it/checked_package_v2_refinement.rs:237-437; TC-062 |

## Coverage

Examined all eight changed refinement tests against FR-061-AC-1, AC-2, AC-3, AC-7, AC-8 and AC-9. The code also traces `TC-062`; the test case is a planned procedure, while the runtime evidence is the actual Rust assertions. No new reverse code-to-spec orphan was established in this diff. Plan completion: not assessed.

## Verdict

CONDITIONAL. Correct the trace claims while preserving the unchanged full AC text and the IR-1014 open-work list.

## Dispositions

Fix round 1 at PR head `b71dcf3022e9c94017b081f0d336eaa0e4efaaea`:

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Both owner-only and row-empty positive tests no longer trace FR-061-AC-1. After excerpt: `#[test] fn refinement_selected_state_machine_owner_is_admitted()` and `#[test] fn refinement_row_empty_model_refinement_is_admitted()`. |
| FND-002 | fixed | The sidecar-only and row-empty vectors no longer trace FR-061-AC-8. The positive and adverse AC-8 vectors remain open under IR-1014. After excerpt: `#[test] fn refinement_step_row_sidecar_shape_and_order_are_exact()` and `#[test] fn refinement_row_empty_model_refinement_is_admitted()`. |
| FND-003 | fixed | No `TC-062` trace argument remains in the new refinement test file; the four remaining trace attributes name FR-061-AC-2 or FR-061-AC-9 only. After excerpt: `#[trace("FR-061-AC-2")]` and `#[trace("FR-061-AC-9")]`. |

Final verdict: PASS for the corrected trace bindings in this partial slice. The untouched full FR-061 acceptance criteria remain open where their complete evidence is absent.

## String-edge gate re-review

At PR head `421176f25daed888a2fb549bfef55bd1ec76e97b`, the new `refinement_open_argument_has_one_closed_spelling` test traces FR-061-AC-2's closed-wire argument rule. It establishes that exact `open` passes closed-body validation and reaches the later stale-key stage, while another spelling refuses at the argument path. No `TC-062`, FR-061-AC-1 or FR-061-AC-8 tag was reintroduced. The remaining full AC-2 vectors stay open under IR-1014; no new false trace finding was established for this delta. PASS remains for this head.
