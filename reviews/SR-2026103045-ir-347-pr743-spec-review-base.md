---
id: SR-2026103045
title: "IR-347 PR 743 spec-review/base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/model/matrix/tests.md, spec/tests.md"
review_set: subset
---

## Summary

Reviewed PR #743 documentation status corrections against FR-028 AC-1 through AC-5 and the tagged TC-041 tests at frozen head 2a8ed518. The AC-1-through-AC-5 coverage statement overclaims AC-4 because its sole tagged test is ignored; QSL graph and cross-repository composition remain clearly partial.

## Verdict

**CONDITIONAL** — AC-4 is only tagged by an ignored test, so the new inclusive coverage statement needs correction. No applicable AssuranceProfile was found under `spec/`.

## Examined units

- **FR-028-AC-1** (`context_only`, `spec/model/functional/FR-028-separate-cycle-free-contract-model.md`): Cargo metadata for every production feature combination is acyclic and contains no owner or TL dependency reachable from `quire-contract-model`.
- **FR-028-AC-2** (`examined`, `spec/model/functional/FR-028-separate-cycle-free-contract-model.md`): Schemas, canonical bytes, identities, diagnostics and the conformance corpus are byte/result identical through `quire_contract_model` paths, and the root package's runner produces the same results from them.
- **FR-028-AC-3** (`context_only`, `spec/model/functional/FR-028-separate-cycle-free-contract-model.md`): The root package's own manifest names no dependency in any kind on a crate from the agent-ix/quire-spec-language repository, by name or by git source.
- **FR-028-AC-4** (`examined`, `spec/model/functional/FR-028-separate-cycle-free-contract-model.md`): Default, all-feature and minimum-version builds prove that no optional, dev or historical dependency leaks into the production graph.
- **FR-028-AC-5** (`examined`, `spec/model/functional/FR-028-separate-cycle-free-contract-model.md`): The split introduces no copied owner wire type, public validation constructor, callback, trait object, trust flag or local QSL/observation/protocol/TL parser.
- **FR-028 matrix status** (`examined`, `spec/model/matrix/tests.md`): | FR-028 | FR-028-AC-1 through FR-028-AC-5 | TC-041 | 🚧 AC-1 through AC-5 are backed by tagged tests in `tests/it/cycle_free_model.rs`. AC-2 compares model and root runner corpus results byte for byte, including canonical identities and diagnostics, and checks both paths' refusal of a changed schema. AC-5 checks the production source boundary for copied owner types, parsers and admission escape surfaces, with adverse controls. AC-3 checks that the root manifest has no QSL-repository dependency by name or git source. QSL-side graph and cross-repo composition checks remain planned in agent-ix/qu
- **TC-041 status** (`examined`, `spec/model/matrix/tests.md`): | TC-041 | Reach the model through `quire_contract_model` while proving the model/owner/root Cargo graph is acyclic | Integration | P0 | FR-028-AC-1, FR-028-AC-2, FR-028-AC-3, FR-028-AC-4, FR-028-AC-5 | 🚧 AC-1 through AC-5 have tagged tests in `tests/it/cycle_free_model.rs`; AC-2 covers model/root corpus parity and changed-schema refusal, and AC-5 covers the source boundary with adverse controls. QSL-side graph and cross-repo composition checks remain planned in agent-ix/quire-integration (Linear IR-358). |
- **Model index status** (`examined`, `spec/tests.md`): | Model | FR-013, FR-014, FR-015, FR-016, FR-017, FR-019, FR-023, FR-028 | [model](./model/matrix/tests.md) | 🚧 FR-013-AC-5 and FR-016-AC-5 through FR-016-AC-8 are implemented (IR-274 code change B); FR-019-AC-5 is implemented and backed by TC-058, including the no-glob, inventory and path probes; FR-019-AC-6 artifact-reference member sets are backed by TC-018. FR-028-AC-2 and AC-5 are backed by tagged TC-041 model/root corpus-parity, changed-schema refusal and source-boundary tests; the QSL-side graph and cross-repo composition checks are outside this repository. FR-023-AC-6 is implemented in

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-028-AC-4 is tagged only by an ignored build-lane test, so the new AC-1-through-AC-5 coverage statement overstates normal runnable evidence. | spec/model/matrix/tests.md:20,32 |
