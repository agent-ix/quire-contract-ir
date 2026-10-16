---
id: SR-11853
title: "IR-822 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@edd0830f45634d804bc6adb93654bc874b98fdc9 (base 4895a92daf8a16e292a17ca528cb7711c07ac3c3); FR-020-AC-15 against tests/it/conformance.rs and crates/quire-contract-model/src/conformance.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-020
    type: references
---

## Summary

Ticket: IR-822. Planless gap-analysis scoped to the PR. Plan completion: not assessed. Read the computed matrix with `quire coverage --json` at origin/main, at pre-tag commit 79123dd and at head, and the implementing code path (`validate_successful_package_schema`, `execute_expression`).

| Commit | backed / total | unbacked_rows | FR-020-AC-15 |
| --- | --- | --- | --- |
| origin/main 4895a92 | 495 / 559 | 48 | absent |
| 79123dd (criterion, no tag) | 495 / 560 | 48 | untagged, backed false |
| edd0830 (head) | 496 / 560 | 48 | tagged, backed true |

The 48 is unchanged because `unbacked_rows` lists reference rows none of whose targets is bound, and the AC-15 row also targets TC-018, which is bound. The 48 are elsewhere (FR-052 11, kani matrix 6, FR-045 6, FR-036 5, and others); none is in FR-020.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. FR-020-AC-15 has exactly one binder, the binder asserts the criterion's response, and the mutation in SR-11852 shows it can fail. No new code lacks an owning requirement; the only unbound behaviour (a decodable but invalid document is not schema-checked) is a wording issue in the criterion, raised as SR-11850 FND-001.
