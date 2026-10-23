---
id: SR-16272
title: "IR-984 PR 528 hybrid reader gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@b56d43e6c3dbefc6d6433ced2a7e94a64f607a7a; FR-038-AC-311..317; tests/conformance_qspec/hybrid.rs; crates/quire-contract-model/src/checked_package/v2/hybrid.rs; QSpec FR-481/TC-966@da44f70b1fb93eebd3e0fddd92efecf1b477643d"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: references }
---
# SR-16272: IR-984 PR 528 hybrid reader gap analysis

## Summary

This planless diff-scoped audit reconciles FR-038-AC-311..317 with the computed matrix, tagged conformance tests and the new reader code. All seven criteria have static tags and all 16 focused tests pass, but the tagged evidence omits required exact loci and several acceptance branches.

## Verdict

FAIL: FR-038-AC-317's exact-locus obligation is not backed by its tagged tests; further evidence gaps remain.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Tagged hybrid refusal tests do not assert exact RFC 6901 loci; the strict-wire table accepts any path under the node | FR-038-AC-317; tests/conformance_qspec/hybrid.rs:93-100 |
| FND-002 | medium | Three positive claim tests pass on `Incomplete`, so they do not prove admission | FR-038-AC-313, FR-038-AC-315, FR-038-AC-317; tests/conformance_qspec/hybrid.rs:425-468 |
| FND-003 | medium | No tagged test exercises called-function parameter/result type checks, despite AC-314 requiring them | FR-038-AC-314; crates/quire-contract-model/src/checked_package/v2/hybrid.rs:189-209 |
| FND-004 | medium | No tagged test combines hybrid and protocol defects or tests the wrong interval/profile before fairness, despite explicit precedence clauses | FR-038-AC-315, FR-038-AC-316; tests/conformance_qspec/hybrid.rs |

## Finding Detail

- FND-001: `qspec_hybrid_schema_defects_refuse_before_identity` checks `MalformedWire` and only `starts_with("/semantic_graph/nodes/{position}")`. The other negative tests compare code or code/cause but contain no `refusal.path` assertion. AC-317 and TC-048 demand the exact published code, cause and locus for strict-wire and rebuilt-key cases. A refusal at the wrong field within the node would pass these tests.
- FND-002: `qspec_hybrid_braking_model_and_claim_admit_from_published_nodes`, `qspec_hybrid_restored_model_and_claims_admit_from_published_nodes` and `qspec_hybrid_timed_open_upper_admits_under_timed_profile` panic only for `Refused`; `CheckedPackageV2ReadResult` also has `Incomplete`, which passes those tests. Assert `Admitted` explicitly.
- FND-003: The QSpec rule includes called functions only when every parameter and the result are Boolean or numeric. `reads_allowed` implements that check, but no hybrid conformance mutation supplies an ineligible function parameter or result. The static tag on AC-314 therefore does not demonstrate this branch.
- FND-004: AC-315 requires a wrong interval form to win over a fairness defect, and AC-316 requires a hybrid defect to win over a protocol defect while a valid hybrid claim yields the protocol refusal. The 16 hybrid tests contain no protocol mutation and no combined-defect package; static tagging does not establish those outcomes.

## Coverage

- Reconciliation: `quire matrix --scope . --format tsv` at exact PR head; seven of seven new criteria compute `tagged`. No run-evidence store assessed.
- Plan completion: not assessed
- Focused execution: `QUIRE_SPECIFICATION_DIR=/home/peter/dev/worktrees/ir984-qspec414 cargo test --locked --test conformance_qspec qspec_hybrid -- --nocapture`: 16 passed, 0 failed.
- Reverse gap: no new hybrid source behavior lacks an owning FR-038-AC-311..317 criterion; the defects above concern incomplete verification of named behavior.

## Dispositions

At PR #528 head `48fb6f96e1b776f7a72835e498a6efc3f2665aea`, same-reviewer round 1 inspected the test diff and reran the focused QSpec hybrid tests: 20 passed, 0 failed.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The strict-wire table and rebuilt-key refusals assert exact code, cause, and path; other changed negative cases also assert exact loci. |
| FND-002 | fixed | Each of the three named positive tests now requires `CheckedPackageV2ReadResult::Admitted(_)`. |
| FND-003 | fixed | The added call test checks ineligible function result and parameter types with exact refusal code, cause, and locus. |
| FND-004 | fixed | New combined-defect tests check hybrid precedence over protocol and wrong interval/profile precedence over fairness with exact refusals. |

## Final head confirmation

PR #528 head `d6292182a9a00f2f5a5c55fe26378fcb66f90b6b` adds a combined capture type and fairness defect case. The capture refusal wins with exact code, cause, and locus, as FR-038-AC-316 requires. All 20 focused QSpec hybrid tests pass. The round-1 dispositions remain fixed; no new gap finding in this delta. Gap analysis: PASS at this head. The author retains the final aggregate gate.

## Gate-fix head confirmation

At PR #528 head `a3522ad7528d64f7f4d0c4f41331be462c463165`, TC-226's exact work-boundary test, TC-048's nonhybrid timed-bound refusal, TC-058's public-item inventory test and the 21 QSpec hybrid tests pass. The new mixed-package test checks the nonhybrid timed-null refusal with exact code, cause and locus. FR-019-AC-5 is tagged in the computed matrix. No new gap finding in the six-file delta; PASS at this head. The final aggregate gate remains author-owned.
