---
id: FR-017
title: "Handle schema evolution and classify trace coverage"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-016
    type: depends_on
---
# FR-017: Handle schema evolution and classify trace coverage

## Description

The library shall reject unsupported schema versions, apply only registered
explicit migrations, and classify artifact traces as shallow, deep, uncovered,
or orphaned.

## Inputs

Schema identity, migration request, current package identities, and artifact
trace references.

## Outputs

A supported package or structured version diagnostic, plus a coverage class for
every current requirement and referenced artifact.

## Behavior

Unknown majors never receive best-effort interpretation. Migrations state their
source and target versions and preserve prior identity provenance. A trace to a
missing or stale requirement revision is orphaned and never contributes covered
status.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-017-AC-1 | Unknown majors and unregistered migration paths fail before semantic interpretation. | Test (TC-017) |
| FR-017-AC-2 | Orphaned and stale-revision artifacts are reported separately and cannot make any current requirement appear covered. | Test (TC-017) |

## Dependencies

FR-011 defines revision identity; FR-016 defines stable digests.
