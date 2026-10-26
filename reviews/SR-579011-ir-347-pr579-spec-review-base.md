---
id: SR-579011
title: "IR-347 PR #579 spec-review/base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #579 head 4c8d4bc3b0f44b718ddafbd26bbbb99c7967779e, origin/main...HEAD; spec/kani/functional/FR-039-root-crate-public-interface.md, spec/kani/matrix/TC-055-root-crate-public-interface.md, spec/kani/matrix/tests.md, spec/core/matrix/tests.md, spec/tests.md, spec/assurance/AD-005-qsl-consumption-seam.md; evidence tests/it/kani_shared.rs, tests/it/cycle_free_model.rs, src/kani/; IR-347"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-039
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-055
    type: reviews
---

## Summary

Ticket: IR-347. Base checklist review of the TC-055 negative-corpus correction. The PR removes
the TC-041 citation from FR-039-AC-4 and the TC-055 procedure, keeps TC-042, and flips the
FR-039, TC-055, StR-001, Kani index and AD-005 D-3 status prose to implemented.

Source check: TC-041's tests in `tests/it/cycle_free_model.rs` are Cargo-graph and model-boundary
checks with no Kani input and no `catch_unwind`. The model matrix defines TC-041 the same way.
Removing the TC-041 citation is accurate and drops no executable obligation. The universal clause
"every public `kani` function over untrusted input returns a typed error with no public panic path"
is unchanged, word for word.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-039-AC-4 now limits its universal claim to "the TC-042 negative corpus". TC-042's defined cases (profile and matrix, finite population, dispatch, outcome kinds) do not reach every public `kani` function over untrusted input. The serde readers for `FiniteInput`, `ModuleDescriptor` and `KaniOutcome`, including the cancellation wire, are run under `catch_unwind` only by tests tagged TC-055, TC-443 or TC-449, not TC-042. As written, a panic in `serde_json::from_value::<KaniOutcome>` would not violate AC-4. The AC should cite the negative corpus TC-055 actually runs (the TC-042 negatives plus the public wire-reader and outcome-construction negatives), or drop the corpus qualifier and let TC-055 enumerate the cases. | spec/kani/functional/FR-039-root-crate-public-interface.md:116; spec/kani/matrix/tests.md:34; tests/it/kani_shared.rs:298; tests/it/kani_shared.rs:395; tests/it/kani_shared.rs:538; tests/it/kani_shared.rs:581 |

## Verdict

Not mergeable as written. Dropping TC-041 is correct and backed by the source. The universal
obligation text is kept. However, the replacement qualifier names a corpus narrower than both the
obligation and the evidence that backs it, so AC-4 does not keep its full every-public-function
scope. In substance the code holds: every fallible public `kani` entry point over untrusted input
(`KaniProfile::new` and its wire reader, `classify`, `FiniteInput::validate` and its reader,
`DispatchIndex::new`/`resolve`, the `ModuleDescriptor` reader, `KaniOutcome::proved`,
`non_success`, the `KaniOutcome` and `CapabilityDisposition` readers) has a typed-error negative case
under `catch_unwind` that is tagged FR-039-AC-4. The defect is in the citation, not the code. The
ID formats, the AC/TC link (`Test (TC-055)`) and the Quire validation (exit 0) are clean. Tests were
not run, per the spec-only scope.

## Coverage

```yaml
scope:
- id: FR-039-AC-4
  path: spec/kani/functional/FR-039-root-crate-public-interface.md
  role: examined
  excerpt: "Every public `kani` function over untrusted input returns a typed error with no public panic path under the TC-042 negative corpus."
- id: TC-055
  path: spec/kani/matrix/TC-055-root-crate-public-interface.md
  role: examined
  excerpt: "and must be absent. Run the TC-042 negative corpus under `catch_unwind`."
- id: TC-042 cases
  path: spec/kani/matrix/tests.md
  role: context_only
  excerpt: "every profile/matrix construct and unknown/conflicting/missing entry; ... each valid, duplicate, dangling, foreign, wrong-type, incomplete, unavailable and one-over-bound population/snapshot/reference/collection case; no-assumption invalid-input probes; every typed outcome kind and Boolean-field absence; arithmetic/definedness, object/reference/graph and collection/query dispatch ownership"
- id: TC-041
  path: spec/model/matrix/tests.md
  role: context_only
  excerpt: "Reach the model through `quire_contract_model` while proving the model/owner/root Cargo graph is acyclic"
```

## Dispositions

Round 1. Checked at head bcc20f7b11cab92c481eab21666091ba37313243 against fix diff 4c8d4bc3..bcc20f7b (one commit, "Preserve full public Kani error surface in TC-055"), six spec files, no code. Cargo, Kani and `make ci` were not run (spec-only). `quire validate` over spec, plan and reviews is grammar-clean; `quire coverage --strict` reports 68 unbacked rows, byte-identical to `origin/main`, none in a file this PR touches.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-039-AC-4 now reads "Every public `kani` function over untrusted input returns a typed error with no public panic path." The corpus qualifier is gone, so the obligation again covers every public `kani` function over untrusted input. The TC-055 procedure now enumerates negative cases for profile construction and classification, finite-input validation, dispatch construction and resolution, outcome construction, and JSON readers for public `kani` types. Each category has a `catch_unwind` typed-error test tagged FR-039-AC-4 in `tests/it/kani_shared.rs` (lines 99, 162, 207, 261, 298, 395, 490, 538, 581), which includes the `FiniteInput`, `ModuleDescriptor` and `KaniOutcome` readers the finding named. |
