---
id: SR-8470
title: ears-conformance review of typed source selections
type: SpecReview
analysis: ears-conformance
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
review_set: subset
---

## Summary

Ticket: QSL-706. Original frozen candidate, before any fix. CONDITIONAL. Manual statement review supplements the engine check. The engine emitted no warnings for the changed FR; manual review found the wording defects below.

## Examined scope

```yaml
scope:
- id: FR-035
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: '

    An immutable admitted `quire.checked-package/v2`, its selected admitted

    dependency packages, requested item set, resolved `CompleteLoweringProfileV2`

    and resource limits. The resolved profile owns `reference: LoweringProfileRef`,

    `required_source_selections: Vec<CheckedSelection>`, and its existing

    `supported_tags`, `require_bounds`, and `work_limit`. `CheckedSelection` is the

    existing typed FR-322 `Selection` `{role, definition}`, where `definition` is

    `DefinitionRef` `{authority, identity}`. The producer of the resolved profile

    supplies its declared requirements directly; no lookup from the'
- id: FR-035
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: 'After checked-input admission and before per-item semantic-node traversal
    or

    emission, the lowerer SHALL check every `required_source_selections` row against

    the admitted input''s `lock.profile_selections`. A row matches only if its typed

    `role`, `definition.authority` and `definition.identity` all equal one lock row

    exactly, without aliases, case folding, prefixes, subtype or digest comparisons.

    This is conjunction of row membership, not array equality or a role-only lookup.

    An empty requirement vector imposes no restriction; duplicate identical

    requirements and either vector''s order do not af'
- id: FR-035
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: 'When any requirement is missing or differs from all candidate rows, the
    lowerer SHALL return `CompleteLoweringRecordV2::InvalidInput { node_id }` for
    every

    requested item in request order before unsupported-tag, bounds or body checks.

    The FR-330 projection is `invalid_input` with only the existing requested

    `node_id` payload, no new diagnostic/pointer field, no emitted semantic nodes,

    and the exact available source occurrences (empty for an absent key). No backend

    artifact is produced. Existing limits remain enforced: stopping the compatibility

    check uses the existing `failed` result, and a pa'
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The new compatibility requirement starts with After rather than the EARS event trigger When. Express admission completion as a named event while retaining pre-traversal ordering. | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:71-73 |
| FND-002 | medium | The preservation sentence joins two SHALL obligations with a semicolon. Use one concrete preservation obligation or separate atomic sentences so each response is independently traceable. | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:93-95 |

## Engine and semantic check

IR original full-spec validation exited 0; 85/85 documents grammar-clean, zero engine grammar findings. Three newly authored requirement sentences examined: the mismatch response is concrete and clean; two manual grammar findings remain. I12 and TC vectors are not requirement-bearing FR/NFR/StR statements and are outside this lens.

## Verdict

CONDITIONAL. Resolve the recorded findings; this artifact reviews only the original head. Fix disposition belongs in an appended section. No build or runtime qualification was performed.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | The replacement contract-adoption sentence joins two SHALL obligations with a semicolon. Split implementation and prohibition into separate atomic sentences so each response is independently traceable. | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:71-77 |

## Dispositions

Round 1 reviewed. The original findings above remain unchanged.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed in the reviewed PR. |
| FND-002 | fixed | Fixed in the reviewed PR. |
| FND-003 | still-open | The replacement FR-035 sentence combines "the lowerer SHALL implement" and "it SHALL NOT introduce" with a semicolon. Split the two atomic obligations into separate sentences without changing meaning. |

```yaml
dispositions:
- fnd: FND-001
  outcome: fixed
  after_excerpt: 'QSpec FR-330''s "Typed source-selection compatibility" is the sole
    owner of

    source-selection matching, its charged per-request procedure, refusal precedence

    and exact lock preservation. The lowerer SHALL implement that contract using

    its typed `CompleteLoweringProfileV2.required_source_selections` field and

    existing FR-038 per-request work meter; it SHALL NOT introduce a separately

    normalized requirement representation, replacement budget or reference-derived

    requirement lookup.'
- fnd: FND-002
  outcome: fixed
  after_excerpt: 'The output adapter retains `profile.reference` as the sole emitted

    lowering-profile value and retains the original checked source provenance under

    FR-330, without adding a requirement member, diagnostic payload or vocabulary.'
- fnd: FND-003
  outcome: still-open
  reason: The replacement FR-035 sentence combines "the lowerer SHALL implement" and
    "it SHALL NOT introduce" with a semicolon. Split the two atomic obligations into
    separate sentences without changing meaning.
new_findings:
- fnd: FND-003
  severity: medium
  confidence: high
  method: spec-review/ears-conformance
  check: compound
  artifact_id: FR-035
  related_ids: []
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  lines: 71-77
  excerpt: 'QSpec FR-330''s "Typed source-selection compatibility" is the sole owner
    of

    source-selection matching, its charged per-request procedure, refusal precedence

    and exact lock preservation. The lowerer SHALL implement that contract using

    its typed `CompleteLoweringProfileV2.required_source_selections` field and

    existing FR-038 per-request work meter; it SHALL NOT introduce a separately

    normalized requirement representation, replacement budget or reference-derived

    requirement lookup.'
  finding: The replacement contract-adoption sentence joins two SHALL obligations
    with a semicolon. Split implementation and prohibition into separate atomic sentences
    so each response is independently traceable.
```

### Round 1 evidence

Both new criteria now appear in actual Quire matrices as untagged, consistent with this specification-only slice; implementation coverage is not claimed. QSpec owns request/requirement/candidate charges, no reset, exact boundary vectors and unchanged source provenance. IR invokes that contract and owns only typed adapter assertions. Whole copied vectors were removed. No source or build was changed by review.

Current disposition verdict: CONDITIONAL.

### Round 2

Reviewed. Only the requested wording hunk changed; the prior matrix repair, charged predicate, work vectors and single normative ownership remain intact. No new finding or regression.

| FND | outcome | reason |
| --- | --- | --- |
| FND-003 | fixed | Fixed in the reviewed PR. |

```yaml
dispositions:
- fnd: FND-003
  outcome: fixed
  after_excerpt: 'The lowerer SHALL implement that contract using

    its typed `CompleteLoweringProfileV2.required_source_selections` field and

    existing FR-038 per-request work meter. The lowerer SHALL NOT introduce a separately

    normalized requirement representation, replacement budget or reference-derived

    requirement lookup.'
```

Current disposition verdict: PASS. All findings for this method are resolved.
