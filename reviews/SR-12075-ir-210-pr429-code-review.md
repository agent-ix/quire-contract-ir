---
id: SR-12075
title: "IR-210 slice D code review (rust lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs, crates/quire-contract-model/src/checked_package/v2/model_members/tests/intake_retention.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

## Summary

Ticket: IR-210 (slice D), PR quire-contract-ir#429, over origin/main (the merge base). The PR adds a `#[trace(...)]` attribute to 17 tests: 10 in `model_members/tests.rs` and 7 in `model_members/tests/intake_retention.rs`. It also edits one doc line, removing FR-038-AC-189 from `/// Trace:` on `tc_048_intake_retains_actual_nested_declaration_not_owner_or_missing_target`. A diff filter over `git diff -U0 origin/main...HEAD` finds nothing else: every changed line except that doc line matches `^\+#\[trace\("TC-048"(, "FR-038-AC-[0-9]+")*\)\]$`. No test body, assertion or function name changed. The rust-review lane is folded into this file. Each id was read against its criterion text in `spec/checked_package/functional/FR-038-consume-checked-package-v2.md` and against the test body. Coverage accounting is in the gap-analysis artifact (SR-12076).

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-27 | examined | A domain package selection is admitted only by the document the evidence supplies under its digest: a document supplied under no digest of the row refuses `missing_import`/`missing-selection` at the row's `digest`;... a document naming another identity refuses `invalid_model_binding`/`wrong-model-selection` at the row's `identity`; and a matching document admits. |
| FR-038-AC-28 | examined | A domain package document's declarations refuse at the row, in FR-154's order:... a node failing two rows reports the earlier...; a `typeRef` naming a relationship refuses `invalid_model_binding`/`malformed-declaration` wherever its declaring node sorts; two nodes with no identity refuse `malformed-declaration`, never `conflicting-binding`. |
| FR-038-AC-29 | examined |... an inherited field resolves on a subtype and a subtype conforms to its supertype in either operand order; a field typed at an `Int[lo, hi]` value type has the integer-range member type `[lo, hi]`. |
| FR-038-AC-30 | examined | Reading a selected domain package and resolving a model-owned member are charged to the `work` limit: a limit one below the work a read used returns `incomplete` for `work` with the pointer `/lock/model_selections/<i>` of the row, and the exact work admits. |
| FR-038-AC-174 | examined |... the reader retains that exact FCD identity in the private refusal returned by `admit_selection`, including refused object-id spelling. Missing or non-string identity yields None... |
| FR-038-AC-175 | examined | Each private `SelectionRefusal` constructor... preserves the selected retained metadata record through the private `admit_selection` return after document release.... Pre-declaration constructors preserve genuine metadata absence. |
| FR-038-AC-186 | examined | Given a located refusal with a valid FCD Source origin, the reader returns its exact typed source member values and optional-member presence..., including all four end-coordinate presence combinations and the existing admitted 2^53 boundary. |
| FR-038-AC-187 | examined | Given a located refusal with a valid FCD Generated origin, the reader returns exact generator identity/version text and input identity sequence... Input order/repetition remains unchanged and no source span is present. |
| FR-038-AC-188 | examined | When an existing field..., operation..., parameter... or relationship-end check refuses, the reader retains the actual offending... supplied identity and valid origin. Missing nested metadata remains None; an enclosing owner or unresolved target cannot substitute. |
| FR-038-AC-189 | examined | Given a located refusal with missing or malformed origin, the reader returns None for origin while independently retaining any available declaration identity. Both-branch, partial-branch, unknown-member and invalid-member cases cannot yield a salvaged branch or default origin. |
| FR-038-AC-190 | examined | For every same-identity type group..., the reader retains its common supplied identity and retains origin only if every candidate has a valid origin and all typed values and optional-member presence agree.... Candidate permutation cannot select a winner. |
| FR-038-AC-191 | examined | Retaining metadata does not change the existing first code, cause or selection-row member in node/table/member-path order.... distinct Limit and InexactNumber alternatives retain their current payloads and precedence. |
| FR-038-AC-192 | examined | Retained metadata is acquired and released within the existing bounded document/read/work accounting, without a second parse, raised ceiling or unbounded diagnostic collection. Exact sufficient work succeeds; the first unpayable charge preserves the existing incomplete result and no partial package. |
| FR-038-AC-196 | examined | A relationship identity repeated sequentially, including under two different owners, retains the later actual offending relationship's supplied identity and its own valid origin... Its missing or malformed origin yields None.... type-group origin consensus does not apply. |
| TC-048 | context_only | CheckedPackage V2 strict reader re-derives package and nominal identities |
| model_members/tests.rs | examined | 10 added `#[trace("TC-048"...)]` attributes (AC-27, AC-28 x5, AC-29 x2, AC-30, TC-048 only on the byte-limit test) |
| model_members/tests/intake_retention.rs | examined | 7 added `#[trace("TC-048"...)]` attributes and one `/// Trace:` line with AC-189 removed |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

APPROVE. No findings. Mergeable on this method.

Per-binding judgement. Each binding was checked against the AC text above and the test body.

- **AC-27 on `tc_048_a_selection_admits_only_the_document_it_names`: correct.** The test asserts that a matching document admits, and it covers three refusals: wrong identity (`wrong-model-selection` at `identity`), an unknown digest (`missing-selection` at `digest`), and forged bytes (`content-mismatch` mismatch record). It does not cover the 2^53 and `package.version` legs, and the matrix already records AC-27 as partial. The attribute matches the doc line that was there before.
- **AC-28 on the five declaration tests: correct.**
  - `semantic_ir_declaration_defects` covers each FR-154 cause.
  - `reference_to_a_refused_node` shows the refused node's own refusal wins in both sort positions, including for a duplicate identity.
  - `one_nodes_failures_report_in_table_order` shows dangling before multiplicity, and malformed member before both.
  - `type_ref_to_a_relationship` covers both node orders.
  - `nodes_with_no_identity` shows malformed, not conflicting.
- **AC-29 on two tests: correct as complementary legs.** One test covers inherited-field resolution and conformance in both operand orders. The other covers `Int[lo,hi]` to `IntRange`. The `dispatch_call` legs are still backed by `tests/it/checked_package_v2_model_members.rs` (lines 1810 and 1984).
- **AC-30: correct.** The test asserts exact work admits, and one less gives `incomplete` at `/lock/model_selections/2` for the read and at `/1` for resolution.
- **AC-174, AC-175 and AC-186 on `intake_retains_source_values_after_authentic_document_release`: correct.**
  - The `bad-id` spelling is retained.
  - The document is dropped inside `selected` before any assertion.
  - All four end-coordinate presence combinations are checked.
  - `startLine` is 9007199254740992, which is 2^53.
- **AC-187: correct.** The test checks generator identity and version text, including case-differing build metadata. It checks repeated, ordered inputs in two permutations. The `Generated` variant has no span.
- **AC-174 and AC-188 on the nested-declaration test: correct.**
  - It covers field, operation, parameter, relationship, operation-return reset and list-reset contexts.
  - Missing nested metadata gives None and does not fall back to the owner.
  - A null or numeric identity gives None while the origin is kept, which is the AC-174 "non-string" leg.
- **AC-189 dropped from the nested-declaration test: the drop is right.** No case in that body has a missing or malformed origin while the declaration identity is still available and kept. Every None-origin case there also has a None identity (lines 214-221 and 279-294). The null/42 case is the reverse: identity None with the origin kept.
- **AC-189 and AC-190 on the group-origin test: correct.**
  - Lines 405-421 cover null, both-branch, partial-branch, missing-member, unknown-member and empty-input origins. Each asserts that the `bad-id` identity is kept and the origin is None.
  - The same-identity groups (conflicting and malformed) are run in both permutations against valid-equal, different, different-presence, missing and malformed candidates.
- **AC-192 kept on the group-origin test: acceptable.** This test only asserts `three_work > two_work`, meaning an extra missing-origin candidate is charged work. That is a real leg of "acquired... within the existing bounded... work accounting, without... unbounded diagnostic collection". The exact-sufficient and first-unpayable legs are asserted by the other AC-192 test, `tc_048_intake_metadata_work_and_original_first_refusal_are_exact`, which has exact independent charges, an `expected - 1` limit, and admitted and refused documents at `used` and `used - 1`. The two tests together cover the AC, and the tag here was already present as a doc line on main. This is not a defect.
- **AC-191 and AC-192 on `intake_metadata_work_and_original_first_refusal_are_exact`: correct.** Meaning is shown to precede Reference. The exact-work refusal equals the unlimited one, and `used - 1` gives the work limit at the row.
- **AC-175 and AC-191 on `intake_predeclaration_failures`: correct.** Missing-selection, content-mismatch and wrong-selection all have None metadata. `InexactNumber` precedes both the digest and the declaration checks for both selected digests.
- **AC-196: correct.** The later relationship's origin wins both within one owner and across two owners in node order. The earlier relationship's different valid origin is not combined into a None by consensus. A later relationship with no origin gives None.
- **TC-048-only on `tc_048_a_declaration_key_over_the_byte_limit_refuses_at_its_selection_row`: correct.** The test's own doc comment says it backs a FR-038 prose rule ("Every identity digest is computed through quire-canonical") that no numbered criterion states. FR-038-AC-90 is about `NominalIdentityPreimage::digest` limits, and AC-95 is about lowering, so neither fits. A TC-only tag is honest.

Rust lane:

- **Attribute spelling.** The form `#[trace("TC-048", "FR-038-AC-n"...)]` matches the crate's existing tagged tests, for example `#[trace("TC-048", "FR-038-AC-103")]` and `#[trace("TC-048", "FR-038-AC-89")]`. The convention is a TC id first and then the criterion ids, placed above `#[test]`, with the doc line listing the criterion ids.
- **Imports.** `intake_retention.rs` brings `trace` in through `use super::*;`, which re-exports the parent's `use ix_trace_rs::trace;`.
- **Doc lines.** Every attribute's criterion ids equal its `/// Trace:` doc-line ids.
- **Coverage of tests.** Every `#[test]` in both files now carries an attribute: 22 of 22 in `tests.rs` and 7 of 7 in `intake_retention.rs`.
- **Surface.** The diff has no panic, unsafe or integer-conversion surface.

Gates re-run on:

- `cargo fmt --all -- --check` is clean.
- `cargo clippy --locked -p quire-contract-model --all-targets -- -D warnings` is clean.
- `cargo test --locked -p quire-contract-model --lib model_members` passes 29 of 29.

No mutants were run. No body or assertion changed, so a mutant would only re-measure the strength of tests that already exist on main. It cannot tell us anything about these tags, which were judged by reading each assertion against the AC.
