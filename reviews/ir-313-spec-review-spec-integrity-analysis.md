---
id: SR-977
title: "PR #204 integrity review of the accepted AD-001 rulings"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/ (15 files, git diff origin/main...HEAD)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/AD-001
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-031
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-030
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-039
    type: reviews
---
# SR-977: PR #204 integrity review of the accepted AD-001 rulings

Former id: SR-591 (cited by the marker on IR-313).

## Summary

Cross-document consistency review of the spec PR that accepts AD-001 and
states the owner rulings on the root re-export (OQ-1), the Kani lowering
placement (OQ-2) and the unmapped Kani outcomes (OQ-3).

Ticket: IR-313.

Examined: AD-001, AD-002, AD-003, FR-019, FR-028, FR-029 (context), FR-030,
FR-031, FR-036, FR-037, FR-039, TC-055, TC-058, TC-223, spec/index.md,
spec/contract-test-matrix.md. Cross-repo reads (git show, no fetch):
quire-spec-language (the rev IR pins) and origin/main
(`qsl-replay/src/proof_result.rs`, ADR-013 O-16); quire-specification
origin/main (FR-331); quire-contract-codegen origin/main
(ADR-002, FR-007).

## Verdict

**FAIL** until FND-001 is fixed. OQ-1 (no root re-export, by-name model
re-exports) and OQ-2 (profile/abi/dispatch/outcome/provenance stay; family
lowerings to codegen; no replay or witness module) are stated consistently in
every changed file. AD-001 is accepted with no Open questions section and no
OQ reference left anywhere under spec/ outside reviews. The four versioned
contracts are described in current form. FR-019's Public items table lists
233 items and equals the model crate root's public items exactly
(rustdoc all.html, zero deltas, kinds and defining modules match). The FR-031
map rows match O-16 and FR-331, and `UnavailabilityCause::{SolverAbsent,
BackendAbsent}`, `ProofRefusalCause::{Refused, InvalidInput,
IncompleteInput}` and `IncompleteCause::{TimedOut, Cancelled,
ResourceExhausted}` all exist in qsl-replay.
`kani_solver_absent`/`kani_backend_absent` are spelled the same in all five
files that use them. The public-repo grep of added lines finds no WP numbers,
no quire-research reference, no ticket ids, no dates and no ruling narration.

OQ-3 is stated in its superseded form (FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | OQ-3 is stated in its superseded design. A non-vacuous Inconclusive maps to a QSL inconclusive terminal arm, with a typed absence while qsl-replay lacks one. The amended ruling rejects that: map it to an existing QSL value or have the profile refuse the construct up front. The design recurs in AD-001:106-111 and 183-187, AD-002:26 and 45, FR-031:38-41, 58, 85-90 and AC-5, FR-036:74, FR-039:66-70, 85 and AC-3, TC-055:31-35, TC-223 title, 19-20 and 51, index.md:96-100 and matrix rows FR-031, TC-223 and TC-055 cases. | spec/contract/FR-031-bounded-kani-dispatch-replay-provenance.md:85-90 |
| FND-002 | medium | FR-029's matrix `inconclusive` still records "resource exhaustion, cancellation, unavailable Kani executable". The new FR-030 text and the FR-031 map route those cases to Unavailable (kani_backend_absent) and Incomplete, so one condition now has two outcome kinds and two FR-331 results. | spec/contract/FR-029-versioned-bounded-kani-profile.md:31 |
| FND-003 | medium | AD-001, FR-031, FR-036 and FR-039 cite quire-contract-codegen ADR-002 and FR-007 as the owners of the family lowerings. At codegen origin/main, ADR-002 is Proposed and does not mention the lowerings, and FR-007 has the generator consume Contract IR's profile. Neither states that codegen owns them. | spec/assurance/AD-001-contract-ir-architecture.md:88-92 |

## New findings (disposition pass 1)


| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | high | A vacuous proof now maps to "QSL's `inconclusive` terminal value carrying `InconclusiveCause::KaniVacuousProof`" and "never to `Proved`". `qsl_replay::TerminalValue` has no such variant. Its only representation of a vacuous proof is `Proved { success_checks: 0 }`, which has category Inconclusive and vacuous_proof_cause KaniVacuousProof. That is the variant FR-031-AC-5 now forbids, so AC-5 cannot be met. FR-031's Status says "today QSL represents it as `Proved { success_checks: 0 }`", which reintroduces the pending-QSL-arm pattern and conflicts with AD-001's decision "one existing QSL `TerminalValue`, with no new terminal value". The same text is in AD-001:110-112 and 186, FR-031:67 and 136-138, TC-223:45-48 and index.md:98-99. | spec/contract/FR-031-bounded-kani-dispatch-replay-provenance.md:67 |
| FND-005 | medium | The `unsupported` negotiation disposition carries "a warning naming the construct kind" (FR-029:31 and AC-2, AD-001:93, AD-002:26, FR-031:82, FR-036:73-74, TC-042 cases). FR-331-AC-4 and FR-290 require the warning to name the item's capability kind from the closed `quire.capability-kind/v1` vocabulary, and FR-036's next sentence says the same. A `kani-bounded/1` construct id such as `checked-add` is not a member, so an FR-331 reader refuses it as `invalid_capability`/`unknown-kind`. | spec/contract/FR-029-versioned-bounded-kani-profile.md:31 |

## Dispositions

Round 1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | resolved |
| FND-002 | fixed | resolved |
| FND-003 | fixed | resolved |

Round 2.

| FND | outcome | reason |
| --- | --- | --- |
| FND-004 | fixed | resolved |
| FND-005 | fixed | resolved |
