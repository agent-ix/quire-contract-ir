---
id: SR-999
title: "spec review of PR 264 (identity digests and v1 integers through quire-canonical)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@184e6668bb8722f03c68a6b9924586593233118d; git diff origin/main...HEAD (base 7d7716d94c5596058485e24a5bd6310ee10f63d9, current main, merges clean): schemas/contract-conformance-fixture-v1.schema.json, schemas/contract-package-reference-v1.schema.json, spec/model (FR-013, FR-016, FR-019), spec/core (FR-011, FR-012), spec/conformance (FR-020), spec/checked_package (FR-038, TC-048), spec/output_mapping (FR-032, FR-033, FR-034, STD-003), spec/assurance (AD-004, AD-005), the five matrices and spec/tests.md; checked against crates/quire-contract-model/src at the same sha, quire-canonical origin/main 59fe4f06, quire-spec-language origin/main and PR 613 head (FR-056), quire-specification origin/main (IntegerString, FR-322)"
review_set: base
---
# SR-999: spec review of PR 264

## Summary

Ticket: IR-274. Base and correctness review of a spec PR that moves every IR
identity digest and the v1 canonical profile onto `quire-canonical`, spells the
eight v1 integer members as QSpec `IntegerString`, and meters the three
output-mapping identity steps under `maximum_request_bytes`.

Measured at the reviewed sha:

- `make corpus` fails: `invalid_corpus` at `fixtures.expression-arity.input`,
  "instance does not match schema". `cargo test --workspace --all-targets`
  fails 26 tests: `tc_018` (4), `tc_035` (5) and `tc_043` (17). The same
  tests pass at origin/main and pass at the PR head once `schemas/` is restored
  from origin/main, so the two schema files alone cause every failure. The
  runner validates every fixture input against the fixture schema
  (`conformance.rs:570-574`) and the executable-projection binder validates
  against both schemas via `include_str!` (`binding.rs:400-410`). The repo's CI
  workflow is `workflow_dispatch` only, so the PR's checks show CLA alone.
- quire-canonical 59fe4f06: the integer bound is inclusive of 2^53
  (`number.rs` `MAX_EXACT_INTEGER_MAGNITUDE = 1 << 53`, refuse when
  `magnitude > bound`), so 9007199254740992 admits and 9007199254740993
  refuses, as the PR states. `sha256_with_domain` hashes
  `u64::to_be_bytes(len)` then the label (`lib.rs:240-253`), so FR-016's
  explicit prefix is right. There is no `serde_json` feature (only
  `test-preserve-order`) and no `impl Encode for serde_json::Value`; no PR is
  open for one. `quire_canonical::read` already yields a `Document` whose
  `Number` keeps its source text (`read.rs:196-212`) and that implements
  `Encode` iteratively.
- QSpec `IntegerString` is `^(0|-?[1-9][0-9]*)$` (checked-package-v2
  `schema.json:81`), as the PR states.
- Code facts confirmed true: v1 integer members are `i64`, with
  `maximum_denominator: u64` limited to `i64::MAX` (`expression.rs:56-130`).
  The v1 canonical path is `serde_json::to_value`, then source stripping and set
  sorting, then `CanonicalWriter` (`canonical.rs:476-700`). `digest_json` has five
  production call sites. `package_id` is already `quire_canonical::sha256`
  (`v2/mod.rs:759`). The lowered package bytes are `serde_json::to_vec`
  (`lower.rs:306`). The three output-mapping steps pass `u64::MAX`
  (`output_mapping.rs:1960, 2270, 2578`). AD-005's call-site inventory is
  accurate apart from FND-008.
- Corpus impact, not stated in the PR: 47 of 75 canonical files and 59 of 99
  inputs hold one of the eight members as a number. No expectation file does.
- Correct as stated (no finding): FR-011-AC-3 and FR-012-AC-6 amendments
  (`positive_revision!`, `SourceLocation::new`). FR-016-AC-5 and AC-8. FR-019's
  serialized surface. FR-038-AC-89, AC-90 and AC-92. FR-033-AC-6.
  FR-034-AC-7. STD-003's row. AD-004 and AD-005. The v1 depth limits (576/256)
  and the v2 16,384 deviation are untouched. #243's dropped items are moot:
  there is no 576 reader ceiling, the spec says "no encoder stringifies", and
  with strings no identity encode refuses for an integer. AC numbering 89 to 94
  follows #263's AC-88.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The PR body says it is spec only and changes no code, but the two schema files are runtime inputs: the corpus runner validates every input against the fixture schema and the executable-projection binder compiles both in with `include_str!`. At the reviewed sha `make corpus` fails (`invalid_corpus`, expression-arity input) and 26 tests fail (tc_018 x4, tc_035 x5, tc_043 x17); with origin/main's schemas restored all pass. Merging breaks `make ci` on main. The schema edits belong with code PR B (wire change plus corpus regeneration) | schemas/contract-conformance-fixture-v1.schema.json:21-34, 187-192, 292-293, 320; schemas/contract-package-reference-v1.schema.json:15-29 |
| FND-002 | high | The new refusal of a selected model document that holds an integer past 2^53 (`noncanonical_wire` with `document_pointer`) contradicts QSL FR-056 at origin/main and at PR 613 head (lines 101-106): such an integer is encoded for the digest as the nearest double, and declarations read a 64-bit integer exactly. QSpec has no such refusal, so the prose claim that it is "the code QSpec and quire-spec-language use" is unsupported. IR today reads `Int[lo, hi]` bounds as exact `i64` numbers (`model_members.rs:1014`). Such a document, admitted by IR today and by QSL, would be refused. The relayed claim that QSL FR-056 now matches was not true at either measured ref | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:484-518 |
| FND-003 | high | The planner rule (relayed) is decided on the number's text before any rounding: exponent and past-u64 spellings such as `1e20` and `9.007199254740993e15` refuse, and +/-2^53 admits. The PR words it as "a number denoting a whole number", digested "over the document read as a `Value`". The document is parsed by `strict_json_value` into a serde_json `Value`, which keeps no text, and `9.007199254740993e15` parses to exactly 2^53, so it admits. AC-93 has no exponent-form vector and does not say "decided on the text". `quire_canonical::read`'s `Number::text` gives the text route | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1414 |
| FND-004 | high | FR-038-AC-94 returns `incomplete` for `bytes` at `/lock/model_selections/0/digest`. Merged FR-038-AC-26 says the byte limit's refusal "carries none" (no pointer). The two ACs contradict each other. The pointer convention also differs from AC-30, which reports the same document's `work` charge at the row `/lock/model_selections/<i>` | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1415, 1356, 1360 |
| FND-005 | medium | "Canonical encoding of the wire types" says `quire-canonical` "provides the iterative `Encode` for `serde_json::Value`, behind its `serde_json` feature", and the Dependencies paragraph lists it as owned. At quire-canonical 59fe4f06 neither the feature nor the impl exists, and no PR is open. FR-038-AC-91 requires the feature enabled. The prose states as fact a dependency that does not exist, names no owning ticket, and the matrix says AC-77/AC-78 "keep verifying" while their evidence is IR's own walker (`encode.rs` `value_to_vec`), which the amended prose says does not exist | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:334-337, 382-390, 1421-1426 |
| FND-006 | medium | Lowering encodes under the byte limit the admitted package retains: a node over it is `failed`, and a lowered package over it is `failed` for every requested record. No AC states this. The planner (relayed) requires one. It is testable: through a unit seam that takes a ceiling, as FR-034-AC-6 does for the package identity step, or by reading a package at `limits.bytes` equal to its length when its lowered `ContractPackage` is larger. The retained limit is also new state on the admitted package, and it is not named in FR-019. Today `lower.rs:306` `.expect`s this encode | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:430-439 |
| FND-007 | low | FR-013 says an in-grammar string outside the member's range refuses `invalid_numeric_bounds` "as a wider integer did before". For the six `i64` members that is false: a JSON number past `i64` fails serde decode today and is `invalid_wire_format` (`wire.rs:586`, `wire_type_error`). Only `maximum_denominator` past `i64::MAX` gave `invalid_numeric_bounds` (`expression.rs:118`). This is a behaviour change and should be stated as one | spec/model/functional/FR-013-type-system.md:55-57 |
| FND-008 | low | The inventory of serde_json canonical encodes that FR-038 and AD-005 retire omits `v2/identity.rs:716`, which orders a dimension preimage's terms by `serde_json::to_vec(term)` bytes. That is a canonical encode on the nominal-identity check. FR-038-AC-91's enumerated "digest path" list does not cover it, so "no encoder of this repository's own remains" can pass with it still in place | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1412 |
| FND-009 | low | FR-016 says the change alters "the digest of every v1 canonical object". Only objects holding one of the eight members change: 47 of 75 corpus canonical files and 59 of 99 inputs. The corpus impact is not stated anywhere in the PR | spec/model/functional/FR-016-canonicalization-digests.md:69-73 |
| FND-010 | low | `maximum_request_bytes` now also bounds record and package identity material, which is mapper-supplied. A request admitted at exactly its limit (the existing exact-limit test sizes it that way) can then refuse at `record.identity` or `package.identity`. FR-032's limit statement and STD-003's condition name the new paths, but neither says the request limit now bounds mapping output material too | spec/output_mapping/functional/FR-034-assemble-output-package-atomically.md:58-74 |

## Verdict

Changes needed. The direction matches the IR-274 rulings and most amended
statements are true of the code. Four highs block merge:

- FND-001: move the schema edits to code PR B with the corpus regeneration, and
  keep FR-020-AC-3 planned. Or regenerate the corpus here, which this PR does
  not.
- FND-002 and FND-003: reconcile with QSL FR-056 before merging. Either QSL
  adopts the refusal and the text-based decision, or IR digests the double as
  QSL does. Then state the text-based decision and add the `1e20`,
  `9.007199254740993e15` and +/-2^53 vectors to AC-93. Name the reading route
  (`quire_canonical::read`), since a serde_json `Value` cannot carry it.
- FND-004: amend AC-26 to name the exception, or drop the pointer from AC-94,
  and align its pointer with AC-30.

FND-005 and FND-006: mark the `Value` encode as pending on its owner's ticket,
and add a lowering-ceiling AC.

## New findings (disposition pass 1)

Reviewed at agent-ix/quire-contract-ir@553736cfce62fe6949915559c471d4a7f5349d21 (main still 7d7716d; not rebased onto #263, whose FR-038, TC-048 and checked_package tests.md hunks conflict textually, expected rebase work). Proof re-run: no schemas/, corpus or code file in `git diff origin/main --stat`; `make corpus` exit 0; workspace tests all pass (204 + 101, 0 failed); validate passes, grammar 1 (FR-014 baseline), 215 ACs, strict 23 unbacked, coverage rows 237.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-011 | low | FR-013 says `maximum_denominator` "already refused `invalid_numeric_bounds` outside `1` through `9223372036854775807`". Today that holds for zero and for a `u64` above `i64::MAX` (`expression.rs:117-118`), but a JSON number past the `u64` range fails serde decode and is `invalid_wire_format` (`wire.rs:45` `u64` field). Say "within the `u64` range", or name the past-`u64` case as part of the same reader change | spec/model/functional/FR-013-type-system.md:62-63 |
| FND-012 | low | The second bullet of "A model document is a value inside a supplied document" still says the refusal "carries the RFC 6901 pointer of that integer" and is "about one integer". The rule now refuses any number whose text denotes a magnitude above 2^53, including `1e20` and fractional spellings, so "integer" understates it | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:514-516 |

## New findings (disposition pass 2)

Reviewed at agent-ix/quire-contract-ir@0ff5f99b81634759348b813ff6bb13897d9dc719. Main is still 7d7716d. `git diff origin/main --stat` lists spec files only: no `schemas/`, `corpus/`, `crates/`, `tests/` or `src/`. `quire validate` passes, with 1 grammar finding (the FR-014 baseline) and 215 ACs; strict coverage has 23 unbacked rows; coverage rows total 237.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-013 | low | Three summaries of the model-document rule still say "integer past 2^53": the class-4 order item (line 229), the FR-322 step 1 order (line 703) and FR-038-AC-27's clause (line 1382). The rule, and AC-93, cover any number whose text denotes a magnitude past 2^53, including non-integer spellings such as `9007199254740993.5`. Say "a number whose text denotes a magnitude past 2^53" in all three, or point them at "A model document is a value inside a supplied document" | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:229, 703, 1382 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 553736c |
| FND-002 | fixed | 553736c |
| FND-003 | fixed | 553736c |
| FND-004 | fixed | 553736c |
| FND-005 | fixed | 553736c |
| FND-006 | fixed | 553736c |
| FND-007 | fixed | 553736c |
| FND-008 | fixed | 553736c |
| FND-009 | fixed | 553736c |
| FND-010 | fixed | 553736c |
| FND-011 | fixed | 0ff5f99 |
| FND-012 | fixed | 0ff5f99 |
| FND-013 | fixed | 59125e9 |
