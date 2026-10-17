---
id: SR-12133
title: "IR-851 gap analysis of IR PR #435"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@dcd84d128d6ce891d555032cb9767a4301d55fbe (PR #435); spec/checked_package/functional/FR-038-consume-checked-package-v2.md (AC-262..267), crates/quire-contract-model/src/checked_package/v2/{operations.rs,model_members.rs,model_fields.rs,mod.rs,parameter_declaration.rs}, tests/it/checked_package_v2_model_members.rs; consumer agent-ix/quire-contract-codegen PR #417 spec/kani/functional/FR-015-bounded-kani-obligations.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

# IR-851 gap analysis of IR PR #435

## Summary

Ticket: IR-851. This is a planless gap analysis; plan completion was not assessed. The PR is spec-only, so there is no production code to match. The analysis checks the spec's claims about origin/main code, whether the fixtures can be built, and whether the accessor fits its consumer. `quire coverage --strict` reports 43 unbacked rows (the baseline), because only PLANNED rows were added.

What was verified:

- **Retention.** `RetainedModels::new` runs after every validation stage (`mod.rs:969`), so retaining relationships there charges no work and moves no refusal. The `PartialEq` for `CheckedPackageV2` compares only `wire` and `kinds` (`mod.rs:559-563`), so the retained state is outside equality.
- **Admitted-claim guarantees.** On the reaches path, `navigation_type` refuses an unbounded or ordered destination end (`operations.rs:2261`, `model_members.rs:259-270`). `operations.rs:2265` refuses a heterogeneous relationship. So `ordered` false, `upper` Some and destination type == receiver end type hold for every admitted claim.
- **Fixtures.** The `reaches_over` / `relationship_operation_package` builders in `tests/it/checked_package_v2_model_members.rs:783-1100` take an arbitrary relation JSON and role, so AC-262/263 fixtures can be built from them. The document reader accepts `{1,1}` with `unique` false (`model_members.rs:1874-1897`). A missing target role is admitted (`role: Option`, AC-166).
- **Consumer fit.** CG #417 (FR-015-AC-105) needs `lower`, `upper` and `unique` for a finite end draw, and the accessor gives exactly those. CG's unbounded and ordered refusal paths for a relationship end cannot be reached from an admitted claim, because admission refuses them first.

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-038 measured-gap paragraph | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | RetainedModels keeps only the declaration index, the object types' field tables and the operation parameter types |
| FR-038 admitted-claim paragraph | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | For every admitted reaches claim the accessor therefore returns ordered false, upper Some |
| FR-038 UnresolvedEnd statement | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | or whose destination type has no model declaration node in the graph shall refuse UnresolvedEnd. Admission refuses each of those states ... no reader-admitted package returns it. |
| FR-038-AC-262 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | source end is {1,1} and whose target end is {2,3} unique with role peer |
| FR-038-AC-263 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | {0,3} unique, {0,3} not unique, {2,3}, {0,1} and {1,1} |
| FR-038-AC-265 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | or removes the destination type's model declaration node from the graph index ... the same four defects in a package given to the reader are refused at admission as before |
| FR-038 population deferral | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | Deriving a reaches claim's population is a separate decision (IR-852). |
| CG FR-015-AC-105 | context_only | spec/kani/functional/FR-015-bounded-kani-obligations.md (quire-contract-codegen PR #417) | a Collection field or finite multi-valued relationship end a count in [lower, upper] (a unique end admits no repeated target ...) |
| CG FR-015-AC-108 | context_only | spec/kani/functional/FR-015-bounded-kani-obligations.md (quire-contract-codegen PR #417) | While the population selection for the claim is Unavailable (no IR mapping from the claim to a population, IR-851), the claim settles unsupported naming it. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The fourth UnresolvedEnd cause, "destination type has no model declaration node in the graph", can be reached from a reader-admitted package. The spec says admission refuses it and AC-265 says "refused at admission as before". On the reaches path, admission computes the destination key but never looks it up in the graph: the result is Boolean (`operations.rs:2245-2272`). Operands are checked through `conforms_to` over the domain model's ancestors (`model_members.rs:901-907`), and the declaration index is built from documents, not graph nodes (`mod.rs:2015-2022`). So subtype operands (`Reference<Subtype>`, AC-253) admit even when the declared end type's own declaration node is absent. On such a package the accessor would refuse a legitimate claim, and CG would settle `unsupported`. Not reproduced; traced. Fix: return `destination_type` as the derived id (`quire.checked-semantic-node/v1` plus the declaration key) whether or not the node is in the graph, and say it may be absent from the graph. Drop the fourth cause from UnresolvedEnd and from AC-265. Add a subtype-operand case to AC-263. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3262-3267, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4612, crates/quire-contract-model/src/checked_package/v2/operations.rs:2245-2272 |
| FND-002 | medium | Cross-repo trace. CG #417 FR-015-AC-108 (and the Description sequence, the TC-025 row and the tests.md row) names IR-851 as the source of "the IR mapping from the claim to a population". This PR correctly leaves that out and points to IR-852. Once this merges, CG's `Unavailable` disposition and its success gating cite a ticket that will never deliver the mapping. This IR spec is correct; the fix is in CG #417: re-point the population half to IR-852 and keep IR-851 for relationship-end multiplicity only. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3279-3280 |

## Verdict

PASS WITH FINDINGS. The key claim (no relationship retention today; retention outside equality, uncharged, refusal order unchanged) is true and can be implemented. RetainedModels already retains derived operation types the same way, after validation. The result fields give CG what it needs. FND-001 must be fixed in this PR. FND-002 is an action on CG #417.
