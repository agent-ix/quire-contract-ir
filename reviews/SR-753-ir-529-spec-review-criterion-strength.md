---
id: SR-753
title: "criterion-strength review of PR 251 (IR-529 artifact references)"
type: SpecReview
analysis: criterion-strength
scope: "agent-ix/quire-contract-ir@0b106a64ea261a86c0f2f64eb9ab8f31f546d39d; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (FR-038-AC-46 through FR-038-AC-64); FR-038-AC-4 as context"
review_set: subset
---
# SR-753: criterion-strength review of PR 251

## Summary

Ticket: IR-529. I judged each new AC for whether it can fail against a wrong reader.
This was a manual judgment over the spec text and the code; the Jev judge was not used.

Clean units, each of which names an observable code, pointer or member set that a
wrong reader would violate:

- Admission: FR-038-AC-46 and AC-47.
- Refusals: AC-48 to AC-52, AC-56, AC-58 and AC-59.
- Joins compared by pair: AC-53 and AC-54.
- Catalog reading: AC-55 and AC-64.
- Public shape: AC-60 to AC-62.

Some of these carry correctness defects recorded in SR-750: AC-48, AC-50, AC-51, AC-52,
AC-54 and AC-58. That does not make them unable to fail.

Adverse-case coverage per FR: FR-038's new section covers the extra-member,
missing-member, wrong-kind, empty, wrong-domain and join-miss cases.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-038-AC-57's first clause ("when one definition row's `identity` is edited and the preimage and `package_id` are re-derived, the `package_id` shall change") asserts a property of the fixture's own re-derivation, a hash of different bytes. It does not assert anything the reader does, so no reader defect can fail it. Its second clause repeats FR-038-AC-4 ("editing ... raw source digest leaves it unchanged"). Restate it as a reader observation: the re-derived package admits with the new id, and the old id refuses `stale_dependency`. Or drop it in favour of AC-4. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:851,800 |
| FND-002 | low | FR-038-AC-63 says "the reader shall emit each definition row of `identity_preimage`". The reader reads the preimage and does not emit one, and "definition row" does not say whether `edition` and `profile_selections` definitions are included. The oracle is unclear: the serialized typed preimage, or the accessor. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:857 |

## Verdict

The criteria are strong overall. Two low findings, on AC-57 and AC-63.

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f (delta from 0b106a64ea261a86c0f2f64eb9ab8f31f546d39d; base origin/main e80ea70ab8874676de47ac1b7fb439497ced7991 unchanged).

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f |
| FND-002 | fixed | 9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f: criterion removed; coverage remains in FR-038-AC-46 and FR-038-AC-48 |
