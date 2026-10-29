---
id: SR-7705
title: evidence review of IR-490 structural forwarding specification
type: SpecReview
analysis: evidence
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md;
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; spec/checked_package/matrix/tests.md;
  baseline crates/quire-contract-model/src/checked_package/v2/operations.rs and checked_package/v2/mod.rs
review_set: subset
---

## Summary

Ticket: IR-490. Independent specification-only review of PR #348. The deterministic quoin advise record for FR-038-AC-224 normalizes authored Test, reports mismatch false and inconclusive false, and recommends Test-class metamorphic/property methods from idempotence along with Analysis and monitoring suggestions from temporal wording. Test is appropriate for observed meter deltas and typed exact/one-less outcomes; this is reviewer judgement on the advisor recommendations. Planned/unrun status correctly claims no executed test binding.

## Verdict

**PASS** — no defects in this method's scoped checks.

## Examined Scope

```yaml
scope:
- id: FR-038 structural forwarding charge
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'For an application''s operation validation, resolving a structural type
    through

    an `alias` or a non-`model_population` bounded domain shall charge one work

    unit for each distinct forwarding node first entered.'
- id: FR-038 structural memo scope
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The resolution memo belongs to that one application''s

    operation validation, including its union-arm constraint, inner-type checks,

    result-inner selection, and both passes of its leaf walk.'
- id: FR-038 typed work refusal
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'A failed charge returns

    the reader''s typed `incomplete`/`work` outcome at the operation value being

    validated, including from a union-arm predicate or a leaf edge; it shall not

    be turned into an absent type, an `ill_typed` refusal, or an unchecked leaf.'
- id: FR-038-AC-224
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'For a 300-field record naming different entries of one 300-alias chain,
    measure total operation work W under a nonbinding limit: W reaches the ordinary
    `operation-law-missing` decision and W-1 returns typed Work. The total retains
    independent leaf-visit and `text_profile` pin charges, while the two leaf passes
    in that application together charge each of the 300 structural forwarding nodes
    once; repeated pin lookups retain their separate first-visit charge rule. '
- id: TC-048 isolated structural forwarding probe
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: 'PLANNED/UNRUN (IR-490). Use a private operation-stage probe with an isolated

    meter to resolve a chain containing both `alias` and non-population

    `bounded_domain` nodes. Measure the before/after counter for a chain with N

    distinct forwarding nodes: the first resolution consumes N units, the same

    head and each resolved suffix consume zero more in the same application''s

    validation, and a second application consumes N again. The terminal node is

    uncharged. At exactly N units the first resolution succeeds; at N-1 it returns

    the typed Work '
- id: TC-048 shared-chain control
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: 'At the operation stage, reuse the 300-field/300-alias shared-chain case
    of

    AC-44. Under a nonbinding limit, measure total work W for the ordinary

    missing-leaf decision. The same decision holds at W; W-1 returns typed Work.

    Decompose W with operation-stage counters: the two leaf passes together charge

    one unit per distinct structural forwarding node, while leaf visits and the

    existing `text_profile` pin each keep their own charges. Preserve the pin

    control by verifying that the first lookup o'
- id: TC-048 four caller controls
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: 'that reach structural resolution through `union_arms`, `inner:0`,

    `result_inner`, and a leaf edge. Give each its measured exact remaining budget

    and one less; the exact case reaches the ordinary operation decision, while

    the one-less case returns the reader''s typed `incomplete`/`work` at that

    operation. Assert no case turns Work into `ill_typed`/`operator-ineligible`,

    a skipped union constraint, or an unchecked leaf. Bind each eventual test to

    `FR-038-AC-224`; no runtime result'
- id: TestMatrix pending AC-224
  path: spec/checked_package/matrix/tests.md
  role: examined
  excerpt: "| FR-038 | FR-038-AC-224 | TC-048 | \U0001F6A7 PLANNED/UNRUN; no tagged\
    \ implementation test yet | First resolution of each distinct alias or non-population\
    \ bounded-domain forwarding node costs one work unit per application, repeated\
    \ resolution within the application costs zero, and a second application charges\
    \ again; exact/one-less meter boundaries; a shared 300-alias chain across two\
    \ leaf passes with separate text-profile pin charges; typed work exhaustion through\
    \ union arms, inner type, result inner and leaf edges, without a skipped constraint\
    \ or ill-typed fallback. |"
- id: FR-038-AC-44
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: '`operation.leaves` is exactly the derived leaves, the text leaves and,
    for a type that reaches itself, its recursion leaves (FR-038-AC-70, FR-038-AC-71):
    a record with `a`, `b` and `c` (an option of text) fields over a `text_bounds`
    text type that binds the `nfc` profile admits the leaves `["field:a"]`, `["field:b"]`
    and `["field:c", "inner"]`, a tuple the `position:<n>` segments, and `collection.contains`
    over a set of text the one empty path, each carrying exactly one catalogued `text_profile`
    law the lock selects and the mode `{kind: text_profile, value: nfc}`; a text leaf
    whose t'
```

## Source-ground Accounting

Baseline structural_type at operations.rs:2239-2261 follows alias and non-population bounded-domain semantic_type without WorkMeter. Callers are union_type:1329-1330 through the bool union_arms_hold/check_operands constraint, inner_type:2270-2276 through inner:0 and result_inner, LeafWalk::edge:2677-2683, and direct result_inner selection:3131. All four are named by the amendment. WorkMeter::charge at mod.rs:494 increments consumed work and returns typed incomplete Work before the caller continues. Current LeafWalk Work propagates at operations.rs:3250; unresolved remains its existing separate refusal. text_profile_pin:2307-2350 has its own walk memo and charge. The new operation memo must span check_operands through check_leaf_count within one operation_defect; the next application call in validate_operations gets a new memo. No source implementation or executed runtime proof is claimed.

The existing 300-field alias-chain test at operations.rs:5508 supplies empty leaves and reaches Missing. check_leaf_count:3166-3173 returns before first_leaf_fault when the supplied count is short.

## Validation

Exact GitHub head was verified independently. Scoped Quire validation of all three changed files and git diff --check passed. No applicable AssuranceProfile or required review selection exists under spec/. make spec retains the inherited strict coverage failure (48 unbacked rows, 0 contradicted statuses); it is not classified as an introduced deficit. No Cargo or code tests ran for this Markdown-only diff. No source or author worktree was modified.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
