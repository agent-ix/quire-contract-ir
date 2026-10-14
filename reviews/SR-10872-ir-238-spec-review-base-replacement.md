---
id: SR-10872
title: "IR-238 PR 389 spec-review/base replacement"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md, spec/checked_package/matrix/TC-451-typed-v2-envelope-encoding.md, spec/checked_package/matrix/tests.md, spec/spec.md, spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-051", type: reviews }
  - { target: "ix://agent-ix/quire-contract-ir/TC-451", type: reviews }
---

## Summary

Replacement spec consistency review of PR #389 at frozen head `f6abcf0f1a95cae9c87e18f547b307c8d8f5d9ea`. One material inconsistency remains against current main: FR-051 and TC-451 describe the API as implemented and require checking the default-feature export inventory, while FR-019 still calls the API future and omits the now-exported function from its inventory. The earlier clean conclusion does not hold against new main.

## Verdict

**CONDITIONAL** — One medium finding requires inventory reconciliation.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-019 still describes `encode_checked_package_v2` as future and omits it from the default-feature export inventory even though this diff implements and exports it; TC-451's prescribed inventory comparison therefore fails. A reader of FR-019 will classify the shipped function as not part of the current public surface. | spec/model/functional/FR-019-rust-library-interface.md:154-158, 178; spec/checked_package/matrix/TC-451-typed-v2-envelope-encoding.md:49-55 |

## Coverage

Compared changed FR-051, TC-451, matrix summary and spec index with current-main FR-019 and the crate root export. Verified the missing inventory entry directly in current source and spec, rather than relying on the earlier stale-base test failure report. Plan completion: not assessed.
