---
id: SR-1553
title: "Integrity review of quire-contract-ir PR #295 (IR-627 anonymous structural node bodies)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@984c283099ce117b5ab7cba2b8f03fe3d6e5e5bc; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (section 'Anonymous structural node bodies and keys', FR-038-AC-123..130)"
review_set: subset
---

## Summary

Ticket: IR-627. Checked the new section's measured claims against the code and against the sibling
QSpec and QSL checkouts. Confirmed: `check_model_member` (operations.rs:1773-1875) compares only the
`result_type` digest with `MemberType::node_key` and never reads the named node's body;
`MemberType::node_key` is `pub(super)` and derives Boolean, Integer, `Int[lo, hi]`, Reference, Option,
the four collection forms and `collection_bounds` from QSL FR-092/FR-094 (model_members.rs:432-542);
identity.rs maps every `bounded_domain`, anonymous scalar, composite and value form to no nominal
preimage. Two measured claims are wrong, and the first-refusal position of the new check is not stated.

## Verdict

Changes requested: two medium findings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Two measured claims are false. (a) "the key check re-derives exactly two families" leaves out model declaration node keys: owner recovery computes the `quire.structural-node/v1` ModelDeclarationNode key from each declaration and refuses `stale-node-key` on a mismatched fixed member (model_members.rs:257-280, :918). (b) "QSpec FR-322 publishes the nominal and application preimages and not this one" is wrong: FR-322 step 2 publishes the `quire.structural-node/v1` ModelDeclarationNode preimage (node-identity-preimage.schema.json, model-member-type-vectors.json), and step 4 says each anonymous node is keyed "exactly as QSL FR-092 and FR-094 key it". So QSpec delegates the anonymous preimage to QSL, which publishes it with golden vectors. Q1, Q2 and AC-130 therefore ask the wrong owner and the wrong vector source | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1752, :1770-1772, :1811-1823, :2454 |
| FND-002 | medium | The new check's place in the first-refusal order is undefined. It runs in step 4 at the reading application's `operator-ineligible` check, but it reports `stale-node-key` at the read node. AC-127's "ascending node-id digest order" could mean the tampered node's digest or the order in which reading applications are visited. Its position against the stale-key stage, a stale application key and an `ill_typed` defect in another application is not stated, so two readers could report different first refusals | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1776-1795, :2451 |

## Dispositions

Round 1, reviewed at ab860cac3a7162c5eea073d99bb5bcf5433e1237.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ab860cac3a7162c5eea073d99bb5bcf5433e1237 |
| FND-002 | fixed | ab860cac3a7162c5eea073d99bb5bcf5433e1237 |
