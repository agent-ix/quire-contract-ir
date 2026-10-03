---
id: SR-1013
title: "gap analysis of PR 265 against the checked-package v2 reader code and the planned TC-048 oracle"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@8edd1e5e1f660fe5e493c909a9a2113d09b33a6e; FR-038-AC-93 and FR-038-AC-96 to AC-98 and the new FR-038 paragraph, measured against crates/quire-contract-model/src/checked_package/common.rs (read_value, require_canonical_bytes, strict_json_value, digest_json), v2/model_members.rs (admit_document), shared.rs (CheckedPackageRefusalCause), v2/mod.rs (CheckedDiagnosticCause) and TC-048 \"Inexact numbers\""
review_set: subset
---
# SR-1013: gap analysis of PR 265

## Summary

Ticket: IR-542. Plan completion: not assessed. This review measures the
author's claims about today's code, and how strong the planned TC-048 oracle
is.

The author's claims below hold.

- **Package documents.** A package document holding an inexact number is
  refused `noncanonical_wire` today, with no pointer. `read_value` calls
  `require_canonical_bytes`, which returns `refused_bytes` when the bytes
  differ from the canonical re-encoding. That happens after strict parsing and
  before `admit`, so the refusal comes before any grammar, `package_id` or
  graph refusal, as AC-98 says.
- **Model documents.** A selected model document goes through
  `strict_json_value` and is then digested by `digest_json`, which is
  `serde_json::to_vec`. It has no check for inexact numbers and none for
  integers past 2^53. A probe with serde_json 1.0.151 showed that `0.1` and
  `0.1000000000000000000001` re-encode to the same bytes, so they digest
  alike.
- **No pointer.** `document_pointer` appears nowhere in `crates/` or `tests/`.
- **No causes.** Neither cause exists in `CheckedPackageRefusalCause` or
  `CheckedDiagnosticCause`, and there is no `inexact` string anywhere in the
  crates.
- **AC-93's status is true.** It is planned with AC-89 to AC-95 (IR-274),
  nothing in the code implements it, and no test is tagged FR-038-AC-93. AC-97
  refines AC-93 and is consistent with it.

The planned oracle is strong.

- 1e20 kills an "inexact means binary-inexact" mutant on the integer side.
- 0.1, 5e-324 and 2.5e-10 kill the same mutant on the admitted side.
- 9007199254740993.0 kills a mutant that checks for `inexact-number` before
  `inexact-integer`.
- The two-order mixed document kills a mutant that scans for integers first.
- Checking under both the document's own digest and another digest pins the
  claim that this refusal comes before `byte-digest-mismatch`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new paragraph says `9007199254740993.5` and `9007199254740994` "are read as one value and share one digest today". That is false, measured with serde_json 1.0.151 and the unified `float_roundtrip`. The first parses as an f64 and re-encodes as `9007199254740994.0`. The second parses as a u64 and re-encodes as `9007199254740994`. The bytes differ, so the digests differ. The pair that does digest alike is `9007199254740993.5` and `9007199254740994.0` | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:550 |
| FND-002 | medium | FR-272 makes a number past the double range an `inexact-integer`, because its text "denotes a whole value of magnitude above 2^53, however it is spelled". Examples are 1e400 and -1e400. serde_json refuses these at parse time ("number out of range", measured), and `admit_document` turns any parse failure into `stale_dependency`/`byte-digest-mismatch` at `digest`. No criterion covers this case. The IR-542 code change can therefore pass every planned row and still not conform to FR-272. Add 1e400 and -1e400 to AC-97, expecting `inexact-integer` at their pointer | crates/quire-contract-model/src/checked_package/v2/model_members.rs:873 |
| FND-003 | low | AC-97 asserts the `inexact-integer` cause only for the numbers it lists. AC-93 also refuses an integer value type whose upper bound is `9007199254740993`, and no criterion or TC-048 step asserts a cause or pointer for that refusal. The new paragraph says the cause covers "every number FR-038-AC-93 refuses" | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1600 |
| FND-004 | low | The paragraph says "This reader reads a number as its nearest double". That holds only because serde_json's `float_roundtrip` is switched on through feature unification, by quire-verification-contracts' jsonschema and serde_json_canonicalizer. The crate's own manifest asks only for `unbounded_depth`. Without `float_roundtrip`, 593418 of 2,000,000 random shortest round-trip texts were misparsed (for example `1.0715660391465826e-75`), and `9007199254740993.0` parses to ...994. Under AC-96 each of those would become a false `inexact-number` refusal. Either the classification works from the number's text, or the crate declares `float_roundtrip` itself | crates/quire-contract-model/Cargo.toml:33 |

## Verdict

Changes requested for FND-001 (a false claim about today's behaviour) and
FND-002 (FR-272 cases that no planned row covers). FND-003 and FND-004 are
small additions. The other claims held when measured, AC-93's planned status
is true, and the planned oracle is otherwise strong.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | The feature-independence clause added to AC-96 and to TC-048 cannot be falsified as written. The clause is "every decision above is the same whichever `serde_json` features other crates in the build turn on, and the crate's own manifest declares the number-parsing feature it relies on"; TC-048 says "with the `serde_json` number features other crates enable switched off". `cargo tree -p quire-contract-model -e features,normal` shows `float_roundtrip` turned on by the crate's own normal dependency quire-verification-contracts (through jsonschema and serde_json_canonicalizer). No cargo invocation in this repo therefore builds the crate with that feature off, and the TC step names a configuration no test can produce. The manifest half does not name the feature, so a manifest-reading test has nothing exact to check, and a reader that classifies from source text relies on none. Name it: either "the crate's manifest declares `serde_json`'s `float_roundtrip`, checked by a test that reads the manifest", or "the decision is made from the number's source text, checked by a unit test of the classifier over the listed texts". Then drop the "switched off" step | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1611 |

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@2f80c1ecb15272f49513f6db26a4632d9629b194, the delta 8edd1e5..2f80c1e only. The digest claim was re-measured with serde_json 1.0.151 `to_vec`, with and without `float_roundtrip`: `9007199254740993.5` and `9007199254740994.0` both re-encode as `9007199254740994.0`, and `1e400`/`-1e400` fail with "number out of range", which `admit_document` maps to `stale_dependency`/`byte-digest-mismatch`.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 2f80c1e: the pair is now `9007199254740993.5` and `9007199254740994.0`; both re-encode as `9007199254740994.0` (measured), so the claim is true |
| FND-002 | fixed | 2f80c1e: AC-97 adds `1e400` and `-1e400` as `inexact-integer`, refused `noncanonical_wire` at the row's `digest` with their `document_pointer` where today they refuse `stale_dependency`/`byte-digest-mismatch` (measured); the classification matches FR-272's "whole value of magnitude above 2^53, however it is spelled"; TC-048 checks "not `byte-digest-mismatch`" |
| FND-003 | fixed | 2f80c1e: AC-97 asserts `inexact-integer` for the integer value type whose upper bound is `9007199254740993`, and the TC-048 step includes it |
| FND-004 | fixed | 2f80c1e: the paragraph now says the reader decides from the number's text, independent of features that other crates enable, and that the manifest declares the feature it relies on; AC-96 and TC-048 carry the clause (its testability is FND-005) |

Round 2, reviewed at agent-ix/quire-contract-ir@24bd465e9c950424b79662ee7da413696385d43b, the delta 2f80c1e..24bd465 only. `make spec` was run in a detached throwaway worktree: validate passes over 343 docs, including both reviews/ copies; 1 grammar finding (FR-014); strict coverage 23 unbacked and 0 contradicted. The reviews/ copies were byte-identical to the round-1 files.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-005 | fixed | 24bd465: AC-96 now names the feature: "the manifest of the crate that holds the reader declares `serde_json` with the feature `float_roundtrip`". TC-048's "switched off" step is replaced by a source-level manifest check, which a test can fail: crates/quire-contract-model/Cargo.toml:33 declares only `unbounded_depth` today, as planned |
