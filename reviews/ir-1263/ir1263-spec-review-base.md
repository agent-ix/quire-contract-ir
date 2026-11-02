---
id: SR-2026110101266
title: "IR-1263 PR 816 base spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR 816; FR-038-AC-417..419 and TC-048 Text bounds shape"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: reviews
---

## Summary

Reviewed FR-038-AC-417 through AC-419 and the TC-048 Text bounds shape plan against QSpec FR-322-AC-306 and published TS-01 through TS-10. No applicable AssuranceProfile was found under spec/; no enforced spec-review policy applies.

## Verdict

**PASS. The three unique criteria cover complete shape admission, malformed-shape refusal and precedence, and public-reader vector evidence. TC-048 verifies each and adds wrong-kind, reorder and negative-bound controls. The QSpec reference edge resolves to FR-322-AC-306.**

## Scope

- **FR-038-AC-417** (spec/checked_package/functional/FR-038-consume-checked-package-v2.md; role: examined). Excerpt: planned until IR-1263 lands. A `bounded_domain`/`text_bounds` node with exactly ordered `aggregate{min, max, text_profile}` admits when both bounds are canonical nonnegative integer literals with `min <= max` and the text profile is one of `unicode-scalars`, `nfc`, `nfd`, `nfkc`, `nfkd`, or `binary-utf8`. `Text[0, 64; nfc]` and the equal-bound `Text[2, 2; binary-utf8]` admit; no unbounded `Text[profile]` shape exists.
- **FR-038-AC-418** (spec/checked_package/functional/FR-038-consume-checked-package-v2.md; role: examined). Excerpt: planned until IR-1263 lands. A profile-only `Text[nfc]`, absent `min`, `max` or `text_profile`, an extra or reordered member, a noninteger or negative bound, `min > max`, or a profile outside the six refuses `invalid_package`/`invalid-derived-shape` at `/semantic_graph/nodes/{n}`, with that node as locus and no admitted package. Shape refusal precedes Text literal membership even when a literal typed by the malformed node would itself fail its length or normalization check.
- **FR-038-AC-419** (spec/checked_package/functional/FR-038-consume-checked-package-v2.md; role: examined). Excerpt: planned until IR-1263 lands. TC-048 reads QSpec `proposals/checked-package-v2/text-bounds-shape-vectors.json` from the selected checkout by reference, verifies the exact `TS-01` through `TS-10` case set, and checks each through the public strict reader for admission or the specified code, cause, primary RFC 6901 node pointer and structural node locus. It stores no local vector or schema copy; the existing profile-only recursive-leaves fixture gains explicit `min` and `max` so its positive read remains valid.
- **TC-048** (spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; role: examined). Excerpt: Planned until IR-1263 lands. Read QSpec's
`proposals/checked-package-v2/text-bounds-shape-vectors.json` directly from
the selected checkout. Assert the exact `TS-01` through `TS-10` case set;
construct complete IR-owned V2 packages with each published body, keeping all
other package facts valid. Read through the public strict reader and compare
admission or exact `invalid_package`/`invalid-derived-shape`, primary RFC 6901
node pointer and structural node locus. Independently check the complete
ordered three-binding shape, nonnegative integer bounds, inclusive ordering
and six-profile vocabular

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence

QSpec vector case IDs and expected outcomes match the criterion. No hash, digest, pin, local vector or schema copy was added.
