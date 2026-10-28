---
id: SR-612062
title: "IR-1059 slice 1 code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 634 Rust diff in crates/quire-contract-model/src/checked_package/v2 and tests/it/checked_package_v2_model_members.rs; FR-069-AC-1,4,6"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-069
    type: reviews
---

## Summary

Ticket: IR-1059. Reviewed PR #634 at 127a8932459e6084b62ed058d14766b6bddf2ddf, limited to tuple and variant payload selected intake, model owner, vocabulary, and general type-family use. The slice leaves the complete FR-069 conformance and projection work open.

## Verdict

**FAIL** — A payload-bearing model variant is recorded as a union by the new selected model map, but the general reader family resolver still classifies that same declaration as an enum. No full Cargo gate was run during review; hosted checks were in progress. The diff changes no CI workflow, vendored files, unsafe code, or dependency graph.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `resolve_family_with` returns `enum` for every owner-backed model variant, even when the selected cases have a payload. The new `variant_family` override is only used by `composite_operands`; `literal_defect` and application argument checking use the old resolver. A payload-bearing `Maybe` is therefore treated as enum for those checks, permitting/rejecting operations under the wrong family rather than the union family required by FR-069. | crates/quire-contract-model/src/checked_package/v2/operations.rs:1421 |

## Coverage

- FR-069-AC-1, role: examined. Excerpt: “The authoritative positive model tuple and union package admits with `model`/`tuple_value_type` declarations for `Pair`, `Swap`, `Nest` and `Cell`, owner-nested position identities, ordered zero-based positions and their exact type references; no position is a graph node or source-map key.” Checked the production tuple intake and selected owner join; the published fixture/vector comparison is deferred outside slice 1.
- FR-069-AC-4, role: examined. Excerpt: “A payload-free variant has family `enum`; a mixed nullary/payload variant has family `union`, case order is ascending UTF-8 name, and payload cases retain exactly one resolved position-zero type.” FND-001 affects the family used by general reader paths.
- FR-069-AC-6, role: examined. Excerpt: “Every published intake and wire mutation returns its exact QSpec code, cause and location at its specified stage, including duplicate or gapped tuple positions, absent/unsupported/unresolved position `typeRef`, illegal tuple members, unresolved/unsupported `Variant.payloadType`, malformed owner, stale tuple key and owner-kind mismatch.” The new tests cover a local subset only; full conformance remains open.
- FR-069-AC-2, AC-3, AC-5, AC-7 and AC-8, role: context_only. They describe later identity, projection, published operator, writer and conformance work outside this urgent slice.
- Changed files checked for Rust API, panics and wire boundaries: composite_operands.rs, derived_keys.rs, identity.rs, lower.rs, mod.rs, model_fields.rs, model_members.rs and tests.rs, operations.rs, owner.rs, structural.rs, vocabulary.rs, checked_package_v2_model_fields.rs and checked_package_v2_model_members.rs.
- Plan completion: not assessed.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | At 4831e13a, the selected model-owner lookup supplies union/enum family to the common resolver at each visited type node. Literal typing, operation operand, type-argument member result, inner result and mode checks now use that selected resolution; `composite_operands` uses the same selected-aware walk. A bounded-domain chain reaches its selected variant before the fallback enum branch. A focused structural-equality versus enum-equality regression test was added. |

## Disposition verdict

Round 1 at 4831e13a0f472f468ad927d77c17b8168df89a72: FND-001 fixed on inspection. The focused regression and hosted checks were still running at this review point; the lead owns the final gate result.
