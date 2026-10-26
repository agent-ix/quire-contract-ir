---
id: TM-003
title: "quire-contract-ir core test matrix"
type: TestMatrix
---
# quire-contract-ir core test matrix

## Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Status |
|---|---|---|---|
| StR-001 | FR-011 through FR-015, FR-019, FR-023, FR-028 through FR-040, FR-044, FR-344, FR-346 | TC-015 through TC-018, TC-035, TC-041 through TC-045, TC-047, TC-048, TC-050 through TC-053, TC-055, TC-056, TC-058, TC-222, TC-223, TC-225, TC-226, TC-227, TC-442, TC-443, TC-449 | 🚧 bounded-Kani profile, firewall and dispatch, the target-neutral output-mapping foundation, complete-V1 ContractPackage lowering, the V2 reader and its frame entries, operation anchors and state clauses are implemented; the typed `Std001Code` (FR-044) and its use as `KaniOutcome.code` (FR-030-AC-6, TC-443) are implemented, including the `proved`-with-count-zero error clause; the proof check count and two fixed `Unavailable` causes are implemented and backed by TC-223 (FR-030-AC-4/5); provider negotiation and the FR-344 refusals are planned; the root public inventory, typed no-panic Kani cases and model import boundary are backed by TC-055 and TC-058; TC-055 includes cases also tagged TC-042, TC-443 and TC-449 |
| StR-002 | FR-016 through FR-018, FR-020 | TC-017, TC-018, TC-019, TC-456 | 🚧 existing canonicalization and coverage behavior implemented; FR-016-AC-2 and FR-017-AC-3, AC-4 and AC-6 have direct test evidence; FR-016-AC-1, AC-3, FR-017-AC-2 and cross-platform NFR-001 criteria await direct evidence; TC-456 backs NFR-002-AC-1 and TC-015 backs NFR-002-AC-3 after removal of the temporary registered-code allowance |
| StR-003 | FR-012, FR-014, FR-015, FR-017 through FR-020, FR-056 | TC-015, TC-016, TC-017, TC-018, TC-058, TC-460 | 🚧 existing behavior implemented; FR-017-AC-3 through AC-6 and NFR-003-AC-3 have exact test evidence; FR-017-AC-2 remains unbacked; FR-056/TC-460 is planned and has no consumer gate or actual-source-clean credit |
| StR-004 | FR-045 | TC-446 | 🚧 planned; no derived proof-coverage baseline exists |

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-011 | FR-011-AC-1 through FR-011-AC-3 | TC-015 | 🚧 implemented, including the clause of AC-3 that refuses a zero source or requirement revision and one above 9007199254740992 (2^53) as `invalid_source_revision` or `invalid_requirement_revision`, through the constructors, the package decoder (`tc_015_a_revision_or_byte_offset_above_two_to_the_53_is_refused`), the expression operation and the binding expression decoder (`wire::tests::tc_015_*`, IR-569). AC-3 is scoped to those constructors and decoders (IR-574); a whole executable projection first meets its published schema and refuses an over-2^53 revision as `invalid_wire_format` at `projection` (FR-023-AC-6, implemented in TC-035). Probe and coverage inputs meet their fixture schema before typed decoding (FR-020-AC-5). STD-001 lists the over-2^53 conditions. |
| FR-012 | FR-012-AC-1 through FR-012-AC-6 | TC-015, TC-016, TC-018 | 🚧 implemented, including the clause of AC-6 that refuses a byte offset above 9007199254740992 (2^53) as `invalid_source_span`, through the constructors, the package decoder (`tc_015_a_revision_or_byte_offset_above_two_to_the_53_is_refused`), the expression operation and the binding expression decoder (`wire::tests::tc_015_*`, IR-569). AC-6 is scoped to those constructors and decoders (IR-574); a whole executable projection first meets its published schema and refuses an over-2^53 offset as `invalid_wire_format` at `projection` (FR-023-AC-6, implemented in TC-035). TC-018 verifies that over-bound coverage trace offsets instead meet their fixture schema before typed decoding and return `invalid_corpus` at `fixtures.coverage-deep.input`; the exact bound passes that schema. |
| FR-044 | FR-044-AC-1 through FR-044-AC-5 | TC-442 | ✅ implemented (Linear IR-605) |
| FR-045 | FR-045-AC-1 through FR-045-AC-6 | TC-446 | 🚧 planned; source-derived population and qualified evidence projections do not exist |
| FR-056 | FR-056-AC-1 through FR-056-AC-8 | TC-460 | 🚧 planned; authoritative external scanner consumer wiring, real mutation and neutral-marker compiler controls, open-key and error propagation checks, edge/rule review, and actual-source clean qualification remain unimplemented; fixture preparation alone gives no implementation credit |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-001 | repeated corpus runs, direct canonical byte comparison for admitted inputs, and ordered diagnostic tuple comparison for rejected inputs | TC-017, TC-019 | 🚧 same-process canonical fixtures exist (TC-017); cross-platform canonical byte equality (AC-2) and ordered diagnostic `(code, path)` equality (AC-3) await direct TC-019 evidence |
| NFR-002 | vocabulary, schema and API inspection | TC-015, TC-016, TC-456 | ✅ AC-1, AC-3 and AC-4 implemented: the registered Kani absence code is `kani_checker_absent`; TC-456 has no forbidden-name allowance, and TC-015 checks diagnostic wire vocabulary. |
| NFR-003 | negative corpus, mutation, panic-free, orphan, and exact-span checks | TC-017, TC-018, TC-454 | ✅ AC-1 and AC-2 implemented (TC-454, `tests/it/nfr003_invalid_input.rs`): every STD-001 semantic code class driven through `run_corpus` and package decoding with a stable code and no panic, and every rejected corpus fixture without canonical identity or covered status; AC-3 exact source/target span checks are tagged in TC-017 |

## Diagnostic Registry Coverage

| Registry | Verification | Test Cases | Status |
|---|---|---|---|
| STD-001 | exact registered code sets, precedence, structured fields, and no message parsing | TC-015 through TC-018, TC-223, TC-442, TC-443 | ✅ closed code catalogs implemented and verified by TC-015 through TC-018; the typed `Std001Code`, the registered Kani cause codes and `invalid_code_form` are implemented and verified by TC-442; `kani_outcome_invalid` is raised for zero-count `proved` requests and invalid broad non-success requests, verified by TC-223 and TC-443 (FR-030-AC-4 through AC-6) |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-015 | Package, revision, anchor, clause, dependency, and diagnostic identities conform | Integration | P0 | FR-011, FR-012, NFR-002, STD-001 | ✅ `tests/it/identity.rs` checks diagnostic wire vocabulary and carries NFR-002-AC-3 after the registered-code rename. |
| TC-442 | `Std001Code` is built only by validation, a checked literal, a registered constant or a `DiagnosticCode`, serializes as the bare string and spells every registered code once | Unit | P0 | FR-044 | ✅ implemented |
| TC-446 | Proof-coverage baseline retains the population and qualifies evidence | Integration | P0 | FR-045 | 🚧 planned; no executable test |
| TC-019 | Determinism metrics meet thresholds | Analysis | P0 | NFR-001 | 🚧 planned: direct admitted-input canonical byte comparison and rejected-input ordered diagnostic tuple comparison have no executable test; NFR-002-AC-1 is TC-456's |
| TC-456 | Public schema and model names exclude implementation-specific vocabulary | Unit | P0 | NFR-002-AC-1 | ✅ source/schema inventory, schema `const` values, public associated items, seven-table STD-001 registry reconciliation, and extractor-fed adverse controls run with no allowance; the retired Kani solver code is rejected. |
| TC-460 | Shared string-edge consumer rejects post-intake wire dispatch | Integration | P0 | FR-056 | 🚧 planned; no normal consumer gate or actual-source-clean acceptance evidence; AD-004 Q-6 remains outside this syntactic scan claim |
| TC-454 | Every declared invalid-input class fails closed with a stable code, no public panic, no canonical identity and no covered status | Integration | P0 | NFR-003 | 🚧 AC-1 and AC-2 implemented in `tests/it/nfr003_invalid_input.rs`; AC-3 is TC-017's |
