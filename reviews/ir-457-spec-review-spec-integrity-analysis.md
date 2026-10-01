---
id: SR-636
title: "spec integrity review of PR 235 (FR-038 and TC-048 staleness edits)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@b50cd28fd9debf730c4e3f72d9fb2abe6c17e7ce; spec/contract/FR-038-consume-checked-package-v2.md, spec/contract/TC-048-checked-package-v2-strict-reader.md"
review_set: base
---
# SR-636: spec integrity review of PR 235

## Summary

Ticket: IR-457. Spec review (integrity sub-analysis: consistency, completeness, atomicity) of the FR-038 and TC-048 edits. Edited: FR-038 Inputs, the reader paragraph, class 2 of the `model_selections` total order, the `dependency_selections` binding paragraph, FR-038-AC-2, AC-27 and AC-37 (clauses dropped, no AC or row removed), and the TC-048 model-owned case line. `quire validate` over spec/plan/reviews: 238/238 docs grammar-clean, 0 findings. `make spec` exits 1 on both main and head with the same 17 unbacked rows (pre-existing).

Checked consistent: Inputs, reader paragraph and AC-27 no longer mention raw-artifact digests; the binding paragraph and AC-37 state `package_id` as the only dependency binding and match the code (`dependency_references.rs:111`); class 2's rewritten rationale matches class 5 and the code (v2/mod.rs:1105-1116); TC-048's new "no supplied document as `missing_import`" matches FR-038 class 5 and `admit_document`. No STD-001 row names `revision-mismatch` or a raw-artifact refusal, so the registry needs no change. FR-038-AC-4's "editing ... raw source digest ... leaves it unchanged" is about package-id coverage, not staleness, and remains true.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-038 still uses the removed "evidence attests a digest" vocabulary for domain package selections: the reader paragraph says "an entry the evidence does not attest" / "an entry whose digest the evidence does not attest" (lines 194-204), and FR-038-AC-20 says "independently attested by the package evidence" and "a digest the evidence does not attest" (line 660). After this PR the evidence attests no digests; it supplies documents under digests, and the reader recomputes them. Reword to "no document supplied under its digest" so the text matches the class-5 wording the PR introduced. | spec/contract/FR-038-consume-checked-package-v2.md:660 |

## Verdict

The edits are consistent with the code and with each other, with no AC or row removed. One low wording leftover (FND-001). Mergeable.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | FR-038-AC-19 still says "an entry of empty `identity` or `version` beside an entry whose digest the evidence does not attest refuses as `malformed_wire`". The fix reworded the reader paragraph, AC-11 and AC-20 but missed this sibling, which uses the same removed attestation vocabulary. | spec/contract/FR-038-consume-checked-package-v2.md:660 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 32e3371: the reader paragraph (lines 194-205), AC-11 and AC-20 now speak of documents the evidence supplies rather than digests it attests |
