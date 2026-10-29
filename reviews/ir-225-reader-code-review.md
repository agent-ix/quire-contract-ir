---
id: SR-11674
title: code-review and rust-review of IR-225 catalogue-driven member-result admission
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; PR 409 branch code/ir-225-member-family; diff origin/main...HEAD;
  crates/quire-contract-model/src/checked_package/v2/operations.rs, tests/it/admitted_guard_context.rs, tests/it/admitted_result_context.rs, tests/it/checked_package_v2_ieee_admission.rs, tests/it/checked_package_v2_operand_families.rs, tests/it/checked_package_v2_recursive_leaves.rs, spec/checked_package (FR-038, TC-048, tests.md)
review_set: subset
---

## Summary

Ticket: IR-225. PR: quire-contract-ir#409. Method: code-review with the
rust-review lane folded in. The authority is the merged spec (PR #408):
FR-038-AC-247 through FR-038-AC-251 and the TC-048 IR-225 steps. The catalogue
is `contracts/checked-operation-catalog-v1.json` in quire-verification-contracts,
locked.

The catalogue has 21 entries with `result: member` and `member: type_argument`.
19 of them carry a `member_family` constraint. `quire.op.collection.fold` and
`quire.op.collection.reduce` carry none: their only constraint is
`bound_is_member`. Two further entries have `member: type_argument` and a
different result form: `quire.op.model.lookup` (`result: absence`) and
`quire.op.model.all_instances` (`result: set`). The QSL producer emits all four.
See qsl-semantics/src/check/lowering.rs:3633-3667 and :3980-4000 at QSL main.

Selection is driven by the entry's shape, not by a list of names. An entry is
selected when `entry.member == type_argument`, or when `result == member` and
the entry has a `member_family`. A selected entry is then guarded: when its
result is not `member`, or it has no `member_family`, the guard refuses the
package. Neither arm checks whether the entry is one of the 21. The guard
therefore refuses every application of the four entries above.

The reviewer probed `CheckedPackageV2::read` with scratch tests (not
committed). Each package was rebuilt with fresh identities from the
operand-families fixture. The same packages were read once with the HEAD
reader and once with main's `operations.rs`:

| Identity (equal declaration and result unless noted) | main | HEAD |
| --- | --- | --- |
| collection.fold (integer) | admitted | **refused** ill_typed/operator-ineligible at `/body/result_type` |
| collection.reduce (integer) | admitted | **refused**, same |
| model.all_instances (result Set) | admitted | **refused**, same |
| model.lookup (result Option, mode absence) | admitted | **refused**, same |
| collection.count (integer) | admitted | admitted |
| collection.count (boolean, family mismatch) | admitted | refused at `result_type` (intended) |
| collection.sum.integer equal / mismatch | admitted / admitted | admitted / refused (intended) |
| collection.sum.rational, collection.size (integer), collection.convert (Sequence) | admitted | admitted |
| numeric.convert equal / mismatch | admitted / admitted | admitted / refused (intended) |
| rational.narrow, reference.upcast | admitted | admitted |

Existing tests at HEAD already admit to_float32, to_float64, to_rational,
quantity.convert, numeric.narrow, collection.sum.quantity and collection.size
(model_population). The reviewer built no admitted fixture for five
identities: ieee.from_exact, numeric.convert_rounding, collection.sum.decimal,
collection.sum.float32 and collection.sum.float64. They need IEEE profile laws
or a decimal scale type, and none exists anywhere in the repository.

## Verdict

**FAIL (two high findings).** The intended behaviour is correct for the 19
familied entries. `result_type` must equal `member.declaration`. The family
check uses `family_fits` against the catalogued `member_family`, and is skipped
when the declaration node is `model_population`, `compound_unit` or `alias`.
The `model_population` skip is real and needed: dropping it is killed by
`collection_size_admits_equal_model_population_result`. Every refusal is
`ill_typed`/`operator-ineligible` at `/body/result_type`, with the application
node as locus. The member shape is strict: missing, malformed, extra and
misspelled (`type_argument` key) members refuse
`invalid_package`/`operation-member-mismatch` at `operation.member`.
`member_declaration` has no fallback to a `type_argument` key. All 11 old
spellings at origin/main are migrated, and the only remaining one at HEAD is
the adverse case at tests/it/checked_package_v2_ieee_admission.rs:563. The
result check sits after `check_operands` and before `check_inner_result`,
`check_state_clause_result`, model-member, mode-type and leaf checks, which is
the order the spec gives. It adds no meter charge, so `consumed_work` is
unchanged. The five other member-result kinds (`field`, `position`, `element`,
`relationship_end`, `operation`) are not selected.

The failure is in selection. The `entry.member == type_argument` arm, together
with the fail-closed guard, also catches entries that were never member-result
entries (lookup, all_instances) and member-result entries that have no
`member_family` (fold, reduce). All four are refused at HEAD, whatever the
package contains (FND-001, FND-002).

Spec tension for the leader: AC-251 asks for a refusal when an entry "changes
away from" `result: member` or "loses a required `member_family`". A rule that
reads only the entry's shape cannot tell an entry that changed from one that
never had that shape, unless it keeps a snapshot of which entries are
currently catalogued. The fix needs a ruling on where that snapshot lives.

Rust lane: no `unwrap`/`expect`/indexing on wire data in the new production
code. Typed refusal codes only. No new `#[allow]`. The removed IEEE allow-list
leaves no dead code. Module docs are updated. Two low findings cover
representation (FND-003, FND-004).

Gates, CARGO_TARGET_DIR in the throwaway worktree, with
QUIRE_SPEC_LANGUAGE_DIR set to a QSL main checkout:

- `cargo test --locked --workspace --test it checked_package_v2`: 443 passed.
- `cargo test --locked -p quire-contract-model --lib checked_package::v2`: 171 passed.
- `cargo fmt --all --check`: exit 0.
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: exit 0.
- `quire validate`: 884/884 docs.
- `quire coverage --strict`: 50 unbacked rows, 0 contradicted.

## Examined scope

```yaml
- id: FR-038-AC-247
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Through `CheckedPackageV2::read`, a catalogued `result: member`/`member: type_argument` application whose member has a valid `declaration` admits only when `result_type` equals that declaration, regardless of whether its family resolves. A mismatch refuses `ill_typed`/`operator-ineligible` at `/semantic_graph/nodes/{i}/body/result_type`, with the application node as locus.
- id: FR-038-AC-248
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: For a catalogued `type_argument` member result with `member_family`, a declared type whose family the reader resolves under the FR-322 ruling must fit the catalogue family; a resolved mismatch refuses `ill_typed`/`operator-ineligible` at `/semantic_graph/nodes/{i}/body/result_type`, with the application node as locus.
- id: FR-038-AC-249
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `type_argument` member missing `declaration`, carrying a malformed declaration, or carrying `type_argument` instead of `declaration` refuses `invalid_package`/`operation-member-mismatch` at `/semantic_graph/nodes/{i}/body/operation/member`, with the application node as locus.
- id: FR-038-AC-250
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The `field`, `position`, `element`, `relationship_end` and `operation` member-result kinds do not acquire the type-argument equality rule; in particular, valid `tuple.project` and `collection.index` packages with member declarations distinct from their `result_type` remain admitted.
- id: FR-038-AC-251
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: If a currently catalogued `type_argument` member-result entry changes away from `result: member`/`member: type_argument`, or loses a required `member_family`, the reader refuses the typed package.
- id: operations.rs/check_type_argument_member_result
  path: crates/quire-contract-model/src/checked_package/v2/operations.rs
  role: examined
  excerpt: fn check_type_argument_member_result(application, operation, entry, graph, catalog) -> Option<ValidationFailure>
- id: operations.rs/operation_defect
  path: crates/quire-contract-model/src/checked_package/v2/operations.rs
  role: examined
  excerpt: 'OperationMemberKind::TypeArgument => { member.as_object().is_some_and(|fields| fields.len() == 2) && member_declaration(&operation).is_some() }'
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: context_only
  excerpt: The catalogue's `result: member` entries with `member: type_argument` are `quire.op.quantity.convert`, ... `quire.op.collection.fold`, `quire.op.collection.reduce`, ... `quire.op.collection.size` and `quire.op.collection.convert`.
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Selection arm `entry.member == type_argument` also selects `quire.op.model.lookup` (`result: absence`) and `quire.op.model.all_instances` (`result: set`), and the shape guard then refuses because `result != "member"`. Every such application is now refused `ill_typed`/`operator-ineligible` at `result_type`, whatever its content. Reproduced through `CheckedPackageV2::read`: admitted at main, refused. QSL lowering emits both identities with a `type_argument` member. Neither entry is a member-result entry, so AC-247 to AC-251 do not reach it. | crates/quire-contract-model/src/checked_package/v2/operations.rs:528-549 |
| FND-002 | high | `quire.op.collection.fold` and `quire.op.collection.reduce` are catalogued `result: member`/`member: type_argument` but carry no `member_family`, so the missing-family guard refuses every application, including one whose `result_type` equals the declaration. AC-247 requires such an application to admit, and AC-248 applies the family check only "with `member_family`". Reproduced through the public reader: admitted at main, refused at HEAD. QSL lowering emits both identities. | crates/quire-contract-model/src/checked_package/v2/operations.rs:550-557 |
| FND-003 | low | The deferred-family skip compares `node.semantic_form` strings although the typed `graph.kinds` (`CheckedNodeKind`) is available. Its `compound_unit` and `alias` arms are redundant: `operand_family` already returns `None` for both, and each form belongs to a single tag. The mutant that deletes those arms is equivalent and survives every test. The arms are not wrong, but nothing pins them. The next form deferred by FR-322 needs another string here, and nothing ties this list to the deferral list. | crates/quire-contract-model/src/checked_package/v2/operations.rs:572-580 |
| FND-004 | low | The closed `type_argument` member shape (`kind` plus `declaration`, exactly two keys) is inlined in `operation_defect` behind a `_ =>` arm over the closed `OperationMemberKind`. Meanwhile `member_is_well_formed` (temporal.rs), the function whose doc says it owns each kind's closed shape, still returns `true` for `TypeArgument`. The same rule therefore has two homes, and the wildcard will silently route a new member kind to a function that accepts any shape. | crates/quire-contract-model/src/checked_package/v2/operations.rs:756-765; crates/quire-contract-model/src/checked_package/v2/temporal.rs:410-427 |

## Dispositions

Round 1. Verified by reviewing
`git diff origin/main...HEAD` () in a throwaway
worktree.

Public-reader probe table, rerun with the same scratch packages. At main,
all 15 cases admit (as in the review pass). At HEAD, fold, reduce,
all_instances, lookup, count, sum.integer, sum.rational, size, convert,
numeric.convert, rational.narrow and upcast admit. Only the intended
mismatches refuse at `result_type`: count with a boolean result,
sum.integer with a mismatch, numeric.convert with a mismatch.

Mutants rerun against the new tests:
- Planner mutant P1 (select by member kind alone) is killed by
  `non_member_model_results_remain_admitted` and the catalogue
  differential.
- Planner mutant P2 (allow-list of the three IEEE identities) is killed.
- Mutant M10 (fold/reduce refused without a family) is killed.
- Mutants M12 (extra keys accepted) and M13 (shape check removed in
  `member_is_well_formed`) are killed.

Gates:
- `checked_package_v2` integration tests: 446 passed.
- `quire-contract-model` v2 library tests: 170 passed.
- fmt and clippy: exit 0.
- `quire validate`: 884/884.
- `quire coverage --strict`: 50 unbacked rows, 0 contradicted.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Selection is now exactly `result == member` and `member == type_argument`. `model.lookup` and `model.all_instances` return `None` before any check and are admitted again (probe and `non_member_model_results_remain_admitted`). |
| FND-002 | fixed | `member_family` is now optional. Fold and reduce get the equality check only: they admit an equal result and refuse an unequal one at `result_type`. |
| FND-003 | fixed | The skip now compares the typed `CheckedNodeKind::BoundedDomain(ModelPopulation)` from `graph.kinds`. The redundant `compound_unit` and `alias` string arms are removed, and the old equivalent mutant no longer exists. |
| FND-004 | fixed | The type-argument shape is now owned by `member_is_well_formed` (temporal.rs:410-435), through a `deny_unknown_fields` `TypeArgumentMember { kind, declaration: CheckedNodeId }`. The inline rule and the `_ =>` wildcard in `operation_defect` are gone. |

Round 2. Re-verified
(`git diff origin/main...HEAD`:). The diff touches only tests
and spec, so the round-1 fixes in operations.rs and temporal.rs are
unchanged. No finding in this file needed a new outcome.

All 18 mutants are killed. These include the round-1 set and three new
ones: result check before the member-kind check, a broken `family_fits`
for `ieee_float`/`decimal`, and dropped `collection.sum.*` equality.

Gates:
- `checked_package_v2` integration tests: 447 passed.
- `quire-contract-model` v2 library tests: 170 passed.
- fmt and clippy: exit 0.
- `quire validate`: 884/884.
- `quire coverage --strict`: 50 unbacked rows, 0 contradicted.
