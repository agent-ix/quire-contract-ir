---
id: SR-2026110602354632
title: "IR-1370 PR 937 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #937; FR-038-AC-492, FR-038-AC-493, FR-019-AC-5"
review_set: subset
---

## Summary

Reviewed PR #937 head a29e7ffcf and the listed criteria. No applicable AssuranceProfile was found under `spec/`. The public AdmissionBinding type is omitted from the model crate root export and FR-019 inventory, so external consumers cannot name or import the promised capability.

## Verdict

**FAIL** — The public AdmissionBinding type is omitted from the model crate root export and FR-019 inventory, so external consumers cannot name or import the promised capability.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The public AdmissionBinding type is omitted from the model crate root export and FR-019 inventory, so external consumers cannot name or import the promised capability. | crates/quire-contract-model/src/lib.rs:149 |

## Coverage

- FR-038-AC-492, role: examined. Excerpt: An admitted `CheckedPackageV2` shall issue a cloneable, opaque `AdmissionBinding` capability and report whether it retains that capability. Clones from one read are members; an independent read of equal canonical bytes is not.
- FR-038-AC-493, role: examined. Excerpt: The public capability shall use the reader's original admission binding and expose no construction path, buffer contents, copied carrier, or compatibility representation.
- FR-019-AC-5, role: examined. Excerpt: The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists as implemented at the tested revision. Entries explicitly marked planned are excluded until their owning code lands; exporting a planned item fails the inventory until its row is marked implemented. The public-item inventory fails on an added or missing implemented item; no implemented item in the table is reachable through a `quire_contract_ir` path. The retired `PACKAGE_DOMAIN_V2` export is absent.

## Evidence

`AdmissionBinding` is public inside private `checked_package::v2`, but `checked_package` and `v2` are private modules. The model crate root has explicit named exports and omits `AdmissionBinding`; FR-019 declares that root inventory the stable API. The package method can be used by inference but external consumers cannot import or name the capability type. The binding itself uses Arc pointer identity and checks original retained buffers. The focused integration test passed in the coding lane; no test or gate was rerun here.
