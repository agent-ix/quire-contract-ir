---
id: SR-2026102982
title: "spec-review-base review of IR-1141 PR #721"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; FR-038, TC-048, TC-226"
review_set: subset
---

## Summary

The shared-label and exact-ID requirements align with QSpec #535’s proposed wire contract. The adverse cycle-label test setup is ambiguous. Ticket: IR-1141. Reviewed PR head ee52f02ea9a92bdc4c7654fdb8846fd1d64455c2 against IR main 45e1fccf and open QSpec #535 head 6dbc0763. Final alignment requires upstream merge and remeasurement.

## Examined scope

- `FR-038` node-id, recursion and reader prose (examined): `spec/checked_package/functional/FR-038-consume-checked-package-v2.md` lines 352-370, 596-607, 1190-1203 and 2493-2542.
- `FR-038-AC-88`, `AC-145`, `AC-147`, `AC-150`, `AC-227`, `AC-241`, `AC-364` (examined): same FR file.
- `TC-048` group, protocol and union controls (examined): `spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md`.
- `TC-226` group and fixture controls (examined): `spec/checked_package/matrix/TC-226-checked-package-v2-anonymous-structural-node-bodies.md`.
- `QSpec FR-322` cycle rule, grouped identity, AC-274 and PR #535 proposed label-only rule (context): `agent-ix/quire-specification/spec/objects/interfaces/FR-322-checked-package-artifact.md`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-226 step 3 says to give “one node” an invalid or mismatched cycle label. If that node is an off-cycle member, a different non-empty label is legal under QSpec’s rule that only members of one actual cycle must share a label, so the claimed refusal need not occur. Select one member of a real cycle and change only its label while the other cycle members retain theirs; treat malformed/empty labels as a separate shape mutation if intended. | spec/checked_package/matrix/TC-226-checked-package-v2-anonymous-structural-node-bodies.md:19-31 |

## Verdict

FAIL until TC-226 selects a member of an actual cycle for the mismatched-label mutation.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Head 19e97605 changes exactly one member of an actual cyclic reference component to a different non-empty label while every other member keeps the original, so the cycle rule must refuse; it separately mutates that member to an empty invalid label for the shape refusal. |

## Disposition verdict

PASS for the IR spec diff. Final alignment remains held on QSpec #535 merge and remeasurement.

## Post-review delta verification

At frozen head `072aac8d366d3623da8765996af6d0fa0c486f76`, the only delta from `69d7ce02` qualifies FR-038-AC-147's lone acyclic-label admission to nodes whose closed shape permits `recursion_group` and explicitly preserves QSpec FR-322-AC-29's model-declaration refusal. This matches merged QSpec #535 (`a57dc869`): FR-322 admits a lone acyclic label, TC-946 K10 admits an acyclic protocol node with one, and AC-29 refuses a model declaration carrying it. FR-038-AC-146 already states the same closed-shape condition. The prior FND-001 fixed disposition remains current; no new finding arises from this delta.
