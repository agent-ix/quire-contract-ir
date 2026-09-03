---
id: REV-009
title: "Corpus depth and native-runner remediation gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "issues #30, #31, and #36 remediation candidate"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-018
    type: reviews
---

# REV-009: Corpus depth and native-runner remediation gap analysis

## Summary

The exact wire-depth cliff is now visible in the public diagnostic projection and the published
corpus: the 576-level shape reaches ordinary decoding at `document`, while 577 levels are refused
by the pre-decode guard at `document.nesting`. Their expectation digests differ. The standalone
Make target delegates directly to the domain runner and no longer contains a second census.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-901 | high | **FIXED** — the two depth expectations are distinct and the coverage observer requires the corresponding code/path pair. | FR-018; STD-001; `package-wire-depth-*`; TC-018 | correct-requirement-no-evidence |
| FND-902 | high | **FIXED** — the native runner is the only corpus verdict source; Python optimization cannot delete the gate. | Makefile `corpus`; issue #31 | correct-requirement-no-evidence |
| FND-903 | medium | **FIXED** — mismatch rows and operational errors retain the runner's own channels instead of becoming a Python assertion. | Makefile `corpus`; issue #36 | correct-requirement-no-evidence |
| FND-904 | medium | **RETAINED** — GNU Make maps failed recipes to its own failure exit, so consumers needing the runner's 0/1/2 protocol must invoke `quire-contract-conformance` directly. The output class is no longer hidden. | corpus README; Makefile | wrong-requirement |
| FND-905 | medium | **OPEN PROCESS GATE** — independent exact-head review is required before landing. | issues #30, #31, #36 | correct-requirement-no-evidence |

## Verification

- `cargo test --test conformance`: 3/3 passing, including 99/99 published fixtures.
- `python3 scripts/generate_conformance_corpus.py --check`: byte-for-byte pass.
- `PYTHONOPTIMIZE=1 make corpus`: exit 0 over the complete native runner result; no Python assertion
  participates in the verdict.
- In an isolated scratch clone and Cargo target, replacing `json_nesting_exceeds` with unconditional
  `false` makes the runner exit 2 because `package-wire-depth-over` no longer observes its declared
  `boundary:wire.depth.over_maximum` token.
- Full clean-head native, specification, MSRV, and shared-assurance verification remains required
  after the candidate commit.

No hosted workflow was dispatched or changed.
