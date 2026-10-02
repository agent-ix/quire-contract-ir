---
id: SR-715
title: "code review of PR 246 (IR-475 KaniProfile validated on every construction path)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@1bd313563d7f6a1ea704e7c0868fb3b4d4f94f4e; src/kani/profile.rs, src/kani/arithmetic.rs, src/kani/collections.rs, src/kani/objects.rs, tests/it/kani_shared.rs"
review_set: subset
---
# SR-715: code review of PR 246

## Summary

Ticket: IR-475. Reviewed head 1bd3135 against origin/main 4233b56 (the merge base). The
code-review and rust-review lanes are in this one file.

The change makes the two `KaniProfile` fields private. It adds `selection()` and
`capabilities()` accessors. It routes `Deserialize` through a private `KaniProfileWire`
with `#[serde(try_from = "KaniProfileWire")]`, and the `TryFrom` impl calls
`KaniProfile::new`. The three in-crate lowering modules switch from `profile.selection` to
`profile.selection()`.

What was measured:

- Construction paths. `KaniProfile` has no `Default`, no builder, no other `pub fn` that
  returns `Self`, and no `From`/`TryFrom` impl except the private-wire one. The accessors
  return `&ProfileSelection` and `&[CapabilityEntry]`, so a caller cannot mutate the
  validated selection or matrix in place. `Clone` copies a value that is already valid. No
  type in IR embeds `KaniProfile` as a field (grep over `src/` and `tests/`). Any future
  embedding type that derives `Deserialize` goes through the same `try_from`.
- Wire form. `Serialize` is still derived over the same two fields in the same order, so
  existing valid profiles serialize to identical bytes. The wire struct has no
  `deny_unknown_fields`, and neither did the old derive, so inputs that were accepted
  before and are valid are still accepted. Input that was accepted before but is invalid is
  now refused. That is the fix.
- Error fidelity. serde maps the `ProfileError` through its `Display`, so the deserialize
  error carries the same message as `new` (for example "invalid capability matrix: a"). The
  typed variant is lost, which is a serde limitation and is acceptable.
- Mutation. In a scratch copy I deleted only the `#[serde(try_from ...)]` line. The new
  test then fails at tests/it/kani_shared.rs:115 ("deserialize refuses ..." on the
  wrong-family case). The oracle discriminates. Each of the four invalid cases hits a
  different check in `new`: the family check, the blank-revision check, the duplicate
  check, and the blank-construct check. Deleting any one of those checks makes the `new`
  arm fail for its case.
- Gates. `cargo test --test it kani_` passes 13 of 13. `cargo clippy --workspace
  --all-targets -D warnings` is clean. `make spec` on head and on main both report "23
  unbacked row(s) and 0 contradicted status(es)". The only difference is one more bound
  symbol (239 to 240 bound, 267 to 268 candidates), and no new warning. The coder's
  ir-475-ci.log ends with `head=1bd313563d7f6a1ea704e7c0868fb3b4d4f94f4e exit=2`, and its
  only failing target is `spec` (Makefile:63).
- Downstream. quire-driver and quire-contract-runtime have no `KaniProfile` reference. CG
  has no `KaniProfile { .. }` literal. Every CG construction is `KaniProfile::new`, and CG
  never deserializes a `KaniProfile`. CG does read the public `selection` field, so it
  breaks when it next bumps IR. See FND-001.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The PR body's list of CG call sites comes from a stale CG checkout, 25 commits behind origin/main. On CG origin/main 2130cd9 the `profile.selection` reads are at src/bounded_kani_corpus.rs:394, 399, 404, 450, 494 and 499, not 206, 211, 216, 310, 315 and 329. Line 450 is a borrow (`profile: &profile.selection`) that becomes `profile.selection()`, not `.selection().revision`. The compiler will find every site, so the risk is only a misleading handoff | PR body; quire-contract-codegen/src/bounded_kani_corpus.rs:394-499 |
| FND-002 | low | The deserialize arm of the new test asserts only `is_err()`. It does not check that the refusal is the one `new` gives (same `ProfileError` message). If the wire shape later drifts from `Serialize`, a plain shape error would satisfy the assertion and the arm would go vacuous. The mutation passes today. Comparing `err.to_string()` with the `new` error's `Display` would pin it | tests/it/kani_shared.rs:111-115 |

## Verdict

Mergeable. The validation is complete for every construction path that exists: `new`,
`Deserialize` (direct and nested), and `Clone`. Struct literals and field mutation are
closed by the compiler. Serialization is byte-stable for valid profiles. The test fails
without the fix. Both findings are low and need not block the merge. FND-001 should be
corrected in the PR body or in the CG follow-up ticket.

On the CG break (brief question 4): landing this before CG is the only workable order,
because CG cannot call `selection()` until IR has it. CG's lock is on IR 968ba9b, so
nothing breaks until CG bumps. The leader should file a CG follow-up that bumps the IR lock
past this merge and switches the six reads to `profile.selection()` in the same PR, and
should not demand that CG change here. Removing the `pub` fields is a breaking change to the
public API. The only first-party consumer is CG, so this is acceptable for prerelease
software.

PR hygiene: the title carries no ticket id, and the body says "Closes IR-475". No spec file
changed, so no SHA was added to a spec. Apart from FND-001, the body is accurate.

## New findings (disposition pass 1)

Reviewed at 66967e8cec9449b75d668aeb07d8cf6708bfcd4e (delta 1bd313563d7f6a1ea704e7c0868fb3b4d4f94f4e..66967e8cec9449b75d668aeb07d8cf6708bfcd4e, one commit, tests/it/kani_shared.rs only).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | The PR body uses abbreviated commit ids: "Re-measured on origin/main 4233b56" and "Re-measured on CG origin/main 1629715". The first is also stale: IR origin/main is now a91d5bd39be24ac85463fb007aade51c9e1fd8d8 (PR #247 landed), and the PR shows BEHIND. Use full ids or drop them | PR body |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | No commit: the PR body and Linear comment 7deadaea-2440-41d8-b660-a48dc80c07aa now list bounded_kani_corpus.rs:394, 399, 404, 450, 494, 499, with :450 as a borrow. Re-measured on CG origin/main 16297152d361bf022d3c1f05f0e2ec4caaf4f3f8: the six `profile.selection` reads are at exactly those lines |
| FND-002 | fixed | 66967e8cec9449b75d668aeb07d8cf6708bfcd4e: the deserialize arm now does `expect_err` and `assert_eq!(deserialized.to_string(), constructed.to_string())`. Mutation check: adding `#[serde(rename = "sel")]` to `KaniProfileWire.selection` fails it at tests/it/kani_shared.rs:117 (left "missing field `sel`", right "unsupported bounded-Kani profile `kani-bounded/2`"). The test passes at head |
| FND-003 | still-open | Found this round. The PR body still has the abbreviated ids. Fixing it is a body edit only, with no code change |
