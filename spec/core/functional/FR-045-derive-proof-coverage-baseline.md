---
id: FR-045
title: "Derive a qualified proof-coverage baseline"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-004
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-029
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-030
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/AD-008
    type: references
---
# FR-045: Derive a qualified proof-coverage baseline

## Description

When a proof-coverage baseline is requested, the Contract IR accounting boundary shall
derive candidate claims from authoritative requirement, obligation, and
production-source inventories, join qualified producer results, and report distinct
requirement, module, function, and obligation projections with reasons for gaps.

## Inputs

- Selected source state and its owning requirement/criterion, semantic-obligation,
  production-module, and production-function inventories.
- Reviewed eligibility policy and explicit claim-to-obligation-to-source mappings.
- Capability-negotiation dispositions and pre-execution refusals with their producer,
  stage and reason; producer execution results with method, source/profile, proof
  subject, assumptions, declared/effective bounds, run identity, terminal state and reason.

## Outputs

One deterministic baseline with the candidate claim rows, eligibility decisions,
evidence joins, per-level numerators and denominators, and all gaps and exclusions.

## Behavior

The boundary shall key each candidate by owning criterion, semantic obligation,
source identity at the selected source state, and declared domain, and enumerate
candidates and eligibility before considering evidence. It shall keep method, proof
subject, backend/profile revision, assumptions and run identity in joined evidence,
so adding or changing an attempt cannot alter the source-derived denominator. The
boundary shall preserve each candidate that lacks a mapping, capability decision or result
with an explicit reason. It shall join an FR-029 `unsupported` negotiation
disposition without requiring an artifact, run or Kani outcome, and shall record
pre-execution refusals with their producer and stage. It shall reject duplicate or
mismatched result joins as proof credit without dropping the candidate. It shall
credit a Kani result only when its exact candidate, source/profile, proof subject,
declared domain and effective bounds
match, its terminal result is proved, and its nonvacuity and production reachability
evidence identify a load-bearing check for that claim. It shall record the selected
method and evidence strength separately from the claim's eligibility.

The boundary shall treat a bound narrower than the declared domain, a shadow proof, and sampled
native refinement as scoped or conditional evidence. A shadow proof with no
refinement shall not receive unqualified production-proof credit. LLVM vacuity
observations, test tags and SUCCESS-check totals alone shall not receive proof credit.
Each projection shall report its own eligible denominator, qualified numerator,
missing/refused/inconclusive and other terminal gaps, exclusions and reasons. Within
one method and declared domain, a criterion, module or function shall receive full
credit only when it has at least one eligible mapped obligation and every such
obligation qualifies; zero-mapping and partly proved entities shall remain gaps,
with qualified subsets shown where present.
No numeric target or floor is specified by this requirement.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-045-AC-1 | From a fixture whose inventories contain two criteria, three obligations and two production functions in one module, the baseline emits every source-semantic candidate and explicit mappings or unmapped reasons before joining results; removing a result or adding/changing a method, proof subject or backend/profile attempt changes evidence or gaps but not any eligible denominator. | Test |
| FR-045-AC-2 | A proved production Kani result credits only its exactly matched claim and declared finite domain when a load-bearing check and production reachability are evidenced; a zero-check result, tag-only record, LLVM probe-only record, duplicate result, or mismatched source/profile/subject/domain receives no proof credit and has a stated reason. | Test |
| FR-045-AC-3 | An effective bound narrower than declared is reported only for that narrower domain; a shadow proof without refinement is a production-proof gap, and a shadow proof with separately executed sampled native refinement is labeled conditional and sampled, never unqualified production-proof credit. | Test |
| FR-045-AC-4 | Requirement, module, function and obligation reports have separately derived eligible denominators and qualified numerators; an eligible entity with zero eligible mapped obligations remains a visible mapping gap, one with one of two eligible mapped obligations proved remains a partial gap, and no cross-level or cross-method blended percentage is emitted. | Test |
| FR-045-AC-5 | An FR-029 `unsupported` negotiation disposition remains visible with its capability and reason and requires no Kani outcome or run; pre-execution and Kani refusals retain producer and stage. Missing, counterexample, invalid/incomplete input, unavailable, timed out, resource exhausted, cancelled, inconclusive and unfamiliar producer states remain visible with reasons; an ineligible candidate remains visible with its policy reason and is excluded only from its applicable denominator. | Test |
| FR-045-AC-6 | Repeating a baseline on unchanged authoritative inventories, policy and results produces the same rows and totals; changing an inventory updates candidates without editing a separate list of file hashes, SHAs, tool pins or snapshots. | Test |

## Dependencies

[StR-004](../stakeholder/StR-004-reviewable-proof-coverage.md) states the reviewer
need. [AD-008](../../assurance/AD-008-proof-coverage-accounting.md) allocates
owner roles and proof strength. [FR-029](../../kani/functional/FR-029-versioned-bounded-kani-profile.md)
and [FR-030](../../kani/functional/FR-030-bounded-kani-domain-and-outcomes.md)
govern IR Kani profile and outcomes. Codegen ADR-003, FR-004 and FR-028 govern
its production/shadow/refinement and LLVM vacuity evidence; their current
implementation status does not satisfy this requirement.

## Status

Planned. The first measured baseline and independently reviewed eligibility
policy are prerequisites to any numerical goal.
