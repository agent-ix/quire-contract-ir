---
id: SR-791
title: "gap analysis of PR 250 against IR-505 (content-only ModelOwner)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@286dc707986d19f2d9a38f3b752be436d5b278ec; spec/checked_package/functional/FR-038 (ModelOwner paragraph, Model-owned members step 2, FR-038-AC-5, FR-038-AC-45); spec/checked_package/functional/FR-040 (AC-3, AC-7, AC-13); spec/checked_package/matrix/TC-048 (FR-038-AC-45 procedure); spec/checked_package/matrix/tests.md (FR-038, FR-040, TC-048, TC-056 rows); IR-505 ticket and comments (untrusted data)"
review_set: subset
---
# SR-791: gap analysis of PR 250 against IR-505

## Summary

Ticket: IR-505. This gap analysis checks the diff against the merged spec (#249): the
FR-038 `ModelOwner` "shall" paragraph, step 2 owner recovery, FR-038-AC-45, FR-040
AC-3/AC-7/AC-13, and the TC-048 FR-038-AC-45 procedure.

Spec coverage, measured with `make spec`:

| | merge base db5ca1c | head 286dc70 |
| --- | --- | --- |
| Rows backed | 163/185 | 164/185 |
| FR-038 | 42/43 | 43/43 |
| FR-040 | 13/13 | 13/13 |
| Strict unbacked rows | 23 | 23 |
| Contradicted statuses | 0 | 0 |

At both commits the 23 unbacked rows are all in `spec/kani`, `spec/core` and
`spec/model`, and none is in `spec/checked_package`. The coder's `ir-505-ci.log` ends
`head=286dc707986d19f2d9a38f3b752be436d5b278ec exit=2`, and the only failing target is
`spec`. fmt, clippy, the workspace tests (181 + 88 + 2), deny, audit and the unsafe
audit are all green.

Requirement-to-code-to-test, per unit:

- **FR-038 ModelOwner paragraph and step 2.** The reader decodes `{kind, identity, node}`,
  refuses `version` as `unknown_member`, and recovers owners by the version-free
  `declaration_key`. The lock's selection version is still checked as selection evidence
  (model_members.rs:886, FR-038-AC-27 unchanged). Implemented.
- **FR-038-AC-45.** Its clauses and their tests:
  - Content-only package admits: `tc_048_a_content_only_model_owner_...`.
  - Version-only change admits with the same graph: the same test, and
    `tc_048_a_declaration_node_key_is_unchanged_...`, which re-reads at 2.0.0. Probe 1
    in SR-790 shows that check is real.
  - `version` member refuses `unknown_member` at that member: the reader test. Probe 2
    in SR-790 shows it is real.
  - Empty identity or node refuses `invalid_semantic_graph`: the reader test.
  - The same node keyed under another domain package refuses: see FND-001.
- **FR-040 AC-3, AC-7, AC-13.** The "unselected domain package" cases now key under
  `acme/other` and assert `MissingDeclaration`/`MissingSelection` at the exact member
  pointer. They are not vacuous: under the old other-version key the node resolves and
  admits, so the assertion can fail. #[trace] tags: the AC-3 test carries FR-040-AC-3,
  the AC-7 test FR-040-AC-7, and the reaches_field refusal test FR-040-AC-13. All three
  are correct, and none is invented.
- **Matrix flips.**
  - FR-040 🚧 to ✅ and TC-056 🚧 to ✅: backed (13/13, with the re-keyed cases).
  - TC-048 🚧 to ✅: backed (AC-5's model-owner clause through
    `tc_048_model_owners_join_sha256_jcs_domain_package_selections`, and AC-45 above).
  - FR-038 stays 🚧: see FND-002.
- **PR hygiene.** The title carries no ticket id. The body has no short SHAs; the only
  SHA is the full head. For "Closes IR-505", see FND-003.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-45's clause "the same node keyed under another domain package's identity refuses `missing_declaration`/`missing-selection` at the member's `declaration`" has no reader-level oracle under an AC-45 trace. The AC-45-tagged test only checks `assert_ne!(declaration_key_of(ORDER, IDENTITY), declaration_key_of(ORDER, OTHER_PACKAGE))`. That compares two hashes computed by the test helper, and it passes whatever the reader does. The reader refusal is asserted only in `tc_056_reaches_field_refuses_an_invalid_edge_where_it_fails`, which is traced to FR-040-AC-13 alone. The TC-048 procedure names this step explicitly. | tests/it/checked_package_v2_model_members.rs:288 |
| FND-002 | low | The FR-038 matrix row stays 🚧, but the rewritten cell now says AC-1 through AC-15, AC-17 through AC-33 and AC-35 through AC-45 are all implemented. The only listed ids missing from that range, AC-16 and AC-34, do not exist. The cell states no reason for 🚧. The reason the PR body calls "pre-existing" lives in FR-038's own text, not the cell: the STD-125 depth-ceiling deviation and incomplete operand typing. | spec/checked_package/matrix/tests.md:15 |
| FND-003 | low | "Closes IR-505" will auto-close the ticket with two of its stated items not done. First, it asks for "a TC that a QSL A3k package admits". The PR builds the content-only shape in-repo and rightly copies no QSL vector, so nothing anchors the reader to QSL's emitted bytes. Second, it says to "close IR-243", which is still Backlog. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:50 |

## Verdict

The PR delivers IR-505's code scope, and the scope widening recorded in the ticket
comments: `declaration_key` and the AC-3/AC-7/AC-13 fixtures in both test files. The
matrix flips for FR-040, TC-048 and TC-056 are genuinely backed. `make spec` improves
by one backed row (FR-038-AC-45) and adds no regression.

The 🚧 on FR-038 is defensible. FR-038 itself records open deviations: the STD-125
depth ceiling, and leaves left unchecked until operand typing is complete. The cell
should say so (FND-002), and the PR did not introduce that gap.

Fixes, in order:

1. FND-001, before merge. Either add `FR-038-AC-45` (with TC-048) to the trace of the
   AC-13 refusal test, or replace the `assert_ne!` with a reader read of a package
   whose declaration node is keyed under `OTHER_PACKAGE`, asserting
   `MissingDeclaration`/`MissingSelection` at the member's `declaration`.
2. FND-002 and FND-003, cheap tracker and matrix hygiene. Name the 🚧 reason in the
   cell. Before IR-505 closes, close IR-243 as superseded by STD-145. Record on IR-505
   where the QSL A3k admission check lives: QSL-353's emission-to-admission corpus, or
   a follow-up ticket. The alternative is "Part of IR-505".

origin/main is two commits ahead of the merge base, and neither touches this diff.
Under the repo's cadence the full gate runs again on the merged state before merge.

## Dispositions

Round 1, reviewed at head 90f6a3a33ceee1315cdea212aca8b4c503636bf5. The PR was rebased onto
origin/main e80ea70, which is also its merge base. `git range-diff` shows the original
commit unchanged (286dc70 = fd27d14), so only the fix commit is new.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 90f6a3a33ceee1315cdea212aca8b4c503636bf5: the new `tc_048_a_declaration_node_keyed_under_another_domain_package_refuses` (traced TC-048, FR-038-AC-45) reads a real package whose member declaration is `Order` keyed under `OTHER_PACKAGE`. It asserts `MissingDeclaration`/`MissingSelection` at `body/operation/member/declaration`, plus a positive control: the selected package's key admits. Probes: changing the reader's `unselected()` cause to `MissingName` fails it; putting `version` back into `declaration_key` fails its positive control. |
| FND-002 | fixed | 90f6a3a33ceee1315cdea212aca8b4c503636bf5: the FR-038 cell now says AC-16 and AC-34 do not exist. It names the reason for 🚧: the depth-ceiling deviation pending STD-125, and incomplete operand typing (operands not refused `operator-ineligible`). Both match FR-038's own text. |
| FND-003 | fixed | PR body edit at head 90f6a3a33ceee1315cdea212aca8b4c503636bf5 (not a commit): the body now opens "Part of IR-505" and says the QSL-emitted A3k admission test is not here and IR-243 is left open. IR-505 therefore stays open for those two items, and IR-243 is still Backlog. |

Round-1 measurements:
- `make -k ci` log (`ir-505-final-ci.log`) ends `head=90f6a3a33ceee1315cdea212aca8b4c503636bf5 exit=2`. Only `spec` fails: 164/185 rows backed, 23 strict unbacked, 0 contradicted (the same baseline). The workspace tests pass (183 integration, 88 unit, 2 doc), and fmt, clippy, deny, audit and the unsafe audit are green.
- Focused tests rerun here: `cargo test --test it checked_package_v2` 103 passed, quire-contract-model unit tests 88 passed.
- No new findings.
