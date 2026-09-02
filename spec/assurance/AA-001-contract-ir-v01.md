---
id: AA-001
title: "quire-contract-ir v0.1 source candidate argument"
type: AssuranceArgument
status: active
owner: kreneskyp
profile: ix://agent-ix/quire-contract-ir/AP-001
top_claim:
  id: claim-v01-source
  statement: the identified quire-contract-ir v0.1 source candidate is suitable for an independently authorized source tag within the declared profile
  subject: no source-release candidate selected; Wave 1 implementation baseline 5c49ebfd1c87415f74420ad047392bd03b1bd202 is complete
  status: open
reasoning:
  - id: reasoning-semantic-contract
    statement: require complete requirements, validated architecture, deterministic conformance evidence, and independent review before the claim can be supported
    supports: claim-v01-source
    sufficiency_criteria: [all native child issues are Done, every matrix criterion is backed, the complete corpus and golden identities pass, code review and gap analysis have no blocking finding, the human owner records an exact candidate decision]
assumptions:
  - id: assumption-downstream-independence
    statement: downstream repositories consume the published contract without redefining its semantics
    owner: kreneskyp
    status: open
    review_by: "2026-12-31T00:00:00Z"
  - id: assumption-toolchain-reproduction
    statement: tool and environment identities remain obtainable for independent reproduction from the Quoin evidence store a run is recorded into, not from this repository, which retains none for the pre-stable phase
    owner: kreneskyp
    status: open
    review_by: "2026-12-31T00:00:00Z"
participants:
  - id: kreneskyp
    role: source-release decision owner
    authority: approve reject or defer one exact source candidate and tag
    independence: human review is separate from agent-authored implementation and evidence
challenges:
  - id: challenge-wave4-release-evidence
    target: claim-v01-source
    statement: cross-platform and independent downstream evidence plus the human source decision remain assigned to PGM-02 Wave 4
    status: open
    owner: kreneskyp
relationships:
  - target: ix://agent-ix/quire-contract-ir/AP-001
    type: references
  - target: ix://agent-ix/quire-contract-ir/MP-001
    type: references
---
# quire-contract-ir v0.1 source candidate argument

## Claim

The source-release claim remains intentionally open. Issues #5, #6, #8, #9,
and #10 are complete, and merge commit
`5c49ebfd1c87415f74420ad047392bd03b1bd202` is the Wave 1 implementation
baseline. It is not selected as a source-release candidate by this artifact.

## Reasoning

Support requires traceable requirements, a bounded architecture, complete
semantic and conformance implementation, deterministic retained measurements,
closed blocking review findings, explicit assumptions, and a named human
sufficiency decision for one revision.

## Sufficiency Decision

No source-release sufficiency decision has been recorded. The program handoff
assigns human release decisions and source tags to PGM-02 in Wave 4 after all
eight repository epics complete. Closing the Wave 1 implementation epic does
not change the top claim from `open`, and automated checks or agent-authored
evidence cannot do so later.

## Challenges

The semantic implementation, schemas, and corpus are now present. Their child
evidence is no longer retained: the repository owner released the
evidence-preservation constraint for the pre-stable phase on 2026-09-02
(agent-ix/engineering-assurance#7) and this repository's retained records are
deleted. Nothing in this argument rests on them, and none is restated as
though it still verified. Retention returns as a support for this claim at the
move toward stable releases, which is before any source-release candidate is
selected. Cross-platform golden comparison remains deferred
while automatic CI is intentionally off; independent downstream execution and
the exact human source decision also remain absent. REV-007 carries these
limitations into the Wave 4 decision without treating them as Wave 1
implementation failures or as successful release evidence.
