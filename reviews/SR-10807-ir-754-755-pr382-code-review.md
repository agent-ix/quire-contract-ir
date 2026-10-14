---
id: SR-10807
title: "IR-754/755 PR 382 code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 382 Cargo.toml, Cargo.lock, Makefile, tests/it/cycle_free_model.rs; FR-028-AC-1 through FR-028-AC-5"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-028
    type: reviews
---

## Summary

Ticket: IR-754 and IR-755. Reviewed frozen source head 91af8a42fa1078dd194fb474aeef7f8f26f98f13 against 815ed48. The changed Rust is test/control code; the real model and runner paths are exercised, but three controls have false negative cases.

## Verdict

FAIL. The production graph check and owner boundary scan can pass while FR-028-AC-1 or FR-028-AC-5 is violated. No full Cargo, Kani, replay, or release gate was run in this read-only review. `git diff --check` passed. No applicable AssuranceProfile was found under `spec/`.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Host-filtered Cargo metadata omits target-specific production edges. On Linux, adding `target.'cfg(windows)'.dependencies.tl-syntax` to the model leaves this test green although a Windows production graph reaches TL. | tests/it/cycle_free_model.rs:20-30 | correct-requirement-no-evidence |
| FND-002 | high | Forbidden-package classification treats a path dependency's null `source` as empty and does not recognize every owner crate name. A transitive path edge to QSL-owned `quire-exact` is accepted despite reaching an owner package. | tests/it/cycle_free_model.rs:63-90 | correct-requirement-no-evidence |
| FND-003 | high | Admission signature inspection does not resolve callback or trait-object type aliases. `pub type Gate = dyn Fn(&[u8]) -> bool; pub fn admit(gate: &Gate)` adds a public validation callback yet the boundary scan reports no violation. | tests/it/cycle_free_model.rs:489-512 | correct-requirement-no-evidence |

## Coverage

- Examined FR-028-AC-1: resolved graph, feature combinations, production/build/dev edge handling, adverse graph control. FND-001 and FND-002 affect this unit.
- Examined FR-028-AC-2: model `run_corpus` versus root runner JSONL bytes, canonical/identity/diagnostic presence and mutated schema refusal. No additional finding.
- Examined FR-028-AC-3: root manifest dependency name/source check, as context. No additional finding.
- Examined FR-028-AC-4: ignored named build lane, default/all-feature/MSRV command construction and explicit Make target. Execution result was not independently measured. No code finding.
- Examined FR-028-AC-5: production source scan, root public export check and adverse examples. FND-003 affects this unit.
- Examined Cargo.toml and Cargo.lock addition of `syn`, Makefile target, and unchanged CI workflow. No new vendoring, production code, unsafe, lint suppression, or CI lane removal in the diff.

Plan completion: not assessed.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-002 | rejected | The stated `quire-exact` scenario is wrong: direct inspection of the current QSL workspace manifest shows `quire-exact` is a separate repository, and current QSL workspace crates have `qsl-*` or `quire-spec-language` package names. This finding must not be used as a blocker. |
| FND-001 | fixed | Head e66c9d3 removes host filtering from Cargo metadata and adds a target-gated owner edge adverse control at tests/it/cycle_free_model.rs:224-230. |
| FND-003 | fixed | Head e66c9d3 visits public type aliases for callback/trait-object types and adds `dyn Fn` and bare-function alias adverse controls at tests/it/cycle_free_model.rs:504-514, 657-664. |

## Disposition verdict

Round 1 at e66c9d3: all valid code-review findings fixed. FND-002 remains rejected as an erroneous original finding. No new defect found in the two-fix diff.
