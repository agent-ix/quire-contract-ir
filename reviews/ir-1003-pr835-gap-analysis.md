---
id: SR-2026110283502
title: "PR 835 FR-352-AC-9 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1003; Cargo.toml, Makefile, tests/conformance_qsl_exact_producer/; FR-352-AC-9"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-contract-ir/FR-352", type: references}
---

## Summary

Scoped independent review of PR 835's real QSL selected-model producer qualification. No substantive defects found within the requested criterion and changed paths.

## Verdict

PASS within the FR-352-AC-9 scope. This is not a repository-wide assurance or merge verdict.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-352-AC-9, role: examined, path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md. Excerpt: A QSL-side selected-model identity producer supplies `SelectedExactIdentities` alongside a checked package; an external caller passes it to the public projection API. The IR join accepts identities only for matching admitted selected-document, declaration form and original IR declaration or field identities. Absent, conflicting, foreign-document and wrong-form entries return the named typed outcome; a caller-supplied bare `NodeKey`, `EffectiveId`, `MemberId`, or node-name pair cannot enter the projection as an authoritative witness.
- Entire changed Cargo.toml, Makefile and both producer test sources examined; no CI workflow gate narrowed. Production join/projector and QSL graph producer inspected as supporting context.
- Reconciliation: quire matrix --scope . --format json computed during review; FR-352-AC-9 has one nonignored test_function binder at tests/conformance_qsl_exact_producer/main.rs:45. Quire coverage receipt removes precisely this unbacked row (94 to 93), adds none, and has zero status contradictions. The remaining repository gaps are outside this scoped verdict.
- Plan completion: not assessed
- No new unowned behavior, stubs, replaced production path, hand-minted positive identity witness or inflated tag found in scope.
- Semantic inspection explicitly requested in reviewer brief for this one criterion; no repository-wide optional semantic pass performed.
- No applicable AssuranceProfile found in spec.

The external helper uses public QSL parse, form building, model intake/admission, assembly/checking, link and emission. Strict IR read uses the same selected document bytes, then the checked graph supplies selected_exact_identities(&admitted). Record and reference projections consume that joined witness. The inherited serial field and unread amount/confirmed fields are checked against genuine producer MemberIds; adverse declaration/field document, original identity, form, namespace, duplicate and absent entries assert named typed outcomes. All four compile-negative consumers run after the positive consumer succeeds and require a sole E0308 diagnostic naming SelectedExactIdentities. Metadata verifies one model crate instance at the tested workspace path.

The negative probes establish the joined-witness parameter boundary only. Public trusted-stage constructors remain available, and opaque Exact payload authenticity remains the producer's responsibility; this is not a Rust language unforgeability claim. No public shim, production API, spec, dependency or lockfile changed.

## Validation evidence

Existing receipts inspected, without repeating heavy builds: native producer lane passes one test and all four compile-negative probes; root workspace Clippy default/no-default lanes and formatting pass; independently linted byte-identical helper passes Clippy. Helper lint and native runtime resolved different quire-walk inputs; these are separate source-input evidence, not one identical binary cohort. Full aggregate verification remains the lead's gate. QSL and QSpec inputs are read-only; no QSpec fixture is copied.
