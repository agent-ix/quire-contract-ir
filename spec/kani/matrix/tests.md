---
id: TM-007
title: "quire-contract-ir bounded Kani test matrix"
type: TestMatrix
---
# quire-contract-ir bounded Kani test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-029 | FR-029-AC-1 through FR-029-AC-3 | TC-042 | 🚧 AC-1 and AC-3 implemented; AC-2 is planned: the matrix admits only `Supported` and `Refused` and rejects legacy `inconclusive` dispositions; the missing `unsupported` entry must settle the item at negotiation with a warning naming the item's capability kind from the closed `quire.capability-kind/v1` vocabulary (QSpec FR-290) and no artifact or outcome |
| FR-030 | FR-030-AC-1 through FR-030-AC-7 | TC-042, TC-223, TC-443, TC-449 | 🚧 AC-1 through AC-3 implemented and verified by TC-042; AC-6, the `Std001Code` type of `KaniOutcome.code` (FR-044, IR-605), is partly implemented and verified by TC-443, and its clause that reads the `KaniOutcomeError` code for a `proved` request with a count of zero is planned with AC-4 (the test reads it through the non-success request for a `proved` outcome); AC-4, the SUCCESS check count a `proved` outcome carries, and AC-5, the two `Unavailable` cause codes and the private fields, are planned (TC-223, no test); AC-7 has TC-449 IR carrier tests, while CG's live cancellation-event reversal control remains planned under IR-701 |
| FR-031 | FR-031-AC-1 | TC-042 | 🚧 AC-1 implemented and verified by TC-042. FR-031-AC-5, the `KaniOutcome` to QSL `qsl_replay::TerminalValue` map, is retired and its ID is not reused (ADR-0056): the map is owned by agent-ix/quire-contract-codegen, tracked there under Linear IR-358. IR retains the three-family dispatch index and no family lowerings. Codegen retains collection and graph lowerers; arithmetic was retired under FR-015-AC-109. |
| FR-036 | FR-036-AC-1 through FR-036-AC-5 | TC-045 | 🚧 planned; shared by Contract IR's `ContractPackage` and the codegen provider, neither implemented |
| FR-037 | FR-037-AC-6 | TC-055 | ✅ TC-055 checks the public surface and source for replay and witness absence. |
| FR-039 | FR-039-AC-1 through FR-039-AC-4 | TC-055 | ✅ AC-1 through AC-3 are backed by exact inventory, excluded-item compile-fail probes and source checks. AC-4 is backed by TC-055-tagged typed-error, no-panic cases under `catch_unwind` for profile, finite-input, dispatch, outcome-construction and public JSON-reader refusals. |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-042 | Versioned bounded-Kani profile, finite input/outcome firewall and module dispatch conform | Integration | P0 | FR-029, FR-030-AC-1, FR-030-AC-2, FR-030-AC-3, FR-031-AC-1 | 🚧 FR-029-AC-2's `unsupported` negotiation disposition is planned; IR profile, firewall and three-family dispatch controls remain in `kani_shared.rs`; codegen retains collection and graph lowerers; arithmetic was retired under FR-015-AC-109 |
| TC-223 | Kani outcomes are built only by validated constructors, with their check count and cause codes | Integration | P0 | FR-030-AC-4, FR-030-AC-5 | 🚧 planned; no test is tagged for it; the obsolete IR-owned `KaniProviderResult` map and its test were removed under IR-347 and did not back this case |
| TC-443 | A Kani outcome and its error carry a typed STD-001 code | Integration | P0 | FR-030-AC-6 | 🚧 partly implemented: `tc_443_the_outcome_and_its_error_carry_a_typed_std001_code` in `tests/it/kani_shared.rs`; the string-argument probes are `compile_fail` doctests on `KaniOutcome::non_success`, run by `make test`; the clause for a `proved` request with a count of zero is planned with FR-030-AC-4 |
| TC-449 | Kani cancellation carries the producer's requested or deadline origin | Integration | P0 | FR-030-AC-7 | 🚧 IR carrier and serialization tests tagged in `tests/it/kani_shared.rs`; CG IR-701 still owns the actual Kani event-to-outcome reversal control and QSL map |
| TC-045 | Exact backend capability negotiation conforms | Integration | P0 | FR-036 | 🚧 planned; mirrors quire-specification:TC-218; binds the no-advertised-capability absence path (quire-specification:FR-290 vocabulary) |
| TC-055 | The root crate's public interface is exactly the listed items and names no QSL-owned replay type | Integration | P0 | FR-039, FR-037-AC-6 | ✅ inventory, forbidden-source checks, per-item compile-fail probes and TC-055-tagged negative cases for public `kani` constructors, validators, dispatch and JSON readers implemented |

## Coverage Design

| Test | Coverage rule | Required cases |
|---|---|---|
| TC-042 cases | Coverage, boundary, error, transition | every profile/matrix construct and unknown/conflicting/missing entry; an `unsupported` entry settling its item `unsupported` with a warning naming the item's capability kind from the closed `quire.capability-kind/v1` vocabulary (QSpec FR-290) and no artifact, `KaniOutcome` or `TerminalValue`; no matrix entry for exhaustion, cancellation or an absent executable; each valid, duplicate, dangling, foreign, wrong-type, incomplete, unavailable and one-over-bound population/snapshot/reference/collection case; no-assumption invalid-input probes; every typed outcome kind and Boolean-field absence; arithmetic/definedness, object/reference/graph and collection/query dispatch ownership |
| TC-055 cases | Coverage, compile-fail, closure | public-item inventory equal to FR-039's table with no model item; no `pub use` of `quire_contract_model` in `src/lib.rs` and a failing model-item probe through `quire_contract_ir`; no `replay`, `witness`, `arithmetic`, `collections` or `objects` module under `src/kani/`; no `quire_spec_language::runtime` name under `src/`; one compile-fail probe per QSL-owned and codegen-owned item; a search of `src/` finding no `KaniOutcome` to `TerminalValue` map (owned by agent-ix/quire-contract-codegen, Linear IR-358); negative cases for public `kani` constructors, validators, dispatch and JSON readers under `catch_unwind` |
