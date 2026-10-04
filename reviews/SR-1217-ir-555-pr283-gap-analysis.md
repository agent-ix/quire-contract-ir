---
id: SR-1217
title: "PR #283 IR-555 FR-038-AC-109..111 gap analysis"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@de9bf5c8de38d2b53fbf309e7ab3c7b2d0db3c7e; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, spec/tests.md, tests/it/checked_package_v2_model_members.rs, crates/quire-contract-model/src/checked_package/v2/model_members.rs (git diff origin/main...HEAD)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: reviews
---
# SR-1217: PR #283 IR-555 FR-038-AC-109..111 gap analysis

## Summary

Ticket: IR-555 (and IR-542). PR agent-ix/quire-contract-ir#283, head
de9bf5c8de38d2b53fbf309e7ab3c7b2d0db3c7e. This is a planless gap analysis
(plan completion not assessed) of FR-038-AC-109, AC-110 and AC-111, the FR-038
inexact-number note, and the status rows in the FR-038 matrix row, TC-048 and
`spec/tests.md`.

What I examined:

- FR-038-AC-109: the underflow `1e-400` was already in the AC-109 test on main.
  The new `tc_048_a_number_below_the_double_range_refuses_inexact_number_on_its_text`
  adds `-1e-400` under both digests. Hand-written expectations. Holds.
- FR-038-AC-110: the `1e400` and `-1e400` clause is covered by
  `tc_048_a_number_past_the_double_range_refuses_inexact_integer_with_its_pointer`
  under the document's own digest and another digest, in an array at `/a/1` and
  at top level (empty pointer). It also covers `1e309`, which is beyond the AC
  text. The expectations are hand-written `number_refusal(...)` values. A
  mapping to `byte-digest-mismatch` or a root pointer would fail them, but a
  mapping hard-coded to `inexact-integer` would not (SR-1216 FND-002). The
  clause "an inexact number at `/b` and then a whole number past 2^53 at `/a/0`
  names `/b`" is verified only for finite whole numbers. With `1e400`, which
  AC-110 itself lists as a whole number past 2^53, it fails (FND-001).
- FR-038-AC-111: unchanged by this PR. Its tests in
  `tests/it/checked_package_v2_canonical_encoding.rs` are untouched.
- Spec gates on the head: `quire validate` passes, with 1 grammar finding
  (FR-014, ac:vague-response, as at base). `quire coverage --scope . --strict`
  reports 23 unbacked rows and 0 contradicted statuses, the known baseline.

The bindings `tc_048_a_number_past_the_double_range_refuses_inexact_integer_with_its_pointer`
to FR-038-AC-110 and
`tc_048_a_number_below_the_double_range_refuses_inexact_number_on_its_text`
to FR-038-AC-109 are correct.

## Verdict

**REQUEST CHANGES.**

AC-110 is flipped from partial to implemented in four places: the FR-038
matrix row, the TC-048 "Inexact numbers" section, `spec/tests.md` and the FR-038
note. The AC's document-order clause does not hold when the second number is
out of range, and no test exercises that case. The flip is not honest. Keep
AC-110 partial, name the failing clause, and link a follow-up ticket.

The right fix is upstream or in the spec, never a pre-scan:

- either `quire-canonical` keeps reading past an out-of-range number and
  exposes it as a node with its text, so IR's existing `first_inexact_number`
  walker decides in document order;
- or the owner amends FR-038-AC-110 (and raises it against QSpec FR-272's
  "first such number in document order") to say an out-of-range number is
  named wherever it sits.

That choice belongs to the owner.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038-AC-110 is marked implemented in the FR-038 matrix row, TC-048 and `spec/tests.md`, but its clause "a document holding an inexact number at `/b` and then a whole number past 2^53 at `/a/0` names `/b` with `inexact-number`, the first in document order" fails when that whole number is `1e400`. I reproduced it: the document names `/a/0` with `inexact-integer`. TC-048's procedure ("put an inexact number and a whole number past 2^53 in one document in both orders and check the first in document order is named") would fail with `1e400`, and no test covers the case. AC-110 should stay partial, naming this clause and a linked follow-up ticket, and the case should be recorded as an expected gap. | spec/checked_package/matrix/tests.md:16; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:656-661; spec/tests.md:15; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2308 |
| FND-002 | low | The FR-038 note records tracker state, "QSL-219 (reopened, Specced)", in spec prose, and it is already false: QSL-219 reads Done in Linear. The note should state the facts only: quire-canonical #9 merged as 5dc4e12, `read` returns `NumberOutOfRange` with pointer and lexeme, and IR maps it. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:718-721 |
| FND-003 | low | The FR-038 note says "A number past the double range (`1e400`, `-1e400`) denotes a whole value past 2^53, so it is an `inexact-integer`". That is not true of every such number. A non-whole literal past the double range (400 nines then `.5`) is not whole, and the code, following QSpec FR-272, refuses it `inexact-number`. The note should say the cause follows the number's text, as the code does. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:686-688 |

## Dispositions

Round 1 was reviewed at 3b3cd146dbf7d2df988cd13c9a2e3daf248e7ef0. Validate
passes with 1 grammar finding (FR-014). `quire coverage --strict` reports 23
unbacked rows and 0 contradicted statuses. The FR-038-AC-110 row text is
unchanged and carries no status.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 3b3cd14. AC-110 is partial again, and the status reads the same in all four places. The FR-038 matrix row says AC-109 and AC-111 are implemented and AC-110 is partial: its mapping is implemented (IR-555), but the "first in document order" clause is unmet when the out-of-range number follows an inexact one (IR-573). TC-048 and `spec/tests.md` say the same, and the FR-038 note records the unmet clause. IR-573 appears in the matrix notes only. |
| FND-002 | fixed | 3b3cd14. "differ: the FR-056 amendment is recorded under QSL-219, and `quire-canonical` #9 (5dc4e12...) is merged". No tracker state remains. |
| FND-003 | fixed | 3b3cd14. The note now says `admit_document` maps it "with the cause `inexact-integer` when the text denotes a whole value past 2^53 and `inexact-number` otherwise (a literal such as 400 nines followed by `.5`)". |
