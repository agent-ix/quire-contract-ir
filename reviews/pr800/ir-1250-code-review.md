---
id: SR-202611010200
title: "IR-1250 PR 800 CertificateMember code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #800; crates/quire-contract-model/src/certificate_member.rs; crates/quire-contract-model/src/lib.rs; tests/conformance_qspec/certificate_member.rs; tests/conformance_qspec/main.rs; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-070", type: references }
---

## Summary

Reviewed PR #800 at `df25d4e2dd6e37a40e063b228a41076a4f9b038a` against FR-070 AC-1 through AC-4 and FR-019 AC-5. Rust review covered the public enum, derives, explicit export, single-source vocabulary, schema conformance, traceability, and cross-repository ownership. Ticket: IR-1250.

## Verdict

**FAIL** — the new authoritative enum temporarily duplicates QSL's still-live local enum without the user-approved expiring exception required by the global AGENTS instructions. A separate maintenance gap lets a newly added enum variant be absent from `ALL` while the conformance test passes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The new model enum duplicates QSL's live 51-member CertificateMember until QSL-1437 removes it; this temporary cross-repository copy requires an explicit user-approved exception with the removal condition recorded. | crates/quire-contract-model/src/certificate_member.rs:8 |
| FND-002 | medium | The enum, ALL, and as_str maintain the 51-member vocabulary in three independent lists; adding a variant only to the enum can pass the QSpec conformance test, violating ALL's completeness guarantee. | crates/quire-contract-model/src/certificate_member.rs:8 |

## Coverage

- FR-070-AC-1, examined: default-feature root export, 51 variants, ALL, as_str, derives, and cycle-free dependencies.
- FR-070-AC-2, examined: QSpec schema-by-ref, certificate-owned property closure, enum set, duplicate and malformed-source behavior.
- FR-070-AC-3, examined: schema enum order versus ALL order.
- FR-070-AC-4, examined: exact spelling and absence of locus structure or serializer.
- FR-019-AC-5, examined: public inventory row and explicit root re-export.
- Global AGENTS cross-repository copy rule, examined; QSL main `a1456801d173d2395f0d6b618126745bb11ccb0f` still has its local enum.

## Evidence

The model crate has no new dependencies and explicitly re-exports the public type. The focused QSpec test, model Clippy, conformance Clippy, and formatting passed in the coder's recorded run. The PR changes no CI workflow or gate. The actual full gate remains with the code lead on the final reviewed head. QSL's main source still defines `CertificateMember` through its `certificate_members!` macro; QSL-1437 is Backlog, and the user's earlier AbsenceMode exception did not cover this type. Rust-style's implementation rule keeps a vocabulary in one typed table or generated representation; QSL currently uses such a table, while this PR spells the same catalog separately in the enum, ALL, and match. The conformance test enumerates ALL, so a variant absent from ALL is invisible to that test.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | still-open | QSL main still defines its local CertificateMember and no direct user-approved, expiring exception has been established for this temporary duplicate. QSL-1437 remains the removal ticket. |
| FND-002 | fixed | Commit `ad10bb4e` derives enum variants, ALL, documentation, and exhaustive as_str arms from one `certificate_members!` table. An added table row now enters ALL and as_str together; the updated public-inventory parser recognizes that macro. |

## Round 1 evidence

The fix uses one declarative row per member; the macro repeats each variant into the enum, ALL slice, and exhaustive match. The QSpec conformance test still compares the resulting 51 spellings and order against the external schema. The changed `public_interface.rs` parser includes `certificate_members`, so the default-feature export inventory can inspect the generated public items. No new defect was found in this fix slice. The QSL duplicate remains a separate policy decision.
