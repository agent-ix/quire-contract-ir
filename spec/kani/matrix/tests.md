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
| FR-030 | FR-030-AC-1 through FR-030-AC-5 | TC-042, TC-223 | 🚧 AC-1 through AC-3 implemented and verified by TC-042; AC-4, the SUCCESS check count a `proved` outcome carries, and AC-5, the two `Unavailable` cause codes, are planned (TC-223) |
| FR-031 | FR-031-AC-1 | TC-042 | 🚧 AC-1 implemented and verified by TC-042. FR-031-AC-5, the `KaniOutcome` to QSL `qsl_replay::TerminalValue` map, is retired and its ID is not reused (ADR-0056): the map is owned by agent-ix/quire-contract-codegen, tracked there under Linear IR-358. The family lowerings (`src/kani/arithmetic.rs`, `collections.rs`, `objects.rs`) are codegen's backend adapter's and still live here. |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-042 | Versioned bounded-Kani profile, finite input/outcome firewall and module dispatch conform | Integration | P0 | FR-029, FR-030-AC-1, FR-030-AC-2, FR-030-AC-3, FR-031-AC-1 | 🚧 FR-029-AC-2's `unsupported` negotiation disposition is planned; profile, firewall and dispatch implemented across the shared, arithmetic, graph and collection suites; the arithmetic, graph and collection lowering suites belong with the family lowerings in codegen's backend adapter |
| TC-223 | Kani outcomes are built only by validated constructors, with their check count and cause codes | Integration | P0 | FR-030-AC-4, FR-030-AC-5 | 🚧 planned; the TC-223-tagged test at `tests/it/kani_shared.rs:250` verifies the retired `KaniProviderResult` map and does not back this case |

## Coverage Design

| Test | Coverage rule | Required cases |
|---|---|---|
| TC-042 cases | Coverage, boundary, error, transition | every profile/matrix construct and unknown/conflicting/missing entry; an `unsupported` entry settling its item `unsupported` with a warning naming the item's capability kind from the closed `quire.capability-kind/v1` vocabulary (QSpec FR-290) and no artifact, `KaniOutcome` or `TerminalValue`; no matrix entry for exhaustion, cancellation or an absent executable; each valid, duplicate, dangling, foreign, wrong-type, incomplete, unavailable and one-over-bound population/snapshot/reference/collection case; no-assumption invalid-input probes; every typed outcome kind and Boolean-field absence; arithmetic/definedness, object/reference/graph and collection/query dispatch ownership |
