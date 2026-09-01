---
id: FR-021
title: "Reconcile shared assurance ownership and campaign dispositions"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---

# FR-021: Reconcile shared assurance ownership and campaign dispositions

## Description

The campaign SHALL assign each shared verification and assurance responsibility
to exactly one authoritative owner and SHALL remove legacy campaign text that
prescribes a universal executor, common evidence envelope, or parallel
retention authority.

## Inputs

- Engineering Assurance semantic ownership and common-work dependency order.
- Quire static export, Quoin intake/retention/audit/report, native domain-result,
  and ix-flow human-decision boundaries.
- Historical PGM-01 v1/v2 records and corrections.
- Issues #1, #7, #20, and the standalone `quire-evidence` prototype inventory.

## Outputs

- One normative ownership table in PGM-01.
- Reconciled campaign issue text and linked dispositions for issues #7 and #20.
- A preservation/rejection inventory for the legacy prototype.

## Behavior

- Contract and temporal repositories own their domain producers, structured
  domain results, diagnostics, oracles, corpora, and domain failure behavior.
- Quire owns static definition export and never invokes a producer.
- Quoin owns explicit-input validation, retention, integrity, audit, and report
  views and never invokes a producer.
- ix-flow owns attributed human decision events; no tool infers sufficiency.
- Published runtime and domain crates remain runtime-independent from Quire and
  Quoin; development-time adoption uses exact released pins.
- Historical records and corrections remain governed and readable in place.
- Prototype threat cases may become domain-specific fixtures.
- The campaign SHALL NOT adopt the prototype executor, profile, aggregate verdict, authority index, adoption command, or retention model.
- The eight repository migration issues remain outside this governance ticket.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-021-AC-1 | Every shared responsibility appears once with one authoritative owner. | Test (TC-023) |
| FR-021-AC-2 | Domain repositories own producers/results, published crates have no Quire/Quoin runtime dependency, and both Quire and Quoin are explicitly non-executing. | Test (TC-025) |
| FR-021-AC-3 | Issue #1 no longer defers integration architecture; issue #7 is re-scoped and #20 is closed as superseded with replacement-epic links. | Inspection (TC-026) |
| FR-021-AC-4 | The legacy prototype inventory preserves adversarial/domain cases and rejects executor/profile/verdict/authority/retention behavior. | Test (TC-027) |
| FR-021-AC-5 | No campaign document prescribes a conflicting common envelope, executor, or evidence-retention owner. | Static (TC-028) |

## Dependencies

- Engineering Assurance #5 must be reviewed and merged before this
  reconciliation is accepted.
- Quire CLI #74, Quoin CLI #322/#323, and Engineering Assurance #8/#9/#10
  follow this governance gate; they are not implemented here.
