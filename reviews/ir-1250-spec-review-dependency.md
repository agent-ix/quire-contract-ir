---
id: SR-202611010106
title: "IR-1250 PR 796 certificate vocabulary dependency review"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-contract-ir; IR-1250; PR #796; spec/model/functional/FR-070-certificate-member-vocabulary.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Checked the added FR-070 relationship edges and downstream ownership sequence. QSpec supplies the normative schema, FR-019 inventories the model export, and QSL-1437 owns the later re-export; no circular crate dependency is specified.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-070 frontmatter relationships and Dependencies, role: examined. Checked `traces_to` FR-019 and `depends_on` QSpec FR-331.
- FR-019 planned inventory row, role: examined. Checked the new module/type maps to FR-070.
- FR-070-AC-1 and FR-070-AC-4, role: context_only. Read to identify model and QSL dependency directions.

## Evidence

FR-070 forbids a model dependency on QSL or a backend. The planned QSL re-export is assigned to QSL-1437, after model ownership, while the conformance check reads the QSpec checkout through a configured path rather than vendoring its schema.
