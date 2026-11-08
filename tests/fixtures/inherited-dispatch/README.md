# Inherited-dispatch fixtures

`d10-package.json` is the package QSL emitted for the D10 case (agent-ix/quire-contract-ir#950),
converted to the FR-322 operand-slot wire form by `wire_operands`
(`tests/it/support/checked_package.rs`): every `{"node": i}` index segment became an
`{"operand": k}` slot, numbered by first occurrence, with the node's `operand_rows` entry. The
conversion is deterministic and makes no semantic edit; resolving the slots with `resolve_operands`
reproduces the emitted index-form package exactly (checked when it was converted, and the slot form
is re-derived by `converted_d10_fixture_resolves_to_the_admitted_index_form_graph`).

Replace the file with a fresh QSL emission once QSL emits `operand_rows` (QSL-1675).
