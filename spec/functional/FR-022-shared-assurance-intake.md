---
id: FR-022
title: "Adopt the pinned shared assurance intake and legacy compatibility path"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---

# FR-022: Adopt the pinned shared assurance intake and legacy compatibility path

## Description

The repository SHALL reach the shared assurance components only at their
accepted released pins, SHALL deliver its native structured domain result to
Quoin through a declared adapter without either Quire or Quoin executing a
producer, and SHALL read its immutable PGM-01 history only through Engineering
Assurance's read-only compatibility mapping.

## Inputs

- The accepted compatibility matrix packaged with Engineering Assurance 0.2.0,
  which is the authority on which component versions are pinned.
- The observed local toolchain: `quire provenance`, `quoin --version`,
  `ix-flow --version`, and the installed `engineering-assurance` distribution.
- The native conformance runner's `quire.contract.conformance-jsonl/v1` stream
  over `corpus/contract-v0.1/manifest.json`.
- Quire's static export for this repository, `quire coverage --scope . --json`.
- The immutable `evidence/pgm-01-*/` records and `evidence/corrections/`
  bytes, plus declared negative fixtures under `tests/fixtures/legacy-compat/`.
- The declared change-assurance inputs under `assurance/`.

## Outputs

- A pin classification report naming every component, its observed version, and
  one of `compatible`, `incompatible`, or `unknown`.
- A Quoin-retained evidence run for the conformance result, and a Quoin
  change-assurance record, sealed attestations, retained output bytes, and a
  verification receipt.
- A read-only compatibility view per legacy record, carrying the source digest,
  its mapped fields, its unmapped fields, and one of `compatible`, `lossy`,
  `incompatible`, or `unreadable`.

## Behavior

- Version classification is delegated to
  `engineering_assurance.compatibility`; this repository observes versions and
  does not restate the rules that judge them. A component that cannot be
  observed is `unknown`, and `unknown` never satisfies the gate.
- The Engineering Assurance artifacts this repository consumes are additionally
  pinned by SHA-256 in `assurance/pins.json`; a digest that does not match the
  installed distribution fails closed rather than reading the drifted file.
- The producer is invoked natively by the operator or the project build system.
  No shared component executes it: Quire exports static facts, and Quoin
  transcribes, retains, audits, and reports bytes it is handed.
- The structured domain result reaches Quoin through the `contract-conformance`
  adapter over the runner's own protocol. No verdict is read from stdout or
  stderr, and an empty run is refused rather than read as a clean run.
- A sealed attestation binds one already-produced result file to the reviewed
  record, the candidate revision, and the declared result; the retained output
  digest and size are the only fields derived from those bytes.
- A verification receipt is assembled only from explicitly named inputs. An
  unattested proof, an absent human decision, and a stale candidate binding stay
  their own outcomes and are never resolved into a pass.
- The human decision is an ix-flow event and is never synthesized. A receipt
  assembled with no decision recorded is `incomplete`, and that is the honest
  outcome, not a failure of the path.
- The compatibility mapping performs no write to the evidence tree, preserves
  the source digest, and reports `lossy`, `incompatible`, or `unreadable`
  rather than synthesizing a field the legacy record never carried.
- `pass`, `fail`, `unavailable`, `not-computed`, `malformed`, `partial`,
  `stale`, `suspect`, `vacuous`, `tampered`, `unsupported`, and `inconclusive`
  each have a demonstrated case, and no case is satisfied by another's outcome.
- Mutation probes weaken each load-bearing check in turn; a probe that leaves
  the gate green is itself a failure.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-022-AC-1 | Every shared component is classified against the accepted matrix, an unobservable component is `unknown`, and a drifted consumed artifact digest fails closed. | Test (TC-029) |
| FR-022-AC-2 | The native conformance result reaches Quoin through the declared adapter, an empty run is refused, and no verdict is read from a console stream. | Test (TC-030) |
| FR-022-AC-3 | Quire's static export is retained by digest through change-assurance intake, and neither Quire nor Quoin invokes the producer. | Test (TC-031) |
| FR-022-AC-4 | Every immutable legacy record maps read-only with its source digest preserved, and unreadable, unsupported, and tampered inputs each keep their own outcome. | Test (TC-032) |
| FR-022-AC-5 | Each of the twelve required result states has a demonstrated case, and none collapses into another. | Test (TC-033) |
| FR-022-AC-6 | Removing or weakening any load-bearing check turns its gate red. | Test (TC-034) |

## Dependencies

- **Governed by**: [PGM-01](../program/PGM-01-governance.md).
- Supersedes the repository-local evidence verifier that
  [FR-009](./FR-009-evidence-retention.md) previously specified.
- Requires the accepted Engineering Assurance compatibility matrix
  (`agent-ix/engineering-assurance#8`) and migration contract
  (`agent-ix/engineering-assurance#10`).
