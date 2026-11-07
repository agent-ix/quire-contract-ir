---
id: INS-001
title: "Checked package V2 admission capability inspection"
type: Inspections
---

# Checked package V2 admission capability inspection

## Inspections

| ID | Obligation | Who | Commit | Verdict | Note |
| --- | --- | --- | --- | --- | --- |
| INSP-001 | FR-038-AC-493 | Codex independent reviewer (gpt-6.1-sol) | 58386de291c8291a1b313e591aa06b431b728052 | Pass | Inspected `AdmissionBinding` and `CheckedPackageV2` in `crates/quire-contract-model/src/checked_package/v2/mod.rs:891-1009,1443-1465` and the named crate-root export in `crates/quire-contract-model/src/lib.rs:150`. `AdmissionBinding` is a public tuple struct with a private `Arc<AdmissionBuffers>` field; its only public trait surface is redacted `Debug`, `Clone`, `Eq` and `PartialEq`. It has no public constructor or buffer accessor. The private retention path clones the reader's original immutable `Arc` buffers after admission, without copying a carrier. No compatibility representation is exposed. |
