---
id: SR-1127
title: "criterion strength review of PR 272 (FR-038-AC-112 and AC-113)"
type: SpecReview
analysis: criterion-strength
scope: "agent-ix/quire-contract-ir@29cdb1495b4a623e25df532705553da1e03d6112; spec/checked_package/functional/FR-038-consume-checked-package-v2.md FR-038-AC-112 and FR-038-AC-113; checked against quire-specification origin/main 396493c4 proposals/checked-package-v2/fixtures/adverse.json, fixtures/positive-all-families.json, dependency-selection-vectors.json, README.md, tests/checked_package_v2.rs, and against the production reader at the reviewed sha (throwaway probe, not committed)"
review_set: subset
---
# SR-1127: criterion strength review of PR 272

## Summary

Ticket: IR-552. AC-112 and AC-113 are new planned conformance-harness criteria.
Both are direct assertions with no "shall". Both are marked planned in the
matrix, and both fail closed when the variable is unset or empty, or when the
file is missing or is not JSON.

Measured against QSpec origin/main 396493c4:

- `adverse.json` has `structural_mutations` (6) and `body_grammar_mutations`
  (5). Every entry has `id`, `pointer`, `replacement` and `outcome`, and the
  body-grammar entries also have `flattened`. Every outcome is
  `refused:<code>` and none carries a `/cause`. Every pointer resolves in
  `positive-all-families.json`.
- `dependency-selection-vectors.json` has `base`
  (`fixtures/positive-all-families.json`), `dependency_selections` (two
  version-free `{identity, package_id}` entries), `package_id`
  (`0606043a...abf2`), `entry_mutations` and `order_vectors`. The AC's
  "version-free" claim matches the file. QSpec's README still describes the
  entries as `{identity, version, package_id}`, which is stale QSpec prose and
  not this PR's defect.
- A throwaway probe in the review worktree (discarded, never committed) applied
  each mutation literally to `positive-all-families.json` and called
  `CheckedPackageV2::read`. All six structural mutations refuse with the
  recorded code today. The five body-grammar mutations refuse with:
  `invalid_package`/`stale-node-key` at `/semantic_graph/nodes/5/node_id`;
  `invalid_semantic_graph` at `/semantic_graph/nodes/1/dependencies` (two of
  them); and `stale_dependency` at
  `/identity_preimage/identity_projection/1/body/members` (two of them).
- `package_id` recomputation over the fixture's `identity_preimage` gives
  `3a32d747...03df`, the base's own id. With
  `identity_preimage.dependency_selections` replaced by the file's entries it
  gives `0606043a...abf2`, the recorded value.
- IR-495 shows Done in Linear. Its code has not landed:
  `crates/quire-contract-model/Cargo.toml` still depends on `stacker` and
  `serde_stacker`, and `binding.rs:204` and `conformance.rs:826` still use
  them.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-112 applies each mutation "at its `pointer` with its `replacement`" and reads the result, but it does not say whether the harness then refreshes the identities: the mutated node's `node_id`, the references to it, `identity_preimage.identity_projection` and `package_id`. FR-038 runs the `package_id` and projection checks before the grammar and graph checks. Applied literally, all five body-grammar mutations refuse at identity checks (stale node key, `dependencies`, `stale_dependency` at the projection), never at the grammar. So even after IR-495 lands, the five IR-495 entries would keep failing for a harness reason. The expected-failure list would then hide a harness defect as the reader's. State the refresh, or state the level at which the outcome is compared | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2147 |
| FND-002 | high | AC-113 recomputes `package_id` "with the lock's `dependency_selections` replaced". `package_id` is computed over `identity_preimage` (FR-038 line 433), and the lock's members must equal the preimage's. Replacing only the lock leaves the identity at the base's `3a32d747...`, never the recorded `0606043a...`, so the AC as written cannot pass. QSpec's README and its `with_dependency_selections` replace the entries in both `lock` and `identity_preimage` | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2148, 433 |
| FND-003 | medium | AC-113 says the harness "recomputes the `package_id`" but does not say with what. A harness that hashes the JSON itself with `sha256` and JCS passes without running any code in this repository, so the AC does not test the reader's identity derivation. Name the production path: the reader's `CheckedPackageIdentityPreimageV2` encode through `quire-canonical`, or a read of the substituted package that reaches the identity check | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2148 |
| FND-004 | medium | The AC-112 expected-failure list can still go vacuous. Nothing says that a listed id absent from `adverse.json`, for example one renamed or removed upstream, fails the run. A harness that loops over the file's mutations and checks each against the list never sees an orphan entry. Nor is "still fails as listed" defined: it is unclear whether an entry records the observed refusal, so that a change from one wrong outcome to another fails the run | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2147 |
| FND-005 | medium | AC-112 requires "the open ticket that owns the missing refusal (IR-495 for the five `body_grammar_mutations`)". IR-495 is Done in Linear, but its reader change has not landed (stacker is still present, and the probe shows no `malformed_wire`). The AC does not assume that IR-495 landed, but it builds a tracker state into a requirement, and that state is false at review time. Reopen IR-495 or move the work to an open ticket, and keep the ticket id in the matrix, not in the AC text | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2147 |
| FND-006 | low | AC-112 ignores each body-grammar mutation's `flattened` form. QSpec TC-427 BG-02 uses it as the positive control: the same meaning, written with a reference, passes. Without it, a harness whose substitution breaks the package in some other way still sees a refusal for every mutation | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2147 |

## Verdict

Changes needed. The two highs block merge. FND-001: AC-112 must say how
identities are made consistent after a mutation, or the IR-495 entries can
never clear. FND-002: AC-113 must replace `dependency_selections` in both
`lock` and `identity_preimage`. The fail-closed list and the stale-entry rule
are otherwise sound. Both ACs read from `QUIRE_SPECIFICATION_DIR` and copy
nothing.

## New findings (disposition pass 1)

Reviewed at agent-ix/quire-contract-ir@cbcdd758c34ce335b39f4684cf20dff9ca32cd38, against QSpec origin/main 396493c4. AC-112 now states that the harness refreshes no identity, and that a mutation refused at an identity check instead of its recorded code fails. That deliberately departs from FND-001's suggestion. On the merits it matches merged QSpec text. FR-322 "Identity and validation" says "A strict reader first validates the `contract_version` document-type tag, closed wire shape and canonical bytes, then recomputes `package_id` from `identity_preimage` and each node key from its preimage before graph admission". FR-322-AC-40 makes a body outside the grammar fail the V2 schema and refuse `malformed_wire`. QSpec TC-427 BG-02 and `tests/checked_package_v2.rs` apply the mutations unrefreshed. So a conformant reader refuses an unrefreshed body-grammar mutation `malformed_wire`. The identity-first refusals the round-0 probe measured are an IR reader defect, of order as well as of grammar, and AC-112 states the conformant outcome. AC-113 re-verified: with `dependency_selections` replaced in both `lock` and `identity_preimage`, the preimage hashes to `0606043a...abf2`, the recorded value; the unchanged base hashes to `3a32d747...03df`. `CheckedPackageIdentityPreimageV2` is `pub`, re-exported and `Deserialize`, so the named derivation can be called from the integration harness.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | high | FR-038 still says "The order of checks is: strict syntax, duplicate member and depth; canonical bytes; closed-schema decode and header; `package_id` recomputation; then the grammar and graph checks". That puts the body grammar after the identity recomputation. AC-112 now requires the opposite, as merged FR-322 does: the body grammar is checked at strict wire validation, ahead of every identity check. The new deferral paragraph hands IR-495 only "the nested-application readings below", not this ordering sentence. A reader built to FR-038's stated order, including an IR-495 that refuses nested terms in the grammar pass, still refuses `aggregate-in-group-members` and `binding-as-body-root` `stale_dependency` at the identity projection, and `application-in-application-arguments` `stale-node-key`. Those expected-failure entries then never clear, and the FR contradicts its own AC. Amend the order sentence so the body grammar is part of strict wire validation ahead of the `package_id` and node-key recomputation, or name that sentence explicitly as IR-495's to change | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:543-545, 1361-1367, 2160 |

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@cbcdd758c34ce335b39f4684cf20dff9ca32cd38.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | cbcdd75 |
| FND-002 | fixed | cbcdd75 |
| FND-003 | fixed | cbcdd75 |
| FND-004 | fixed | cbcdd75 |
| FND-005 | fixed | cbcdd75 |
| FND-006 | fixed | cbcdd75 |

Round 2, reviewed at agent-ix/quire-contract-ir@f741471d2516e50d21883aaa195d34d7b3668bac. The delta against cbcdd75 is one hunk, FR-038's order-of-checks sentence, and nothing else moved. The sentence now puts the body grammar in strict wire validation, ahead of the `package_id` and node-key recomputation. It cites merged FR-322 "Identity and validation", "Body grammar" and FR-322-AC-40, and names IR-495's code as the change that moves the reader's check. No other FR-038 or TC-048 sentence contradicts the new order:
- The canonical-bytes paragraph (lines 516-522) and the negative-bound term-walk sentences (lines 1018-1036 and 1203-1212) place the term walk after the decode and before the graph identity checks.
- The nested-application deviation (lines 1606-1616) falls under the "IR-495's to reconcile" paragraph.
- TC-048 line 251 agrees.

Clarification of FND-007's rationale, which leaves the original row unchanged: in the reader's code, `validate` recomputes `package_id` over `identity_preimage`, which a body mutation does not change. `validate_graph` then runs the term walk (`validate_body`, mod.rs:1725) before the `dependencies` checks and the identity-projection comparison (mod.rs:1857). So the identity refusals the round-0 probe saw come from the term walk admitting nested terms, not from the order alone. Once IR-495 makes the term walk refuse nested terms, those refusals come first. The "reader as it stands" sentence is accurate: the term walk does run after the `package_id` recomputation. `make spec`: validate passes, grammar 1 (FR-014), strict 23 unbacked.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-007 | fixed | f741471 |
