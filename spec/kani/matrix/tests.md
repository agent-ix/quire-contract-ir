---
id: TM-007
title: "quire-contract-ir bounded Kani test matrix"
type: TestMatrix
---
# quire-contract-ir bounded Kani test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-029 | FR-029-AC-1 through FR-029-AC-3 | TC-042 | 🚧 AC-1 and AC-3 implemented; AC-2 is planned: the matrix has a `CapabilityDisposition::Inconclusive` entry that becomes an `Inconclusive` run outcome, where the target is an `unsupported` entry that settles the item at negotiation with a warning naming the item's capability kind from the closed `quire.capability-kind/v1` vocabulary (QSpec FR-290) and no artifact or outcome |
| FR-030 | FR-030-AC-1 through FR-030-AC-6 | TC-042, TC-223, TC-443 | 🚧 AC-1 through AC-3 implemented and verified by TC-042; AC-6, the `Std001Code` type of `KaniOutcome.code` (FR-044, IR-605), is partly implemented and verified by TC-443, and its clause that reads the `KaniOutcomeError` code for a `proved` request with a count of zero is planned with AC-4 (the test reads it through the non-success request for a `proved` outcome); AC-4, the SUCCESS check count a `proved` outcome carries, and AC-5, the two `Unavailable` cause codes and the private fields, are planned (TC-223, no test) |
| FR-031 | FR-031-AC-1 | TC-042 | 🚧 AC-1 implemented and verified by TC-042. FR-031-AC-5, the `KaniOutcome` to QSL `qsl_replay::TerminalValue` map, is retired and its ID is not reused (ADR-0056): the map is owned by agent-ix/quire-contract-codegen, tracked there under Linear IR-358. The family lowerings (`src/kani/arithmetic.rs`, `collections.rs`, `objects.rs`) are codegen's backend adapter's and still live here. |
| FR-036 | FR-036-AC-1 through FR-036-AC-5 | TC-045 | 🚧 planned; shared by Contract IR's `ContractPackage` and the codegen provider, neither implemented |
| FR-037 | FR-037-AC-6 | TC-055 | 🚧 planned: TC-055's public-surface and source checks also show that Contract IR defines no replay envelope or witness, has no `replay` or `witness` module and calls no executor; the replay and witness modules no longer exist, and the row stays planned until TC-055's checks are authored. |
| FR-039 | FR-039-AC-1 through FR-039-AC-4 | TC-055 | 🚧 planned; the root crate no longer re-exports the model or the QSL-owned `KaniProviderRecord` and `KaniProviderResult`; it still exports the family lowerings codegen owns, and TC-055's complete public-inventory and negative-corpus checks remain to be authored |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-042 | Versioned bounded-Kani profile, finite input/outcome firewall and module dispatch conform | Integration | P0 | FR-029, FR-030-AC-1, FR-030-AC-2, FR-030-AC-3, FR-031-AC-1 | 🚧 FR-029-AC-2's `unsupported` negotiation disposition is planned; profile, firewall and dispatch implemented across the shared, arithmetic, graph and collection suites; the arithmetic, graph and collection lowering suites belong with the family lowerings in codegen's backend adapter |
| TC-223 | Kani outcomes are built only by validated constructors, with their check count and cause codes | Integration | P0 | FR-030-AC-4, FR-030-AC-5 | 🚧 planned; no test is tagged for it; the obsolete IR-owned `KaniProviderResult` map and its test were removed under IR-347 and did not back this case |
| TC-443 | A Kani outcome and its error carry a typed STD-001 code | Integration | P0 | FR-030-AC-6 | 🚧 partly implemented: `tc_443_the_outcome_and_its_error_carry_a_typed_std001_code` in `tests/it/kani_shared.rs`; the string-argument probes are `compile_fail` doctests on `KaniOutcome::non_success`, run by `make test`; the clause for a `proved` request with a count of zero is planned with FR-030-AC-4 |
| TC-045 | Exact backend capability negotiation conforms | Integration | P0 | FR-036 | 🚧 planned; mirrors quire-specification:TC-218; binds the no-advertised-capability absence path (quire-specification:FR-290 vocabulary) |
| TC-055 | The root crate's public interface is exactly the listed items and names no QSL-owned replay type | Integration | P0 | FR-039, FR-037-AC-6 | 🚧 planned; the two removed provider types have compile-fail doctest probes, but the complete inventory and remaining cases are not implemented |

## Coverage Design

| Test | Coverage rule | Required cases |
|---|---|---|
| TC-042 cases | Coverage, boundary, error, transition | every profile/matrix construct and unknown/conflicting/missing entry; an `unsupported` entry settling its item `unsupported` with a warning naming the item's capability kind from the closed `quire.capability-kind/v1` vocabulary (QSpec FR-290) and no artifact, `KaniOutcome` or `TerminalValue`; no matrix entry for exhaustion, cancellation or an absent executable; each valid, duplicate, dangling, foreign, wrong-type, incomplete, unavailable and one-over-bound population/snapshot/reference/collection case; no-assumption invalid-input probes; every typed outcome kind and Boolean-field absence; arithmetic/definedness, object/reference/graph and collection/query dispatch ownership |
| TC-055 cases | Coverage, compile-fail, closure | public-item inventory equal to FR-039's table with no model item; no `pub use` of `quire_contract_model` in `src/lib.rs` and a failing model-item probe through `quire_contract_ir`; no `replay`, `witness`, `arithmetic`, `collections` or `objects` module under `src/kani/`; no `quire_spec_language::runtime` name under `src/`; one compile-fail probe per QSL-owned and codegen-owned item; a search of `src/` finding no `KaniOutcome` to `TerminalValue` map (owned by agent-ix/quire-contract-codegen, Linear IR-358); negative corpora under `catch_unwind` |
