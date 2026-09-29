---
id: SR-593
title: "PR #205 spec text integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@97bcea493dbc71b1785ba67d25ff632a2f8fd141; spec/contract/FR-040-admit-frame-entries-and-state-clauses.md, spec/contract/FR-038-consume-checked-package-v2.md, spec/contract/TC-053-checked-package-v2-frame-bodies.md, spec/contract/TC-056-checked-package-v2-frame-entries-and-state-clauses.md, spec/contract/TC-057-qspec-node-identity-vectors.md, spec/contract-test-matrix.md (git diff origin/main...HEAD)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-040
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---
# SR-593: PR #205 spec text integrity review

## Summary

Ticket: IR-89.

The spec edits were checked against QSpec origin/main `e56756f`:

- FR-340, FR-341, FR-342 and FR-322 "Model-owned members".
- `proposals/checked-package-v2/schema.json`: `ModelNode`, `ModelOwner`,
  `FrameModifiesEntry`, `OperationAnchorBody`, `ParameterBody` and
  `BodyBindingRules`.
- The QSpec `README.md` reader order.

They were also checked against the code at `97bcea4`.

## Verdict

**CHANGES REQUESTED** (two medium findings).

These edits are consistent with QSpec and the code:

- FR-040's 15 forms, entry shapes, roles, loci and reader order.
- The FR-038 `ModelOwner.version` edit, which matches QSpec's schema.
- The FR-038-AC-22 `/body` locus.
- The TC-053 and TC-057 procedure edits.
- The TC-057 keying of non-stale mutations, which QSpec's README order
  supports: stale-key runs before operation checks.
- The matrix FR-040 and FR-038 rows (apart from FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-040 frames `record_value_type` field-name resolution as a gap in QSpec ("pending a QSpec ruling", "QSpec's `ModelDeclarationNode` has no such form"). QSpec FR-340 "Field entries" does state a rule: for a `model`/`record_value_type` node, `name` matches the fields its record declaration declares (see also FR-340-AC-10). FR-040 should name this as a deviation and cite the tracking ticket, and it names none. Its "admits with its name unresolved" also holds only for a source-declared node (SR-591 FND-001). | spec/contract/FR-040-admit-frame-entries-and-state-clauses.md:105-107; spec/contract/FR-040-admit-frame-entries-and-state-clauses.md:222; spec/contract-test-matrix.md:48 |
| FND-002 | medium | FR-038 now says "A closed-schema decode refusal inside a nominal preimage or its owner refuses as `invalid_semantic_graph` ... and not as `malformed_wire`". The code remaps only `malformed_wire`: an unknown member inside a preimage keeps `unknown_member`, per `mod.rs`'s own comment. An unknown member is also a closed-schema decode refusal, so two readers of this sentence would implement different codes. | spec/contract/FR-038-consume-checked-package-v2.md:105-107 |
| FND-003 | low | The TC-048 procedure still says a parameter mutation refuses "at the member it breaks". FR-038-AC-22 and `checked_package_v2_qsl_parameters.rs` now put every binding defect at the node's `body`. The TC-048 document itself was not updated. | spec/contract/TC-048-checked-package-v2-strict-reader.md:75-78 |
| FND-004 | low | The "count replayed from each array equals the count published" clause in FR-040-AC-13, FR-038-AC-40 and the TC-057 cases row cannot fail, because the only published count is the array's own length. Either state an independent expected count or drop the clause. | spec/contract/FR-040-admit-frame-entries-and-state-clauses.md:232; spec/contract/FR-038-consume-checked-package-v2.md:652 |

## Dispositions

Round 1, reviewed at `1d89455fc04ddfb60cd2ac932886f1b223cd3688` (rebased on origin/main `a38f3db`).

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 1d89455 |
| FND-002 | fixed | 1d89455 |
| FND-003 | fixed | 1d89455 |
| FND-004 | fixed | 1d89455 |

## New findings (disposition pass 2)

Round 2, reviewed at `995bd4bec7fe526c7891f728044fec4c2e7d469e`. FND-001 to FND-004 stay fixed; FND-004's clauses were removed along with AC-13 and AC-40.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | Two live references still point at artifacts this PR removed. The matrix TC-053 row says "QSpec's `frame_mutations` vectors are TC-056's", but TC-056 no longer replays them. ADR-0056 lists foreign ids in `.../v2/operations/model_member_vectors.rs` and in the `Makefile:109-112` comment, and both are deleted. | spec/contract-test-matrix.md:96; spec/decisions/ADR-0056-spec-layout-convention.md:375-376 |
