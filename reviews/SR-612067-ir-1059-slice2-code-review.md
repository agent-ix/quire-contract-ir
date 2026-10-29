---
id: SR-612067
title: "IR-1059 slice 2 model union code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 636 operations.rs and tests/conformance_qspec/model_tuple_union.rs; FR-069-AC-4/5/8"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-069
    type: reviews
---

## Summary

Ticket: IR-1059. Reviewed PR #636 against main. The published QSpec full-document fixture is consumed from its authoritative checkout and the selected model union now has a case-member path, but that path cannot resolve every payload type accepted by selected intake.

## Verdict

**FAIL** — a legal model union payload can be retained at selected intake and then be treated as no union member type by the new reader path. Hosted Rust Checks at this head failed TC-058 on FR-019 prose already corrected in PR #635, outside this diff; no full local gate was run. The diff changes no CI workflow, dependencies, unsafe code or vendored files; `git diff --check` passed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `selected_union_members` calls `DomainModel::element_type` and returns `None` for its entire union when a payload has a selected type that `element_type` does not map, including native `Rational`, `Decimal`, `Float32`, `Float64` and bare `Text`. Selected intake admits those non-Clock payloadType references, so a `union_value` of such a case refuses `ill_typed`/`type-mismatch`, and a case arm becomes ineligible, instead of resolving its required position-zero payload type. | crates/quire-contract-model/src/checked_package/v2/operations.rs:1791 |

## Coverage

- FR-069-AC-4, role: examined. Excerpt: “A payload-free variant has family `enum`; a mixed nullary/payload variant has family `union`, case order is ascending UTF-8 name, and payload cases retain exactly one resolved position-zero type.” The BTreeMap preserves name order; FND-001 affects some valid payload types.
- FR-069-AC-5, role: examined. Excerpt: “The published `union_value`, `enum_value`, equality, case-arm and structural-leaf cases pass through the production reader.” The new path covers union_value and case-arm positive forms for the selected types it can derive; FND-001 affects legal payloads beyond the current fixture.
- FR-069-AC-8, role: examined. Excerpt: “`make conformance-qspec` loads the positive fixture, `model-tuple-union-vectors.json`, its full Semantic IR `selected_document` ... compares the document's derived `domain_package` projection with the vector, hashes the full document for the selection digest, recomputes identities and runs every tuple/union case.” This PR tests full-document digest and positive package admission only, and states full TC-940 remains open.
- FR-069-AC-1/2/3/6/7, role: context_only. Remaining tuple identity, projection, mutation and writer obligations remain open across IR-1059 slices.
- Plan completion: not assessed.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | still-open | adds FR-322 step-4 mappings for native Float32 and Float64 to scalar type nodes and a focused ordered-case key test. The original finding's Rational, Decimal, bare Text and unbounded value-type examples remain: selected intake accepts them, while `DomainModel::element_type` returns `None` and `selected_union_members` returns no members. QSpec STD-532 is open for a normative node mapping or intake refusal at the exact stage; IR must follow that ruling. |
| FND-001 | fixed | now refuses unsupported Rational, Decimal, bare Text and unbounded declared value-type payloads during selected model intake with `unsupported_construct`/`declaration-form`, before union member resolution. The refusal carries the model selection row and the exact selected-document `Variant.payloadType` pointer. Supported Float32/Float64 and known unit payloads remain admitted; a focused unit and ordered-case regression asserts their member node keys. FR-322 defines unsupported payload intake refusal; STD-532 still governs any later positive mapping. |

## Disposition verdict

Round 1: Float32/Float64 portion fixed; FND-001 remains open for the omitted selected type families under QSpec STD-532. The focused test and hosted checks are still running at this review point.

Round 2: FND-001 fixed by typed selected-intake refusal for unsupported payloads and the exact `Variant.payloadType` document pointer. Formal Rust-review of this narrow final-head fix found no additional defect. Focused tests and hosted gates remained queued at review time; this is a code and coverage disposition, not a gate result.

## Final-head Rust review addendum

I inspected the follow-up compiler repairs against the initial review: the selected unit/float member assertion uses the unambiguous name comparison, and the public model-field test handles Float32/Float64 exhaustively in its type adapter, node builder, perturbation helper and closed-shape check. The supported unit payload still resolves through `model.unit_refs` and the MUnit quantity key assertion. No new Rust-review finding. The coder reported focused unit, conformance QSpec and public-shape tests passing at this head; hosted final gates remain the lead's merge check.
