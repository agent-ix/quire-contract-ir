---
id: SR-579012
title: "IR-347 PR #579 spec-review/spec-integrity-analysis review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; PR #579 head 4c8d4bc3b0f44b718ddafbd26bbbb99c7967779e, origin/main...HEAD; spec/kani/matrix/TC-055-root-crate-public-interface.md, spec/kani/matrix/tests.md, spec/core/matrix/tests.md, spec/tests.md, spec/assurance/AD-005-qsl-consumption-seam.md; evidence tests/it/kani_shared.rs; IR-347"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/TC-055
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-039
    type: reviews
---

## Summary

Ticket: IR-347. This integrity sub-analysis checks that the status and trace prose the PR changed
agrees across the five status locations and with the tests that carry the FR-039-AC-4 trace.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The changed status prose credits AC-4 to TC-042 alone. The TC-055 procedure and Status, the Kani matrix FR-039 row ("AC-4 is backed by typed no-panic TC-042 negative cases"), and the Kani index row ("backed by TC-055 and TC-042") all say so. Four of the AC-4-tagged negative tests are not TC-042 tests: `tc_055_malformed_finite_input_and_descriptor_wire_do_not_panic` and `tc_055_negative_kani_corpus_returns_typed_errors_without_panics` (TC-055 only), `malformed_or_mismatched_cancellation_wire_is_refused` (TC-449), and `tc_443_the_outcome_and_its_error_carry_a_typed_std001_code` (TC-443). The claim is true but incomplete: a reader who followed it would audit only TC-042 and miss the wire-reader negatives that close AC-4. | spec/kani/matrix/TC-055-root-crate-public-interface.md:46; spec/kani/matrix/tests.md:17; spec/tests.md:17; tests/it/kani_shared.rs:298; tests/it/kani_shared.rs:395; tests/it/kani_shared.rs:538; tests/it/kani_shared.rs:581 |

## Verdict

Mergeable once FND-001 is aligned with the fix for SR-579011 FND-001. The five changed status
locations (TC-055 Status, the Kani matrix FR-039 and TC-055 rows, the core StR-001 row, the
spec index Kani row, and AD-005 D-3) agree with each other. None still describes TC-041 as an
open gap. The TC-041 statement in TC-055 Status ("covers the Cargo dependency graph and model
boundary, and supplies no Kani negative cases") matches `tests/it/cycle_free_model.rs` and the
model matrix. The spec index row was stale before this PR: it listed FR-037-AC-6 and FR-039 as
planned while the Kani matrix marked FR-037-AC-6 implemented. The PR corrects that. The evidence
files named in TC-055 Status (`tests/it/public_interface.rs`, `tests/it/kani_shared.rs`, and the
`compile_fail` probes in `src/kani/mod.rs`) exist. Pre-existing and outside this diff: TC-042 has
no TC document of its own. Its corpus is defined only by the "TC-042 cases" row in the Kani matrix.

## Coverage

```yaml
scope:
- id: TC-055 Status
  path: spec/kani/matrix/TC-055-root-crate-public-interface.md
  role: examined
  excerpt: "Implemented by `tests/it/public_interface.rs`, `tests/it/kani_shared.rs`, and the per-item compile-fail probes in `src/kani/mod.rs`. The TC-042 negative Kani paths run under `catch_unwind`. TC-041 covers the Cargo dependency graph and model boundary, and supplies no Kani negative cases."
- id: kani matrix FR-039 row
  path: spec/kani/matrix/tests.md
  role: examined
  excerpt: "✅ AC-1 through AC-3 are backed by exact inventory, excluded-item compile-fail probes and source checks. AC-4 is backed by typed no-panic TC-042 negative cases under `catch_unwind`."
- id: kani matrix TC-055 row
  path: spec/kani/matrix/tests.md
  role: examined
  excerpt: "✅ inventory, forbidden-source checks, per-item compile-fail probes and TC-042 no-panic negatives implemented"
- id: StR-001 row
  path: spec/core/matrix/tests.md
  role: examined
  excerpt: "the root public inventory, typed no-panic Kani negative corpus and model import boundary are backed by TC-055, TC-042 and TC-058"
- id: spec index Kani row
  path: spec/tests.md
  role: examined
  excerpt: "FR-037-AC-6 and FR-039 are implemented and backed by TC-055 and TC-042"
- id: AD-005 D-3
  path: spec/assurance/AD-005-qsl-consumption-seam.md
  role: examined
  excerpt: "The complete TC-055 public-inventory check and its TC-042 negative Kani corpus are implemented (FR-039-AC-4)."
```

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | The fix's StR-001 status prose now says "TC-055 includes cases also tagged TC-042, TC-443 and TC-449", but the row's Test/Validation column lists TC-443 and not TC-449. TC-449 backs FR-030-AC-7, and FR-030 is inside StR-001's "FR-028 through FR-040" trace range. The omission predates this PR (the column on `origin/main` has no TC-449 either), but the row now names a test its own validation column does not carry. Add TC-449 to the column, or drop it from the prose. | spec/core/matrix/tests.md:12; spec/kani/matrix/tests.md:26; tests/it/kani_shared.rs:395 |

## Dispositions

Round 1. Checked at head bcc20f7b11cab92c481eab21666091ba37313243 against fix diff 4c8d4bc3..bcc20f7b. Spec-only; no Cargo, Kani or `make ci`.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | No status location credits AC-4 to TC-042 alone any more. TC-055 Status now says "The TC-055-tagged negative cases in `tests/it/kani_shared.rs` include profile, finite-input, dispatch, outcome-construction, and public JSON-reader refusals under `catch_unwind`, including cases also tagged TC-042, TC-443, and TC-449." The Kani matrix FR-039 row, the TC-055 summary row and the TC-055 coverage-design row, the StR-001 row, the spec index Kani row and AD-005 D-3 all credit TC-055-tagged cases, naming TC-042, TC-443 and TC-449 only as co-tags. Each claim matches the trace tags at `tests/it/kani_shared.rs` 99, 162, 207, 261, 298, 395, 490, 538 and 581. None overclaims: FR-039 ✅ and AD-005 D-3 "implemented" rest on those tests and the TC-055 inventory, and the partly-implemented FR-030-AC-6/TC-443 and the IR-701 TC-449 caveats are unchanged. |

Round 2. Checked at head 13adc3d71743f88d4e8f415a4db2350ec2fd158f against fix diff bcc20f7b..13adc3d7 (one line in `spec/core/matrix/tests.md`). Spec-only; no Cargo, Kani or `make ci`.

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | The StR-001 Test/Validation column now ends "TC-442, TC-443, TC-449", so it carries every TC its status prose names ("TC-055 includes cases also tagged TC-042, TC-443 and TC-449"). The addition is grounded: `spec/kani/matrix/TC-449-kani-cancellation-origin.md` exists, the Kani matrix maps TC-449 to FR-030-AC-7 and lists it on the FR-030 row, FR-030 is inside StR-001's "FR-028 through FR-040" trace, and `tests/it/kani_shared.rs` 363, 395 and 437 carry `trace("TC-449")`. The StR-001 status stays 🚧, consistent with TC-449's 🚧 IR-701 caveat. No other cell changed; `quire validate` on the file exits 0. No new defect. |
