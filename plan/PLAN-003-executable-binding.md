---
id: PLAN-003
title: "Executable projection binding prerequisite"
type: Plan
status: in-progress
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-023
    type: references
---

# PLAN-003: Executable projection binding

Issue #50 is a derived semantic binding boundary, not permission to use a
hand-authored expressions sidecar as the campaign's normal source. The #52
frontend/model campaign remains the authoritative source join.

1. Review FR-023 and the public API with the codegen/vacuity consumer before
   implementation; preserve existing conformance wire output byte-for-byte.
2. Factor the current private expression checker so the binder receives typed
   values directly and the conformance wrapper continues to emit its own results.
3. Implement bounded projection decoding, complete clause binding and canonical
   identity with no partial-success path or local evidence mechanism.
4. Add schema, shared-form synthetic fixtures and public consumer/mutation tests;
   matrix status remains planned until they execute.
5. Independently review exact committed head, run stable/MSRV/full native gates,
   and qualify the codegen consumer before integration. PR #51 and shared-tool
   prerequisites remain separately tracked; no human release approval is inferred.
