---
id: SR-581
title: "PR #108 review — CheckedPackage V2 re-pin to QSpec 5626bc8 (AD-006)"
type: SpecReview
analysis: gap-analysis
scope: "Contract IR PR #108 diff (origin/main...ea91a5c): FR-038, TC-047..TC-050"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: reviews }
  - { target: "ix://agent-ix/quire-contract-ir/TC-048", type: reviews }
  - { target: "ix://agent-ix/quire-contract-ir/TC-049", type: reviews }
---

## Summary

One review of the PR #108 diff only: code review, Rust review and gap analysis.
The PR re-pins the V2 reader to quire-specification `5626bc8f`. `ModelRef` is
now `{identity, version, digest_domain: sha256-jcs, digest}`. `ModelOwner` is
now `{kind, identity, node}`. `model_export` is gone.

Checked against upstream:

- **Vendored bytes.** All 11 vendored blobs equal the upstream tree at `5626bc8`.
  The 3 changed files' SHA-256 values match `PROVENANCE`. The compare
  `5aa00f3...5626bc8` changes only README, schema and node-identity-preimage
  among the vendored paths. The two new model-effective-declaration files are
  honestly listed as not vendored. The 6 normative requirement blobs named in
  `PROVENANCE` match the tree.
- **Strict ModelRef.** `CheckedDomainPackageRef` uses `deny_unknown_fields`.
  The retired lock shape and owner shape (`authority`/`revision`/`export`)
  refuse as `unknown_member`. A foreign domain refuses as
  `digest_domain_mismatch`, checked before shape. An empty version refuses as
  `malformed_wire`. A version with no evidence refuses as `stale_dependency`.
  Each case is also checked against the vendored schema.
- **Typed sha256-jcs evidence.** `CheckedDomainPackageLocator` keys a separate
  `domain_packages` map. `validate_domain_package` never reads raw artifact
  evidence. A test proves that equal digest bytes attested only as a raw
  artifact refuse.
- **Owner join.** A model owner joins by domain package identity and needs a
  nonempty `node`. An unselected identity, an empty lock or an empty node
  refuses as `invalid_semantic_graph`.
- **Migration.** `models_correspond` accepts only when neither side selects a
  model. Nothing is inferred. It is tested in both directions (unit test) and
  end to end (a target selecting a domain package refuses as
  `migration_target_incompatible`).
- **V1 unchanged.** `v1.rs` and `common.rs` are not in the diff. The evidence
  change only adds a map. The test helper still attests V1 compiled models as
  raw artifacts. `checked_package_v1_golden` and `checked_package_dispatch`
  pass.

Spec gaps the implementer flagged (not findings against this PR):

- *Migration contract still in the old model form.* Handled conservatively:
  any model selection on either side is `migration_target_incompatible`, and
  FR-038 says so.
- *`node` not joined to the lock.* Handled honestly: only presence is
  required, and FR-038-AC-5 and a code comment say so. No lookup is invented.
- *No sha256-jcs evidence rule.* Handled conservatively: the digest must be
  attested by the caller under (identity, version). Otherwise it is stale.
  Nothing is computed or inferred.
- *Duplicate V2 model selections not forbidden.* Not refused. This matches the
  reader's existing treatment of source and definition selections, and inventing
  a rule is outside the PR. An exact duplicate `(identity, version)` cannot carry
  two digests, because the evidence has one entry. Two versions of one identity
  both satisfy an owner join. That is consistent with node preimages excluding
  versions. Acceptable; FR-038 is silent on it.

Targeted tests run (Rust 1.98.1): `checked_package_v2_reader` (9),
`checked_package_migration` (3), `checked_package_v1_golden` (1),
`checked_package_dispatch` (3) and `quire-contract-model --lib tc_049` (3). All
pass. rustfmt `--check` is clean on the touched Rust files.

## Verdict

**FAIL**: one touched acceptance criterion has no test for the new model
selection shape (FND-001). The fix is small and belongs in this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-4 says editing a selection changes the package id and refuses unless mirrored. This PR replaced the model selection type inside `identity_preimage`, but the `included` mutation list only edits required feature, edition, recursion group and profile selection. No case edits `lock.model_selections`. Fix: add `("model selection", Box::new(\|v\| v["lock"]["model_selections"] = json!([{"identity":"test/orders","version":"1","digest_domain":"sha256-jcs","digest":"5".repeat(64)}])))` to the list. The existing loop then asserts `stale_dependency` when not mirrored, and a changed, re-derived id once `refresh_identity` mirrors it (`evidence_for` already attests V2 domain packages). | tests/checked_package_v2_reader.rs:546-576; FR-038-AC-4 |
| FND-002 | low | `use serde_json::json;` came after the test fn inside `mod tests` rather than with the other `use` lines. Resolved by deletion: the module it sat in is removed with the migration contract. | (module deleted) |

## Coverage

- Touched criteria: FR-038-AC-2 is backed by `tc_048_model_owners_join_sha256_jcs_domain_package_selections` and `tc_048_model_export_is_not_a_v2_model_form`. FR-038-AC-5 is backed by `tc_048_model_owners_join_sha256_jcs_domain_package_selections`. FR-038-AC-1 is backed by the contract-version refusal test. FR-038-AC-4 has a gap for model selections (FND-001).
- Untraced behaviours / stubs: 0 in the diff.
- Semantic review: done for the model-selection paths above; intent, test and code agree apart from FND-001.
