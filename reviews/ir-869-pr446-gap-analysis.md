---
id: SR-12401
title: "IR-869 gap-analysis of IR PR #446"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@72d6520eae2eb0f62b46162a70fef4cce93abcc4; .github/workflows/ci.yml, spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

# IR-869 gap-analysis of IR PR #446

## Summary

Ticket IR-869. Independent `quire coverage --scope . --strict` at base 07b14f9 and reviewed head 72d6520 reports 527/584 backed at both revisions and zero contradicted statuses. Unbacked rows increase from 39 to 40 because AC-195 moves from an untagged Test criterion to an Analysis criterion verified by the `make lint` gate, with no source symbol. The head computed matrix records AC-195 as `method-without-symbol`, method `Analysis`, and no test binder. No Rust test gains an AC-195 claim. The broad hand-written FR-038 matrix row excludes AC-195 after the fix; its dedicated lint row names the gate.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4756 | examined criterion | Analysis (`make lint`) |
| spec/checked_package/matrix/tests.md:21-22 | examined mapping | TC-048 range excludes AC-195; dedicated lint row names it |
| .github/workflows/ci.yml:26 | examined evidence path | `run: make lint` |
| Makefile:53-57 | examined lane | workspace and model-consumer Clippy |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No new gap or inflated test claim in this diff (placeholder) | - |

## Verdict

PASS for the changed evidence chain. The 40 unbacked rows remain visible in strict coverage; the extra row reflects the non-Rust lint lane rather than a new implementation omission.

## Coverage

Plan completion: not assessed. Semantic review: not opted in. Computed matrix and source-to-gate mapping were inspected for the changed AC-195 obligation. Repository-wide strict coverage exits 1 at both revisions.
