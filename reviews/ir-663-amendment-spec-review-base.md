---
id: SR-2940
title: "Base checklist review of the IR-663 private intake retention amendment"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir PR 321 (IR-663 amendment, specification only); spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

## Summary

Ticket: IR-663. Base checklist over the two-file amendment that narrows the
FR-038 declaration-refusal retention contract from the public
`CheckedPackageRefusal` to the private `SelectionRefusal` returned by
`admit_selection`, and rewrites FR-038-AC-167/173/174/175 and the TC-048
retention procedures in place. The amendment itself is well-formed and its
claims about the current code hold, but the narrowing is incomplete: two
unchanged FR-038 passages still contradict it (one still promises the
retention in the public read-result Outputs).

Sources measured independently: QSpec FR-154 (merged text, declaration-refusal
retention paragraph); FCD `schema/semantic/v1/common.schema.json` `$defs.origin`,
`sourceLocus`, `generatedOrigin`; IR code `SelectionRefusal`, `SelectionFailure`,
`From<ModelFailure> for SelectionFailure`, `admit_document`, `admit_selection`
and `read_semantic_ir` in `model_members.rs`; the `SelectionFailure::Refused`
conversion in `v2/mod.rs`; `ValidationFailure::refused_because` in `common.rs`;
`CheckedPackageRefusal` in `shared.rs`; QSpec FR-322 (no origin retention on
`CheckedPackageRefusal`).

## Examined units

- FR-038 retention section "Declaration-refusal identity and origin retention" (examined)
- FR-038 model-declaration paragraph beginning "These declaration checks run in step 1" (examined)
- FR-038 operation-step paragraph ("selection-row pointer; private intake declaration metadata is governed above") (examined)
- FR-038 Outputs list, first bullet (examined; unchanged by the PR)
- FR-038 relationship schema paragraph ("The refusal does not retain FCD origin ...") (examined; unchanged by the PR)
- FR-038-AC-167, FR-038-AC-173, FR-038-AC-174, FR-038-AC-175 (examined)
- FR-038-AC-168, FR-038-AC-183, FR-038-AC-184, FR-038-AC-185 (context_only)
- TC-048 relationship procedure steps 3, 4 and 9 and their Expected paragraph (examined)
- TC-048 "Planned private intake refusal-origin checks" steps 1 to 6 (examined)
- TC-048 "Retained expected node key" section (context_only)

## Verified clean

- Claims about today's code are accurate: the `SelectionFailure::Refused` arm
  builds `ValidationFailure::refused_because(code, path, cause)`, which leaves
  `locus`, `contract_version`, `document_pointer` and the expected node id
  absent; `SelectionRefusal` today holds only `refusal` and `member`;
  `Limit` and `InexactNumber` are distinct variants; the document is released
  by `drop_value` inside `admit_selection`.
- The Source/Generated member sets agree with FCD `sourceLocus` and
  `generatedOrigin`; positivity and optional ends agree with FCD minimums.
- No new ids; the computed matrix keeps all 336 document/criterion identities
  with identical statuses and binders; only the statements of AC-167, 173, 174
  and 175 change. Strict matrix exits 1 at both base and head (31 untagged,
  unchanged).
- No commit ids, local paths, conflict markers, compatibility layer or copied
  foreign schema in the added lines; `git diff --check` is clean.
- No contradiction with the merged IR-680 expected-node-key criteria
  (AC-183..185) or the IR-651 and IR-654 criteria.
- `quire validate` on each changed document (quire 0.36.1, engine 0.50.1) exits
  0 with module-registry notices only (DuplicateArchetype x5,
  DuplicateInverseEdge, semantic.inline-data-schema); none concern these
  documents.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038 Outputs still promises public retention: the read-result bullet says "Located model-declaration refusals additionally retain authentic declaration identity and valid typed origin under the PLANNED / UNRUN retention contract below", which contradicts the amended contract ("Public reader metadata observability is not claimed") and AC-175's inspection that the public conversion stays code/path/cause-only. A consumer reading Outputs expects metadata on the public refusal. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:96-98 |
| FND-002 | medium | Unqualified "The refusal does not retain FCD origin, sourceIdentity, path, span or artifact metadata" for relationship non-end member refusals contradicts the new private contract, under which the private intake refusal of a relationship refused for, say, a bad `direction` with a valid origin SHALL retain that origin (AC-173, AC-174). The sentence should be scoped to the public refusal or removed. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1037-1039 |
| FND-003 | low | The amendment restates FCD's origin member names ("map directly to FCD's sourceIdentity, startLine, ... inputIdentities") and positivity rules alongside the `$defs.origin` link. This duplicates the owning schema and can drift from it; the link and the IR-owned Rust shape are enough. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1170-1176 |
| FND-004 | low | AC-175 (and TC-048 step 6) inspects that no "handoff" field is added, but no driver handoff type exists in this repository, so the inspection cannot be performed here; scope it to IR-owned surfaces (`CheckedPackageRefusal`, the dispatch result). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3807; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1290-1293 |

## Verdict

Not merge-ready: FND-001 is a live contradiction between the Outputs section
and the narrowed retention contract, and FND-002 is a second unchanged passage
that conflicts with it. Both are one-sentence fixes inside FR-038.

## Dispositions

Round 1, fix commit "spec: close IR663 private intake review findings".

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | fixed by commit "spec: close IR663 private intake review findings" |
| FND-002 | fixed | fixed by commit "spec: close IR663 private intake review findings" |
| FND-003 | fixed | fixed by commit "spec: close IR663 private intake review findings" |
| FND-004 | fixed | fixed by commit "spec: close IR663 private intake review findings" |
