---
id: SR-2026111219210500000001
title: "code-review and rust-review of IR-1528 operation_bodies sidecar"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #1018 at 306ded5e (base origin/main 270410bb); crates/quire-contract-model/src/checked_package/v2/{operation_bodies.rs,operation_effects.rs,mod.rs,intake.rs,owner.rs,document_de.rs,native_join_budget.rs}, crates/quire-contract-model/src/lib.rs, tests/conformance_qspec/operation_bodies.rs and the CheckedPackageSidecarsRef literal updates"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for quire-contract-ir PR #1018. Ticket: IR-1528. Reference (read only, nothing copied): QSpec origin/main f268d62f (agent-ix/quire-specification PR #651), FR-322 "Operation-body carrier", FR-322-AC-397, TC-987, `proposals/checked-package-v2/{schema.json,operation-body-vectors.json}` and `checked-term-data-vectors.json` (DT-01).

Commands run, all with `QUIRE_SPECIFICATION_DIR` pointing at a detached QSpec worktree at f268d62f:
`cargo test --test conformance_qspec -- operation_` (22 passed, including TC-987 positive, TC-987 refusal and TC-977), `cargo test --features conformance-cli --test it -- operation_effect canonical_encoding` (28 passed), `cargo clippy --locked --workspace --all-targets --features conformance-cli -D warnings` (clean), `cargo clippy --locked -p quire-contract-model -D warnings` (clean), `cargo fmt --check` (only the known unrelated drift at monitor_plan/layout_context.rs:590).

Probes were run in a throwaway detached worktree (not the PR branch), adding a second dispatch candidate to OBC-05:
- A valid redefinition (Premium redefines Order.settle, own body clause function and own row) with two candidates for selected operation (Order, settle): admitted without the carrier, refused `invalid_package`/`invalid-value` at `/operation_bodies/2/body` with a correct carrier. With only one candidate it is admitted.
- A candidate whose `operation_body` is a non-function node: refused at `/operation_bodies/2/body` with the carrier, at `/checked_term_data/dispatch_tables/0/candidates/0/operation_body` without it.
- OBR-20's own `dispatch_candidates` without the carrier: refused at `/checked_term_data/dispatch_tables/0/candidates/0/operation_body`, because the candidate's body node is absent from the graph.

## Examined scope

- operation_bodies.rs `OperationBody` strict Deserialize (examined): `no_body` with `node` refuses (unknown_field), `function` without `node` refuses; OBS-10 and OBS-11 return `malformed_wire` at `/operation_bodies/{i}/body`.
- operation_bodies.rs `validate` (examined): join order, has_body derivation, body-node checks, candidate agreement, repeated key, key order, coverage, metering.
- operation_bodies.rs accessors `operation_body` and `operation_body_rows` (examined): binary search over admitted, sorted rows.
- operation_effects.rs refactor into `join_member`, `sorted_keys`, `first_repeated`, `has_uncovered_operation`, `key_of` (examined): the check order, codes, causes and loci are unchanged; TC-977 and 28 effect tests pass.
- intake.rs and document_de.rs member registration and the `operation_effect_schema_parent_length` loci for `operation_bodies` (examined).
- mod.rs envelope, encoder and `CheckedPackageSidecarsRef::operation_bodies` field (examined); the about 33 struct literal updates (examined): every in-repo user compiles under `--workspace --all-targets`.
- tests/conformance_qspec/operation_bodies.rs (examined): reads the vectors by path from the QSpec checkout; no vector, schema or fixture is copied into the repo.

## Verdict

Not mergeable as-is: FND-001 is a wrong refusal of a valid package. The carrier, the strict body deserializer, the encoder input, the accessors and the operation_effects refactor are otherwise correct and match FR-322's join order. The code implements FR-322's candidate rule literally, so FND-001 also needs a QSpec correction (see the spec-review SR). The OBR-21 deviation is not a code defect: the owner join stage refuses the node first (see the gap-analysis SR).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The candidate check requires every dispatch candidate whose `selected_operation` is the row's `(declaring, operation)` to carry the row's body node. A dispatch table holds one candidate per selection key, all sharing the family's `selected_operation` but with distinct `operation_body` nodes (QSpec DT-01-two-candidates: FastService.run and SafeService.run under Service.run). So any package with polymorphic dispatch and an `operation_bodies` member is refused `invalid_package`/`invalid-value` at the family row's `body`. Reproduced: a valid redefinition package that is admitted without the carrier is refused at `/operation_bodies/2/body` with a correct carrier. QSL-1899 will emit this member. | crates/quire-contract-model/src/checked_package/v2/operation_bodies.rs:334-342 |
| FND-002 | medium | The carrier validates before `checked_term_data` and reads candidates from the unvalidated wire. A package whose only defect is a dispatch candidate's `operation_body` is refused at `/operation_bodies/{i}/body`, which blames the correct row, instead of at `/checked_term_data/dispatch_tables/0/candidates/0/operation_body` (reproduced). No invalid package is admitted, because dispatch validation still runs afterwards. QSpec's OBR-20 only meets its recorded locus because of this order: its candidate names a body node absent from the graph, so it holds two defects. | crates/quire-contract-model/src/checked_package/v2/mod.rs:2065-2077 |
| FND-003 | medium | Adding a public field to the non-`Default`, non-`#[non_exhaustive]` struct `CheckedPackageSidecarsRef` follows the repo's norm, and every in-repo user compiles. It still breaks quire-spec-language, which tracks this crate on `branch = "main"` and builds the struct literally at `qsl-package/src/emit.rs:5625`, `emit.rs:6688`, `emit/monitor_plan.rs:1305` and `emit/tests/operation_effects.rs:656`. QSL's adaptation (QSL-1899) must land with this merge, or QSL main goes red on its next lock update. | crates/quire-contract-model/src/checked_package/v2/mod.rs:932-933 |
| FND-004 | low | `is_body_clause` is a private one-line wrapper that only forwards to `super::owner::is_body_clause`. Call the owner function directly. | crates/quire-contract-model/src/checked_package/v2/operation_bodies.rs:185-188 |

## New findings (disposition pass 1)

Re-review of the delta at 8e4d0cf4 (commits 3f7cc94a and 8e4d0cf4). Probes ran in a throwaway detached worktree, not the PR branch, with `QUIRE_SPECIFICATION_DIR` at an extract of QSpec origin/main proposals: (A) Premium redefines Order.settle (`redefines`), with its own body clause function and its own row, and two candidates (Order and Premium, each with its own body): admitted. (B) The same package with the Premium candidate carrying Order's body: refused `invalid_package`/`invalid-value` at `/operation_bodies/4/body`. The PR's focused tests (`cargo test --test conformance_qspec -- operation_`) all pass at head.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | medium | The interim join keys a candidate on the declaration its `selection_key.original` names. A candidate whose receiver inherits the operation (no own member, so no row) joins nothing and goes unchecked, even when its `operation_body` differs from the body of the declaration it inherits from. FR-322 as written refuses that package (the candidate carries the row's `selected_operation`), and under the intended meaning it is also inconsistent: the receiver's effective body is the inherited row's body. The PR's own multi-candidate test admits exactly this package (Premium inherits Order.settle, candidate body `Order/settle/other`, Order row body `Order/settle/body`). No other stage catches it: dispatch validation only checks that the body is a `function` node. Join the candidate to the row of its effective declaration instead: resolve `selected_operation.operation` from `selection_key.original.node` (the model's `resolve`/`operation_in_family` path already finds the effective operation). That refuses the inherited mismatch, keeps probe A admitted and probe B refused, and makes the join final, not interim. | crates/quire-contract-model/src/checked_package/v2/operation_bodies.rs:228-237, 340-348 |
| FND-006 | medium | `two_dispatch_candidates_with_their_own_bodies_admit_beside_the_carrier` does not test what its name and doc comment claim. Its second candidate's receiver, Premium, declares no `settle`, so it has no row and no body of its own: its `operation_body` is a second body clause of Order.settle. It admits only because the candidate is joined to nothing. The test would pass with no candidate check at all (the first candidate agrees with its row), so it cannot fail for the intended DT-01 shape, and it pins FND-005's under-check as expected behavior. Replace it with the probe A shape (the second receiver redefines the operation, with its own body node and its own row, and both candidates are joined), plus probe B as its refusal twin. | tests/conformance_qspec/operation_bodies.rs:495-531 |

## New findings (disposition pass 2)

Measured at 9f4b2267, with probes in a throwaway detached worktree and `QUIRE_SPECIFICATION_DIR` at the QSpec proposals extract. Outcomes:
- (A) Premium redefines `settle`, candidate carries its own body: admitted.
- (B) The same, with the candidate carrying Order's body: refused at `/operation_bodies/4/body`.
- (C) Premium inherits, candidate carries Order's body: admitted.
- (C') Premium inherits, candidate carries `Order/settle/other`: refused at `/operation_bodies/2/body`.
- Gold extends Premium, which extends Order:
  - Gold inherits Premium's redefinition: admitted with Premium's body, refused at the Premium row with Order's body.
  - Gold inherits Order through Premium: admitted with Order's body, refused at the Order row with another body.
  - Gold redefines: admitted with its own body, refused at the Gold row with Premium's body.
- A candidate receiver that is not an object type (a field, a missing node, an operation member, an unselected package): refused at `/checked_term_data/dispatch_tables/0/candidates/1/selection_key`.
- A receiver that is another selected package's process (`acme/billing` Ledger): refused, but at `/operation_bodies/0/body` (see FND-007).
All 8 `operation_bodies` tests pass at head.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | low | The carrier joins a candidate whose receiver resolves in any selected model, even one outside the dispatch table's family. A candidate whose `selection_key` names another selected package's process (Ledger in `acme/billing`, under an Order.settle table) is joined to the unrelated Ledger row and refused at `/operation_bodies/0/body`. Dispatch validation would blame the real defect at `.../candidates/1/selection_key`. The decision is right (refused) but the locus is wrong, and the comments in mod.rs and operation_bodies.rs ("candidates whose receiver does not resolve are left to dispatch validation") do not cover this case. Join only when the receiver is in the table's family: same package as the context, and `operation_in_family(original.node, selected_operation.declaring node, name)` is true. Otherwise skip. | crates/quire-contract-model/src/checked_package/v2/operation_bodies.rs:231-249 |
| FND-008 | low | `effective_operation_owner` charges one work unit per object type, but scans every type's operations with `.any(...)` uncharged. `operation_in_family` (just above) charges one unit per operation it scans. Charge per operation, or look the owner up through the declaration index rather than a linear scan. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:1958-1980 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 8e4d0cf4 joins each candidate through `selection_key.original` (package, node) plus the operation name, not through `selected_operation`. Probe A (a valid redefinition with its own row and two candidates) is admitted; probe B (a redefinition candidate carrying the ancestor's body) is refused at `/operation_bodies/4/body`. The inherited-receiver under-check left by this join is FND-005. |
| FND-002 | still-open | No change and no stated rationale: `operation_bodies::validate` still runs before `dispatch_projection::validate_checked_term_data` (mod.rs:2065-2077) and reads unvalidated candidates. OBR-20 still meets its locus only because of this order. Either move the candidate agreement check after dispatch validation, or record `accepted-no-change` with the FR-322 stage-order reason. |
| FND-003 | deferred | Not fixable in this repo. The PR body now lists the six QSL literal sites, and QSL-1899 (In Progress) owns the adaptation. Merge condition: land together with QSL-1899's adaptation. |
| FND-004 | fixed | 3f7cc94a removed the wrapper; both call sites use `super::owner::is_body_clause` directly. |
| FND-002 | accepted-no-change | The reason in mod.rs is valid: QSpec OBR-20's published oracle records the carrier locus for a package that also has a dispatch defect, so the carrier has to run first to meet the vector. No wrong admit or refuse decision results; only the locus of a two-defect package changes. The one wrong-locus case not covered by that reason is new finding FND-007. |
| FND-005 | fixed | 9f4b2267 joins each candidate to the row of its effective declaration (`effective_operation_owner`). The inherited mismatch (C') is refused, the valid redefinition (A) admitted, and three-level chains join the nearest redefining ancestor (D1/D2/D3 and their refusal twins). |
| FND-006 | fixed | The vacuous test was replaced by `a_redefining_receivers_candidate_is_joined_to_its_own_row`, `a_redefining_receivers_candidate_carrying_the_ancestors_body_refuses` and `an_inheriting_receivers_candidate_is_joined_to_the_inherited_row`. Each can fail, and the inheriting test asserts both admission and refusal. |
