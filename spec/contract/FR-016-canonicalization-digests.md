---
id: FR-016
title: "Canonicalize contracts and compute stable identities"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-002
    type: traces_to
---
# FR-016: Canonicalize contracts and compute stable identities

## Description

The library shall define one canonical encoding and SHA-256 identity for every
supported package, requirement revision, clause, declaration, and expression.

## Inputs

A validated v0.1 package and the canonicalization profile identity.

## Outputs

Canonical UTF-8 JSON bytes and lowercase SHA-256 digests for every addressable
semantic object.

## Behavior

Object members use fixed ordering, sets use semantic-identity ordering, strings
use JSON escaping, integers use minimal decimal form, and insignificant source
formatting is excluded. Semantic operator order remains unchanged unless the
specification declares the construct unordered.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-016-AC-1 | Equivalent supported inputs with different authored member or set order produce byte-identical canonical encodings and digests. | Test (TC-017) |
| FR-016-AC-2 | A semantic change to a clause changes that clause, requirement, and package digest while unrelated clause digests remain stable. | Test (TC-017) |

## Dependencies

FR-011 through FR-015 define canonicalized semantic content.
