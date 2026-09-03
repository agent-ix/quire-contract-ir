---
id: REV-012
title: "Local release-gate composition preimplementation review"
type: SpecReview
analysis: code-review
scope: "issue #35 local composite gates after shared-assurance migration"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/NFR-002
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/NFR-004
    type: reviews
---

# REV-012: Local release-gate composition preimplementation review

## Summary

Issue #35 predates the shared-assurance migration. Two findings remain applicable: the local `ci`
composite omits the specification gate and the declared Rust 1.75 check. Its request for
`evidence-verify` is obsolete because the migration contract deleted that repository-local generic
verifier and assigned retention and receipt verification to Quoin. Hosted workflows remain
manual-dispatch only by current operator direction.

## Verdict

**PASS to implement the narrow local composition change.** Add `spec` and `msrv` to `ci`, and make
`release-check` an alias of that complete local gate. Do not recreate `evidence-verify`, inspect the
Makefile from a test, or change/dispatch a hosted workflow.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-1201 | high | `make ci` can return success without running strict Quire validation/coverage or the matrix-status census. | issue #35; NFR-004-AC-5 | correct-requirement-no-evidence |
| FND-1202 | high | `make ci` can return success without checking all targets against exact Rust 1.75, despite NFR-002-AC-2 requiring that check before a source decision. | issue #35; NFR-002-AC-2 | correct-requirement-no-evidence |
| FND-1203 | medium | Reintroducing `evidence-verify` would violate the reviewed migration: Quoin owns retained-record audit and receipt verification. | migration contract; FR-022 | wrong-requirement |
| FND-1204 | medium | A test that treats the Make prerequisite list as a trust root would duplicate orchestration policy and still be bypassable by Make control features. The residual remains issue #46. | issue #35; issue #46; migration contract step 7 | wrong-requirement |
| FND-1205 | medium | Hosted workflow expansion is intentionally deferred while CI auto-runs are off; no workflow change or dispatch is authorized. | issue #13; operator direction | correct-requirement-no-evidence |

