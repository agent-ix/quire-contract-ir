---
id: SR-11000
title: base review of IR-782 protocol reference reader specification
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md
review_set: subset
---

## Summary

Reviewed the three changed IR-782 spec files against QSpec FR-322-AC-66, FR-330-AC-4 and TC-931 at 6abc5fe, and the current Contract IR reader sequence. No defect found in this analysis.

## Examined scope

- `FR-038` (examined, spec/checked_package/functional/FR-038-consume-checked-package-v2.md) — When Contract IR receives checked-package bytes, the consumer shall parse
`contract_version` exactly once before any further decoding, admit a
`quire.checked-package/v2` package only after re-deriving every package and
nominal node identity, and lower admitted items independently. Any other
contract version is refused with a typed `unknown_contract_version` code.
- `TC-048` (examined, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md) — For FR-038-AC-239 through FR-038-AC-241, construct a local
`protocol`/`protocol_clause` target and a `protocol`/`protocol_reference`
carrier using the public typed vocabulary. Give the carrier one `expression`
occurrence, a closed `reference` body, the target's exact typed key as body
target and sole dependency, and the target's semantic type. Independently
compute the carrier's QSL FR-092 structural preimage and canonical key,
mirror it in the graph identity projection, refresh the package id, and read
the package through `CheckedPackageV2::read`. Check the typed form, exact
local target, si
- `FR-038 matrix row` (examined, spec/checked_package/matrix/tests.md) — 🚧 AC-239 through AC-241 (IR-782) are PLANNED/UNRUN: local protocol-reference carrier vocabulary, shape, identity and target join. ✅ AC-27, AC-37, AC-64 and AC-215 through AC-218 include the implemented selected-model, dependency-selection and own-package `content-mismatch` branches with typed expected/actual identities (IR-238); existing selection, missing-evidence, owner and version cases remain implemented. 🚧 AC-210 through AC-213 (IR-484 operand families and literal/mode checks) are PLANNED/UNRUN and unbacked by code; compound-unit, enum-alias and model-population family decisions await ups
- `FR-038-AC-239` (examined, spec/checked_package/functional/FR-038-consume-checked-package-v2.md) — PLANNED/UNRUN (IR-782). A package with a `protocol`/`protocol_reference` carrier whose closed reference body targets an exact local `protocol`/`protocol_clause` key, with one `expression` occurrence, the same semantic type and exactly that target as its dependency, admits through the public V2 reader as `ProtocolForm::ProtocolReference`. The structural key, identity projection and package id are independently recomputed; changing the target and re-deriving these identities changes the carrier key. The admitted package retains one graph node collection and its source correspondence.
- `FR-038-AC-240` (examined, spec/checked_package/functional/FR-038-consume-checked-package-v2.md) — PLANNED/UNRUN (IR-782). An extra dependency or a dependency unequal to the carrier's body target refuses `invalid_semantic_graph` at `/semantic_graph/nodes/{i}/dependencies` in graph-shape validation; a stale carrier key with matching dependency refuses `invalid_package`/`stale-node-key` before the local target join. With both body target and sole dependency changed together and identities independently recomputed, an absent local target refuses `missing_declaration`/`missing-name` at `/semantic_graph/nodes/{i}/body/target`, even if an admitted selected dependency contains that key. A present 
- `FR-038-AC-241` (examined, spec/checked_package/functional/FR-038-consume-checked-package-v2.md) — PLANNED/UNRUN (IR-782). A carrier with a non-reference or `dependency_reference` body or an extra body member fails the closed V2 schema. A carrier with an extra occurrence, declaration, owner, nominal preimage or recursion group refuses at its offending member before target resolution. An ordinary node holding a generic reference remains an ordinary node; no second protocol-reference form or separate collection is admitted.

## Verdict

PASS: no findings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
