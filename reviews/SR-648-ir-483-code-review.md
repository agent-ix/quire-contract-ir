---
id: SR-648
title: "code review of PR 238 (IR-483 operation leaves counted per text leaf)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@b696d0e8363f01fa63de82f4cd6cd521c8d97943; crates/quire-contract-model/src/checked_package/v2/operations.rs"
review_set: base
---
# SR-648: code review of PR 238

## Summary

Ticket: IR-483. The PR removes the old check (`entry.leaves.is_some() && operation.leaves.is_empty()` refused `operation-law-missing`). In its place, `check_leaf_count` runs after the operand-family and mode-type checks. It counts the text leaves of the compared type with `text_leaf_count` and refuses only when fewer leaves are supplied than that count. Rust-review lane folded in.

What is right. For `operand:0` and `inner:0`, the walk matches QSpec FR-322 lines 174-179 and the upstream reference `collect_leaves` (quire-specification `tests/checked_package_v2.rs` 1467-1516). It follows aliases and non-population bounded domains, so `text_bounds` over text counts as a text leaf. It walks record fields, tuple positions and the inner type of option, sequence, set, bag and ordered_set. Enum, ordered enum, quantity, decimal, reference and population count 0, the same as upstream. Probes confirmed alias to text, `text_bounds` field, tuple plus option plus sequence (3 leaves), enum (0) and float plus text (1). The new order (leaves after `operator-ineligible`) is the order FR-322 lines 183-191 own. Previously the reader reported a missing leaf before an ill-typed operand, which was out of order. No existing test depended on the removed check, which had no test. The real QSL package `int_record_eq.json` is admitted by the head reader. Mutations "text counted 0" and "count off by one" each turn both refuse tests red.

Gate at head, run by the reviewer: `make fmt-check lint test corpus` exit 0, `make deny` advisories, bans, licenses and sources ok.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `result_inner` counts the text leaves of a `sequence` result as well. QSpec counts only a `set`, `bag` or `ordered_set` result, so a spec-valid `flatten`/`map`/`flat_map`/`convert` to a sequence of text with `leaves: []` is refused `operation-law-missing` | crates/quire-contract-model/src/checked_package/v2/operations.rs:1802-1806, 1708-1726 |
| FND-002 | high | `text_leaf_count` has no memo and no `WorkMeter` charge, so its cost grows exponentially over a shared record DAG. 15 graph nodes (12 levels, 4 fields each) take 50 s. At 16 levels and 10 fields the cost is about 10^16 calls, so a tiny hostile package hangs the reader | crates/quire-contract-model/src/checked_package/v2/operations.rs:1731-1780 |
| FND-003 | medium | "Undecided = no refusal" admits `leaves: []` for a type with text leaves. Examples: a recursive record with a text field, and an acyclic text leaf nested more than 16 deep. Upstream refuses these as `operator-ineligible` (cycle) or `operation-law-missing` (depth), and the old check refused both | crates/quire-contract-model/src/checked_package/v2/operations.rs:1738-1740, 1813 |

## Verdict

Not mergeable. FND-001 refuses spec-valid QSL output (the same class of bug IR-483 fixes, moved to sequence results). FND-002 is an unbounded-work path on untrusted input in a strict reader. FND-003 widens what the law-missing check misses beyond the stated "leaf paths/laws not compared" limit, although junk leaves already pass that check (a probe admitted 3 junk leaves over 3 text leaves).

Probe evidence (temporary unit tests in the review worktree, since reverted):

- `flatten` from `sequence<sequence<text>>` to `sequence<text>` with `[]`: refused `OperationLawMissing`. Upstream `expected_leaves` for a non-set result is `[]`, so upstream admits.
- Fan-out timing: width 2: 30 ms. Width 3: 2.0 s. Width 4: 49.7 s. Debug build.
- `option^18<text>` field with `[]`: admitted. Recursive `R {name: text, next: option<R>}` with `[]`: admitted.

Suggested fixes:

- FND-001: gate `result_inner` on a set, bag or ordered_set result, as upstream does.
- FND-002: memoize per type node, or charge the meter per visit.
- FND-003: refuse when the count is undecided, as upstream does with `operator-ineligible`, or at least keep refusing `[]`. Drop the arbitrary depth bound once memoized with a visiting set.
