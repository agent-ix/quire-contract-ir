---
id: SR-1394
title: "scope-boundary review of PR 289 (AD-007 decisions and routing)"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-contract-ir@cb94d57e427dd1154a442b8ba7ba0461e99443e1; spec/assurance/AD-007-cross-repo-dependency-graph.md (System Boundary, Decisions A-D, invariants G-1..G-5, O-1..O-3, routed gaps R-1..R-4 and the RT, CG and IR-owned routing lines)"
review_set: subset
---
# SR-1394: scope-boundary review of PR 289

## Summary

Ticket: IR-346. The AD states that it records measurements and IR's position and
decides nothing that belongs to another repository. Routing is mostly right: R-1,
R-2 and R-4 go to QSL, R-3 to QVC and quire-canonical, the `src/exact` deletion to
RT (IR-349), the root-crate edge to CG (O-2, AD-006), stack growth to quire-walk or
each repository (O-3), and `KaniProvider*` (IR-347), `bans` (IR-343) and the AD-005
text to IR. O-1 is an explicit owner decision (kreneskyp) with a recommendation,
QSL carries the change (R-4) only if accepted, and order is left to QSL and RT.
None of R-1..R-4, O-2 or O-3 is decided on another owner's behalf in its own text.
One decision does bind other repositories.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Decision C ("Edges between first-party repositories use `git` with `branch = \"main\"` and no `rev`") is stated as this AD's decision for every repository, which contradicts the AD's own boundary ("does not decide anything that belongs to another repository") and its routing: R-1 offers QSL "justify each or move it to the rule", which Decision C forecloses. Either cite the ecosystem rule's existing source and state C as IR's position (IR's own edges conform), with the QSL `rev` edges as R-1's open question, or drop "justify" from R-1. G-2, G-3 and G-5 have the same reach and should be phrased as targets proposed to each owner, not invariants this AD sets. | spec/assurance/AD-007-cross-repo-dependency-graph.md:164-165, spec/assurance/AD-007-cross-repo-dependency-graph.md:173-179, spec/assurance/AD-007-cross-repo-dependency-graph.md:244 |

## Verdict

Routing and the O-1 framing as an owner decision are sound. FND-001 must be fixed so
the AD's decisions stay inside IR's boundary and agree with R-1.

## Dispositions

Re-reviewed at agent-ix/quire-contract-ir@568adea9219264dec13494a3b3fdb10174784845. Decision C's cited source is real: QSL `Cargo.toml` lines 81-83 say "one-copy-deps (agent-ix org-wide, IR #225/RT #88): branch = \"main\", no rev". CG's and RT's first-party edges are all `branch = "main"` with no `rev`. No new findings in this lens.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 568adea9219264dec13494a3b3fdb10174784845: decision C is scoped to IR's position on IR, CG and RT edges, with QSL's edges a routed recommendation (R-1, "QSL's call"); G-1..G-5 are framed as targets proposed to each owner |
