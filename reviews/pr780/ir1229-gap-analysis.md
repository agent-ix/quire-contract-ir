---
id: SR-202610310140
title: "IR-1229 PR 780 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir PR #780 at 3e02f11713b5844520470780091d660a31f4a5be; AD-005 D-2, FR-028-AC-1/3, TC-041, deny.toml, tests/it/cycle_free_model.rs"
review_set: subset
---

## Summary

Checked the changed rule against its AD decision, the computed matrix, the tests and cargo-deny behavior. Plan completion: not assessed. The ban has genuine positive and forbidden-dependency evidence, but one new test tag names a criterion that does not state the tested boundary.

## Verdict

**CONDITIONAL** — Correct the new test's FR-028-AC-1 trace claim; keep TC-041 and identify AD-005 D-2 as its provenance.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | New downstream-name refusal test overclaims FR-028-AC-1 coverage; that criterion concerns owner and TL dependencies | tests/it/cycle_free_model.rs:250 |

## Coverage

- AD-005 D-2 requires no IR model or root dependency on the codegen/runtime crates in any kind. `deny.toml` names both crates, and the existing `make ci` includes `make deny`. An isolated path-dependency probe produced two exact `error[banned]` messages and exit 2. The allowed IR graph passed `cargo deny check bans` at the PR head.
- Computed `quoin matrix --repo . --json` reports FR-028-AC-1 tagged by three TC-041 test functions, including the new `model_graph_refuses_downstream_crate_names`; FR-028-AC-3 remains tagged by the root manifest test. The latter still asserts its QSL requirement, so that binding remains valid.
- No source stub, duplicate renderer, or new unowned runtime behavior was found in the diff. The finding concerns the new trace claim, not the ban's behavior.

## Dispositions

Round 1 at `d9b818af0197c57831a73768089bcd1089ed014e`. Rechecked the sole finding against the fix diff and the complete changed test. No new finding was introduced; the latest outcome is fixed and this review is mergeable.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The new test now traces TC-041 alone and records AD-005 D-2 as provenance. FR-028-AC-1 remains bound to the existing owner/TL graph tests and no longer receives credit from the downstream-name control. |

