---
id: SR-1322
title: "PR #287 IR-573 / IR-542 AC-109..AC-111 gap analysis"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@30198026e458630a9abf3e7a2bf86452c060b55b; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (AC-109, AC-110, AC-111), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, spec/tests.md, tests/it/checked_package_v2_model_members.rs, tests/it/checked_package_v2_canonical_encoding.rs (git diff origin/main...HEAD, base 1c222c53bdfb9ab104a40df2ea4d68ec74a340e0)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: reviews
---
# SR-1322: PR #287 IR-573 / IR-542 AC-109..AC-111 gap analysis

## Summary

Ticket: IR-573. The PR body also says it closes IR-542. PR
agent-ix/quire-contract-ir#287, head 30198026e458630a9abf3e7a2bf86452c060b55b.
Method: gap-analysis, run manually, criterion to test to code, over the three
criteria the PR flips to implemented.

| Criterion clause | Backing test(s) |
| --- | --- |
| AC-109, inexact-number set, both digests, tie texts, `serde_json`-free | `tc_048_a_model_document_number_with_no_exact_rfc_8785_spelling_refuses_inexact_number`, `..._whose_value_its_encoding_keeps_is_digested`, and the AC-109 test for `1e-400`/`-1e-400` (line 899) |
| AC-110, `inexact-integer` set including `1e400`/`-1e400` mapping and pointer | `tc_048_a_number_past_the_double_range_refuses_inexact_integer_with_its_pointer`, the AC-93 tests |
| AC-110, document order between reader-decided numbers | `tc_048_the_first_inexact_number_in_document_order_is_named_with_its_own_cause` |
| AC-110, first-fault rule (all seven documents of the new clause) | `tc_048_the_first_reader_fault_decides_when_faults_coexist` (new) |
| AC-110, `1e-400`/`-1e-400` `inexact-number` | the AC-109 test at line 892 |
| AC-111 | `tests/it/checked_package_v2_canonical_encoding.rs:695` |

Every clause has a test that could fail. The production mapping is in
`admit_document`, `crates/quire-contract-model/src/checked_package/v2/model_members.rs:1063-1094`,
and this PR leaves it unchanged. IR-542, read as data, asks for a refusal of a
number whose text is not the exact spelling of its double, with
`noncanonical_wire`, a `document_pointer` and cause `inexact-number`, spec
first and then code. With AC-109 through AC-111 implemented, nothing in that
request is left open. The FR-038 row stays 🚧 for its other open items, and its
AC-107 CI-wiring caveat is unchanged and accurate.

`make test` passes on this head. So do `quire validate` (one grammar finding,
FR-014) and `quire coverage --strict` (23 unbacked rows, the same as base).

## Verdict

**APPROVE.** No acceptance-criterion gap, stub or coverage inflation. The flip
to implemented has a test behind every clause. Spec-parity wording issues are
recorded in SR-1323.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
