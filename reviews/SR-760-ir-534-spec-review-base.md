---
id: SR-760
title: "base and correctness review of PR 252 (IR-534 selections bind by identity)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@31d2f9654bd4136ae64c1500074ce01793382af8; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (classes 1-4, ModelOwner, Selections bind by identity, step 1, Dependency selections, AC-2/10/19/20/27/31/32/45/62/63/64); QSpec origin/main proposals/checked-package-v2/schema.json, FR-322, FR-321, FR-154; crates/quire-contract-model/src/checked_package/v2/{mod.rs,model_members.rs,dependency_references.rs}"
review_set: subset
---
# SR-760: base and correctness review of PR 252 (IR-534 selections bind by identity)

## Summary

Ticket: IR-534. I checked every amended and new criterion against QSpec origin/main
(read-only fetch of agent-ix/quire-specification) and against the reader code at the reviewed sha.

Confirmed independently, not from the PR body:

- QSpec `schema.json`: `ModelRef` is closed `{identity, digest_domain: const sha256-jcs, digest}`;
  `DependencySelection` is closed `{identity, package_id}`. Both are the item types of
  `PackageLock` and `IdentityPreimage` (`model_selections` and `dependency_selections`, both
  `uniqueItems`). The only `version` members left in the schema are format tags
  (`IdentityPreimage.version` and the four nominal preimage `version` consts). No selection
  carries a version.
- QSpec FR-322 step 1 compares the document's identity only, and step 2 says the declared version
  label is not part of the key. FR-322-AC-32 and FR-154-AC-2 say the same.
- Code today: `CheckedDomainPackageRef` and `CheckedDependencySelection` are `deny_unknown_fields`
  and carry `version` (mod.rs:146-172). The class-2 sweep (mod.rs:935-949) is the
  same-identity, different-version `malformed_wire` check. The shape check (mod.rs:1092-1105) runs
  before the same-locator digest sweep (mod.rs:1106-1118), which runs before any per-row
  evidence. `admit_document` compares identity and then version (model_members.rs:891-896).
  Dependency binding is already version-free (dependency_references.rs:100-117). So the amended
  four-class order matches today's code once the version sweep and the version shape member are
  removed. The amended criteria describe reachable behaviour after IR-535, and every refusal code
  named is in the reader's taxonomy.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The amended spec says only that the document's own version "is compared with nothing". It does not say whether a domain package document must still carry `package.version`. Today `semantic_ir_identity` (model_members.rs:901-907) returns None when `package.version` is absent or not a string. `admit_document` then refuses `invalid_model_binding`/`wrong-model-selection` at the row's `identity` (model_members.rs:895), and `read_semantic_ir` keeps `DomainModel.version` (model_members.rs:1161,1191). As written, IR-535 could keep requiring the member and misattribute a version-less document as an identity mismatch, or drop it. QSpec FR-322 step 1 and FR-154's admission table read only the identity. The spec should state that step 1 reads `package.identity` alone, and that `package.version` is neither required nor read by the reader: a document with no `package.version`, or a non-string one, admits when its identity and digest match. Add that case to AC-27 or AC-64 and to TC-048. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:881 |
| FND-002 | medium | Keeping same-identity, different-digest as `stale_dependency` is defensible: it is the code AC-10 already pins on main, and the reader already sweeps it (mod.rs:1106-1118). But it is an IR-local choice where QSpec is silent. FR-322 says only that FR-321 admits one selection per identity. FR-321-AC-5 refuses a second selection of one identity as `duplicate_selection`/`duplicate-identity`, which is checker-side and not in the FR-322 reader vocabulary. The nearest QSpec reader rule is the `DependencySelection` one: a second entry of one identity refuses `invalid_package`/`conflicting-definition`. FR-038 itself uses that code for dependencies. The new section's third bullet presents the model and dependency rules as one rule, although their codes differ, and nothing says the model code is not taken from QSpec. `malformed_wire` would be worse: it would merge class 4 into class 1/3. Keep `stale_dependency`, add one sentence that QSpec's reader assigns no code to this case and IR keeps AC-10's, and ask QSpec which reader code applies. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:874 |
| FND-003 | medium | AC-63 says an entry carrying `version` refuses `unknown_member` at that member, without qualification. The Dependency selections prose (line 502-506) and `classify_dependency_entry_shape` (mod.rs:529-570) treat an entry that lacks a required member and carries an extra one as the wrong shape as a whole: `malformed_wire` at the entry. An old-shape entry such as `{identity, version}` with no `package_id` therefore gets two contradictory outcomes, `unknown_member` from AC-63 and `malformed_wire` from the prose and code. AC-62 has the same overlap for a model row that lacks a member and carries `version`, where it gives `unknown_member` and `malformed_wire` at once (serde reports the unknown key first in canonical order). Restrict AC-63, and AC-62, to an entry carrying every required member plus `version`, and say which outcome the lacking-plus-version case draws. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:916 |

## Verdict

Request changes, no high finding. The amendment is faithful to QSpec, explained per AC, and adds no
compatibility layer. `version` is refused by the closed shape, which is a refusal control.
Three medium gaps remain:

- FND-001: the document's own `package.version` is left unspecified.
- FND-002: the same-identity code is an unrecorded IR-local choice.
- FND-003: AC-63 contradicts the entry-shape prose.

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@a1f44f02c78fa786dc3e5582a45f46a069d74bd5 (delta from 31d2f9654bd4136ae64c1500074ce01793382af8; base origin/main eedc378d7fe879e0d107e04318773528f9595422 unchanged; merge clean). `make spec` at round 1: validate passes, 1 grammar finding (baseline), 23 strict unbacked rows (baseline), 163/204 rows backed, FR-038 42/62. No new finding.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | a1f44f02c78fa786dc3e5582a45f46a069d74bd5 |
| FND-002 | fixed | a1f44f02c78fa786dc3e5582a45f46a069d74bd5 |
| FND-003 | fixed | a1f44f02c78fa786dc3e5582a45f46a069d74bd5 |
