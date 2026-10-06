---
id: SR-2100
title: "Code review \u2014 IR-661 FR-038/TC-048 relationship reader amendment"
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir@705cef3f7a07370f111f19e8e7d86e1e07fd31d7; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; base 1540b3b6c0e4d167fe1ed9116c296e45e7dff258; Ticket IR-661
review_set: subset
---

## Summary

Code review of the IR-661 spec-only diff (two Markdown files, 259 added lines, no source, Cargo, CI, Makefile, fixture or schema change). The rust-review checklist was loaded before reading Rust source for grounding. The Rust lane does not apply, because no `.rs` or `Cargo.toml` file changed. One P0 duplication finding: the amendment re-states merged QSpec FR-152's navigation rule in this repository's spec.

Ticket: IR-661. Method: `code-review`. Reviewer model `claude-opus-5-5`, run `236a8098-415e-43ce-aa4b-9a892aa7e251`.

## Verdict

**FAIL**: one `high` duplication finding (P0, not softened). Clean, as examined: TC-048 forbids copying FCD schema/fixture/binary bytes and asks for self-authored documents, so nothing is vendored; no limit is raised or added ("no new limit or budget domain"); registry agreement stays with the frontend, as FCD FR-094 assigns; no capability or execution is claimed, since every new criterion and procedure is marked PLANNED / UNRUN. Grounding: `operations.rs` today checks a `relationship_end` member for presence and kind only, so the relationship-end order this amendment calls "existing" is new behaviour to implement (see SR-2103).

## Scope Examined

- `FR-038#pinned-links` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:977-983
- `FR-038#receiver-endpoint` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1039-1047
- `FR-038#navigation-derivation` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1046-1054
- `FR-038#other-operations` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1058-1063
- `FR-038#declaration-order-accounting` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1065-1072
- `FR-038#operation-order-paths` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1078-1086
- `TC-048#fcd-relationships-status` (examined) spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:937-944
- `TC-048#fcd-step-1` (examined) spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:946-953
- `TC-048#fcd-step-6` (examined) spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1014-1022
- `operations.rs relationship_end scope note` (context_only) crates/quire-contract-model/src/checked_package/v2/operations.rs:31-38
- `QSpec FR-152 Navigation` (context_only) agent-ix/quire-specification@60630b0d3d5e9cca048d1c314675db3bf9c6e4f2:spec/functional/type-model/FR-152-bind-systems-model-structures.md:118-126
- `FCD FR-094 reader role clause` (context_only) agent-ix/filament-core-data@68c0acba2390eb1593cc003d0a3e144638540a86:spec/functional/FR-094-lower-relationships-operations-and-clauses.md:66-68

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038 re-states merged QSpec FR-152 Navigation's direction table and multiplicity-to-type derivation | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1046-1054 |

## Finding Detail

- **FND-001** (high, confidence medium, check `other`, unit `FR-038#navigation-derivation`): FR-038 re-states merged QSpec FR-152 Navigation's traversal-direction eligibility and its multiplicity-to-result-type derivation (`[0,1]` Option, `[1,1]` Reference, finite Set/Bag, unbounded or ordered refused `unsupported_construct`/`expression-form`) as its own normative prose, beside a link to FR-152. The only IR-specific content is which end is the destination and the wire paths. A later FR-152 change leaves two diverging statements of one rule, and the commit-pinned link (SR-2101 FND-002) hides the drift. The same requirement re-stated in a second repository's spec is duplication.

## Dispositions

Round 1, reviewed at `agent-ix/quire-contract-ir@c50050da5f20d4af41ab2dd6ea573d5e4b9abacc` (prior review `705cef3f7a07370f111f19e8e7d86e1e07fd31d7`), reviewer model `claude-opus-5-5`, run `ae1a4470-06bc-44c7-90df-81c70be53842`. Every outcome was verified against the fix commit's own text, not the author's receipt. All new criteria and procedures remain PLANNED / UNRUN; no implementation, CI gate or procedure coverage is claimed.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed c50050da5f20d4af41ab2dd6ea573d5e4b9abacc | The direction table and multiplicity-to-type derivation are removed. FR-152 Navigation is the canonical rule, applied by reference, and FR-038 keeps only the role-to-endpoint binding and the result-node comparison. TC-048's numeric cases are test inputs, not a second normative table. |
