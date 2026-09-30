---
id: SR-626
title: "gap analysis of PR 232 (make ci green by backing FR-344 and removing unbuilt requirements)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@122c21f6cacb25222b35b6629d19ed90e541d81a; spec/**, tests/it/checked_package_v2_adr002_members.rs, src/lib.rs, src/kani/, crates/quire-contract-model/src/lib.rs"
review_set: subset
---
# SR-626: gap analysis of PR 232

## Summary

Ticket: IR-448. Planless gap analysis. Plan completion: not assessed.

The gates were measured, not taken from the PR:

- `make ci` on the PR head exits 0.
- `make spec` on origin/main 70792e7 exits 1, with "22 unbacked row(s) and 0 contradicted status(es)". Because `ci` depends on `spec`, `make ci` on main fails.

The 22 rows are FR-036 (×5 ACs, plus the matrix row), FR-037-AC-6, FR-039 (×4 ACs, plus the matrix row), FR-019-AC-5, FR-344 (×3 ACs, plus the matrix row), TC-045, TC-055, TC-058 and TC-222.

quire offers no planned-row exemption:

- The status classes are `complete`, `pending`, `failed` and `retired`, and `--strict` fires on any non-empty `unbacked_rows` (quire-rs FR-050).
- Measured on main: changing FR-036's matrix status to ⛔ (retired) still leaves FR-036 and its ACs unbacked.
- `--severity coverage:unbacked-row=warning` still exits 1 under `--strict`.
- `no_source_symbol` (Inspection, Analysis) withdraws only the status-lie verdict. The row stays unbacked.
- `exclude:` is module-level only; the repo has no way to declare it.

So a spec that keeps these rows cannot be green under `--strict`. The coder's claim holds.

The facts behind the removals were checked against the code:

- `src/lib.rs:12` has `pub use quire_contract_model::*`.
- The model crate root has seven glob re-exports.
- `src/kani/replay.rs`, `witness.rs`, `arithmetic.rs`, `collections.rs` and `objects.rs` all exist.

FR-039, FR-019-AC-5 and FR-037-AC-6 are therefore false today, and no test could honestly back them.

FR-036 (negotiation) is implemented and owned in agent-ix/quire-contract-codegen (its FR-019 `negotiate_*` settlement and FR-015 records).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Deleting FR-039 and FR-019-AC-5 removes the only testable form of the IR-313 owner rulings (one import path, no model globs, lowerings to codegen) that IR-347 is to implement; no ticket or remaining AC carries them, and the no-glob rule is gone from FR-019 | spec/interface/FR-019-rust-library-interface.md:53 |
| FND-002 | low | IR-448 also lists six `tag-on-non-binding-symbol` warnings; they are still emitted at head, so the PR does not close everything the ticket names | crates/quire-contract-model/src/checked_package/common.rs:569 |

## Finding Detail

- FND-001: IR-347 (Backlog, "Refactor IR to its layout AD (globs, compatibility bridge, witness/replay deletion...)") has a one-paragraph description with no acceptance criteria. The deleted FR-039-AC-1 to AC-4 and FR-019-AC-5 were those criteria: an inventory equal to a named item table, a compile-fail probe per QSL-owned and codegen-owned item, and no glob in the model root.

  After this PR, the rules survive only as untested prose in AD-001 and FR-028. FR-019 now reads "The crate root re-exports them from their modules". That drops the no-glob rule that AD-001:42-43 and :155-156 still require.

  This is a real requirement deleted, not ceremony. Removing it is the right way to make `--strict` green, but the owner (Peter) should approve it. Before merge, the deleted AC text should be carried into IR-347 (or a child ticket) so the implementing PR can re-add the FR with its tests. The "Items QSL owns" and "Items codegen owns" lists from FR-039 are useful to carry as well.
- FND-002: `make ci` at head prints the six `[tag-on-non-binding-symbol]` warnings, at common.rs:569, dependency_references.rs:324, lower.rs:319, v2/mod.rs:788, operations.rs:691 and tests/it/checked_package_v2_dependency_reference.rs:153. They do not fail the gate. Either fix them in this PR or state in IR-448 that they are split out. IR-448's NFR-001 uncatalogued-method bullet is also still printed as a warning.

## Scope

- FR-344-AC-1, spec/contract/FR-344-admit-or-refuse-the-adr-002-2-0-0-members.md, examined: "A `quire.checked-package/v2` document whose semantic graph carries a node attempting to represent a supertype list, an abstractness flag, a subsets edge or a redefines edge, via any `node_tag` outside `CheckedNodeTag::ALL` or any `semantic_form` outside that tag's closed form enum, refuses ..." Backed by a real oracle (SR-625).
- FR-344-AC-2, examined: "A `relation`/`population` node whose `body` carries content satisfying no branch of the closed `SemanticTerm` grammar refuses `invalid_semantic_graph` ..." Backed by a real oracle.
- FR-344-AC-3, examined: "... refuses `unknown_member` at the pointer of the extra member ..." Backed, but with a weak locus (SR-625 FND-001).
- FR-036 (deleted), examined: its negotiation is owned by codegen FR-019 and FR-015. Removal justified; cross-repo links are in SR-628.
- FR-037-AC-6 (deleted), examined: "Contract IR's public API names no replay envelope ... the root crate has no `replay` or `witness` module" is false today (src/kani/replay.rs). IR-327 separately asks to retire witness/replay from the spec. Removal justified.
- FR-039-AC-1 to AC-4 (deleted), examined (FND-001).
- FR-019-AC-5 (deleted), examined (FND-001).
- TC-045, TC-055 and TC-058 (deleted), examined: they follow their FRs.
- `make ci` at head and `make spec` at origin/main, examined: exit 0 and exit 1 respectively.
- Pins, digests and tracking records in the diff, examined: none introduced.

## Verdict

The approach is correct. The 22 rows cannot be kept green under `quire --strict`, and each deleted AC is false today. FR-036 already lives in codegen.

FND-001 is a requirement-loss risk, not a defect in the diff. It needs the owner's sign-off, and the deleted AC text should be carried into IR-347, before merge.
