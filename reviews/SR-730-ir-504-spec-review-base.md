---
id: SR-730
title: "spec review of PR 249 (IR-504 content-only ModelOwner identity)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@fdad364d27f9e77c4525d06f2653eb2e69eca569; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/tests.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; compared with origin/main, crates/quire-contract-model/src/checked_package/v2/{identity.rs,model_members.rs,operations.rs,mod.rs}, and quire-specification origin/main c76c6ae proposals/checked-package-v2/{schema.json,node-identity-preimage.schema.json}, spec/objects/interfaces/FR-321 and FR-322"
review_set: subset
---
# SR-730: spec review of PR 249

## Summary

Ticket: IR-504. Reviewed head fdad364 against origin/main. One commit, three spec files.

Measured independently:

- Code today: `NominalOwner::Model { identity, version, node }` with
  `deny_unknown_fields` (`identity.rs:44-70`). `validate_owner` joins by identity alone and
  also requires a nonempty `version` and `node` (`identity.rs:536-546`).
  `locate_owner_failure` lists `["kind", "identity", "version", "node"]` for `model`
  (`identity.rs:214`). An unknown owner member is classified `unknown_member`
  (`common.rs:319`, kept by `locate_in_preimage` in `v2/mod.rs:506-530`). A missing or
  wrongly typed member becomes `invalid_semantic_graph`. So the refusal classes AC-45 names
  are the reader's real ones.
- The code also derives the `ModelDeclarationNode` key with a version-bearing owner:
  `declaration_key(identity, version, form, node)` builds
  `"owner": {"kind": "model", "identity", "version", "node"}` (`model_members.rs:239-254`).
  It is used for step 2 owner recovery (`model_members.rs:617,702`) and for reference
  member types (`operations.rs:1234`).
- QSpec main: `ModelOwner` is `{kind: model, identity, node}`, all required,
  `additionalProperties: false`, in both `schema.json` and
  `node-identity-preimage.schema.json:23`. `ModelDeclarationNode.owner` is `$ref ModelOwner`
  (`node-identity-preimage.schema.json:85`). FR-322-AC-28 says a `ModelOwner` preimage has
  no version label, and its closed schema refuses a `version` member.
- Citation: FR-322 is right. FR-321 (model selection artifact) does not mention
  `ModelOwner`, so the ticket's FR-321 is wrong. STD-145 is a QSpec Linear ticket (Done),
  not a spec file. It appears in QSpec only in review files. FR-038 already cites tracker
  ids this way (STD-125 at line 152, STD-105 at line 345), so the citation follows the
  file's convention. FR-322-AC-28 is the normative anchor.
- IR-243 asked whether a version-independent model-owned key is intended or a collision.
  STD-145 settles it as intended, so removing the IR-243 tracking claim is justified.
  IR-243 is still Backlog. IR-505's description takes on closing it.
- PR hygiene: the title has no ticket id. The body says `Closes IR-504`. No pins, SHAs,
  vendoring or compatibility layer. The new text explicitly says "not a reader for an
  earlier one". No requirement or AC was removed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The amendment makes only the nominal preimage owner content-only. QSpec's `ModelDeclarationNode` preimage also embeds `ModelOwner`, and IR derives that key with `version` (`declaration_key`). FR-038 step 2 owner recovery and the new paragraph and AC-45 leave that key version-bearing. An IR-505 that implements AC-45 as written would still refuse every QSL A3k package with a model-owned member as `missing_declaration`/`missing-selection`, so IR-504's goal ("a package QSL emits under A3k admits") is not specified. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:271-283,324-326,756 |

## Verdict

Changes requested. The nominal-owner amendment is accurate against QSpec's schema and
FR-322-AC-28. The refusal classes match the reader's taxonomy, and the IR-243 removal
is justified. But the content-only identity has to cover every `ModelOwner`-keyed preimage,
including the model declaration node key used in "Model-owned members" step 2 and in
reference member types. AC-45, or a sibling AC, needs a model-owned member resolving
under a content-only declaration key whose node key does not change with the domain
package version. Until then, `Closes IR-504` overclaims.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | b091fe5 |
