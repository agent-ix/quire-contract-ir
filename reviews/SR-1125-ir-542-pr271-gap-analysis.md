---
id: SR-1125
title: "gap analysis of PR 271 against FR-038-AC-109 and FR-038-AC-111 (IR-542 code part 1)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@cb01b8277f92404d729f696baa91d8cfdd441695; git diff origin/main...cb01b82 (base 316fae1); spec/checked_package/functional/FR-038-consume-checked-package-v2.md (AC-109, AC-111; AC-110 read as context only, deferred to IR-555), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, spec/tests.md"
review_set: subset
---
# SR-1125: gap analysis of PR 271 against FR-038-AC-109 and FR-038-AC-111 (IR-542 code part 1)

## Summary

Ticket: IR-542. In scope: FR-038-AC-109 and FR-038-AC-111, traced to tests and code, and the trace-matrix edits. FR-038-AC-110 is deliberately not implemented here: its `1e400`/`-1e400` clause waits on a quire-canonical change, tracked as IR-555. Its absence is not a finding. Tests that assert its clauses are reported, as the brief asks.

What I examined:

- **FR-038-AC-109.** Every refused text in the AC is in the refusal test, under the document's own digest and under another digest: `0.1000000000000000000001`, `9007199254740993.5`, `-0.1000000000000000000001`, `4.9e-324`, `1e-400`, and the odd-digit ties `1125899906842624.3`, `1500000000000000.3` and `2.9802322387695313e-8`.
  - Every admitted text is in the admitted test, digested against the canonical text's own digest: `0.1`, `0.5`, `1.5`, `-0.25`, `5e-324`, `2.5e-10`, `1.0`, `-0`, `1e2`, and the even-digit ties.
  - The 0.1 / 0.1000000000000000000001 pair clause holds through those two tests.
  - The "through no serde_json value" clause holds by construction: the scan reads `quire_canonical::Document` nodes (`Number::text`, `Number::value`).
  - The oracles would catch a wrong scan. Rust's `{:?}` writes the odd-digit text for all three ties, so a scan on Rust formatting fails both tests. A scan on exact decimal values fails `0.1`. Each refusal compares the whole refusal: code, path, cause and `document_pointer`.
- **FR-038-AC-111.** The two inexact texts in a node body refuse `noncanonical()`, which has no path, no `document_pointer` and no cause. They refuse ahead of the stale package id that the same document earns. `0.1` and the three shortest round-trip texts reach `stale()`, so they are not refused `noncanonical_wire`. The manifest declares `float_roundtrip`, and a source check confirms it. A scratch serde_json build without the feature misreads the three texts, which confirms they are the right probes. On how strong the oracle is, see SR-1123 FND-001.
- **Document pointer and order.** `/package/count` and `/b` are tested. So are `/a/0`, an array element of a member, and the nested `/types/1/constraints/1/operands/value` of the AC-93 test. In the members-first order test, members come in document order (`b` before `a`). A walk in sorted order would fail it.
- **Trace matrix.** spec/checked_package/matrix/tests.md now says AC-109 and AC-111 are implemented and verified by TC-048 in the two named files, and AC-110 is planned. That is accurate at file level. spec/tests.md says the same. `make spec` keeps the baseline: validate passes, with 1 grammar finding and 23 unbacked under strict. The count is identical on origin/main.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A test tagged to AC-109 asserts an AC-110 clause. tc_048_the_first_inexact_number_in_document_order_is_named_with_its_own_cause carries `#[trace("TC-048", "FR-038-AC-109")]`. AC-109 says nothing about document order or about which cause wins. The test asserts AC-110's last sentence verbatim: 'a document holding an inexact number at /b and then a whole number past 2^53 at /a/0 names /b with inexact-number ... and the reverse order names the whole number with inexact-integer'. The matrix says AC-110 is planned, yet this AC-110 clause is implemented and asserted under the wrong tag. The existing AC-93 tests now also assert `InexactInteger`, which is AC-110's first clause. TC-048 discloses this, and it is unavoidable because those tests compare whole refusals, so it is not a separate finding. Fix: retag this test to FR-038-AC-110. Then either let the matrix call AC-110 partial ('implemented except the 1e400/-1e400 clause, IR-555') or keep the test untagged until IR-555. Do not leave it backing AC-109. | tests/it/checked_package_v2_model_members.rs:710-743 |
| FND-002 | low | The `9007199254740992.5` case is an AC-110 clause ('9007199254740992.5 (whose nearest double is 9007199254740992) refuses inexact-number'). It is asserted inside the AC-109-tagged refusal test, and AC-109 does not name it. The behaviour is right and the assertion is strong. Only the binding is wrong. Move it with FND-001's AC-110 assertions. | tests/it/checked_package_v2_model_members.rs:661-662 |
| FND-003 | medium | FR-038's prose now contradicts the code this PR adds. Lines 603-606 say two model documents that differ only in `0.1000000000000000000001` and `0.1` 'share one digest: the reader refuses neither'. The PR refuses the first. Lines 618-620 say 'The refusal causes CheckedPackageRefusalCause carries have no inexact-integer or inexact-number member'. The PR adds both to shared.rs. The code PR should turn these 'today' sentences into past or current state, as earlier code changes did for their own 'today' prose (for example the IR-549 and IR-530 amendments). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:603-606, 618-620 |
| FND-004 | low | AC-109 and TC-048 fix the pointer as `/package/ratio`. AC-109 says 'holding at /package/ratio the number ... with document_pointer equal to /package/ratio', and TC-048 lines 638-640 say the same. The tests use `document_bytes_with_count`, so the pointer they assert is `/package/count`. The behaviour does not depend on the member name, but the test does not do what the AC and TC literally state. Either add a `/package/ratio` builder, or loosen AC-109 and TC-048 to 'a member of package'. | tests/it/checked_package_v2_model_members.rs:461-468, 645-743 |
| FND-005 | low | The spec/tests.md Checked package cell was edited by this PR, and it still makes stale claims that contradict the checked_package matrix row: 'AC-62 through AC-64 planned, IR-535' and 'AC-89 through AC-95 planned, IR-274'. The matrix row says AC-62 through AC-64 are implemented and verified (IR-535), and that AC-89 through AC-95 are implemented by code change A, with AC-91/AC-92 partial. The text predates this PR, but this PR rewrote the cell around it. Bring the summary in line with the matrix row. | spec/tests.md:15 |

## Verdict

AC-109 and AC-111 are fully implemented. Their tests have strong oracles: whole-refusal equality under both digests, admitted cases digested against the canonical text, and tie, order and sign cases that a wrong scan would fail. Four bindings were checked. Three are correct, and one is wrong (FND-001: an AC-110 clause tagged AC-109). FND-002 is a stray AC-110 assertion inside a correct AC-109 test. FND-003 is FR-038 'today' prose that this PR makes false. FND-004 and FND-005 are low text-to-test and matrix-summary mismatches. Nothing asserts AC-110's deferred `1e400`/`-1e400` clause. No production code without an owning requirement was found.

## New findings (disposition pass 1)

These were found at 3a34916a52918fb7ae1c1c96df4e40842b829c88.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | The FND-004 fix brought the same kind of mismatch back on FR-038-AC-93. `document_bytes_with_count` was renamed to `document_bytes_with_number` and now writes `/package/ratio`. The AC-93 tests share that builder (tc_048_a_model_document_number_past_2_pow_53_refuses_with_its_document_pointer and tc_048_a_model_document_number_at_or_under_2_pow_53_is_digested), so they now assert `document_pointer` `/package/ratio`. FR-038-AC-93 (line 2137) and TC-048 (lines 602-604) still fix that pointer as `/package/count`. Give the builder the member name as a parameter, using `count` for AC-93 and `ratio` for AC-109, or align the AC-93 and TC-048 text. | tests/it/checked_package_v2_model_members.rs:459-468, 505-517; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2137 |
| FND-007 | low | FR-038 line 641 now cites 'QSL ticket: QSL-219' as the ticket where QSL will amend FR-056 and change quire-canonical's read error to carry the number's pointer and lexeme. QSL-219 is in the Done state. Its scope is the intake-digest spec gap for lone surrogates and integers with no exact double, and the read-error change is not among its acceptance criteria. IR-555, the IR follow-up, also says the ruling is 'not yet recorded on a QSL ticket'. The same line adds 'no quire-canonical PR exists yet', which is true: quire-canonical's PRs #1 to #7 are all merged and none is this change. Cite the QSL ticket that actually tracks the read-error change, or keep 'pending' and point to IR-555. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:641 |

## Dispositions

Round 1 was reviewed at 3a34916a52918fb7ae1c1c96df4e40842b829c88. The changed integration modules pass (34). make spec stops at the baseline: validate passes, grammar 1, strict 23 unbacked, 0 contradicted. AC-110 is now tagged and the matrix records it as partial, and strict reports no contradiction.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed 3a34916 | tc_048_the_first_inexact_number_in_document_order_is_named_with_its_own_cause now carries `#[trace("TC-048", "FR-038-AC-110")]`. TC-048 says 'FR-038-AC-110 is partial: all but its `1e400` and `-1e400` clause is verified, and that clause stays planned (IR-555)'. Both matrices say the same. |
| FND-002 | fixed 3a34916 | `9007199254740992.5` was removed from the AC-109 refusal test. It is now the first case of the AC-110-tagged order test: `("\"b\":9007199254740992.5", "/b", CheckedPackageRefusalCause::InexactNumber)`. |
| FND-003 | fixed 3a34916 | Line 602 now reads: 'because the reader refuses a number whose text is not the exact spelling of its double's value, two model documents that differ only in such a number ... no longer share one digest: the first of each pair is refused'. The causes sentence now reads: '`CheckedPackageRefusalCause` carries the members `inexact-integer` and `inexact-number`'. |
| FND-004 | fixed 3a34916 | The builder now writes `/package/ratio`, and the AC-109 tests assert `number_refusal("/package/ratio", CheckedPackageRefusalCause::InexactNumber)`. The fix moved the AC-93 tests off their pointer; that is recorded as FND-006. |
| FND-005 | fixed 3a34916 | The spec/tests.md cell no longer claims AC-62 through AC-64 or AC-89 through AC-95 are planned. It now reads: '(AC-81 through AC-86 and AC-88 planned and untagged, IR-532 binding PR; AC-96 through AC-108 implemented, IR-549, and AC-66 retired; AC-109 and AC-111 implemented and AC-110 partial (all but its `1e400`/`-1e400` clause, IR-555), IR-542; ...)'. |

Round 2 was reviewed at 1f931f3726fac2f3e2e208855e1857769f55608d, which is one commit on 3a34916 and touches only tests/it/checked_package_v2_model_members.rs and the FR-038 note. Nothing else moved. The changed integration modules pass (34). make fmt-check and make lint pass. make spec stops at the baseline: grammar 1, strict 23 unbacked, 0 contradicted.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-006 | fixed 1f931f3 | `fn document_bytes_with_number(member: &str, number: &str)`. The AC-93 tests pass `"count"` and assert `/package/count`, as FR-038-AC-93 and TC-048 state. The AC-109 tests pass `"ratio"` and assert `/package/ratio`. |
| FND-007 | fixed 1f931f3 | The note now reads '(not yet recorded on a QSL ticket, tracked on the IR side as IR-555; no `quire-canonical` PR exists yet)'. It no longer cites QSL-219. |
