---
id: SR-1156
title: "code review of PR 278 (IR-274 part B: v1 decimal-string wire and canonical objects through quire-canonical)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@297810203764f5cab680427ea79ff9da26956bf4; git diff origin/main...HEAD (base 4d705fdbdb705063a7119e6a7cc7ba48be23f57c): crates/quire-contract-model/src/{binding,canonical,conformance,decimal,expression,identity,lib,output_mapping,wire}.rs, schemas/contract-conformance-fixture-v1.schema.json, schemas/contract-package-reference-v1.schema.json, scripts/generate_conformance_corpus.py, corpus/contract-v0.1/**, tests/it/{canonical_v1_quire_canonical,canonicalization,checked_package_v2_identity_digests,conformance,identity,main}.rs; checked against quire-canonical b4bb97a5 (Cargo.lock) and read-only checkouts of quire-contract-codegen 37ff360, quire-spec-language eddbdc54, quire-driver c0b2f31, quire-analyze 588e05f, quire-protocol 0ba20ec"
review_set: subset
---
# SR-1156: code review of PR 278

## Summary

Ticket: IR-274 (code change B). Code review with the Rust-review lane folded in.
The PR moves the five v1 canonical object kinds onto `quire_canonical::Encode`
(explicit-stack walks for `ValueType`, `Expression`, `CollectionType` and
`ReferenceBody`, plain writer walks for the fixed-depth projections), deletes
`CanonicalWriter` and `canonical_envelope_bytes`, spells the eight integer
members as decimal strings through a new `IntegerString` wire type, bounds
revisions and byte offsets at 2^53 at construction, hashes the bound identity
envelope with `quire_canonical::sha256`, and re-records the corpus.

What was measured at the reviewed sha, in a throwaway worktree with its own
target dir:

- `make test`: 307 integration, 134 model unit and 7 doc tests pass.
  `make fmt-check` and both `make lint` clippy lanes are clean.
- `make corpus`: exit 0, 107 rows, every one `match`.
- `make spec`: validate passes with 1 grammar finding (FR-014, the baseline);
  `quire coverage --strict` reports 23 unbacked rows (the baseline).
- Corpus, checked by my own script against `origin/main`: each of the 47
  modified canonical files equals its base bytes with exactly the eight
  members' JSON numbers rewritten as strings, and nothing else; the 28
  unchanged files hold none of the eight members as a number; each of the 59
  modified inputs equals its base JSON with only those members respelled; the
  30 modified expectations change only `digest` values (47 lines). The old
  bytes came from the deleted encoder, so this is an independent oracle for
  the re-recording.
- The generator run twice into scratch directories produces byte-identical
  trees, and both equal the committed corpus. Replacing one canonical file
  with its base version makes the runner report `canonical_bytes` mismatch and
  exit 1, so a stale file is refused.
- `IntegerString`: grammar check is byte-based, so `-0`, `+1`, `01`, `""`,
  `" 1"`, `1e3`, `1.0` and non-ASCII digits are refused; only `visit_str` is
  implemented, so numbers and booleans are refused by the type
  (`invalid_wire_format`); range is decided after the grammar
  (`invalid_numeric_bounds`, path `type.integer.bounds`,
  `type.rational.bounds` or `expression.integer_literal.value`). No reader of
  the number spelling remains anywhere in the crate (the expression wire is
  the only v1 reader of these members).
- No recursion is left in the canonical path: `walk` pops an explicit
  `Vec<Step>`; the only `writer.serialize` calls are on `FixedShape` leaves of
  fixed depth. Member order is left to the writer, and the byte-equality tests
  against hand-written RFC 8785 strings (five kinds, plus zero and rational
  cases) confirm it.
- Digest: SHA-256 over `quire-contract-ir\0<profile>\0<kind>\0` and the bytes
  of the one `quire_canonical::to_vec` call, not `sha256_with_domain`; the
  test also shows it differs from both `sha256_with_domain` and bare `sha256`.
  The bound identity digest is a bare `quire_canonical::sha256` of a derived
  `FixedShape` envelope, and `tc_035_bound_digest_matches_an_independently_written_canonical_envelope`
  compares it with SHA-256 over hand-written bytes, so the "no prefix" choice
  is byte-identical to the old helper.
- `binding.rs` now decodes the projection's package with
  `ContractPackage::from_json_value`: the same preflight, `WirePackage` and
  `validate`, and the outer `from_json_bytes` already enforces the file-byte
  and `MAX_WIRE_JSON_DEPTH` bounds on the whole projection, so the inner
  depth check dropped from the package path is subsumed (the package is one
  level deeper, so the new path is at least as strict). Only the serde
  message text loses its line/column.
- Scans: no `CanonicalWriter`, `canonical_envelope_bytes`, `digest_json`,
  `value_to_vec` or `serde_json_canonicalizer` anywhere in the crate, and no
  `serde_json::to_vec`/`to_value` in `canonical.rs`, `binding.rs` or
  `output_mapping.rs` production code (my grep agrees with the TC-017 and
  TC-048 scans). No hand-written `impl FixedShape` or `const DEPTH`; the new
  `Encode` impls are what FR-038-AC-80/92 allow. `serde_json_canonicalizer`
  is in `Cargo.lock` only through `quire-verification-contracts`; no manifest
  names it. `Cargo.lock`, manifests, `deny.toml` and `.github` are untouched.
- Public API: `CanonicalBody` loses `canonical_body_value` and gains an
  `Encode` supertrait; `FixedShape` and `Encode` impls are added to public
  model types; the public `Serialize` of `IntegerType`, `RationalType` and
  `ExpressionKind` now emits strings. No consumer in the five sibling repos
  implements `CanonicalBody` or reads that `Serialize` output.
- External consumers: QSL `src/lowering/wire.rs` writes `minimum`, `maximum`
  and `IntegerLiteral.value` as numbers and feeds `ir::BoundPackage::from_json_bytes`
  (`src/lowering.rs:494`), as the PR says. quire-contract-codegen also depends
  on IR `branch = "main"` and builds executable projections with the number
  spelling (FND-001). quire-driver, quire-analyze and quire-protocol pin older
  IR revisions and are not affected until they move.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The PR body names only QSL as a downstream that breaks. quire-contract-codegen depends on `quire-contract-ir` `branch = "main"` and builds executable projections with the number spelling of the eight members in `src/kani_obligations.rs` (`render_probe_package`, a unit-test fixture fed to `BoundPackage::from_json_bytes`), `tests/common/withdraw_fixture.rs`, `tests/it/bound_generation.rs`, `tests/it/bound_strategy_generation.rs`, `tests/it/kani_witness_join.rs` and `tests/it/kani_obligations.rs`. After its next lock update these fixtures are refused (`invalid_corpus` from the projection schema or `invalid_wire_format`). The planner allows the break, but the PR body must say so | PR body "Not in this repo"; quire-contract-codegen src/kani_obligations.rs:2550-2578, tests/common/withdraw_fixture.rs:122, tests/it/bound_strategy_generation.rs:74-137,393-467 |
| FND-002 | low | `canonicalize` (and the bound-identity hash in `binding.rs`) maps every `quire_canonical::Error` to `canonicalization_resource_exhausted` with the message "canonical byte allocation exceeded available resources". The code is the only one STD-001 registers for this path, so the code is right, but a `Protocol` or `Internal` error from a bug in one of the new hand-written `Encode` walks would be reported as a resource limit with no trace. Match `Error::Limit(_)` for the limit message and give the unreachable variants their own message (and a `debug_assert!`) so a walk bug is not disguised as exhaustion | crates/quire-contract-model/src/canonical.rs:385-420; crates/quire-contract-model/src/binding.rs:403-411 |
| FND-003 | low | A successful `document_json` input of the expression operation is never checked against the fixture schema: the `expressionOperationInput` `oneOf` branch admits any string, and `validate_successful_package_schema` returns early for `Expression`. Successful package `document_json` inputs are re-validated against the package schema. So an expression fixture wrapped in `document_json` that the decoder accepts but the schema would refuse still runs as a `match`, which weakens the "valid-operation input is checked against the schema first" contract FR-020 states. None of the four new fixtures is affected (all are refusals) | crates/quire-contract-model/src/conformance.rs:776-797; schemas/contract-conformance-fixture-v1.schema.json:426-436 |

## Verdict

Approve after FND-001 (a PR-body correction; no code change). The code is
sound: the eight members decode only from in-grammar strings and refuse
numbers by type with no compatibility reader, every canonical byte comes from
one `quire_canonical::to_vec` call under the caller's limit, the digest is
the explicit domain prefix, the walks hold no recursion, and the corpus
re-recording is exactly the respelling it claims (all 47 files verified
against base). FND-002 and FND-003 are cheap and can ride the same round or a
follow-up.

## Dispositions

Round 1, reviewed at e8d73a5a74888339dbe09274de4ddda9f50e3768 (fix commits
bc0b172 and e8d73a5 since 2978102). Re-measured in a fresh worktree: `make test`
308 integration, 135 model unit and 7 doc tests pass; `make fmt-check` and
`make lint` clean; `make corpus` exit 0 with 107 `match` and no mismatch;
`make spec` at the baseline (grammar 1, strict 23).

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | PR body as of head e8d73a5 (a body edit, not a commit): a "Breaks downstream after merge" section names QSL `src/lowering/wire.rs` and the six quire-contract-codegen files. A read-only re-grep of codegen 37ff360 finds exactly those six as number-wire producers; the other hits are codegen's own schemas and `tests/exact_scalar_support/agreement.rs`, which already writes strings |
| FND-002 | fixed | bc0b172: `canonical::encoder_refusal` keeps `canonicalization_resource_exhausted` (the one code STD-001 registers) and gives `Error::Limit`/`Error::Allocation` the resource message and every other variant "canonical encoder failed unexpectedly: {error}"; `canonicalize` and `binding.rs` both route through it; unit test `tc_017_a_limit_and_an_encoder_fault_share_the_code_but_not_the_message` pins both messages |
| FND-003 | fixed | bc0b172: a successful expression `document_json` input is parsed and validated against a newly compiled `expressionInput` named schema; `tc_018_a_successful_expression_document_is_checked_against_the_schema` shows a control document matches and the same document with an unknown expression member (which the decoder ignores) makes the run `invalid_corpus`. The corpus is unchanged and still runs 107 `match` |
