---
id: FR-032
title: "Admit one exact output-mapping request"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-023
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-028
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-120
    type: implements
  - target: ix://agent-ix/quire-specification/FR-297
    type: implements
  - target: ix://agent-ix/quire-specification/NFR-060
    type: constrained_by
---
# FR-032: Admit one exact output-mapping request

## Description

When admitting an output-mapping request, the Contract IR coordinator shall
derive one immutable request from a strictly read bound package, a nonempty
ordered unique obligation selection, exact source/model/semantic selections,
one target profile, and explicit aggregate limits before mapper dispatch.

## Inputs

- One constructor-private `BoundPackage` produced by strict executable-projection admission.
- An ordered nonempty list of executable clause identities and their exact source-fact states.
- Exact native, model, and semantic selection identities, revisions, and raw digests.
- One target family, ordered target-standard references, mapping profile identity,
  mapping revision, raw mapping-rule digest, and closed requested-capability set.
- Explicit request-byte, obligation, expression-node, nesting-depth, mapping-work,
  record, and emitted-byte limits.

## Outputs

- One immutable admitted request exposing read-only source obligations, selections,
  target profile, and limits.
- One typed refusal with a stable code and affected field path, with no mapper
  invocation or target bytes.

## Behavior

- The coordinator shall resolve each selected clause through the supplied
  `BoundPackage` and retain its clause identity, kind, anchor, source span,
  declaration digest, expression digest, and ordered dependencies.
- The coordinator shall reject an empty, duplicate, informational, missing,
  stale, foreign, or reordered-after-admission obligation selection.
- The coordinator shall accept only the fixed FS06 target-family/profile pairs
  and revision `1-draft.1` without inferring a target, revision, digest,
  capability, tool, observer, path, or default.
- The coordinator shall preserve native/model/semantic selections independently
  from the selected target profile.
- The coordinator shall reject zero, exceeded, or arithmetically overflowing
  limits before mapper dispatch whenever the nonempty request cannot fit.
- The coordinator shall expose no partially admitted request after validation,
  allocation, cancellation, or resource failure.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-032-AC-1 | A valid request retains the exact bound package, ordered unique executable obligations, three source selections, one matching target profile, closed capabilities, and all limits. | Test (TC-043) |
| FR-032-AC-2 | Empty, duplicate, informational, missing, stale, foreign, cross-family, unknown, omitted, zero-limit, over-limit, and overflowed inputs refuse before mapper dispatch with no target bytes. | Test (TC-043) |
| FR-032-AC-3 | Target family, standard references, mapping revision/digest, capability, and native/model/semantic selection mutations change request equality or refuse admission. | Test (TC-043) |
| FR-032-AC-4 | Path, timestamp, locale, display text, installed software, observer state, and previous requests cannot supply or alter an admitted semantic selection. | Test (TC-043) |

## Dependencies

- [FR-023](FR-023-executable-projection-binding.md) supplies the strict bound-package
  and bound-clause boundary.
- [FR-028](FR-028-separate-cycle-free-contract-model.md) requires this target-neutral
  contract to remain in the cycle-free model crate.
- `ix://agent-ix/quire-specification/FR-120`, `FR-297`, and `NFR-060` are the
  accepted FS06 request, profile, and resource authorities.
