---
id: SR-602
title: "gap analysis of PR 221 (schema 1.1 only, no registered migration)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/contract/FR-016..FR-018, spec/contract/STD-001, spec/interface/FR-019, spec/interface/FR-020, spec/nonfunctional/NFR-003, spec/stakeholder/StR-002, spec/contract-test-matrix.md; code read for follow-up inventory: crates/quire-contract-model/src, schemas, corpus/contract-v0.1, scripts/generate_conformance_corpus.py, tests"
review_set: base
---
# SR-602: gap analysis of PR 221

## Summary

Ticket: IR-362. Planless gap analysis. Plan completion: not assessed. The diff is spec-only; the owner deferred the code change to a follow-up PR, so spec/code disagreement on migration items is excluded from findings. Instead this records every code, corpus, schema, script and test location the follow-up must change. `make spec`: validate passes (243/243); coverage reports 22 unbacked rows and 0 contradicted statuses, the same rows as origin/main (FR-036/037/039/344 and their ACs, TC-045/055/058/222, FR-019-AC-5); none is new and none belongs to an AC this PR edits.

Follow-up inventory:

- crates/quire-contract-model/src/identity.rs:82 remove `UnregisteredMigration` from the DiagnosticCode registry (drops it from `DiagnosticCode::ALL` and the inventory).
- crates/quire-contract-model/src/identity.rs:368 remove `SchemaVersion::V1_0`.
- crates/quire-contract-model/src/identity.rs:1293-1305 VersionPreflight accepts `(1, 1)` only; every other version returns `UnsupportedSchemaVersion` at the path STD-001 settles (SR-601 FND-008).
- crates/quire-contract-model/src/canonical.rs:330-344 `ensure_supported` accepts `(1, 1)` only and stops emitting `UnregisteredMigration`; the code and path for the in-memory case await SR-601 FND-005.
- crates/quire-contract-model/src/canonical.rs:735-800 delete `MigrationReceipt` and `migrate_reference_body`.
- crates/quire-contract-model/src/conformance.rs:14 drop the `migrate_reference_body` import.
- crates/quire-contract-model/src/conformance.rs:94 remove the `migration.reference_body_1_0_to_1_1` construct tag from PUBLIC_CONSTRUCT_TAGS.
- crates/quire-contract-model/src/conformance.rs:135,138 replace `schema.1_0` and `schema.unregistered_minor` with `schema.unsupported_minor` in CONFORMANCE_BOUNDARIES (keep sorted).
- crates/quire-contract-model/src/conformance.rs:159,167,175,184,193 remove `ConformanceOperation::Migration` and its `migrationInput`/`migrationExpectation` arms.
- crates/quire-contract-model/src/conformance.rs:801 the `Migration | Coverage` arm becomes `Coverage`.
- crates/quire-contract-model/src/conformance.rs:1204-1219 boundary observation: `(1, 1)` records `schema.1_1`, `(0, _)` records `schema.zero_major`, any other `(1, _)` (including 1.0) records `schema.unsupported_minor`, other majors record `schema.unknown_major`.
- crates/quire-contract-model/src/conformance.rs:1398 map `boundary:schema.unsupported_minor` to `unsupported_schema_version`.
- crates/quire-contract-model/src/conformance.rs:1685-1691 delete the migration construct observation.
- crates/quire-contract-model/src/conformance.rs:1855, 1989-2019, 2115-2122 delete `execute_migration`, `MigrationInput` and `migration_invalid_actual`.
- crates/quire-contract-model/src/conformance.rs:2172-2174 remove the `migration_receipt` mismatch kind.
- schemas/contract-package-reference-v1.schema.json:36 `minor` becomes `{ "const": 1 }`.
- schemas/contract-conformance-fixture-v1.schema.json:139 remove `unregistered_migration` from the diagnostic code enum.
- schemas/contract-conformance-fixture-v1.schema.json:417-425, 484-505 delete `migrationInput`, `migrationReceipt` and `migrationExpectation`. Its own `schemaVersion` (:60-66) is deliberately open-range for negative probes and stays.
- schemas/conformance-trace-map-v1.json:4 drop `operation:migration`.
- schemas/conformance-trace-map-v1.json:12 drop `boundary:schema.1_0`.
- schemas/conformance-trace-map-v1.json:28 drop `migration` from the `operations` list.
- schemas/conformance-trace-map-v1.json:92 FR-017-AC-1 covers become `boundary:schema.unknown_major`, `boundary:schema.unsupported_minor`, `diagnostic:unsupported_schema_version`.
- scripts/generate_conformance_corpus.py:399-408 replace the three migration cases with package-operation cases: 1.0 and 1.2 (`schema.unsupported_minor`) and 2.0 (`schema.unknown_major`), all `unsupported_schema_version`.
- scripts/generate_conformance_corpus.py:453-454 delete the migration expectation branch.
- corpus/contract-v0.1/inputs/migration-{valid,unregistered,unsupported}.json, expectations/migration-*.json and canonical/migration-valid-0.json: delete; regenerate the corpus with the script's byte-for-byte gate.
- tests/it/canonicalization.rs:2 drop the `migrate_reference_body` import.
- tests/it/canonicalization.rs:69 `package_fixture` builds `SchemaVersion::V1_1`; regenerate the golden bytes at :99 (currently `"minor":0`) and the digest at :104.
- tests/it/canonicalization.rs:169-183 the 1.9 canonicalization case expects the code SR-601 FND-005 settles, not `UnregisteredMigration`.
- tests/it/canonicalization.rs:634-685 rewrite `tc_017_version_preflight_and_registered_migration_fail_closed` (FR-017-AC-1) as preflight-only: 1.1 accepted; 1.0, 1.2/1.9 and 2.0 give `UnsupportedSchemaVersion`; 0.x gives `InvalidSchemaVersion`; missing version gives `InvalidWireFormat`.
- tests/it/conformance.rs:236-240 delete the migration-receipt mutation.
- tests/it/conformance.rs:284 expected kind set drops `migration_receipt` (six kinds).
- tests/it/identity.rs:92,139; tests/it/integration.rs:8-9; tests/it/cycle_free_model.rs:184-185; tests/fixtures/bridge-qsl-consumer/src/lib.rs:9; tests/fixtures/model-alias-consumer/src/lib.rs:5 move off `V1_0` to `V1_1`, adjusting assertions on minor.
- spec/contract-test-matrix.md:26,27,29 keep "implemented" true: until the follow-up merges, FR-017, FR-018 and FR-020 rows claim ✅ against code that still implements migration.
- plan/PLAN-002-contract-ir-v01/TASK-008-canonicalization.md:26-28 and TASK-009-conformance.md:24 still describe the 1.0 source and the migration fixtures. They are historical plan records; update only if the repo treats plans as live.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

- `FR-017-AC-1` (spec/contract/FR-017-version-orphan-coverage.md), examined: Version preflight accepts schema 1.1 and rejects every other major or minor with `unsupported_schema_version` before semantic interpretation.
- `FR-018-AC-2` (spec/contract/FR-018-conformance-corpus.md), examined: Mutation fixtures independently alter schema validity, diagnostic code/path/order/span/obligation, canonical byte, digest, dependency, and coverage row/reason; each produces the exact mismatch result without message parsing.
- `FR-020-AC-2` (spec/interface/FR-020-json-conformance-interface.md), examined: Process fixtures pin exit 1 with all seven mismatch kinds in fixed order and exit 2 for each of the six closed operational codes ...
- `FR-019-AC-2` (spec/interface/FR-019-rust-library-interface.md), examined: The complete negative corpus executes package/expression decode, validation, canonicalization, and coverage through `catch_unwind`; ...
- `TC-017` (spec/contract-test-matrix.md), examined: Canonical bytes, digests, version preflight, and orphan classes conform.

## Verdict

No gap beyond the owner-deferred code change: no new unbacked row and no contradicted status, and every edited AC keeps an existing tagged test (tc_017_version_preflight_and_registered_migration_fail_closed for FR-017-AC-1, tc_018_all_mismatch_kinds_and_exit_classes_are_stable for FR-018-AC-2 and FR-020-AC-2). Those tests assert the old migration behaviour until the follow-up rewrites them per the inventory above.
