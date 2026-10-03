---
id: SR-1122
title: "gap analysis of PR 270 against quire-canonical's number writer, QSL's merged rule and the planned TC-048 oracle"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@0280d3aa38288301304c41e053f5f33f8b206224; FR-038-AC-109 to AC-111 and the FR-038 canonical-encoding paragraph, measured against quire-canonical b4bb97a (src/writer.rs Writer::number, src/number.rs with_double_text, src/read.rs number), ryu-js 1.0.3, crates/quire-contract-model/src/checked_package/v2/model_members.rs (admit_document) and common.rs (read_value), and quire-spec-language 8244dd2 FR-056-AC-13/AC-14"
review_set: subset
---
# SR-1122: gap analysis of PR 270

## Summary

Ticket: IR-542. Plan completion: not assessed. This review measures the PR's
numeric claims against the quire-canonical writer and checks how strong the
planned TC-048 oracle is.

- Writer: in quire-canonical b4bb97a, `Writer::number` writes
  `with_double_text`. That writes `0` for either zero and otherwise
  `ryu_js::Buffer::format_finite`, which is ECMAScript `Number::toString`.
- Ties, measured three ways: a throwaway ryu-js 1.0.3 program, node's
  `String(Number(s))`, and Python `decimal` at 200 digits.
  - The nearest doubles are exactly 1125899906842624.25,
    1500000000000000.25 and 2.98023223876953125e-8.
  - The `.2`/`.3` (`…312`/`…313`) candidates are each exactly 0.05 (5e-25)
    from them. The shortest round-trip text has 17 digits, so the two are
    equally close shortest candidates.
  - ryu-js and V8 both write the even one: `1125899906842624.2`,
    `1500000000000000.2`, `2.9802322387695312e-8`. So the even spellings are
    admitted and the odd ones differ from the writer's text, giving
    `inexact-number`, as AC-109 says.
- Other examples, under the amended rule:
  - `-0` writes as `0` and `1.0` as `1`, and both are exact by digits and
    scale. `5e-324` is exact.
  - `4.9e-324` writes as `5e-324` and `1e-400` writes as `0`, so both refuse.
  - AC-109's `0.1`, `0.5`, `1.5`, `-0.25` and `2.5e-10` are admitted.
    `0.1000000000000000000001` writes as `0.1` and its negative as `-0.1`,
    so both refuse.
  - `9007199254740993.5` writes as `9007199254740994`, is not whole, and
    refuses `inexact-number` (AC-109). `9007199254740992.5` writes as
    `9007199254740992`, is not whole, and refuses `inexact-number` (AC-110).
  - `9007199254740993.0` is whole and past 2^53, so it is `inexact-integer`
    (AC-110). `9.007199254740992e15` is whole and within 2^53, and equals the
    writer's `9007199254740992` by digits and scale, so it is admitted
    (AC-110).
  - All are consistent with QSL FR-056-AC-13/AC-14.
- Reader path: at the PR base and on origin/main, `admit_document` reads a
  model document once with `quire_canonical::read`. That parses each literal
  with core `str::parse::<f64>`, which is correctly rounded and not
  feature-dependent, and keeps the text. The digest is
  `quire_canonical::sha256` over that reading. The new deciding rule therefore
  maps directly onto the code: (text, double) from the reading, compared with
  `Writer::number`'s text for that double. Only the package document goes
  through `serde_json` (`read_value` → `strict_parse`). See SR-1121 FND-001.
- Oracle strength: the odd-digit cases discriminate between three readings.
  "Even digit on a tie" passes them. "Any shortest round-trip text", which
  admits `.3`, fails. "Round half up", which writes `.3`, would refuse the
  even cases. Comparing the two as doubles admits every inexact number, and
  the `0.1000000000000000000001` cases catch that.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | "Compared by their digits and scale" has no witness in AC-109. Every admitted AC-109 example (`0.1`, `0.5`, `1.5`, `-0.25`, `5e-324`, `2.5e-10`, and the three even tie texts) is byte-identical to the writer's text, so a byte-equality comparison passes all of AC-109. Only AC-110's `9.007199254740992e15` would catch it. No IR case pins trailing zeros or zero's sign. QSL FR-056-AC-14 admits `1.0`, `-0`, `-0.0` and `1e15`, and the PR claims the two readers agree on every number. Adding `1.0` and `-0` (and optionally `1e2`) to AC-109's admitted list would witness the clause and match QSL | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2139; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:629 |

## Verdict

Approve on substance, with one low oracle-strength finding. Every numeric
claim in the amendment, and every example in AC-109 to AC-111, was measured
correct under the new wording. The tie cases are exact ties, and the writer
chooses the even digit.

## New findings (disposition pass 1)

Reviewed at agent-ix/quire-contract-ir@ea56b1196a65c2267d96ae1d4acf69945cb96339 (fix commit ea56b11 on the rebased 1b6ed14 = 0280d3a).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | AC-111 now says `float_roundtrip` "keeps `0.1` admitted whatever other crates in the build turn on", but `0.1` does not depend on the feature. Measured with serde_json 1.0.151 and default features without `float_roundtrip`: `0.1` parses exactly (fast path), while shortest texts such as `1.2793061557049685` (read as `1.2793061557049683`), `1.2106592671318679` and `1.3567384036451073` misparse. That would re-encode differently and falsely refuse the package `noncanonical_wire`. The manifest oracle is source-level and unaffected. Name such a number as the witness, or drop the causal "so keeps `0.1` admitted" | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2148 |

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@ea56b1196a65c2267d96ae1d4acf69945cb96339.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ea56b11: AC-109's admitted list adds `1.0`, `-0` and `1e2`, "spelled other than the text `quire-canonical` writes, `1`, `0` and `100`", and TC-048 repeats them. Measured: ryu-js/ECMAScript write `1`, `0` (with_double_text's zero) and `100`, so each case now witnesses the digits-and-scale comparison against byte equality and matches QSL FR-056-AC-14 |

Round 2, reviewed at agent-ix/quire-contract-ir@fb44bf440cebcbd92456740f9d48b21383a6e6b8.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-002 | fixed | fb44bf4: AC-111 names `1.2793061557049685`, `1.2106592671318679` and `1.3567384036451073` as the package-document numbers that `float_roundtrip` keeps from a false `noncanonical_wire`, and no longer credits `0.1` to the feature. TC-048's AC-111 step checks them alongside `0.1`, which remains only as the plain admitted case. These are the values measured in round 1: each is ryu's shortest text, and serde_json 1.0.151 misparses each without the feature |
