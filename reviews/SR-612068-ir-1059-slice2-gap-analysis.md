---
id: SR-612068
title: "IR-1059 slice 2 model union test gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 636 tests/conformance_qspec/model_tuple_union.rs and operations.rs; FR-069-AC-4/5/8"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-069
    type: reviews
---

## Summary

Ticket: IR-1059. Planless, PR-diff-scoped acceptance-to-test review. The QSpec positive fixture is read through production intake, but the test asserts only digest matching and package admission; it does not trace a whole FR-069 AC.

## Verdict

**CONDITIONAL** — no changed test exercises the newly added selected-union path's failure behavior for a legal payload type outside the positive fixture, such as native Rational. No false full-AC trace tags were added. The complete TC-940 conformance remains open.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new positive-fixture test covers only the published Boolean/tuple payloads. It never mutates the selected document to a legal native payload such as `Rational` and checks that the resulting `union_value` and case-arm types resolve. Thus the partial test passes while the new `selected_union_members` path silently returns `None` for an admitted payload type. | tests/conformance_qspec/model_tuple_union.rs:14; crates/quire-contract-model/src/checked_package/v2/operations.rs:1791 |

## Coverage

- FR-069-AC-4, role: examined. Excerpt: “payload cases retain exactly one resolved position-zero type.” Only the current fixture's payload types are exercised.
- FR-069-AC-5, role: examined. Excerpt: “The published `union_value`, `enum_value`, equality, case-arm and structural-leaf cases pass through the production reader.” The new test asserts package admission, not each case's exact outcome; full vector conformance remains open.
- FR-069-AC-8, role: examined. Excerpt: “`make conformance-qspec` loads the positive fixture ... compares the document's derived `domain_package` projection with the vector ... and runs every tuple/union case.” This new test intentionally covers only full-document positive admission and does not claim full AC8.
- FR-069-AC-1/2/3/6/7, role: context_only. These remain open across slices.
- Reverse trace: selected model union read behavior is owned by FR-069; no unowned source found in this diff. The new test has no FR-069 or TC-940 trace tag, consistent with partial coverage.
- Plan completion: not assessed.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | still-open | Head b20fbe77 adds a selected Float32/Float64 ordered-case node-key unit test, so that defined native float branch is covered. Rational, Decimal, bare Text and unbounded value types remain accepted at intake without a resolved union member node; no focused production reader test covers their normative outcome. QSpec STD-532 must settle mapping versus intake refusal before this residual can be closed. |
| FND-001 | fixed | Head 41ab6e85 adds a production `CheckedPackageV2::read` test that mutates the authoritative selected document to Rational, Decimal, bare Text and unbounded value-type payloads and asserts `unsupported_construct`/`declaration-form` at the exact selected `Variant.payloadType` pointer. The selected Float32/Float64 and known unit positive member-node regression protects the admitted cases. The full FR-069/TC-940 matrix remains open as already scoped. |

## Disposition verdict

Round 1 at b20fbe77a5274273122e7f2b18b54732bbb90eab: the defined float coverage is fixed; FND-001 remains open for the QSpec STD-532 type families. No additional broad review sweep was run.

Round 2 at 41ab6e85cea7b24ca5d46b18624255df8bee5c60: FND-001 fixed by the focused production-reader refusal matrix and supported payload node-key test. Focused tests and hosted gates remained queued at review time; no full review sweep was repeated.

## Final-head addendum

At 0b5052d001841b2739a35d35a81d6486c342df6b, the compiler-only follow-up leaves the unsupported-payload production-reader matrix and positive selected-unit/float member test intact. The coder reported focused tests passing; full FR-069/TC-940 coverage remains open as stated in this artifact.
