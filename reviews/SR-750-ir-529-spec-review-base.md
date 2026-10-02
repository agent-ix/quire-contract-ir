---
id: SR-750
title: "base spec review of PR 251 (IR-529 artifact references)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@0b106a64ea261a86c0f2f64eb9ab8f31f546d39d; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Artifact references section, FR-038-AC-46 through FR-038-AC-64); spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; spec/checked_package/matrix/tests.md; spec/tests.md; spec/assurance/AD-004-checked-package-seam.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---
# SR-750: base spec review of PR 251

## Summary

Ticket: IR-529. Diff: `git diff origin/main...HEAD`. The merge base equals `origin/main`
(e80ea70ab8874676de47ac1b7fb439497ced7991), so the base is current and GitHub reports
the PR as MERGEABLE.

I checked the amendment against two sources, independently of the PR body:

- **QSpec.** `agent-ix/quire-specification` origin/main c76c6aed683ee93eae04abcd26af1b5115f61b90,
  `proposals/checked-package-v2/schema.json` and FR-322.
  - `DefinitionRef` is closed `{authority, identity}` (Nonempty). It is used by
    `Selection.definition`, `PackageLock.definition_selections`,
    `IdentityPreimage.definition_selections`, `OperationLaw.definition` and
    `Diagnostics.catalog`. Through `Selection`, it is also used by `edition` and
    `profile_selections` in both `PackageLock` and `IdentityPreimage`.
  - `RawSourceRef` is closed `{authority, identity, digest_domain: const quire.source.bytes/v1, digest: ^[0-9a-f]{64}$}`
    with no revision. It is used by `PackageLock.sources`, `SourceRegion.source` and,
    through `SourceRegion`, `Diagnostic.loci`.
  - The operation catalog's `law_roles` entries on quire-verification-contracts
    origin/main are `{authority, identity}`.
- **The code** at the head sha:
  - `shared.rs` defines `CheckedArtifactRef`, `CheckedRevision` and `CheckedArtifactLocator`.
  - `common.rs` holds `decode_closed`, `validate_locked_artifact` and
    `validate_source_map_entries`.
  - `v2/mod.rs` holds `validate_lock`, `non_graph_lock_difference` and the diagnostics loci.
  - `v2/operations.rs` holds the `OperationWire` decode and the law join.
  - Also read: `v2/operation_catalog.rs` and `v2/identity.rs` (`validate_owner`).

The shapes the PR states match QSpec, and so does the dropped `export`. Lock, catalog
and source-map decoding is typed (`decode_closed`), so `unknown_member` and
`malformed_wire` are the real codes at those sites. The wire must be RFC 8785
canonical, so "document order" is key order. Under that order the five-member shape's
first extra member is `digest`, and that is consistent. Nothing is removed: the diff
only adds FR-038 rows. No pin, SHA, vendored file or compatibility layer is added, and
the old shape refuses with no legacy reader. The matrix markers are honest: 🚧 planned
pending IR-530, in the FR-038 row, the TC-048 row and spec/tests.md.

`make spec`:

| | origin/main | head |
| --- | --- | --- |
| validate | passes | passes |
| grammar findings | 1 | 20 (+19 `ac:non-canonical-shape`, see SR-752) |
| strict unbacked | 23 | 23 |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038-AC-48, AC-50 and AC-54 assign `unknown_member` and `malformed_wire` to `operation.laws[].definition`, but the reader decodes `operation` with its own decoder and refuses every shape failure there as `invalid_semantic_graph` at the decoder pointer. The node body is an untyped `Value` in the wire decode. FR-038-AC-36 already states this graph-body taxonomy (an extra member refuses `invalid_semantic_graph`). The PR says it uses the existing taxonomy, so either the law site's codes become `invalid_semantic_graph` or the change of code is stated explicitly. The TC-048 step that adds `revision` to a law `definition` and expects `unknown_member` would fail against the reader as specified elsewhere. | crates/quire-contract-model/src/checked_package/v2/operations.rs:405-412; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:842,844,848 |
| FND-002 | medium | FR-038-AC-52 says a wrong source `digest_domain` refuses `digest_domain_mismatch` "including when another member of the same row is also malformed". FR-038-AC-51 defines malformed as absent, empty, not a string or not hex. An absent or non-string member fails the typed decode first (`decode_closed`, `malformed_wire`) before the domain check in `validate_locked_artifact` runs. So the clause is true only for empty or non-hex string members. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:846; crates/quire-contract-model/src/checked_package/common.rs:316-325,436-466 |
| FND-003 | medium | "Source row" is ambiguous between `lock.sources` and region `source`, and FR-038-AC-51 and AC-52 conflict with FR-038-AC-58 for a region `source`. The prose makes `CheckedSourceRef` the type of both, and AC-49 names both explicitly. For a region `source` with an empty member, a non-hex digest or a wrong domain, AC-51 or AC-52 says `malformed_wire` or `digest_domain_mismatch`. AC-58 says `invalid_source_map`, because the region equals no lock row. The reader checks `contains` before `validate_locked_artifact`. The precedence is not stated. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:845,846,852; crates/quire-contract-model/src/checked_package/common.rs:531-537 |
| FND-004 | medium | FR-038-AC-46 lists the definition-reference sites, and AC-48's refusal is scoped to "any place FR-038-AC-46 names". The list omits `identity_preimage.edition.definition`, `identity_preimage.profile_selections[].definition` and `identity_preimage.definition_selections`, which are all `DefinitionRef` in QSpec's `IdentityPreimage`. The prose names only the definition_selections mirror. An extra member in a preimage reference is therefore unspecified, and in canonical key order it is the first one decoded. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:318-323,840,842 |
| FND-005 | medium | FR-038-AC-47 (and the prose, AC-49, AC-51 and AC-58) names only `lock.sources` and the source-map region `source` as `CheckedSourceRef` sites. `diagnostics.entries[].loci[]` are also `SourceRegion`s in QSpec (`Diagnostic.loci`) and `CheckedSourceRegion`s in the reader, and the reader joins them against `lock.sources` (invalid_source_map at `loci/n/source`). That third site is not covered. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:333-335,841; crates/quire-contract-model/src/checked_package/v2/mod.rs:337,2082-2088 |

## Verdict

Changes requested. The amendment is correct in substance against QSpec. It is explicit
about the requirement change, removes no AC and keeps every planned marker honest. One
high finding remains: the law-definition refusal codes contradict the reader's real
graph-body taxonomy. Four medium findings concern precedence and coverage of the
reference sites. All of them are spec text fixes inside this PR.

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f (delta from 0b106a64ea261a86c0f2f64eb9ab8f31f546d39d; base origin/main e80ea70ab8874676de47ac1b7fb439497ced7991 unchanged).

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f |
| FND-002 | fixed | 9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f |
| FND-003 | fixed | 9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f |
| FND-004 | fixed | 9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f |
| FND-005 | fixed | 9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f |
