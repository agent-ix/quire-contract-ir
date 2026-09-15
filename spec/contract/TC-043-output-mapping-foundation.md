---
id: TC-043
title: "Output-mapping coordinator and package foundation conform"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-032
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-033
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-034
    type: verifies
  - target: ix://agent-ix/quire-specification/TC-150
    type: references
  - target: ix://agent-ix/quire-specification/TC-154
    type: references
  - target: ix://agent-ix/quire-specification/TC-155
    type: references
---
# TC-043: Output-mapping coordinator and package foundation conform

## Description

Verify the target-neutral Rust request, mapper, record, budget, package, and
observer-reference boundary against the accepted FS06 portable controls without
implementing or claiming an OCL, SysML/KerML, or FRETish correspondence.

## Test Procedure

Strict-read a bound package containing multiple executable clauses and one
informational clause. Admit every exact FS06 target profile independently over
ordered source obligations and a complete limit set. Drive a deterministic test
mapper through every disposition and source-fact state, then assemble and replay
the package. Mutate every request, source, profile, dependency, candidate,
disposition, condition, cause, adequacy, region, generator, digest, order, and
limit axis independently. Exercise empty, duplicate, foreign, stale, zero,
exact, just-over, checked-overflow, cancellation, allocation-failure, and mapper-
failure cases. Repeat equal inputs while varying path, time, locale, display, and
downstream observer facts.

## Expected Results

Every valid request yields one ordered record per requested obligation and one
byte-identical immutable package. Every mutation either changes the owning
identity or returns the exact typed refusal before exposing a partial package.
Unsupported, pending, incomplete, refused, exhausted, and cancelled cases never
become preservation or Boolean success. Observer facts remain separately typed,
downstream, and unable to alter bytes, records, dispositions, or identity.
