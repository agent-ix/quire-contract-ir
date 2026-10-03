---
id: SR-1123
title: "code review of PR 271: inexact-integer and inexact-number causes, Writer::number text scan, float_roundtrip (IR-542)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@cb01b8277f92404d729f696baa91d8cfdd441695; git diff origin/main...cb01b82 (base 316fae1, 11 files: crates/quire-contract-model/Cargo.toml, crates/quire-contract-model/src/checked_package/{common.rs,shared.rs,v2/mod.rs,v2/model_members.rs}, tests/it/checked_package_v2_{model_members,canonical_encoding,temporal}.rs, spec/checked_package/matrix/{tests.md,TC-048-checked-package-v2-strict-reader.md}, spec/tests.md)"
review_set: subset
---
# SR-1123: code review of PR 271: inexact-integer and inexact-number causes, Writer::number text scan, float_roundtrip (IR-542)

## Summary

Ticket: IR-542. PR 271 is code part 1. It adds the causes `InexactInteger` and `InexactNumber` to `CheckedPackageRefusalCause`. It replaces `first_number_past_2_pow_53` with `first_inexact_number`. That function compares each model-document number's text with the text `quire_canonical::Writer::number` writes for its double, by digits and scale (`Spelling`). It also turns on `serde_json/float_roundtrip` in the model crate's manifest. FR-038-AC-110 is deliberately out of scope here (IR-555), so its `1e400` clause is not reviewed as missing. The PR body and the coder's claims were treated as data and measured.

What I measured, in a detached throwaway worktree at cb01b82 with its own target directory (both since removed):

- Gates. The touched integration modules pass: model_members, canonical_encoding and temporal, 66 tests. `make fmt-check` and `make lint` pass, covering the workspace clippy and the model-only clippy lanes. `make spec` stops at the known baseline: validate passes, with 1 grammar finding (FR-014) and 23 unbacked under strict. That count is the same on origin/main.
- Correctness of the scan. Rust's own shortest formatter picks the other tie: `{:?}` writes `1125899906842624.3`, `1500000000000000.3` and `2.9802322387695313e-8` for all six tie texts. A scan built on Rust formatting, or on any odd-digit tie rule, would therefore fail both the admitted and the refused tie assertions. A scan that compared exact decimal values would refuse `0.1` and fail the admitted test. A scan that read members in canonical (sorted) order would name `/a/0`, not `/b`, and fail the order test. `Spelling` handles the `e+NN` form that `Writer` emits (`parse_exponent` strips `+`), signed zero (zero has no sign), and trailing and leading zeros.
- `1e-400` reads as the double 0, which is written `0`, so it is refused as inexact-number. `4.9e-324` reads as `5e-324`.
- Copying. quire-canonical at the locked b4bb97a has no digit-and-scale comparison and no `Spelling` type. `number.rs` has only `with_double_text` and `exact_integer_double`. The new code calls quire-canonical's public `Writer::number`, `Number::text` and `Number::value`. Nothing is copied.
- DOUBLE_TEXT_BYTES = 64 is not a raised-limit smell. It is the byte room of a scratch `Writer` that writes one double. The longest RFC 8785 double text is about 25 bytes (for example `-2.2250738585072014e-308`). The constant is never compared with a document size, and it caps no input.
- float_roundtrip. A scratch build of serde_json 1.0.151 without the feature reads `1.2793061557049685`, `1.2106592671318679` and `1.3567384036451073` as neighbouring doubles, and reads `0.1` exactly. So the AC-111 numbers are well chosen. However, `cargo tree -e features -i serde_json` shows `float_roundtrip` already on in this build, through quire-canonical's `serde_json` feature (its manifest lists `serde_json = { features = ["alloc", "float_roundtrip"] }`). See FND-001.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The numeric half of the AC-111 test cannot fail on the defect its comment describes. quire-canonical's `serde_json` feature, which this crate enables, already turns on `serde_json/float_roundtrip` through feature unification. Measured: with `"float_roundtrip"` removed from crates/quire-contract-model/Cargo.toml, `cargo tree` still shows the feature on. The test then fails only at the manifest assertion (line 725), and the `1.2793061557049685` / `1.2106592671318679` / `1.3567384036451073` loop still passes. The manifest line is right, and it is what AC-111 asks for. The behavioural loop is a regression check for the whole build, not an oracle for the manifest declaration. The comment at lines 703-708 implies the loop would catch a missing feature. Say instead that in this build the feature is also on through quire-canonical, so the manifest scan is the oracle for the declaration. | tests/it/checked_package_v2_canonical_encoding.rs:703-725 |

## Verdict

The production change is correct and tightly scoped. The scan decides on the number's text and the text quire-canonical writes, never on doubles. It reports the first number in document order, from the existing heap-stack walk, and gives the integer cause precedence. The refusal-site change (`refused_number_in_document` now carrying a cause) is the only behavioural change outside the scan. The test oracles are strong: every refusal assertion compares the whole `CheckedPackageRefusal` (code, path, cause and `document_pointer`), under both digests. The admitted cases are digested against the canonical text's own digest, so a missing canonicalisation or a wrong tie rule fails them. The one low finding is about what a test comment claims, not about the code. No copied quire-canonical code was found. DOUBLE_TEXT_BYTES is not a raised limit.

## Dispositions

Round 1 was reviewed at 3a34916a52918fb7ae1c1c96df4e40842b829c88, which is one fix commit on cb01b82. I reran the changed modules in a throwaway worktree, since removed. The model_members and canonical_encoding tests pass (34). make fmt-check and make lint pass. make spec stops at the baseline: grammar 1, strict 23 unbacked, 0 contradicted.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed 3a34916 | The comment now says the loop checks the build as a whole and does not fail if this crate's manifest drops the feature, because quire-canonical's `serde_json` feature also turns it on. It names the manifest assertion as the oracle for AC-111's declaration clause. |
