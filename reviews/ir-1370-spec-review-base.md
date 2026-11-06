---
id: SR-2026110602354634
title: "IR-1370 PR 937 spec-review/base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #937; FR-038-AC-492, FR-038-AC-493, FR-019-AC-5"
review_set: subset
---

## Summary

Reviewed PR #937 head a29e7ffcf and the listed criteria. No applicable AssuranceProfile was found under `spec/`. AC-492 promises a public capability, but FR-019 requires every public model item to appear in its inventory and be exported by name at the crate root; neither contains AdmissionBinding.

## Verdict

**FAIL** — AC-492 promises a public capability, but FR-019 requires every public model item to appear in its inventory and be exported by name at the crate root; neither contains AdmissionBinding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-492 promises a public capability, but FR-019 requires every public model item to appear in its inventory and be exported by name at the crate root; neither contains AdmissionBinding. | spec/model/functional/FR-019-rust-library-interface.md:285 |

## Coverage

- FR-038-AC-492, role: examined. Excerpt: An admitted `CheckedPackageV2` shall issue a cloneable, opaque `AdmissionBinding` capability and report whether it retains that capability. Clones from one read are members; an independent read of equal canonical bytes is not.
- FR-038-AC-493, role: examined. Excerpt: The public capability shall use the reader's original admission binding and expose no construction path, buffer contents, copied carrier, or compatibility representation.
- FR-019-AC-5, role: examined. Excerpt: The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists as implemented at the tested revision. Entries explicitly marked planned are excluded until their owning code lands; exporting a planned item fails the inventory until its row is marked implemented. The public-item inventory fails on an added or missing implemented item; no implemented item in the table is reachable through a `quire_contract_ir` path. The retired `PACKAGE_DOMAIN_V2` export is absent.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-019 now inventories AdmissionBinding in the V2 reader row and the model crate root exports it by name, so AC-492 and FR-019-AC-5 agree. |
