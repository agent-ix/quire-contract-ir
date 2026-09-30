---
id: SR-592
title: "PR #204 base checklist and coverage review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/ (15 files, git diff origin/main...HEAD)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-028
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-030
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-031
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: reviews
---
# SR-592: PR #204 base checklist and coverage review

## Summary

This review covers the base checklist: ID formats, AC and TC quality, and
test-matrix coverage and honesty for the changed requirements.

Ticket: IR-313.

Measured, with logs in the reviewer scratchpad:

- `make spec` exits 2. In both runs the only
  failures are the MP-001 and MP-002 frontmatter errors from `quire validate`.
- `quire coverage --scope . --strict` exits 1 at both commits. Unbacked rows
  go from 37 to 39, which is 224/267 to 224/270 rows. The two new unbacked
  rows are FR-019-AC-5 and TC-058.
- `scripts/validate_matrix_status.py` exits 0 at the PR head.

Checked against the AC texts: FR-019-AC-5, FR-028-AC-1 to AC-5,
FR-030-AC-4 and AC-5, FR-031-AC-5, FR-039-AC-1 to AC-4, FR-037-AC-6, TC-058,
TC-055, TC-223, and the matrix rows FR-019, FR-028, FR-030, FR-031, FR-039,
TC-041, TC-042, TC-223, TC-058, StR-001 and StR-003.

## Verdict

**PASS with findings**. The matrix rows the PR moved to planned (FR-019,
FR-028, TC-041, StR-003) describe the code honestly and are not narrowed to
match it. TC-058 is well formed and is indexed. TC-020's AD-001 markers
(`type: ArchitectureDescription`, `## System Boundary`, `## Risks`,
`owner: kreneskyp`) are still present. The findings concern test tags that
now claim ACs their tests do not verify, and one AC that is weaker than its
statement.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-028-AC-2 now requires model reachability through quire_contract_model with no root re-export. The test still tagged FR-028-AC-2 asserts the opposite (`pub use quire_contract_model::*` present in src/lib.rs). quire coverage therefore reports FR-028-AC-2 backed while the matrix says planned. | tests/it/cycle_free_model.rs:211-247 |
| FND-002 | medium | tc_223_every_kani_outcome_kind_maps_to_its_one_fr331_result carries TC-223 and FR-031-AC-5 tags but tests the retired KaniProviderResult map, not qsl_replay::TerminalValue. Coverage reports FR-031-AC-5 and TC-223 backed. Because of the TC-223 tag, the FR-030 matrix row also reads backed, although FR-030-AC-4 and FR-030-AC-5 are backed:false at AC level. | tests/it/kani_shared.rs:250-252 |
| FND-003 | medium | FR-030 says no public constructor builds an `unavailable` outcome with another cause and that such a request returns a typed refusal. KaniOutcome has all-pub fields, and non_success accepts any kind and code, so a struct literal bypasses the rule. AC-5 checks only "yields no unavailable outcome" and does not check the typed refusal. | spec/contract/FR-030-bounded-kani-domain-and-outcomes.md:52 |

## Dispositions

Round 1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | resolved |
| FND-002 | accepted-no-change | The stale tag on tests/it/kani_shared.rs:250 is code, and this PR is spec-only. The FR-031 matrix row, the TC-223 row and status, and FR-031's Status now each state that the tag is stale and does not back FR-031-AC-5 or TC-223. `quire coverage` still counts the row as backed until the TC-223 implementation retags the test. |
| FND-003 | fixed | resolved |
