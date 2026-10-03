---
id: SR-818
title: "criterion-strength review of PR 258 canonical encoding (IR-533)"
type: SpecReview
analysis: criterion-strength
scope: "agent-ix/quire-contract-ir@f3606b05780f22897c15fccc0eb2756ab284f14e; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (FR-038-AC-73 to FR-038-AC-78)"
review_set: subset
---
# SR-818: criterion-strength review of PR 258 canonical encoding

## Summary

Ticket: IR-533. For each new criterion, I asked whether a plausible wrong implementation
fails it.

- AC-73 to AC-75 can fail. A hand-written `Encode` that writes `null` for an absent
  `Option` member, misnames or drops a member, or nests a level wrong gives bytes different
  from the `serde_json` form, and the per-type assertion fails. The `serde_json` oracle is
  today's encoder, and it is independent of the quire-canonical path being added.
- AC-76 fails on any native recursion over a 100000-deep `Value` on a 256 KiB stack. Its
  expected text is written by repetition, not by `serde_json`.
- AC-77's integer clause fails if a `Value` integer goes through `Writer::number`, because
  that rounds instead of refusing.
- AC-78's "Encode types do not implement `FixedShape`" clause is enforced by the compiler
  anyway. quire-canonical's blanket `impl<T: FixedShape> Encode for T` makes a hand-written
  `Encode` plus `FixedShape` a conflicting impl.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No criterion observes the reader's own change. AC-73's clause "which the reader recomputes through `Encode`" cannot fail: over the fixtures the bytes are identical, so a reader that still hashes `serde_json` bytes passes. AC-77's ceiling and integer clauses call the encoder directly, which repeats quire-canonical's own `tests/limits.rs` and `tests/encode.rs`. The section's reader-level rules are therefore unbacked: an encode refusal reports `malformed_wire` at `identity_preimage`, and the ceiling is the read's byte limit. Add a reader-level case: a package whose preimage body holds 9007199254740993, read through `CheckedPackageV2::read`, returns the refusal FR-038 settles on (SR-815 FND-001). That case is the only observable difference between the old path and the new one. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1236,1240 |
| FND-002 | low | AC-74's crafted graph tests "integer" bodies and nothing at the edges where `serde_json` and RFC 8785 begin to disagree. The in-repo fixtures have no non-ASCII content and no large integer, so a criterion that names those edges is the only place the "bytes are unchanged" claim gets tested at its limits. Add string values with non-ASCII and astral characters, and the integers 2^53 and -2^53. These must still be byte-equal; 2^53 + 1 is AC-77's refusal. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1237 |

## Verdict

Each new criterion can fail, but none of them pins the reader-level behaviour this
requirement changes. Fix FND-001 together with SR-815 FND-001. FND-002 is optional
hardening.

## Dispositions

Round 1 at 2f491e0a30bd3b29b9ae6f9abf81f28ba09bac81. The criteria are now AC-74 to AC-80
after the renumbering. The new AC-79 is read through the reader. It fails if the reader
keeps the `serde_json` canonical check, which admits ±9007199254740993, admits `2.0`, and
refuses UTF-16 order. It also fails if any grammar or `package_id` refusal decides first,
because each document also carries a grammar defect. AC-74 and AC-75 now add non-ASCII and
astral strings, ±2^53, 0, -1, and names ordered the same under UTF-8 and UTF-16.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | fixed 2f491e0a30bd3b29b9ae6f9abf81f28ba09bac81 |
| FND-002 | fixed | fixed 2f491e0a30bd3b29b9ae6f9abf81f28ba09bac81 |
