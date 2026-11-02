---
id: SR-202611010117
title: "IR-1250 PR 800 CertificateMember gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #800; spec/model/functional/FR-070-certificate-member-vocabulary.md; spec/model/functional/FR-019-rust-library-interface.md; crates/quire-contract-model/src/certificate_member.rs; tests/conformance_qspec/certificate_member.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-070", type: references }
---

## Summary

Compared IR-1250's requirements, tagged QSpec conformance test, and implementation without selecting a plan. The positive QSpec checkout test passes, but its malformed-source check misses a dangling reference inside a relevant certificate-owned definition. Ticket: IR-1250. Plan completion: not assessed.

## Verdict

**CONDITIONAL** — FR-070-AC-2's malformed-source refusal is incomplete.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A certificate-owned property can refer to a nonexistent `$defs` target and the conformance test still passes, because the helper checks only that `$ref` is a string and never resolves it. | tests/conformance_qspec/certificate_member.rs:29 |

## Coverage

- Plan completion: not assessed
- FR-070-AC-1, examined: public import and derive assertions through the integration test; source inspection confirms no QSL/backend dependency.
- FR-070-AC-2, examined: one test traces the criterion and checks current schema set equality, duplicates, and missing top-level definitions; a dangling nested reference leaves the collected names unchanged and is not refused.
- FR-070-AC-3, examined: ordered `model == published` assertion would fail on a schema-only reorder.
- FR-070-AC-4, examined: all currently listed variants return a plain canonical name; no locus structure or serializer exists in the model module.
- FR-019-AC-5, examined: inventory row changes from planned to implemented and the root export is explicit.
- Computed `quire matrix --scope . --format json`: all four FR-070 criteria are tagged to `model_certificate_members_match_qspec_schema` at line 105; the matrix cannot distinguish the malformed-reference gap.
- Optional repository-wide semantic review: not selected; targeted PR-diff AC-to-test inspection performed.

## Evidence

For example, changing the `CertificateIdentity.properties.node_id.$ref` value in the configured QSpec schema to `#/$defs/DoesNotExist` leaves every property name and malformed-member enum entry unchanged. `certificate_properties` calls `as_str()` on that reference and accepts it; all final set and sequence assertions still pass. The input is a malformed relevant definition, which FR-070-AC-2 says must fail clearly. The test is tagged to all four FR-070 criteria and passed against QSpec main `6fee29e352dfacfe403fe86ba1d0eb0e3b18c56a`; this positive run does not exercise the malformed reference.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Commit `ad10bb4e` passes the complete schema `$defs` map into the certificate-owned property walk, rejects empty or missing local targets, and adds a tagged negative test for `#/$defs/DoesNotExist`. The focused QSpec conformance run passed both tests. |

## Round 1 evidence

The walk now validates each local `$ref` under the certificate-owned definitions against the referenced schema's `$defs`. The new `certificate_property_refuses_dangling_local_definition` test constructs an owned property with a missing local target, asserts an error, and checks the target appears in the diagnostic. The original positive QSpec schema test remains present and passes. No new gap was found in this fix slice.
