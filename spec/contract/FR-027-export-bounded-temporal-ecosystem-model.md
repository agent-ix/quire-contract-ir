---
id: FR-027
title: "Export a bounded non-authoritative temporal ecosystem model"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-016
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-025
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-026
    type: depends_on
  - target: ix://agent-ix/tl-syntax/IF-006
    type: implements
  - target: ix://agent-ix/tl-syntax/VO-008
    type: references
---
# FR-027: Export a bounded non-authoritative temporal ecosystem model

## Description

When an exact merged campaign manifest is selected, the Contract IR ecosystem-model subsystem SHALL strict-read that manifest and export one deterministic bounded description of the temporal ecosystem without executing, authorizing, accepting or certifying any described component.

The input contract is
`quire.contract.temporal-ecosystem-manifest/v1`; the output contract is
`quire.contract.temporal-ecosystem-model/v1`. Both publish immutable schema
bytes and lowercase SHA-256 schema digests.

## Subsystem and public API

The implementation SHALL be organized as
`ecosystem_model::{manifest,graph,document,reader}` over the shared
`contract`, `canonical`, `identity`, `limits` and `diagnostic`
foundations. It exposes:

```text
ecosystem_model::manifest::read(bytes, ExpectedCampaign, Limits)
  -> Result<CheckedManifestSet, ModelDecision>
ecosystem_model::export(&CheckedManifestSet, Limits)
  -> Result<ModelDocument, ModelDecision>
ecosystem_model::read(bytes, &CheckedManifestSet, Limits)
  -> Result<ValidatedEcosystemModel, ModelDecision>
```

`CheckedManifestSet` and `ValidatedEcosystemModel` have no public
constructors. `ModelDocument` exposes exact canonical bytes, a content
identity and the enclosing byte digest.

## Manifest admission

The manifest fixes campaign identity, format version and the complete sorted
distinct populations of:

- the nine repository identities and exact 40-character lowercase merged
  revisions;
- runtime components and their owning repository;
- shared semantic objects, executable interfaces and exact owner contract
  selections;
- requirements, test cases and review artifacts by immutable identity and
  revision; and
- typed ownership, runtime-dependency, normative-reference, consumption and
  verification edges.

The manifest reader rejects an absent campaign member, duplicate or dangling
node, multiple executable owners for one contract, unknown node/edge kind,
moving revision, dependency cycle, self-edge where forbidden, schema mismatch,
or any edge whose endpoints do not admit that relation.

## Model construction and reading

Export sorts every node by `(kind, identity)` and every edge by
`(kind, source, target)`. The model retains the complete manifest selection,
node/edge populations, exact contract schema digests, runtime and normative
dependency directions, implementation/test/review links, unresolved gaps and
its effective limits. It adds no fact not present in the checked manifest
except deterministic adjacency and topological-order projections.

The model identity is lowercase SHA-256 over
`quire.contract.temporal-ecosystem-model/v1`, a zero byte and canonical model
bytes with `identity` omitted. The reader strict-reads the model, re-exports
from the independently validated manifest and requires byte equality.

Unknown, duplicate, missing or out-of-order fields; trailing data;
noncanonical JSON; identity/digest mismatch; graph disagreement; same identity
on unequal bytes; and every foreign campaign/revision/contract selection are
refused with no partial view.

## Bounds and failure

Limits independently bound manifest/model bytes, JSON depth, string bytes,
repositories, components, objects, interfaces, contracts, requirements, tests,
reviews, edges and total visited fields. Work is charged before retention or
graph traversal. Exact limits succeed; one-over and allocation failure return
one deterministic typed decision and no partial manifest, model, adjacency
list or topological order. Caller limits may lower but not raise owner maxima.

## Authority boundary

The model is descriptive output only. It cannot be supplied to contract
admission, owner readers, evaluators, result producers, evidence acceptance or
release decisions. It cannot mark a requirement, test, review, implementation,
result or itself accepted. An improvement is an external proposal that must
enter the ordinary owner specification, review, merge and exact-selection
path. The subsystem performs no network lookup, Markdown/source-language
parsing, semantic evaluation, plugin/callback invocation or ambient repository
discovery.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-027-AC-1 | The exact nine-repository manifest strict-reads and exports byte-identical models under input permutation; each semantic node/edge/revision/contract mutation changes the applicable model identity. | Test (TC-040) |
| FR-027-AC-2 | Missing, duplicate, dangling, multiply owned, ill-typed, cyclic, self-forbidden or moving-revision graphs refuse before a validated manifest or partial model is returned. | Test (TC-040) |
| FR-027-AC-3 | Model reading re-exports from the independently validated manifest and rejects every byte, identity, schema, campaign, revision, count, adjacency and topological-order disagreement. | Test (TC-040) |
| FR-027-AC-4 | Exact and one-over bounds cover every byte/depth/string/node/edge/work dimension, and allocation failure yields one deterministic decision with no retained partial graph. | Test (TC-040) |
| FR-027-AC-5 | No public constructor, trust flag, callback, parser, evaluator, network lookup or ambient discovery can create a checked manifest/model or route model output into an authority or acceptance input. | Test (TC-040) |
| FR-027-AC-6 | An improvement proposal retains its source model identity but has no acceptance state and becomes effective only through a distinct externally reviewed owner revision and contract selection. | Test (TC-040) |

## Dependencies

FR-016 supplies canonical encoding and digest rules. FR-025 and FR-026 supply
the complete bridge objects described by the model. The shared semantic objects
are selected from exact merged `quire-specification` revisions; owner
contracts remain in their executable repositories. The application interface
and non-authority boundary are defined by `tl-syntax` IF-006 and ADR-003.

## Status

Implemented by PLAN-007 and TC-040 for `quire-contract-ir#74`, the Contract IR
allocation of `tl-syntax` PLAN-010 Task-011.
