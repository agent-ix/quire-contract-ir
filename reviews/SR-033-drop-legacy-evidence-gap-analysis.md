---
id: SR-033
title: "Gap analysis — drop the legacy evidence cruft"
type: SpecReview
analysis: gap-analysis
scope: "branch chore/drop-legacy-evidence against origin/main 69cf238; spec/, tests/, schemas/, assurance/, Makefile"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-022
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-009
    type: references
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---

# SR-033: Gap analysis — drop the legacy evidence cruft

## Summary

Whether the deletion left a gap: an orphaned matrix row, a spec claim nothing
checks, a gate that lost strength without going red, or an argument that still
rests on evidence no longer present.

## Verdict

**PASS** with one declared, tested loss (`suspect`) and one accepted reduction
(six mutation probes). Neither is hidden; both are stated in the specification
and, where testable, held in place by a test.

## Exit criteria

| Criterion | Evidence |
| --- | --- |
| No matrix row becomes unbacked | `quire coverage --scope . --json` reports `unbacked_rows: 0` before and after. Rows fell 118 → 111 and every one of the seven reconciles to a deliberately retired criterion or test case. |
| No spec claim survives without a check behind it | Four acceptance criteria retired in writing (FR-009-AC-4, FR-009-AC-5, FR-022-AC-4, NFR-004-AC-4), each with the authority cited and the identifier marked not reused. `quire coverage --strict` passes. |
| No gate silently weakened | Mutation probes 13 → 7, stated, all six removed ones guarding the removed census. Adapter probes 3 → 4. The state check is per-state, not a union; probed red. |
| No argument rests on deleted evidence | Whole `spec/` tree swept; four live arguments found and amended (`AA-001` Challenges, `AA-001` toolchain assumption, `AD-001` risk control, `AP-001` impact controls, `spec/index.md` scope). |
| The release gate stays satisfiable | `make release-check` = `ci spec`; both green. `AA-001`'s Sufficiency Decision never gated on retained evidence and still resolves. |
| Nothing rewritten to look like it still verifies | Deletion only. No record backdated or re-sealed; none could be, since none survives. |

## Traceability after the change

| Requirement | Criteria | Test cases | Status |
| --- | --- | --- | --- |
| FR-009 | AC-1, AC-2, AC-6 | TC-004, TC-006, TC-025 | covered |
| FR-022 | AC-1, AC-2, AC-3, AC-5, AC-6 | TC-029, TC-030, TC-031, TC-033, TC-034 | covered |
| NFR-004 | AC-1, AC-2, AC-3 | TC-014, TC-020, TC-021 | covered |
| PGM-01-R09 | AC-1, AC-2 | TC-004, TC-029, TC-033 | covered |

Retired and not reused: `FR-009-AC-4`, `FR-009-AC-5`, `FR-022-AC-4`,
`NFR-004-AC-4`. Deleted test cases: `TC-022`, `TC-024`, `TC-032`.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-401 | medium | `NFR-004-AC-4` was backed only by a test reading the deleted correction record | spec/nonfunctional/NFR-004-assurance-release-boundary.md | missing-requirement |
| FND-402 | medium | Two required states were declared demonstrated while bound to nothing at all | tests/test_shared_assurance.py | implementation-bug-despite-evidence |
| FND-403 | low | `corpus/evidence-corrections/` named the deleted record as its only valid fixture, with no runner behind it | corpus/evidence-corrections/manifest.json | correct-requirement-no-evidence |
| FND-404 | low | The historical lock also froze a live schema, so deleting it unfroze `derivation-evidence-envelope-v1` | schemas/README.md | correct-requirement-no-evidence |
| FND-405 | low | `suspect` has no demonstrated case and cannot be given one without inventing a stand-in | spec/functional/FR-022-shared-assurance-intake.md | missing-requirement |
| FND-406 | medium | The first `malformed` demonstrator corrupted the only row in its stream, so a dropping adapter and a refusing adapter produced the same non-zero exit | scripts/assurance_chain.py | implementation-bug-despite-evidence |
| FND-407 | medium | `assurance/change-assurance.json` is sealed into the receipt and exempt from every census, and carried a stale identity and an incomplete adapter contract | assurance/change-assurance.json | correct-requirement-no-evidence |
| FND-408 | low | `tc_028`'s total-only floor cannot catch a directory vanishing, and its declared list already names a directory (`docs/`) that does not exist | tests/governance_reconciliation.rs | missing-requirement |

## Gaps found and closed during this work

| ID | Gap | Closure |
| --- | --- | --- |
| GAP-301 | `NFR-004-AC-4` was backed only by `TC-022`, which read `evidence/corrections/`. Deleting the tree would have left the criterion asserting something about an artifact that no longer exists. The brief did not anticipate this criterion. | Retired with the correction and the claim it superseded, which went together. |
| GAP-302 | `corpus/evidence-corrections/` named the deleted correction record as its only valid fixture. No runner consumed it, so it would have become a broken manifest nothing noticed. | Deleted with the record. |
| GAP-303 | `tests/fixtures/historical-pgm01-files.sha256` locked `evidence/**` **and** three schema files, one of which (`derivation-evidence-envelope-v1`) is live. Deleting the lock silently unfroze a live schema. | Recorded; the seven `validate_governance.py` mutation probes are the surviving freeze, and `schemas/README.md` now says so. |
| GAP-304 | Four states named the compatibility census as their only home. Two of them (`unsupported`, `malformed`) turned out to be unconditional literals demonstrated by nothing at all. | Both bound to matched adapter probes; `inconclusive` bound to the live governance corpus; `suspect` declared lost. |
| GAP-305 | Three adapter refusals all exit 1, so rehoming two states onto them would have made three states mutually indistinguishable. | A test compares the recorded refusal details; probed red. |
| GAP-306 | The replacement `malformed` demonstrator was falsifiable only by accident. Filling a gap with something unfalsifiable reads identically to filling it properly. | Rebuilt on the real 99-row producer stream with one row truncated and 98 left intact, so a dropped row leaves a valid run that exits 0. Two named defects introduced, both observed red. |
| GAP-307 | The sealed `change-assurance.json` is exempt from every census, so a claim corrected in the spec, the argument and the README could still be sealed stale from there. | Re-read in full, last, against every claim corrected elsewhere. Three corrections made; see SR-032, "The sealed file, re-read last". |

## Declared residual gaps

| Gap | Why it is not closed here |
| --- | --- |
| `suspect` has no demonstrated case. | It meant "a retained record an append-only correction names". Nothing in this repository is now suspect, and inventing a stand-in would be the state collapse the gate exists to prevent. Held in `LOST_STATES` with a test that keeps it declared, so restoring it requires naming a demonstrator. Needed before this repository moves toward a stable release. |
| Six census mutation probes removed. | Each mutated the deleted census's own mapper — digest unbinding, state collapsing, outcome repair, source-identity dropping, derivation forging, source unlocking. None covered a surviving check. |
| `tc_028`'s floor is total-only, and `docs/` is declared but absent. | Pre-existing and not introduced here; this change raises that census's population rather than lowering it. A correct per-directory guard cannot be built from the same array the walk iterates, so it needs a separate `const` or `read_dir` discovery — machinery this change was not asked to add. Measured and recorded in SR-032 instead: losing the entire `spec/` directory (40 of 101 files) would leave 61 and not trip a floor of 20. |
| `engineering-assurance#20` remains open. | The v0.2.0 tag records `pending_human_acceptance`; the acceptance landed on `main` after the tag and no v0.2.1 was cut. Unchanged by this work and still recorded in `assurance/pins.json` `known_drift` and in the change-assurance `unknowns`. |
| `UNKNOWN-human-decision-absent` remains open. | Only `@kreneskyp` can record an ix-flow decision. Unchanged. |

## Deviations from the shared brief

The brief described a uniform shape across the eight repositories. Five of its
assumptions did not hold here, and each was handled on the evidence rather than
by pattern-matching a sibling:

1. **`scripts/legacy_evidence_view.py` does not exist.** The role is played by
   `scripts/pgm01_compatibility_view.py`, located by finding the caller of
   `map_pgm01_bytes`. Removed.
2. **`scripts/assurance_chain.py` has no legacy wiring.** The brief expected a
   `PROOF-legacy` media type, an `INPUTS` entry and a `derive_result` branch.
   None exists; the chain only ever named `PROOF-conformance`,
   `PROOF-quire-static-export` and `PROOF-msrv`. Nothing to remove.
3. **`AA-001` has no paragraph arguing from legacy compatibility.** It has a
   Challenges sentence asserting child evidence is retained, which is a
   different and smaller thing, amended rather than deleted.
4. **The acceptance criterion is not `FR-006-AC-4`.** Here the evidence claims
   are spread across `FR-009-AC-4`, `FR-009-AC-5`, `FR-022-AC-4` and
   `NFR-004-AC-4`, plus `PGM-01-R09` and `STD-002`.
5. **Two schemas were frozen, not four.** The sibling freeze list was not
   inherited; each of the five was grepped in this tree, and one whose filename
   contains "evidence" turned out to be a live validator.

The corrected coverage gate — *no row becomes unbacked as a side effect*, rather
than "0 unbacked rows" — was used, and the arithmetic is shown in SR-032.

## Method note

The state census was measured on the **pre-deletion** tree, per state, before
anything was removed. Had it been measured afterwards the four census-only
states would have had to be reconstructed from a deleted file. It was that
measurement, not the deletion, that surfaced GAP-304: two states had been
declared demonstrated while bound to nothing, a defect that predates this change
and that a post-hoc measurement could not have distinguished from collateral
damage.

An independent adversarial review was commissioned before merge rather than
relying on this agent's own reading. It raised eight findings; six are FIXED and
two ACCEPTED, dispositioned individually in SR-032.
