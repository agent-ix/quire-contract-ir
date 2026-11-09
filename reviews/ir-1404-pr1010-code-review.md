---
id: SR-20261109095521
title: "IR-1404 PR 1010 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2, tests/it/cycle_free_model.rs, tests/it/public_interface.rs at a12ad8b91"
---

# IR-1404 PR 1010 code and Rust review

## Summary

Ticket: IR-1404. PR agent-ix/quire-contract-ir#1010, head a12ad8b91, diff origin/main...HEAD.
Code review and the Rust lane in one artifact. Reviewed the refinement refusals, the reader
pointer change, wire_fault, the temporal fixes, node_map order, the lossless rename, the
narrowed callback scan and the string_edge markers. No cargo runs; every finding is traced
through the source.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | The lossless rename hides a reachable input depth ceiling. selection_key's recursive owner_effective_type is compared with no skip, so a chain about 124 owners deep exhausts PATH_SLOTS and refuses malformed_wire instead of the later unsupported_construct, so depth decides the outcome (FR-038-AC-117). The new comment is false. | crates/quire-contract-model/src/checked_package/v2/lossless.rs:17-22 | correct-requirement-no-evidence |
| FND-002 | high | The narrowed callback scan no longer flags an admission callback whose verdict is a custom type: impl Fn(&[u8]) -> Verdict, or an out-parameter verdict, passes. No adverse control covers the new rule; the exact-signature allow-list already used in this file would suffice. | tests/it/cycle_free_model.rs:715-782 | implementation-bug-despite-evidence |
| FND-003 | medium | Refinement shape and member refuse an absent member at the member's own pointer, which does not resolve; FR-038 requires the object that lacks it. | crates/quire-contract-model/src/checked_package/v2/term_view.rs:1040, crates/quire-contract-model/src/checked_package/v2/term_view.rs:1055 | correct-requirement-no-evidence |
| FND-004 | medium | wire_fault reports a non-object leaf or a leaf with no path at leaves/N/path, a pointer that does not resolve; the correct locus is the leaf. | crates/quire-contract-model/src/checked_package/v2/typed_terms.rs:437-440 | correct-requirement-no-evidence |
| FND-005 | medium | An absent name still refuses at /name, while the same rule just moved absent contract_version to the document. FR-038 requires the empty pointer here too. | crates/quire-contract-model/src/checked_package/v2/mod.rs:1542-1549, crates/quire-contract-model/src/checked_package/v2/budgeted_read.rs:737 | implementation-bug-despite-evidence |
| FND-006 | low | wire_fault's doc says a wrong-kind value is named by the object holding it; the code and FR-038 name the value. | crates/quire-contract-model/src/checked_package/v2/typed_terms.rs:334-338 | implementation-bug-despite-evidence |
| FND-007 | low | refinement_fault's doc says a healthy body never reaches these walks; header and catalog run on every refinement body. | crates/quire-contract-model/src/checked_package/v2/term_view.rs:1015-1017, crates/quire-contract-model/src/checked_package/v2/refinement.rs:1164-1167 | implementation-bug-despite-evidence |

## Verdict

Not mergeable until FND-001 and FND-002 are fixed. The refinement first-defect walk matches
the decoder's order (term_view.rs:983-1010) and FR-061-AC-2/5/11, including the catalog closed
wire before rows. The contract_version change, the AC-122 interval rule, the AC-530 raw binding
name lookup and the map_node_ids document order are correct. The string_edge markers are on
genuine closed-string intake functions, as FR-056-AC-3 expects.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-008 | medium | Saturating push removes the only stop on Compare's native recursion. A deep owner_effective_type chain now recurses once per level through Children, each frame holding a 64-slot seen array; the new test needs a 64 MiB stack at depth 200. A chain that used to refuse at 128 levels can now overflow the stack and abort the process. | crates/quire-contract-model/src/checked_package/v2/lossless.rs:73-79, crates/quire-contract-model/src/checked_package/v2/lossless.rs:1279-1286 | implementation-bug-despite-evidence |
| FND-009 | low | The FnOnce() -> R allow-list accepts any one-letter return name without checking it is an unbounded generic, so a niladic trust gate such as FnOnce() -> V, or R: Into<bool>, passes. | tests/it/cycle_free_model.rs:723-729 | implementation-bug-despite-evidence |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified at 2ac175217: push always succeeds, so depth no longer changes the outcome |
| FND-002 | fixed | Verified at 2ac175217: exact-shape allow-list with custom-verdict controls |
| FND-003 | fixed | Verified at 2ac175217 |
| FND-004 | fixed | Verified at 2ac175217 |
| FND-005 | fixed | Verified at 2ac175217 |
| FND-006 | fixed | Verified at 2ac175217 |
| FND-007 | fixed | Verified at 2ac175217 |
| FND-008 | still-open | New this round: Compare recursion is unbounded in chain depth |
| FND-009 | still-open | New this round: plumbing_callback does not confirm R is an unbounded generic |
