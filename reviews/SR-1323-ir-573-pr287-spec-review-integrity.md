---
id: SR-1323
title: "PR #287 IR-573 first-fault amendment of FR-038-AC-110: spec review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@30198026e458630a9abf3e7a2bf86452c060b55b; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, spec/tests.md (git diff origin/main...HEAD, base 1c222c53bdfb9ab104a40df2ea4d68ec74a340e0)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: reviews
---
# SR-1323: PR #287 IR-573 first-fault amendment of FR-038-AC-110: spec review

## Summary

Ticket: IR-573 (the PR also closes IR-542). PR agent-ix/quire-contract-ir#287,
head 30198026e458630a9abf3e7a2bf86452c060b55b. Method: spec-review, integrity
sub-analysis, with the QSL source checked read-only.

The PR amends FR-038-AC-110 and the FR-038 note on number spelling to QSL's
merged first-fault rule. It flips AC-109 through AC-111 to implemented in the
FR-038 matrix row, TC-048 and `spec/tests.md`, and removes the IR-573 gap notes
and the QSL-219 "the two differ" paragraph.

I checked the QSL source myself. quire-spec-language 9d2145443013e4a55f14697058a9ed0547265fe6
(QSL #625) is an ancestor of origin/main. Merged FR-056, lines 100-115 and
150-153 of `spec/functional/FR-056-admit-domain-package-model-declarations.md`,
says:

- "`[{"a":1,"a":2},1e400]` is digested raw, because the repeated name is the
  reader's first refusal."
- "When the bytes carry several reader faults (a number with no finite double, a
  repeated member name the reader detects when the object closes, truncation),
  the refusal is the first one `quire-canonical`'s read returns.
  `{"a":1,"a":2,"n":1e400}` refuses `noncanonical_wire`/`inexact-integer` at
  `/n`, and `[1e400` refuses it at `/0`, while `{"a":1,"a":2,"n":1e-400}` has a
  repeated name the reader refuses first, so it is digested raw. A number with
  no finite double is also named ahead of an earlier inexact number."
- "the RFC 6901 pointer of the first such number in document order, except that
  a number with no finite double is named when the reader reaches it, before
  any tree exists, so it can be named ahead of an earlier inexact number."

I also read quire-canonical 5dc4e12db63b8fae647825b4e4b78df9b6dcadfe
`src/read.rs`. The byte limit is checked first. Then the whole input is checked
as UTF-8 (`core::str::from_utf8`) before any byte is parsed. After that a
single pass reads in document order. `number()` returns `NumberOutOfRange` as
soon as a literal parses to a non-finite `f64`. `close()` calls
`check_unique_names`, so a repeated name is raised only when its object closes.
Truncation and other malformation are raised at their offset, and the pass
stops at the first fault.

Where each document and outcome in the amended AC-110 clause comes from:

| Document | Outcome | Source |
| --- | --- | --- |
| `{"a":1,"a":2,"n":1e400}` | `/n`, `inexact-integer` | QSL text |
| `[1e400` | `/0` | QSL text |
| `[{"a":1,"a":2},1e400]` | raw-digest path | QSL text |
| `{"a":1,"a":2,"n":1e-400}` | raw-digest path | QSL text |
| `{"b":0.1000000000000000000001,"a":[1e400]}`, `{"b":9007199254740993,"a":[1e400]}` | `/a/0`, `inexact-integer` | QSL's "named ahead of an earlier inexact number", made concrete with IR-573's two reproduced documents |
| `1e-400`, `-1e-400` | `inexact-number` | QSL AC-2 and the existing IR rule |

The test's own additions (`{"n":1e400,"a":1,"a":2}`, `[-1e400]`, and 400 nines
then `.5` giving `inexact-number`) are not in the AC text. Each follows from
QSL's rule and from the FR-038 note that predates this PR, so none of them adds
a requirement.

## Verdict

**APPROVE WITH CHANGES** (three medium findings and one low).

The amendment matches QSL's merged rule, and every outcome it names is QSL's or
follows directly from it. The stale wording is gone: grep finds no IR-573 gap
text, no "QSL-219", no "the two differ" and no "partial" for AC-110 under
`spec/`. The FR-038 row's AC-107 CI caveat is unchanged and still true: no
workflow runs `make conformance-qspec`, and the Makefile says "not in ci".
`quire validate` passes with one grammar finding (FR-014), and
`quire coverage --strict` reports 23 unbacked rows on both base and head.

What is left is wording. AC-110 still carries an unqualified "the first in
document order" clause that its new clause overrides. The note claims more
document-order behaviour than the reader has. And one raw-digest outcome is
pinned that QSL's text words differently.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-110 still says "a document holding an inexact number at `/b` and then a whole number past 2^53 at `/a/0` names `/b` with `inexact-number`, the first in document order". The same AC lists `1e400` among the whole numbers past 2^53, and its new clause says `{"b":0.1000000000000000000001,"a":[1e400]}` names `/a/0`. Read alone, the AC contradicts itself. The only reconciliation is a sentence in the FR prose ("among the numbers this reader decides on their text after a read that succeeds"). QSL's own sentence carries an explicit "except that a number with no finite double is named when the reader reaches it". Fix: qualify the older clause in the AC, for example "a whole number past 2^53 that has a finite double", or "except as the first-fault clause below says". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2337 |
| FND-002 | medium | The amended FR-038 note says "The read goes in document order: ... truncation or any other malformation at the byte where it occurs, and the read stops at the first". This is false for invalid UTF-8. quire-canonical 5dc4e12 `read` validates the whole input with `core::str::from_utf8` before it parses anything. I reproduced it in a throwaway probe: `[1e400,"<0xFF>"]` refuses `stale_dependency`/`byte-digest-mismatch` with no `document_pointer`, although the number comes first. A lone surrogate escape after the number (`[1e400,"\uD800"]`) does refuse at `/0`, so the problem is limited to UTF-8. QSL's text says only "the first one quire-canonical's read returns", which is right; IR's added explanation is not. Fix: drop or qualify the document-order explanation (UTF-8 validity is checked over the whole input first), or defer to `read`'s order as QSL does. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:705-710 |
| FND-003 | medium | The AC-110 clause says `[{"a":1,"a":2},1e400]` and `{"a":1,"a":2,"n":1e-400}` "refuse `stale_dependency`/`byte-digest-mismatch` at the row's `digest`". The new test asserts this under the bytes' own raw SHA-256 (`sha256_hex(&bytes)`) as well as under another digest. The note says "the two readers agree on these documents too". QSL FR-056 (lines 93-98) says unparseable bytes are digested raw and refuse `invalid_model_binding`/`wrong-model-selection` "when the selected digest happens to equal their raw digest". IR's `admit_document` maps every non-limit read error to `byte-digest-mismatch` without computing a raw digest, which is older behaviour. This PR is the first to pin that outcome under the raw digest, and it presents it as QSL's rule. Fix: limit the clause and the test's raw-path cases to a digest other than the raw one, or record the raw-digest divergence from QSL FR-056 and ticket it. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2337, 745-750; tests/it/checked_package_v2_model_members.rs:791-800, 856-860 |
| FND-004 | low | Two wording slips in the note. `[1e400` names `/0` is grouped under "a repeated name in the object that still holds the number open is not yet detected", but `[1e400` is a truncation case with no repeated name. "The 'first in document order' of FR-038-AC-109 through FR-038-AC-111 and FR-038-AC-93" names AC-109 and AC-111, and neither contains that phrase (AC-111 is about the package document). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:714-721 |

## Dispositions

Round 1, reviewed at 4cc3a814a41dce085fa0add11d1eed1f0b8d723c (fix commit 4cc3a81 on top of 3019802). Checks run on this head: `make fmt-check` and `make lint` pass, the touched module passes (22 tests), and `make spec` gives one grammar finding (FR-014) and 23 strict unbacked rows, as on base. The full `make test` was not rerun. That is acceptable: the delta is spec wording plus two cases and a digest filter in one test of the module that was run, and no production code changed. 3019802 passed `make test` with 517 tests.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 4cc3a81. AC-110's older clause now continues "…the reverse order names the whole number with `inexact-integer`, except that a number with no finite double is named when the reader reaches it, and so can be named ahead of an earlier inexact number", which mirrors QSL FR-056's own exception. AC-93's "names `/b`" sentence and the FR body's `document_pointer` sentence carry the same exception. Grep finds no other unqualified number "first in document order" (the remaining uses, at AC-62/AC-63, concern `unknown_member`). |
| FND-002 | fixed | 4cc3a81. The note now gives the order as the byte limit, then UTF-8 over the whole input, then one pass in document order. I reproduced both examples in a throwaway probe: `[1e400,"<0xFF>"]` gives `stale_dependency`/`byte-digest-mismatch` with no `document_pointer`, and `[1e400,"\ud800"]` gives `/0` with `inexact-integer`. The test gains both cases. |
| FND-003 | fixed | 4cc3a81. AC-110 now pins the raw-path documents only "as this reader's reading … offered under a digest that is not their raw digest". The test filters `expected == raw_digest_refusal()` to the foreign digest only. The note records QSL's `wrong-model-selection` under the raw digest as unspecified and tracked as IR-578 (Backlog, child of IR-573). IR-578 appears only in the note, TC-048 and the matrix note, never in an AC. |
| FND-004 | fixed | 4cc3a81. `[1e400` now has its own clause ("a truncation after the number is read after it (`[1e400` names `/0`)"), and the reconciling sentence names only FR-038-AC-93, FR-038-AC-110 and the note's `document_pointer` sentence. |
