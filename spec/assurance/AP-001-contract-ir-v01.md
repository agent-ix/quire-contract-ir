---
id: AP-001
title: "quire-contract-ir v0.1 assurance profile"
type: AssuranceProfile
status: proposed
owner: kreneskyp
profile_version: 0.2
profile_kind: general
scope: one identified quire-contract-ir v0.1 source candidate and its declared schema/corpus/tool environment
impact_assessments:
  - id: impact-semantic-drift
    scenario: two downstream lowerings interpret one contract differently
    severity: material
    verifiability:
      class: cheap-conclusive
      stochastic_dependency: none
    detect_before_harm:
      expected: true
      control_ref: ix://agent-ix/quire-contract-ir/CAC-001
  - id: impact-false-coverage
    scenario: a stale or orphaned artifact makes a current requirement appear covered
    severity: material
    verifiability:
      class: cheap-conclusive
      stochastic_dependency: none
    detect_before_harm:
      expected: true
      control_ref: ix://agent-ix/quire-contract-ir/CAC-001
  - id: impact-unstable-identity
    scenario: equivalent contracts produce different canonical identities
    severity: moderate
    verifiability:
      class: cheap-conclusive
      stochastic_dependency: none
    detect_before_harm:
      expected: true
      control_ref: ix://agent-ix/quire-contract-ir/MP-001
review_policy:
  mode: require
  operations: [spec-review, code-review, gap-analysis]
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---
# quire-contract-ir v0.1 assurance profile

## Decision Boundary

The decision concerns one identified source candidate, schema/corpus revision,
canonicalization profile, toolchain, and local verification environment. It can
support an independently owned source-tag decision. It does not approve a
downstream code generator, solver analysis, runtime, consuming project, or
later revision.

## Impact Scenarios

Material scenarios are semantic drift between independent lowerings, false
coverage from stale identities, silent undefined behavior, and unstable
canonical digests. Controls are deterministic validation, independent schema
fixtures, golden digests, orphan classification, review, and retained evidence.

## Evidence Policy

Evidence identifies the candidate revision, all material inputs and outputs,
tool/dependency versions and digests, environment, commands, individual
outcomes, review findings, limitations, and skipped checks. Self-attesting local
evidence is labeled as such. An automated pass never closes the release claim.

## Exceptions

An exception names its bounded scope, affected impact, owner `@kreneskyp`,
rationale, timestamp, expiry or one-time action, and compensating evidence. No
exception authorizes registry publication or project accreditation.
