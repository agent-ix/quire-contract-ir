---
id: TC-047
title: "CheckedPackage version dispatch keeps V1 frozen and V2 separate"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: references
  - target: ix://agent-ix/quire-specification/TC-217
    type: references
---
# TC-047: CheckedPackage version dispatch keeps V1 frozen and V2 separate

## Description

Verify FR-038-AC-1 (QSpec FR-322-AC-11, FR-201-AC-5): one strict parse selects
exactly one closed contract version, and the frozen V1 reader, types and
lowering are byte- and behavior-identical to Contract IR #104.

## Test Procedure

Canonicalize the vendored V1 all-families fixture and both V2 positive fixtures
and read each through the dispatcher with an authoritative digest context.
Replace `contract_version` with an unknown value and remove it. Read V2 bytes
through the V1 reader, add `nominal_identity_preimage` to a V1 node, and read V1
bytes through the V2 reader. Compare a V1 and a V2 package key with equal digest
bytes. Replay a constructed V1 package through `CheckedPackage::read` and
`lower`, and compare the complete typed result against a recorded golden.

## Expected Results

V1 and V2 fixtures admit only on their own path. Unknown versions refuse as
`unknown_contract_version` and an absent version as `malformed_wire`. Each
reader refuses the other version, the V2-only member refuses as
`unknown_member`, the two package keys are unequal, and the V1 golden matches.
