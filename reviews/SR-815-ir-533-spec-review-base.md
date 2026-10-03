---
id: SR-815
title: "spec review of PR 258 canonical encoding of the V2 wire types (IR-533)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@f3606b05780f22897c15fccc0eb2756ab284f14e; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/model/functional/FR-019-rust-library-interface.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md"
review_set: subset
---
# SR-815: spec review of PR 258 canonical encoding of the V2 wire types

## Summary

Ticket: IR-533. PR #258 is spec only. It amends FR-038's `package_id` paragraph and adds a
section, "Canonical encoding of the wire types". It adds FR-038-AC-73 to AC-78, one FR-019
paragraph, the TC-048 procedure and traces, and the FR-038 and TC-048 rows of tests.md. The
base is current: the merge-base is origin/main 1a59efc, and the PR merges cleanly onto main.

Checked independently against the code at the reviewed sha and quire-canonical origin/main
59fe4f06370fbdbd8de4fa446ad78f3974926252.

- The type table is right. `CheckedSemanticId`, `CheckedCapability`,
  `CheckedSourceMapEntry` and `CheckedPackageLockV2` hold only strings, `u64`, `Vec`,
  `Option` and closed enums (shared.rs:340-462, v2/mod.rs:146-192), so their depth is fixed.
  The `closed_vocabulary!` enums take a derive through their `$(#[$meta])*` and serialize as
  one string (vocabulary.rs:34-73), so DEPTH 0 is correct. The preimage, the graph and the
  diagnostics hold `serde_json::Value` (v2/mod.rs:97, 126, 335). I agree with the author that
  `CheckedDiagnosticsV2` takes `Encode`. The ticket listed it as `FixedShape`, which would not
  compile, because quire-canonical has no `FixedShape` for `Value`.
- The quire-canonical claims are true at origin/main. `Encode` is a push-event trait
  (lib.rs). `FixedShape` is derived from every field's `DEPTH` (shape.rs,
  quire-canonical-derive). There is no `MAX_DEPTH` and no depth refusal, only byte limits.
  Integers past 2^53 are refused by `IntegerMagnitudeAboveMaximum`, which names the value
  (error.rs). `sha256` hashes no domain label and `sha256_with_domain` hashes a
  length-prefixed one, so "no domain label hashed in" means `sha256`. The library has no
  `serde_json` dependency, so it has no `Encode` for `Value`.
- Are the bytes identical? The reader already requires the whole document to equal
  `serde_json::to_vec` of its parsed value (common.rs:250-257), and computes `package_id`
  over `serde_json` bytes of the preimage (v2/mod.rs:755-760). With `preserve_order` off,
  keys sort by UTF-8 bytes. RFC 8785 sorts by UTF-16 code units and spells numbers the
  ECMAScript way. The two agree on ASCII keys, on strings, and on integers of magnitude at
  most 2^53. They disagree on keys that mix U+E000 to U+FFFF with supplementary-plane
  characters, on non-integer numbers, and on integers past 2^53. The in-repo fixtures hold
  none of the disagreeing cases: `tests/it/support/checked_package.rs` has no non-ASCII
  content, no float and no large integer. FR-038 says the bytes are unchanged "for every
  in-repo fixture", which is true. The universal claims that follow it are not (FND-001).
- AC-73 to AC-78 are written as direct assertions, the repository's convention.
- `make spec` gives the same counts before and after: validate passes, 1 grammar finding
  (the existing FR-014 `ac:vague-response`), and 23 strict unbacked rows. The strict gate
  exits 2, as it does on main. Coverage is 166/212 before and 166/218 after; FR-038 is
  45/76.
- Hygiene is clean. The PR has no pin, no SHA, no vendored copy and no compatibility layer.
  The dependency is written `branch = "main"`, like this repository's other first-party git
  dependencies. The PR title has no bare ticket id, and the body says "Part of IR-533".

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | "No reader, lowering or QSL output changes with this requirement" is false, and the refusal it brings conflicts with held PR #243 (IR-274). The V2 literal grammar admits any `i64`/`u64` number (common.rs:844-849). A document with a body integer of 9007199254740993 is `serde_json`-canonical, so it passes the document check today and hashes. Under this requirement, the preimage encode refuses it, and the new section reports that as `malformed_wire` at `identity_preimage`. That is an admission change. #243's FR-038 text gives the same input `noncanonical_wire` at the document level, and also says every `package_id` encode goes through quire-canonical. So two open changes give two different refusal codes for one input. The `package_id` check also runs before grammar validation (v2/mod.rs:755). A refused input with a float or with mixed BMP and astral keys can therefore get a different refusal: `stale_dependency` today, and a later grammar refusal after this change, or the reverse. Fix: state the admission change in the new section, add a reader-level AC for it, and record which ticket owns the code. Either align with IR-274's `noncanonical_wire`, or have IR-274 adopt this ruling. Leader decision. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:349-366 |
| FND-002 | medium | The new section lists `CheckedRevision` among the types that derive `FixedShape`. FR-038's merged artifact-reference amendment (AC-46 to AC-61, IR-529) already makes a reference `{authority, identity}`, with a source row `{authority, identity, digest_domain, digest}`. Open PR #253 (IR-530) deletes `CheckedRevision` and adds `CheckedSourceRef` as `CheckedSourceRegion.source`. The spec should not name a type its own merged text retires. Name the types by role ("every artifact and source reference type"), or list `CheckedSourceRef` and drop `CheckedRevision`. Then order the IR-533 code after #253, or rebase it over #253. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:334-338 |
| FND-003 | medium | The list of types that derive `FixedShape` is incomplete. The section has each `Encode` type delegate "a fixed-depth member to `Writer::serialize`", and `Writer::serialize` requires `T: FixedShape` (quire-canonical writer.rs:395). The members the graph, preimage and diagnostics delegate therefore need the derive too, and neither the list nor AC-78 names them: `CheckedOccurrence`, `CheckedDeclaration`, `NominalIdentityPreimage` with its four preimage structs, `NominalOwner`, `DimensionTerm`, `CheckedRational`, and `CheckedDiagnosticStage`, `CheckedDiagnosticCode` and `CheckedDiagnosticCause`. An implementer could hand-write events or a `FixedShape` impl for them, and AC-78 forbids the second. Extend the rule to "every fixed-depth type an `Encode` implementation delegates", and give AC-78 the same scope. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:334-346 |
| FND-004 | low | "a string, number, boolean and null are the matching scalar event" leaves the number event open. `Writer::number(f64)` rounds 9007199254740993 to 9007199254740992 silently, while `Writer::integer` refuses it. AC-77 would catch the wrong choice, but the prose should say it: an integer `Value` number is written with `integer`, and any other number with `number`. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:349-353 |

## Verdict

Changes requested. The type assignment, the quire-canonical facts, and the
"no wrapper / no copy / no literal DEPTH" rules are correct. Fix FND-001 before merge,
because it states a no-change guarantee that the requirement itself breaks, and it collides
with IR-274. FND-002 and FND-003 are small text fixes. FND-004 is a nit.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | Three wording problems in the new section "Canonical bytes are quire-canonical's bytes". (1) It opens "narrows what the reader admits, once, for one input and one code", then lists three inputs, and the third one widens admission: names in UTF-16 order were `noncanonical_wire` and now pass. (2) "the same names in UTF-16 order are admitted" overstates the change. A body object with those member names still fails the closed body grammar. What changes is that it passes the canonical-bytes check, which is what FR-038-AC-79 says. (3) "The held change of IR-274 ... whichever of the two changes merges second keeps this paragraph's text once" is merge-coordination text in a normative section, and it goes stale when either change merges. Say "for three inputs, under one code". Say "pass the canonical-bytes check". Move the merge note to the PR or ticket, or reduce it to a dependency line. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:385-422 |
| FND-006 | low | "for every in-repo fixture, including strings with non-ASCII and astral characters and the integers 9007199254740992 and -9007199254740992" says the fixtures hold those values. They do not: `tests/it/support/checked_package.rs` has no non-ASCII content and no large integer. The crafted cases of FR-038-AC-74 and AC-75 hold them. Say "for every in-repo fixture and for the crafted values of FR-038-AC-74 and FR-038-AC-75". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:375-382 |

## Dispositions

Round 1 at 2f491e0a30bd3b29b9ae6f9abf81f28ba09bac81. The author squashed and rebased onto
main 7ed352beb994355413b9d7c572e45667aee229ba (#257 merged). This round reviewed the delta
only, excluding #257's content.

- QSpec FR-322 (quire-specification origin/main) lists `noncanonical_wire` in the v2 refusal
  vocabulary.
- The reader's byte-stream refusals carry no pointer (`refused_bytes`, common.rs:250-257).
- The canonical check runs in `read_value`, before decode and before the version dispatch.
- The V2 literal grammar admits only `i64`/`u64` (common.rs:844-849), so a whole float is
  refused by the grammar today.
- AC-79's classes are right against quire-canonical 59fe4f06370fbdbd8de4fa446ad78f3974926252:
  - its integer bound is "magnitude exceeds 2^53", so ±9007199254740993 is refused and
    ±9007199254740992 is not;
  - its ECMAScript number text spells 2.0 as `2`;
  - in UTF-16, U+10000 (D800 DC00) sorts before U+E000, and in UTF-8, U+E000 (EE..) sorts
    before U+10000 (F0..).
- The recompute encode cannot refuse after intake: the document's own bytes are the
  canonical encoding, under the read's byte limit.
- `make spec`: 1 grammar finding and 23 strict unbacked rows, the same as main. Coverage is
  166/220, and FR-038 is 45/78.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | fixed 2f491e0a30bd3b29b9ae6f9abf81f28ba09bac81 |
| FND-002 | fixed | fixed 2f491e0a30bd3b29b9ae6f9abf81f28ba09bac81 |
| FND-003 | fixed | fixed 2f491e0a30bd3b29b9ae6f9abf81f28ba09bac81 |
| FND-004 | fixed | fixed 2f491e0a30bd3b29b9ae6f9abf81f28ba09bac81 |

Round 2 at f6195185fbeebc0f697ae76e98c33673591c2324, still based on main
7ed352beb994355413b9d7c572e45667aee229ba. The delta is one commit that changes wording in
FR-038 only: 12 lines added, 13 removed, and no AC, trace or matrix change. `make spec`
gives 1 grammar finding and 23 strict unbacked rows, the same as main. FR-038 coverage is
45/78. No regression and no new findings.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-005 | fixed | fixed f6195185fbeebc0f697ae76e98c33673591c2324 |
| FND-006 | fixed | fixed f6195185fbeebc0f697ae76e98c33673591c2324 |
