---
id: SR-612134
title: "IR-1103 step-relation claim admission specification review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-350-admit-step-relation-claim-body.md; PR #680"
review_set: base
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-350
    type: reviews
---

## Summary

Ticket: IR-1103. Reviewed frozen PR #680 against QSpec origin/main FR-477 and TC-962, with FR-322, FR-341 and FR-402 as controlling context. The review covered every FR-477 acceptance criterion, the IR reader's stage and refusal order, typed retention, selected-model joins, binder identity, writer roundtrip and external QSL readback.

## Verdict

**PASS.** FR-350 allocates all five FR-477 criteria to IR consumption and preserves the stated codes, causes and loci. Its first-failure order preserves the earlier FR-322 stages, and its body step occupies the QSpec position. The model/code claims remain distinct over one claim occurrence. AC-3 adds a falsifiable writer roundtrip and external QSL readback required by IR-1103. No narrowing of the producer contract was found.

An additional public accessor is optional API design, not a missing requirement. QSpec FR-477 defines the node and claim identities, but no IR accessor. IR's existing public `graph()`, `node_kinds()` and `term()` expose the admitted graph, form and typed body. FR-350 requires typed retention and admission; it does not need to prescribe another accessor.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-350-AC-1, role: examined. Excerpt: “Reading a canonical two-execution determinism relation over `V::Vault::step(i)` with an abstraction binding exposes one typed `claim`/`step_relation` node: four ordered root bindings; `V` once in subjects; `x` and `y` in source order, each with six ordered bindings, the same operation anchor, distinct `self` and `i` nodes at levels `[0, 1]` and `[2, 3]`, the operation's exact precondition/postcondition sets, the Vault invariant set, and a bound implementation; declaration `Det`, its source owner, one declaration and one claim occurrence are retained. Its body reading `pre(x.self.l) = pre(y.self.l)` reaches distinct self nodes.”
- FR-350-AC-2, role: examined. Excerpt: “Without an abstraction binding, both implementation members are empty aggregates and the reader exposes the model claim alone; with the binding, both model and code claims are requested from the one claim occurrence with different FR-402 obligation identities.”
- FR-350-AC-3, role: examined. Excerpt: “Adding a Vault invariant, changing `step`'s precondition, or swapping `x` and `y` changes the correctly rekeyed node id and package id; whitespace/comment changes preserve the node id. Writing an admitted typed package and reading its canonical bytes preserves body, typed form and identities; canonical output from the external QSL producer reads back as an admitted typed step relation.”
- FR-350-AC-4, role: examined. Excerpt: “A missing execution `implementation` binding refuses `invalid_semantic_graph`; an unknown alias refuses `missing_declaration`/`missing-name`; an operation target naming an object type and an implementation relation lacking the operation's frame each refuse `invalid_model_binding`/`malformed-declaration`; omitted precondition or invariant references refuse the same at their aggregate; an integer-typed body expression refuses `ill_typed`/`non-boolean-root`, each at FR-477's locus and first applicable reader step.”
- FR-350-AC-5, role: examined. Excerpt: “Replacing `y`'s `self` and `i` parameter references with `x`'s nodes, while recomputing graph identity and dependencies, refuses `invalid_model_binding`/`malformed-declaration` at `y`'s `parameters`; a simultaneous earlier-stage defect wins before this semantic refusal.”
- FR-477-AC-1 through FR-477-AC-5 and TC-962 RN-01 through RN-14, role: context_only. These define the upstream producer vectors allocated to IR.
- FR-322, FR-341 and FR-402, role: context_only. These define the generic package order and identity, binder types and levels, and distinct claim obligation identities.

## Review scope

This was a spec-only review. Focused Quire validation passed. `quire properties --criteria --all` classified all five criteria; three are extractable. No Cargo tests or builds were run.
