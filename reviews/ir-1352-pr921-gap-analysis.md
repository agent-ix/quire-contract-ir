---
id: SR-2026110602354602
title: "IR-1352 PR 921 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1352; PR #921; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs"
review_set: subset
---

## Summary

Compared FR-038-AC-473 and QSpec FR-440 with the five new traced public-reader tests. The arm-local, sibling, shared-body, enclosing sibling, stage binder, and barred-state paths have value or typed-refusal assertions. A required scrutinee escape path is not covered.

## Verdict

**CONDITIONAL** — add a typed refusal control for an arm binder reached from the case scrutinee.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-473 requires a scrutinee reference to an arm binder to refuse with `MissingDeclaration`/`MissingName` and `BinderScope` detail, but all five new tests use a valid `circle` scrutinee; the only outside-arm test changes an enclosing comparison operand. Add a type-correct scrutinee escape refusal assertion. | tests/it/checked_package_v2_protocol_body.rs:10849 |

## Examined scope

- role: examined; id: FR-038-AC-473; excerpt: "A body reference to the other arm's binder, including one in a shared body node reached under both arm scopes, and a reference to an arm binder from the scrutinee or an enclosing sibling expression each refuses `missing_declaration`/`missing-name` at the protocol owner with `BinderScope` detail naming that binder, slot owner, stage and `read`."
- role: context_only; id: FR-038-AC-465/466; excerpt: "A protocol slot's lexical let/query/count/sum binders are in scope only inside their bound expression bodies, including nested binders and shared graph nodes reached under different lexical scopes."

Plan completion: not assessed. The optional repository-wide semantic audit was outside this diff-only review; no Cargo run.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 at PR head `38008979264d250881f0ed98ffa81b8e93364e9b`: a traced public-reader test admits a recursive Shape baseline, then makes the case scrutinee reference the arm binder and asserts MissingDeclaration/MissingName plus protocol owner and BinderScope details. |

## Disposition round 1

**PASS for this finding.** The new baseline proves the recursive union wire is admissible before the one-field scrutinee mutation. The refusal assertion reaches the existing helper and checks typed code, cause, owner and detail. No new finding in the fix diff. Final scoped Cargo gate remains with the lead.
