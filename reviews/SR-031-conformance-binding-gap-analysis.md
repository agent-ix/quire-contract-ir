---
id: SR-031
title: "Conformance result binding gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "issue #44 IR-owned suite and producer trace metadata"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/PLAN-005
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-018
    type: references
  - target: ix://agent-ix/quoin/issues/331
    type: references
---

# SR-031: Conformance result binding gap analysis

## Summary

The IR-owned half is implemented and locally verified: every one of the 99 native corpus rows now
carries the exact non-empty verification-target list declared by its schema-validated fixture, and
the repository declares the suite. The pinned Quoin adapter still drops that field, so nonzero
binding is not claimed.

## Verdict

**CONDITIONAL.** The producer change is ready for independent exact-head review. PLAN-005 and issue
#44 remain open until Quoin issue #331 is released, pinned, and an exact-revision evidence record
demonstrates nonzero bindings.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-301 | high | Open upstream: Quoin 0.23.1's adapter discards producer `trace_ids`, so current `quoin evidence record` still binds zero. | quoin#331, issue #44 | correct-requirement-no-evidence |
| FND-302 | high | Closed locally: the manifest schema requires a non-empty unique closed Test Case set, the runner additionally requires sorted order, and empty/reordered controls fail. | FR-018-AC-1, TC-018 | missing-requirement |
| FND-303 | medium | Closed locally: results name TC-015 through TC-018 and rely on Quire's existing criterion targets instead of copying acceptance criteria into a second graph. | PGM-01-R09, SUR-001 | wrong-requirement |
| FND-304 | medium | Closed locally: complete regeneration changed only manifest/schema identities and added trace metadata; 99 semantic expectations and canonical byte files remain unchanged. | FR-018-AC-3, TC-018 | implementation-bug-despite-evidence |
| FND-305 | medium | Closed locally: the final stacked candidate passes the complete local release gate; see REV-013. | PLAN-005, issue #44 | correct-requirement-no-evidence |
| FND-306 | medium | Open process gate: independent exact-head review remains required before landing. | PLAN-005, issue #44 | correct-requirement-no-evidence |

## Verification performed

- `cargo test --test conformance`: 3/3 tests pass, including 99/99 native fixture matches and the
  manifest failure controls.
- `python3 scripts/generate_conformance_corpus.py --check`: the complete generated corpus matches
  the checked-in corpus byte-for-byte.
- `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, and strict Quire validation pass
  locally (apart from the repository's known duplicate-module warnings).
- In a temporary clone of the clean committed candidate, Quoin 0.23.1 records all 99 rows but
  reports `bound: []`, `suspect: []`, and `unmatched: []`; this independently reproduces FND-301
  without writing an evidence store into the source tree.
- The final stacked candidate passes `make release-check`; REV-013 records the gate composition and
  result.

No hosted workflow was dispatched.
