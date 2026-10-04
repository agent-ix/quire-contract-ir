---
id: TM-003
title: "quire-contract-ir core test matrix"
type: TestMatrix
---
# quire-contract-ir core test matrix

## Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Status |
|---|---|---|---|
| StR-001 | FR-011 through FR-015, FR-019, FR-023, FR-028 through FR-040, FR-044, FR-344, FR-346 | TC-015 through TC-018, TC-035, TC-041 through TC-045, TC-047, TC-048, TC-050 through TC-053, TC-055, TC-056, TC-058, TC-222, TC-223, TC-225, TC-442 | 🚧 bounded-Kani profile, firewall and dispatch, the target-neutral output-mapping foundation, complete-V1 ContractPackage lowering, the V2 reader and its frame entries, operation anchors and state clauses are implemented; the proof check count, the `Unavailable` cause split, the typed `Std001Code` (FR-044) and its use as `KaniOutcome.code` (FR-030-AC-6), provider negotiation, the root and model crate interfaces with no root re-export of the model and the FR-344 refusals are planned |
| StR-002 | FR-016 through FR-018, FR-020 | TC-017, TC-018 | ✅ implemented |
| StR-003 | FR-012, FR-014, FR-015, FR-017 through FR-020 | TC-015, TC-016, TC-018, TC-058 | 🚧 implemented except FR-019-AC-5, the model crate's explicit public item list (TC-058, planned; FR-019-AC-6, the artifact-reference member sets, is implemented) |

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-011 | FR-011-AC-1 through FR-011-AC-3 | TC-015 | 🚧 implemented, including the clause of AC-3 that refuses a zero source or requirement revision and one above 9007199254740992 (2^53) as `invalid_source_revision` or `invalid_requirement_revision`, through the constructors, the package decoder (`tc_015_a_revision_or_byte_offset_above_two_to_the_53_is_refused`), the expression operation and the binding expression decoder (`wire::tests::tc_015_*`, IR-569). Unmet (IR-574): a whole executable projection carrying one is refused `invalid_wire_format` at `projection` because its normative schema (FR-023) is checked first; a clause reference's revision in a binding (`Binding.clause`), the package probe's `clause_resolutions` and the coverage artifact traces are still decoded by serde and give `invalid_wire_format`. STD-001's rows do not list the over-2^53 condition (IR-574) |
| FR-012 | FR-012-AC-1 through FR-012-AC-6 | TC-015, TC-016 | 🚧 implemented, including the clause of AC-6 that refuses a byte offset above 9007199254740992 (2^53) as `invalid_source_span`, through the constructors, the package decoder (`tc_015_a_revision_or_byte_offset_above_two_to_the_53_is_refused`), the expression operation and the binding expression decoder (`wire::tests::tc_015_*`, IR-569). Unmet (IR-574): a whole executable projection carrying one is refused `invalid_wire_format` at `projection` because its normative schema (FR-023) is checked first; the coverage operation's artifact-trace spans (`WireArtifactTrace` source and target spans, `WireTraceDepth` digest span) are still decoded by serde and give `invalid_wire_format` rather than `invalid_source_span` |
| FR-044 | FR-044-AC-1 through FR-044-AC-5 | TC-442 | 🚧 planned (Linear IR-605): the `code` module of `quire-contract-model` does not exist; `KaniOutcome.code` is a `String` |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-001 | repeated golden corpus and cross-platform comparison | TC-017, TC-019 | same-process goldens implemented (TC-017); cross-platform comparison (TC-019) planned |
| NFR-002 | vocabulary, schema and API inspection | TC-015, TC-016, TC-019 | AC-3/4 implemented; cross-platform AC-1 planned |
| NFR-003 | negative corpus, mutation, panic-free, and orphan checks | TC-017 through TC-019 | version and orphan fail-closed checks implemented (TC-017, TC-018); threshold analysis (TC-019) planned |

## Diagnostic Registry Coverage

| Registry | Verification | Test Cases | Status |
|---|---|---|---|
| STD-001 | exact registered code sets, precedence, structured fields, and no message parsing | TC-015 through TC-018, TC-223, TC-442 | 🚧 closed code catalogs implemented and verified by TC-015 through TC-018; `kani_outcome_invalid`, raised by the validated `KaniOutcome` constructors, is planned (TC-223), as are the typed `Std001Code`, the registered Kani cause codes and `invalid_code_form` (TC-442) |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-015 | Package, revision, anchor, clause, dependency, and diagnostic identities conform | Integration | P0 | FR-011, FR-012, NFR-002, STD-001 | ✅ implemented |
| TC-442 | `Std001Code` is built only by validation or a checked literal, serializes as the bare string and spells every registered code once | Unit | P0 | FR-044 | 🚧 planned: no test is tagged for it |
| TC-019 | Determinism, portability, and fail-closed metrics meet thresholds | Analysis | P0 | NFR-001..NFR-003 | 🚧 planned: cross-platform determinism, portability and fail-closed threshold analysis has no executable test |

