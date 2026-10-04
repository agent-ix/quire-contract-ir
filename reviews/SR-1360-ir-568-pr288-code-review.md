---
id: SR-1360
title: "code review of PR 288 (IR-568 written-out bytes for every integer-holding canonical file)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@9adfb5a0259d84da4fc68471f886b56f7e4a73f6; tests/it/conformance.rs, spec/conformance/functional/FR-020-json-conformance-interface.md, spec/conformance/matrix/tests.md, spec/core/matrix/tests.md, spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-020
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-018
    type: reviews
---
# SR-1360: code review of PR 288

## Summary

Ticket: IR-568. PR agent-ix/quire-contract-ir#288. This is a code review with the
rust-review lane folded in. It covers `git diff origin/main...HEAD` at
9adfb5a0259d84da4fc68471f886b56f7e4a73f6: one commit, five files. The only Rust is
`tc_018_the_recorded_canonical_files_spell_the_eight_members_as_strings` in
`tests/it/conformance.rs`.

What was checked, and what holds:

- **No circularity.** The test does not call the encoder: there is no
  `quire_canonical` or encoder call anywhere in the function. Every expectation is a
  raw-string fragment or a slot written in the test. The recorded files are read only
  on the `assert_eq!` side and in the strings-only scan, never to build an
  expectation.
- **Each of the 47 files is covered exactly once.** An independent walk with
  `json.load` over all 78 files in `corpus/contract-v0.1/canonical` found 47 files
  with one of the eight members holding an integer string. That set equals the test's
  set exactly: 29 `expression-*-0.json` files sharing one declaration,
  `expression-numeric-edges-0.json`, and 17 `expression-*-1.json` files. No canonical
  file holds a JSON number in one of the members. No file holds only an integer
  `value`, so the scan's exclusion of `value` from `holds` hides no file.
- **Mutation, on a throwaway copy built with its own `CARGO_MANIFEST_DIR`.** Each of
  these mutations makes the test fail, and the unmutated copy passes:
  - A byte changed in a `-1` file (index, unsigned `maximum` 4 to 5).
  - A byte changed in a shared `-0` file (field-access, `Sensor` to `Sensos`).
  - A trailing newline added.
  - `"maximum":"4"` respelled as the number `4`.
  - A holding file added: "expression-zz-1.json has no expected bytes".
  - A holding file removed: the test panics at the `unwrap`, see FND-002.
  - An expectation dropped: the count assertion fails.
  - An expectation swapped for a duplicate, keeping the count at 47: "is expected
    once".
  - The `<U>`, `<I>` and `<BOOL>` fragments changed, and the numeric-edges slot
    changed.
- **The placeholder composition cannot mask a mismatch.** `replace` rewrites only the
  expected string and never the bytes that were read. Every name is closed by `>`, so
  `<I>`/`<INT>`/`<ITEMS>`, `<U>`/`<UINT>` and `<LIT>`/`<LITU>` cannot match inside
  one another. The corpus holds no `<`, and `assert!(!text.contains('<'))` rejects any
  unresolved name. So a wrong composition gives a wrong expectation, and that fails
  the comparison. It cannot make a wrong file pass.
- **Gate.** `make ci` was run at this head with its target dir inside the worktree.
  - fmt-check and lint (clippy, `-D warnings`) pass.
  - `test` passes: 361 integration tests, 149 model unit tests and 7 doctests.
  - `corpus` runs 107 fixtures, all `match`, exit 0.
  - It stops at `spec`: 23 unbacked rows and 0 contradicted (`--strict`), main's
    baseline. FR-020 is not among them, and quire reports FR-020 at 3/3.
  - `make deny cargo-audit audit-unsafe` was run separately and passes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The test's doc comment still has the partial-coverage wording: "equals the bytes written out here, or holds them only as strings". The test now asserts both conditions for every holding file. The "or" says that one of them is enough, which is the old four-of-47 meaning this PR removes | tests/it/conformance.rs:913-915 |
| FND-002 | low | `fs::read_to_string(canonical.join(name)).unwrap()` gives no file name when an expected file is missing. Removing `expression-index-1.json` panics with only `NotFound` at 1173:55. Use `unwrap_or_else(\|e\| panic!("{name}: {e}"))`, as the `assert_eq!` message already does | tests/it/conformance.rs:1173 |

## Verdict

Both findings are low (doc and diagnostics), and neither changes the verdict. The
oracle is independent of the encoder. It covers each of the 47 files once and fails
when a file is added, removed or changed, or when an expectation or fragment changes.
The composition cannot mask a mismatch. The gate is honest.

Verdict: mergeable. The two low findings can be fixed in this PR or accepted.

## Dispositions

Round 1 at 94ff9d809ad6acffd938b5f7b16796357040e4f4. Only the fix-round delta
9adfb5a..94ff9d8 was reviewed:

- Fix commit 2d9d525: `tests/it/conformance.rs`, 4 insertions and 3 deletions.
- Trailing commit 94ff9d8: adds SR-1360, SR-1361 and SR-1362 under `reviews/`.
  Their sha256 matched the reviewer's files before this section was added.

Checks at the new head:

- fmt-check and clippy (`-D warnings`) pass.
- The 10 `conformance::` integration tests pass.
- With `expression-index-1.json` removed, the test now panics with
  "expression-index-1.json: No such file or directory (os error 2)". The file was
  restored afterwards, leaving a clean tree.

No new findings.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 2d9d525ccfd0ce65726c02dcab2eaa755cebdf02 |
| FND-002 | fixed | 2d9d525ccfd0ce65726c02dcab2eaa755cebdf02 |
