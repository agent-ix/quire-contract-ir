---
id: SR-13250
title: "IR-893 PR 481 code review (with Rust lane): ContractPackage reader refusal tests"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir vs merge-; tests/it/complete_v1_contract_package.rs, spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md, spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md, spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

Ticket: IR-893. PR agent-ix/quire-contract-ir#481. The diff is test-only plus status prose: three new tests for FR-035-AC-12, AC-13 and AC-14, and two FR-035-AC-2 tests split out of AC-5-tagged tests. The Rust lane is folded into this file. Tags use `#[trace("TC-047", "FR-035-AC-n")]`. No `/// Trace:` lines were added. The diff adds no hash, pin, vendored copy or compatibility layer, and it does not touch `Cargo.toml`, `Cargo.lock` or production code.

The reviewer measured the following at the head, in a fresh worktree with its own target dir:

- The focused TC-047 module passes (22 of 22).
- `make fmt-check` passes.
- `make lint` passes (both clippy lanes).

Reviewer production mutants in `crates/quire-contract-model/src/checked_package/v2/contract_package.rs`:

- **AC-12:** `MissingMember` changed to `UnknownMember` on the `contract_version` branch (line 955) fails the new AC-12 test.
- **AC-13:** stripping leading ASCII whitespace at the start of `admit` fails the new AC-13 test.
- **AC-2 (dependency):** reporting the stale-dependency refusal at the root pointer instead of `/package/source/lock/dependency_selections/0` (line 2009) fails the new test.
- **AC-2 (own identity):** reporting the digest refusal at `/identity` instead of `/identity/digest` (line 1973) fails the new test.
- **AC-14:** deleting the ordered request check `record.node_id != &mapped.requested_item` (line 1917) **passes** every TC-047 test (FND-001). Only when the lowered-but-not-represented check (line 1924) is deleted as well does the AC-14 test fail.

The removed cases are all subsumed. The deleted `wrong_request` case put `missing` into the first record, which is `requested_items[3]`, now index 3 of the AC-14 loop. The deleted leading-space check moved into AC-13. The stale own-identity and stale-dependency assertions moved into the AC-2 tests. The AC-5 tests keep their unrelated checks: unknown member, wrong identity domain, missing selection and wrong identity.

## Examined criteria

- FR-035-AC-12 (examined): "Given an otherwise valid canonical `ContractPackageWireV1` that omits any of its six required top-level members (`contract_version`, `package`, `semantic_graph`, `source_map`, `feature_accounting`, or `identity`), `ContractPackageWireV1::read` returns `Refused` with code `invalid_package`, root path `\"\"`, and cause `missing_member`; it returns no admitted package."
- FR-035-AC-13 (examined): "Given otherwise valid `ContractPackageWireV1` bytes with leading whitespace before the canonical JSON value, `ContractPackageWireV1::read` returns `Refused` with code `noncanonical_wire`, absent path, and absent cause; it returns no admitted package."
- FR-035-AC-14 (examined): "Given an otherwise valid canonical `ContractPackageWireV1` except that its first `feature_accounting` record names a requested node other than the first `source_map.requested_items` entry, `ContractPackageWireV1::read` returns `Refused` with code `invalid_package`, path `/feature_accounting`, and cause `invalid_value`; it returns no admitted package."
- FR-035-AC-2 (examined): "A source, type, anchor, identity, bound, dependency, or contract-version mutation that changes or invalidates meaning refuses before a backend artifact; a stale selected dependency does not resolve through an identically keyed local node."
- FR-035-AC-5 (context_only): the retained AC-5 tests were read to confirm the split left their unrelated checks intact.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The AC-14 test cannot fail when the ordered request check is removed. The first record is `lowered`, and every substitute (`requested_items[1..=3]`: a `requires_bound` node, an `unsupported` node and `missing`) is absent from `semantic_graph.nodes`. So the lowered-but-not-represented check (contract_package.rs:1924) refuses with the same `/feature_accounting` and `invalid_value`. A reviewer mutant that deletes the order check at contract_package.rs:1917 passes all 22 TC-047 tests. Without that check, a package whose first lowered record names another represented requested node would be admitted. Fix: add a case whose substitute is a represented requested node other than `requested_items[0]`, for example a request with two distinct lowered roots, with the first record's disposition left consistent. Then confirm that deleting line 1917 fails the test | tests/it/complete_v1_contract_package.rs:209-233 |
| FND-002 | low | `reader_refuses_stale_selected_dependency_at_selection` repeats about 15 lines of setup from `selected_dependencies_are_bound_by_library_identity_before_graph_checks`: admitting the dependency, lowering, rewriting `dependency_selections` and building `supplied`. Fix: extract a helper that returns the admitted dependency and the `lib/a`-selecting wire, and call it from both tests | tests/it/complete_v1_contract_package.rs:1063-1080 |

## Verdict

**PASS with findings.** The AC-12, AC-13 and both AC-2 tests assert code, path and cause exactly as the spec says, and production mutants fail them. FND-001 means AC-14's ordered-correspondence rule is not discriminated by any test. Fix it in this PR. FND-002 is a style nit.

## Dispositions

Disposition pass 1, against (origin/main merged in).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The AC-14 test adds a case that lowers two represented roots (`cccc`, `dddd`). It asserts that both records are `lowered` and that the second root is in `semantic_graph.nodes`, points the first record at it, and recomputes the identity. It then asserts `InvalidPackage`, `/feature_accounting` and `InvalidValue`. The reviewer mutant that deletes the order check at contract_package.rs:1917 now fails this test (panic at line 248: the package is admitted) |
| FND-002 | fixed | A new helper, `wire_selecting_lib_a`, returns the admitted dependency and the `lib/a`-selecting wire. Both dependency tests call it |
