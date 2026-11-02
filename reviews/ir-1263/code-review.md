---
id: SR-2026110101269
title: IR-1263 code and Rust review of Text bounds shape
type: SpecReview
analysis: code-review
scope: 'agent-ix/quire-contract-ir; IR-1263 PR #818; crates/quire-contract-model/src/checked_package/v2/structural.rs,
  tests/conformance_qspec/text_literals.rs, three repaired positive fixtures'
review_set: subset
---

## Summary

Ticket: IR-1263. Reviewed PR #818 at f445c250 against IR FR-038-AC-417..419 and QSpec FR-322-AC-306. The complete-shape path and published vectors work for tested values; an unbounded canonical integer edge is mishandled.

## Verdict

**CONDITIONAL** — one medium finding needs a fix before merge.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Canonical Text max=18446744073709551616 with min=0 and profile=nfc meets the stated shape rule, but text_bounds_shape parses each bound as u64 and returns false on overflow. The reader then refuses invalid_package/invalid-derived-shape. Compare canonical decimal strings without narrowing, and cover the admitted shape and downstream literal behavior. | crates/quire-contract-model/src/checked_package/v2/structural.rs:716 |

## Examined scope

```yaml
scope:
- id: FR-038-AC-417
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-1263 lands. A `bounded_domain`/`text_bounds` node with
    exactly ordered `aggregate{min, max, text_profile}` admits when both bounds are
    canonical nonnegative integer literals with `min <= max` and the text profile
    is one of `unicode-scalars`, `nfc`, `nfd`, `nfkc`, `nfkd`, or `binary-utf8`. `Text[0,
    64; nfc]` and the equal-bound `Text[2, 2; binary-utf8]` admit; no unbounded `Text[profile]`
    shape exists.
- id: FR-038-AC-418
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-1263 lands. A profile-only `Text[nfc]`, absent `min`,
    `max` or `text_profile`, an extra or reordered member, a noninteger or negative
    bound, `min > max`, or a profile outside the six refuses `invalid_package`/`invalid-derived-shape`
    at `/semantic_graph/nodes/{n}`, with that node as locus and no admitted package.
    Shape refusal precedes Text literal membership even when a literal typed by the
    malformed node would itself fail its length or normalization check.
- id: FR-038-AC-419
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-1263 lands. TC-048 reads QSpec `proposals/checked-package-v2/text-bounds-shape-vectors.json`
    from the selected checkout by reference, verifies the exact `TS-01` through `TS-10`
    case set, and checks each through the public strict reader for admission or the
    specified code, cause, primary RFC 6901 node pointer and structural node locus.
    It stores no local vector or schema copy; the existing profile-only recursive-leaves
    fixture gains explicit `min` and `max` so its positive read remains valid.
- id: FR-322-AC-306
  path: /home/peter/dev/worktrees/ir1263-qspec-source/spec/objects/interfaces/FR-322-checked-package-artifact.md
  role: context_only
  excerpt: 'A `bounded_domain`/`text_bounds` node is admitted only with the complete
    body `aggregate{min, max, text_profile}` (`min`, `max` nonnegative integer literals,
    `min <= max`, `text_profile` one of FR-141''s six): `Text[0, 64; nfc]` and `Text[2,
    2; binary-utf8]` are admitted. A profile-only node (`Text[nfc]`), a node missing
    `min`, a node missing `max`, a node missing `text_profile`, a member beside the
    three, `min` above `max` and a profile outside the six each refuse `invalid_package`/`invalid-derived-shape`
    at `/semantic_graph/nodes/{n}`, before any literal typed by the node is checked
    for membe'
```

## Coverage

tests with TS-01..10 and all six profiles pass; final aggregate gate is owned by the lead.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The fix compares canonical decimal bounds without narrowing, checks literal lengths the same way, and saturates only the Exact equality maximum because retained Text lengths are u64. |

## Disposition verdict

**PASS (round 1, source review at 5707491b)** — The original finding is fixed. Focused QSpec 4/4 and equality 2/2 checks pass. The code lead owns the final aggregate gate.
