---
id: SR-8471
title: integrity review of typed source selections
type: SpecReview
analysis: integrity
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md,
  spec/checked_package/matrix/TC-044-complete-v1-contract-package-lowering.md
review_set: subset
---

## Summary

Ticket: QSL-706. Original frozen candidate, before any fix. FAIL. Reviewed typed ownership, exact-row semantics, source preservation, refusal precedence and the integrity of the paired contract. Findings overlap the original base/code review classes and are recorded by this lens independently.

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
- id: FR-035-AC-9
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: '| FR-035-AC-9 | `CompleteLoweringProfileV2.required_source_selections`
    reuses typed FR-322 selections; matching role/authority/identity rows admit, empty
    requirements impose no restriction, unrelated rows and repeated/reordered requirements
    do not affect the predicate, and missing or different rows produce request-ordered
    `InvalidInput { node_id }`/`invalid_input` before node traversal without new payload,
    nodes or backend artifacts. Both success and refusal preserve the exact admitted
    source id, lock contents/order and available request source occurrences. | Test
    |'
- id: TC-044
  path: spec/checked_package/matrix/TC-044-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: "\n## Typed source-selection vectors\n\nUse these exact FR-322 `Selection`\
    \ rows (the short labels below denote typed\nvalues, never identifier-parsing\
    \ rules):\n\n```json\n{\n  \"T\": {\"role\": \"temporal_profile\", \"definition\"\
    : {\"authority\": \"agent-ix\", \"identity\": \"quire.temporal.infinite-trace/v1\"\
    }},\n  \"D\": {\"role\": \"temporal_profile\", \"definition\": {\"authority\"\
    : \"agent-ix\", \"identity\": \"quire.temporal.event-position.false-extension/v1\"\
    }},\n  \"U\": {\"role\": \"protocol_profile\", \"definition\": {\"authority\"\
    : \"agent-ix\", \"identity\": \"quire.protocol.finite-global/v1\"}}\n}\n```\n\n\
    Construct independently admitted FR-"
- id: TC-044
  path: spec/checked_package/matrix/TC-044-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: '| --- | --- | --- | --- |

    | matching | `[T]` | `[T]` | Both requested items lower; source lock and package
    id equal that input exactly. |

    | missing | `[T]` | `[]` | Exactly `invalid_input(b)`, `invalid_input(a)`; no
    semantic nodes; both exact request occurrence sets retained. |

    | differing identity | `[T]` | `[D]` | The same two `invalid_input` records; no
    substitute `T`, no node or backend artifact; output still retains `[D]` and its
    originating package id. |

    | unrelated selection | `[T]` | `[U, T]` | Both items lower; output retains `[U,
    T]` in that order, including the unrelated row. |


    Rep'
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | TC-044 copies the entire QSpec TC-935 typed source-selection vector section, and FR-035 lines71-97 independently restate the QSpec FR-330 predicate and refusal rules. These have two normative owners. Keep the authoritative rules/vectors in QSpec, reference them from IR, and retain only the IR-owned typed adapter and conformance obligations. | spec/checked_package/matrix/TC-044-complete-v1-contract-package-lowering.md:55-100 |
| FND-002 | medium | The compatibility pass can exhaust work, but its charge units, counter lifetime and order relative to FR-038 per-request first charge are undefined. Define whether comparison is global or charged per request, preserve zero-work failed precedence, and add exact-limit/one-under controls without a fresh budget. | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:83-90 |

## Traceability and assumption probes

| Source | Requirement | Stakeholder | Verification |
| --- | --- | --- | --- |
| Consumer contract FR-330/I12 | FR-035 | StR-001 | TC-044 adapter of TC-935, Test; implementation deferred |

No new NFR, CLI, registry, pagination, authenticated API, scaffolder or mutable callback is introduced. Row identity is explicit role plus full DefinitionRef; duplicates and extra rows have a stated policy. No fallback implementation is claimed. Shared typed values are reused. Identity and source preservation have one exact oracle; no caller operation rewrites the admitted lock. A finite row scan needs the explicit work integration identified above; existing graph-cycle rules remain unchanged. No additional failure-domain finding was identified.

## Verdict

FAIL. Resolve the recorded findings; this artifact reviews only the original head. Fix disposition belongs in an appended section. No build or runtime qualification was performed.

## Dispositions

Round 1 reviewed. The original findings above remain unchanged.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed in the reviewed PR. |
| FND-002 | fixed | Fixed in the reviewed PR. |

```yaml
dispositions:
- fnd: FND-001
  outcome: fixed
  after_excerpt: 'Use QSpec [TC-935](ix://agent-ix/quire-specification/TC-935) as
    the single owner

    of the matching, missing, differing-identity and unrelated-selection fixtures,

    empty/duplicate/order controls and exact/one-under compatibility work vectors.

    Do not copy or independently redefine their selection values or expected

    matching semantics here.'
- fnd: FND-002
  outcome: fixed
  after_excerpt: 'QSpec FR-330''s "Typed source-selection compatibility" is the sole
    owner of

    source-selection matching, its charged per-request procedure, refusal precedence

    and exact lock preservation. The lowerer SHALL implement that contract using

    its typed `CompleteLoweringProfileV2.required_source_selections` field and

    existing FR-038 per-request work meter; it SHALL NOT introduce a separately

    normalized requirement representation, replacement budget or reference-derived

    requirement lookup.'
```

### Round 1 evidence

Both new criteria now appear in actual Quire matrices as untagged, consistent with this specification-only slice; implementation coverage is not claimed. QSpec owns request/requirement/candidate charges, no reset, exact boundary vectors and unchanged source provenance. IR invokes that contract and owns only typed adapter assertions. Whole copied vectors were removed. No source or build was changed by review.

Current disposition verdict: PASS.
