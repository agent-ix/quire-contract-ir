---
id: SR-1031
title: "gap analysis of PR 267: amended FR-038-AC-95 against PR 266's lowering code and quire-canonical's refusal, with oracle strength"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@c4f2a24706520a10bbaeb165ae9eef1a14624c3d; FR-038-AC-95 and FR-038 'Every identity digest is computed through quire-canonical', read against crates/quire-contract-model/src/checked_package/v2/lower.rs and v2/lower/ceiling_tests.rs at PR 266 head 476e9b418e69aaaf7e796e3923e3d8321a812d50, checked_package/shared.rs at main ebea678, and agent-ix/quire-canonical@b4bb97a5fe0a946e9d980e6466c7ecf95c6e62f1 src/error.rs, src/writer.rs"
review_set: subset
---
# SR-1031: gap analysis of PR 267

## Summary

Ticket: IR-274 (planner ticket IR-546). Plan completion: not assessed. AC-95
is planned (🚧) and nothing traces to its amended clauses yet. So this pass
checks two things: whether the planned rows can be satisfied by the code they
target, and whether their oracles can fail.

What the code does at PR 266's head:

- `lower_on_stack` lowers each request independently through `lower_one`,
  then assembles the package from the `lowered` records alone. If
  `encode_package` returns `None`, it overwrites every record with
  `failed_records`, using `limit: work_limit` and `consumed: 0`. The package
  still holds its lowered nodes and dependency nodes.
- A node over the ceiling fails inside `lower_one`, through
  `identify_node(&preimage, ceiling)` returning `None`, recorded with the work
  limit.
- The tests are per-function seams in `ceiling_tests.rs`: `identify_node` and
  `encode_package` at length and at length minus one.

Decision (c) defines "sibling records unchanged" as "the records the same call
returns with the failed node's request removed". That definition is consistent
with how records are produced:

- each record comes from `lower_one` and depends on nothing but its own
  request;
- a failed node never enters the package's `lowered` set, so the package with
  or without its request is the same.

So it is testable by comparing two whole calls. The gaps are in which seam
AC-95 names, and in how strong its oracles are.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-95's node step is ambiguous about its seam, and its sibling check can pass vacuously. "Lowers it" at a ceiling equal to the node preimage's length holds only at a per-node seam (`identify_node`). Through a whole call at that ceiling, the package holding that lowered node is longer than the node's preimage (it adds `ir_id`, `source_map`, `node_tag` and the package header), so the package is over the ceiling and every record is `failed`. "Sibling records unchanged", by decision (c)'s definition, needs a whole call. As written, the check also passes when the package is over the ceiling in both calls, because every sibling is then `failed` both times. Name the two seams. Also require that, at the ceiling used for the sibling check, the siblings are `lowered`. That means a fixture where the siblings' package fits under a ceiling below the failed node's preimage length. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1598, 470-473; crates/quire-contract-model/src/checked_package/v2/lower.rs:356-380 |
| FND-002 | medium | The `bytes` value of `consumed` is under-specified, and AC-95's oracle cannot pin it. quire-canonical's `LimitExceeded.required` is "the output length including the bytes being written". That is the length of the prefix written when the encode refused, not the preimage's or the package's canonical length. The prose analogy "as an `incomplete` for `bytes` reports the document's length (FR-038-AC-26)" invites the full-length reading, which SR-1016 FND-002 also suggested. AC-95 asserts only "`consumed` greater than it", so `limit + 1` or `u64::MAX` would pass. State that `consumed` is the encoder's `required`, which is greater than `limit` and at most the canonical length. Have AC-95 compare it with the `required` of encoding the same preimage under the same limit, or at least bound it above by the canonical length. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:459-464, 1598; quire-canonical src/error.rs:41-47 |
| FND-003 | low | "`consumed` ... always greater than `limit`" holds only for the `CanonicalBytes` refusal. quire-canonical can also refuse with `LimitKind::ObjectBytes`, whose bound is `u32::MAX`, when an object's buffer passes 4 GiB. With a retained limit above 4 GiB, that refusal's `required` can be at or below `limit`. An allocation failure has no `required` at all. Say which refusals map to a `bytes` failure, or say that only the canonical-bytes refusal reaches this path. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:461-466; quire-canonical src/writer.rs:768-776 |

## Verdict

Changes requested, for FND-001 and FND-002. The code still has everything
decisions (a) and (b) need: closed `CheckedPackageLimit::{Bytes, Work}`, and a
`Failed` record with no kind today. The rest is real planned work: a kind
field, a `Result` from `identify_node` and `encode_package` so the code can
read `required`, and an empty `lowered` and `dependencies` when the package is
over the ceiling. That last item is the same defect SR-1014 FND-002 raised on
PR 266. The work-limit claim "consumed > limit" holds in `lower_one`, since
`failed(work)` fires only at `work > work_limit`.

## New findings (disposition pass 1)

Reviewed at ec21b9c00b9e3838115e19dc8e35eb6ee4313a33 (delta c4f2a24..ec21b9c),
against PR 266 head 249acbcd028536586d0e8162b1a15ddfc59e75a9.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | high | The new rule cannot be met by `lower` as specified. FR-038 says that for any encoder refusal other than canonical bytes (an unencodable number, the object-buffer bound, an allocation failure), "lowering returns it as the encoder's own refusal and never panics on it". But `lower` has no refusal channel. At 249acbc it returns `CompleteLoweringResultV2` unconditionally. FR-035 Outputs requires "exactly one record for every requested item, drawn from the closed seven-member vocabulary". FR-038-AC-7 says "no eighth kind is reachable". So returning the encoder's refusal needs a new error path that the spec does not define, and would contradict FR-035 Outputs. PR 266 does the opposite of the new text: `required_bytes` maps any `Error::Limit`, including `ObjectBytes`, to a `bytes` failure with its `required`, and maps every other error to `ceiling + 1`. That is the "invented `consumed`" the spec now forbids. FR-038:397-399 also still says "the only refusals an encode returns are the byte ceiling and a number". This is not the reader-side pattern of SR-1014 FND-004: those refusals map to existing reader refusal codes, and lowering has no such code. Pick one rule and amend both PRs to it. Option 1: fold the other refusals into an existing record kind and say which. Option 2: state that `lower` returns a `Result`, and amend FR-035 Outputs, FR-038-AC-7 and the FR-019 surface to match. Option 3: keep #266's fold and document `consumed` as a floor of `limit + 1` for non-canonical-bytes refusals. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:473-479, 397-399; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:32-36; crates/quire-contract-model/src/checked_package/v2/lower.rs:284-289, 373-378 |
| FND-005 | medium | The claim "It is a prefix length: ... and not the full length itself" is false at the AC's own boundary. When quire-canonical closes the outermost object, it counts that object's commas and closing brace in one final `count`. So at a ceiling one byte under the canonical length, the overflow happens at that final count, and `required` equals the full canonical length, which is `limit + 1`. As a result, AC-95's equality checks on the `identify_node` seam and on the package case, both at one byte under, cannot tell `required` apart from `limit + 1` or from the full length. A `consumed = ceiling + 1` mutant, which is #266's fallback, passes them. Only the whole-call node clause can tell them apart, and only when its ceiling is well below the preimage's length. Fix: say "at most the full length, and equal to it one byte under". Then require one equality check at a ceiling below `length - 1` that ends inside the encoding, for example half the canonical length. PR 266's whole-call node test asserts only `consumed > limit`, so it still owes the "as above" equality. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:468-472, 1611; quire-canonical src/writer.rs:600-602 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ec21b9c. AC-95 now names `identify_node` for the at-length and one-under check, and a whole call of `lower` for the sibling check, "with the siblings `lowered` in both calls, so the comparison cannot pass because every record failed". PR 266 builds `CheckedPackageV2` directly with its `bytes` field, so `lower` can be called at any retained limit. Its test at 249acbc compares against the call with the request removed, and asserts that `y` and `z` are lowered. |
| FND-002 | fixed | ec21b9c. The prose now names `consumed` as `LimitKind::CanonicalBytes`'s `required`, a prefix length. AC-95 asserts that it equals the `required` of the same preimage or package under the same limit, which is greater than `limit` and at most the canonical length. The remaining weakness at the one-under boundary is new finding FND-005. |
| FND-003 | fixed | ec21b9c. Only the canonical-bytes refusal is a `bytes` failure, so `consumed > limit` holds wherever it is claimed. What happens to the other refusals is new finding FND-004. |

## New findings (disposition pass 2)

Reviewed at ac907bbe858ba81730ae3e214e0f66988e159420 (delta ec21b9c..ac907bb),
against PR 266 head 249acbcd028536586d0e8162b1a15ddfc59e75a9 and
quire-canonical b4bb97a5fe0a946e9d980e6466c7ecf95c6e62f1.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | The refusal list in FR-038:398-401 is closed but incomplete, and `limit + 1` is undefined at the top of the range. FR-038 says "an encode returns refusals only for the byte ceiling, a number ..., an object buffer ... and an allocation failure". quire-canonical's `Error` also has `NonStringMemberName`, `DuplicateMemberName`, `SerdeJsonPrivateToken`, `Sink`, `Protocol`, `Internal` and `Serialize`, and `Error` is `#[non_exhaustive]`. The fold sentence ("Every encoder refusal during lowering is therefore a `failed` record for `bytes`") already covers all of them, so behaviour is fully specified, but the "only" list is false. Separately, a retained limit is any `u64` (`CheckedPackageReadLimits::bytes` has no cap). At `limit = u64::MAX`, `limit + 1` does not exist, and #266's `ceiling.saturating_add(1)` gives `consumed == limit`, against "never at or below `limit`". Reword the list as "any other refusal", and state the `u64::MAX` case: exclude it, or say `consumed` saturates. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:398-401, 477-488 |
| FND-007 | low | The two new AC-95 checks depend on conditions the AC does not state. (1) Half length: quire-canonical writes many one-byte tokens on their own (a string's two quotes at writer.rs:508-510, `:` at 547, array `,` at 441, `{` at 747). If the byte at half the length ends such a write, or ends an escape run, then `required` is `limit + 1`. So "neither `limit + 1` nor the full length" holds for some preimages and not others. (2) Integer past 2^53: the ceiling must be large enough that the encoder reaches the number before the byte ceiling refuses, and below `u64::MAX`. Otherwise the refusal is canonical-bytes, or `limit + 1` overflows. Both are satisfiable by choosing the fixture and ceiling, and the `identify_node` seam can take an in-memory `LoweredNodePreimage`, as #266's `ceiling_tests.rs` already does. State the conditions: a ceiling that ends inside a string run of at least two bytes, and a ceiling at or above the prefix up to the number. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1620 |

## Dispositions (round 2)

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-004 | fixed | ac907bb. Every encoder refusal during lowering is now a `failed` record for `bytes`: `consumed` is the encoder's `required` for the canonical-bytes refusal and `limit + 1` for every other refusal. That adds no record kind and no `Result`, so it is consistent with FR-035 Outputs (one record per request from the closed seven), FR-038-AC-7 and FR-019. FR-038:398-401 now names the fold too. I measured the unreachability claims. The reader refuses an integer past 2^53 in a body (AC-79, implemented under IR-533). `bounded()` defaults `bytes` to 1 << 20. The object bound is `u32::MAX`, and since quire-canonical counts bytes before buffering them, the buffer is never longer than `limit`; so a retained limit of `u32::MAX` or less excludes the object-bound refusal, as stated. Allocation goes through `try_reserve` and fails only on the host. Remaining gap in #266 (not this PR): `required_bytes` maps any `Error::Limit`, `ObjectBytes` included, to its `required`; the spec says `ObjectBytes` gives `limit + 1`. Narrow that arm to `LimitKind::CanonicalBytes`. |
| FND-005 | fixed | ac907bb. The wording now says `required` is at most the full length, and equals it (and `limit + 1`) one byte under the full length. I re-verified this at quire-canonical b4bb97a writer.rs:600-602: `count(member_count.max(1))` counts the outermost object's commas and closing brace in one final step. AC-95 adds an equality check at half the preimage length and a `limit + 1` check for an in-memory integer past 2^53, so a uniform `ceiling + 1` mutant and a uniform `required` mutant are both caught. The conditions those checks need are new finding FND-007. |

## Dispositions (round 3)

Reviewed at 0f2a8ddda23cab5d09b91c1c0cffc01216abbf14 (delta ac907bb..0f2a8dd).
No new findings this round.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-006 | fixed | 0f2a8dd. FR-038:398-402 now reads "the byte ceiling and ... any other cause `quire-canonical` reports (... and its other errors, which it may extend)", which matches the `#[non_exhaustive]` `Error` at b4bb97a. `consumed` is `limit + 1`, "saturating at `u64::MAX`", which matches #266's `ceiling.saturating_add(1)` at 249acbc. The exception claim is exact. At `bound = u64::MAX`, `Writer::count` computes `checked_add(..).unwrap_or(u64::MAX)` and then tests `required > bound`, which is never true, so no canonical-bytes refusal can happen there, and any other refusal records `consumed == limit == u64::MAX`. AC-95 now says "unless the limit is `u64::MAX`". |
| FND-007 | fixed | 0f2a8dd. AC-95 and TC-048 now require a ceiling "chosen for the fixture inside a string value of at least two bytes (not at a byte the encoder writes alone ... nor inside an escape run)". quire-canonical b4bb97a writes a string's quotes on their own (writer.rs:508-510) and the unescaped stretches and escape sequences as separate writes (escape.rs:44-53). So a mid-run ceiling gives a `required` above `limit + 1`. Because the AC also asserts the outcome ("neither `limit + 1` nor the full length"), a fixture where `limit + 1` is the last byte of a stretch fails visibly rather than vacuously. The 2^53 check now states "a ceiling at or above the length of the encoding up to that number and below `u64::MAX`", and says the in-memory preimage skips the reader. A ceiling such as the 1 MiB default meets that. The check can be implemented through `identify_node` with an in-memory `LoweredNodePreimage`, as #266's `ceiling_tests.rs` builds one, and #266 maps the number refusal to `ceiling + 1`. |
