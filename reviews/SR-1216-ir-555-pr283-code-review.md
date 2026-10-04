---
id: SR-1216
title: "PR #283 IR-555 NumberOutOfRange mapping code and Rust review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@de9bf5c8de38d2b53fbf309e7ab3c7b2d0db3c7e; Cargo.lock, crates/quire-contract-model/src/checked_package/v2/model_members.rs, tests/it/checked_package_v2_model_members.rs (git diff origin/main...HEAD, base 543cd8a8b6a5fe2a1974617869824a75f6c38629)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: reviews
---
# SR-1216: PR #283 IR-555 NumberOutOfRange mapping code and Rust review

## Summary

Ticket: IR-555 (and IR-542). PR agent-ix/quire-contract-ir#283, head
de9bf5c8de38d2b53fbf309e7ab3c7b2d0db3c7e. Methods: code-review with the
rust-review lane folded in (repo conventions first).

`admit_document` now maps `quire_canonical::ReadError::NumberOutOfRange
{ pointer, lexeme, .. }` to `SelectionFailure::InexactNumber` (that is,
`noncanonical_wire` at the row's `digest`) with `document_pointer` taken from
the reader's pointer, before the digest comparison. The cause is
`inexact-integer` when `Spelling::of(lexeme).is_whole_past_2_pow_53()` and
`inexact-number` otherwise. There is no pre-scan. The lock moves
`quire-canonical` and `quire-canonical-derive` from b4bb97a to 5dc4e12 (two
`source` lines and nothing else). The manifests are unchanged
(`branch = "main"`), there is one copy of each in the lock, and no SHA is
recorded outside the lock.

Read against quire-canonical 5dc4e12 `src/read.rs`, read-only from the cargo
checkout. `ReadError` is `#[non_exhaustive]`, `Clone` and not `Copy`. The
parser returns `NumberOutOfRange` from the number scanner the moment a literal
parses to a non-finite `f64`, so the read stops at the first such number. An
underflow such as `1e-400` parses to `0.0`, which is finite, and is kept as a
`Number` node with its text.

I ran a throwaway probe test in my own worktree; it was reverted and not
pushed. The probe read a document holding each number below at `/package/ratio`
under its own digest:

| Lexeme | Result |
| --- | --- |
| `1e400`, `-1e400`, `1e309`, `1.7976931348623157e309`, `1.5e400`, `1` followed by 400 zeros | `noncanonical_wire`, `inexact-integer`, `/package/ratio` |
| `1e-400`, `-1e-400` | `noncanonical_wire`, `inexact-number`, `/package/ratio` (read as zero, refused through IR's text rule) |
| 400 nines followed by `.5` (non-whole, past the double range) | `noncanonical_wire`, `inexact-number`, `/package/ratio` |
| `{"a":[{"x/y~z":[0,{"q":[1,2,1e400]}]}],...}` | pointer `/a/0/x~1y~0z/1/q/2` (escaping and indices correct) |
| `{"b":0.1000000000000000000001,"a":[1e400],...}` | `/a/0`, `inexact-integer` (FR-038-AC-110 requires `/b`, `inexact-number`) |
| `{"b":9007199254740993,"a":[1e400],...}` | `/a/0` (FR-038's "first such number in document order" requires `/b`) |
| `{"b":1e400,"a":[0.1000000000000000000001],...}` | `/b`, `inexact-integer` (correct) |
| `{"a":1e400,` (truncated) and `{"a":1e400,"a":1}` (duplicate name) | `noncanonical_wire`/`inexact-integer` at `/a`, where any other malformed document is `byte-digest-mismatch` |

Consumers: the only use of `quire_canonical::read` or `ReadError` in this repo
is `admit_document`, plus one test that unwraps a read of valid bytes. There is
no use of `Malformed::NumberOutOfRange`. Under /home/peter/dev outside
quire-canonical itself and the IR worktrees, nothing references
`quire_canonical::read`, `ReadError` or `Malformed::NumberOutOfRange`. That
includes the quire-spec-language clone. The author's outside-consumer check
holds.

## Verdict

**REQUEST CHANGES.**

The mapping is correct for a document whose first inexact number is the
out-of-range one. That covers every case the new tests build. It runs before
the digest comparison, and the pointer is right, nesting and escaping included.
The lock change is minimal and `make deny` (one-copy) passes. `make fmt-check`,
`make lint`, `make test`, `make deny` and `make conformance-qspec` all pass on
de9bf5c (conformance was run against a throwaway quire-specification origin/main
worktree at 2f846f8).

The author's own report is confirmed (FND-001). Because `read` aborts at the
first out-of-range number, an earlier inexact number is never named. This is a
spec claim the code violates, and the PR flips AC-110 to implemented on top of
it.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `quire_canonical::read` stops at the first out-of-range number, so `admit_document` names it even when an earlier number in document order is inexact. Reproduced: `{"b":0.1000000000000000000001,"a":[1e400]}` refuses at `/a/0` with `inexact-integer`, where FR-038-AC-110 requires `/b` with `inexact-number`. `{"b":9007199254740993,"a":[1e400]}` refuses at `/a/0`, where FR-038's "first such number in document order" (and QSpec FR-272's `document_pointer` rule) requires `/b`. No pre-scan is allowed, so IR cannot fix this alone. It needs a `quire-canonical` change (keep reading and surface the out-of-range number as a node with its text, so IR's existing walker decides in document order) or a spec amendment. Until then AC-110 is not implemented. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:1063-1087 |
| FND-002 | medium | The `inexact-number` arm of the new `NumberOutOfRange` mapping is untested. Every new out-of-range lexeme (`1e400`, `-1e400`, `1e309`) is whole, so a mapping hard-coded to `Cause::InexactInteger` passes every test. The probe shows a non-whole out-of-range lexeme (400 nines then `.5`) reaches that arm and refuses `inexact-number`. That is correct per QSpec FR-272, but nothing pins it. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:1076-1080; tests/it/checked_package_v2_model_members.rs:752-791 |
| FND-003 | low | When `JsonPointer::parse` rejects the reader's pointer, the code falls back silently to `stale_dependency`/`byte-digest-mismatch`. That is unreachable under quire-canonical's documented RFC 6901 contract, and if it were ever reached it would report a false cause (digest mismatch for a document whose digest was never compared) and hide an upstream contract break. `JsonPointer::from_escaped` (pub(super), visible here) or a `debug_assert!` plus the parsed-or-root pointer would keep the cause honest. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:1071-1075 |
| FND-004 | low | The outcome for malformed JSON now depends on document order. Bytes that are not strict JSON have been `byte-digest-mismatch` (FR-038 prose; the code comment at 1058-1059). Now a truncated or duplicate-name document whose fault comes after an out-of-range number refuses `noncanonical_wire`/`inexact-integer` (probe: `{"a":1e400,` and `{"a":1e400,"a":1}`), while the same fault before the number stays `byte-digest-mismatch`. FR-038 does not state which wins, and no test pins either. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:1057-1086 |

## Dispositions

Round 1 was reviewed at 3b3cd146dbf7d2df988cd13c9a2e3daf248e7ef0. The delta
from de9bf5c touches only `model_members.rs` (one hunk), the new test, the
FR-038 note, TC-048, the FR-038 matrix row and `spec/tests.md`. `Cargo.lock`
and the manifests are as at de9bf5c. I ran `make fmt-check lint test` on
3b3cd14 and they pass. `quire coverage --strict` reports 23 unbacked rows, and
validate reports 1 grammar finding (FR-014).

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | deferred | IR-573. The behaviour is upstream (`quire-canonical` `read` aborts at the first out-of-range number) and cannot be fixed in IR without the forbidden pre-scan. 3b3cd14 keeps AC-110 partial and names the unmet "first in document order" clause in the FR-038 matrix row, TC-048 and `spec/tests.md`, with IR-573 in the matrix notes. The FR-038 note also records it as today's behaviour. |
| FND-002 | fixed | 3b3cd14. `tc_048_a_non_whole_number_past_the_double_range_refuses_inexact_number` reads 400 nines then `.5`, and its negative, at `/package/ratio` and expects a hand-written `inexact-number` refusal. I hard-coded `Cause::InexactInteger` in a throwaway copy: this test fails and the other two pass, so it is a real oracle for that arm. |
| FND-003 | fixed | 3b3cd14. `let document_pointer = JsonPointer::from_escaped(pointer);`. quire-canonical's `pointer()` already escapes `~` as `~0` and `/` as `~1` and writes decimal indices (round 0 probe: `/a/0/x~1y~0z/1/q/2`), and `from_escaped` wraps the text without re-escaping. There is no double-escape, no panic path and no false `byte-digest-mismatch` cause. |
| FND-004 | fixed | 3b3cd14. The FR-038 note now states that a document malformed after an out-of-range number refuses as that number does, while one malformed before it refuses `byte-digest-mismatch`, as today's behaviour and not a requirement. That resolves "unspecified"; no test pins it, which is acceptable for a stated non-requirement. |
