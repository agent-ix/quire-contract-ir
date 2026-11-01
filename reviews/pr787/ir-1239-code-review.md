---
id: SR-202610310148
title: IR-1239 code-review scoped review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; IR-1239 PR 787 FR-356, TC-465, option_types.rs,
  model_members.rs, ordinary reader regressions and QSpec conformance adapter
review_set: subset
---

## Summary

Reviewed option/reference resolution, alias cycles, value-kind classification, refusal precedence and paths, canonical wrapping, public-reader fixtures and external vector adapter. Selected value-type/tuple/variant names are collected before the option bodies are read; definition order does not affect their admission. No concrete defect found.

## Verdict

PASS — no scoped findings. This review does not replace the required final full integration or green CI Test gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

Scoped review only; no source edits, Cargo commands or backend proofs were run. Inspected supplied workspace Clippy, four focused-test passes and all OP-01..41 passes by reference against the current QSpec checkout. Full IR integration/CI Test gate remains pending with the lead. No applicable AssuranceProfile exists under spec/. Optional broad semantic-gap fan-out and unrelated catalog analyses were not requested. No new public symbols, dependency, mirror value model, copied vectors or schema snapshots were introduced. Quire matrix was computed with --scope .; the initial unsupported --json flag was corrected to the default matrix output. Scoped document validation succeeded with existing module duplicate-archetype warnings only.

## Examined units

- FR-356-AC-1 — role: examined; path: spec/checked_package/functional/FR-356-admit-option-value-type-definitions.md; excerpt: The public reader admits named Integer, unit and record option definitions and references in fields, parameters, results, aliases, sequence/map definitions, tuple positions and variant payloads under QSpec FR-154-AC-39. Existing native/package/unit typeRef forms remain unchanged.
- FR-356-AC-2 — role: examined; path: spec/checked_package/functional/FR-356-admit-option-value-type-definitions.md; excerpt: Required named options and optional inner fields share the canonical option node; Option<Integer>, Option<Quantity<ms>>, Sequence<Option<Integer>>[0,3], and optional collection wrapping match OP-01..08 and FR-322-AC-298 without flattening.
- FR-356-AC-3 — role: examined; path: spec/checked_package/functional/FR-356-admit-option-value-type-definitions.md; excerpt: Direct/alias/self nested options, Clock/alias of Clock, a namespace inner, reference-to-option, forbidden native Option spellings and optional single-valued option fields refuse with the published codes, causes and exact selected-document paths, including typeRef-before-presence precedence.
- FR-356-AC-4 — role: examined; path: spec/checked_package/functional/FR-356-admit-option-value-type-definitions.md; excerpt: Missing/null/empty/non-string/array/object inner, unresolved inner, forbidden shape members, option constraints and appliesTo-option constraints produce their owning code/cause/locus in shape/constraints/form/resolution/kind order. Earlier package-stage refusals remain earlier.
- FR-356-AC-5 — role: examined; path: spec/checked_package/functional/FR-356-admit-option-value-type-definitions.md; excerpt: A cross-reader gate reads every OP-01..41 row from current QSpec by reference, checks all expected admissions/types/refusals and exact document loci through production admission, and reports the QSpec checkout HEAD separately from implementation data. No copied vectors, schema snapshots, compatibility path or FCD changes are introduced.
- FR-356 — role: examined; path: spec/checked_package/functional/FR-356-admit-option-value-type-definitions.md; excerpt: When a checked-package V2 reader selects a Semantic IR model containing a type definition of `kind` `option`, Contract IR SHALL admit that definition as `Option<T>` precisely under QSpec FR-154-AC-39..41 and FR-322-AC-298.
- TC-465 — role: examined; path: spec/checked_package/matrix/TC-465-named-option-value-types.md; excerpt: Read option model declarations through the public strict reader and compare the production result against the owning QSpec corpus, without copying it.


## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | The existing declaration-refusal regression expected document_pointer to remain absent after the reader added exact selected-document defect pointers, so the aggregate integration target failed on the stale envelope expectation. | tests/it/checked_package_v2_model_members.rs:2618 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | Updated the exact expected refusal envelope with /types/0/supertypes/0 while retaining the model-root locus, MissingDeclaration code, MissingName cause and whole-envelope equality assertion. |

## Disposition pass 1 evidence

Scoped inspection covered the additive-location correction in tc_048_a_declaration_refusal_is_located_at_its_model_root and the generic production defect-path construction. The test still constructs its expected envelope through refusal_cause and asserts the complete observed envelope; it now explicitly expects the precise selected-document pointer. This preserves the selecting root locus and strengthens location coverage without relaxing any assertion. The original review is preserved above. Final scoped disposition verdict: PASS with no open findings. The lead owns the final aggregate rerun; no Cargo or source edits were performed by this reviewer.
