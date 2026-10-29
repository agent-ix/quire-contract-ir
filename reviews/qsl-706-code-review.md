---
id: SR-8468
title: code-review review of typed source selections
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md,
  spec/checked_package/matrix/TC-044-complete-v1-contract-package-lowering.md
review_set: base
---

## Summary

Ticket: QSL-706. Frozen spec-only PR 368. FAIL. Reviewed typed input ownership, exact row equality, refusal ordering, all four vector classes and unchanged lock preservation. No production implementation or runtime qualification is claimed. Optional analyses are pending the user selection.

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
    \ rules):\n\n```json\n{\n \"T\": {\"role\": \"temporal_profile\", \"definition\"\
    : {\"authority\": \"agent-ix\", \"identity\": \"quire.temporal.infinite-trace/v1\"\
    }},\n \"D\": {\"role\": \"temporal_profile\", \"definition\": {\"authority\"\
    : \"agent-ix\", \"identity\": \"quire.temporal.event-position.false-extension/v1\"\
    }},\n \"U\": {\"role\": \"protocol_profile\", \"definition\": {\"authority\"\
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

## Verdict

FAIL. Resolve every finding before merge. Existing CheckedSelection and CompleteLoweringProfileV2.reference reuse is grounded in the actual model source. Exact row membership preserves unrelated/reordered lock rows and source identity; the four requested vectors are explicitly present. Canonical semantic identity is the existing proof-content binding, not new file/version tracking.

## Method and limits

Base checklist and language-independent code review only. No applicable AssuranceProfile found. No new user story, mutable lifecycle, security boundary or performance target is claimed. Runnable implementation bindings are deferred by the explicitly spec-only scope. Quire matrix was inspected; authoring additional executable tests is outside this review. No build or gate was run.

## Dispositions

Round 1 reviewed. The original findings above remain unchanged.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed in the reviewed PR. |

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
```

### Round 1 evidence

Both new criteria now appear in actual Quire matrices as untagged, consistent with this specification-only slice; implementation coverage is not claimed. QSpec owns request/requirement/candidate charges, no reset, exact boundary vectors and unchanged source provenance. IR invokes that contract and owns only typed adapter assertions. Whole copied vectors were removed. No source or build was changed by review.

Current disposition verdict: PASS.
