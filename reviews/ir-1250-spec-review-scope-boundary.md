---
id: SR-202611010107
title: "IR-1250 PR 796 certificate vocabulary scope boundary review"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-contract-ir; IR-1250; PR #796; spec/model/functional/FR-070-certificate-member-vocabulary.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Checked the ownership boundary among the model crate, QSL and QSpec. The requirement correctly assigns the enum and spelling to model and schema authority to QSpec, but AC-4 makes QSL-owned serialization an IR acceptance obligation before the QSL re-export work.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-4 requires QSL's serialized locus to use every model `as_str()` value, while FR-070 assigns locus serialization and the future re-export to QSL-1437. IR can verify its own spelling API, but cannot discharge the QSL integration assertion in this PR or its model implementation. Limit this AC to the model's canonical spelling and absence of locus structure; leave QSL's one-member serialization/re-export assertion to QSL-1437. | spec/model/functional/FR-070-certificate-member-vocabulary.md:85; FR-070-AC-4 |

## Coverage

- FR-070 Description, Outputs and Dependencies, role: examined. Checked type ownership, serialization ownership and the QSL-1437 handoff.
- FR-070-AC-1 through FR-070-AC-4, role: examined. Checked whether each criterion can be verified in the model crate.
- FR-019 planned inventory row, role: examined. Checked that planned export does not claim QSL migration.
- QSL `qsl-replay/src/certificate.rs`, role: context_only. Its current enum is local, and its `CertificateLocus` serializer writes the `member` field from that local type.

## Evidence

At the reviewed QSL main ref, `CertificateMember` is defined in `qsl-replay` and `CertificateLocus::serialize` calls its `as_str()`. The new model type and QSL re-export are not yet implemented. FR-070 itself assigns QSL re-export and local-enum removal to QSL-1437, which is the place that can verify actual use of model spellings in the serialized locus.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Head e6258b50 limits AC-4 to `as_str()` returning the canonical spelling and the model exposing no locus structure or serializer; QSL behavior remains assigned to QSL-1437. |

## Disposition verdict

**PASS** — the original finding is fixed at head e6258b50, and the changed text introduces no new defect in this method.
