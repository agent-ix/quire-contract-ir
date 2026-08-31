---
id: REV-007
title: "quire-contract-ir v0.1 Wave 1 epic closure review"
type: Review
status: complete
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/11
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/PLAN-002
    type: reviews
---
# REV-007: quire-contract-ir v0.1 Wave 1 epic closure review

## Scope and decision boundary

This composite review covers PLAN-002, its five dependency-ordered child
issues, the merged semantic implementation, schema and corpus outputs, retained
child reviews, and the Wave 1-to-Wave 4 handoff. It is a producer closure audit,
not fresh independent approval. REV-003 through REV-006 and their associated
specification reviews retain the independent findings that shaped each child.

The authoritative handoff assigns human release decisions, source tags, and
checksums to PGM-02 in Wave 4 after all eight repository epics complete. This
review can close the Wave 1 implementation substrate. It cannot select or tag a
source-release candidate, publish a crate, or turn an inconclusive review into
an approval.

## Findings

| ID | Severity | Finding | Disposition |
|---|---|---|---|
| EPC-001 | high | PLAN-002 and issue #11's original completion prose pulled the human source decision into Wave 1 even though the program handoff assigns all human release decisions and source tags to PGM-02 Wave 4. | fixed in PLAN-002 and AA-001: Wave 1 closes the implementation substrate, while the exact source decision remains open and explicitly owned by Wave 4. The issue closure note must preserve the same boundary. |
| EPC-002 | medium | AA-001 still said the implementation and corpus were absent after all five child issues had merged. | fixed: AA-001 identifies the merged Wave 1 implementation baseline, records the child outputs as present, and replaces the obsolete challenge with the actual Wave 4 evidence challenge. |
| EPC-003 | high | A completed epic could be misread as source-release approval despite the latest child review being inconclusive and cross-platform/downstream execution being absent. | fixed: PLAN-002, AA-001, this review, and the closure note state that no source candidate, tag, publication, qualification, or accreditation is approved. |
| EPC-004 | medium | The issue #10 evidence names the pre-squash subject rather than the merge commit. | resolved by verification rather than relabeling: `scripts/verify_evidence.py` proves the complete non-evidence tree of subject `097f24a` equals post-squash `main`, verifies 695/695 input bytes and 4/4 retained outputs, and enforces COR-001. Immutable evidence is not rewritten. |
| EPC-005 | medium | Quire reports 85/94 backed and skips its configured status classifier because the archetype requires a different header; treating the exit code as complete traceability would be false. | contained: the separate fail-closed matrix census resolves every completed row to executable completed tests. The Quire limitation remains open and the 85/94 result is not restated as 100%. |
| EPC-006 | medium | Automatic CI, cross-platform comparison, and independent downstream corpus execution remain absent. | carried to Wave 4: CI stays manual-dispatch-only under the operator's instruction, no workflow is dispatched for this closure, and these missing observations are neither hidden nor inferred as passing. |
| EPC-007 | low | The first closure gate found TC-020 coupled AA-001's open release claim to the prose fragment “remains open” instead of its structured `top_claim.status` and program phase. | fixed: TC-020 scopes the `status: open` assertion to the top-claim frontmatter and separately requires the PGM-02/Wave 4 boundary, so editorial wording no longer controls the assurance verdict. |

## Child and dependency audit

| Child | Repository issue | Project state | Retained review/evidence | Result |
|---|---|---|---|---|
| TASK-005 | #5 | closed / Done | REV-002 and corrected PGM-01 evidence | complete |
| TASK-006 | #6 | closed / Done | REV-003 and candidate evidence | complete |
| TASK-007 | #8 | closed / Done | REV-004 and candidate evidence | complete |
| TASK-008 | #9 | closed / Done | REV-005 and candidate evidence | complete |
| TASK-009 | #10 | closed / Done | REV-006 and `pgm-01-097f24a` | complete |

The dependency order is preserved in Git history and in PLAN-002. No child was
left open, and issue #10's project item was explicitly advanced from In review
to Done after its authorized admin squash merge.

## Post-merge gap analysis

| Required Wave 1 outcome | Evidence | Result |
|---|---|---|
| Language-independent semantic substrate | FR-011 through FR-020, validated Rust model, closed wire forms | satisfied |
| Deterministic identities and canonicalization | TC-015 through TC-017 and frozen canonical bytes/digests | satisfied on the tested Linux environment |
| Reusable schema and conformance corpus | two Draft 7 schemas, 99 fixtures, JSON-lines runner, TC-018 | satisfied |
| Complete executable backing for implemented rows | 34 Rust tests, 13 Python tests, local completed-row census | satisfied |
| Retained candidate evidence and correction enforcement | `pgm-01-097f24a`, 4/4 outputs, 695/695 inputs, COR-001 | satisfied |
| Review and reverse-gap disposition | child spec/implementation reviews plus EPC-001 through EPC-007 | satisfied for implementation closure; independent release approval remains open |
| No automatic CI or publication | manual-only workflow, no closure dispatch, `publish = false` | satisfied |
| Human source decision and tag | PGM-02 Wave 4 handoff | deliberately not performed in Wave 1 |

## Verification snapshot

Post-merge `main` at
`5c49ebfd1c87415f74420ad047392bd03b1bd202` passed the isolated full local
`make ci` lane: formatting, Clippy with warnings denied, 13/13 governance
cases, 7/7 governance mutations, 13 Python tests, 34 Rust tests with zero
ignored, doc tests, the ordered 99-row corpus, byte-for-byte corpus
reproduction, license policy, and unsafe-code audit. `make spec` reports 67/67
grammar-clean documents and the local matrix census passes. Rust 1.75
`cargo check --locked --all-targets`, `git diff --check`, and the unique current
evidence verifier pass. The only visible `main` Actions runs predate this Wave 1
candidate; no workflow was dispatched.

## Conclusion

No blocking Wave 1 implementation gap remains. PLAN-002 may be Done and epic
#11 may close as the completed Agent A substrate. AA-001 correctly remains
open for the PGM-02 Wave 4 source-release decision. This conclusion is not a
source-tag recommendation and does not claim fresh independent approval,
cross-platform execution, downstream execution, or publication authority.
