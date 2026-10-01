---
id: TC-055
title: "The root crate's public interface is exactly the listed items and names no QSL-owned replay type"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-039
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-037
    type: verifies
---
# TC-055: The root crate's public interface is exactly the listed items and names no QSL-owned replay type

## Description

Verify FR-039-AC-1 through FR-039-AC-4 and FR-037-AC-6: the `quire-contract-ir` root crate's
public items match FR-039's table and include no `quire_contract_model`
item, it has no dependency on `quire_spec_language::runtime`, the items QSL
and codegen own are absent, and its error surface is closed and panic-free.

## Test Procedure

Inventory every `pub` item reachable from `src/lib.rs` and compare the set
with FR-039's table; read `src/lib.rs` for any `pub use` of
`quire_contract_model`, and compile one probe naming a model item through
`quire_contract_ir` and expect it to fail.
Search `src/` and `crates/quire-contract-model/src/` for
`quire_spec_language::runtime`, `qsl_replay::replay` and any other executor
entry, and for a public replay envelope, request, result, parity or
minimization type. Confirm `src/kani/` has no `replay`, `witness`, `arithmetic`, `collections`
or `objects` module. Compile one probe per item in FR-039's "Items QSL owns"
and "Items codegen owns" sections through `quire_contract_ir` and expect each
to fail, and search `src/` for any function from a `KaniOutcome` to a QSL
`TerminalValue`: that map is owned by `agent-ix/quire-contract-codegen`
(tracked there under Linear IR-358) and must be absent. Run the TC-041 and
TC-042 negative corpora under `catch_unwind`.

## Expected Results

The inventory equals the table with no extra or missing item and no model
item; the search finds nothing; the model-item probe and every QSL-owned and
codegen-owned item probe fail to compile and no `TerminalValue` map is found
under `src/`; no negative case panics.

## Status

Planned.
