# Genuine model enum projection fixture

SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (C) 2026 Agent-IX

These first-party test artifacts were exported on 2026-11-07 from
`quire-spec-language`'s
`qsl-analyze/tests/it/monitor_root.rs` test
`budgeted_converted_model_enum_shapes_are_joined_by_the_native_monitor_closure`.
The source fixture uses the existing `ClauseSource` renderer with a genuine
`Inputs.tier: M::Tier` field and a Boolean atom. The unrelated authored enum
helper remains unreachable. `monitor.native` is the compiler's actual input;
`domain.json` is the supplied `example/config-version` domain evidence;
`checked.json` is the unmodified final emission.

`producer-rows.json` records the original results of
`selected_model_enum_declaration_rows`, `selected_model_enum_member_rows` and
the budgeted `selected_ir_model_enum_shape_rows`: one declaration, two cases,
and one complete shape. NodeKey objects come from the producer's existing
`node_wire` serialization. Shape ordering and variant owners/cases come from
the original Exact getters. The owner test decodes these original component
values; it does not derive opaque identities from checked structural nodes.
No source-owner association or witness was invented for this fixture.

The export run strictly admitted the package and checked that its full,
nonempty `closure.model_enums()` equaled the producer's shape rows, before
unfixed direct projection refused `UnsupportedExactType` at
`model/variant_type`. The regression here uses strict admission and the same
native declaration/member/shape checks and private cache installer as that
closure, then value-asserts direct projection under cumulative exact and
one-short budgets. It qualifies the owner projection boundary; it does not
replace the live QSL same-emission closure test.

These artifacts are licensed under this repository's AGPL-3.0-or-later license.
