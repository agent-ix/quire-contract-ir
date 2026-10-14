---
id: SR-7849
title: "Base requirements review — IR-760 selected population specification"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/model/functional/FR-019-rust-library-interface.md, spec/model/functional/FR-053-selected-population-binding.md, spec/model/matrix/TC-453-selected-population-binding.md, spec/spec.md"
review_set: subset
---

## Summary

The new requirement leaves the selected finite maximum vulnerable to a retargeted admitted parameter type, omits the requested typed unknown-closure outcome, and has no planned TestMatrix entry.

## Verdict

**FAIL** — The new contract has unresolved correctness and tracking gaps.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The selected source maximum is not bound to the selected occurrence across an admitted V2 package | spec/model/functional/FR-053-selected-population-binding.md:102 |
| FND-002 | medium | Unknown closure has no typed result despite the ticket acceptance | spec/model/functional/FR-053-selected-population-binding.md:105 |
| FND-003 | medium | The new criteria have no planned TestMatrix entry | spec/model/functional/FR-053-selected-population-binding.md:102 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The selector now names the exact population node and occurrence, reads the maximum from that node, and requires its exact source-map occurrence. |
| FND-002 | fixed | A typed QSL UnknownClosure input maps to a distinct UnknownClosure output while retaining source identity, provenance, and cause. |
| FND-003 | fixed | TestMatrix now has planned FR-053/TC-453 rows with explicit AC coverage, including the retargeted-pointer adversary. |
### Disposition evidence

FND-001 after excerpt: `The selector SHALL name that exact population node and occurrence. The query SHALL verify that the named node has the admitted bounded_domain / model_population shape, read N from its own max binding, and require a source-map entry for the same population node and exact occurrence before returning the extent.`

FND-002 after excerpt: `The model crate exposes select_population, CheckedSelectedPopulationSource, CheckedSelectedPopulationSelector, CheckedSelectedPopulation, and CheckedSelectedPopulationOutcome in checked_package. A finite result retains the selected package identity, population binding node identity, occurrence, selected model object type identity, inclusive extent [0, N], and the authentic source regions associated with that exact node-occurrence pair. The outcome is typed and distinguishes Finite, Unbounded, UnknownClosure, Unavailable, MissingOccurrence, StaleBinding, ForeignPackage, and NotPopulation.`

FND-003 after excerpt: `| FR-053 | FR-053-AC-1 through FR-053-AC-5 | TC-453 | 🚧 planned for IR-760: TC-453 will tag the QSL-produced direct population-node/occurrence positive, zero bound and distinct maxima (AC-1/2); retargeted-semantic_type and unmatched-occurrence adversaries (AC-2); typed stale, foreign, missing, non-population, unbounded and unavailable outcomes (AC-3/4); the QSL typed UnknownClosure pre-admission mapping (AC-4); and the public model-only consumer and exclusive model_object_fields multiplicity authority (AC-5). No implementation test tags exist yet.`

