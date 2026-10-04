---
id: SR-1236
title: "code review of PR 285 (IR-495 flat v2 wire, no depth limit, iterative walks on quire-walk)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@f3d41bca80fe145bdabd345e1d13749f6be1fcaf; crates/quire-contract-model/src/checked_package/**, crates/quire-contract-model/Cargo.toml, Cargo.lock, deny.toml, Makefile, CLAUDE.md, tests/it/checked_package_v2_*.rs, tests/it/complete_v1_checked_package.rs, tests/it/support/checked_package.rs, tests/conformance_qspec/main.rs"
review_set: subset
---
# SR-1236: code review of PR 285

## Summary

Ticket: IR-495. Code review with the rust-review lane folded in, over
`git diff origin/main...HEAD` at f3d41bca80fe145bdabd345e1d13749f6be1fcaf (merge base
ffb86d9fec6a61c64bc0eaa84ba0386575776d38, two commits). Main has since moved to 69a1116
and the PR is CONFLICTING (one adjacent-line conflict in `spec/tests.md`).

What was checked, and what holds:

- **Flat wire semantics (`v2/flat_wire.rs`).** The `Place` table matches merged QSpec
  FR-322 "Body grammar" stratum for stratum: Member admits a Leaf, a Group or a binding;
  a Group member is a Leaf or a binding of a Leaf; a Tuple member is a Leaf or a Group; a
  Member binding's value is a Leaf, Group or Tuple (decided by the first non-Leaf member);
  a binding never stands at a body root. Every nested application refuses at the
  application: `case` `ill_typed`/`operator-ineligible` at its `operator`, a
  `temporal_formula`/`temporal_fairness` application likewise only in a `details` term,
  every other class (unknown operator included) `malformed_wire`. The walk is document
  pre-order, outermost first (quire-walk enters children in push order). `flat_wire::check`
  runs after `check_package_header` and before the `package_id` recomputation, so AC-116
  holds in both directions. A frame body and an abstraction relation body are scanned by
  `scan_members`; merged FR-038 "The flat wire" ("an application standing in either is
  refused as below") and FR-346 (lines 128-132, merged in #280/#281) already state
  `malformed_wire` for an application inside them, so that is no silent spec change. A
  body-root application with an unknown operator keeps `invalid_semantic_graph` (AC-65).
  R-S8 is as merged: the nested `case` is decided in the same pre-order walk.
- **No depth limit.** `CheckedPackageReadLimits::depth`, `MAXIMUM_DEPTH` and
  `CheckedPackageLimit::Depth` are gone; `strict_parse` uses serde_json's default
  recursion limit, no `disable_recursion_limit`, no `serde_stacker`. The scan test finds
  no `stacker`/`serde_stacker`/`on_stack_for`/`MAX*DEPTH` token under `checked_package/`.
  The break is real downstream: CG origin/main `src/oracle/mod.rs:79,97` and
  `src/oracle/scalar/mod.rs:3427`; QSL origin/main `qsl-package/src/checked_v2.rs`
  (`MAXIMUM_DEPTH` clamp at 238, `Depth => LimitKind::NestingDepth` at 512) and its tests.
  IR-565 (CG) and QSL-488 (QSL) both exist and are blocked by IR-495.
- **Walks.** The term validator, flat-wire check, member scan, dependency-reference walk,
  state and temporal placement walks and the reference collectors are `quire_walk::Walk`s
  with a shared `Cursor` that builds a pointer only on refusal. A self-call scan of every
  function under `checked_package/` finds no recursion over input other than
  `resolve_family` (pre-existing, FND-004) and serde's own decode (FND-002). The graph
  worklists (closure BFS, Tarjan, signature closure) are explicit-stack and unchanged.
  `group_references` rewrites in one visit. The `NodeSourceMaps` index removes the
  per-node scan of the whole source map (quadratic in node count before).
- **Config.** One copy of `quire-walk` in `Cargo.lock` (89d05df); the manifest uses
  `branch = "main"` like quire-canonical (no SHA pin); the `deny.toml` `allow-git` entry is
  one exact URL, consistent with the three existing first-party entries, and is the right
  mechanism (no policy loosened). AGPL-3.0-or-later matches the crate. The `Makefile`
  `LOCAL_PATCHES` entry and the one-line `CLAUDE.md` command-list update are minimal,
  correct and in scope (they document the new dependency in `use-local`).
- **AC-9 "seven" to "six".** Correct: the Inputs list names six limits.
- **Tests adapted.** The depth-charging tests became recursion-limit and no-depth-limit
  tests; the 6,000-level admitted-package tests became the AC-117 tests; the limit tables
  lost only the depth row; catalog words, dependency reference and frame entries now
  expect `malformed_wire` at the nested application as merged AC-114 requires; the AC-65
  unknown-operator test moved to a body root and a new test pins the nested case. The
  self-typed carve-out test's binding root is wrapped in an aggregate because a binding
  body root is now itself `malformed_wire` (covered by the flat-wire tests); the carve-out
  assertion it keeps is unchanged, so that is a legitimate adaptation. The abstraction
  deep-member test went from 3,000 to 60 levels: 3,000 is now past the parse limit, so a
  shallower body is needed to reach the shape check, but 60 sits just under the debug
  overflow window of FND-002.
- **Gates, run in this review's own worktree and target dir.** `make fmt-check`,
  `make lint` (both lanes), `make test` (344 + 142 + 7 plus doctests, all pass; the
  100000-node test 97.6 s), `make corpus`, `make deny` (plus one-copy), `make cargo-audit`,
  `make audit-unsafe` all pass. `make conformance-qspec` against a fresh worktree of
  quire-specification origin/main 2f846f8: 6/6 pass, the five `body_grammar_mutations`
  refuse `malformed_wire`, expected-failure list empty.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Public repo now depends on a private repo. quire-contract-ir, quire-contract-codegen and quire-spec-language are PUBLIC; agent-ix/quire-walk is PRIVATE (gh repo view). After this merges, nobody outside the org can build IR, its manual CI workflow cannot fetch the dependency with the default token, and CG and QSL inherit the private dependency on their next lock bump. Not a defect in the `allow-git` entry itself; it needs an owner decision (make quire-walk public before merge) before landing | crates/quire-contract-model/Cargo.toml:49 |
| FND-002 | high | Untrusted input within the parse limit aborts the process. A canonical package whose node body nests 43 to 61 aggregates (about 91 to 127 JSON levels) read with `CheckedPackageV2::read` on a 256 KiB thread in a debug build dies with "has overflowed its stack" (reproduced at 43, 45, 50, 55, 58-61; 41 and 42 refuse at `.../members/0/members/0`; 62+ refuse at the parse). gdb puts the overflow in the closed-schema decode: `decode_closed` re-deserializes each `Value` body through serde_path_to_error and serde_json's `ValueVisitor`, about 10 frames per level. That recursion ran under `on_stack_for` before; this PR removed it and added nothing. Release passes on 256 KiB. FR-038 "Reading" says no walk after the strict parse recurses on the call stack at any depth. Fix: move each body/details `Value` out of the document before the closed decode and back into the typed wire after (no Value-to-Value re-deserialize), so the only recursion left is the bounded parse and drop; add the 127-JSON-level case on a 256 KiB thread as a test | crates/quire-contract-model/src/checked_package/common.rs:350; crates/quire-contract-model/src/checked_package/v2/mod.rs:602 |
| FND-003 | medium | The 100000-node AC-117 test costs 97.6 s and 5.1 GB peak RSS in debug, and 16 s and 5.1 GB in release (/usr/bin/time -v, run alone). It has no `#[ignore]`, and `make test` would run it anyway (`--include-ignored`), in parallel with the rest of the suite, on a host the owner says is short of disk and memory. The memory is about 29x the 177 MB document (about 51 KB per node), so either the reader or the fixture builder amplifies it. Find where the 5 GB goes (the parsed `Value`, the typed wire, the lowered package that clones every dependency node and its source map) and cut it. Or give the test its own make target that runs in release; AC-117 fixes n at 100000, so n cannot shrink | tests/it/checked_package_v2_flat_wire.rs:543-582 |
| FND-004 | medium | Pre-existing, outside the diff, but against the PR's claim: `resolve_family` recurses through `bounded_domain` chains and returns `None` past `depth > 8`. That is a node-chain depth cap whose outcome follows the input's chain length (an operand family check is skipped at operations.rs:1977, or a family is missing at :740). The AC-117 scan cannot see it because it is a literal, not a `MAX*DEPTH` token. Under the owner's no-depth-cap ruling, replace it with a visited-set walk, or file it as a ticket and name it in the PR body | crates/quire-contract-model/src/checked_package/v2/operations.rs:1053-1082 |
| FND-005 | low | The PR body names only IR-565 as the follow-up for both consumers. QSL's adaptation is QSL-488. The CG reference `src/exact_scalar.rs` near 3366 is stale: on CG origin/main the arm is at `src/oracle/scalar/mod.rs:3427`, plus `src/oracle/mod.rs:79,97`. Name QSL-488 and fix the paths | (PR body) |
| FND-006 | low | `tc_048_qspec_adverse_mutations_refuse_as_recorded` asserts `EXPECTED_FAILURES.is_empty()`, so the AC-112 path ("a mutation the reader does not yet refuse as recorded is named in the expected-failure list ... with the open ticket") is now a test failure, not a listed exception. That contradicts the const's own doc comment ("an entry added here ... must name the open ticket"). Keep the AC-118 check of the five body-grammar ids and drop the blanket emptiness assert, or say in the comment that a new entry must also edit the assert | tests/conformance_qspec/main.rs:212,438-441 |

## Verdict

Request changes. The flat-wire semantics, the order of checks, the API removal, the walk
conversions and the test adaptations are correct and match the merged spec. Two problems
block the merge. FND-002 is a process abort on untrusted input inside the parse limit, at
the 256 KiB stack the spec names, in the repo's own debug test profile. FND-001 is a
public crate depending on a private one, which needs an owner decision. FND-003 is the
gate cost, and FND-004 is a remaining depth cap the PR's "no depth limit" claim does not
cover. The PR is CONFLICTING on `spec/tests.md`, a trivial adjacent-row conflict with
main's IR-567/IR-569 rows; it must be rebased and the gates re-run.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | low | FR-038 "Reading" now says the only recursion left over a deep value is the strict parse and its drop. There is a third: when a stale identity-projection body nested deep is refused, `serde_json::to_value` and `first_difference` walk it in `validate_graph`. The body grammar does not check projection bodies, so they can nest to 127 levels. The path is bounded by the parse limit, and this review measured no overflow: a projection body of 61 aggregates (126 JSON levels) refuses `stale_dependency` at `/identity_preimage/identity_projection/3/body/members` on a 256 KiB debug thread. But no test covers it, and the sentence leaves it out. Name the path in "Reading" and add a deep projection body to the window test | crates/quire-contract-model/src/checked_package/v2/mod.rs:1817-1840 |

## Dispositions

Round 1, reviewed at b7dd914b9c995dc9979d31cebfea5feafb4822ad: a rebase onto main 69a1116 plus one fix commit. `git range-diff` shows the two original commits unchanged apart from the `spec/tests.md` context. Gates rerun by this review: fmt-check, lint, test (348 + 147 + 7, 3:01 wall time, 4.0 GB peak), corpus, deny, cargo-audit, audit-unsafe, conformance-qspec (6/6 against QSpec 2f846f8), validate (1 grammar finding, FR-014) and coverage --strict (23 rows, as on main) all pass.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | still-open | Owner decision. quire-walk is still PRIVATE and IR, CG and QSL are PUBLIC. Nothing in the code can fix this; it is the only blocking finding left |
| FND-002 | fixed | b7dd914 (`v2/intake.rs`). Bodies, projection bodies and `details` terms are detached before the closed decode and put back after it. Reproduced on a 256 KiB debug thread: bodies of 20, 41, 43, 50, 55, 58 and 61 aggregates refuse `malformed_wire` at `.../body/members/0/members/0`, and 62 refuses at the parse. Detach and attach are order-preserving moves (no clone), one slot per document element, and they line up whenever the decode succeeds, so no body can be swapped, duplicated or altered. The closed decode never failed inside a `Value`, so taking them out changes no decode error. The chunked lossless check keeps the projection-before-nodes-before-source-map order. The "rest" comparison runs first, but the only lossy members (`skip_serializing_if` options) live in nodes and projection entries, so which error wins does not change |
| FND-003 | fixed | b7dd914. Measured alone in debug: 87.8 s and 2.27 GB peak (was 97.6 s and 5.1 GB). That is within about 1.6x of the Value-parse floor, and the fixture is now built as text. Acceptable for the gate. A streaming typed decode is optional future work, not required here; ticket it only if the owner wants the reader's memory below the parse floor |
| FND-004 | fixed | b7dd914. `resolve_family` is an iterative loop with a visited set and no cap. `tc_048_a_bounded_domain_chain_resolves_whatever_its_length` covers a 300-long chain, a cycle, a chain ending in no family, and a type outside the graph |
| FND-005 | fixed | b7dd914 (PR body, verified at this head). It names IR-565 (CG) and QSL-488 (QSL), and CG `src/oracle/mod.rs` 79/97 and `src/oracle/scalar/mod.rs` near 3427 |
| FND-006 | fixed | b7dd914. The blanket emptiness assert is gone; the test now asserts that no listed exception is one of the five body-grammar ids |

Round 2, reviewed at 687772900114031597801d8807614465a87230f6. Compared with b7dd914, this round changes only the FR-038 "Reading" sentence and adds one test (+51 -2); no reader code changed. `make fmt-check` and `make lint` pass. The 14 flat-wire tests pass in debug, the new one included: it covers a stale projection body of 61 aggregates (126 JSON levels) and one at 127 levels, each refusing `stale_dependency` at `/identity_preimage/identity_projection/{host}/body/members` on a 256 KiB thread with no abort. `quire validate` and `coverage --strict` are unchanged (23 rows).

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | still-open | Owner decision: quire-walk is still PRIVATE under public IR, CG and QSL. It is the only open finding across SR-1236, SR-1237 and SR-1238 |
| FND-007 | fixed | 6877729. "Reading" now names the stale-projection comparison: `Value` `==` and `serde_json::to_value`, once per level, bounded by the parse limit. `first_difference` steps down iteratively and uses `Value` `==` at each step, which the sentence's "compares the `Value`s" covers. `tc_048_a_deep_stale_projection_body_is_refused_on_a_small_stack` backs it |
