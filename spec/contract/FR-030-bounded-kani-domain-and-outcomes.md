---
id: FR-030
title: "Bind bounded Kani domains and non-Boolean outcomes"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-029
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-015
    type: depends_on
---
# FR-030: Bind bounded Kani domains and non-Boolean outcomes

## Description

The bounded-Kani boundary shall use a typed finite model-domain/snapshot/population ABI and a common typed outcome envelope. The bounded-Kani boundary shall not hide invalid or incomplete populations through Kani assumptions.

## Inputs

A selected `kani-bounded/1` profile, finite typed model declarations, explicit population and snapshot records, model/domain/collection/graph/resource bounds, and a checked native clause.

## Outputs

Either a validated bounded execution input and a typed outcome, or a typed pre-execution refusal/inconclusive result with no partial proof artifact.

## Behavior

The ABI carries the exact model identity, closed typed universes, object identities, ordered duplicate-preserving collection values, reference targets, snapshot and invocation identities, completeness declarations, and every selected bound. Bounds derive only from the selected profile and exact reviewed model/domain declarations; host widths, observed values, and backend defaults are not bounds. The input checker validates complete populations before harness construction: duplicate identities, wrong object types, dangling/foreign references, snapshot/invocation mismatch, impossible creation/deletion deltas, duplicate decoded fields, oversized values, and a claimed-complete but invalid population are `invalid_input`. An incomplete population or unavailable required observation is `incomplete_input`, not an empty universe.

The common envelope has distinct terminal kinds `proved`, `counterexample`, `refused`, `invalid_input`, `incomplete_input`, `unavailable`, `timed_out`, `resource_exhausted`, `cancelled`, and `inconclusive`. Only `proved` carries a Boolean-success claim; only `counterexample` carries a Boolean-failure claim. Every other kind carries a stable code, source/profile identity, bound/resource context, and cause as applicable, and carries no Boolean result. A definedness failure remains the existing typed partial-operation result and cannot become a Kani counterexample or proof by coercion.

An `unavailable` outcome carries exactly one of two cause codes:
`kani_solver_absent`, when no solver satisfying the selected profile's
capability negotiation is present, or `kani_backend_absent`, when no Kani
backend satisfying it is present. No public constructor builds an
`unavailable` outcome with any other cause code; a request to do so returns a
typed refusal and no `unavailable` outcome.

A `proved` outcome carries the number of SUCCESS checks the run's obligation completed with, and that number is at least one. When a Kani run reports a proof whose obligation completed with zero SUCCESS checks, the bounded-Kani boundary shall record `inconclusive` with the cause code `kani_vacuous_proof` and no Boolean claim, never `proved`; that classification is the one shared rule every caller that classifies a Kani run routes through.

The harness constrains only values after the ABI has established that the finite population and relationships are valid. It shall never use `assume` to discard invalid, incomplete, unavailable, or over-bound input; those inputs return their earliest typed result with originating source identity and no partial lowering, oracle, proof, or replay artifact.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-030-AC-1 | Every model, population, snapshot, identity, reference, collection, and resource bound is explicit, exact, and validated before harness construction. | Test (TC-042) |
| FR-030-AC-2 | Invalid and incomplete populations, unavailable observations, and resource limits retain distinct typed outcomes and are never removed by an assumption. | Test (TC-042) |
| FR-030-AC-3 | No invalid, refused, incomplete, unavailable, timed-out, exhausted, cancelled, or inconclusive outcome carries Boolean success; earliest failures emit no partial artifact. | Test (TC-042) |
| FR-030-AC-4 | A `proved` outcome built from a run with three SUCCESS checks carries the count three; a run with zero SUCCESS checks yields `inconclusive` with cause `kani_vacuous_proof`, carries no count and no Boolean claim; and no public constructor builds a `proved` outcome without a count of at least one. | Test (TC-223) |
| FR-030-AC-5 | An `unavailable` outcome built for an absent solver carries cause `kani_solver_absent` and one built for an absent backend carries `kani_backend_absent`, each with no Boolean claim; an attempt to build an `unavailable` outcome with any other cause code yields no `unavailable` outcome. | Test (TC-223) |

## Dependencies

FR-015 defines partial-operation semantics. FR-029 selects the capability and ABI revisions that interpret this boundary.

## Status

Implemented through the shared finite ABI/outcome foundation and three semantic
lanes in Contract IR PRs #88 through #91 (`e1ad842`, `b5bde5d`, `7a74f0b`,
`165ae4c`). Invalid, incomplete, unavailable and over-bound inputs retain typed
non-Boolean outcomes; this status does not claim an undeclared model or domain.
AC-4 and AC-5 are planned: today a `proved` outcome carries no check count
and an `unavailable` outcome accepts any cause code.
