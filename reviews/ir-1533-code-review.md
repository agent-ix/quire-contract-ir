---
id: SR-2026111317273300000001
title: "code-review and rust-review of IR-1533 reaches_population over reaches_field"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #1020 at d082fe9d (git diff origin/main...HEAD); crates/quire-contract-model/src/checked_package/v2/{operations.rs,reaches_population.rs,relationship_end.rs}, read against structural.rs; tests/it/checked_package_v2_model_members/reaches_population.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for quire-contract-ir PR #1020. Ticket: IR-1533. Static read only: the branch was not compiled or tested, because every build lane was busy and the coder has a focused `cargo test -p quire-contract-ir --test it tc_048_` run queued. Every finding below comes from reading the code. None was confirmed by compiling.

## Examined scope

- relationship_end.rs `reaches_or_field_application` (examined). It accepts exactly the `reaches` and `reaches_field` identities. `reaches_application` is unchanged and still used by `relationship_end`, so the relationship-end accessor still refuses a `reaches_field` node.
- reaches_population.rs accessor (examined). It now uses the new lookup. Its population read goes through generic `member_ids`, which `decode_member_ids` fills for every member kind, so it works on a `field` member.
- operations.rs `check_relationship_member` population tail (examined). Moving the role-end lookup ahead of the `linked` lookup does not change the relationship path's outcome. Every branch refuses `ill_typed`/`operator-ineligible` at `member/population`, nothing in between charges work, and the navigate rule and the earlier checks are untouched. `role_end_id` is `&CheckedNodeId` (declaration_id returns `Option<&CheckedNodeId>`), which is correct.
- operations.rs `check_reference_edge` tail (examined). The population check runs only on the conforming branch, after every other edge check. It compares against `edge_id`, the declaration of the field owner `T`, which matches the FR.
- operations.rs `check_linked_population` (examined): see FND-001.
- structural.rs `collect_application_dependencies` / `application_dependencies_at` (context_only): see FND-002.
- The three new tests and their helpers (examined): `reaches`, `Scenario`, `link_application`, `add_population`, `reference_of`, `application_of`, `relationship_package`, `admitted`, `order_declaration_id`. All are reachable through `use super::*`.

## Verdict

Not mergeable. FND-001 is a compile error. FND-002 makes every new linked-population test fail at the structural stage, even after FND-001 is fixed. Apart from those, the accessor change and the factoring of the relationship path into `check_linked_population` are correct, and the relationship path keeps its outcomes. The `&edge_id` the brief asked about compiles through deref coercion but trips clippy (FND-003).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `check_linked_population` keeps the old `let Some(population) = linked else { return ineligible(member_at("population")); };` ahead of the new `ill_typed` closure. `ineligible` is not in scope in the new function (E0425), and its `Result` shape would not match the `Option<ValidationFailure>` return either. The crate does not compile. Fix: delete those three lines and keep the `ill_typed` version. | crates/quire-contract-model/src/checked_package/v2/operations.rs:3978-3980 |
| FND-002 | high | The structural dependency join adds a member `population` only for `ModelReaches`/`ModelNavigate`, and `dependency_order.matches` is an exact comparison. `link_field_population` (through `link_application`) lists the population, and in AC-544 the reference node, in the `reaches_field` application's `dependencies`. So each linked scenario is refused `invalid_semantic_graph` at `/semantic_graph/nodes/{n}/dependencies` before the operation stage. AC-542 panics in `admitted()`, and both AC-544 cases get `BAD_GRAPH` instead of `ILL_TYPED` at `member/population`. Fix: add `quire.op.model.reaches_field` to both join sites, keyed on the identity string because `KnownOperation` has no reaches_field variant, and amend FR-038 "Dependencies" (see the spec-review SR). | crates/quire-contract-model/src/checked_package/v2/structural.rs:2150-2165, 2239-2254 |
| FND-003 | low | `edge_id` is already `&CheckedNodeId`, from `ModelOwners::declaration_id`, so `&edge_id` passes a `&&CheckedNodeId`. It compiles through deref coercion, but `clippy::needless_borrow` fails `make lint` (`-D warnings`). Pass `edge_id`. | crates/quire-contract-model/src/checked_package/v2/operations.rs:3627 |
| FND-004 | low | rustfmt drift. Both `Ok(check_linked_population(...))` calls, whose argument list is under `fn_call_width`, and the `link_field_population` signature (94 columns) fit on one line, so `make fmt-check` fails. Run `make fmt`. | crates/quire-contract-model/src/checked_package/v2/operations.rs:3623-3628, 3937-3942; tests/it/checked_package_v2_model_members/reaches_population.rs:1152-1156 |
| FND-005 | low | `reaches_or_field_application` duplicates `reaches_application` except for the accepted identities. Use one private lookup that takes the accepted identity set. | crates/quire-contract-model/src/checked_package/v2/relationship_end.rs:63-92 |
