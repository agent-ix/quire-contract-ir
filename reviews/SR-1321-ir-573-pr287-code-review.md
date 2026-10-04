---
id: SR-1321
title: "PR #287 IR-573 first-fault test: code and Rust review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@30198026e458630a9abf3e7a2bf86452c060b55b; tests/it/checked_package_v2_model_members.rs (git diff origin/main...HEAD, base 1c222c53bdfb9ab104a40df2ea4d68ec74a340e0)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: reviews
---
# SR-1321: PR #287 IR-573 first-fault test: code and Rust review

## Summary

Ticket: IR-573. PR agent-ix/quire-contract-ir#287, head
30198026e458630a9abf3e7a2bf86452c060b55b. Method: code-review with the
rust-review lane folded in, plus a check of the test oracle's strength. The PR
changes no production code. It adds one test,
`tc_048_the_first_reader_fault_decides_when_faults_coexist`, tagged TC-048 and
FR-038-AC-110, and one helper, `raw_digest_refusal`.

The test reads nine documents. Each is read under its own SHA-256 and under a
foreign digest. Every expectation is a hand-written `CheckedPackageRefusal`
compared whole with `assert_eq!`. The raw-path expectation,
`stale_dependency`/`byte-digest-mismatch` at `/lock/model_selections/0/digest`
with no `document_pointer`, uses the same path as the AC-93 and AC-109
`number_refusal` helper. It is also what `admit_document`'s `Err(_) =>
mismatch` arm produces: a `SelectionRefusal` at the row's `digest` member, with
row index 0.

To check the oracle, I mutated production code in a throwaway copy, ran the
touched module after each change, and reverted each one:

| Mutation in `admit_document` | Result |
| --- | --- |
| `NumberOutOfRange` arm disabled, so it falls to the raw-digest path | new test fails at `{"a":1,"a":2,"n":1e400}` |
| reader pointer replaced by the constant `/a/0` | new test fails at `/n` |
| cause hard-coded to `InexactInteger` | new test fails at 400 nines then `.5`, as does the existing non-whole test |

All three mutants are killed.

## Verdict

**APPROVE.** `make fmt-check` and `make lint` pass. The touched module passes,
22 tests. The test follows the repo's `tc_048_` and `#[trace]` conventions. It
uses no mocks, has no tautology, and does not derive any oracle from the code
under test. A failure message names the document, cut to its first 60
characters. The one problem with what the test asserts is that it pins
`byte-digest-mismatch` under the raw digest. That is a spec-parity question and
is recorded once, in SR-1323 FND-003, rather than here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
