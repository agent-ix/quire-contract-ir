---
id: FR-029
title: "Select a versioned bounded Kani capability profile"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-015
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/ADR-0053
    type: depends_on
---
# FR-029: Select a versioned bounded Kani capability profile

## Description

The Contract IR Kani boundary shall select an immutable, named bounded-Kani profile before it lowers a checked native clause or creates a harness.

## Inputs

An exact checked native clause, its selected Quire source/profile identities, the selected Kani tool version, a profile identifier and revision, and declared finite-domain and resource parameters.

## Outputs

One typed profile-selection result and a complete capability matrix for every construct encountered by the selected clause and model slice.

## Behavior

`kani-bounded/1` is the first profile family. Its profile revision identifies the Kani version range, lowering ABI, harness ABI, oracle/strategy ABI, replay wire schema, diagnostic vocabulary, resource-accounting rules, and exact support matrix. A different profile revision, ABI revision, or capability entry is a different selection and may not reinterpret a prior artifact.

For each encountered construct the matrix reports exactly one of `supported`, `refused`, or `unsupported`. `supported` records the responsible lowering module and proof obligations; `refused` records the source construct and located reason; `unsupported` records a construct for which the profile has no qualified interpretation. An `unsupported` entry is a negotiation-stage disposition in the QSpec FR-331 `dispositions` vocabulary: the item settles `unsupported` with a warning naming the item's capability kind from the closed `quire.capability-kind/v1` vocabulary (QSpec FR-290), and no artifact, `KaniOutcome` or QSL `TerminalValue` exists for it. Resource exhaustion, cancellation and an absent solver or Kani backend are run outcomes, recorded only by FR-030's `resource_exhausted`, `cancelled` and `unavailable` kinds, and never as a matrix entry. The matrix is complete before harness generation. A missing entry, unknown construct, version mismatch, or duplicate/conflicting entry refuses before any partial lowered artifact is exposed.

The initial profile has separately addressable entries for checked arithmetic and definedness, finite scalar state, finite identity-bearing objects and references, finite parent-graph reachability, ordered duplicate-preserving collections and quantified queries, and every other recognized native construct. Support for one entry never implies support for another. In particular, scalar acceptance neither represents nor approximates objects, references, graphs, collections, query semantics, temporal semantics, or an unbounded population.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-029-AC-1 | A selected profile binds its versioned Kani/tool, lowering, harness, oracle/strategy, replay, bounds, and capability-matrix identities before lowering. | Test (TC-042) |
| FR-029-AC-2 | Every encountered construct has exactly one supported, refused, or unsupported matrix entry; an unsupported entry settles its item `unsupported` with a warning naming the item's capability kind from the closed `quire.capability-kind/v1` vocabulary (QSpec FR-290) and emits no artifact, `KaniOutcome` or `TerminalValue`; no matrix entry records resource exhaustion, cancellation or an absent executable; unknown, conflicting, and incomplete matrices refuse with the originating source identity. | Test (TC-042) |
| FR-029-AC-3 | A scalar capability refuses an object, reference, graph, collection, query, temporal, or unbounded-population request with its source identity. | Test (TC-042) |

## Dependencies

FR-015 owns native definedness. ADR-0053 owns native source authority and the finite-admission firewall.

## Status

AC-1 and AC-3 are implemented for the exact `kani-bounded/1` profile through
Contract IR PRs #87 and #88 (`2f9b00b`, `e1ad842`). AC-2 is planned: the matrix
still has a `CapabilityDisposition::Inconclusive` entry, which lowering turns
into an `Inconclusive` run outcome, and no `unsupported` entry. Target-family support remains limited to the
closed capability matrix; no scalar or recognized construct implies support for
another family, unbounded population, Kani version or options selection.
