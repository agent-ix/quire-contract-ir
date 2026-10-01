---
id: SR-646
title: "spec review (base) of PR 237 (IR-482 FR-038-AC-42 and matrix rows)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@9f8699e5736918f3dccf639148297e4d835d2750; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/tests.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---
# SR-646: spec review (base) of PR 237

## Summary

Ticket: IR-482. FR-038-AC-42 was checked against QSpec origin/main 4634f5f:
- FR-322 "Operation families, groups and constraints": an enum is `ordered_enum` when its nominal preimage is ordered and `enum` otherwise, and `enum_kind` = {`enum`, `ordered_enum`}.
- FR-141 body and AC-5: ordering an unordered enumeration, or comparing across declarations, is `ill_typed`.
- The catalog at quire-verification-contracts ead78f3.

The AC states the rule and its outcomes exactly: `ill_typed`/`operator-ineligible` at arguments/0 for an unordered enum, and at arguments/1 for two different enums, which is the `same_type` refusal. It is atomic enough for one TC and testable. It sits in the right subsystem (`spec/checked_package/`). No requirement or AC was deleted. The FR-038 matrix range (`AC-35 through AC-42`) and the TC-048 AC list are updated in `spec/checked_package/matrix/tests.md`. `quire validate` and `make spec` give the same 17 unbacked ids as base. FR-038's body delegates the operation rules to QSpec FR-322 as normative, so leaving the body text unchanged is consistent.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | TC-048 now verifies FR-038-AC-42, but the TC-048 document's Description, Test Procedure and Expected Results do not mention the enum-order cases. The matrix TC-048 summary row still says it is implemented in `tests/it/checked_package_v2_reader.rs` only, while the AC-42 tests are in `tests/it/checked_package_v2_enum_order.rs` | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:13-24 |

## Finding Detail

- FND-001: add one sentence to each TC-048 section, covering ordered and unordered enum `lt/le/gt/ge` and `eq/ne`, and two-enum ordering, with the expected refusal pointers. Name `checked_package_v2_enum_order.rs` in the matrix TC-048 status cell. The TC document already left out some earlier ACs (AC-10, AC-11, AC-17..26), so this follows an existing pattern of drift, but this PR adds to it.

## Verdict

The AC is accurate against FR-322, FR-141 and the catalog. One low documentation finding.
