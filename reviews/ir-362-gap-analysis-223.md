---
id: SR-606
title: "gap analysis of PR 223 (drop schema migration code)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@bbac25d64706d64ee0103cef8e9407d3b04e15a4; diff f21a194...bbac25d against spec on origin/main c658b2b: spec/contract/FR-016, FR-017, FR-018, STD-001, spec/interface/FR-019, FR-020, reviews/ir-362-gap-analysis.md (SR-602 change list)"
review_set: base
---
# SR-606: gap analysis of PR 223

## Summary

Ticket: IR-362. Planless gap analysis. Plan completion: not assessed. This review checks the diff `f21a194...bbac25d` against the merged spec and the SR-602 follow-up inventory. Owner decision: schema 1.1 is the only supported version, with no legacy 1.0 reader and no compatibility layer.

SR-602 inventory, item by item:

- identity.rs `UnregisteredMigration` is removed.
- identity.rs `SchemaVersion::V1_0` is removed.
- identity.rs VersionPreflight accepts only `(1,1)` and reports the STD-001 paths.
- canonical.rs `ensure_supported` accepts only 1.1 and reports `unsupported_schema_version` at `schema_version`.
- canonical.rs `MigrationReceipt` and `migrate_reference_body` are deleted.
- conformance.rs:
  - The import is dropped.
  - The construct tag is removed.
  - The boundaries are replaced and stay sorted.
  - The `Migration` operation and its arms are removed.
  - The `Coverage` arm is in place.
  - The boundary observation and mapping are updated.
  - The construct observation, the executor, `MigrationInput`, `migration_invalid_actual` and the `migration_receipt` mismatch kind are deleted.
- schemas:
  - The package `minor` is `const 1`.
  - The `unregistered_migration` enum entry is removed.
  - `migrationInput`, `migrationReceipt` and `migrationExpectation` are deleted.
  - The fixture `schemaVersion` stays open, as SR-602 intended.
  - The four trace-map edits are made.
- generator: the three package cases (1.0 and 1.2 as `unsupported_minor`, 2.0 as `unknown_major`) are in place and the migration placeholder is deleted.
- corpus: the migration fixtures are deleted and the package-schema fixtures added. A reviewer rerun of the generator into scratch is byte-identical to the checked-in corpus.
- tests:
  - The import is dropped.
  - The fixture uses V1_1.
  - The golden bytes and digest are regenerated from code output, and the test passes.
  - The 1.9 canonical case expects `UnsupportedSchemaVersion`.
  - The FR-017-AC-1 test is rewritten.
  - The receipt mutation and the `migration_receipt` kind are removed.
  - identity.rs, integration.rs and cycle_free_model.rs move to V1_1.
- tests/fixtures/{bridge-qsl-consumer,model-alias-consumer}: not applicable. Those fixtures do not exist at base f21a194 because PR 218 removed them.
- spec/contract-test-matrix.md: the FR-016, FR-017, FR-018 and FR-020 "implemented" rows are now true against the code.
- plan TASK-008/TASK-009: historical `status: done` records; SR-602 marked them optional; no change needed.

Grep on bbac25d for `migrat|V1_0|1_0|unregistered|receipt` finds no code, schema, corpus or test hit.

Gate exit codes, from the reviewer's own run:

| Gate | Exit | Notes |
| --- | --- | --- |
| lint | 0 | |
| test --include-ignored | 0 | |
| corpus | 0 | |
| audit-unsafe | 0 | |
| fmt-check | 0 | |
| spec | 2 | validate passed 208/208; coverage `--strict` found 22 unbacked rows |

The 22 unbacked rows are identical to the SR-602 baseline: FR-036/037/039/344, FR-036-AC-1..5, FR-037-AC-6, FR-039-AC-1..4, FR-344-AC-1..3, FR-019-AC-5 and TC-045/055/058/222. None is new.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-017-AC-1 requires the zero-major rejection to happen "before semantic interpretation". The zero-major loop in tc_017_version_preflight_supports_only_schema_1_1 asserts only `diagnostics[0].code`. It does not assert `diagnostics.len() == 1`, although the invalid `id` is still present, and it does not assert the path. A regression that ran semantic validation after a zero-major rejection, or moved the path, would pass. Fix: assert `len() == 1` and `path == "schema_version.major"` in that loop, as the unsupported-version loop above it already does. | tests/it/canonicalization.rs:662-667 |

## Scope

- `FR-017-AC-1` (spec/contract/FR-017-version-orphan-coverage.md), examined: Version preflight accepts schema 1.1; schemas 1.0, 1.2 and 2.0 each fail `unsupported_schema_version` at the specified path, and a zero major fails `invalid_schema_version`, all before semantic interpretation.
- `FR-017` behavior (spec/contract/FR-017-version-orphan-coverage.md), context_only: Any well-formed version other than 1.1 fails `unsupported_schema_version`: at `schema_version.major` when the major is not 1, otherwise at `schema_version.minor`.
- `FR-016` in-memory version rule (spec/contract/FR-016-canonicalization-digests.md), examined: A package constructed in memory with any other version fails `unsupported_schema_version` at `schema_version` from every canonical API and produces no bytes or digest. SR-605 FND-001 records the test gap here.
- `STD-001` `unsupported_schema_version` row (spec/contract/STD-001-diagnostic-registry.md), examined: preflight: `schema_version.major` when the major is not 1, otherwise `schema_version.minor`; canonical API: `schema_version`; no semantic span.
- `FR-018-AC-2` (spec/contract/FR-018-conformance-corpus.md), examined: Mutation fixtures independently alter schema validity, diagnostic code/path/order/span/obligation, canonical byte, digest, dependency, and coverage row/reason; each produces the exact mismatch result without message parsing.
- `FR-020-AC-2` (spec/interface/FR-020-json-conformance-interface.md), examined: Process fixtures pin exit 1 with all six mismatch kinds in fixed order and exit 2 for each of the six closed operational codes ...
- `FR-019-AC-2` (spec/interface/FR-019-rust-library-interface.md), examined: The complete negative corpus executes package/expression decode, validation, canonicalization, and coverage through `catch_unwind`.

## Verdict

Every SR-602 item that applies is done, and no migration or 1.0 path remains. The preflight and canonical-API paths match FR-017, FR-016 and STD-001.

The rewritten FR-017-AC-1 test is sound. It covers 1.1 acceptance, 1.0, 1.2, 1.9, 2.0 and 2.1 with code, path and a single diagnostic while an invalid id is present, zero major, and a missing version. Its tags `Tracing: TC-017` and `FR-017-AC-1` are correct. FND-001 is the one weaker branch.

Its `NFR-003-AC-1` tag was on the test before this diff and has not changed. That AC names TC-019 as its verification, so the tag is out of scope here.
