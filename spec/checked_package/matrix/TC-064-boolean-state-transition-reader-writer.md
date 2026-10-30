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
inputs, clauses, dependencies, identity and `claim` occurrence. Add a second
checked source declaration of equal transition content with a distinct source
region and occurrence ordinal. Assert that the two `claim` occurrences point
to one transition node, retain both distinct regions and ordinals, and leave
the structural ids and package name unchanged. Write this two-occurrence package,
read it again and compare both occurrences, the complete body and structural ids. Independently
admit a point-qualified pre/post clause pair whose transition uses the
no-point anchor.

Apply the current QSpec `XT-08` and `XT-09` vectors as structural body and source-correspondence controls. A valid operation edit changes the complete relevant bodies; moving only a source region changes correspondence alone.
Separately change only the subject field, one input, `pre`, `post` and selected profile. Compare each complete transition body, structural id and package name before and after. A valid body edit under the same structural id is admitted when independent joins pass; an invalid target or clause refuses at its own state or graph check. A source-region edit changes correspondence without changing the body.

Apply QSpec refusal vectors `XT-02` through `XT-07`, `XT-10` through `XT-18`
one at a time. Retain valid structural ids and rebuild exact dependencies unless a case intentionally tests a duplicate id or missing dependency. Assert
the first refusal's code, cause and RFC 6901 locus, and add independent
two-defect cases at the wire/graph/frame/state/operation boundaries and
within transition checks.
When QSL-1163 publishes an authored Boolean transition source and checked
claim, read that emitted package through the same public IR API; no upstream
fixture or schema is copied into IR.

## Expected Results

The valid transition and its point-qualified variant retain the exact
Boolean subject, pre-state inputs, post-state result, ordered bindings,
selected profile, dependency set, source occurrences, node keys and package
identity through read/write/read. Equal-content checked claims share one
transition node and retain two distinct occurrence ordinals and regions
through read/write/read. The identity comparisons and independent rekey
mutations change exactly the identities FR-067-AC-5 names, while a source-only
edit changes none. Each adverse vector refuses with QSpec's first code,
cause and locus. A QSL-authored claim produces the same admitted public
carrier when its producer is available.
