---
id: TC-055
title: "The root crate's public interface is exactly the listed items and names no QSL-owned replay type"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-039
    type: verifies
---
# TC-055: The root crate's public interface is exactly the listed items and names no QSL-owned replay type

## Description

Verify FR-039-AC-1 through FR-039-AC-4: the `quire-contract-ir` root crate's
public items outside the model re-export match FR-039's table, it has no
dependency on `quire_spec_language::runtime`, the items QSL owns are absent,
and its error surface is closed and panic-free.

## Test Procedure

Inventory every `pub` item reachable from `src/lib.rs` outside the
`quire_contract_model` re-export and compare the set with FR-039's table.
Search `src/` for `quire_spec_language::runtime`. Compile one probe per item in
FR-039's "Items QSL owns" section through `quire_contract_ir` and expect each
to fail; compile a probe that binds the `kani` outcome map's result to
`qsl_replay::TerminalValue` and expect it to build. Read
`BridgeErrorCode::all()` against the STD-001 registry, and run the TC-038
through TC-042 negative corpora under `catch_unwind`.

## Expected Results

The inventory equals the table with no extra or missing item; the search finds
nothing; every QSL-owned-item probe fails to compile and the `TerminalValue`
probe builds; every `BridgeErrorCode` appears once and is registered; no
negative case panics.

## Status

Planned.
