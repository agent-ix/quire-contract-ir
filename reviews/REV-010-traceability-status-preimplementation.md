---
id: REV-010
title: "Criterion binding and matrix-status preimplementation review"
type: SpecReview
analysis: code-review
scope: "issues #33 and #34; stakeholder criterion bindings; governance status census"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/NFR-004
    type: reviews
---

# REV-010: Criterion binding and matrix-status preimplementation review

## Summary

Issue #33 is a static-fact ownership gap: seven stakeholder criteria cited real test cases but no
test symbol carried the criterion identities Quire binds. Issue #34 combines a true gap — the status
script had no owning criterion and its real `main` failure path was untested — with a proposed test-
execution parser that would duplicate Quire's graph and mistake declaration for execution.

## Verdict

**PASS to implement the narrow split.** Existing executable semantic tests should carry only the
stakeholder criteria they actually verify. NFR-004 should own the completed-row/status-census
contract. The script should expose a temporary-root seam so its complete path is tested without
mocking its decision function. Quire remains the sole criterion-to-symbol binder; the normal Rust
test command remains the execution owner.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-1001 | high | Seven stakeholder validation criteria were unbound even though all three stakeholder matrix rows were marked complete. | issue #33; StR-001..003 | correct-requirement-no-evidence |
| FND-1002 | medium | The status validator and its tests cited NFR-004-AC-3, which governs plan dependency state rather than matrix status. | issue #34; NFR-004 | wrong-requirement |
| FND-1003 | high | Replacing `main` with unconditional success left every focused test green because tests called only `validate_documents`. | issue #34; `tests/test_matrix_status.py` | correct-requirement-no-evidence |
| FND-1004 | medium | Parsing a separate execution report here would create another trace graph. This script can prove declared status consistency; Quire proves bindings and Cargo/unittest execute tests. | migration contract; PGM-01 | wrong-requirement |

No hosted CI or generic evidence mechanism is in scope.
