---
id: SR-202611070951309550
title: PR993 lazy protocol clause code-review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; IR-1404; protocol_body.rs clause callbacks and
  native regression
review_set: subset
---

## Summary

Ticket: IR-1404. Independently reviewed frozen PR993 at 36fd4b40 against main161d2516. The only production change moves clause membership, activation-guard and capture-initializer JsonPointer construction into the existing lazy callbacks. Inspected every callback consumer: check_target and boolean call their FnOnce only for a refusal, and check_member_targets invokes its Fn only for a missing reference. The copied closure captures immutable check context and the current scalar index; it cannot escape, and the graph does not change between target resolution and wrong-kind refusal. Target predicates, evaluation and logical-work order, causes, final pointer text and node loci are unchanged. Successful checks no longer allocate the discarded diagnostic path. No public interface, helper, trait, budget, algorithm, dependency, fixture copy, ignored test, gate or warning suppression is introduced. No applicable AssuranceProfile was found under spec/.

## Verdict

PASS for the changed clause callback family: no introduced findings. Whole-corpus read qualification remains unproven and existing repository gaps remain open.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Retained selected Rust1.98.1 workspace native test PASS1 (compile21.95s/test0.01s). The test builds actual typed node/term/index/profile data using the existing protocol body fixtures and production envelope/layout validation, then calls actual check_clause or check_clause_slots through MemberCheck and WorkMeter. It covers no list members, multiple members across all six list classes, each-activation Boolean guard and a capture initializer. No production unit is mocked or replaced. The counting allocator measures the complete helper call, checks every actual request against caller-admitted scratch storage, and exercises ample/exact/one-short/zero/repeated budgets and repeat work deltas. Restoring an eager pointer at the reached list, guard or capture site would add an unreserved measured allocation and fail that equality. Fixture setup is outside measurement; this is helper-family evidence, not a complete public-read fixture proof. Initial fixture type/admission/default-budget failures were corrected in test setup and retained separately, not counted as PASS. Final workspace/all-target Clippy conformance-cli,test-support -D warnings PASS16.36s; scoped formatting and diff checks pass. The final production lint-only correction passes the same Copy closure directly instead of borrowing it, with unchanged native test body and no behavioral change; no runtime was repeated for that correction. Logs: /tmp/ir1404-protocol-clause-{native,clippy,original}.log. No Cargo, Kani, make ci, whole-binary or aggregate command was run by this reviewer.

The unchanged full typed_protocol_fairness_preserves_each_selected_target remains RED at its FIRST admitted read: actual requests1716988 versus admitted1715434, a1554-byte gap before its later semantic assertions. That red is retained explicitly; no oracle, semantic assertion or input was weakened. Earlier894-byte observations used a different cohort and are not an improvement baseline or acceptance oracle. Other protocol control paths, named-field collection and negative diagnostic factories are unchanged and unqualified by this slice. This PASS permits the introduced callback correction only; whole-read AC511, normative AC512 corpus qualification and AC513 remain UNPROVEN.

Reconciliation: quire matrix (quire0.38.1, engine0.51.0), full repository static axis; no evidence-store run evidence read. Counts: tagged1028, untagged92, tagged-by-ignored-test8, method-without-symbol26. Existing repository-wide gaps remain open; this introduced-change review does not assert repository-wide assurance PASS. The native AC511 binding is truthful partial allocation-accounting evidence and does not claim the criterion's entire public-read corpus or every allocation class. The changed callbacks are owned by the existing FR-038 strict-read accounting and unchanged protocol-target rules. No new unspecified surface, unknown criterion tag, source/test stub, internal mock, skip, threshold reduction or coverage inflation was found in the changed scope. No spec/plan changed, so spec-review methods do not apply. Optional broad semantic audit was not requested; ordinary changed-code/test agreement was inspected.

Plan completion: not assessed

Full reviewed scope:

```yaml
scope:
- id: check_clause
  path: crates/quire-contract-model/src/checked_package/v2/protocol_body.rs
  role: examined
  excerpt: let path = || check.argument_path(ordinal).key("members").index(entry);
    let position = check.check_target(target, path, |_| true)?;
- id: check_clause_slots
  path: crates/quire-contract-model/src/checked_package/v2/protocol_body.rs
  role: examined
  excerpt: let path = || check.argument_path(1).key("members").index(2).key("value");
    check_member_targets(check, guard, &path)?; ... check.boolean(guard, path)?;
- id: check_clause_slots::capture_initializer
  path: crates/quire-contract-model/src/checked_package/v2/protocol_body.rs
  role: examined
  excerpt: let path = || check.argument_path(2).key("members").index(entry).key("members").index(1).key("value");
    check_member_targets(check, initializer, &path)?;
- id: MemberCheck::check_target
  path: crates/quire-contract-model/src/checked_package/v2/protocol_body.rs
  role: context_only
  excerpt: 'fn check_target(&self, term: &CheckedTerm, path: impl FnOnce() -> JsonPointer,
    expected: impl Fn(usize) -> bool) -> Result<usize, ValidationFailure>'
- id: MemberCheck::boolean
  path: crates/quire-contract-model/src/checked_package/v2/protocol_body.rs
  role: context_only
  excerpt: 'fn boolean(&self, term: &CheckedTerm, path: impl FnOnce() -> JsonPointer)
    -> Result<(), ValidationFailure>'
- id: check_member_targets
  path: crates/quire-contract-model/src/checked_package/v2/protocol_body.rs
  role: context_only
  excerpt: 'fn check_member_targets(check: &MemberCheck, term: &CheckedTerm, path:
    &dyn Fn() -> JsonPointer) -> Result<(), ValidationFailure>'
- id: healthy_clause_members_guards_and_captures_build_no_diagnostic_paths
  path: crates/quire-contract-model/src/checked_package/v2/protocol_body.rs
  role: examined
  excerpt: fn healthy_clause_members_guards_and_captures_build_no_diagnostic_paths()
- id: typed_protocol_fairness_preserves_each_selected_target
  path: tests/it/checked_package_v2_protocol_body.rs
  role: context_only
  excerpt: fn typed_protocol_fairness_preserves_each_selected_target()
- id: FR-038-AC-511
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "Whole-read accounting, including each read's owned catalog. A test binary\
    \ whose global allocator counts, on the reading thread during the call, the size\
    \ of each allocation request and the new size of each reallocation request reads,\
    \ under an unlimited budget, an exact budget and each one-below budget of AC-497\
    \ through AC-507: every positive TC-048 fixture, the QSpec checked-package conformance\
    \ packages this repository reads, and the adverse shapes \u2014 the 100000-node\
    \ reference chain, a 10000-node flat graph, a node with 10000 dependencies, a\
    \ 1 MiB string member, 10000 source-map entries, 10000 d"
- id: FR-038-AC-103
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: A clause whose `over` argument is a `reference` to a `value`/`parameter`
    node among its `dependencies` admits; one whose `over` references a `value`/`parameter`
    node that is not among its `dependencies` refuses `missing_declaration`/`missing-name`,
    and one that references a declared dependency that is a `scalar_type` node refuses
    `invalid_model_binding`/`malformed-declaration`, each with path `/semantic_graph/nodes/{clause}/body/arguments/0`
    and the locus of the "Path and locus" table; a fairness member whose `declaration`
    and `name` resolve to an operation of a `model`/`object_type` declarati
- id: TC-472
  path: spec/checked_package/matrix/TC-472-strict-v2-read-exact-budget-charging.md
  role: context_only
  excerpt: Verify that the one strict V2 read charges every metered work step and
    every heap allocation it makes to the caller's NativeJoinBudget before the step
    or allocation.
```
