---
id: FR-037
title: "Hand backend counterexamples to replay only through the QSL replay envelope"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/AD-003
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-039
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-036
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-197
    type: references
  - target: ix://agent-ix/quire-specification/FR-336
    type: references
  - target: ix://agent-ix/quire-specification/AD-016
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-070
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-098
    type: references
---
# FR-037: Hand backend counterexamples to replay only through the QSL replay envelope

## Description

Contract IR shall define no replay envelope, replay request, replay result,
parity verdict or minimization lineage of its own. When a backend reports a
counterexample for a ContractPackage item, Contract IR shall not invoke a
replay executor.
Replay of that counterexample is QSL's and codegen's: QSL's `qsl-replay` crate
owns the `WitnessEnvelope`, `ReplayRequest`, `ReplayResult` and the facade
`qsl_replay::replay` (QSL ADR-011 E9, FR-098), and the codegen replay adapter
builds the envelope and calls the facade.

## Inputs

None of Contract IR's own. The envelope's members are QSL's (QSL FR-070); the
checked package, obligation and backend identities it binds come from QSL's
compile and the codegen backend adapter.

## Outputs

None of Contract IR's own. The replay result is QSL's `ReplayResult`.

## Behavior

Canonical round-trip, backend-result digest verification, exact value and
IEEE width preservation, verdict parity and minimization lineage are
properties of the QSL envelope and replay executor (QSL FR-070 through
FR-073, FR-098) and of the codegen replay adapter, not of Contract IR.
Contract IR's part in a counterexample ends at the typed `KaniOutcome` and its
`TerminalValue` ([FR-031](./FR-031-bounded-kani-dispatch-and-terminal-map.md)).
The root crate has no `replay` or `witness` module: the witness and replay
types are QSL's `qsl-replay` types, and the Kani transcript parser is the
codegen backend adapter's.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-037-AC-6 | Contract IR's public API names no replay envelope, replay request, replay result, witness, parity verdict or minimization type other than QSL's, the root crate has no `replay` or `witness` module, and no Contract IR source calls a replay executor. | Test (TC-055) |

## Dependencies

[FR-036](./FR-036-exact-backend-negotiation-and-emission.md) owns backend
inputs. QSpec FR-197 owns the normative replay and corpus contract; QSL
`qsl-replay` implements it.
