---
id: SR-605
title: "code review of PR 223 (drop schema migration code)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; diff origin/main...HEAD: crates/quire-contract-model/src/{canonical,conformance,identity}.rs, schemas/{conformance-trace-map-v1.json,contract-conformance-fixture-v1.schema.json,contract-package-reference-v1.schema.json}, scripts/generate_conformance_corpus.py, corpus/contract-v0.1/{inputs,expectations,canonical}, tests/it/{canonicalization,conformance,cycle_free_model,identity,integration}.rs"
review_set: base
---
# SR-605: code review of PR 223

## Summary

Ticket: IR-362. Code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD` (PR 223 is stacked on PR 218). The diff removes `MigrationReceipt`, `migrate_reference_body`, `SchemaVersion::V1_0`, `DiagnosticCode::UnregisteredMigration`, `ConformanceOperation::Migration` and every schema, trace-map, generator and corpus entry for them. It narrows wire preflight and `ensure_supported` to schema 1.1 only.

The reviewer ran every gate at the PR head with `CARGO_TARGET_DIR=<worktree>/target`. `make lint` exited 0, `cargo test --locked --workspace --all-targets -- --include-ignored` exited 0 (178 + 48 tests passed, 0 failed), `make corpus` exited 0, `make audit-unsafe` exited 0 and `make fmt-check` exited 0. `make spec` exited 2: validate passed (208/208 docs) and `coverage --strict` reported 22 unbacked rows. Those are the same 22 rows SR-602 listed on origin/main, so none is new.

`scripts/generate_conformance_corpus.py --output <scratch>` regenerated a corpus that is byte-identical (`diff -r`) to `corpus/contract-v0.1`. The golden bytes and digest in `tc_017_canonical_bytes_digests_ordering_and_resource_failure_conform` pass against the real canonicalizer.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The canonical-API unsupported-version check asserts only the diagnostic code. It never checks the `schema_version` path that STD-001 and FR-016 require, although this diff changed that path (it was `schema_version.major` for a non-1 major). It probes only `canonical_package` at 1.9, so it misses 1.0 (the retired version), a non-1 major, and the requirement and clause APIs. A path regression, or dropping `ensure_supported` from `canonical_requirement` or `canonical_clause`, would pass every test. Fix: loop over 1.0, 1.9 and 2.0, call `canonical_package`, `canonical_requirement` and `canonical_clause` for each, and assert both `code == UnsupportedSchemaVersion` and `path == "schema_version"`. | tests/it/canonicalization.rs:171-184 |

## Scope

- `crates/quire-contract-model/src/identity.rs:1290-1305` (VersionPreflight::validate), examined. It accepts `(1, 1)`, returns `unsupported_schema_version` at `schema_version.minor` for any other major-1 version and at `schema_version.major` otherwise. This matches FR-017 and STD-001.
- `crates/quire-contract-model/src/canonical.rs:330-340` (ensure_supported), examined. It accepts only `V1_1` and otherwise returns `unsupported_schema_version` at `schema_version`, matching FR-016 and STD-001 for the canonical API. The package, requirement and clause APIs all call it (lines 186, 211, 249).
- `crates/quire-contract-model/src/conformance.rs` (operation, boundary, construct, mismatch-kind and executor removals), examined. The boundary match orders `(1,1)` before `(0,_)`, `(1,_)` and `(_,_)`, and `schema.unsupported_minor` maps to `unsupported_schema_version`.
- `crates/quire-contract-model/src/conformance.rs:1664` (`observe_constructs` without its `operation` parameter), examined. The parameter was read only by the deleted migration-receipt branch, `observe_constructs` has one caller (line 1100) that `observed_coverage` still reaches only on success, and the remaining body walks `input`/`actual` the same way for every operation. The removal is safe.
- `schemas/*.json` and `scripts/generate_conformance_corpus.py`, examined. The schemas no longer mention migration, and the generator's `copy` import is still used elsewhere.
- `tests/it/canonicalization.rs:631-676` (tc_017_version_preflight_supports_only_schema_1_1), examined. See SR-606.
- `tests/it/conformance.rs` (six mismatch kinds), examined. It matches the six `kinds.push` sites in `mismatch_kinds`.
- Repository grep at the PR head for `migrat|V1_0|1_0|unregistered|receipt` (reviews excluded), examined. Remaining hits are only the LICENSE text, the FR-016/FR-017 spec prose that says "no migration", ADR-0053 prose, and the historical `status: done` plan records TASK-008/TASK-009. No code, schema, corpus or test hit remains.

## Verdict

Approve after FND-001, or accept it as a follow-up. The removal is complete and correct. No migration or 1.0 path remains in code, schemas, corpus or tests. Both diagnostic paths match FR-016, FR-017 and STD-001. The corpus is regenerated script output. Every gate except the known `make spec` coverage baseline exits 0. No new unsafe, panic surface, lock or integer-conversion change appears in the diff. The only defect is a test gap on the canonical-API diagnostic path, which the diff changed without pinning.
