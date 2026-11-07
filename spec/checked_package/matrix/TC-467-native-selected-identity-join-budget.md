---
id: TC-467
title: "Shared native selected-identity join budget"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-358
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-352
    type: verifies
---
# TC-467: Shared native selected-identity join budget

## Description

Exercise the mandatory caller-owned work and logical-storage budget through
admitted selected-identity stage rows and all five source-row join families.
Exercise the admitted equality read with the same budget and its existing
producer-selected application and fact rows.

## Test Procedure

Run the FR-358 tagged tests in the `it` integration binary with
`conformance-cli` enabled. For each family, obtain its successful work and
storage charges from an ample budget, then repeat the same admitted value at
each exact limit and one unit below each limit. Join two collection rows under
one budget, including a nonempty prior map copied by `Arc::make_mut`, and
repeat the ordered joins against a cloned admitted package. Run the existing
FR-352 semantic-refusal tests at ample limits and inspect the public API and
call sites for the mandatory signature, cycle boundary and absence of a
parallel budget path.
Read an admitted Boolean equality at ample, exact work, exact storage,
one-below work and storage, and zero storage. Repeat two individually
affordable equality reads against a shared allowance one work unit below
their combined requirement.
Use a QSV materialization refusal with a variable-size exact charge to
compare the full owned-cause storage boundary with a nonzero allowance one
unit below it.

Read the original work and storage limits before and after successful stage
acquisition and type projection, then exhaust the second projection against
their shared allowance. Check the original bound on work and storage refusals,
zero-limit compact refusals, and a work refusal whose owned diagnostic cannot
fit the remaining storage. Check that a semantic refusal has no resource bound.

## Expected Results

Every exact-limit call yields the same selected identity value. Every
one-below call returns the exhausted resource kind, first native locus and
first unpaid charge, with no selected value. Cumulative counters equal the
sum of the two admitted operation deltas; the second operation refuses when
their combined allowance is one below exact. The first selected value remains
unchanged after refusal, and a cloned package gives the same result and
charges. Typed non-budget identity refusals retain their existing causes and
first loci. The crate exports one mandatory `NativeJoinBudget` acquisition
path and its charge metadata without a QSL or runtime dependency.
The equality read returns the same opaque decision at both exact limits,
records a success delta, refuses at the first unpaid shared charge and
returns no owned diagnostic at zero storage.
At the exact diagnostic allowance the original QSV cause is retained; one
unit below it the compact typed code and first QSV locus remain while no
owned cause is returned.

The limit getters remain equal to the caller's original configured values,
independently of consumption or denial. Full and compact native resource
refusals retain the bound for the original denied kind; later diagnostic
storage exhaustion cannot substitute its storage bound for a work bound.
Zero is a present bound, while a non-resource refusal has no configured bound.

## Complete-witness acquisition and rebinding

Run the `complete_identity_witness` integration tests through the public
default-feature model API. Use a strict reader-admitted mixed fixture with
selected declarations, an unread effective record field, private model-tuple
and source-unit inputs, and all five source families. Validate unchanged
contents and cloned admission; omit each channel; compare an independently
admitted package with colliding contents; and exercise the mandatory native
budget at measured exact limits, one unit below, and after earlier acquisition.
Every failure must leave the input unchanged and expose no rebound value.
The IR-owned rows establish native correspondence, not QSL producer authority.

The external model test `complete_witness_public` runs without optional model
features. Its strict obligation-free fixture checks all mandatory entry points
and the 24-unit fresh-stage logical storage floor independently of measured
exact-limit tests. The mixed fixture additionally drops original producer
preparation rows and projects the retained tuple and source-unit identities
through selected record fields. Private stale-buffer controls begin with the
strict reader-admitted relationship fixture, not a manually assembled package.

Fixed-work controls complement measured exact-limit tests: grow an admitted
unrelated non-unit graph under the same allowance and assert the first unpaid
selected-model or declaration candidate, before any kind filter. Independently
sum the binding, selected-document and compared-byte charges for long source
owners and model-unit originals; deny the owner comparison before its bytes
are inspected. A missing model-unit declaration precedes an unread effective
field, and a primitive-node tuple row gives the same native typed refusal in
public acquisition and selected-type projection.
