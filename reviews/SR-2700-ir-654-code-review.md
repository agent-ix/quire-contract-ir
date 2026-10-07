---
id: SR-2700
title: "IR-654 code-review (rust-review lane) of the QSpec selection-evidence conformance harness"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir PR #319; Makefile, crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs, tests/conformance_qspec/main.rs, spec/checked_package/functional/FR-038-consume-checked-package-v2.md (FR-038-AC-176), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; ticket IR-654"
review_set: subset
---

## Summary

Ticket: IR-654, CODE stage, PR #319 (5 files). Code review with the rust-review lane folded
in. The reviewed commit identity is recorded only in the Linear marker, per this repository's
rule against commit identities in the tree. Model `claude-opus-5-5`; run
`bb45ccd3-2be1-4a34-8a7d-e6ec32c95115`. Static review only: the reviewer ran no cargo build or
test. The author reports Rust 429/429, model 178/178, and a conformance run of 11 passed tests
plus the model test, with the IR-630 test still ignored.

Checked against merged FR-038-AC-176 and the TC-048 AC-176 procedure. The reviewer read the
QSpec inputs by path only (the nine `positive-*.json` fixtures,
`domain-package-acme-orders.json`, `model-member-type-vectors.json` and
`node-identity-vectors.json`) to confirm what the harness relies on. Exactly nine positive
fixtures are published. `positive-all-families` and `positive-operation-identities` select
`acme/orders`. The two-owner fixtures each hold one `Point` and one `List` record with a
`source` owner. The twelve declaration vectors cover all three declaration forms.

## Verdict

**Merge-ready after FND-001 (medium, harness-only).** The rest meets the criterion:

- All nine positive packages are read by path from `QUIRE_SPECIFICATION_DIR` and must admit
  through `CheckedPackageV2::read`, with the typed `package_id` compared against the published
  one.
- The selected `acme/orders` document is supplied under the selection's own digest through
  `CheckedPackageEvidence::insert_domain_package_document`. The reader's `admit_document`
  recomputes the RFC 8785 digest of those bytes. A missing document refuses
  `missing_import`/`missing-selection`, and a mismatched one refuses
  `stale_dependency`/`byte-digest-mismatch`. The check is IR production code, not a hashing
  harness.
- Node ids and owners are read back through `admitted.graph().nodes`, with lengths asserted,
  and compared with both the published graph and the identity projection. The `Point` and
  `List` ids differ across the two source owners. IR-630 re-derivation is not claimed.
- The production key oracle calls the `pub(super)` `declaration_key`, the same function the
  reader calls at `model_members.rs:1030`. It runs from a `#[cfg(test)]` child module through
  `use super::*`, so no production API changes. Corrupting `DeclarationForm::tag`/`form`, the
  `ModelOwnerPreimage`, or the `StructuralPreimage` serialization would fail the digest
  comparison. Hashing the vector JSON cannot pass.
- The fail-closed cases are a missing fixture, a missing document, a malformed document, an
  unknown identity, a missing digest and a changed package id. Each starts from a fresh input
  and asserts its own error.
- The diff copies no fixture and adds no digest, pin or checksum catalog. The only hex values
  are `"0".repeat(64)` placeholders. There is no QSL-emitted golden.
- There is no file overlap with PR #318 (IR-680, `tests/it/**` and production refusal files),
  and `git merge-tree` of the two heads is clean.
- Rust idioms: panics and `expect` appear only in test code, the `code_word`/`cause_word`
  matches stay exhaustive, and there is no `unsafe` and no new dependency.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The production-key oracle for `model-member-type-vectors.json` can stop running while every gate stays green. The private test returns early when `QUIRE_SPECIFICATION_DIR` is unset, and `make conformance-qspec` selects it with a name filter, which passes on zero matches | crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs:19-21; Makefile:84 |
| FND-002 | low | The harness hard-codes QSpec's vector count (`assert_eq!(rows.len(), 12)`). This makes IR a second owner of a QSpec fact the criterion does not fix ("every" vector) | crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs:31-35 |
| FND-003 | low | Three fail-closed assertions do not pin the affected file or selection, which TC-048 requires: malformed document (`"is not JSON"`), missing digest (`"no digest"`) and non-admitted package (`"did not admit"`) | tests/conformance_qspec/main.rs:563; tests/conformance_qspec/main.rs:577; tests/conformance_qspec/main.rs:582 |
| FND-004 | low | `derived_shape_mutations` changes the AC-123/AC-133/AC-150 harness from "exactly one matching node" to "every matching node". This is outside AC-176, and neither the PR body nor the TC text discloses it | tests/conformance_qspec/main.rs:597-667 |

### FND-001 detail

`tc_048_qspec_model_declaration_keys_use_production_derivation` starts with
`let Ok(root) = std::env::var("QUIRE_SPECIFICATION_DIR") else { return; };`. Under
`make test` (`--workspace --all-targets -- --include-ignored`) it reports `ok` with no
assertion, yet it carries `#[trace("TC-048", "FR-038-AC-176")]`, so coverage credits it. In
`make conformance-qspec` it is selected by
`cargo test --workspace --lib tc_048_qspec_model_declaration_keys_use_production_derivation`.
libtest exits 0 when a filter matches no test.

Two mutants survive every gate:

- **(a)** Rename the function, or move it behind a `cfg`. The filter then matches zero tests,
  and the target stays green with no vector compared.
- **(b)** Mistype the env-var literal, which is spelled out here rather than shared with the
  harness constant. The test then always returns early, and the conformance target stays green
  because the first command still sets the right variable for the harness.

Either way the key oracle, the AC's central "hashing only the JSON cannot pass" guard, stops
running unnoticed.

Fix: make the conformance run fail closed for this test too. Options:

- Have the Makefile export a required flag, such as `QSPEC_CONFORMANCE=1`, and make the test
  panic when the flag is set and the directory is not.
- Or run the test with `-- --exact checked_package::v2::model_members::tests::<name>` and
  assert the `1 passed` count.

Since `make test` passes `--include-ignored`, `#[ignore]` alone does not fix this.

### FND-002 detail

The criterion says "for every `model_declaration_nodes[*].preimage`". If QSpec publishes a
13th vector, IR fails even though nothing is wrong, and the IR test then has to be edited to
re-record a QSpec fact. The count does guard against an empty list making the loop vacuous,
but `assert!(!rows.is_empty())` gives that guard without owning the number. The three forms
could instead be required to each occur at least once.

### FND-003 detail

TC-048 says to "assert a failure with the affected file or selection named". The produced
messages do include the name today, but the assertions match only generic text. A mutant that
drops the path or selection from the `read_package_with_document` or `read_positive_package`
error formats survives. Assert the affected name instead:

- the scratch path for the malformed document
- `acme/orders` for the missing digest
- `positive-all-families.json` for the non-admitted package

### FND-004 detail

`derived_shape_mutation` used to refuse when a mutation's tag and form matched anything other
than exactly one node. It is now `derived_shape_mutations`, which mutates each match
separately. This plausibly follows QSpec fixture drift and is not weaker coverage. But it
removes the uniqueness check on QSpec's mutation target and changes what the AC-123/AC-133
and IR-630 tests mean. The PR body and TC-048 record only AC-176. Name the change in the PR
and TC text, or split it out.
