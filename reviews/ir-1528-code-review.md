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
