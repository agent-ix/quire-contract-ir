---
id: SR-040
title: "Risk and complexity review of issue 64 native temporal TL correspondence"
type: SpecReview
analysis: risk-complexity
scope: "FR-025 and its external semantic authorities"
review_set: all
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-025
    type: reviews
---
# SR-040: Risk and complexity review of issue 64

## Summary

FR-025 is technically high-risk because it claims semantic equivalence across
repositories and profiles. Volatility is medium because issue #63 and later TL
profiles are unfinished; exact identities and fail-closed versioning bound that
volatility.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-401 | high | **Closed as a specification risk:** formula equality could masquerade as semantic equivalence across different clocks, closure rules, captures, or evaluators. The bridge now binds all dimensions and TC-038 includes differential and mutation probes plus the one-position discriminator. | FR-025 Behavior; FR-025-AC-1/3/7 | missing-requirement |
| FND-402 | medium | **Closed as a volatility control:** unfinished predicate and future TL profile work could silently broaden support. Unknown or unavailable profiles now return unsupported, and each later profile requires a new reviewed identity. | FR-025 Behavior/Dependencies | missing-requirement |

## Risk Register

| Req | Tech Risk | Volatility | Drivers | Mitigation |
|---|---|---|---|---|
| FR-025 | high | medium | cross-repository semantic equivalence; clock and closure distinctions; external issue #63 and TL profiles | exact versioned identities; closed support table; fail-closed decisions; differential, boundary, and mutation tests; dependency-ordered implementation |

## Top Hazards

1. A clock or closure mismatch yields the same formula bytes but different
   truth; the correspondence tuple and discriminator prevent admission.
2. A non-total or stale predicate becomes a proposition; issue #63 is a hard
   gate and no fallback exists.
3. An incomplete closed observation is treated as false extension; FR-025 now
   requires complete closure and TC-038 tests the transition.

## Failure-Domain Cross-Check

SR-036 closes identity, purity, and incomplete-closure gaps. No graph traversal,
concurrency, network protocol, cryptographic primitive, or hard performance SLA
is introduced by the specification.

## Result

**PASS after mitigation in the specification.** High technical risk remains a
reason for dependency-ordered differential implementation, not permission to
prototype around the review gate.
