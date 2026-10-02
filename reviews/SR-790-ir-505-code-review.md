---
id: SR-790
title: "code review of PR 250 against IR-505 (content-only ModelOwner)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@286dc707986d19f2d9a38f3b752be436d5b278ec; crates/quire-contract-model/src/checked_package/v2/{identity.rs,model_members.rs,operations.rs}; tests/it/checked_package_v2_{reader,model_members,frame_entries,dependency_reference}.rs; diff origin/main...HEAD (merge base db5ca1c)"
review_set: subset
---
# SR-790: code review of PR 250 against IR-505

## Summary

Ticket: IR-505. Code review with the rust-review lane, scoped to `git diff origin/main...HEAD`.
The merge base is db5ca1c. origin/main has since moved two commits (#246 KaniProfile, #245
expression.rs). Neither touches a file in this diff, and GitHub reports the PR mergeable.

What was checked independently, at the reviewed head:

- `NominalOwner::Model` is `{identity, node}` with no `version`. The enum keeps
  `deny_unknown_fields`, so a `version` member is an unknown member.
  `locate_owner_failure` lists `["kind", "identity", "node"]` for the model kind.
  `validate_owner` requires a nonempty node and identity, then joins by identity.
- `declaration_key(identity, form, node)` has no version parameter. A grep for every
  caller found three call sites: `DomainModel::element_type` (model_members.rs:616),
  owner recovery in `ModelOwners` (model_members.rs:695) and `check_reference_edge`
  (operations.rs:1234). All three are version-free. No other code builds a model owner
  or a declaration key. That grep covered `src/`, `crates/`, `tests/`, `tests/fixtures`,
  `spec/` and `reviews/`. The only remaining versioned owner text is in the historical
  review `reviews/SR-730-ir-504-spec-review-base.md`, a record that stays as written.
- The preimage bytes match QSpec. In quire-specification origin/main (c76c6ae),
  `proposals/checked-package-v2/node-identity-preimage.schema.json` and `schema.json`
  define `ModelOwner` as exactly `kind`/`identity`/`node`, with
  `additionalProperties: false`. The schema's `ModelDeclarationNode` member set
  (`version`, `node_tag`, `semantic_form`, `semantic_type`, `declaration`, `recursion`,
  `owner`, `body`) matches the JSON that `declaration_key` hashes.
- There is no compatibility layer. No legacy path accepts a versioned owner. The diff
  adds no pins, SHAs or vendored files. It copies no QSL fixture: the PR notes say so,
  and the diff confirms it.
- The diff has no panics, `unwrap` calls, `unsafe` blocks, integer casts or new public
  API.

Tests run in my own worktree with its own target dir:
`cargo test --test it checked_package_v2` gave 102 passed.
`cargo test -p quire-contract-model --lib` gave 88 passed.

Mutation probes (each reverted afterwards):

1. Put a `version` member back into the hashed owner in `declaration_key`. 8 tests fail:
   the new AC-45 test in model_members, the AC-3/AC-7/AC-8-9 frame-entry tests, both
   reaches_field tests and two TC-048 model-owned operation tests. The test helpers
   compute version-free keys on their own, so key drift between the reader and the spec
   shape is caught.
2. Add a legacy `#[serde(default)] version: Option<Box<str>>` to `NominalOwner::Model`.
   `tc_048_a_content_only_model_owner_admits_and_its_keys_ignore_the_selected_version`
   fails on the `unknown_member` assertion, so the no-compatibility-layer rule is
   enforced.
3. Remove the new `!identity.is_empty()` conjunct. All 102 tests still pass. That
   conjunct cannot be observed: FR-038 lock class 4 refuses a selection with an empty
   identity, so an empty owner identity never joins. The behaviour test still holds
   (empty identity refuses `invalid_semantic_graph`). The guard is defensive, not
   load-bearing. I do not record this as a defect.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The edited `validate_owner` comment reads "the join is by package identity and identity and node are only required to be nonempty". The two clauses are spliced together, so it reads as a typo for what is meant: the join is by package identity, and identity and node need only be nonempty. | crates/quire-contract-model/src/checked_package/v2/identity.rs:527 |
| FND-002 | low | The re-keyed AC-3 and AC-7 refusal cases are still labelled "an unselected version". They now key the node under another domain package (`acme/other`), so the label names the retired fixture and misleads anyone reading a failure message. | tests/it/checked_package_v2_frame_entries.rs:1562 |

## Verdict

The code change is correct and minimal, and it matches QSpec's `ModelOwner` and
`ModelDeclarationNode` preimages byte for byte. Every `declaration_key` caller is
version-free. A versioned owner refuses `unknown_member`, and no reader path accepts
one. The re-keyed fixtures are not vacuous. They were re-keyed because, under
version-free keys, the old other-version fixture resolves and admits. The new keys put
the node under an unselected package identity and refuse
`missing_declaration`/`missing-selection` at the asserted member pointer. Probe 1 shows
these tests and the AC-45 tests fail if a version comes back into the key. There is no
externally anchored byte vector from QSL. The tests' oracle is an independent
re-implementation of the preimage shape, checked here against the QSpec schema. A
shared misreading of the schema by reader and tests would go uncaught. SR-791 FND-003
covers that cross-producer check. Two low findings, both cosmetic.

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 90f6a3a33ceee1315cdea212aca8b4c503636bf5: identity.rs:527-529 now reads "the join is by package identity, and the owner's identity and node must each be nonempty (FR-038-AC-45)". |
| FND-002 | fixed | 90f6a3a33ceee1315cdea212aca8b4c503636bf5: both cases (frame_entries.rs:1562, :1616) are relabelled "another domain package". |
