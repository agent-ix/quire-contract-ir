---
id: AD-008
title: "Proof coverage accounting across contract assurance owners"
type: ArchitectureDescription
status: proposed
owner: Contract IR lane
system: proof-coverage claim inventory, evidence joins, and baseline projections across Contract IR, Contract Codegen, and Contract Runtime
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-004
    type: realizes
  - target: ix://agent-ix/quire-contract-ir/FR-045
    type: realizes
  - target: ix://agent-ix/quire-contract-ir/AD-002
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-029
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-030
    type: references
  - target: ix://agent-ix/quire-contract-codegen/ADR-003
    type: references
  - target: ix://agent-ix/quire-contract-codegen/FR-004
    type: references
  - target: ix://agent-ix/quire-contract-codegen/FR-028
    type: references
---
# Proof coverage accounting across contract assurance owners

## System Boundary

This AD owns the meaning of a proof-coverage baseline and the join contract for evidence
produced by the IR, Codegen, and Runtime owners. It defines no solver, harness, proof,
numeric goal, or claim that current repositories already implement the baseline.
Contract IR owns the selected profile, finite-input firewall, and typed Kani outcome;
Contract Codegen owns obligation generation, Kani execution and transcript evidence;
Contract Runtime owns its checked-in kernel harnesses. Each owner publishes an inventory
and results from its authoritative source. Other methods enter only through an identified
producer with a declared population and outcome vocabulary; no Verus, property-test, or
monitoring population is presumed to exist.

## Views

The views below share one candidate census and differ only in their projection
keys and evidence-qualification rules.

### Claim and population

The accounting unit is a source-semantic candidate claim, keyed by its owning
requirement criterion, semantic-obligation identity, source identity at the selected
source state, and declared domain. Method, proof subject, backend/profile revision,
assumptions, run identity, and effective bounds describe evidence attempts joined to
that candidate; changing or adding any of them cannot mint or remove a candidate.
A requirement criterion can generate several obligations, and an obligation can mention
several source modules or functions. These are explicit many-to-many projection links,
with a recorded reason when there is no production function. A
function is credited only when the evidence identifies a load-bearing assertion or
property about its production behavior and establishes the function's reachability in
the proved path; incidental call-graph presence is insufficient. Duplicate evidence
for the same claim never creates another claim or additional credit.

Before reading successful results, the baseline derives the candidate population from
the owning requirement/criterion and semantic-obligation inventories and the production
source/module/function inventories at the selected source state. Each candidate records
eligibility or an explicit ineligible reason under a reviewed policy. A candidate with
missing mapping, producer, capability, or result remains visible as a gap; an absent
artifact cannot silently shrink the denominator. The inventory is recomputed from
authoritative inputs when the baseline runs. No manually maintained file, tool-version,
hash, SHA, or pin catalog determines membership.

### Evidence and projections

An FR-029 `unsupported` decision joins the source candidate at capability negotiation.
It records the unsupported capability and reason, with no artifact, run or Kani outcome.
Pre-execution or generation refusal joins with its producer and stage; a Kani `refused`
outcome joins only if execution reaches the FR-030 boundary. An absent decision or
result remains a missing gap, never an inferred unsupported disposition.

Each execution result joins the exact candidate and declared domain to an artifact/run
identity, source and profile identity, assumptions, method, effective bounds, proof
subject, terminal state and reason. The FR-030 Kani outcome states distinguish proved,
counterexample, refused, invalid/incomplete input, unavailable, timed out, resource
exhausted, cancelled and inconclusive; missing is an accounting gap, not a Kani outcome.
A producer's unfamiliar state is a visible unclassified gap. A positive Kani SUCCESS-check count is necessary under
FR-030 but is not sufficient proof-coverage evidence: the check must bind the claim,
reach the asserted production behavior, and pass the method's nonvacuity conditions.

Requirement, module, function, and obligation views project the same joined claim
population under explicit per-level eligibility rules. Within a method and declared
domain, an eligible requirement criterion, module, or function receives full qualified
credit only if it has at least one eligible mapped obligation and every such obligation
is qualified for that scope. An entity with zero eligible mapped obligations remains a
visible mapping gap, never full credit; a partial result is reported as a gap with the
qualified subset visible. A view reports
its numerator, denominator, gaps and exclusions with reasons. No blended percentage
combines levels or methods. The baseline is an observation to review before choosing
any numeric target or ratchet floor.

### Proof strength

Production Kani proof over the declared finite domain is distinct from a bounded
shadow proof. A narrower effective bound proves only the narrower domain. A bounded
shadow plus separately executed sampled native refinement is reported as conditional,
sampled evidence for the recorded samples and assumptions, not an unqualified proof of
production behavior across the declared domain. A shadow result without refinement
remains an unqualified-production-proof gap. Codegen ADR-003 and FR-028 own the
generation and refinement mechanisms; this AD defines how their eventual evidence is
counted. Codegen FR-004's LLVM clause-entry/consequent observations are vacuity evidence
for execution, not a Kani proof verdict; they can qualify a relevant run but cannot
enter a proof numerator on their own.

## Decisions

- Derive the denominator before joining results, using source inventories and a
  reviewed eligibility policy. Preserve missing, refused and inconclusive rows.
- Use one source-semantic candidate identity and explicit many-to-many projection links; do not
  count harness tags, test presence, SUCCESS-check totals, or a reached callee alone.
- Join negotiation dispositions separately from execution results. `unsupported`
  requires no Kani outcome or run; preserve each refusal's producer and stage.
- Separate method, proof subject, production/shadow strength, declared/effective
  domain, and sampled refinement in every result and aggregate.
- Publish the first baseline with gaps and reasons. A numeric target follows its
  independent review and is outside this decision.

## Risks

- Owner inventories can omit obligations or production functions. The baseline records
  inventory source and unmapped candidates, and independent review must check census
  completeness before interpreting a ratio.
- A result can join the wrong claim or a narrower domain. Exact claim, source, profile,
  subject and domain matching makes that result a gap rather than credit.
- Codegen's shadow/refinement fields and some IR outcome qualification are planned.
  Until they exist, the corresponding rows remain missing or conditional; a future
  producer contract must be reviewed before its method joins this baseline.
