---
id: TM-003
title: "quire-contract-ir core test matrix"
type: TestMatrix
---
# quire-contract-ir core test matrix

## Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Status |
|---|---|---|---|
| StR-001 | FR-011 through FR-015, FR-019, FR-023, FR-028 through FR-040, FR-044, FR-344, FR-346 | TC-015 through TC-018, TC-035, TC-041 through TC-045, TC-047, TC-048, TC-050 through TC-053, TC-055, TC-056, TC-058, TC-222, TC-223, TC-225, TC-226, TC-227, TC-442, TC-443 | 🚧 bounded-Kani profile, firewall and dispatch, the target-neutral output-mapping foundation, complete-V1 ContractPackage lowering, the V2 reader and its frame entries, operation anchors and state clauses are implemented; the typed `Std001Code` (FR-044) is implemented and its use as `KaniOutcome.code` (FR-030-AC-6, TC-443) is partly implemented, its `proved`-with-count-zero error clause being planned with FR-030-AC-4; the proof check count, the `Unavailable` cause split, provider negotiation, the root and model crate interfaces with no root re-export of the model and the FR-344 refusals are planned |
| StR-002 | FR-016 through FR-018, FR-020 | TC-017, TC-018, TC-019 | 🚧 existing canonicalization and coverage behavior implemented; FR-017-AC-3 through AC-6 and cross-platform NFR-001 criteria await direct evidence |
| StR-003 | FR-012, FR-014, FR-015, FR-017 through FR-020 | TC-015, TC-016, TC-017, TC-018, TC-058 | 🚧 existing behavior implemented; FR-017-AC-3 through AC-6 and NFR-003-AC-3 await exact test evidence |
| StR-004 | FR-045 | TC-446 | 🚧 planned; no derived proof-coverage baseline exists |

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-011 | FR-011-AC-1 through FR-011-AC-3 | TC-015 | 🚧 implemented, including the clause of AC-3 that refuses a zero source or requirement revision and one above 9007199254740992 (2^53) as `invalid_source_revision` or `invalid_requirement_revision`, through the constructors, the package decoder (`tc_015_a_revision_or_byte_offset_above_two_to_the_53_is_refused`), the expression operation and the binding expression decoder (`wire::tests::tc_015_*`, IR-569). Unmet (IR-574): a whole executable projection carrying one is refused `invalid_wire_format` at `projection` because its normative schema (FR-023) is checked first; a clause reference's revision in a binding (`Binding.clause`), the package probe's `clause_resolutions` and the coverage artifact traces are still decoded by serde and give `invalid_wire_format`. STD-001's rows do not list the over-2^53 condition (IR-574) |
| FR-012 | FR-012-AC-1 through FR-012-AC-6 | TC-015, TC-016 | 🚧 implemented, including the clause of AC-6 that refuses a byte offset above 9007199254740992 (2^53) as `invalid_source_span`, through the constructors, the package decoder (`tc_015_a_revision_or_byte_offset_above_two_to_the_53_is_refused`), the expression operation and the binding expression decoder (`wire::tests::tc_015_*`, IR-569). Unmet (IR-574): a whole executable projection carrying one is refused `invalid_wire_format` at `projection` because its normative schema (FR-023) is checked first; the coverage operation's artifact-trace source and target spans (`WireArtifactTrace`) are still decoded by serde and give `invalid_wire_format` rather than `invalid_source_span` |
| FR-044 | FR-044-AC-1 through FR-044-AC-5 | TC-442 | ✅ implemented (Linear IR-605) |
| FR-045 | FR-045-AC-1 through FR-045-AC-6 | TC-446 | 🚧 planned; source-derived population and qualified evidence projections do not exist |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-001 | repeated corpus runs, direct canonical byte comparison for admitted inputs, and ordered diagnostic tuple comparison for rejected inputs | TC-017, TC-019 | 🚧 same-process canonical fixtures exist (TC-017); cross-platform canonical byte equality (AC-2) and ordered diagnostic `(code, path)` equality (AC-3) await direct TC-019 evidence |
| NFR-002 | vocabulary, schema and API inspection | TC-015, TC-016, TC-019 | AC-3/4 implemented; cross-platform AC-1 planned |
| NFR-003 | negative corpus, mutation, panic-free, orphan, and exact-span checks | TC-017 through TC-019 | 🚧 version and orphan fail-closed checks implemented (TC-017, TC-018); exact source/target span checks (AC-3) and threshold analysis (TC-019) planned |

## Diagnostic Registry Coverage

| Registry | Verification | Test Cases | Status |
|---|---|---|---|
| STD-001 | exact registered code sets, precedence, structured fields, and no message parsing | TC-015 through TC-018, TC-223, TC-442, TC-443 | 🚧 closed code catalogs implemented and verified by TC-015 through TC-018; the typed `Std001Code`, the registered Kani cause codes and `invalid_code_form` are implemented and verified by TC-442; `kani_outcome_invalid` is raised by the non-success constructor for a `proved` or `counterexample` request (TC-443, FR-030-AC-6), and is planned for the rest of the validated constructors (TC-223, FR-030-AC-4, AC-5) |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-015 | Package, revision, anchor, clause, dependency, and diagnostic identities conform | Integration | P0 | FR-011, FR-012, NFR-002, STD-001 | ✅ implemented |
| TC-442 | `Std001Code` is built only by validation, a checked literal, a registered constant or a `DiagnosticCode`, serializes as the bare string and spells every registered code once | Unit | P0 | FR-044 | ✅ implemented |
| TC-446 | Proof-coverage baseline retains the population and qualifies evidence | Integration | P0 | FR-045 | 🚧 planned; no executable test |
| TC-019 | Determinism, portability, and fail-closed metrics meet thresholds | Analysis | P0 | NFR-001..NFR-003 | 🚧 planned: direct admitted-input canonical byte comparison, rejected-input ordered diagnostic tuple comparison, portability and fail-closed threshold analysis have no executable test |
