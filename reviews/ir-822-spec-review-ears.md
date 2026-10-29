---
id: SR-11851
title: "IR-822 EARS review"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; spec/conformance/functional/FR-020-json-conformance-interface.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-020
    type: references
---

## Summary

Ticket: IR-822. Examined the one new criterion and the edited Behavior clause for a single subject, a single observable response and a stated condition. `quire validate --scope. spec/conformance/functional/FR-020-json-conformance-interface.md` is grammar-clean.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-020-AC-15 | examined | A `document_json` expression document that decodes successfully and that the published schema rejects makes the runner exit 2 with the code `invalid_corpus`. |
| FR-020 statement | context_only | The repository shall expose a versioned JSON package interface and a... |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. FR-020-AC-15 is singular: one input class (a schema-rejected `document_json` expression document, with conjoined preconditions that define that class) and one response (exit 2 with `invalid_corpus`, which FR-020's exit-2 contract defines as a single outcome). It follows the declarative criterion form the other FR-020 criteria use (for example AC-7, AC-9); the `shall` statement lives in the FR body. The precondition's wording is raised separately as SR-11850 FND-001.
