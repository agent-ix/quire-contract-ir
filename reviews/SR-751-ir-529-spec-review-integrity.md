---
id: SR-751
title: "integrity review of PR 251 (IR-529 artifact references)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@0b106a64ea261a86c0f2f64eb9ab8f31f546d39d; spec/ tree grep for revision, digest_domain, export, CheckedRevision, raw-artifact wording; FR-038 (AC-2, AC-4, AC-10, AC-19, AC-20, AC-21, AC-27, AC-31, AC-32, AC-46..64); FR-019 Public items; FR-035; AD-004; spec/checked_package/matrix/tests.md; spec/model/matrix/tests.md"
review_set: subset
---
# SR-751: integrity review of PR 251

## Summary

Ticket: IR-529. I grepped the whole `spec/` tree for wording that still contradicts
the amendment. Most hits are other revision concepts and are clean:

- FR-011, FR-012, FR-017, FR-023, STD-001 and STD-003 are about requirement and source
  revisions.
- FR-029 to FR-031 are about the Kani profile revision.

The amendment leaves these clean:

- FR-038-AC-10 and AC-19 to AC-21 (model_selections).
- AC-27 (domain package evidence).
- AC-31 and AC-32 (dependency selections; a `DefinitionRef` member is still one that
  does not belong).
- FR-038 line 262 (owner join by authority and identity).
- FR-040.

No AC was edited or removed. The findings below are what remains inconsistent.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-2 still says "a lock reference or owner carrying `authority`, `revision` or `export` refuses as `unknown_member`". Read literally, that contradicts FR-038-AC-46: every `lock.definition_selections` row and lock `Selection.definition` is a lock reference that carries `authority` and admits. The PR body says the clause is about domain package selections, but the AC text does not say so. Qualify it ("a domain package selection or model owner"). This narrows wording, not meaning. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:798 |
| FND-002 | low | AD-004's evidence row now says "only `lock.sources` rows carry a byte digest". `lock.model_selections` rows carry a `sha256-jcs` digest, and dependency rows carry a `package_id` digest. The same row says the former is checked against evidence. Say "only `lock.sources` rows carry a raw-source byte digest". | spec/assurance/AD-004-checked-package-seam.md:50 |
| FND-003 | low | FR-019's Public items table now drops `CheckedRevision` and adds `CheckedSourceRef`. The code still exports the former and has no latter until IR-530. FR-019-AC-5 is already planned, but the FR-019 row in spec/model/matrix/tests.md does not record that the `checked_package` row now runs ahead of the code. | spec/model/functional/FR-019-rust-library-interface.md:136; spec/model/matrix/tests.md:17 |
| FND-004 | low | The amendment does not state its effect on FR-035 output. The lowered `ContractPackage` embeds `CheckedSourceMapEntry`, whose regions carry `source`. Dropping `revision` from source rows therefore changes the ContractPackage canonical bytes (FR-035-AC-5), and the FR-035 row is ✅. One sentence in "Artifact references" or the FR-035 row would say so. | crates/quire-contract-model/src/checked_package/v2/lower.rs:322-328; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md |

## Verdict

Mostly consistent. The requirement change is explicit, and the PR leaves no stale
revision or digest wording about definition references. FND-001 is a real literal
contradiction with an easy fix. The rest are wording nits.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | FR-019-AC-5 now has process commentary inside the criterion: "IR-529 extends this criterion rather than adding one, the API-shape claims it previously held as FR-038-AC-60 through AC-62". Those ids existed only on this unmerged branch. After renumbering, FR-038-AC-60 and AC-61 name other ACs (package_id re-derivation and the nominal owner join), so the reference misleads. Drop the parenthetical, or keep only "(FR-038 Artifact references)". | spec/model/functional/FR-019-rust-library-interface.md:152 |

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f (delta from 0b106a64ea261a86c0f2f64eb9ab8f31f546d39d; base origin/main e80ea70ab8874676de47ac1b7fb439497ced7991 unchanged).

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f |
| FND-002 | fixed | 9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f |
| FND-003 | fixed | 9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f |
| FND-004 | fixed | 9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f |

Round 2, reviewed at agent-ix/quire-contract-ir@a4811eab16f0f89208c20ca172291d31344d8f0c (delta from 9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f; base unchanged).

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-005 | fixed | a4811eab16f0f89208c20ca172291d31344d8f0c |
