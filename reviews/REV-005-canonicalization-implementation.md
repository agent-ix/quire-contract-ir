---
id: REV-005
title: "Issue #9 canonicalization implementation review"
type: Review
status: complete
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/9
    type: reviews
---
# REV-005: Issue #9 canonicalization implementation review

## Scope and independence boundary

This review covers the issue #9 implementation against FR-016, FR-017,
STD-001, NFR-001, NFR-003, and TC-017. The independent reviewer was restricted
to read-only static inspection and did not run tests or edit files. Local gate
results are separate producer claims, not independent approval.

The first implementation pass returned exactly two numbered items. Both are
retained below. One was a valid testability gap and is fixed. The other was a
critical supply-chain allegation contradicted by the pinned upstream crate's
own manifest and source; it is rejected, not silently removed or described as
a confirmed vulnerability.

## Findings

| ID | Severity | Finding | Disposition |
|---|---|---|---|
| FND-097 | critical (reported) | The reviewer alleged that `zmij` in the lockfile substituted for `serde_json`'s legitimate `ryu` dependency. | rejected as factually false: the official immutable `serde_json` v1.0.151 source manifest at commit `de8500740cdcabffb9734f503e4889def823cf10` declares `zmij = "1.0"`; the checksummed local crate manifest and source agree, and offline `cargo tree -i zmij` resolves it directly beneath the pinned `serde_json`. Forcing `ryu` would contradict upstream. |
| FND-098 | medium | The deterministic resource-failure harness was unavailable for requirement, clause, and declaration output and untested for expression output. | fixed: all five closed kinds expose `_with_limit` entry points and TC-017 forces limit-zero failure for each, returning only `canonicalization_resource_exhausted`. |
| FND-099 | medium | FND-097's rejection cited local cache and command evidence that was not independently inspectable from the repository. | fixed: this review now retains an immutable official upstream source link resolving the v1.0.151 manifest at the commit above. |
| FND-100 | medium | Resource-exhaustion diagnostics omitted available requirement, clause, and expression spans. | fixed: canonical writers retain the object's optional span and attach it to allocation failures; TC-017 asserts all three spans while package/declaration aggregates remain spanless. |
| FND-101 | medium | TC-017 did not pin normalized rational bytes/digests or authored expression-sequence ordering. | fixed: unreduced and reduced rational literals now compare byte/digest-identically with normalized fields asserted, and a reversed lexical collection item sequence is asserted to retain authored order. |
| FND-102 | low | The closed resolver-diagnostic mapping used a wildcard that could silently relabel a future code as `missing_requirement`. | fixed without a panic: the mapping returns `None` for an unexpected code and `classify_coverage` returns that original diagnostic rather than emitting a false orphan reason. |
| FND-103 | critical (repeated) | A closing reviewer could not access the network and therefore repeated FND-097 as unresolved despite locally consistent manifests and the immutable upstream link. | rejected after authoritative network verification: the crates.io API identifies `zmij ^1.0` as a normal `serde_json` 1.0.151 dependency, attributes both exact releases to verified publisher `dtolnay`, and returns checksums matching this lockfile. |
| FND-104 | critical (repeated) | The final reviewer again alleged that `zmij` replaced a supposed required `ryu` dependency because its sandbox could not fetch crates.io. | rejected: the retained real crates.io dependency response explicitly names `zmij ^1.0`; inability of a later sandbox to repeat a successful upstream request does not contradict that authoritative response. |
| FND-105 | process (reported) | The final reviewer said REV-005 vouched for itself because its sandbox could not open the retained external sources. | rejected: REV-005 records externally sourced API fields, immutable URLs, publishers, and matching checksums; it does not identify its own prose as the authority. |

## Authoritative dependency verification

On 2026-08-30 the producer queried the real crates.io API over HTTPS. The first
request without an identifying user agent returned HTTP 403 and is not used as
evidence. Retrying with user agent
`quire-contract-ir-review/0.1 (dependency verification)` succeeded:

- [`serde_json` 1.0.151 dependencies](https://crates.io/api/v1/crates/serde_json/1.0.151/dependencies)
  returned normal dependency `crate_id: "zmij"`, requirement `^1.0`, dependency
  ID `31298629`, version ID `2825732`.
- [`serde_json` 1.0.151 metadata](https://crates.io/api/v1/crates/serde_json/1.0.151)
  returned verified publisher `dtolnay` / David Tolnay and checksum
  `c841b55ecdae098c80dcae9cf767f6f8a0c2cdb3416bbef72181df4d0fe73f14`,
  exactly matching `Cargo.lock`.
- [`zmij` 1.0.23 metadata](https://crates.io/api/v1/crates/zmij/1.0.23)
  returned the same verified publisher, repository
  `https://github.com/dtolnay/zmij`, `yanked: false`, and checksum
  `29666d0abbfad1e3dc4dcf6144730dd3a3ab225bbbdac83319345b1b44ccfc1b`,
  exactly matching `Cargo.lock`.

This network evidence independently corroborates the official immutable source
manifest and disproves the substitution allegation. It is dependency
verification, not independent approval of the issue #9 implementation.

## Closing gate

The closing static reviewer explicitly reported that the issue #9 code side and
FND-098 through FND-102 were clean. Its only remaining items, retained as
FND-104 and FND-105, repeated the disproved dependency allegation and a process
variant caused by its own lack of network access. No actionable code finding is
open. FND-097/FND-099/FND-103 through FND-105 remain independently checkable in
the official immutable
[v1.0.151 manifest](https://github.com/serde-rs/json/blob/de8500740cdcabffb9734f503e4889def823cf10/Cargo.toml#L15-L20)
and crates.io API links above.

The producer's full local `make ci` gate passed formatting, Clippy with warnings
denied, the pinned governance runtime, 13/13 governance corpus cases, 7/7
mutation probes, 11 Python tests, all 31 Rust tests including four TC-017 tests,
doc tests, license policy, and the unsafe-code audit. The independent reviewer
did not execute these commands, and this review is not independent evidence
approval or source-release authorization. No GitHub Actions workflow was
dispatched.
