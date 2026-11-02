---
id: SR-2026110283705
title: IR-1273 PR 837 scope-boundary review
type: SpecReview
analysis: scope-boundary
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md;
  spec/model/functional/FR-019-rust-library-interface.md
review_set: subset
---

## Summary

Ticket IR-1273, draft PR #837, frozen HEAD 0328065cf043fef3211694bced70a429c03b8a27. IR owns byte admission and typed source references; callers own supplying raw bytes; QSpec owns normative wire/cause contracts; QSL owns source interpretation. This feature authenticates caller-supplied bytes against claims and makes no public provenance or producer authenticity guarantee. No applicable AssuranceProfile exists under spec/. Scope is only the proposed delta.

## Examined scope

```yaml
scope:
- id: FR-038-proposal-0-0
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: (IR-1273; not implemented)
- id: FR-038-proposal-1-0
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'This subsection and AC-435 through AC-440 are a proposed delta, planned
    until

    IR-1273 lands. They do not describe the current reader. Reader implementation

    is gated on the QSpec owner amending FR-322''s prohibition of source-byte

    integrity, both authoritative V2 schemas and the conformance schema''s source

    reference, and FR-272''s closed diagnostic cause pairing. QSpec owns those

    artifacts; IR neither edits nor copies them. The proposal serves QSL FR-518

    row 28m and AC-9 (spec PR 2144), cited by QSL-1501.'
- id: FR-038-proposal-2-0
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The proposed closed `CheckedSourceRef` shape is

    `{authority, identity, source_integrity}`. `source_integrity` is exactly 64

    lowercase ASCII hexadecimal characters encoding SHA-256 of the complete raw

    source byte sequence, with no decoding, newline conversion, normalization,

    canonical JSON encoding, domain prefix or other preimage transformation.

    `CheckedArtifactRef`, `CheckedArtifactLocator` and source nominal owners retain

    their current shapes. A source''s nominal identity remains its exact

    (`authority`, `identity`) pair; integrity is an additional authenticated claim,

    not a replacem'
- id: FR-038-proposal-2-1
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: ent identity or a package/definition digest.
- id: FR-038-proposal-3-0
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The caller supplies bytes through

    `CheckedPackageEvidence::insert_source_document(authority, identity, bytes)`.

    The evidence retains one byte sequence per exact pair, independently of domain

    documents. Equal raw-byte repeats succeed and retain the first sequence;

    different repeats return `SourceDocumentConflict` containing the pair and leave

    evidence unchanged. There is no filesystem lookup, URI resolution or network

    access. An unused evidence entry is neither read nor charged. No implicit

    source bytes are inferred from package bytes, domain documents or dependencies.'
- id: FR-038-proposal-4-0
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'After strict JSON, canonical-byte and ordinary closed-schema checks, a
    dedicated

    source-integrity phase precedes header/body, graph, lock and source-map semantic

    validation. For this phase only, absence or wrong JSON kind of

    `source_integrity` is retained as an integrity fault rather than an ordinary

    missing-member or wrong-kind decode refusal. Extra members, malformed JSON and

    other missing/wrong-kind reference members retain existing decode precedence.

    The phase visits all actual `CheckedSourceRef` sites: `lock.sources` in array

    order, then source-map entries and their regions in a'
- id: FR-038-proposal-4-1
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'rray order, then diagnostic

    entries and their loci in array order. At each reference, a missing, wrong-kind

    or non-lowercase-64-hex integrity value, missing caller bytes, or unequal digest

    refuses `invalid_package`/`source-integrity` at that reference''s RFC 6901

    pointer, never at its integrity member. Within one reference the order is

    integrity syntax, byte availability, resource charge, digest comparison. The

    first failing reference decides; no legacy two-member source reference admits.'
- id: FR-038-proposal-5-0
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Each first encountered pair with available bytes charges its full byte
    length

    against a per-read cumulative source-byte ceiling equal to `limits.bytes`,

    before hashing; this counter is separate from the existing package wire-byte

    counter. Overflow or exceeding the ceiling returns `incomplete`, limit `bytes`,

    with that ceiling, saturated consumed count and the reference pointer. Each

    visited reference consumes one `work` visit before inspection. The reader

    computes and caches at most one digest per pair per read; repeats compare their

    claim with that cached digest and do not charge so'
- id: FR-038-proposal-5-1
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'urce bytes again. This bounds

    hash input by `limits.bytes` and traversal by `limits.work`. Empty bytes are a

    valid source, requiring their actual SHA-256 value.'
- id: FR-038-proposal-6-0
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'All three members participate in exact source-reference equality for region

    membership in `lock.sources`; nominal owner joins still use the pair. Integrity

    checking precedes region membership, so a differing claim reports

    `source-integrity` before `invalid_source_map`. A source-map reference with a

    valid claim for a different unlocked pair retains the existing

    `invalid_source_map` refusal. The writer preserves the required integrity

    member in every reference, including diagnostics and lowering source regions;

    it never substitutes, computes from a source range or silently defaults it.'
- id: FR-038-proposal-6-1
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: '

    No change to interval bounds, source interpretation, producer path identities

    or package structural equality is introduced.'
- id: FR-019-proposal-0-0
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: (IR-1273; not implemented)
- id: FR-019-proposal-1-0
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: 'FR-019-AC-6 describes the implemented two-member source reference. The

    FR-038 proposed source-byte-integrity delta is planned until IR-1273 lands and

    its QSpec owner dependencies are resolved. It changes only `CheckedSourceRef`

    to require the public third member `source_integrity: Box<str>` and adds

    `CheckedPackageEvidence::insert_source_document(authority, identity, bytes)`

    with the keyed `SourceDocumentConflict` insertion error. The eventual reader

    change must replace AC-6''s source-only two-member assertion and its compile

    controls; `CheckedArtifactRef`, `CheckedArtifactLocator`, a'
- id: FR-019-proposal-1-1
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: 'bsent

    `CheckedRevision` and their existing forbidden members remain unchanged.

    No implemented API or test-evidence claim is made by this proposed delta.'
- id: FR-038-AC-435 (planned until IR-1273 lands)
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The closed source reference requires `authority`, `identity`, `source_integrity`;
    the last is lowercase hexadecimal SHA-256 over exact raw bytes. Matching ASCII,
    non-UTF-8, empty and newline-distinct sources admit through the public reader
    with caller-supplied evidence; writer output preserves the member. Definition
    refs and locators remain two-member shapes.
- id: FR-038-AC-436 (planned until IR-1273 lands)
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Missing, null, wrong-kind, malformed or mismatching integrity, and unavailable
    source bytes, refuse `invalid_package`/`source-integrity` at the exact reference
    pointer in each lock, source-map and diagnostic site. A one-byte evidence mutation
    leaves package bytes unchanged and refuses as QSL FR-518 row 28m requires; no
    two-member compatibility path exists.
- id: FR-038-AC-437 (planned until IR-1273 lands)
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Multiple integrity failures select lock, source-map, diagnostic traversal
    order and then array order; ordinary strict JSON/canonical/closed-schema errors
    precede the integrity phase. An integrity mismatch precedes header/body/graph/lock/source-map
    semantic faults. Invalid integrity syntax precedes absent bytes, which precedes
    source-byte charging and mismatch comparison.
- id: FR-038-AC-438 (planned until IR-1273 lands)
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Evidence insertion keys by the exact authority/identity pair, equal repeats
    retain the first bytes and succeed, unequal repeats return the keyed SourceDocumentConflict
    and preserve evidence. Same identity under different authorities remains independent.
    No reader filesystem/network IO occurs.
- id: FR-038-AC-439 (planned until IR-1273 lands)
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Unique referenced source bytes exactly at limits.bytes admit; one byte
    above returns incomplete/bytes at the first reference requiring the excess, before
    hashing. Repeated pairs charge bytes once, unused evidence charges nothing, each
    visited reference consumes one work visit, and one below required work returns
    incomplete/work at the first uncharged reference.
- id: FR-038-AC-440 (planned until IR-1273 lands)
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Region membership compares all three reference members, nominal owners
    join the pair alone, a bad repeated integrity claim refuses source-integrity,
    and a valid but unlocked pair refuses invalid_source_map. Equal admitted packages
    and their lowering retain equal source references including integrity.
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for the spec-first proposal; reader implementation remains blocked on the stated QSpec owner amendments. Existing reader/API source was inspected, not executed. No Cargo gate applies to this Markdown-only change.
