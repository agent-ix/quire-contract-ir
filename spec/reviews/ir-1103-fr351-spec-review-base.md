---
id: SR-623006
title: IR-1103 FR-351 identifier correction spec-review/base
type: SpecReview
analysis: base
scope: 'agent-ix/quire-contract-ir; spec/checked_package/functional/FR-351-admit-step-relation-claim-body.md;
  spec/reviews/SR-612134-ir1103-step-relation-spec-review-base.md; PR #684'
review_set: subset
relationships:
- target: ix://agent-ix/quire-contract-ir/FR-351
  type: reviews
---

## Summary

Ticket: IR-1103. Frozen PR #684 at 6d5f229d38e5653d9a3ddfe60063c8e242f6d7e7. FR-351 is unique; its five AC texts and Test verification cells exactly match the former step-relation FR-350. Temporal FR-350 and TC-461 remain untouched.

## Verdict

**PASS.** No finding in this review method. The computed strict matrix remains non-green because the newly distinct criteria are untagged on this spec-only head; code PR #681 owns trace retagging.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-351-AC-1, role: examined. Excerpt: Reading a canonical two-execution determinism relation over `V::Vault::step(i)` with an abstraction binding exposes one typed `claim`/`step_relation` node: four ordered root bindings; `V` once in subjects; `x` and `y` in source order, each with six ordered bindings, the same operation anchor, distinct `self` and `i` nodes at levels `[0, 1]` and `[2, 3]`, the operation's exact precondition/postcondition sets, the Vault invariant set, and a bound implementation; declaration `Det`, its source owner, one declaration and one claim occurrence are retained. Its body reading `pre(x.self.l) = pre(y.sel
- FR-351-AC-2, role: examined. Excerpt: Without an abstraction binding, both implementation members are empty aggregates and the reader exposes the model claim alone; with the binding, both model and code claims are requested from the one claim occurrence with different FR-402 obligation identities.
- FR-351-AC-3, role: examined. Excerpt: Adding a Vault invariant, changing `step`'s precondition, or swapping `x` and `y` changes the correctly rekeyed node id and package id; whitespace/comment changes preserve the node id. Writing an admitted typed package and reading its canonical bytes preserves body, typed form and identities; canonical output from the external QSL producer reads back as an admitted typed step relation.
- FR-351-AC-4, role: examined. Excerpt: A missing execution `implementation` binding refuses `invalid_semantic_graph`; an unknown alias refuses `missing_declaration`/`missing-name`; an operation target naming an object type and an implementation relation lacking the operation's frame each refuse `invalid_model_binding`/`malformed-declaration`; omitted precondition or invariant references refuse the same at their aggregate; an integer-typed body expression refuses `ill_typed`/`non-boolean-root`, each at FR-477's locus and first applicable reader step.
- FR-351-AC-5, role: examined. Excerpt: Replacing `y`'s `self` and `i` parameter references with `x`'s nodes, while recomputing graph identity and dependencies, refuses `invalid_model_binding`/`malformed-declaration` at `y`'s `parameters`; a simultaneous earlier-stage defect wins before this semantic refusal.
- FR-351, role: examined. Excerpt: When the Contract IR V2 reader receives a `quire.checked-package/v2` package containing a `claim` node with `semantic_form: "step_relation"`, it shall admit the node only when its body and joins satisfy QSpec FR-477, and retain the admitted claim as a closed `ClaimForm::StepRelation` with a typed body. The reader shall return the first applicable typed refusal under QSpec FR-322's package order and FR-477's node order, with no partially admitted package. This requirement specifies IR's consumption and writing of the QSpec artifact; QSpec FR-477 owns production, and QSpec FR-402 owns obligation
- FR-350, role: examined. Excerpt: Establish native temporal and TL result correspondence; eight acceptance criteria remain under FR-350.
- TC-461, role: examined. Excerpt: Project native temporal subjects and join owner results through correspondence.
- SR-612134, role: examined. Excerpt: Historical review provenance: PR #680 reviewed the step-relation requirement as FR-350. The requirement was later renamed FR-351 to resolve an identifier collision; the relationship above follows its current identity.
