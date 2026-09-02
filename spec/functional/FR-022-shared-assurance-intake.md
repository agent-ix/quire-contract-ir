---
id: FR-022
title: "Adopt the pinned shared assurance intake path"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---

# FR-022: Adopt the pinned shared assurance intake path

## Description

The repository SHALL reach the shared assurance components only at their
accepted released pins, and SHALL deliver its native structured domain result
to Quoin through a declared adapter without either Quire or Quoin executing a
producer.

## Inputs

- The accepted compatibility matrix packaged with Engineering Assurance 0.2.0,
  which is the authority on which component versions are pinned.
- The observed local toolchain: `quire provenance`, `quoin --version`,
  `ix-flow --version`, and the installed `engineering-assurance` distribution.
- The native conformance runner's `quire.contract.conformance-jsonl/v1` stream
  over `corpus/contract-v0.1/manifest.json`.
- Quire's static export for this repository, `quire coverage --scope . --json`.
- The declared change-assurance inputs under `assurance/`.

## Outputs

- A pin classification report naming every component, its observed version, and
  one of `compatible`, `incompatible`, or `unknown`.
- A Quoin-retained evidence run for the conformance result, and a Quoin
  change-assurance record, sealed attestations, retained output bytes, and a
  verification receipt.

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
  stderr. An empty run, a stream declaring a protocol the adapter does not
  support, and a stream whose bytes do not decode are each refused rather than
  read as a clean run, and each refusal states its own reason so that `vacuous`,
  `unsupported` and `malformed` cannot stand in for one another.
- A sealed attestation binds one already-produced result file to the reviewed
  record, the candidate revision, and the declared result; the retained output
  digest and size are the only fields derived from those bytes.
- A verification receipt is assembled only from explicitly named inputs. An
  unattested proof, an absent human decision, and a stale candidate binding stay
  their own outcomes and are never resolved into a pass.
- The human decision is an ix-flow event and is never synthesized. A receipt
  assembled with no decision recorded is `incomplete`, and that is the honest
  outcome, not a failure of the path.
- `pass`, `fail`, `unavailable`, `not-computed`, `partial`, `stale`,
  `tampered`, `vacuous`, `unsupported`, and `malformed` each have a
  demonstrated case, and no case is satisfied by another's outcome. A
  state's demonstrator is named per state, not pooled: a state whose named
  demonstrator stops showing it fails the gate rather than being covered by a
  neighbour.
- `suspect` and `inconclusive` have no demonstrated case in this repository and
  are declared as lost rather than reassigned. Neither is satisfied by any other
  state's outcome.
- Mutation probes weaken each load-bearing check in turn; a probe that leaves
  the gate green is itself a failure.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-022-AC-1 | Every shared component is classified against the accepted matrix, an unobservable component is `unknown`, and a drifted consumed artifact digest fails closed. | Test (TC-029) |
| FR-022-AC-2 | The native conformance result reaches Quoin through the declared adapter; an empty run, a foreign protocol, and a stream carrying undecodable bytes are each refused and each names its own reason; and no verdict is read from a console stream. | Test (TC-030) |
| FR-022-AC-3 | Quire's static export is retained by digest through change-assurance intake, and neither Quire nor Quoin invokes the producer. | Test (TC-031) |
| FR-022-AC-5 | Each of the ten demonstrated result states has a named demonstrator, none collapses into another, and `suspect` and `inconclusive` stay declared lost rather than quietly covered. | Test (TC-033) |
| FR-022-AC-6 | Removing or weakening any load-bearing check turns its gate red. | Test (TC-034) |

### Retired criteria

`FR-022-AC-4` required every immutable legacy record to map read-only through
Engineering Assurance's compatibility mapping with its source digest preserved,
and unreadable, unsupported and tampered inputs each to keep their own outcome.
The repository owner released the evidence-preservation constraint for the
pre-stable phase on 2026-09-02, recorded in
[engineering-assurance#7](https://github.com/agent-ix/engineering-assurance/issues/7)
under "Preservation constraint released for the pre-stable phase". The retained
records, the census that read them, and their declared negative fixtures are
deleted, so the criterion is retired rather than restated: there is no legacy
record left to map, and a weaker version of this claim would assert something
nobody checks. The identifier is not reused.

`FR-022-AC-5` previously required twelve result states. The demonstrator for
each was measured individually against the pre-deletion tree, before anything
was removed, rather than by checking whether the union of all sources still
reached twelve — a union check would have stayed green while a state silently
lost its only demonstrator. The measurement found the Quoin chain showing
`pass`, `fail`, `unavailable`, `not-computed`, `partial`, `stale` and
`tampered`; the adapter showing `vacuous`; and the compatibility census over the
retained records the sole declared home of `unsupported`, `inconclusive`,
`malformed` and `suspect` — of which, on inspection, only `inconclusive` and
`suspect` were demonstrated by it at all; `unsupported` and `malformed` were
unconditional literals in its set, backed by nothing.

Each of those four was then checked against the surviving path rather than
written off:

- `unsupported` is demonstrated by the adapter's `refuses-a-foreign-protocol`
  probe. It is worth being exact about what changed, because the honest version
  is less flattering than "rehomed": before this change `unsupported` was an
  unconditional string literal in the census's demonstrated set. Nothing bound
  it to the census or to anything else. The probe ran on every invocation and
  no gate connected the two, so the state was declared demonstrated and was
  not. It is now bound to that probe's recorded outcome, which is a
  strengthening rather than a like-for-like move.
- `malformed` was the same unconditional literal, demonstrated by nothing. It
  is now bound to a new adapter refusal of a stream whose bytes do not parse.
- `inconclusive` was demonstrated by TC-006, which validated a live solver
  fixture in the domain governance corpus and required its inconclusive result
  to stay inconclusive.
- `suspect` is a genuine loss. It meant "a retained record that an append-only
  correction names". With no retained records and no corrections, nothing in
  this repository is suspect, and no stand-in was invented. It is recorded as
  lost with a test asserting it stays recorded, so restoring it requires saying
  what demonstrates it.

**Amended again when PGM-01-R08 was withdrawn.** The `quire.derivation-evidence/v1`
envelope schema, `scripts/validate_governance.py` and the `corpus/governance/`
fixtures are deleted with that rule. TC-006 went with them, so `inconclusive`
lost the demonstrator it had just been rehomed onto and joins `suspect` as a
declared loss. It was not moved a second time: no other component in this
repository produces an inconclusive result, and binding the state to a
neighbouring outcome is exactly the collapse this table exists to prevent.

Ten of the twelve states therefore keep a named demonstrator and two are
declared lost. `suspect` and `inconclusive` will each need a demonstrated case
before this repository moves toward a stable release.

**What "demonstrated" means here, stated plainly because the phrase reads
stronger than it is.** It means each state was produced by the component that
owns it and observed at that component's own boundary — a chain scenario's
declared outcome, an adapter refusal's recorded reason, a corpus fixture's
status. It does **not** mean eleven distinct values arrive at the verification
receipt, and it does not mean the states are distinguishable everywhere
downstream. The three adapter-borne states are a case in point: all three
refusals exit 1, so at the exit-code boundary they are one value, and what keeps
them apart is the reason each records. That is why the refusals are checked
before intake and compared by reason rather than by exit status — after the
adapter, a refused row and a failed row are the same non-zero integer, which is
precisely where `malformed` would collapse into `fail`.

## Dependencies

- **Governed by**: [PGM-01](../program/PGM-01-governance.md).
- Supersedes the repository-local evidence verifier that
  [FR-009](./FR-009-evidence-retention.md) previously specified.
- Requires the accepted Engineering Assurance compatibility matrix
  (`agent-ix/engineering-assurance#8`) and migration contract
  (`agent-ix/engineering-assurance#10`).
