---
id: SR-202611010103
title: "IR-1250 PR 796 certificate vocabulary base spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; IR-1250; PR #796; spec/model/functional/FR-070-certificate-member-vocabulary.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Reviewed the FR-070 vocabulary contract and FR-019 planned export row against QSpec FR-331-AC-46, TC-431 BP-42 and the published schema at the source checkout examined for this review. No applicable AssuranceProfile was found under `spec/`. The set equality and independent IR order choice are clear; the source revision pin violates the repository's requirement policy.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-070 Inputs pins the source checkout to a commit, and AC-2 depends on that pin. Remove the commit threshold from normative text; identify the configured QSpec checkout and require its referenced schema and FR-331 contract to be read and validated. Keep the examined source revision in review evidence only. | spec/model/functional/FR-070-certificate-member-vocabulary.md:26; FR-070, FR-070-AC-2 |

## Coverage

- FR-070 Description, Inputs, Outputs and Behavior, role: examined. Checked owner, schema source, set rule, duplicate handling and IR-only ordering.
- FR-070-AC-1 through FR-070-AC-4, role: examined. Checked public API, conformance, order and QSL boundary.
- FR-019 public inventory row, role: examined. The row explicitly marks the export planned for IR-1250, so it does not claim existing code.
- QSpec FR-331-AC-46, TC-431 BP-42 and backend-provider schema, role: context_only. QSpec requires 51-name set equality and says the set order is not normative; the published malformed-member enum contains 51 entries.
- QSL `qsl-replay::CertificateMember`, role: context_only. Its local enum currently has the same published spellings and serializes a one-member certificate locus.

## Evidence

The repository's `AGENTS.md` and `CLAUDE.md` prohibit introducing SHAs or pins, and the quoin base checklist classifies any requirement pin as high. FR-070 Inputs names an exact commit; AC-2 tests against “the stated commit or later.” The PR diff contains no schema copy or compatibility layer. The source checkout revision used for this review is evidence, not an appropriate normative threshold.
