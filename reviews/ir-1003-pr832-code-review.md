---
id: SR-2026110283201
title: "IR-1003 PR 832 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1003 PR 832; tests/it/checked_package_v2_reader.rs and tests/it/checked_package_v2_model_fields.rs; FR-352-AC-1 through FR-352-AC-4"
review_set: subset
---

## Summary

Reviewed only FR-352 AC-1 through AC-4, both changed integration-test files and the production projection/selected-row joins they exercise. No production, specification, dependency, lockfile or CI change is present. The public integration boundary uses admitted packages and real Exact values. No applicable AssuranceProfile was found.

Scalar tests assert Boolean, Integer and all three required inclusive intervals, plus endpoints beyond host integer widths. Exact uses arbitrary-precision integers; no canonical integer endpoint lies outside that representable domain, so an invented ceiling/refusal would be incorrect. Record assertions compare the entire ordered identity/name/MemberId/type/presence tuples for Receipt, Attempt (inherited optional member), Flag (redefinition), and unread Tick.count. Exact construction rejects absent count and count=10 with typed causes. Missing field witnesses and foreign records refuse via Result without a view.

Reference assertions compare each selected object's actual EffectiveId, reject standalone objects and missing witnesses, and prove foreign/dangling targets fail admission at the reference locus before projection. Identity tests cover changed full path, ordinal, package, selected document, original identity and form; duplicate, conflicting and ambiguous owners refuse at acquisition. Same display names remain distinct and a selected-document alias cannot join.

## Verdict

**PASS** for this four-criterion subset; no findings. The code-review includes the Rust lane. This does not assert whole-repository coverage or aggregate release qualification.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-352-AC-1 — role: examined; excerpt: An external crate reads an admitted node's semantic type through one public IR-model API and receives Exact `Boolean`, `Integer`, or `Int` for independently authored Boolean, Integer, and inclusive ranges `[0,1000]`, `[0,3]`, and `[0,9]`; it inspects no JSON and reparses no source. An endpoint outside Exact's representable domain returns a typed refusal, never a narrowed interval.

- FR-352-AC-2 — role: examined; excerpt: Through the public `ExactTypeProjection`, admitted selected `Receipt`, `Attempt`, `Flag`, and `Tick` return distinct `Composite(NodeKey)` identities and public `SelectedRecordView` values. Each view contains every effective member in selected declaration order with selected IR identity, name, Exact `MemberId`, projected type and presence; `Tick.count: Int[0,9]` appears even with no read of `count`. The view can supply Exact `FieldDeclaration` inputs that reject a complete record missing a required member or carrying an out-of-range `count`; a member identity absent from the producer, or a similarly named record in another package, refuses without a partial view.

- FR-352-AC-3 — role: examined; excerpt: An admitted reference to selected `Node` projects to `Reference(EffectiveId)` matching its authoritative object-type witness. The object declaration alone is not returned as a standalone Exact value-type variant. A reference to another selected object type, a foreign package, or a dangling target never produces Node's identity.

- FR-352-AC-4 — role: examined; excerpt: Exact structural identity is matched by package, full resolved path, and ordinal. A changed path segment, ordinal, owner package, or selected declaration refuses with a named typed outcome; duplicate or conflicting witnesses and ambiguous selected owners refuse before any value-type result. Alias equality and display-name equality cannot establish a join.

The computed `quire matrix --scope . --format json` reports these four rows tagged by exactly their four real, nonignored integration tests. No hand-written matrix was added. Existing scanner evidence reports unbacked rows 94 to 90 and zero contradicted, exactly these four rows; unrelated residuals are outside this review. No new reverse gap or stub exists in this test-only diff. Semantic intent/test/code correspondence was explicitly requested in the review brief and inspected above.

Reused qualification receipts in `/home/peter/dev/qualification-logs/ir1003-projection/`: scalar.log, records.log, identity2.log and reference2.log each show one test passed; fmt.log and lint.log show formatting and workspace/all-target plus default-model Clippy success. Earlier reference failure was superseded by the final passing reference receipt. No duplicate heavy Cargo runs were performed. Aggregate tests and hosted Test are pending the lead's final gate; this review does not claim those passed.

Plan completion: not assessed
