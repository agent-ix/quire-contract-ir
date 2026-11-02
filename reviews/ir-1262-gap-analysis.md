---
id: SR-2026110101293
title: "Gap analysis — IR-1262 compensation clock rows"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #825 FR-038-AC-412..416, FR-019-AC-22, TC-048, TC-058, reader and tests"
review_set: subset
---

## Summary

Ticket: IR-1262. Computed `quire matrix --scope . --format tsv` and checked the six changed criteria against their actual tagged test, reader path and QSpec CB-01 through CB-08 vectors. Plan completion: not assessed. Optional full semantic review was not selected; this PR review checked intent only for the changed criteria.

## Verdict

**CONDITIONAL** — FR-038-AC-412 through AC-416 have a behavioral vector-backed test. FR-019-AC-22 is tagged by that test, but its default-feature external-consumer condition has no matching compile evidence.

## Coverage

The computed matrix marks all six criteria `tagged` to `tests/it/checked_package_v2_protocol_body.rs:6423`. The test reads the published QSpec vector by reference, constructs local complete packages, checks exact admission or refusal, and borrows the admitted temporal rows. It runs as `quire-contract-ir`'s integration target, where the `quire-contract-model` dev dependency enables `fault-injection` (root `Cargo.toml`). No standalone default-feature consumer is compiled for FR-019-AC-22. No vector, schema or fixture is copied into this repository. Plan completion: not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The sole FR-019-AC-22 test runs with `quire-contract-model`'s `fault-injection` feature through the root dev dependency; it does not compile the required default-feature external consumer. | Cargo.toml:98; tests/it/checked_package_v2_protocol_body.rs:6411 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `crates/quire-contract-model/tests/temporal_binding_public.rs` now compiles an external model-crate consumer with only the default feature and borrows an admitted typed clock row. The existing CB vector test checks three-row compensation and absent-row behavior through the same feature-independent reader. |
