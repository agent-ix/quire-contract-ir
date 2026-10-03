---
id: SR-837
title: "code review of PR 261 (IR-533 encode the v2 wire types through quire-canonical)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@82e60e791442a63426f7a739a8cb9256aa0aa8f8; Cargo.toml, Cargo.lock, deny.toml, Makefile, CLAUDE.md, crates/quire-contract-model/Cargo.toml, crates/quire-contract-model/src/checked_package/common.rs, crates/quire-contract-model/src/checked_package/shared.rs, crates/quire-contract-model/src/checked_package/v2/encode.rs, crates/quire-contract-model/src/checked_package/v2/identity.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs, crates/quire-contract-model/src/checked_package/v2/vocabulary.rs, tests/it/checked_package_v2_canonical_encoding.rs, tests/it/checked_package_v2_reader.rs, tests/it/main.rs, spec/checked_package/matrix/tests.md"
review_set: subset
---
# SR-837: code review of PR 261

## Summary

Ticket: IR-533. This is a code review with the rust-review lane folded in. It covers
`git diff origin/main...HEAD` at 82e60e791442a63426f7a739a8cb9256aa0aa8f8. The PR was cut
from a41c0e5f7360d37b8a3ce740108d8077329b36c8 (the spec merge, #258). Main is now
1042762d (#260), one commit ahead. The merge conflicts on one matrix row (FND-002).

What was checked, and what holds:

- **Dependency.** `quire-canonical` is a `branch = "main"` git dependency of the model crate
  and a root dev-dependency. Both carry `version = "=0.3.0"`. This is not an invented pin.
  IR's other first-party git dependencies use the same spelling (`ix-trace-rs` and
  `quire-verification-contracts`, both `version = "=0.1.0", git = ..., branch = "main"`).
  The repo's CLAUDE.md says package versions live in Cargo.toml and the lockfile. `version`
  is cargo's own field: it is checked against the fetched package and it is what
  `cargo publish` keeps. No finding.
- **Lock and deny.** Cargo.lock has one `quire-canonical` entry and one
  `quire-canonical-derive` entry, both from `git+https://github.com/agent-ix/quire-canonical?branch=main#59fe4f06370fbdbd8de4fa446ad78f3974926252`.
  `deny.toml` `allow-git` gains the source in the file's own format, sorted. I ran
  `make deny`: advisories, bans, licenses and sources are ok, and the one-copy gate passes.
- **Makefile `LOCAL_PATCHES` and the CLAUDE.md `use-local` line.** These were not asked
  for, but both follow the repo's convention. The Makefile comment says `use-local`
  "patches each first-party git dependency", and the CLAUDE.md line lists those
  dependencies. Without the entry, `make use-local` would leave quire-canonical on the
  remote. The derive crate needs its own entry because it comes from the same git source.
  I checked it: `make use-local` against a scratch siblings directory with quire-canonical
  at 59fe4f06 wrote one `[patch]` table with both crates, and passed the unused-patch check.
  `make use-remote` restored the lock. Keep both hunks.
- **`FixedShape`.** It is derived on exactly 25 structs and enums: the 24 that FR-038 names,
  plus `CheckedRevision`, as on main. The `closed_vocabulary!` macro derives it at depth 0
  for every closed vocabulary, which covers `CheckedSelectionRole` and
  `CheckedCapabilityDisposition`. None of the six `Value`-holding types derives it, and
  `const` negative assertions in the test enforce that.
- **`encode.rs`.**
  - Each hand-written `Encode` writes its members in serde's field order. The struct field
    names have no `rename`, `flatten` or `with`.
  - Every `Option` field carries `skip_serializing_if = "Option::is_none"`, and `present`
    omits it to match. Vectors are always written, including empty ones, as serde does.
  - `write_value` keeps an explicit `Vec<Open>` stack of slice and map iterators. It never
    calls itself.
  - `Writer::serialize` is used only for `FixedShape` members.
  - `i64` and `u64` go through `Writer::integer`, and everything else through
    `Writer::number`.
  - quire-canonical's `Writer::end_object` sorts members by UTF-16 code unit (`order.rs`),
    so the `Value` map's order does not matter.
- **Reader.** `require_canonical_bytes` runs inside `read_value`, after `strict_parse`
  (syntax, duplicate members, depth) and before `admit`. It goes through
  `encode::value_to_vec` with the read's byte limit as the ceiling. A mismatch or an
  encoder error refuses `noncanonical_wire` with no pointer.
- **Not using `quire_canonical::read`.** The coder's reason is sound. `read` keeps a
  number as the double its text denotes, so 18014398509481984 (2^54) would round-trip
  and pass. Going through `serde_json` keeps the integer kind for the `i64`/`u64` range.
  There is one hole past `u64::MAX` (FND-001).
- **`package_id`.** It is `quire_canonical::sha256` (no domain) under
  `Limits::new(limits.bytes)`. Every fixture's recorded id recomputes (AC-74 test and the
  31 reader tests). `CanonicalWriter` and `digest_json` remain, as IR-274 owns them. The
  `MalformedWire` mapping of an encoder error at recompute is unreachable: `validate` is
  only called from `read`, after intake.
- **Changed tests.** The reader test changes only the `2.0` case, which was
  `invalid_semantic_graph` and is now `noncanonical_wire`. The `1.5` case still asserts the
  integer refusal. The unit test `tc_048_number_tokens_...` changes as follows:
  - `2.0` and `u64::MAX` are now not canonical.
  - The literal classification now uses a direct serde parse, with the same four
    `is_literal_value` expectations.
  - The private-number case is unchanged.

  This is what the spec states (see FND-004 for one weaker oracle).
- **Tests and gate.**
  - `cargo test --test it checked_package_v2_canonical_encoding`: 14/14 pass in 0.30 s,
    including the 100,000-deep tests on 256 KiB threads.
  - Model unit tests: 101/101 pass.
  - Reader tests: 31/31 pass.
  - All `checked_package_v2` integration tests: 118/118 pass.
  - The coder's `make -k ci` log ends `head=82e60e791442a63426f7a739a8cb9256aa0aa8f8
    exit=2`, failing only `spec`, with 23 unbacked rows and 0 contradicted (main's strict
    baseline), in 85 s.
- **Mutation probes.** The permission layer refused source edits in the review worktree,
  so the coder's six mutation probes were not re-run. Instead, the oracle that kills each
  one was traced:
  - `null` for an absent `Option`: `assert_same_bytes` on fixtures with absent members,
    and the explicit `!contains("\"recursion_group\":null")`.
  - A dropped member: `assert_same_bytes`.
  - The reader back on `serde_json` bytes: the UTF-8-order case and the 2^53+1 case
    expect `noncanonical_wire`.
  - `sha256_with_domain`: the fixture admits and the AC-74 digest equality.
  - `i64` through `Writer::number`: the `-9007199254740993` case expects
    `IntegerMagnitudeAboveMaximum`.
  - Recursion over `Value`: the 100,000-deep encode on a 256 KiB thread.
- **No compatibility layer, no vendoring.** There is no copy of quire-canonical code or
  vectors, no wrapper, and no QSL dependency.
- **PR text.** The title carries no bare ticket id, and no short SHAs appear.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | An integer token past `u64::MAX` passes the canonical-bytes check. FR-038 says "An integer whose magnitude exceeds 2^53 anywhere in the document ... refuses `noncanonical_wire`". `serde_json` parses a token outside the `i64`/`u64` range as `f64`, `write_number` sends it to `Writer::number`, and ryu-js writes whole doubles below 1e21 as plain integers. So `100000000000000000000`, `-100000000000000000000` and `18446744073709552000` round-trip byte-for-byte and are not refused at intake. Reproduced with a probe on this head. With a stale id and a non-term body, the document refuses `stale_dependency` at `/package_id/digest`. As a `literal` value with a recomputed id, it refuses `invalid_semantic_graph` at `/semantic_graph/nodes/0/body`. On main, `serde_json` spells these `1e+20`/`1.8446744073709552e+19`, so they refused `noncanonical_wire`. The code a document gets for this input has changed against the spec, and an encoder-independent grammar refusal now decides it. Fix: in the reader's check, refuse a number that is neither `i64` nor `u64` but is integral with magnitude > 2^53 (or state in FR-038 which tokens count as integers), and add the case to the AC-79 test | crates/quire-contract-model/src/checked_package/v2/encode.rs:146, crates/quire-contract-model/src/checked_package/common.rs:261 |
| FND-002 | medium | The PR does not merge cleanly with main 1042762d (GitHub `mergeable: CONFLICTING`). #260 rewrote the FR-038 row of the checked-package matrix: it moved AC-73 into the implemented list ("AC-70 through AC-73") and deleted its planned sentence. This PR rewrites the same row to delete the AC-74..AC-80 planned sentence. To resolve, rebase and take main's row, then delete the AC-74..80 planned sentence, write "AC-70 through AC-80 implemented" (or "AC-70 through AC-73 and AC-74 through AC-80"), and keep the PR's added AC-74..80 evidence sentence. Rerun `make spec` | spec/checked_package/matrix/tests.md:15 |
| FND-003 | low | The PR's edit reformatted the existing `serde` line to `serde ={ version = "=1.0.228", features = ["derive"] }` (missing space). It is valid TOML, but it is a stray whitespace change, unlike every other manifest line | crates/quire-contract-model/Cargo.toml:32 |
| FND-004 | low | `tc_048_number_tokens_decode_identically_with_or_without_arbitrary_precision` now asserts only `canonical_value(..).is_ok() == canonical`. For `2.0` and `18446744073709551615` it accepts any refusal, not specifically `NoncanonicalWire`. Before the change it asserted the exact outcome (admitted). Assert `Err(ValidationFailure::refused_bytes(NoncanonicalWire))`, as the private-number case below it already does | crates/quire-contract-model/src/checked_package/common.rs:1406 |
| FND-005 | low | Process: the lead reports that the coder ran a stray no-op `sed -i_unused 's/x/x/' /dev/null`, which breaches the Edit-only rule. No file changed: the diff holds only the 16 expected files, and no `/dev/null_unused` exists. Recorded for the coder lane, with no code action | - |

## Verdict

The PR is sound in design and close to mergeable:

- The types take the paths FR-038 assigns.
- The `Value` writer is iterative.
- Bytes are identical to the `serde_json` route for every fixture and crafted value.
- `package_id` recomputes unchanged.
- The reader's check refuses exactly the three stated inputs within the `i64`/`u64` range.

Two medium findings block merge:

- FND-001 is a spec-contradicting gap for integer tokens past `u64::MAX`, which QSL is
  measured never to emit. A fix in this PR is preferred, being small. Alternatively, a spec
  clarification with an explicit decision.
- FND-002 is a matrix-row rebase conflict.

The three low findings are cleanup.

Verdict: changes requested (FND-001, FND-002); merge after the fix round and a clean
rebase.

## Dispositions

Round 1 at 0c2508e107b480d51e8ddb718c89d29048253823, rebased onto main
1042762d0d77e1774d6e3d1c5af1155917f8c8a1 (main has not moved since; GitHub reports
MERGEABLE). Only the delta was reviewed:

- The rebase commit e9c8838. `git range-diff` against 82e60e7 shows that only the FR-038
  matrix row changed.
- The fix commit 0c2508e.

FND-001. `require_canonical_bytes` now runs `holds_float_integer_past_2_pow_53` before
encoding. It walks the value with an explicit `Vec` stack and refuses `noncanonical_wire`
with no pointer for any number that is neither `i64` nor `u64` and is whole with
magnitude > 2^53.

- The new test `tc_048_an_integer_spelled_past_the_64_bit_range_is_noncanonical` covers
  100000000000000000000, its negative, 18446744073709552000 and -9223372036854777000. Each
  document also carries a stale id and a non-term body, so the canonical-bytes refusal has
  to win.
- `1.5` is left to later checks, and that is correct:
  - FR-038 changes exactly three inputs: an integer past 2^53, a whole float, and
    UTF-8-ordered names.
  - `1.5`'s RFC 8785 (ECMAScript) text is `1.5`, so it is canonical bytes.
  - FR-038 says a fractional literal "refuses today as a literal grammar defect once the
    document is canonical". The reader test still asserts the integer-grammar refusal for
    `1.5`.
- The check also refuses `1e+21`-style whole floats. That fits the whole-float bullet.
- Mutation: the check removed is exactly the pre-fix head 82e60e7. My round-0 probe on that
  head measured `stale_dependency` for these inputs, so the new test fails without the
  check. No source edit was attempted, because the permission layer had refused one in
  round 0.

FND-002. The rebase landed cleanly. Against main, only the FR-038 and TC-048 rows differ.
The AC-74..80 "planned" sentence is gone, the row reads "AC-70 through AC-80
implemented", and the evidence sentence is kept.

FND-003. The `serde` line is restored byte-for-byte. The manifest diff against main is now
only the three added quire-canonical lines.

FND-004. The unit test now asserts `Err(refused_bytes(NoncanonicalWire))` for `2.0` and
`18446744073709551615`, and `None` for the canonical cases.

FND-005. Acknowledged as process only. The rebase's range-diff shows only the intended
matrix-row change.

Gate and tests:

- The coder's log ends `head=0c2508e107b480d51e8ddb718c89d29048253823 exit=2`. Only
  `spec` fails, with 23 unbacked rows and 0 contradicted, which is main's baseline.
- I ran these myself and all pass: 15/15 encoding, 31/31 reader and 101/101 model unit
  tests, plus `make deny` with the one-copy gate.

No new findings.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 0c2508e107b480d51e8ddb718c89d29048253823 |
| FND-002 | fixed | e9c8838 (rebase onto 1042762d0d77e1774d6e3d1c5af1155917f8c8a1) |
| FND-003 | fixed | 0c2508e107b480d51e8ddb718c89d29048253823 |
| FND-004 | fixed | 0c2508e107b480d51e8ddb718c89d29048253823 |
| FND-005 | accepted-no-change | Process finding about the coder's tool use, acknowledged by the lead; no file was affected and the rebase diff holds only the intended matrix-row change |
