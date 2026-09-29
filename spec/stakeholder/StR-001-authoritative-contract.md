---
id: StR-001
title: "Author one semantic contract for every lowering"
type: StR
---
# StR-001: Author one semantic contract for every lowering

## Stakeholder Need

Requirement authors need one versioned semantic contract whose identity,
expressions, definedness conditions, and execution anchors survive every
downstream lowering without hand-authored semantic copies.

## Rationale

Independent oracle, proof, generator, and solver models drift. A shared source
model makes disagreements observable and gives every derived artifact the same
requirement and revision identity.

## Validation Criteria

| ID | Criteria | Validation |
|---|---|---|
| StR-001-VC-1 | Every executable clause has a stable requirement revision, named anchor, typed expression, dependency set, and source location. | Contract-model inspection and conformance fixtures (TC-015, TC-016) |
| StR-001-VC-2 | Downstream artifacts can cite the same package, requirement, clause, and expression identities. | Interface and canonicalization review (TC-015, TC-017) |

## Stakeholders

Requirement authors, code-generation maintainers, analysis maintainers, and
assurance reviewers.
