---
id: FR-011
title: "Identify versioned contract packages and requirement revisions"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
---
# FR-011: Identify versioned contract packages and requirement revisions

## Description

The model shall identify a contract package, its schema version, every
requirement ID and revision, and the source document revision from which it was
derived.

## Inputs

Package namespace, schema major/minor, requirement identifier, monotonic
requirement revision, and source-document identity.

## Outputs

Validated package and requirement identities suitable for serialization and
downstream citation.

## Behavior

Identity validation rejects empty namespaces, malformed identifiers, zero
revisions, duplicate requirement revisions, and references to a different
package. Changing a requirement revision changes every derived downstream
identity that cites it.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-011-AC-1 | A package round trip preserves namespace, schema version, requirement ID, requirement revision, and source-document identity exactly. | Test (TC-015) |
| FR-011-AC-2 | Incrementing one requirement revision changes its clause and dependency identities without changing unrelated requirement identities. | Test (TC-015) |

## Dependencies

PGM-01-R01 governs compatibility and migration behavior.
