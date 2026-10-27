---
id: TC-064
title: "Read and write a Boolean state transition"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-067
    type: verifies
---

# TC-064: Read and write a Boolean state transition

## Description

Verify [FR-067](../functional/FR-067-admit-boolean-state-transition.md)
through the public checked-package V2 reader and typed writer, with QSpec
TC-233 `XT-01` through `XT-18` as the authoritative external oracle.

## Test Procedure

Resolve the published QSpec `positive-state-transition.json` and TC-233
transition mutations from the selected QSpec checkout at test time. Supply
the actual selected `acme/orders` model and lock profile. Read the positive
package, inspect the transition's typed five-binding application, subject,
inputs, clauses, dependencies, identity and `claim` occurrence, write it,
read it again and compare. Independently admit a point-qualified pre/post
clause pair whose transition uses the no-point anchor.

Apply QSpec `XT-02` through `XT-18` mutations one at a time. Rebuild exact
dependencies, node keys and package identity unless the case specifically
tests a stale key or missing dependency. Assert the first refusal's code,
cause and RFC 6901 locus, and add independent two-defect cases at the
wire/graph/frame/state/operation boundaries and within transition checks.
When QSL-1163 publishes an authored Boolean transition source and checked
claim, read that emitted package through the same public IR API; no upstream
fixture or schema is copied into IR.

## Expected Results

The valid transition and its point-qualified variant retain the exact
Boolean subject, pre-state inputs, post-state result, ordered bindings,
selected profile, dependency set, source occurrences, node keys and package
identity through read/write/read. Each adverse vector refuses with QSpec's
first code, cause and locus. A QSL-authored claim produces the same admitted
public carrier when its producer is available.
